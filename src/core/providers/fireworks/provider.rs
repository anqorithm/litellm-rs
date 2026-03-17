//! Main Fireworks AI Provider Implementation
//!
//! Implements the LLMProvider trait for Fireworks AI's fast inference platform.

use std::collections::HashMap;
use std::sync::Arc;
use tracing::debug;

use super::config::FireworksConfig;
use super::model_info::{get_available_models, get_model_info};
use crate::core::providers::base::{GlobalPoolManager, HttpErrorMapper, HttpMethod, header};
use crate::core::providers::unified_provider::ProviderError;
use crate::core::streaming::utils::is_done_marker;
use crate::core::traits::{
    provider::ProviderConfig as _, provider::llm_provider::trait_definition::LLMProvider,
};
use crate::core::types::{
    chat::ChatRequest, model::ModelInfo, model::ProviderCapability, responses::ChatChunk,
};

/// Static capabilities for Fireworks AI provider
const FIREWORKS_CAPABILITIES: &[ProviderCapability] = &[
    ProviderCapability::ChatCompletion,
    ProviderCapability::ChatCompletionStream,
    ProviderCapability::ToolCalling,
];

fn parse_fireworks_sse_line(line: &str) -> Option<Result<ChatChunk, ProviderError>> {
    let data = line.strip_prefix("data: ")?;
    if is_done_marker(data) {
        return None;
    }

    Some(serde_json::from_str::<ChatChunk>(data).map_err(|e| {
        ProviderError::api_error("fireworks", 500, format!("Failed to parse chunk: {}", e))
    }))
}

/// Fireworks AI provider implementation
#[derive(Debug, Clone)]
pub struct FireworksProvider {
    config: FireworksConfig,
    pool_manager: Arc<GlobalPoolManager>,
    models: Vec<ModelInfo>,
}

impl FireworksProvider {
    /// Create a new Fireworks AI provider instance
    pub async fn new(config: FireworksConfig) -> Result<Self, ProviderError> {
        // Validate configuration
        config
            .validate()
            .map_err(|e| ProviderError::configuration("fireworks", e))?;

        // Create pool manager
        let pool_manager = Arc::new(GlobalPoolManager::new().map_err(|e| {
            ProviderError::configuration(
                "fireworks",
                format!("Failed to create pool manager: {}", e),
            )
        })?);

        // Build model list from static configuration
        let models = get_available_models()
            .iter()
            .filter_map(|id| get_model_info(id))
            .map(|info| {
                let mut capabilities = vec![
                    ProviderCapability::ChatCompletion,
                    ProviderCapability::ChatCompletionStream,
                ];
                if info.supports_tools {
                    capabilities.push(ProviderCapability::ToolCalling);
                }

                ModelInfo {
                    id: info.model_id.to_string(),
                    name: info.display_name.to_string(),
                    provider: "fireworks".to_string(),
                    max_context_length: info.max_context_length,
                    max_output_length: Some(info.max_output_length),
                    supports_streaming: true,
                    supports_tools: info.supports_tools,
                    supports_multimodal: info.supports_multimodal,
                    input_cost_per_1k_tokens: Some(info.input_cost_per_million / 1000.0),
                    output_cost_per_1k_tokens: Some(info.output_cost_per_million / 1000.0),
                    currency: "USD".to_string(),
                    capabilities,
                    created_at: None,
                    updated_at: None,
                    metadata: HashMap::new(),
                }
            })
            .collect();

        Ok(Self {
            config,
            pool_manager,
            models,
        })
    }

    /// Create provider with API key only
    pub async fn with_api_key(api_key: impl Into<String>) -> Result<Self, ProviderError> {
        let config = FireworksConfig {
            api_key: Some(api_key.into()),
            ..Default::default()
        };
        Self::new(config).await
    }

    /// Transform messages for Fireworks AI API
    fn transform_messages(&self, _request: &mut ChatRequest) {
        // Fireworks AI uses standard OpenAI-compatible messages
        // No special transformation needed
    }

