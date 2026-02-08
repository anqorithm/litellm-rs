use crate::core::providers::unified_provider::ProviderError;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorContext {
    pub error_type: String,
    pub message: String,
    pub provider: String,
    pub request_id: Option<String>,
    pub timestamp: i64,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ErrorCategory {
    ClientError,    // 4xx errors
    ServerError,    // 5xx errors
    TransientError, // Retryable errors
    PermanentError, // Non-retryable errors
}

pub struct ErrorUtils;

impl ErrorUtils {
    pub fn map_http_status_to_error(status_code: u16, message: Option<String>) -> ProviderError {
        let msg = message.unwrap_or_else(|| format!("HTTP error {}", status_code));

        match status_code {
            400 => ProviderError::InvalidRequest {
                provider: "unknown",
                message: msg,
            },
            401 => ProviderError::Authentication {
                provider: "unknown",
                message: msg,
            },
            403 => ProviderError::Authentication {
                provider: "unknown",
                message: format!("Permission denied: {}", msg),
            },
            404 => ProviderError::ModelNotFound {
                provider: "unknown",
                model: msg,
            },
            429 => ProviderError::rate_limit_with_retry("unknown", msg, Some(60)),
            408 | 504 => ProviderError::Timeout {
                provider: "unknown",
                message: msg,
            },
            500 | 502 | 503 => ProviderError::ProviderUnavailable {
                provider: "unknown",
                message: msg,
            },
            _ => ProviderError::Other {
                provider: "unknown",
                message: msg,
            },
        }
    }

    pub fn extract_retry_after(headers: &HashMap<String, String>) -> Option<Duration> {
        // Check for Retry-After header
        if let Some(retry_after) = headers.get("retry-after") {
            if let Ok(seconds) = retry_after.parse::<u64>() {
                return Some(Duration::from_secs(seconds));
            }
        }

        // Check for X-RateLimit-Reset header
        if let Some(reset) = headers.get("x-ratelimit-reset") {
            if let Ok(timestamp) = reset.parse::<i64>() {
                let now = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs() as i64;

                if timestamp > now {
                    return Some(Duration::from_secs((timestamp - now) as u64));
                }
            }
        }

        None
    }

    pub fn parse_openai_error(response_body: &str) -> ProviderError {
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(response_body) {
            if let Some(error) = json.get("error") {
                let error_type = error.get("type").and_then(|v| v.as_str()).unwrap_or("");
                let message = error
                    .get("message")
                    .and_then(|v| v.as_str())
                    .unwrap_or("Unknown error")
                    .to_string();

                return match error_type {
                    "invalid_request_error" => ProviderError::InvalidRequest {
                        provider: "openai",
                        message,
                    },
                    "authentication_error" => ProviderError::Authentication {
                        provider: "openai",
                        message,
                    },
                    "permission_error" => ProviderError::Authentication {
                        provider: "openai",
                        message,
                    },
                    "rate_limit_error" => {
                        ProviderError::rate_limit_with_retry("openai", message, Some(60))
                    }
                    "model_not_found_error" => ProviderError::ModelNotFound {
                        provider: "openai",
                        model: message,
                    },
                    "context_length_exceeded" => ProviderError::InvalidRequest {
                        provider: "openai",
                        message: format!("Context length exceeded: {}", message),
                    },
                    "timeout_error" => ProviderError::Timeout {
                        provider: "openai",
                        message,
                    },
                    "server_error" => ProviderError::ProviderUnavailable {
                        provider: "openai",
                        message,
                    },
                    _ => ProviderError::Other {
                        provider: "openai",
                        message,
                    },
                };
            }
        }

        ProviderError::Other {
            provider: "openai",
            message: response_body.to_string(),
        }
    }

    pub fn parse_anthropic_error(response_body: &str) -> ProviderError {
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(response_body) {
            if let Some(error) = json.get("error") {
                let error_type = error.get("type").and_then(|v| v.as_str()).unwrap_or("");
                let message = error
                    .get("message")
                    .and_then(|v| v.as_str())
                    .unwrap_or("Unknown error")
                    .to_string();

                return match error_type {
                    "invalid_request_error" => ProviderError::InvalidRequest {
                        provider: "anthropic",
                        message,
                    },
                    "authentication_error" => ProviderError::Authentication {
                        provider: "anthropic",
                        message,
                    },
                    "permission_error" => ProviderError::Authentication {
                        provider: "anthropic",
                        message,
                    },
                    "rate_limit_error" => {
                        ProviderError::rate_limit_with_retry("anthropic", message, Some(60))
                    }
                    "not_found_error" => ProviderError::ModelNotFound {
                        provider: "anthropic",
                        model: message,
                    },
                    "overloaded_error" => ProviderError::ProviderUnavailable {
                        provider: "anthropic",
                        message,
                    },
                    _ => ProviderError::Other {
                        provider: "anthropic",
                        message,
                    },
                };
            }
        }

        ProviderError::Other {
            provider: "anthropic",
            message: response_body.to_string(),
        }
    }

    pub fn parse_google_error(response_body: &str) -> ProviderError {
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(response_body) {
            if let Some(error) = json.get("error") {
                let status = error.get("status").and_then(|v| v.as_str()).unwrap_or("");
                let message = error
                    .get("message")
                    .and_then(|v| v.as_str())
                    .unwrap_or("Unknown error")
                    .to_string();

                return match status {
                    "INVALID_ARGUMENT" => ProviderError::InvalidRequest {
                        provider: "google",
                        message,
                    },
                    "UNAUTHENTICATED" => ProviderError::Authentication {
                        provider: "google",
                        message,
                    },
                    "PERMISSION_DENIED" => ProviderError::Authentication {
                        provider: "google",
                        message,
                    },
                    "RESOURCE_EXHAUSTED" => ProviderError::rate_limit_simple("google", message),
                    "NOT_FOUND" => ProviderError::ModelNotFound {
                        provider: "google",
                        model: message,
                    },
                    "INTERNAL" => ProviderError::Other {
                        provider: "google",
                        message,
                    },
                    "UNAVAILABLE" => ProviderError::ProviderUnavailable {
                        provider: "google",
                        message,
                    },
                    _ => ProviderError::Other {
                        provider: "google",
                        message,
                    },
                };
            }
        }

        ProviderError::Other {
            provider: "unknown",
            message: response_body.to_string(),
        }
    }

    pub fn parse_provider_error(
        provider: &str,
        status_code: u16,
        response_body: &str,
    ) -> ProviderError {
        match provider.to_lowercase().as_str() {
            "openai" => Self::parse_openai_error(response_body),
            "anthropic" => Self::parse_anthropic_error(response_body),
            "google" => Self::parse_google_error(response_body),
            _ => Self::map_http_status_to_error(status_code, Some(response_body.to_string())),
        }
    }

    pub fn format_error_for_user(error: &ProviderError) -> String {
        match error {
            ProviderError::Authentication { message, .. } => {
                format!("Authentication failed: {}", message)
            }
            ProviderError::InvalidRequest { message, .. } => {
                format!("Request validation failed: {}", message)
            }
            ProviderError::RateLimit { message, .. } => {
                format!("Rate limit exceeded: {}", message)
            }
            ProviderError::QuotaExceeded { message, .. } => {
                format!("Quota exceeded: {}", message)
            }
            ProviderError::ModelNotFound { model, .. } => {
                format!("Model not supported: {}", model)
            }
            ProviderError::Timeout { message, .. } => {
                format!("Request timeout: {}", message)
            }
            ProviderError::Other { message, .. } => {
                format!("Provider error: {}", message)
            }
            ProviderError::Network { message, .. } => {
                format!("Network error: {}", message)
            }
            ProviderError::ProviderUnavailable { message, .. } => {
                format!("Provider unavailable: {}", message)
            }
            ProviderError::Serialization { message, .. } => {
                format!("Parsing error: {}", message)
            }
            _ => {
                format!("Provider error: {}", error)
            }
        }
    }

    pub fn get_error_category(error: &ProviderError) -> ErrorCategory {
        match error {
            ProviderError::InvalidRequest { .. } => ErrorCategory::ClientError,
            ProviderError::Authentication { .. } => ErrorCategory::ClientError,
            ProviderError::ModelNotFound { .. } => ErrorCategory::ClientError,
            ProviderError::RateLimit { .. } => ErrorCategory::TransientError,
            ProviderError::QuotaExceeded { .. } => ErrorCategory::ClientError,
            ProviderError::Network { .. } => ErrorCategory::TransientError,
            ProviderError::Timeout { .. } => ErrorCategory::TransientError,
            ProviderError::ProviderUnavailable { .. } => ErrorCategory::TransientError,
            ProviderError::Configuration { .. } => ErrorCategory::PermanentError,
            ProviderError::NotSupported { .. } => ErrorCategory::PermanentError,
            ProviderError::NotImplemented { .. } => ErrorCategory::PermanentError,
            _ => ErrorCategory::ServerError,
        }
    }

    pub fn should_retry(error: &ProviderError) -> bool {
        matches!(
            error,
            ProviderError::Network { .. }
                | ProviderError::Timeout { .. }
                | ProviderError::ProviderUnavailable { .. }
                | ProviderError::RateLimit { .. }
        )
    }

    pub fn get_retry_delay(error: &ProviderError) -> Duration {
        match error {
            ProviderError::RateLimit { retry_after, .. } => {
                Duration::from_secs(retry_after.unwrap_or(60))
            }
            ProviderError::ProviderUnavailable { .. } => Duration::from_secs(5),
            ProviderError::Network { .. } => Duration::from_secs(1),
            ProviderError::Timeout { .. } => Duration::from_secs(2),
            _ => Duration::from_secs(1),
        }
    }
}

#[cfg(test)]
#[path = "utils_tests.rs"]
mod tests;
