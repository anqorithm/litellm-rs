//! Vertex AI Embeddings Module
//!
//! Legacy embedding models and compatibility

use crate::ProviderError;
use serde::{Deserialize, Serialize};

/// Legacy embedding types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LegacyEmbeddingTypes {
    pub model: String,
    pub dimensions: usize,
}

/// Legacy embedding handler
pub struct VertexEmbeddingHandler;

impl VertexEmbeddingHandler {
    /// Handle legacy embedding requests
    pub async fn handle_legacy_embedding(
        model: &str,
        _text: &str,
    ) -> Result<Vec<f32>, ProviderError> {
        // Handle legacy models like textembedding-gecko
        match model {
            "textembedding-gecko" | "textembedding-gecko@003" => {
                // NOTE: gecko embedding not yet implemented
                Ok(vec![0.0; 768])
            }
            "textembedding-gecko-multilingual" => {
                // NOTE: multilingual gecko not yet implemented
                Ok(vec![0.0; 768])
            }
            _ => Err(ProviderError::model_not_found("vertex_ai", model)),
        }
    }
}
