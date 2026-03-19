//! Embedding methods (placeholder for future implementation)

use super::llm_client::LLMClient;
use crate::sdk::errors::*;

impl LLMClient {
    /// Generate embeddings for text
    ///
    /// This is a placeholder for future embedding functionality.
    /// Will support various embedding models across different providers.
    pub async fn embedding(&self, _text: &str, _model: Option<&str>) -> Result<Vec<f32>> {
        // NOTE: embedding functionality not yet implemented
        Err(SDKError::NotSupported(
            "Embedding functionality not yet implemented".to_string(),
        ))
    }

    /// Generate embeddings for multiple texts in batch
    ///
    /// This is a placeholder for future batch embedding functionality.
    /// Will support batch processing for efficiency.
    pub async fn batch_embedding(
        &self,
        _texts: &[String],
        _model: Option<&str>,
    ) -> Result<Vec<Vec<f32>>> {
        // NOTE: batch embedding functionality not yet implemented
        Err(SDKError::NotSupported(
            "Batch embedding functionality not yet implemented".to_string(),
        ))
    }
}
