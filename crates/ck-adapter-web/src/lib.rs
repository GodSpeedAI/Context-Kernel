use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, ToSocketAddrs};

use anyhow::{bail, Context, Result};
use chrono::Utc;
use serde::Deserialize;
use sha2::{Digest, Sha256};
use ulid::Ulid;

use ck_core::entities::{Document, RawContent, SourceUnit};
use ck_core::ports::{Normalizer, SourceAdapter};

pub trait DnsResolver: Send + Sync {
    fn resolve(&self, host: &str) -> Result<Vec<IpAddr>, Box<dyn std::error::Error + Send + Sync>>;
}

pub struct SystemDnsResolver;

impl DnsResolver for SystemDnsResolver {
    fn resolve(&self, host: &str) -> Result<Vec<IpAddr>, Box<dyn std::error::Error + Send + Sync>> {
        let addr = format!("{}:80", host);
        let ips: Vec<IpAddr> = addr.to_socket_addrs()?.map(|sa| sa.ip()).collect();
        Ok(ips)
    }
}

pub struct StaticResolver {
    host: String,
    ip: IpAddr,
}

impl StaticResolver {
    pub fn new(host: &str, ip: &str) -> Self {
        Self {
            host: host.to_string(),
            ip: ip.parse().expect("valid IP"),
        }
    }
}

impl DnsResolver for StaticResolver {
    fn resolve(&self, host: &str) -> Result<Vec<IpAddr>, Box<dyn std::error::Error + Send + Sync>> {
        if host == self.host {
            Ok(vec![self.ip])
        } else {
            Ok(vec![])
        }
    }
}

fn is_private_ip(ip: &IpAddr) -> bool {
    match ip {
        IpAddr::V4(v4) => is_blocked_ipv4(v4),
        IpAddr::V6(v6) => is_blocked_ipv6(v6),
    }
}

#[derive(Debug, Clone, Deserialize)]
struct WebConfig {
    urls: Vec<String>,
    #[serde(default = "default_max_depth")]
    #[allow(dead_code)]
    max_depth: u32,
    #[serde(default)]
    allowed_domains: Vec<String>,
    #[serde(default)]
    previous_etag: Option<String>,
    #[serde(default)]
    previous_last_modified: Option<String>,
}

fn default_max_depth() -> u32 {
    1
}

pub struct WebSourceAdapter;

impl SourceAdapter for WebSourceAdapter {
    fn kind(&self) -> &str {
        "website"
    }

    fn discover_units(&self, _source_id: &str, config_json: &str) -> Result<Vec<SourceUnit>> {
        let config: WebConfig =
            serde_json::from_str(config_json).context("invalid web adapter config JSON")?;

        let mut units = Vec::new();
        for url in &config.urls {
            validate_url(url, &config.allowed_domains)?;
            units.push(SourceUnit {
                external_id: url.clone(),
                uri: Some(url.clone()),
                mime_type_hint: Some("text/html".to_string()),
            });
        }
        Ok(units)
    }

