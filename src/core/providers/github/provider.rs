//! Main GitHub Models Provider Implementation
//!
//! Implements the LLMProvider trait for GitHub Models API.
//! The API is OpenAI-compatible, making the implementation straightforward.

use futures::Stream;
use std::collections::HashMap;
use std::pin::Pin;
use std::sync::Arc;

use super::config::GitHubConfig;
use super::model_info::{get_available_models, get_model_info};
use crate::core::providers::base::{GlobalPoolManager, HttpErrorMapper, HttpMethod, header};
use crate::core::providers::unified_provider::ProviderError;
use crate::core::traits::{
    provider::ProviderConfig as _, provider::llm_provider::trait_definition::LLMProvider,
};
use crate::core::types::{model::ModelInfo, model::ProviderCapability, responses::ChatChunk};

/// Static capabilities for GitHub Models provider
const GITHUB_CAPABILITIES: &[ProviderCapability] = &[
    ProviderCapability::ChatCompletion,
    ProviderCapability::ChatCompletionStream,
    ProviderCapability::ToolCalling,
];

/// GitHub Models provider implementation
#[derive(Debug, Clone)]
pub struct GitHubProvider {
    config: GitHubConfig,
    pool_manager: Arc<GlobalPoolManager>,
    models: Vec<ModelInfo>,
}

impl GitHubProvider {
    /// Create a new GitHub provider instance
    pub async fn new(config: GitHubConfig) -> Result<Self, ProviderError> {
        // Validate configuration
        config
            .validate()
            .map_err(|e| ProviderError::configuration("github", e))?;

        // Create pool manager
        let pool_manager = Arc::new(GlobalPoolManager::new().map_err(|e| {
            ProviderError::configuration("github", format!("Failed to create pool manager: {}", e))
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
                    provider: "github".to_string(),
                    max_context_length: info.max_context_length,
                    max_output_length: Some(info.max_output_length),
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
        let config = GitHubConfig {
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
            .map_err(|e| ProviderError::network("github", e.to_string()))?;

        let status = response.status();
        let response_bytes = response
            .bytes()
            .await
            .map_err(|e| ProviderError::network("github", e.to_string()))?;

        if !status.is_success() {
            let body_str = String::from_utf8_lossy(&response_bytes);
            let status_code = status.as_u16();
            return Err(match status_code {
                401 => ProviderError::authentication("github", "Invalid API key"),
                404 => ProviderError::model_not_found("github", body_str.to_string()),
                429 => ProviderError::rate_limit("github", None),
                400 => ProviderError::invalid_request("github", body_str.to_string()),
                500..=599 => ProviderError::provider_unavailable("github", body_str.to_string()),
                _ => HttpErrorMapper::map_status_code("github", status_code, &body_str),
            });
        }

        serde_json::from_slice(&response_bytes).map_err(|e| {
            ProviderError::api_error("github", 500, format!("Failed to parse response: {}", e))
        })
    }
}

/// SSE stream implementation for GitHub Models
use bytes::Bytes;

pub struct GitHubStream {
    inner: Pin<Box<dyn Stream<Item = Result<Bytes, reqwest::Error>> + Send>>,
    buffer: String,
}

impl GitHubStream {
    pub fn new(stream: impl Stream<Item = Result<Bytes, reqwest::Error>> + Send + 'static) -> Self {
        Self {
            inner: Box::pin(stream),
            buffer: String::new(),
        }
    }

    fn parse_sse_line(&self, line: &str) -> Option<Result<ChatChunk, ProviderError>> {
        if line.is_empty() || line.starts_with(':') {
            return None;
        }

        if let Some(data) = line.strip_prefix("data: ") {
            let data = data.trim();

            if data == "[DONE]" {
                return None;
            }

            match serde_json::from_str::<ChatChunk>(data) {
                Ok(chunk) => Some(Ok(chunk)),
                Err(e) => Some(Err(ProviderError::api_error(
                    "github",
                    500,
                    format!("Failed to parse chunk: {}", e),
                ))),
            }
        } else {
            None
        }
    }
}

impl Stream for GitHubStream {
    type Item = Result<ChatChunk, ProviderError>;

    fn poll_next(
        mut self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        loop {
            // Check if we have complete lines in the buffer
            if let Some(newline_pos) = self.buffer.find('\n') {
                let line = self.buffer[..newline_pos].to_string();
                self.buffer = self.buffer[newline_pos + 1..].to_string();

                if let Some(result) = self.parse_sse_line(&line) {
                    return std::task::Poll::Ready(Some(result));
                }
                continue;
            }

            // Need more data
            match self.inner.as_mut().poll_next(cx) {
                std::task::Poll::Ready(Some(Ok(bytes))) => {
                    self.buffer.push_str(&String::from_utf8_lossy(&bytes));
                }
                std::task::Poll::Ready(Some(Err(e))) => {
                    return std::task::Poll::Ready(Some(Err(ProviderError::network(
                        "github",
                        e.to_string(),
                    ))));
                }
                std::task::Poll::Ready(None) => {
                    // Stream ended, check remaining buffer
                    if !self.buffer.is_empty() {
                        let line = std::mem::take(&mut self.buffer);
                        if let Some(result) = self.parse_sse_line(&line) {
                            return std::task::Poll::Ready(Some(result));
                        }
                    }
                    return std::task::Poll::Ready(None);
                }
                std::task::Poll::Pending => return std::task::Poll::Pending,
            }
        }
    }
}
