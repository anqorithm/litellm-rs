//! Shared utilities for all providers
//!
//! This module contains common functionality that can be reused across all providers,
//! following the DRY principle and Rust's composition over inheritance pattern.

use reqwest::{Client, Response, StatusCode};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::time::Duration;
use tracing::warn;

use crate::core::providers::unified_provider::ProviderError;
use crate::core::types::responses::{FinishReason, Usage};
use crate::core::types::{message::MessageContent, message::MessageRole};

// ============================================================================
// HTTP Client Builder
// ============================================================================

/// Shared HTTP client builder with common configuration
pub struct HttpClientBuilder {
    timeout: Duration,
    max_retries: u32,
    default_headers: HashMap<String, String>,
}

impl Default for HttpClientBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl HttpClientBuilder {
    pub fn new() -> Self {
        Self {
            timeout: Duration::from_secs(60),
            max_retries: 3,
            default_headers: HashMap::new(),
        }
    }

    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    pub fn max_retries(mut self, max_retries: u32) -> Self {
        self.max_retries = max_retries;
        self
    }

    pub fn default_header(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.default_headers.insert(key.into(), value.into());
        self
    }

    pub fn build(self) -> Result<(Client, RetryConfig), ProviderError> {
        let mut builder = Client::builder().timeout(self.timeout);

        // Build default headers if any
        if !self.default_headers.is_empty() {
            use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
            let mut headers = HeaderMap::new();
            for (key, value) in &self.default_headers {
                let header_name = HeaderName::from_bytes(key.as_bytes()).map_err(|_| {
                    ProviderError::Configuration {
                        provider: "shared",
                        message: format!("Invalid header name: {}", key),
                    }
                })?;
                let header_value =
                    HeaderValue::from_str(value).map_err(|_| ProviderError::Configuration {
                        provider: "shared",
                        message: format!("Invalid header value: {}", value),
                    })?;
                headers.insert(header_name, header_value);
            }
            builder = builder.default_headers(headers);
        }

        let client = builder.build().map_err(|e| ProviderError::Configuration {
            provider: "shared",
            message: format!("Failed to build HTTP client: {}", e),
        })?;

        let retry_config = RetryConfig {
            max_retries: self.max_retries,
            initial_delay: Duration::from_secs(1),
            max_delay: Duration::from_secs(60),
            exponential_base: 2,
        };

        Ok((client, retry_config))
    }
}

// ============================================================================
// Retry Configuration
// ============================================================================

#[derive(Debug, Clone)]
pub struct RetryConfig {
    pub max_retries: u32,
    pub initial_delay: Duration,
    pub max_delay: Duration,
    pub exponential_base: u32,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_retries: 3,
            initial_delay: Duration::from_secs(1),
            max_delay: Duration::from_secs(60),
            exponential_base: 2,
        }
    }
}

// ============================================================================
// Request Executor with Retry Logic
// ============================================================================

pub struct RequestExecutor {
    _client: Client,
    retry_config: RetryConfig,
}

impl RequestExecutor {
    pub fn new(client: Client, retry_config: RetryConfig) -> Self {
        Self {
            _client: client,
            retry_config,
        }
    }