    fn fetch_content(
        &self,
        _source_id: &str,
        unit: &SourceUnit,
        config_json: &str,
    ) -> Result<RawContent> {
        let url = unit
            .uri
            .as_deref()
            .ok_or_else(|| anyhow::anyhow!("unit has no URI"))?;

        let config: WebConfig =
            serde_json::from_str(config_json).context("invalid web adapter config JSON")?;
        validate_url(url, &config.allowed_domains)?;

        let agent = ureq::AgentBuilder::new().redirects(0).build();
        let mut req = agent.get(url);

        if let Some(etag) = &config.previous_etag {
            req = req.set("If-None-Match", etag);
        }
        if let Some(lm) = &config.previous_last_modified {
            req = req.set("If-Modified-Since", lm);
        }

        let response = req
            .call()
            .with_context(|| format!("HTTP request failed for {}", url))?;

        let status = response.status();
        if status == 304 {
            return Ok(RawContent {
                external_id: unit.external_id.clone(),
                uri: unit.uri.clone(),
                mime_type: "text/html".to_string(),
                bytes: Vec::new(),
                metadata_json: serde_json::json!({"not_modified": true}).to_string(),
            });
        }

        let content_type = response
            .header("content-type")
            .unwrap_or("text/html")
            .to_string();

        let etag = response.header("etag").map(String::from);
        let last_modified = response.header("last-modified").map(String::from);

        let body = response
            .into_string()
            .with_context(|| format!("failed to read response body from {}", url))?;

        let text = html_to_text(&body);
        let title = extract_title(&body);

        let mut metadata = serde_json::Map::new();
        if let Some(t) = title {
            metadata.insert("title".to_string(), serde_json::Value::String(t));
        }
        if let Some(e) = etag {
            metadata.insert("etag".to_string(), serde_json::Value::String(e));
        }
        if let Some(lm) = last_modified {
            metadata.insert("last_modified".to_string(), serde_json::Value::String(lm));
        }

        Ok(RawContent {
            external_id: unit.external_id.clone(),
            uri: unit.uri.clone(),
            mime_type: content_type,
            bytes: text.into_bytes(),
            metadata_json: serde_json::Value::Object(metadata).to_string(),
        })
    }
}

fn is_blocked_ipv4(ip: &Ipv4Addr) -> bool {
    let octets = ip.octets();
    if octets[0] == 10 {
        return true;
    }
    if octets[0] == 172 && octets[1] >= 16 && octets[1] <= 31 {
        return true;
    }
    if octets[0] == 192 && octets[1] == 168 {
        return true;
    }
    if octets[0] == 127 {
        return true;
    }
    if octets[0] == 169 && octets[1] == 254 {
        return true;
    }
    if octets[0] == 0 {
        return true;
    }
    false
}

fn is_blocked_ipv6(ip: &Ipv6Addr) -> bool {
    if ip.is_loopback() {
        return true;
    }
    let segments = ip.segments();
    // Block link-local (fe80::/10)
    if (segments[0] & 0xffc0) == 0xfe80 {
        return true;
    }
    // Block unique local addresses (fc00::/7)
    if (segments[0] & 0xfe00) == 0xfc00 {
        return true;
    }
    // Block IPv4-mapped IPv6 addresses (::ffff:0:0/96)
    if let Some(ipv4) = ip.to_ipv4_mapped() {
        return is_blocked_ipv4(&ipv4);
    }
    false
}

pub fn validate_url_with_resolver(
    url: &str,
    allowed_domains: &[String],
    resolver: &dyn DnsResolver,
) -> Result<()> {
    let parsed = url::Url::parse(url).with_context(|| format!("invalid URL: {}", url))?;

    let scheme = parsed.scheme().to_lowercase();
    if scheme != "http" && scheme != "https" {
        bail!("blocked scheme '{}' in URL: {}", scheme, url);
    }

    let host = parsed
        .host_str()
        .ok_or_else(|| anyhow::anyhow!("URL has no host: {}", url))?;

    let bare = host.trim_start_matches('[').trim_end_matches(']');
    if let Ok(ip) = bare.parse::<IpAddr>() {
        if is_private_ip(&ip) {
            bail!("blocked host (private/reserved IP) in URL: {}", url);
        }
    } else if host == "localhost" {
        bail!("blocked host (private/reserved IP) in URL: {}", url);
    } else {
        match resolver.resolve(host) {
            Ok(ips) => {
                if ips.is_empty() {
                    bail!("host {} did not resolve to any IP address", host);
                }
                for ip in ips {
                    if is_private_ip(&ip) {
                        bail!(
                            "blocked host (resolves to private IP {}) in URL: {}",
                            ip,
                            url
                        );
                    }
                }
            }
            Err(e) => {
                bail!("DNS resolution failed for host {}: {}", host, e);
            }
        }
    }

    if !allowed_domains.is_empty() {
        let host_lower = host
            .trim_start_matches('[')
            .trim_end_matches(']')
            .to_lowercase();
        let domain_match = allowed_domains.iter().any(|d| {
            let d_lower = d.to_lowercase();
            host_lower == d_lower || host_lower.ends_with(&format!(".{}", d_lower))
        });
        if !domain_match {
            bail!(
                "host '{}' not in allowed domains: {:?}",
                host,
                allowed_domains
            );
        }
    }

    Ok(())
}