    /// Transform tools to remove unsupported fields
    fn transform_tools(&self, request: &mut ChatRequest) {
        if let Some(ref mut tools) = request.tools {
            for tool in tools.iter_mut() {
                // Remove 'strict' field from function parameters if present
                if let Some(ref mut params) = tool.function.parameters
                    && let Some(obj) = params.as_object_mut()
                {
                    obj.remove("strict");
                }
            }
        }
    }

    /// Handle response_format with tool calling
    fn handle_response_format(&self, request: &mut ChatRequest) {
        // Fireworks AI doesn't support tools and response_format together
        // If both are set, convert response_format to a tool
        if request.tools.is_some() && request.response_format.is_some() {
            // For now, prioritize tools over response_format
            // In a full implementation, we'd convert response_format to a tool
            debug!("Fireworks AI: tools and response_format both set, using tools");
        }

        // Transform json_schema format to json_object
        if let Some(ref mut format) = request.response_format
            && format.format_type == "json_schema"
            && format.json_schema.is_some()
        {
            // Fireworks uses json_object with a schema field
            format.format_type = "json_object".to_string();
            // Keep the schema in json_schema field
        }
    }

    /// Handle tool_choice mapping
    fn map_tool_choice(&self, request: &mut ChatRequest) {
        if let Some(ref mut tool_choice) = request.tool_choice {
            // Fireworks AI uses "any" instead of "required"
            match tool_choice {
                crate::core::types::tools::ToolChoice::String(s) if s == "required" => {
                    *s = "any".to_string();
                }
                _ => {}
            }
        }
    }

    /// Execute an HTTP request
    async fn execute_request(
        &self,
        endpoint: &str,
        body: serde_json::Value,
    ) -> Result<serde_json::Value, ProviderError> {
        let url = format!("{}{}", self.config.get_api_base(), endpoint);

        let mut headers = Vec::with_capacity(2);
        if let Some(api_key) = &self.config.get_api_key() {
            headers.push(header("Authorization", format!("Bearer {}", api_key)));
        }
        headers.push(header("Content-Type", "application/json".to_string()));

        let response = self
            .pool_manager
            .execute_request(&url, HttpMethod::POST, headers, Some(body))
            .await
            .map_err(|e| ProviderError::network("fireworks", e.to_string()))?;

        let status = response.status();
        let response_bytes = response
            .bytes()
            .await
            .map_err(|e| ProviderError::network("fireworks", e.to_string()))?;

        if !status.is_success() {
            let body_str = String::from_utf8_lossy(&response_bytes);
            let status_code = status.as_u16();
            return Err(match status_code {
                401 => ProviderError::authentication("fireworks", "Invalid API key"),
                404 => ProviderError::model_not_found("fireworks", body_str.to_string()),
                429 => ProviderError::rate_limit("fireworks", None),
                400 => ProviderError::invalid_request("fireworks", body_str.to_string()),
                500..=599 => ProviderError::provider_unavailable("fireworks", body_str.to_string()),
                _ => HttpErrorMapper::map_status_code("fireworks", status_code, &body_str),
            });
        }

        serde_json::from_slice(&response_bytes).map_err(|e| {
            ProviderError::api_error("fireworks", 500, format!("Failed to parse response: {}", e))
        })
    }
}

#[cfg(test)]
mod streaming_tests {
    use super::parse_fireworks_sse_line;

    #[test]
    fn test_parse_fireworks_sse_line_done_marker() {
        assert!(parse_fireworks_sse_line("data: [DONE]").is_none());
    }

    #[test]
    fn test_parse_fireworks_sse_line_valid_chunk() {
        let line = r#"data: {"id":"chatcmpl-123","object":"chat.completion.chunk","created":1234567890,"model":"llama-v3p1-70b-instruct","choices":[{"index":0,"delta":{"content":"Hello"},"finish_reason":null}]}"#;
        let parsed = parse_fireworks_sse_line(line).expect("expected a parsed chunk result");
        let chunk = parsed.expect("expected valid chat chunk");
        assert_eq!(chunk.id, "chatcmpl-123");
        assert_eq!(chunk.choices.len(), 1);
    }

    #[test]
    fn test_parse_fireworks_sse_line_invalid_chunk() {
        let parsed = parse_fireworks_sse_line("data: {invalid json")
            .expect("expected parse attempt for malformed chunk");
        assert!(parsed.is_err());
    }
}
