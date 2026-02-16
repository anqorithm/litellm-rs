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
    retry_config: RetryConfig,
}

impl RequestExecutor {
    pub fn new(client: Client, retry_config: RetryConfig) -> Self {
        let _ = client;
        Self { retry_config }
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
}

impl RateLimiter {
    pub fn new(requests_per_second: u32) -> Self {
        Self {
            semaphore: Arc::new(Semaphore::new(requests_per_second as usize)),
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
            if choices.as_array().is_none_or(|a| a.is_empty()) {
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
mod tests {
    use super::*;
    use crate::core::types::content::ContentPart;

    // ==================== HttpClientBuilder Tests ====================

    #[test]
    fn test_http_client_builder_default() {
        let builder = HttpClientBuilder::default();
        let result = builder.build();
        assert!(result.is_ok());

        let (_client, retry_config) = result.unwrap();
        // Client exists (built successfully)
        assert_eq!(retry_config.max_retries, 3);
    }

    #[test]
    fn test_http_client_builder_new() {
        let builder = HttpClientBuilder::new();
        let result = builder.build();
        assert!(result.is_ok());
    }

    #[test]
    fn test_http_client_builder_with_timeout() {
        let builder = HttpClientBuilder::new().timeout(Duration::from_secs(120));
        let result = builder.build();
        assert!(result.is_ok());
    }

    #[test]
    fn test_http_client_builder_with_max_retries() {
        let builder = HttpClientBuilder::new().max_retries(5);
        let result = builder.build();
        assert!(result.is_ok());

        let (_, retry_config) = result.unwrap();
        assert_eq!(retry_config.max_retries, 5);
    }

    #[test]
    fn test_http_client_builder_with_default_header() {
        let builder = HttpClientBuilder::new().default_header("X-Custom-Header", "test-value");
        let result = builder.build();
        assert!(result.is_ok());
    }

    #[test]
    fn test_http_client_builder_with_multiple_headers() {
        let builder = HttpClientBuilder::new()
            .default_header("X-Header-1", "value1")
            .default_header("X-Header-2", "value2")
            .default_header("Authorization", "Bearer token");
        let result = builder.build();
        assert!(result.is_ok());
    }

    #[test]
    fn test_http_client_builder_chained() {
        let builder = HttpClientBuilder::new()
            .timeout(Duration::from_secs(30))
            .max_retries(2)
            .default_header("Content-Type", "application/json");
        let result = builder.build();
        assert!(result.is_ok());

        let (_, retry_config) = result.unwrap();
        assert_eq!(retry_config.max_retries, 2);
    }

    // ==================== RetryConfig Tests ====================

    #[test]
    fn test_retry_config_default() {
        let config = RetryConfig::default();
        assert_eq!(config.max_retries, 3);
        assert_eq!(config.initial_delay, Duration::from_secs(1));
        assert_eq!(config.max_delay, Duration::from_secs(60));
        assert_eq!(config.exponential_base, 2);
    }

    #[test]
    fn test_retry_config_clone() {
        let config = RetryConfig {
            max_retries: 5,
            initial_delay: Duration::from_millis(500),
            max_delay: Duration::from_secs(30),
            exponential_base: 3,
        };
        let cloned = config.clone();

        assert_eq!(cloned.max_retries, 5);
        assert_eq!(cloned.initial_delay, Duration::from_millis(500));
        assert_eq!(cloned.max_delay, Duration::from_secs(30));
        assert_eq!(cloned.exponential_base, 3);
    }

    #[test]
    fn test_retry_config_debug() {
        let config = RetryConfig::default();
        let debug_str = format!("{:?}", config);
        assert!(debug_str.contains("RetryConfig"));
        assert!(debug_str.contains("max_retries"));
    }

    // ==================== RequestExecutor Tests ====================

    #[test]
    fn test_request_executor_new() {
        let (client, retry_config) = HttpClientBuilder::new().build().unwrap();
        let executor = RequestExecutor::new(client, retry_config);
        // Just verify it doesn't panic
        let _ = executor;
    }

    // ==================== MessageTransformer Tests ====================

    #[test]
    fn test_message_transformer_role_to_string_system() {
        assert_eq!(
            MessageTransformer::role_to_string(&MessageRole::System),
            "system"
        );
    }

    #[test]
    fn test_message_transformer_role_to_string_user() {
        assert_eq!(
            MessageTransformer::role_to_string(&MessageRole::User),
            "user"
        );
    }

    #[test]
    fn test_message_transformer_role_to_string_assistant() {
        assert_eq!(
            MessageTransformer::role_to_string(&MessageRole::Assistant),
            "assistant"
        );
    }

    #[test]
    fn test_message_transformer_role_to_string_tool() {
        assert_eq!(
            MessageTransformer::role_to_string(&MessageRole::Tool),
            "tool"
        );
    }

    #[test]
    fn test_message_transformer_role_to_string_function() {
        assert_eq!(
            MessageTransformer::role_to_string(&MessageRole::Function),
            "function"
        );
    }

    #[test]
    fn test_message_transformer_string_to_role_system() {
        assert_eq!(
            MessageTransformer::string_to_role("system"),
            MessageRole::System
        );
    }

    #[test]
    fn test_message_transformer_string_to_role_user() {
        assert_eq!(
            MessageTransformer::string_to_role("user"),
            MessageRole::User
        );
    }

    #[test]
    fn test_message_transformer_string_to_role_assistant() {
        assert_eq!(
            MessageTransformer::string_to_role("assistant"),
            MessageRole::Assistant
        );
    }

    #[test]
    fn test_message_transformer_string_to_role_tool() {
        assert_eq!(
            MessageTransformer::string_to_role("tool"),
            MessageRole::Tool
        );
    }

    #[test]
    fn test_message_transformer_string_to_role_function() {
        assert_eq!(
            MessageTransformer::string_to_role("function"),
            MessageRole::Function
        );
    }

    #[test]
    fn test_message_transformer_string_to_role_unknown() {
        assert_eq!(
            MessageTransformer::string_to_role("unknown"),
            MessageRole::User
        );
        assert_eq!(MessageTransformer::string_to_role(""), MessageRole::User);
    }

    #[test]
    fn test_message_transformer_content_to_value_text() {
        let content = Some(MessageContent::Text("Hello, world!".to_string()));
        let value = MessageTransformer::content_to_value(&content);
        assert_eq!(value, Value::String("Hello, world!".to_string()));
    }

    #[test]
    fn test_message_transformer_content_to_value_parts() {
        let content = Some(MessageContent::Parts(vec![
            ContentPart::Text {
                text: "Part 1".to_string(),
            },
            ContentPart::Text {
                text: "Part 2".to_string(),
            },
        ]));
        let value = MessageTransformer::content_to_value(&content);
        assert!(value.is_array());
    }

    #[test]
    fn test_message_transformer_content_to_value_none() {
        let content: Option<MessageContent> = None;
        let value = MessageTransformer::content_to_value(&content);
        assert!(value.is_null());
    }

    #[test]
    fn test_message_transformer_parse_finish_reason_stop() {
        assert_eq!(
            MessageTransformer::parse_finish_reason("stop"),
            Some(FinishReason::Stop)
        );
    }

    #[test]
    fn test_message_transformer_parse_finish_reason_length() {
        assert_eq!(
            MessageTransformer::parse_finish_reason("length"),
            Some(FinishReason::Length)
        );
        assert_eq!(
            MessageTransformer::parse_finish_reason("max_tokens"),
            Some(FinishReason::Length)
        );
    }

    #[test]
    fn test_message_transformer_parse_finish_reason_tool_calls() {
        assert_eq!(
            MessageTransformer::parse_finish_reason("tool_calls"),
            Some(FinishReason::ToolCalls)
        );
        assert_eq!(
            MessageTransformer::parse_finish_reason("function_call"),
            Some(FinishReason::ToolCalls)
        );
    }

    #[test]
    fn test_message_transformer_parse_finish_reason_content_filter() {
        assert_eq!(
            MessageTransformer::parse_finish_reason("content_filter"),
            Some(FinishReason::ContentFilter)
        );
    }

    #[test]
    fn test_message_transformer_parse_finish_reason_unknown() {
        assert_eq!(MessageTransformer::parse_finish_reason("unknown"), None);
        assert_eq!(MessageTransformer::parse_finish_reason(""), None);
    }

    // ==================== CommonProviderConfig Tests ====================

    #[test]
    fn test_common_provider_config_default() {
        let config = CommonProviderConfig::default();
        assert_eq!(config.api_key, "");
        assert_eq!(config.base_url, "");
        assert_eq!(config.timeout, 60);
        assert_eq!(config.max_retries, 3);
        assert!(config.custom_headers.is_empty());
    }

    #[test]
    fn test_common_provider_config_with_values() {
        let config = CommonProviderConfig {
            api_key: "test-api-key".to_string(),
            base_url: "https://api.example.com".to_string(),
            timeout: 120,
            max_retries: 5,
            custom_headers: HashMap::from([("X-Custom".to_string(), "value".to_string())]),
        };

        assert_eq!(config.api_key, "test-api-key");
        assert_eq!(config.base_url, "https://api.example.com");
        assert_eq!(config.timeout, 120);
        assert_eq!(config.max_retries, 5);
        assert_eq!(config.custom_headers.len(), 1);
    }

    #[test]
    fn test_common_provider_config_serialization() {
        let config = CommonProviderConfig {
            api_key: "key123".to_string(),
            base_url: "https://api.test.com".to_string(),
            timeout: 30,
            max_retries: 2,
            custom_headers: HashMap::new(),
        };

        let json = serde_json::to_value(&config).unwrap();
        assert_eq!(json["api_key"], "key123");
        assert_eq!(json["base_url"], "https://api.test.com");
        assert_eq!(json["timeout"], 30);
        assert_eq!(json["max_retries"], 2);
    }

    #[test]
    fn test_common_provider_config_deserialization() {
        let json = r#"{
            "api_key": "my-key",
            "base_url": "https://example.com",
            "timeout": 45,
            "max_retries": 4
        }"#;

        let config: CommonProviderConfig = serde_json::from_str(json).unwrap();
        assert_eq!(config.api_key, "my-key");
        assert_eq!(config.base_url, "https://example.com");
        assert_eq!(config.timeout, 45);
        assert_eq!(config.max_retries, 4);
    }

    #[test]
    fn test_common_provider_config_clone() {
        let config = CommonProviderConfig {
            api_key: "key".to_string(),
            base_url: "url".to_string(),
            timeout: 10,
            max_retries: 1,
            custom_headers: HashMap::from([("h".to_string(), "v".to_string())]),
        };
        let cloned = config.clone();

        assert_eq!(cloned.api_key, config.api_key);
        assert_eq!(cloned.custom_headers, config.custom_headers);
    }

    #[test]
    fn test_common_provider_config_debug() {
        let config = CommonProviderConfig::default();
        let debug_str = format!("{:?}", config);
        assert!(debug_str.contains("CommonProviderConfig"));
    }

    // ==================== RateLimiter Tests ====================

    #[test]
    fn test_rate_limiter_new() {
        let limiter = RateLimiter::new(10);
        assert_eq!(limiter.available_permits(), 10);
    }

    #[tokio::test]
    async fn test_rate_limiter_acquire() {
        let limiter = RateLimiter::new(10);
        assert_eq!(limiter.available_permits(), 10);

        let _permit = limiter.acquire().await.unwrap();
        assert_eq!(limiter.available_permits(), 9);
    }

    #[tokio::test]
    async fn test_rate_limiter_acquire_multiple() {
        let limiter = RateLimiter::new(5);

        let _permit1 = limiter.acquire().await.unwrap();
        let _permit2 = limiter.acquire().await.unwrap();
        let _permit3 = limiter.acquire().await.unwrap();

        assert_eq!(limiter.available_permits(), 2);
    }

    #[tokio::test]
    async fn test_rate_limiter_release() {
        let limiter = RateLimiter::new(10);

        {
            let _permit = limiter.acquire().await.unwrap();
            assert_eq!(limiter.available_permits(), 9);
        }
        // Permit is dropped, should be released
        assert_eq!(limiter.available_permits(), 10);
    }

    // ==================== ResponseValidator Tests ====================

    #[test]
    fn test_response_validator_valid_response() {
        let response = serde_json::json!({
            "id": "test-id",
            "choices": [{"message": {"content": "Hello"}}],
            "created": 1234567890,
            "model": "gpt-4"
        });

        let result = ResponseValidator::validate_chat_response(&response, "test");
        assert!(result.is_ok());
    }

    #[test]
    fn test_response_validator_missing_id() {
        let response = serde_json::json!({
            "choices": [{"message": {"content": "Hello"}}],
            "created": 1234567890,
            "model": "gpt-4"
        });

        let result = ResponseValidator::validate_chat_response(&response, "test");
        assert!(result.is_err());
    }

    #[test]
    fn test_response_validator_missing_choices() {
        let response = serde_json::json!({
            "id": "test-id",
            "created": 1234567890,
            "model": "gpt-4"
        });

        let result = ResponseValidator::validate_chat_response(&response, "test");
        assert!(result.is_err());
    }

    #[test]
    fn test_response_validator_empty_choices() {
        let response = serde_json::json!({
            "id": "test-id",
            "choices": [],
            "created": 1234567890,
            "model": "gpt-4"
        });

        let result = ResponseValidator::validate_chat_response(&response, "test");
        assert!(result.is_err());
    }

    #[test]
    fn test_response_validator_not_object() {
        let response = serde_json::json!([1, 2, 3]);

        let result = ResponseValidator::validate_chat_response(&response, "test");
        assert!(result.is_err());
    }

    // ==================== TokenCostCalculator Tests ====================

    #[test]
    fn test_token_cost_calculator_new() {
        let calculator = TokenCostCalculator::new(0.01, 0.02);
        assert_eq!(calculator.input_cost_per_1k, 0.01);
        assert_eq!(calculator.output_cost_per_1k, 0.02);
    }

    #[test]
    fn test_token_cost_calculator_calculate_cost() {
        let calculator = TokenCostCalculator::new(0.01, 0.02);
        let usage = Usage {
            prompt_tokens: 1000,
            completion_tokens: 500,
            total_tokens: 1500,
            completion_tokens_details: None,
            prompt_tokens_details: None,
            thinking_usage: None,
        };
        let cost = calculator.calculate_cost(&usage);
        // (1000/1000 * 0.01) + (500/1000 * 0.02) = 0.01 + 0.01 = 0.02
        assert!((cost - 0.02).abs() < 0.0001);
    }

    #[test]
    fn test_token_cost_calculator_zero_tokens() {
        let calculator = TokenCostCalculator::new(0.01, 0.02);
        let usage = Usage {
            prompt_tokens: 0,
            completion_tokens: 0,
            total_tokens: 0,
            completion_tokens_details: None,
            prompt_tokens_details: None,
            thinking_usage: None,
        };
        let cost = calculator.calculate_cost(&usage);
        assert!((cost - 0.0).abs() < 0.0001);
    }

    #[test]
    fn test_token_cost_calculator_only_input() {
        let calculator = TokenCostCalculator::new(0.01, 0.02);
        let usage = Usage {
            prompt_tokens: 1000,
            completion_tokens: 0,
            total_tokens: 1000,
            completion_tokens_details: None,
            prompt_tokens_details: None,
            thinking_usage: None,
        };
        let cost = calculator.calculate_cost(&usage);
        assert!((cost - 0.01).abs() < 0.0001);
    }

    #[test]
    fn test_token_cost_calculator_only_output() {
        let calculator = TokenCostCalculator::new(0.01, 0.02);
        let usage = Usage {
            prompt_tokens: 0,
            completion_tokens: 1000,
            total_tokens: 1000,
            completion_tokens_details: None,
            prompt_tokens_details: None,
            thinking_usage: None,
        };
        let cost = calculator.calculate_cost(&usage);
        assert!((cost - 0.02).abs() < 0.0001);
    }

    #[test]
    fn test_token_cost_calculator_large_tokens() {
        let calculator = TokenCostCalculator::new(0.003, 0.015);
        let usage = Usage {
            prompt_tokens: 100000,
            completion_tokens: 50000,
            total_tokens: 150000,
            completion_tokens_details: None,
            prompt_tokens_details: None,
            thinking_usage: None,
        };
        let cost = calculator.calculate_cost(&usage);
        // (100 * 0.003) + (50 * 0.015) = 0.3 + 0.75 = 1.05
        assert!((cost - 1.05).abs() < 0.001);
    }

    #[test]
    fn test_token_cost_calculator_clone() {
        let calculator = TokenCostCalculator::new(0.01, 0.02);
        let cloned = calculator.clone();

        assert_eq!(cloned.input_cost_per_1k, calculator.input_cost_per_1k);
        assert_eq!(cloned.output_cost_per_1k, calculator.output_cost_per_1k);
    }

    #[test]
    fn test_token_cost_calculator_debug() {
        let calculator = TokenCostCalculator::new(0.01, 0.02);
        let debug_str = format!("{:?}", calculator);
        assert!(debug_str.contains("TokenCostCalculator"));
    }

    // ==================== Test Utilities Tests ====================

    #[test]
    fn test_mock_message() {
        let message = test_utils::mock_message(MessageRole::User, "Hello");

        assert_eq!(message.role, MessageRole::User);
        match &message.content {
            Some(MessageContent::Text(text)) => assert_eq!(text, "Hello"),
            _ => panic!("Expected text content"),
        }
    }

    #[test]
    fn test_mock_usage() {
        let usage = test_utils::mock_usage(100, 50);

        assert_eq!(usage.prompt_tokens, 100);
        assert_eq!(usage.completion_tokens, 50);
        assert_eq!(usage.total_tokens, 150);
    }
}
