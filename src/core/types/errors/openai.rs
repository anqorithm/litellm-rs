//! OpenAI provider error types

use super::litellm::LiteLLMError;
use super::traits::ProviderErrorTrait;
use crate::{impl_from_reqwest_error, impl_from_serde_error};

/// OpenAI provider error types
#[derive(Debug, thiserror::Error)]
pub enum OpenAIError {
    #[error("OpenAI API error: {message}")]
    ApiError {
        message: String,
        status_code: Option<u16>,
        error_type: Option<String>,
    },

    #[error("Authentication failed: {0}")]
    Authentication(String),

    #[error("Rate limit exceeded: {0}")]
    RateLimit(String),

    #[error("Model '{model}' not found")]
    ModelNotFound { model: String },

    #[error("Invalid request: {0}")]
    InvalidRequest(String),

    #[error("Network error: {0}")]
    Network(String),

    #[error("Timeout: {0}")]
    Timeout(String),

    #[error("Parsing error: {0}")]
    Parsing(String),

    #[error("Streaming error: {0}")]
    Streaming(String),

    #[error("Unsupported feature: {0}")]
    UnsupportedFeature(String),

    #[error("Feature not implemented: {0}")]
    NotImplemented(String),

    #[error("Other OpenAI error: {0}")]
    Other(String),
}

impl OpenAIError {
    pub fn not_supported(feature: &str) -> Self {
        Self::UnsupportedFeature(format!("{} is not supported", feature))
    }
}

impl ProviderErrorTrait for OpenAIError {
    fn error_type(&self) -> &'static str {
        match self {
            Self::ApiError { .. } => "api_error",
            Self::Authentication(_) => "authentication_error",
            Self::RateLimit(_) => "rate_limit_error",
            Self::ModelNotFound { .. } => "model_not_found",
            Self::InvalidRequest(_) => "invalid_request_error",
            Self::Network(_) => "network_error",
            Self::Timeout(_) => "timeout_error",
            Self::Parsing(_) => "parsing_error",
            Self::Streaming(_) => "streaming_error",
            Self::UnsupportedFeature(_) => "unsupported_feature",
            Self::NotImplemented(_) => "not_implemented",
            Self::Other(_) => "other_error",
        }
    }

    fn is_retryable(&self) -> bool {
        match self {
            Self::Network(_) | Self::Timeout(_) | Self::Streaming(_) => true,
            Self::ApiError {
                status_code: Some(code),
                ..
            } => matches!(*code, 429 | 500 | 502 | 503 | 504),
            Self::RateLimit(_) => true,
            _ => false,
        }
    }

    fn retry_delay(&self) -> Option<u64> {
        match self {
            Self::RateLimit(_) => Some(60),
            Self::Network(_) | Self::Timeout(_) => Some(1),
            Self::ApiError {
                status_code: Some(429),
                ..
            } => Some(60),
            Self::ApiError {
                status_code: Some(code),
                ..
            } if *code >= 500 => Some(5),
            _ => None,
        }
    }

    fn http_status(&self) -> u16 {
        match self {
            Self::Authentication(_) => 401,
            Self::RateLimit(_) => 429,
            Self::ModelNotFound { .. } => 404,
            Self::InvalidRequest(_) => 400,
            Self::UnsupportedFeature(_) => 405,
            Self::NotImplemented(_) => 501,
            Self::ApiError {
                status_code: Some(code),
                ..
            } => *code,
            Self::Network(_) | Self::Timeout(_) => 503,
            _ => 500,
        }
    }

    fn not_supported(feature: &str) -> Self {
        Self::UnsupportedFeature(format!(
            "Feature '{}' is not supported by OpenAI provider",
            feature
        ))
    }

    fn authentication_failed(reason: &str) -> Self {
        Self::Authentication(reason.to_string())
    }

    fn rate_limited(retry_after: Option<u64>) -> Self {
        let message = if let Some(seconds) = retry_after {
            format!("Rate limit exceeded. Retry after {} seconds", seconds)
        } else {
            "Rate limit exceeded".to_string()
        };
        Self::RateLimit(message)
    }

    fn network_error(details: &str) -> Self {
        Self::Network(details.to_string())
    }

    fn parsing_error(details: &str) -> Self {
        Self::Parsing(details.to_string())
    }

    fn not_implemented(feature: &str) -> Self {
        Self::NotImplemented(format!("Feature '{}' not yet implemented", feature))
    }
}

