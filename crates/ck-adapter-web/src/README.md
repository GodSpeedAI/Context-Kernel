# ck-adapter-web src

## Overview

Web source adapter for Context Kernel ingestion with SSRF guardrails, implementing the `ck-core` `SourceAdapter` and `Normalizer` ports. `WebSourceAdapter` validates each configured URL, fetches it with redirects disabled and optional conditional-GET headers, and converts the HTML body to plain text. URL validation is centralized in `validate_url_with_resolver`, which checks the scheme, host IP literals, DNS resolution results, and an optional domain allowlist against an injectable `DnsResolver` trait. The crate is implemented and unit-tested at the crate level, but it is not composed into the `ck-bin` binary (`ck-bin/Cargo.toml` does not list it as a dependency).

## Interface

- [lib.rs:DnsResolver] - `pub trait` (`Send + Sync`); `resolve(host)` returns the resolved IP addresses so callers and tests can inject DNS behavior.
- [lib.rs:SystemDnsResolver] - `pub struct`; default resolver using `std::net::ToSocketAddrs`.
- [lib.rs:StaticResolver] - `pub struct` with `new(host, ip)`; resolver that pins one host to one IP and returns no addresses for any other host.
- [lib.rs:WebSourceAdapter] - `pub struct`; implements `SourceAdapter` with kind `"website"`; `discover_units` validates configured URLs, `fetch_content` performs the HTTP GET and HTML-to-text conversion.
- [lib.rs:validate_url_with_resolver] - `pub fn`; full URL validation (scheme, private-IP, DNS, allowlist) against a supplied resolver.
- [lib.rs:validate_url] - `pub fn`; `validate_url_with_resolver` using the system resolver.
- [lib.rs:WebNormalizer] - `pub struct`; implements `Normalizer`; converts one `RawContent` into one `Document` (emitted as `text/plain`, title taken from fetch metadata).

## Invariants

- Only `http` and `https` schemes are allowed [lib.rs:validate_url_with_resolver]
- Hosts given as IP literals and `localhost` are blocked when they are private or reserved [lib.rs:validate_url_with_resolver]
- Blocked IPv4 ranges: `10/8`, `172.16/12`, `192.168/16`, `127/8`, `169.254/16`, and `0/8` [lib.rs:is_blocked_ipv4]
- Blocked IPv6 ranges: loopback, link-local `fe80::/10`, and unique-local `fc00::/7`; IPv4-mapped addresses are re-checked against the IPv4 rules [lib.rs:is_blocked_ipv6]
- DNS resolution failure or an empty resolution result is an error; validation fails closed [lib.rs:validate_url_with_resolver]
- Every resolved IP address must be public; resolution to any blocked IP rejects the URL [lib.rs:validate_url_with_resolver]
- When `allowed_domains` is non-empty, the host must equal or be a subdomain of one of its entries (case-insensitive) [lib.rs:validate_url_with_resolver]
- The HTTP agent is built with `redirects(0)`, so HTTP redirects are never followed [lib.rs:WebSourceAdapter::fetch_content]
- A `304 Not Modified` response yields empty bytes with `not_modified: true` metadata instead of content [lib.rs:WebSourceAdapter::fetch_content]
- HTML-to-text conversion strips tags, drops `script`/`style` bodies, and decodes a small fixed set of HTML entities [lib.rs:html_to_text]
- Residual risk: validation resolves DNS separately from the subsequent HTTP request, which re-resolves the host by name, so the validated IP is not pinned (DNS-rebinding window remains) [lib.rs:WebSourceAdapter::fetch_content]
- Normalization requires valid UTF-8 content [lib.rs:WebNormalizer]

## Side Effects

- Outbound HTTP GET requests via `ureq`, including conditional headers `If-None-Match` / `If-Modified-Since` when previous validators are configured [lib.rs:WebSourceAdapter::fetch_content]
- DNS resolution through the system resolver (`ToSocketAddrs`) during URL validation [lib.rs:SystemDnsResolver]
- Uses current wall-clock time (`Utc::now`) for document timestamps [lib.rs:WebNormalizer]

## Verification

- Run: `cargo test -p ck-adapter-web`
