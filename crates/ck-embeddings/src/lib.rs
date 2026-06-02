use anyhow::Result;
use ck_core::ports::EmbeddingPort;
use serde::Deserialize;

pub struct NoopEmbeddingProvider;

impl EmbeddingPort for NoopEmbeddingProvider {
    fn embed(&self, _text: &str) -> Result<Option<Vec<f64>>> {
        Ok(None)
    }

    fn embed_batch(&self, texts: &[&str]) -> Result<Vec<Option<Vec<f64>>>> {
        Ok(texts.iter().map(|_| None).collect())
    }

    fn model_id(&self) -> &str {
        "none"
    }

    fn dimensions(&self) -> usize {
        0
    }

    fn is_available(&self) -> bool {
        false
    }
}

pub struct OpenAIEmbeddingProvider {
    api_base: String,
    api_key: String,
    model: String,
    dimensions: usize,
    agent: ureq::Agent,
}

#[derive(Deserialize)]
struct EmbeddingResponse {
    data: Vec<EmbeddingData>,
}

#[derive(Deserialize)]
struct EmbeddingData {
    embedding: Vec<f64>,
}

impl OpenAIEmbeddingProvider {
    pub fn new(api_base: String, api_key: String, model: String, dimensions: usize) -> Self {
        let agent = ureq::AgentBuilder::new()
            .timeout_read(std::time::Duration::from_secs(30))
            .timeout_write(std::time::Duration::from_secs(10))
            .build();
        Self {
            api_base,
            api_key,
            model,
            dimensions,
            agent,
        }
    }

    fn build_request_body(&self, input: &serde_json::Value) -> serde_json::Value {
        let mut body = serde_json::json!({
            "model": self.model,
            "input": input,
        });
        if self.dimensions > 0 {
            body["dimensions"] = serde_json::json!(self.dimensions);
        }
        body
    }

    fn post_embedding(&self, input: &serde_json::Value) -> Result<EmbeddingResponse> {
        let url = format!("{}/embeddings", self.api_base.trim_end_matches('/'));
        let body = self.build_request_body(input);

        let response: EmbeddingResponse = self
            .agent
            .post(&url)
            .set("Authorization", &format!("Bearer {}", self.api_key))
            .set("Content-Type", "application/json")
            .send_json(&body)?
            .into_json()?;

        Ok(response)
    }
}

impl EmbeddingPort for OpenAIEmbeddingProvider {
    fn embed(&self, text: &str) -> Result<Option<Vec<f64>>> {
        if !self.is_available() {
            return Ok(None);
        }

        let input = serde_json::json!(text);
        let response = self.post_embedding(&input)?;

        response
            .data
            .into_iter()
            .next()
            .map(|d| d.embedding)
            .ok_or_else(|| anyhow::anyhow!("no embedding returned in response"))
            .map(Some)
    }

    fn embed_batch(&self, texts: &[&str]) -> Result<Vec<Option<Vec<f64>>>> {
        if !self.is_available() {
            return Ok(texts.iter().map(|_| None).collect());
        }

        let input: Vec<&str> = texts.to_vec();
        let input = serde_json::json!(input);
        let response = self.post_embedding(&input)?;

        let mut results: Vec<Option<Vec<f64>>> = vec![None; texts.len()];
        for (i, item) in response.data.into_iter().enumerate() {
            if i < results.len() {
                results[i] = Some(item.embedding);
            }
        }

        Ok(results)
    }

    fn model_id(&self) -> &str {
        &self.model
    }

    fn dimensions(&self) -> usize {
        self.dimensions
    }

    fn is_available(&self) -> bool {
        !self.api_key.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn noop_returns_none() {
        let provider = NoopEmbeddingProvider;
        assert!(!provider.is_available());
        assert_eq!(provider.model_id(), "none");
        assert_eq!(provider.dimensions(), 0);
        assert!(provider.embed("hello").unwrap().is_none());
    }

    #[test]
    fn noop_batch_returns_none_vec() {
        let provider = NoopEmbeddingProvider;
        let result = provider.embed_batch(&["a", "b", "c"]).unwrap();
        assert_eq!(result.len(), 3);
        assert!(result.iter().all(|r| r.is_none()));
    }

    #[test]
    fn openai_empty_key_not_available() {
        let provider = OpenAIEmbeddingProvider::new(
            "https://api.openai.com/v1".to_string(),
            String::new(),
            "text-embedding-3-small".to_string(),
            1536,
        );
        assert!(!provider.is_available());
        assert!(provider.embed("hello").unwrap().is_none());
    }

    #[test]
    fn openai_with_key_is_available() {
        let provider = OpenAIEmbeddingProvider::new(
            "https://api.openai.com/v1".to_string(),
            "sk-test-key".to_string(),
            "text-embedding-3-small".to_string(),
            1536,
        );
        assert!(provider.is_available());
        assert_eq!(provider.model_id(), "text-embedding-3-small");
        assert_eq!(provider.dimensions(), 1536);
    }

    #[test]
    fn openai_batch_with_empty_key_returns_none() {
        let provider = OpenAIEmbeddingProvider::new(
            "https://api.openai.com/v1".to_string(),
            String::new(),
            "text-embedding-3-small".to_string(),
            1536,
        );
        let result = provider.embed_batch(&["a", "b"]).unwrap();
        assert_eq!(result.len(), 2);
        assert!(result.iter().all(|r| r.is_none()));
    }
}