impl_from_reqwest_error!(OpenAIError,
    timeout => |e| Self::Timeout(e.to_string()),
    connect => |e| Self::Network(e.to_string()),
    other   => |e| Self::Other(e.to_string())
);

impl_from_serde_error!(OpenAIError, |e| Self::Parsing(e.to_string()));

impl From<OpenAIError> for LiteLLMError {
    fn from(err: OpenAIError) -> Self {
        Self::provider_error_with_source("openai", err.to_string(), Box::new(err))
    }
}

/// Result type alias
pub type OpenAIResult<T> = Result<T, OpenAIError>;

#[cfg(test)]
mod tests {
    use super::*;

    // ==================== Error Type Tests ====================

    #[test]
    fn test_api_error() {
        let err = OpenAIError::ApiError {
            message: "API failure".to_string(),
            status_code: Some(500),
            error_type: Some("server_error".to_string()),
        };
        assert!(err.to_string().contains("API failure"));
    }

    #[test]
    fn test_authentication_error() {
        let err = OpenAIError::Authentication("Invalid API key".to_string());
        assert!(err.to_string().contains("Invalid API key"));
    }

    #[test]
    fn test_rate_limit_error() {
        let err = OpenAIError::RateLimit("Too many requests".to_string());
        assert!(err.to_string().contains("Too many requests"));
    }

    #[test]
    fn test_model_not_found_error() {
        let err = OpenAIError::ModelNotFound {
            model: "gpt-5".to_string(),
        };
        assert!(err.to_string().contains("gpt-5"));
        assert!(err.to_string().contains("not found"));
    }

    #[test]
    fn test_invalid_request_error() {
        let err = OpenAIError::InvalidRequest("Missing messages".to_string());
        assert!(err.to_string().contains("Missing messages"));
    }

    #[test]
    fn test_network_error() {
        let err = OpenAIError::Network("Connection refused".to_string());
        assert!(err.to_string().contains("Connection refused"));
    }

    #[test]
    fn test_timeout_error() {
        let err = OpenAIError::Timeout("Request timed out".to_string());
        assert!(err.to_string().contains("timed out"));
    }

    #[test]
    fn test_parsing_error() {
        let err = OpenAIError::Parsing("Invalid JSON".to_string());
        assert!(err.to_string().contains("Invalid JSON"));
    }

    #[test]
    fn test_streaming_error() {
        let err = OpenAIError::Streaming("Stream interrupted".to_string());
        assert!(err.to_string().contains("Stream interrupted"));
    }

    #[test]
    fn test_unsupported_feature_error() {
        let err = OpenAIError::UnsupportedFeature("vision".to_string());
        assert!(err.to_string().contains("vision"));
    }

    #[test]
    fn test_not_implemented_error() {
        let err = OpenAIError::NotImplemented("batch processing".to_string());
        assert!(err.to_string().contains("batch processing"));
    }

    #[test]
    fn test_other_error() {
        let err = OpenAIError::Other("Unknown error".to_string());
        assert!(err.to_string().contains("Unknown error"));
    }

    // ==================== Constructor Tests ====================

    #[test]
    fn test_not_supported_constructor() {
        let err = OpenAIError::not_supported("function calling");
        assert!(matches!(err, OpenAIError::UnsupportedFeature(_)));
        assert!(err.to_string().contains("function calling"));
    }

    // ==================== ProviderErrorTrait Tests ====================

    #[test]
    fn test_error_type_api_error() {
        let err = OpenAIError::ApiError {
            message: "test".to_string(),
            status_code: None,
            error_type: None,
        };
        assert_eq!(err.error_type(), "api_error");
    }

