use ck_core::ports::AuthPort;
use subtle::ConstantTimeEq;

pub struct BearerAuth {
    valid_tokens: Vec<String>,
    local_trust_mode: bool,
    allow_tools: Vec<String>,
    allow_corpora: Vec<String>,
}

impl BearerAuth {
    pub fn new(valid_tokens: Vec<String>, local_trust_mode: bool) -> Self {
        Self {
            valid_tokens,
            local_trust_mode,
            allow_tools: vec!["*".to_string()],
            allow_corpora: vec!["*".to_string()],
        }
    }

    pub fn no_auth() -> Self {
        Self {
            valid_tokens: vec![],
            local_trust_mode: true,
            allow_tools: vec!["*".to_string()],
            allow_corpora: vec!["*".to_string()],
        }
    }

    pub fn with_policy(
        valid_tokens: Vec<String>,
        local_trust_mode: bool,
        allow_tools: Vec<String>,
        allow_corpora: Vec<String>,
    ) -> Self {
        Self {
            valid_tokens,
            local_trust_mode,
            allow_tools: normalize_allowlist(allow_tools),
            allow_corpora: normalize_allowlist(allow_corpora),
        }
    }

    fn token_is_authorized(&self, token: Option<&str>) -> bool {
        if self.local_trust_mode && self.valid_tokens.is_empty() {
            return true;
        }

        token.is_some_and(|t| self.validate_token(t))
    }
}

impl AuthPort for BearerAuth {
    fn validate_token(&self, token: &str) -> bool {
        if self.valid_tokens.is_empty() {
            return self.local_trust_mode;
        }
        let token = token.strip_prefix("Bearer ").unwrap_or(token);
        self.valid_tokens
            .iter()
            .any(|t| t.as_bytes().ct_eq(token.as_bytes()).into())
    }

    fn is_local_trust_mode(&self) -> bool {
        self.local_trust_mode
    }

    fn authorize_tool(&self, token: Option<&str>, tool_name: &str) -> bool {
        self.token_is_authorized(token) && is_allowed(&self.allow_tools, tool_name)
    }

    fn authorize_corpus(&self, token: Option<&str>, corpus_id: &str) -> bool {
        self.token_is_authorized(token) && is_allowed(&self.allow_corpora, corpus_id)
    }
}

fn normalize_allowlist(values: Vec<String>) -> Vec<String> {
    if values.is_empty() {
        return vec!["*".to_string()];
    }
    values
}

fn is_allowed(allowlist: &[String], value: &str) -> bool {
    allowlist
        .iter()
        .any(|allowed| allowed == "*" || allowed == value)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_auth_allows_everything() {
        let auth = BearerAuth::no_auth();
        assert!(auth.validate_token("anything"));
        assert!(auth.validate_token(""));
        assert!(auth.is_local_trust_mode());
    }

    #[test]
    fn bearer_token_validation() {
        let auth = BearerAuth::new(vec!["secret123".to_string()], false);
        assert!(auth.validate_token("Bearer secret123"));
        assert!(auth.validate_token("secret123"));
        assert!(!auth.validate_token("wrong"));
        assert!(!auth.validate_token("Bearer wrong"));
        assert!(!auth.is_local_trust_mode());
    }

    #[test]
    fn empty_tokens_fail_closed_without_local_trust_mode() {
        let auth = BearerAuth::new(vec![], false);
        assert!(!auth.validate_token("anything"));
    }

    #[test]
    fn empty_tokens_allow_all_only_in_local_trust_mode() {
        let auth = BearerAuth::new(vec![], true);
        assert!(auth.validate_token("anything"));
    }

    #[test]
    fn tool_authorization_requires_valid_token_and_allowed_tool() {
        let auth = BearerAuth::with_policy(
            vec!["secret123".to_string()],
            false,
            vec!["get_status".to_string()],
            vec!["public_docs".to_string()],
        );

        assert!(auth.authorize_tool(Some("Bearer secret123"), "get_status"));
        assert!(!auth.authorize_tool(Some("Bearer secret123"), "ingest_source"));
        assert!(!auth.authorize_tool(Some("wrong"), "get_status"));
        assert!(!auth.authorize_tool(None, "get_status"));
    }

    #[test]
    fn corpus_authorization_uses_configured_allowlist() {
        let auth = BearerAuth::with_policy(
            vec!["secret123".to_string()],
            false,
            vec!["*".to_string()],
            vec!["public_docs".to_string()],
        );

        assert!(auth.authorize_corpus(Some("secret123"), "public_docs"));
        assert!(!auth.authorize_corpus(Some("secret123"), "private_docs"));
    }
}