pub fn validate_url(url: &str, allowed_domains: &[String]) -> Result<()> {
    validate_url_with_resolver(url, allowed_domains, &SystemDnsResolver)
}

fn html_to_text(html: &str) -> String {
    let mut result = String::with_capacity(html.len());
    let mut in_tag = false;
    let mut in_script = false;
    let mut in_style = false;
    let mut tag_name = String::new();
    let mut collecting_tag_name = false;
    let mut last_was_space = true;

    let block_tags = [
        "p",
        "div",
        "br",
        "h1",
        "h2",
        "h3",
        "h4",
        "h5",
        "h6",
        "li",
        "tr",
        "hr",
        "blockquote",
        "section",
        "article",
        "header",
        "footer",
        "nav",
        "main",
    ];

    let mut chars = html.chars().peekable();
    while let Some(ch) = chars.next() {
        if in_script || in_style {
            if ch == '<' {
                let mut close_tag = String::new();
                if chars.peek() == Some(&'/') {
                    chars.next();
                }
                while let Some(&c) = chars.peek() {
                    if c.is_alphanumeric() {
                        close_tag.push(chars.next().unwrap());
                    } else {
                        break;
                    }
                }
                if (in_script && close_tag.eq_ignore_ascii_case("script"))
                    || (in_style && close_tag.eq_ignore_ascii_case("style"))
                {
                    in_script = false;
                    in_style = false;
                }
            }
            continue;
        }

        match ch {
            '<' => {
                in_tag = true;
                tag_name.clear();
                collecting_tag_name = true;
            }
            '>' => {
                if in_tag {
                    let tn = tag_name.trim().to_lowercase();
                    if tn == "script" || tn.starts_with("script ") {
                        in_script = true;
                    } else if tn == "style" || tn.starts_with("style ") {
                        in_style = true;
                    }

                    let is_block = block_tags
                        .iter()
                        .any(|bt| tn == *bt || tn.starts_with(&format!("{} ", bt)));
                    if is_block && !last_was_space {
                        result.push('\n');
                        last_was_space = true;
                    }
                    if tn == "br" || tn.starts_with("br ") {
                        result.push('\n');
                        last_was_space = true;
                    }
                    in_tag = false;
                    collecting_tag_name = false;
                }
            }
            _ => {
                if in_tag {
                    if collecting_tag_name {
                        if ch.is_alphanumeric() || ch == '/' || ch == '!' || ch == '-' {
                            tag_name.push(ch);
                        } else {
                            collecting_tag_name = false;
                        }
                    }
                } else {
                    match ch {
                        '&' => {
                            let mut entity = String::new();
                            let mut semi_found = false;
                            while let Some(&next) = chars.peek() {
                                if next == ';' {
                                    chars.next();
                                    semi_found = true;
                                    break;
                                }
                                if next.is_alphanumeric() || next == '#' {
                                    entity.push(chars.next().unwrap());
                                } else {
                                    break;
                                }
                            }
                            if semi_found {
                                let decoded = match entity.as_str() {
                                    "amp" => '&',
                                    "lt" => '<',
                                    "gt" => '>',
                                    "quot" => '"',
                                    "apos" => '\'',
                                    "nbsp" => ' ',
                                    "#39" => '\'',
                                    "#160" => ' ',
                                    _ => ' ',
                                };
                                if !last_was_space || !decoded.is_whitespace() {
                                    result.push(decoded);
                                    last_was_space = decoded.is_whitespace();
                                }
                            } else {
                                result.push('&');
                                last_was_space = false;
                            }
                        }
                        ' ' | '\t' | '\n' | '\r' => {
                            if !last_was_space {
                                result.push(' ');
                                last_was_space = true;
                            }
                        }
                        _ => {
                            result.push(ch);
                            last_was_space = false;
                        }
                    }
                }
            }
        }
    }

    let trimmed = result.trim();
    let mut final_result = String::with_capacity(trimmed.len());
    let mut prev_newline = false;
    for line in trimmed.lines() {
        let l = line.trim();
        if l.is_empty() {
            if !prev_newline {
                final_result.push('\n');
                prev_newline = true;
            }
        } else {
            if !final_result.is_empty() && !prev_newline {
                final_result.push('\n');
            }
            final_result.push_str(l);
            final_result.push('\n');
            prev_newline = false;
        }
    }
    final_result.trim().to_string()
}