    #[test]
    fn test_error_type_authentication() {
        let err = OpenAIError::Authentication("test".to_string());
        assert_eq!(err.error_type(), "authentication_error");
    }

    #[test]
    fn test_error_type_rate_limit() {
        let err = OpenAIError::RateLimit("test".to_string());
        assert_eq!(err.error_type(), "rate_limit_error");
    }

    #[test]
    fn test_error_type_model_not_found() {
        let err = OpenAIError::ModelNotFound {
            model: "test".to_string(),
        };
        assert_eq!(err.error_type(), "model_not_found");
    }

    #[test]
    fn test_error_type_network() {
        let err = OpenAIError::Network("test".to_string());
        assert_eq!(err.error_type(), "network_error");
    }

    #[test]
    fn test_error_type_timeout() {
        let err = OpenAIError::Timeout("test".to_string());
        assert_eq!(err.error_type(), "timeout_error");
    }

    // ==================== is_retryable Tests ====================

    #[test]
    fn test_is_retryable_network() {
        let err = OpenAIError::Network("failed".to_string());
        assert!(err.is_retryable());
    }

    #[test]
    fn test_is_retryable_timeout() {
        let err = OpenAIError::Timeout("timed out".to_string());
        assert!(err.is_retryable());
    }

    #[test]
    fn test_is_retryable_streaming() {
        let err = OpenAIError::Streaming("interrupted".to_string());
        assert!(err.is_retryable());
    }

    #[test]
    fn test_is_retryable_rate_limit() {
        let err = OpenAIError::RateLimit("limited".to_string());
        assert!(err.is_retryable());
    }

    #[test]
    fn test_is_retryable_api_error_429() {
        let err = OpenAIError::ApiError {
            message: "rate limited".to_string(),
            status_code: Some(429),
            error_type: None,
        };
        assert!(err.is_retryable());
    }

    #[test]
    fn test_is_retryable_api_error_500() {
        let err = OpenAIError::ApiError {
            message: "server error".to_string(),
            status_code: Some(500),
            error_type: None,
        };
        assert!(err.is_retryable());
    }

    #[test]
    fn test_is_retryable_api_error_503() {
        let err = OpenAIError::ApiError {
            message: "unavailable".to_string(),
            status_code: Some(503),
            error_type: None,
        };
        assert!(err.is_retryable());
    }

    #[test]
    fn test_is_not_retryable_authentication() {
        let err = OpenAIError::Authentication("invalid".to_string());
        assert!(!err.is_retryable());
    }

    #[test]
    fn test_is_not_retryable_invalid_request() {
        let err = OpenAIError::InvalidRequest("bad request".to_string());
        assert!(!err.is_retryable());
    }

    #[test]
    fn test_is_not_retryable_api_error_400() {
        let err = OpenAIError::ApiError {
            message: "bad request".to_string(),
            status_code: Some(400),
            error_type: None,
        };
        assert!(!err.is_retryable());
    }

    // ==================== retry_delay Tests ====================

    #[test]
    fn test_retry_delay_rate_limit() {
        let err = OpenAIError::RateLimit("limited".to_string());
        assert_eq!(err.retry_delay(), Some(60));
    }

    #[test]
    fn test_retry_delay_network() {
        let err = OpenAIError::Network("failed".to_string());
        assert_eq!(err.retry_delay(), Some(1));
    }

    #[test]
    fn test_retry_delay_timeout() {
        let err = OpenAIError::Timeout("timed out".to_string());
        assert_eq!(err.retry_delay(), Some(1));
    }

    #[test]
    fn test_retry_delay_api_error_429() {
        let err = OpenAIError::ApiError {
            message: "rate limited".to_string(),
            status_code: Some(429),
            error_type: None,
        };
        assert_eq!(err.retry_delay(), Some(60));
    }

    #[test]
    fn test_retry_delay_api_error_500() {
        let err = OpenAIError::ApiError {
            message: "server error".to_string(),
            status_code: Some(500),
            error_type: None,
        };
        assert_eq!(err.retry_delay(), Some(5));
    }

