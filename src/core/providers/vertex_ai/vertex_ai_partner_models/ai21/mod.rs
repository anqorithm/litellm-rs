//! AI21 Partner Model Support

use crate::ProviderError;

/// AI21 transformation handler
pub struct AI21Handler;

impl AI21Handler {
    /// Handle AI21 model requests
    pub async fn handle_request(
        request: serde_json::Value,
    ) -> Result<serde_json::Value, ProviderError> {
        // NOTE: AI21 request handling not yet implemented
        Ok(request)
    }

    /// Transform request for Jamba models
    pub fn transform_jamba_request(request: serde_json::Value) -> serde_json::Value {
        // AI21 Jamba-specific transformations
        request
    }
}
