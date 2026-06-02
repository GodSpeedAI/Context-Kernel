/// Port for authentication and authorization.
pub trait AuthPort: Send + Sync {
    /// Validate a bearer token. Returns true if valid.
    fn validate_token(&self, token: &str) -> bool;

    /// Check if the current trust mode allows unauthenticated access.
    fn is_local_trust_mode(&self) -> bool;

    /// Check whether a token may call a tool.
    fn authorize_tool(&self, token: Option<&str>, tool_name: &str) -> bool;

    /// Check whether a token may access a corpus.
    fn authorize_corpus(&self, token: Option<&str>, corpus_id: &str) -> bool;
}