    /// Execute a request with automatic retry logic
    pub async fn execute<F, Fut>(
        &self,
        provider_name: &'static str,
        mut request_fn: F,
    ) -> Result<Response, ProviderError>
    where
        F: FnMut() -> Fut,
        Fut: std::future::Future<Output = Result<Response, reqwest::Error>>,
    {
        let mut retries = 0;
        let mut delay = self.retry_config.initial_delay;

        loop {
            match request_fn().await {
                Ok(response) => {
                    if response.status().is_success() {
                        return Ok(response);
                    }

                    // Handle specific error status codes
                    let status = response.status();
                    let should_retry = matches!(status.as_u16(), 429 | 500 | 502 | 503 | 504);

                    if should_retry && retries < self.retry_config.max_retries {
                        retries += 1;
                        warn!(
                            "Provider {} returned status {}, retrying ({}/{})",
                            provider_name, status, retries, self.retry_config.max_retries
                        );
                        tokio::time::sleep(delay).await;
                        delay = std::cmp::min(
                            delay * self.retry_config.exponential_base,
                            self.retry_config.max_delay,
                        );
                        continue;
                    }

                    // Convert to appropriate error
                    return Err(self.status_to_error(provider_name, status, response).await);
                }
                Err(e) if retries < self.retry_config.max_retries => {
                    retries += 1;
                    warn!(
                        "Provider {} request failed: {}, retrying ({}/{})",
                        provider_name, e, retries, self.retry_config.max_retries
                    );
                    tokio::time::sleep(delay).await;
                    delay = std::cmp::min(
                        delay * self.retry_config.exponential_base,
                        self.retry_config.max_delay,
                    );
                }
                Err(e) => {
                    return Err(ProviderError::Network {
                        provider: provider_name,
                        message: format!("Request failed after {} retries: {}", retries, e),
                    });
                }
            }
        }
    }

    async fn status_to_error(
        &self,
        provider: &'static str,
        status: StatusCode,
        response: Response,
    ) -> ProviderError {
        let error_text = response.text().await.unwrap_or_default();

        match status.as_u16() {
            401 => ProviderError::Authentication {
                provider,
                message: format!("Authentication failed: {}", error_text),
            },
            402 => ProviderError::QuotaExceeded {
                provider,
                message: format!("Quota exceeded: {}", error_text),
            },
            403 => ProviderError::InvalidRequest {
                provider,
                message: format!("Authorization failed: {}", error_text),
            },
            404 => ProviderError::ModelNotFound {
                provider,
                model: error_text,
            },
            429 => ProviderError::rate_limit_simple(
                provider,
                format!("Rate limit exceeded: {}", error_text),
            ),
            500..=599 => ProviderError::ProviderUnavailable {
                provider,
                message: format!("Service error {}: {}", status, error_text),
            },
            _ => ProviderError::Other {
                provider,
                message: format!("Unexpected status {}: {}", status, error_text),
            },
        }
    }
}

// ============================================================================
// Message Transformation Utilities
// ============================================================================

pub struct MessageTransformer;

impl MessageTransformer {
    /// Convert role to OpenAI-compatible string
    pub fn role_to_string(role: &MessageRole) -> &'static str {
        match role {
            MessageRole::System => "system",
            MessageRole::User => "user",
            MessageRole::Assistant => "assistant",
            MessageRole::Tool => "tool",
            MessageRole::Function => "function",
        }
    }

    /// Parse string to MessageRole
    pub fn string_to_role(role: &str) -> MessageRole {
        match role {
            "system" => MessageRole::System,
            "user" => MessageRole::User,
            "assistant" => MessageRole::Assistant,
            "tool" => MessageRole::Tool,
            "function" => MessageRole::Function,
            _ => MessageRole::User,
        }
    }

    /// Convert MessageContent to JSON Value
    pub fn content_to_value(content: &Option<MessageContent>) -> Value {
        match content {
            Some(MessageContent::Text(text)) => Value::String(text.clone()),
            Some(MessageContent::Parts(parts)) => {
                serde_json::to_value(parts).unwrap_or(Value::Null)
            }
            None => Value::Null,
        }
    }

    /// Parse finish reason string
    pub fn parse_finish_reason(reason: &str) -> Option<FinishReason> {
        match reason {
            "stop" => Some(FinishReason::Stop),
            "length" | "max_tokens" => Some(FinishReason::Length),
            "tool_calls" | "function_call" => Some(FinishReason::ToolCalls),
            "content_filter" => Some(FinishReason::ContentFilter),
            _ => None,
        }
    }
}

// ============================================================================
// Common Request/Response Types
// ============================================================================

/// Common configuration shared by most providers
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CommonProviderConfig {
    pub api_key: String,
    pub base_url: String,
    pub timeout: u64,
    pub max_retries: u32,
    #[serde(default)]
    pub custom_headers: HashMap<String, String>,
}

