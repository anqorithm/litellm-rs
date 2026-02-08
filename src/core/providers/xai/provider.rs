//! Main xAI Provider Implementation
//!
//! Implements the LLMProvider trait for xAI's Grok models with OpenAI-compatible API.

use async_trait::async_trait;
use futures::Stream;
use std::collections::HashMap;
use std::pin::Pin;
use std::sync::Arc;
use tracing::{debug, info};

use super::config::XAIConfig;
use super::error::XAIError;
use super::model_info::{calculate_cost_with_reasoning, get_available_models, get_model_info};
use crate::core::providers::base::{GlobalPoolManager, HttpMethod, header};
use crate::core::traits::{
    ProviderConfig as _, provider::llm_provider::trait_definition::LLMProvider,
};
use crate::core::types::{
    ChatRequest, EmbeddingRequest, RequestContext,
    health::HealthStatus,
    model::ModelInfo,
    model::ProviderCapability,
    responses::{ChatChunk, ChatResponse, CompletionTokensDetails, EmbeddingResponse},
};

/// Static capabilities for xAI provider
const XAI_CAPABILITIES: &[ProviderCapability] = &[
    ProviderCapability::ChatCompletion,
    ProviderCapability::ChatCompletionStream,
    ProviderCapability::ToolCalling,
];

/// xAI provider implementation
#[derive(Debug, Clone)]
pub struct XAIProvider {
    config: XAIConfig,
    pool_manager: Arc<GlobalPoolManager>,
    models: Vec<ModelInfo>,
}

