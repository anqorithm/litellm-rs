//! Chat completion functionality for Moonshot provider

pub mod transformation;

use serde_json::Value;
use tracing::{debug, info};

use crate::core::providers::moonshot::{MoonshotConfig, MoonshotError};
pub use transformation::MoonshotChatTransformation;

/// Moonshot chat handler
#[derive(Debug, Clone)]
pub struct MoonshotChatHandler {
    transformation: MoonshotChatTransformation,
}

impl MoonshotChatHandler {
    /// Create a new chat handler
    pub fn new(_config: MoonshotConfig) -> Result<Self, MoonshotError> {
        Ok(Self {
            transformation: MoonshotChatTransformation::new(),
        })
    }

    /// Transform a standard chat request to Moonshot format
    pub fn transform_request(
        &self,
        request: crate::core::types::chat::ChatRequest,
    ) -> Result<Value, MoonshotError> {
        debug!("Transforming chat request for Moonshot");

        // Apply Moonshot-specific transformations
        let transformed = self.transformation.transform_request(request)?;

        Ok(transformed)
    }

    /// Transform a Moonshot response to standard format
    pub fn transform_response(
        &self,
        response: Value,
    ) -> Result<crate::core::types::responses::ChatResponse, MoonshotError> {
        debug!("Transforming Moonshot response");

        // Apply Moonshot-specific transformations
        let standard_response = self.transformation.transform_response(response)?;

        info!("Successfully transformed Moonshot response");
        Ok(standard_response)
    }
}