    #[test]
    fn test_retry_delay_authentication() {
        let err = OpenAIError::Authentication("invalid".to_string());
        assert_eq!(err.retry_delay(), None);
    }

    // ==================== http_status Tests ====================

    #[test]
    fn test_http_status_authentication() {
        let err = OpenAIError::Authentication("invalid".to_string());
        assert_eq!(err.http_status(), 401);
    }

    #[test]
    fn test_http_status_rate_limit() {
        let err = OpenAIError::RateLimit("limited".to_string());
        assert_eq!(err.http_status(), 429);
    }

    #[test]
    fn test_http_status_model_not_found() {
        let err = OpenAIError::ModelNotFound {
            model: "test".to_string(),
        };
        assert_eq!(err.http_status(), 404);
    }

    #[test]
    fn test_http_status_invalid_request() {
        let err = OpenAIError::InvalidRequest("bad".to_string());
        assert_eq!(err.http_status(), 400);
    }

    #[test]
    fn test_http_status_unsupported_feature() {
        let err = OpenAIError::UnsupportedFeature("feature".to_string());
        assert_eq!(err.http_status(), 405);
    }

    #[test]
    fn test_http_status_not_implemented() {
        let err = OpenAIError::NotImplemented("feature".to_string());
        assert_eq!(err.http_status(), 501);
    }

    #[test]
    fn test_http_status_api_error_with_code() {
        let err = OpenAIError::ApiError {
            message: "test".to_string(),
            status_code: Some(422),
            error_type: None,
        };
        assert_eq!(err.http_status(), 422);
    }

    #[test]
    fn test_http_status_network() {
        let err = OpenAIError::Network("failed".to_string());
        assert_eq!(err.http_status(), 503);
    }

    // ==================== Trait Constructor Tests ====================

    #[test]
    fn test_trait_not_supported() {
        let err = <OpenAIError as ProviderErrorTrait>::not_supported("feature");
        assert!(matches!(err, OpenAIError::UnsupportedFeature(_)));
    }

    #[test]
    fn test_trait_authentication_failed() {
        let err = <OpenAIError as ProviderErrorTrait>::authentication_failed("bad key");
        assert!(matches!(err, OpenAIError::Authentication(_)));
    }

    #[test]
    fn test_trait_rate_limited_with_retry() {
        let err = <OpenAIError as ProviderErrorTrait>::rate_limited(Some(30));
        assert!(matches!(err, OpenAIError::RateLimit(_)));
        assert!(err.to_string().contains("30"));
    }

    #[test]
    fn test_trait_rate_limited_without_retry() {
        let err = <OpenAIError as ProviderErrorTrait>::rate_limited(None);
        assert!(matches!(err, OpenAIError::RateLimit(_)));
    }

    #[test]
    fn test_trait_network_error() {
        let err = <OpenAIError as ProviderErrorTrait>::network_error("connection failed");
        assert!(matches!(err, OpenAIError::Network(_)));
    }

    #[test]
    fn test_trait_parsing_error() {
        let err = <OpenAIError as ProviderErrorTrait>::parsing_error("invalid json");
        assert!(matches!(err, OpenAIError::Parsing(_)));
    }

    #[test]
    fn test_trait_not_implemented() {
        let err = <OpenAIError as ProviderErrorTrait>::not_implemented("feature");
        assert!(matches!(err, OpenAIError::NotImplemented(_)));
    }

    // ==================== From Implementations ====================

    #[test]
    fn test_from_serde_json_error() {
        let json_err = serde_json::from_str::<serde_json::Value>("invalid").unwrap_err();
        let err: OpenAIError = json_err.into();
        assert!(matches!(err, OpenAIError::Parsing(_)));
    }

    #[test]
    fn test_to_litellm_error() {
        let openai_err = OpenAIError::Authentication("invalid key".to_string());
        let litellm_err: LiteLLMError = openai_err.into();
        assert!(matches!(litellm_err, LiteLLMError::Provider { .. }));
    }
}
