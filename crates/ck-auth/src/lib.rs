use ck_core::ports::AuthPort;
use subtle::ConstantTimeEq;

pub struct BearerAuth {
    valid_tokens: Vec<String>,
    local_trust_mode: bool,
}

impl BearerAuth {
    pub fn new(valid_tokens: Vec<String>, local_trust_mode: bool) -> Self {
        Self {
            valid_tokens,
            local_trust_mode,
        }
    }

    pub fn no_auth() -> Self {
        Self {
            valid_tokens: vec![],
            local_trust_mode: true,
        }
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
}
