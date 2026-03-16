//! Llama 3 Partner Model Support

use crate::ProviderError;

/// Llama3 transformation handler
pub struct Llama3Handler;

impl Llama3Handler {
    /// Handle Llama3 model requests
    pub async fn handle_request(
        request: serde_json::Value,
    ) -> Result<serde_json::Value, ProviderError> {
        // Transform for Llama models on Vertex AI
        let transformed = Self::transform_llama_request(request)?;
        Ok(transformed)
    }

    /// Transform request for Llama models
    fn transform_llama_request(
        request: serde_json::Value,
    ) -> Result<serde_json::Value, ProviderError> {
        // Llama models use chat template format
        Ok(serde_json::json!({
            "instances": [{
                "messages": request.get("messages").unwrap_or(&serde_json::Value::Null)
            }],
            "parameters": {
                "temperature": request.get("temperature").unwrap_or(&serde_json::json!(0.7)),
                "maxOutputTokens": request.get("max_tokens").unwrap_or(&serde_json::Value::Number(2048.into())),
            }
        }))
    }
}
