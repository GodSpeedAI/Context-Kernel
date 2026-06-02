use anyhow::Result;

/// Port for generating embeddings from text.
pub trait EmbeddingPort: Send + Sync {
    /// Generate an embedding for a single text input.
    /// Returns None if the embedding provider is unavailable.
    fn embed(&self, text: &str) -> Result<Option<Vec<f64>>>;

    /// Generate embeddings for a batch of text inputs.
    /// Returns None for each item where the provider is unavailable.
    fn embed_batch(&self, texts: &[&str]) -> Result<Vec<Option<Vec<f64>>>>;

    /// Return the model identifier (e.g., "text-embedding-3-small").
    fn model_id(&self) -> &str;

    /// Return the embedding dimensionality.
    fn dimensions(&self) -> usize;

    /// Whether the provider is available and configured.
    fn is_available(&self) -> bool;
}
