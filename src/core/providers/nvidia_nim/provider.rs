//! Main NVIDIA NIM Provider Implementation
//!
//! Implements the LLMProvider trait for NVIDIA NIM's inference microservices.

use futures::Stream;
use std::collections::HashMap;
use std::pin::Pin;
use std::sync::Arc;

use super::config::NvidiaNimConfig;
use super::model_info::{get_available_models, get_model_info, get_supported_params};
use crate::core::providers::base::{GlobalPoolManager, HttpErrorMapper, HttpMethod, header};
use crate::core::providers::unified_provider::ProviderError;
use crate::core::traits::{
    provider::ProviderConfig as _, provider::llm_provider::trait_definition::LLMProvider,
};
use crate::core::types::{model::ModelInfo, model::ProviderCapability, responses::ChatChunk};

/// Static capabilities for NVIDIA NIM provider
const NVIDIA_NIM_CAPABILITIES: &[ProviderCapability] = &[
    ProviderCapability::ChatCompletion,
    ProviderCapability::ChatCompletionStream,
    ProviderCapability::ToolCalling,
    ProviderCapability::Embeddings,
];

/// NVIDIA NIM provider implementation
#[derive(Debug, Clone)]
pub struct NvidiaNimProvider {
    config: NvidiaNimConfig,
    pool_manager: Arc<GlobalPoolManager>,
    models: Vec<ModelInfo>,
}

impl NvidiaNimProvider {
    /// Create a new NVIDIA NIM provider instance
    pub async fn new(config: NvidiaNimConfig) -> Result<Self, ProviderError> {
        // Validate configuration
        config
            .validate()
            .map_err(|e| ProviderError::configuration("nvidia_nim", e))?;

        // Create pool manager
        let pool_manager = Arc::new(GlobalPoolManager::new().map_err(|e| {
            ProviderError::configuration(
                "nvidia_nim",
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
                    provider: "nvidia_nim".to_string(),
                    max_context_length: info.max_context_length as u32,
                    max_output_length: Some(info.max_output_length as u32),
                    supports_streaming: info.supports_streaming,
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
        let config = NvidiaNimConfig::from_env().with_api_key(api_key.into());
        Self::new(config).await
    }

    /// Execute an HTTP request to NVIDIA NIM API
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
            .map_err(|e| ProviderError::network("nvidia_nim", e.to_string()))?;

        // Check status
        if !response.status().is_success() {
            let status = response.status().as_u16();
            let body_text = response.text().await.unwrap_or_default();
            return Err(match status {
                400 => ProviderError::invalid_request("nvidia_nim", body_text),
                401 => ProviderError::authentication("nvidia_nim", "Invalid API key"),
                404 => ProviderError::model_not_found("nvidia_nim", "Model not found"),
                429 => ProviderError::rate_limit_simple("nvidia_nim", "Rate limit exceeded"),
                _ => HttpErrorMapper::map_status_code("nvidia_nim", status, &body_text),
            });
        }

        let response_bytes = response
            .bytes()
            .await
            .map_err(|e| ProviderError::network("nvidia_nim", e.to_string()))?;

        serde_json::from_slice(&response_bytes).map_err(|e| {
            ProviderError::response_parsing(
                "nvidia_nim",
                format!("Failed to parse response: {}", e),
            )
        })
    }

    /// Map OpenAI parameters to NVIDIA NIM format
    fn map_params(&self, params: &mut serde_json::Value, model: &str) {
        let supported = get_supported_params(model);

        // Filter out unsupported parameters
        if let Some(obj) = params.as_object_mut() {
            let keys_to_remove: Vec<String> = obj
                .keys()
                .filter(|k| !supported.contains(&k.as_str()) && *k != "messages" && *k != "model")
                .cloned()
                .collect();

            for key in keys_to_remove {
                obj.remove(&key);
            }

            // Map max_completion_tokens to max_tokens if present
            if let Some(max_completion) = obj.remove("max_completion_tokens")
                && !obj.contains_key("max_tokens")
            {
                obj.insert("max_tokens".to_string(), max_completion);
            }
        }
    }
}

// ==================== Streaming Support ====================

use bytes::Bytes;

/// NVIDIA NIM streaming response parser
pub struct NvidiaNimStream {
    inner: Pin<Box<dyn Stream<Item = Result<Bytes, reqwest::Error>> + Send>>,
    buffer: String,
}

impl NvidiaNimStream {
    pub fn new(stream: impl Stream<Item = Result<Bytes, reqwest::Error>> + Send + 'static) -> Self {
        Self {
            inner: Box::pin(stream),
            buffer: String::new(),
        }
    }
}

impl Stream for NvidiaNimStream {
    type Item = Result<ChatChunk, ProviderError>;

    fn poll_next(
        mut self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        loop {
            // Try to parse a complete SSE message from buffer
            if let Some(pos) = self.buffer.find("\n\n") {
                let message = self.buffer[..pos].to_string();
                self.buffer = self.buffer[pos + 2..].to_string();

                // Parse SSE message
                for line in message.lines() {
                    if let Some(data) = line.strip_prefix("data: ") {
                        if data == "[DONE]" {
                            return std::task::Poll::Ready(None);
                        }

                        match serde_json::from_str::<ChatChunk>(data) {
                            Ok(chunk) => return std::task::Poll::Ready(Some(Ok(chunk))),
                            Err(e) => {
                                return std::task::Poll::Ready(Some(Err(
                                    ProviderError::api_error(
                                        "nvidia_nim",
                                        500,
                                        format!("Failed to parse chunk: {}", e),
                                    ),
                                )));
                            }
                        }
                    }
                }
            }

            // Need more data
            match self.inner.as_mut().poll_next(cx) {
                std::task::Poll::Ready(Some(Ok(bytes))) => {
                    if let Ok(text) = String::from_utf8(bytes.to_vec()) {
                        self.buffer.push_str(&text);
                    }
                }
                std::task::Poll::Ready(Some(Err(e))) => {
                    return std::task::Poll::Ready(Some(Err(ProviderError::network(
                        "nvidia_nim",
                        e.to_string(),
                    ))));
                }
                std::task::Poll::Ready(None) => {
                    return std::task::Poll::Ready(None);
                }
                std::task::Poll::Pending => {
                    return std::task::Poll::Pending;
                }
            }
        }
    }
}