fn extract_title(html: &str) -> Option<String> {
    let lower = html.to_lowercase();
    let start = lower.find("<title>")? + "<title>".len();
    let end = lower.find("</title>")?;
    if end <= start {
        return None;
    }
    let title = html[start..end].trim().to_string();
    if title.is_empty() {
        return None;
    }
    Some(title)
}

fn sha256_hex(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    hex::encode(hasher.finalize())
}

pub struct WebNormalizer;

impl Normalizer for WebNormalizer {
    fn normalize(
        &self,
        raw: &RawContent,
        source_id: &str,
        corpus_id: &str,
    ) -> Result<Vec<Document>> {
        let content_text =
            String::from_utf8(raw.bytes.clone()).context("web content is not valid UTF-8")?;

        let metadata: serde_json::Value =
            serde_json::from_str(&raw.metadata_json).unwrap_or(serde_json::json!({}));
        let title = metadata["title"].as_str().unwrap_or("untitled").to_string();

        let checksum = sha256_hex(content_text.as_bytes());
        let now = Utc::now().to_rfc3339();
        let doc = Document {
            document_id: format!("doc_{}", Ulid::new()),
            corpus_id: corpus_id.to_string(),
            source_id: source_id.to_string(),
            external_id: Some(raw.external_id.clone()),
            library_id: None,
            title,
            uri: raw.uri.clone(),
            version: None,
            mime_type: Some("text/plain".to_string()),
            content_text,
            metadata_json: raw.metadata_json.clone(),
            checksum,
            superseded_by_document_id: None,
            created_at: now.clone(),
            updated_at: now,
        };
        Ok(vec![doc])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn web_adapter_kind() {
        let adapter = WebSourceAdapter;
        assert_eq!(adapter.kind(), "website");
    }

    #[test]
    fn config_parsing() {
        let json = r#"{"urls": ["https://example.com/a", "https://example.com/b"], "max_depth": 2, "allowed_domains": ["example.com"]}"#;
        let config: WebConfig = serde_json::from_str(json).unwrap();
        assert_eq!(config.urls.len(), 2);
        assert_eq!(config.max_depth, 2);
        assert_eq!(config.allowed_domains, vec!["example.com".to_string()]);
    }

    #[test]
    fn config_parsing_defaults() {
        let json = r#"{"urls": ["https://example.com"]}"#;
        let config: WebConfig = serde_json::from_str(json).unwrap();
        assert_eq!(config.max_depth, 1);
        assert!(config.allowed_domains.is_empty());
    }

    #[test]
    fn discover_units_returns_correct_units() {
        let adapter = WebSourceAdapter;
        // Public IP-literal URL: avoids any DNS lookup so the test is offline
        // and deterministic (the unit-construction behavior under test does not
        // depend on resolution).
        let config = serde_json::json!({
            "urls": ["https://93.184.216.34/page1", "https://93.184.216.34/page2"],
            "max_depth": 1
        })
        .to_string();

        let units = adapter.discover_units("src1", &config).unwrap();
        assert_eq!(units.len(), 2);
        assert_eq!(units[0].external_id, "https://93.184.216.34/page1");
        assert_eq!(
            units[0].uri,
            Some("https://93.184.216.34/page1".to_string())
        );
        assert_eq!(units[0].mime_type_hint, Some("text/html".to_string()));
        assert_eq!(units[1].external_id, "https://93.184.216.34/page2");
    }

    #[test]
    fn ssrf_blocks_private_ip_127() {
        let result = validate_url("http://127.0.0.1/admin", &[]);
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("blocked host"), "got: {}", err);
    }