impl Default for CommonProviderConfig {
    fn default() -> Self {
        Self {
            api_key: String::new(),
            base_url: String::new(),
            timeout: 60,
            max_retries: 3,
            custom_headers: HashMap::new(),
        }
    }
}

// ============================================================================
// Rate Limiting
// ============================================================================

use std::sync::Arc;
use tokio::sync::Semaphore;

/// Rate limiter for providers
pub struct RateLimiter {
    semaphore: Arc<Semaphore>,
    _requests_per_second: u32,
}

impl RateLimiter {
    pub fn new(requests_per_second: u32) -> Self {
        Self {
            semaphore: Arc::new(Semaphore::new(requests_per_second as usize)),
            _requests_per_second: requests_per_second,
        }
    }

    pub async fn acquire(&self) -> Result<tokio::sync::SemaphorePermit<'_>, ProviderError> {
        self.semaphore
            .acquire()
            .await
            .map_err(|_| ProviderError::Other {
                provider: "rate_limiter",
                message: "Failed to acquire rate limit permit".to_string(),
            })
    }

    pub fn available_permits(&self) -> usize {
        self.semaphore.available_permits()
    }
}

// ============================================================================
// Response Validation
// ============================================================================

pub struct ResponseValidator;

impl ResponseValidator {
    /// Validate that a response has required fields
    pub fn validate_chat_response(
        response: &Value,
        provider: &'static str,
    ) -> Result<(), ProviderError> {
        if !response.is_object() {
            return Err(ProviderError::ResponseParsing {
                provider,
                message: "Response is not an object".to_string(),
            });
        }

        // Check for required fields
        let required_fields = ["id", "choices", "created", "model"];
        for field in &required_fields {
            if response.get(field).is_none() {
                return Err(ProviderError::ResponseParsing {
                    provider,
                    message: format!("Missing required field: {}", field),
                });
            }
        }

        // Validate choices array
        if let Some(choices) = response.get("choices") {
            let Some(choices_array) = choices.as_array() else {
                return Err(ProviderError::ResponseParsing {
                    provider,
                    message: "Choices must be a non-empty array".to_string(),
                });
            };

            if choices_array.is_empty() {
                return Err(ProviderError::ResponseParsing {
                    provider,
                    message: "Choices must be a non-empty array".to_string(),
                });
            }
        }

        Ok(())
    }
}

// ============================================================================
// Cost Calculation Utilities
// ============================================================================

#[derive(Debug, Clone)]
pub struct TokenCostCalculator {
    input_cost_per_1k: f64,
    output_cost_per_1k: f64,
}

impl TokenCostCalculator {
    pub fn new(input_cost_per_1k: f64, output_cost_per_1k: f64) -> Self {
        Self {
            input_cost_per_1k,
            output_cost_per_1k,
        }
    }

    pub fn calculate_cost(&self, usage: &Usage) -> f64 {
        let input_cost = (usage.prompt_tokens as f64 / 1000.0) * self.input_cost_per_1k;
        let output_cost = (usage.completion_tokens as f64 / 1000.0) * self.output_cost_per_1k;
        input_cost + output_cost
    }
}

// ============================================================================
// Testing Utilities
// ============================================================================

#[cfg(test)]
pub mod test_utils {
    use super::*;
    use crate::core::types::chat::ChatMessage;

    /// Create a mock ChatMessage for testing
    pub fn mock_message(role: MessageRole, content: &str) -> ChatMessage {
        ChatMessage {
            role,
            content: Some(MessageContent::Text(content.to_string())),
            ..Default::default()
        }
    }

    /// Create mock usage for testing
    pub fn mock_usage(prompt: u32, completion: u32) -> Usage {
        Usage {
            prompt_tokens: prompt,
            completion_tokens: completion,
            total_tokens: prompt + completion,
            completion_tokens_details: None,
            prompt_tokens_details: None,
            thinking_usage: None,
        }
    }
}

// ============================================================================
// Tests
// ============================================================================


#[cfg(test)]
#[path = "shared_tests.rs"]
mod tests;
