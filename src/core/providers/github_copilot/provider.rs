//! Main GitHub Copilot Provider Implementation
//!
//! Implements the LLMProvider trait for GitHub Copilot API.
//! Handles OAuth authentication and OpenAI-compatible chat completions.

use bytes::Bytes;
use futures::Stream;
use std::collections::HashMap;
use std::pin::Pin;
use std::sync::Arc;
use tokio::sync::RwLock;

use super::authenticator::CopilotAuthenticator;
use super::config::{GITHUB_COPILOT_API_BASE, GitHubCopilotConfig, get_copilot_default_headers};
use super::model_info::{get_available_models, get_model_info};
use crate::ProviderError;
use crate::core::streaming::utils::is_done_marker;
use crate::core::traits::provider::llm_provider::trait_definition::LLMProvider;
use crate::core::types::{
    chat::ChatMessage, message::MessageRole, model::ModelInfo, model::ProviderCapability,
    responses::ChatChunk,
};

/// Static capabilities for GitHub Copilot provider
const GITHUB_COPILOT_CAPABILITIES: &[ProviderCapability] = &[
    ProviderCapability::ChatCompletion,
    ProviderCapability::ChatCompletionStream,
    ProviderCapability::ToolCalling,
];

/// GitHub Copilot provider implementation
#[derive(Debug)]
pub struct GitHubCopilotProvider {
    config: GitHubCopilotConfig,
    authenticator: CopilotAuthenticator,
    models: Vec<ModelInfo>,
    /// Cached API key
    cached_api_key: Arc<RwLock<Option<String>>>,
    /// Cached API base
    cached_api_base: Arc<RwLock<Option<String>>>,
}

impl Clone for GitHubCopilotProvider {
    fn clone(&self) -> Self {
        Self {
            config: self.config.clone(),
            authenticator: self.authenticator.clone(),
            models: self.models.clone(),
            cached_api_key: Arc::new(RwLock::new(None)),
            cached_api_base: Arc::new(RwLock::new(None)),
        }
    }
}

impl GitHubCopilotProvider {
    /// Create a new GitHub Copilot provider instance
    pub async fn new(config: GitHubCopilotConfig) -> Result<Self, ProviderError> {
        let authenticator = CopilotAuthenticator::new(&config);

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
                    provider: "github_copilot".to_string(),
                    max_context_length: info.max_context_length,
                    max_output_length: Some(info.max_output_length),
                    supports_streaming: info.supports_streaming,
                    supports_tools: info.supports_tools,
                    supports_multimodal: info.supports_multimodal,
                    input_cost_per_1k_tokens: None, // Copilot is subscription-based
                    output_cost_per_1k_tokens: None,
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
            authenticator,
            models,
            cached_api_key: Arc::new(RwLock::new(None)),
            cached_api_base: Arc::new(RwLock::new(None)),
        })
    }

    /// Get the API key, using cache or refreshing if needed
    async fn get_api_key(&self) -> Result<String, ProviderError> {
        // Check cache first
        {
            let cache = self.cached_api_key.read().await;
            if let Some(ref key) = *cache {
                return Ok(key.clone());
            }
        }

        // Get fresh key
        let key = self.authenticator.get_api_key().await?;

        // Update cache
        {
            let mut cache = self.cached_api_key.write().await;
            *cache = Some(key.clone());
        }

        // Also update API base cache
        if let Some(api_base) = self.authenticator.get_api_base() {
            let mut cache = self.cached_api_base.write().await;
            *cache = Some(api_base);
        }

        Ok(key)
    }

    /// Get the API base URL
    async fn get_api_base(&self) -> String {
        // Check cache first
        {
            let cache = self.cached_api_base.read().await;
            if let Some(ref base) = *cache {
                return base.clone();
            }
        }

        // Use config or authenticator
        self.config
            .api_base
            .clone()
            .or_else(|| self.authenticator.get_api_base())
            .unwrap_or_else(|| GITHUB_COPILOT_API_BASE.to_string())
    }

    /// Clear cached credentials (for refresh)
    async fn clear_cache(&self) {
        {
            let mut cache = self.cached_api_key.write().await;
            *cache = None;
        }
        {
            let mut cache = self.cached_api_base.write().await;
            *cache = None;
        }
    }

    /// Transform messages for Copilot API
    fn transform_messages(&self, messages: &mut [ChatMessage]) {
        if self.config.disable_system_to_assistant {
            return;
        }

        // Convert system messages to assistant messages (Copilot requirement)
        for message in messages.iter_mut() {
            if message.role == MessageRole::System {
                message.role = MessageRole::Assistant;
            }
        }
    }

    /// Determine X-Initiator header value
    fn determine_initiator(&self, messages: &[ChatMessage]) -> &'static str {
        for message in messages {
            if message.role == MessageRole::Tool || message.role == MessageRole::Assistant {
                return "agent";
            }
        }
        "user"
    }

    /// Check if request contains vision content
    fn has_vision_content(&self, messages: &[ChatMessage]) -> bool {
        for message in messages {
            if let Some(crate::core::types::message::MessageContent::Parts(parts)) =
                &message.content
            {
                for part in parts {
                    if let crate::core::types::content::ContentPart::ImageUrl { .. } = part {
                        return true;
                    }
                }
            }
        }
        false
    }

    /// Build request headers
    async fn build_headers(
        &self,
        messages: &[ChatMessage],
    ) -> Result<reqwest::header::HeaderMap, ProviderError> {
        let api_key = self.get_api_key().await?;
        let default_headers = get_copilot_default_headers(&api_key);

        let mut headers = reqwest::header::HeaderMap::new();
        for (key, value) in default_headers {
            headers.insert(
                reqwest::header::HeaderName::from_bytes(key.as_bytes()).map_err(|e| {
                    ProviderError::configuration(
                        "github_copilot",
                        format!("Invalid header name: {}", e),
                    )
                })?,
                value.parse().map_err(|e| {
                    ProviderError::configuration(
                        "github_copilot",
                        format!("Invalid header value: {}", e),
                    )
                })?,
            );
        }

        // Add X-Initiator header
        let initiator = self.determine_initiator(messages);
        headers.insert("x-initiator", initiator.parse().unwrap());

        // Add Copilot-Vision-Request if contains images
        if self.has_vision_content(messages) {
            headers.insert("copilot-vision-request", "true".parse().unwrap());
        }

        Ok(headers)
    }
}

/// SSE stream implementation for GitHub Copilot
pub struct GitHubCopilotStream {
    inner: Pin<Box<dyn Stream<Item = Result<Bytes, reqwest::Error>> + Send>>,
    buffer: String,
}

impl GitHubCopilotStream {
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

            if is_done_marker(data) {
                return None;
            }

            match serde_json::from_str::<ChatChunk>(data) {
                Ok(chunk) => Some(Ok(chunk)),
                Err(e) => Some(Err(ProviderError::api_error(
                    "github_copilot",
                    500,
                    format!("Failed to parse chunk: {}", e),
                ))),
            }
        } else {
            None
        }
    }
}

impl Stream for GitHubCopilotStream {
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
                        "github_copilot",
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