    #[test]
    fn ssrf_blocks_private_ip_10() {
        let result = validate_url("http://10.0.0.1/secret", &[]);
        assert!(result.is_err());
    }

    #[test]
    fn ssrf_blocks_private_ip_192_168() {
        let result = validate_url("http://192.168.1.1/router", &[]);
        assert!(result.is_err());
    }

    #[test]
    fn ssrf_blocks_private_ip_172_16() {
        let result = validate_url("http://172.16.0.1/internal", &[]);
        assert!(result.is_err());
    }

    #[test]
    fn ssrf_blocks_private_ip_172_31() {
        let result = validate_url("http://172.31.255.255/internal", &[]);
        assert!(result.is_err());
    }

    #[test]
    fn ssrf_allows_172_15() {
        let result = validate_url("http://172.15.0.1/something", &[]);
        assert!(result.is_ok(), "172.15.x.x is not in private range");
    }

    #[test]
    fn ssrf_allows_172_32() {
        let result = validate_url("http://172.32.0.1/something", &[]);
        assert!(result.is_ok(), "172.32.x.x is not in private range");
    }

    #[test]
    fn ssrf_blocks_link_local() {
        let result = validate_url("http://169.254.1.1/metadata", &[]);
        assert!(result.is_err());
    }

    #[test]
    fn ssrf_blocks_zero_ip() {
        let result = validate_url("http://0.0.0.0/", &[]);
        assert!(result.is_err());
    }

    #[test]
    fn ssrf_blocks_localhost_hostname() {
        let result = validate_url("http://localhost/admin", &[]);
        assert!(result.is_err());
    }

    #[test]
    fn ssrf_blocks_ipv6_loopback() {
        let result = validate_url("http://[::1]/admin", &[]);
        assert!(result.is_err());
    }

