/// Port for authentication and authorization.
pub trait AuthPort: Send + Sync {
    /// Validate a bearer token. Returns true if valid.
    fn validate_token(&self, token: &str) -> bool;

    /// Check if the current trust mode allows unauthenticated access.
    fn is_local_trust_mode(&self) -> bool;
}