impl XAIProvider {
    /// Create a new xAI provider instance
    pub async fn new(config: XAIConfig) -> Result<Self, XAIError> {
        // Validate configuration
        config
            .validate()
            .map_err(|e| XAIError::configuration("xai", e))?;

        // Create pool manager
        let pool_manager = Arc::new(GlobalPoolManager::new().map_err(|e| {
            XAIError::configuration("xai", format!("Failed to create pool manager: {}", e))
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
                    id: format!("xai/{}", info.model_id),
                    name: info.display_name.to_string(),
                    provider: "xai".to_string(),
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
    pub async fn with_api_key(api_key: impl Into<String>) -> Result<Self, XAIError> {
        let config = XAIConfig {
            api_key: Some(api_key.into()),
            ..Default::default()
        };
        Self::new(config).await
    }

    /// Execute an HTTP request
    async fn execute_request(
        &self,
        endpoint: &str,
        body: serde_json::Value,
    ) -> Result<serde_json::Value, XAIError> {
        let url = format!("{}{}", self.config.get_api_base(), endpoint);

        let mut headers = Vec::with_capacity(2);
        if let Some(api_key) = self.config.get_api_key() {
            headers.push(header("Authorization", format!("Bearer {}", api_key)));
        }
        headers.push(header("Content-Type", "application/json".to_string()));

        let response = self
            .pool_manager
            .execute_request(&url, HttpMethod::POST, headers, Some(body))
            .await
            .map_err(|e| XAIError::network("xai", e.to_string()))?;

        let response_bytes = response
            .bytes()
            .await
            .map_err(|e| XAIError::network("xai", e.to_string()))?;

        serde_json::from_slice(&response_bytes).map_err(|e| {
            XAIError::api_error("xai", 500, format!("Failed to parse response: {}", e))
        })
    }

    /// Add web search parameter to request JSON
    fn add_web_search_to_json(&self, request_json: &mut serde_json::Value) {
        if self.config.enable_web_search {
            if let Some(obj) = request_json.as_object_mut() {
                obj.insert("web_search".to_string(), serde_json::json!(true));
            }
        }
    }

    /// Extract reasoning tokens from response if present
    fn extract_reasoning_tokens(&self, response: &serde_json::Value) -> Option<u32> {
        response
            .get("usage")
            .and_then(|usage| usage.get("completion_tokens_details"))
            .and_then(|details| details.get("reasoning_tokens"))
            .and_then(|tokens| tokens.as_u64())
            .map(|tokens| tokens as u32)
    }
}

#[async_trait]
impl LLMProvider for XAIProvider {
    type Config = XAIConfig;
    type Error = XAIError;
    type ErrorMapper = crate::core::traits::error_mapper::DefaultErrorMapper;

    fn name(&self) -> &'static str {
        "xai"
    }

    fn capabilities(&self) -> &'static [ProviderCapability] {
        XAI_CAPABILITIES
    }

    fn models(&self) -> &[ModelInfo] {
        &self.models
    }

    fn get_supported_openai_params(&self, model: &str) -> &'static [&'static str] {
        let supports_reasoning = get_model_info(model)
            .map(|info| info.supports_reasoning)
            .unwrap_or(false);

        if supports_reasoning {
            &[
                "temperature",
                "top_p",
                "max_tokens",
                "max_completion_tokens",
                "stream",
                "stop",
                "frequency_penalty",
                "presence_penalty",
                "n",
                "response_format",
                "seed",
                "tools",
                "tool_choice",
                "parallel_tool_calls",
                "user",
                "logprobs",
                "top_logprobs",
                "reasoning_effort", // xAI-specific for reasoning models
            ]
        } else {
            &[
                "temperature",
                "top_p",
                "max_tokens",
                "max_completion_tokens",
                "stream",
                "stop",
                "frequency_penalty",
                "presence_penalty",
                "n",
                "response_format",
                "seed",
                "tools",
                "tool_choice",
                "parallel_tool_calls",
                "user",
                "logprobs",
                "top_logprobs",
            ]
        }
    }

    async fn map_openai_params(
        &self,
        params: HashMap<String, serde_json::Value>,
        _model: &str,
    ) -> Result<HashMap<String, serde_json::Value>, Self::Error> {
        // xAI uses OpenAI-compatible parameters
        Ok(params)
    }

    async fn transform_request(
        &self,
        request: ChatRequest,
        _context: RequestContext,
    ) -> Result<serde_json::Value, Self::Error> {
        // Convert to JSON value
        let mut request_json = serde_json::to_value(&request)
            .map_err(|e| XAIError::invalid_request("xai", e.to_string()))?;

        // Add web search if enabled
        self.add_web_search_to_json(&mut request_json);

        Ok(request_json)
    }

    async fn transform_response(
        &self,
        raw_response: &[u8],
        model: &str,
        _request_id: &str,
    ) -> Result<ChatResponse, Self::Error> {
        let response_json: serde_json::Value =
            serde_json::from_slice(raw_response).map_err(|e| {
                XAIError::api_error("xai", 500, format!("Failed to parse response: {}", e))
            })?;

        // Extract reasoning tokens if present
        let reasoning_tokens = self.extract_reasoning_tokens(&response_json);

        // Parse standard response
        let mut chat_response: ChatResponse = serde_json::from_value(response_json.clone())
            .map_err(|e| {
                XAIError::api_error("xai", 500, format!("Failed to parse chat response: {}", e))
            })?;

        // If we have reasoning tokens, update the usage
        if let Some(reasoning_tokens) = reasoning_tokens {
            if let Some(ref mut usage) = chat_response.usage {
                // Ensure completion_tokens_details exists and has reasoning_tokens
                if usage.completion_tokens_details.is_none() {
                    usage.completion_tokens_details = Some(CompletionTokensDetails {
                        reasoning_tokens: Some(reasoning_tokens),
                        audio_tokens: None,
                    });
                } else if let Some(ref mut details) = usage.completion_tokens_details {
                    details.reasoning_tokens = Some(reasoning_tokens);
                }

                // Update total tokens to include reasoning
                usage.total_tokens =
                    usage.prompt_tokens + usage.completion_tokens + reasoning_tokens;
            }
        }

        // Log if web search was used
        if let Some(web_search_used) = response_json
            .get("web_search_used")
            .and_then(|v| v.as_bool())
        {
            if web_search_used {
                info!("Web search was used for model: {}", model);
            }
        }

        Ok(chat_response)
    }

    fn get_error_mapper(&self) -> Self::ErrorMapper {
        crate::core::traits::error_mapper::DefaultErrorMapper
    }

    async fn chat_completion(
        &self,
        request: ChatRequest,
        _context: RequestContext,
    ) -> Result<ChatResponse, Self::Error> {
        debug!("xAI chat request: model={}", request.model);

        // Transform and execute
        let mut request_json = serde_json::to_value(&request)
            .map_err(|e| XAIError::invalid_request("xai", e.to_string()))?;

        // Add web search parameter at the top level if enabled
        if self.config.enable_web_search {
            if let Some(obj) = request_json.as_object_mut() {
                obj.insert("web_search".to_string(), serde_json::json!(true));
            }
        }

        let response = self
            .execute_request("/chat/completions", request_json)
            .await?;

        // Extract reasoning tokens if present
        let reasoning_tokens = self.extract_reasoning_tokens(&response);

        // Parse response
        let mut chat_response: ChatResponse =
            serde_json::from_value(response.clone()).map_err(|e| {
                XAIError::api_error("xai", 500, format!("Failed to parse chat response: {}", e))
            })?;

        // Handle reasoning tokens in usage
        if let Some(reasoning_tokens) = reasoning_tokens {
            if let Some(ref mut usage) = chat_response.usage {
                // Ensure completion_tokens_details exists and has reasoning_tokens
                if usage.completion_tokens_details.is_none() {
                    usage.completion_tokens_details = Some(CompletionTokensDetails {
                        reasoning_tokens: Some(reasoning_tokens),
                        audio_tokens: None,
                    });
                } else if let Some(ref mut details) = usage.completion_tokens_details {
                    details.reasoning_tokens = Some(reasoning_tokens);
                }
            }
        }

        Ok(chat_response)
    }

    async fn chat_completion_stream(
        &self,
        mut request: ChatRequest,
        _context: RequestContext,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<ChatChunk, Self::Error>> + Send>>, Self::Error>
    {
        debug!("xAI streaming request: model={}", request.model);

        // Set streaming flag
        request.stream = true;

        // Get API configuration
        let api_key = self
            .config
            .get_api_key()
            .ok_or_else(|| XAIError::authentication("xai", "API key is required"))?;

        // We'll add web search to JSON request body below

        // Execute streaming request using reqwest directly for SSE
        let url = format!("{}/chat/completions", self.config.get_api_base());
        let client = reqwest::Client::new();

        let mut request_json = serde_json::to_value(&request)
            .map_err(|e| XAIError::invalid_request("xai", e.to_string()))?;

        // Add web search at top level
        if self.config.enable_web_search {
            if let Some(obj) = request_json.as_object_mut() {
                obj.insert("web_search".to_string(), serde_json::json!(true));
            }
        }

        let response = client
            .post(&url)
            .header("Authorization", format!("Bearer {}", api_key))
            .header("Content-Type", "application/json")
            .json(&request_json)
            .send()
            .await
            .map_err(|e| XAIError::network("xai", e.to_string()))?;

        // Check status
        if !response.status().is_success() {
            let status = response.status().as_u16();
            let body = response.text().await.ok();
            return Err(match status {
                400 => XAIError::invalid_request(
                    "xai",
                    body.unwrap_or_else(|| "Bad request".to_string()),
                ),
                401 => XAIError::authentication("xai", "Invalid API key"),
                429 => XAIError::rate_limit("xai", None),
                _ => {
                    XAIError::api_error("xai", status, format!("Stream request failed: {}", status))
                }
            });
        }

        // TODO: Implement proper SSE streaming for xAI
        // For now, return an error as streaming implementation needs more work
        Err(XAIError::not_supported(
            "xai",
            "Streaming is not yet fully implemented for xAI provider",
        ))
    }

    async fn embeddings(
        &self,
        _request: EmbeddingRequest,
        _context: RequestContext,
    ) -> Result<EmbeddingResponse, Self::Error> {
        Err(XAIError::not_supported(
            "xai",
            "xAI does not currently support embeddings. Use chat models instead.",
        ))
    }

    async fn health_check(&self) -> HealthStatus {
        // Simple health check - try to get models list
        let url = format!("{}/models", self.config.get_api_base());
        let mut headers = Vec::with_capacity(1);
        if let Some(api_key) = self.config.get_api_key() {
            headers.push(header("Authorization", format!("Bearer {}", api_key)));
        }

        match self
            .pool_manager
            .execute_request(&url, HttpMethod::GET, headers, None::<serde_json::Value>)
            .await
        {
            Ok(_) => HealthStatus::Healthy,
            Err(_) => HealthStatus::Unhealthy,
        }
    }

    async fn calculate_cost(
        &self,
        model: &str,
        input_tokens: u32,
        output_tokens: u32,
    ) -> Result<f64, Self::Error> {
        // For now, calculate without reasoning tokens
        // In production, reasoning tokens would be extracted from the actual response
        calculate_cost_with_reasoning(model, input_tokens, output_tokens, None)
            .ok_or_else(|| XAIError::model_not_found("xai", format!("Unknown model: {}", model)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_provider_creation() {
        // Test with API key
        let config = XAIConfig {
            api_key: Some("test_key".to_string()),
            ..Default::default()
        };

        let provider = XAIProvider::new(config).await;
        assert!(provider.is_ok());

        let provider = provider.unwrap();
        assert_eq!(provider.name(), "xai");
        assert!(!provider.models().is_empty());
    }

    #[tokio::test]
    async fn test_provider_without_api_key() {
        let config = XAIConfig {
            api_key: None,
            ..Default::default()
        };

        let provider = XAIProvider::new(config).await;
        assert!(provider.is_err());
        if let Err(e) = provider {
            assert!(matches!(e, XAIError::Configuration { .. }));
        }
    }

    #[test]
    fn test_capabilities() {
        assert!(XAI_CAPABILITIES.contains(&ProviderCapability::ChatCompletion));
        assert!(XAI_CAPABILITIES.contains(&ProviderCapability::ChatCompletionStream));
        assert!(XAI_CAPABILITIES.contains(&ProviderCapability::ToolCalling));
    }

    #[test]
    fn test_supported_openai_params() {
        let config = XAIConfig {
            api_key: Some("test_key".to_string()),
            ..Default::default()
        };

        // We need to use block_on to test sync methods that require an async context
        let runtime = tokio::runtime::Runtime::new().unwrap();
        let provider = runtime.block_on(XAIProvider::new(config)).unwrap();

        // Test reasoning model
        let params = provider.get_supported_openai_params("grok-2");
        assert!(params.contains(&"reasoning_effort"));
        assert!(params.contains(&"temperature"));

        // Test non-reasoning model
        let params = provider.get_supported_openai_params("grok-2-mini");
        assert!(!params.contains(&"reasoning_effort"));
        assert!(params.contains(&"temperature"));
    }

    #[test]
    fn test_add_web_search_to_json() {
        let config = XAIConfig {
            api_key: Some("test_key".to_string()),
            enable_web_search: true,
            ..Default::default()
        };

        let runtime = tokio::runtime::Runtime::new().unwrap();
        let provider = runtime.block_on(XAIProvider::new(config)).unwrap();

        let mut json = serde_json::json!({
            "model": "grok-2",
            "messages": []
        });

        provider.add_web_search_to_json(&mut json);
        assert_eq!(json.get("web_search"), Some(&serde_json::json!(true)));
    }

    #[test]
    fn test_extract_reasoning_tokens() {
        let config = XAIConfig {
            api_key: Some("test_key".to_string()),
            ..Default::default()
        };

        let runtime = tokio::runtime::Runtime::new().unwrap();
        let provider = runtime.block_on(XAIProvider::new(config)).unwrap();

        let response = serde_json::json!({
            "usage": {
                "completion_tokens_details": {
                    "reasoning_tokens": 150
                }
            }
        });

        let tokens = provider.extract_reasoning_tokens(&response);
        assert_eq!(tokens, Some(150));

        // Test without reasoning tokens
        let response = serde_json::json!({
            "usage": {
                "completion_tokens": 100
            }
        });

        let tokens = provider.extract_reasoning_tokens(&response);
        assert_eq!(tokens, None);
    }
}