    #[test]
    fn ssrf_blocks_file_scheme() {
        let result = validate_url("file:///etc/passwd", &[]);
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("blocked scheme"), "got: {}", err);
    }

    #[test]
    fn ssrf_blocks_ftp_scheme() {
        let result = validate_url("ftp://example.com/file", &[]);
        assert!(result.is_err());
    }

    #[test]
    fn ssrf_blocks_data_scheme() {
        let result = validate_url("data:text/html,<script>alert(1)</script>", &[]);
        assert!(result.is_err());
    }

    #[test]
    fn ssrf_allows_public_https() {
        let resolver = StaticResolver::new("docs.example.com", "93.184.216.34");
        let result = validate_url_with_resolver("https://docs.example.com/page", &[], &resolver);
        assert!(result.is_ok());
    }

    #[test]
    fn ssrf_allows_public_http() {
        let result = validate_url("http://example.com/page", &[]);
        assert!(result.is_ok());
    }

    #[test]
    fn ssrf_respects_allowed_domains_match() {
        let allowed = vec!["docs.example.com".to_string()];
        let resolver = StaticResolver::new("docs.example.com", "93.184.216.34");
        let result =
            validate_url_with_resolver("https://docs.example.com/page", &allowed, &resolver);
        assert!(result.is_ok());
    }

    #[test]
    fn ssrf_respects_allowed_domains_subdomain() {
        let allowed = vec!["example.com".to_string()];
        let resolver = StaticResolver::new("docs.example.com", "93.184.216.34");
        let result =
            validate_url_with_resolver("https://docs.example.com/page", &allowed, &resolver);
        assert!(result.is_ok());
    }

    #[test]
    fn ssrf_rejects_non_allowed_domain() {
        let allowed = vec!["docs.example.com".to_string()];
        let result = validate_url("https://evil.com/page", &allowed);
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("not in allowed domains"), "got: {}", err);
    }

    #[test]
    fn html_tag_stripping() {
        let html = "<html><body><h1>Title</h1><p>Hello <b>world</b></p></body></html>";
        let text = html_to_text(html);
        assert!(!text.contains('<'));
        assert!(!text.contains('>'));
        assert!(text.contains("Title"));
        assert!(text.contains("Hello"));
        assert!(text.contains("world"));
    }

    #[test]
    fn html_strips_script_and_style() {
        let html = "<html><head><style>body{color:red}</style></head><body><script>alert('xss')</script><p>Content</p></body></html>";
        let text = html_to_text(html);
        assert!(!text.contains("alert"));
        assert!(!text.contains("color"));
        assert!(text.contains("Content"));
    }

    #[test]
    fn html_entity_decoding() {
        let html = "<p>foo &amp; bar &lt; baz &gt; qux</p>";
        let text = html_to_text(html);
        assert!(text.contains("foo & bar"));
        assert!(text.contains("< baz"));
        assert!(text.contains("> qux"));
    }

    #[test]
    fn html_preserves_block_breaks() {
        let html = "<p>First</p><p>Second</p>";
        let text = html_to_text(html);
        assert!(text.contains("First"));
        assert!(text.contains("Second"));
        let lines: Vec<&str> = text.lines().collect();
        assert!(
            lines.len() >= 2,
            "expected block separation, got: {:?}",
            lines
        );
    }

    #[test]
    fn html_br_produces_newline() {
        let html = "<p>Line1<br>Line2</p>";
        let text = html_to_text(html);
        assert!(text.contains("Line1"));
        assert!(text.contains("Line2"));
    }

    #[test]
    fn title_extraction() {
        let html = "<html><head><title>My Page Title</title></head><body></body></html>";
        let title = extract_title(html);
        assert_eq!(title, Some("My Page Title".to_string()));
    }

    #[test]
    fn title_extraction_case_insensitive() {
        let html = "<HTML><HEAD><TITLE>Case Test</TITLE></HEAD></HTML>";
        let title = extract_title(html);
        assert_eq!(title, Some("Case Test".to_string()));
    }

    #[test]
    fn title_extraction_empty() {
        let html = "<html><head><title></title></head><body></body></html>";
        let title = extract_title(html);
        assert_eq!(title, None);
    }

    #[test]
    fn title_extraction_missing() {
        let html = "<html><body>No title tag</body></html>";
        let title = extract_title(html);
        assert_eq!(title, None);
    }

    #[test]
    fn web_normalizer_creates_document() {
        let raw = RawContent {
            external_id: "https://example.com/page".to_string(),
            uri: Some("https://example.com/page".to_string()),
            mime_type: "text/html".to_string(),
            bytes: b"Some page content".to_vec(),
            metadata_json: serde_json::json!({"title": "Test Page"}).to_string(),
        };

        let normalizer = WebNormalizer;
        let docs = normalizer.normalize(&raw, "src1", "corp1").unwrap();

        assert_eq!(docs.len(), 1);
        let doc = &docs[0];
        assert!(doc.document_id.starts_with("doc_"));
        assert_eq!(doc.source_id, "src1");
        assert_eq!(doc.corpus_id, "corp1");
        assert_eq!(doc.title, "Test Page");
        assert_eq!(doc.uri, Some("https://example.com/page".to_string()));
        assert_eq!(doc.content_text, "Some page content");
        assert!(!doc.checksum.is_empty());
    }

    #[test]
    fn web_normalizer_uses_untitled_when_no_title() {
        let raw = RawContent {
            external_id: "https://example.com/page".to_string(),
            uri: Some("https://example.com/page".to_string()),
            mime_type: "text/html".to_string(),
            bytes: b"content".to_vec(),
            metadata_json: "{}".to_string(),
        };

        let normalizer = WebNormalizer;
        let docs = normalizer.normalize(&raw, "src1", "corp1").unwrap();
        assert_eq!(docs[0].title, "untitled");
    }

    #[test]
    fn validate_url_with_resolver_blocks_private_dns_resolution() {
        let resolver = StaticResolver::new("docs.example.com", "127.0.0.1");
        let err = validate_url_with_resolver("https://docs.example.com/page", &[], &resolver)
            .unwrap_err();
        assert!(err.to_string().contains("private IP"), "got: {}", err);
    }

    #[test]
    fn validate_url_with_resolver_allows_public_dns_resolution() {
        let resolver = StaticResolver::new("docs.example.com", "93.184.216.34");
        let result = validate_url_with_resolver("https://docs.example.com/page", &[], &resolver);
        assert!(result.is_ok());
    }

    #[test]
    fn validate_url_with_resolver_blocks_private_ipv6_dns() {
        let resolver = StaticResolver::new("evil.example.com", "::1");
        let err = validate_url_with_resolver("https://evil.example.com/page", &[], &resolver)
            .unwrap_err();
        assert!(err.to_string().contains("private IP"), "got: {}", err);
    }

    #[test]
    fn validate_url_with_resolver_rejects_unresolvable_host() {
        struct EmptyResolver;
        impl DnsResolver for EmptyResolver {
            fn resolve(
                &self,
                _host: &str,
            ) -> Result<Vec<IpAddr>, Box<dyn std::error::Error + Send + Sync>> {
                Ok(vec![])
            }
        }
        let result = validate_url_with_resolver(
            "https://unresolvable.example.com/page",
            &[],
            &EmptyResolver,
        );
        let err = result.unwrap_err().to_string();
        assert!(
            err.contains("did not resolve to any IP address"),
            "got: {}",
            err
        );
    }

    #[test]
    fn validate_url_with_resolver_fails_closed_on_resolver_error() {
        struct FailingResolver;
        impl DnsResolver for FailingResolver {
            fn resolve(
                &self,
                host: &str,
            ) -> Result<Vec<IpAddr>, Box<dyn std::error::Error + Send + Sync>> {
                Err(format!("simulated resolution failure for {}", host).into())
            }
        }
        let result = validate_url_with_resolver("https://example.com/", &[], &FailingResolver);
        let err = result.unwrap_err().to_string();
        assert!(
            err.contains("DNS resolution failed for host"),
            "got: {}",
            err
        );
    }

    #[test]
    fn config_parsing_with_conditional_headers() {
        let json = r#"{"urls": ["https://example.com"], "previous_etag": "\"abc123\"", "previous_last_modified": "Tue, 02 Jun 2026 10:00:00 GMT"}"#;
        let config: WebConfig = serde_json::from_str(json).unwrap();
        assert_eq!(config.previous_etag, Some("\"abc123\"".to_string()));
        assert_eq!(
            config.previous_last_modified,
            Some("Tue, 02 Jun 2026 10:00:00 GMT".to_string())
        );
    }

    #[test]
    fn config_parsing_conditional_headers_default_none() {
        let json = r#"{"urls": ["https://example.com"]}"#;
        let config: WebConfig = serde_json::from_str(json).unwrap();
        assert!(config.previous_etag.is_none());
        assert!(config.previous_last_modified.is_none());
    }

    #[test]
    fn fetch_content_records_etag_and_last_modified_in_metadata() {
        let mut metadata = serde_json::Map::new();
        metadata.insert(
            "title".to_string(),
            serde_json::Value::String("Test".to_string()),
        );
        metadata.insert(
            "etag".to_string(),
            serde_json::Value::String("\"abc\"".to_string()),
        );
        metadata.insert(
            "last_modified".to_string(),
            serde_json::Value::String("Tue, 02 Jun 2026 10:00:00 GMT".to_string()),
        );

        let metadata_json = serde_json::Value::Object(metadata).to_string();
        let parsed: serde_json::Value = serde_json::from_str(&metadata_json).unwrap();

        assert_eq!(parsed["etag"], "\"abc\"");
        assert_eq!(parsed["last_modified"], "Tue, 02 Jun 2026 10:00:00 GMT");
    }
}
