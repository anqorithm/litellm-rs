//! OpenRouter provider error types

use super::litellm::LiteLLMError;
use super::openai::OpenAIError;
use super::traits::ProviderErrorTrait;
use crate::impl_from_serde_error;

/// OpenRouter provider error types
#[derive(Debug, thiserror::Error)]
pub enum OpenRouterError {
    #[error("OpenRouter API error: {message}")]
    ApiError {
        message: String,
        status_code: Option<u16>,
    },

    #[error("Authentication failed: {0}")]
    Authentication(String),

    #[error("Rate limit exceeded: {0}")]
    RateLimit(String),

    #[error("Model '{0}' not found")]
    ModelNotFound(String),

    #[error("Invalid request: {0}")]
    InvalidRequest(String),

    #[error("Network error: {0}")]
    Network(String),

    #[error("Timeout: {0}")]
    Timeout(String),

    #[error("Parsing error: {0}")]
    Parsing(String),

    #[error("Configuration error: {0}")]
    Configuration(String),

    #[error("Transformation error: {0}")]
    Transformation(String),

    #[error("Unsupported feature: {0}")]
    UnsupportedFeature(String),

    #[error("Feature not implemented: {0}")]
    NotImplemented(String),
}

impl_from_serde_error!(OpenRouterError, |e| Self::Parsing(e.to_string()));

impl From<OpenAIError> for OpenRouterError {
    fn from(err: OpenAIError) -> Self {
        Self::Transformation(err.to_string())
    }
}

impl ProviderErrorTrait for OpenRouterError {
    fn error_type(&self) -> &'static str {
        match self {
            Self::ApiError { .. } => "api_error",
            Self::Authentication(_) => "authentication_error",
            Self::RateLimit(_) => "rate_limit_error",
            Self::ModelNotFound(_) => "model_not_found",
            Self::InvalidRequest(_) => "invalid_request_error",
            Self::Network(_) => "network_error",
            Self::Timeout(_) => "timeout_error",
            Self::Parsing(_) => "parsing_error",
            Self::Configuration(_) => "configuration_error",
            Self::Transformation(_) => "transformation_error",
            Self::UnsupportedFeature(_) => "unsupported_feature",
            Self::NotImplemented(_) => "not_implemented",
        }
    }

    fn is_retryable(&self) -> bool {
        match self {
            Self::Network(_) | Self::Timeout(_) => true,
            Self::RateLimit(_) => true,
            Self::ApiError {
                status_code: Some(code),
                ..
            } => matches!(*code, 429 | 500 | 502 | 503 | 504),
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
            Self::ModelNotFound(_) => 404,
            Self::InvalidRequest(_) => 400,
            Self::Configuration(_) => 400,
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
            "Feature '{}' is not supported by OpenRouter",
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

impl From<OpenRouterError> for LiteLLMError {
    fn from(err: OpenRouterError) -> Self {
        Self::provider_error_with_source("openrouter", err.to_string(), Box::new(err))
    }
}

/// Result type alias
pub type OpenRouterResult<T> = Result<T, OpenRouterError>;

#[cfg(test)]
mod tests {
    use super::*;

    // ==================== Error Type Tests ====================

    #[test]
    fn test_api_error() {
        let err = OpenRouterError::ApiError {
            message: "API failure".to_string(),
            status_code: Some(500),
        };
        assert!(err.to_string().contains("API failure"));
    }

    #[test]
    fn test_authentication_error() {
        let err = OpenRouterError::Authentication("Invalid API key".to_string());
        assert!(err.to_string().contains("Invalid API key"));
    }

    #[test]
    fn test_rate_limit_error() {
        let err = OpenRouterError::RateLimit("Too many requests".to_string());
        assert!(err.to_string().contains("Too many requests"));
    }

    #[test]
    fn test_model_not_found_error() {
        let err = OpenRouterError::ModelNotFound("anthropic/claude-3".to_string());
        assert!(err.to_string().contains("anthropic/claude-3"));
        assert!(err.to_string().contains("not found"));
    }

    #[test]
    fn test_invalid_request_error() {
        let err = OpenRouterError::InvalidRequest("Missing model".to_string());
        assert!(err.to_string().contains("Missing model"));
    }

    #[test]
    fn test_network_error() {
        let err = OpenRouterError::Network("Connection refused".to_string());
        assert!(err.to_string().contains("Connection refused"));
    }

    #[test]
    fn test_timeout_error() {
        let err = OpenRouterError::Timeout("Request timed out".to_string());
        assert!(err.to_string().contains("timed out"));
    }

    #[test]
    fn test_parsing_error() {
        let err = OpenRouterError::Parsing("Invalid JSON".to_string());
        assert!(err.to_string().contains("Invalid JSON"));
    }

    #[test]
    fn test_configuration_error() {
        let err = OpenRouterError::Configuration("Missing API key".to_string());
        assert!(err.to_string().contains("Missing API key"));
    }

    #[test]
    fn test_transformation_error() {
        let err = OpenRouterError::Transformation("Failed to convert".to_string());
        assert!(err.to_string().contains("Failed to convert"));
    }

    #[test]
    fn test_unsupported_feature_error() {
        let err = OpenRouterError::UnsupportedFeature("streaming".to_string());
        assert!(err.to_string().contains("streaming"));
    }

    #[test]
    fn test_not_implemented_error() {
        let err = OpenRouterError::NotImplemented("batch".to_string());
        assert!(err.to_string().contains("batch"));
    }

    // ==================== ProviderErrorTrait Tests ====================

    #[test]
    fn test_error_type_api_error() {
        let err = OpenRouterError::ApiError {
            message: "test".to_string(),
            status_code: None,
        };
        assert_eq!(err.error_type(), "api_error");
    }

    #[test]
    fn test_error_type_authentication() {
        let err = OpenRouterError::Authentication("test".to_string());
        assert_eq!(err.error_type(), "authentication_error");
    }

    #[test]
    fn test_error_type_rate_limit() {
        let err = OpenRouterError::RateLimit("test".to_string());
        assert_eq!(err.error_type(), "rate_limit_error");
    }

    #[test]
    fn test_error_type_model_not_found() {
        let err = OpenRouterError::ModelNotFound("test".to_string());
        assert_eq!(err.error_type(), "model_not_found");
    }

    #[test]
    fn test_error_type_configuration() {
        let err = OpenRouterError::Configuration("test".to_string());
        assert_eq!(err.error_type(), "configuration_error");
    }

    #[test]
    fn test_error_type_transformation() {
        let err = OpenRouterError::Transformation("test".to_string());
        assert_eq!(err.error_type(), "transformation_error");
    }

    // ==================== is_retryable Tests ====================

    #[test]
    fn test_is_retryable_network() {
        let err = OpenRouterError::Network("failed".to_string());
        assert!(err.is_retryable());
    }

    #[test]
    fn test_is_retryable_timeout() {
        let err = OpenRouterError::Timeout("timed out".to_string());
        assert!(err.is_retryable());
    }

    #[test]
    fn test_is_retryable_rate_limit() {
        let err = OpenRouterError::RateLimit("limited".to_string());
        assert!(err.is_retryable());
    }

    #[test]
    fn test_is_retryable_api_error_429() {
        let err = OpenRouterError::ApiError {
            message: "rate limited".to_string(),
            status_code: Some(429),
        };
        assert!(err.is_retryable());
    }

    #[test]
    fn test_is_retryable_api_error_503() {
        let err = OpenRouterError::ApiError {
            message: "unavailable".to_string(),
            status_code: Some(503),
        };
        assert!(err.is_retryable());
    }

    #[test]
    fn test_is_not_retryable_authentication() {
        let err = OpenRouterError::Authentication("invalid".to_string());
        assert!(!err.is_retryable());
    }

    #[test]
    fn test_is_not_retryable_invalid_request() {
        let err = OpenRouterError::InvalidRequest("bad request".to_string());
        assert!(!err.is_retryable());
    }

    #[test]
    fn test_is_not_retryable_configuration() {
        let err = OpenRouterError::Configuration("missing key".to_string());
        assert!(!err.is_retryable());
    }

    // ==================== retry_delay Tests ====================

    #[test]
    fn test_retry_delay_rate_limit() {
        let err = OpenRouterError::RateLimit("limited".to_string());
        assert_eq!(err.retry_delay(), Some(60));
    }

    #[test]
    fn test_retry_delay_network() {
        let err = OpenRouterError::Network("failed".to_string());
        assert_eq!(err.retry_delay(), Some(1));
    }

    #[test]
    fn test_retry_delay_timeout() {
        let err = OpenRouterError::Timeout("timed out".to_string());
        assert_eq!(err.retry_delay(), Some(1));
    }

    #[test]
    fn test_retry_delay_api_error_429() {
        let err = OpenRouterError::ApiError {
            message: "rate limited".to_string(),
            status_code: Some(429),
        };
        assert_eq!(err.retry_delay(), Some(60));
    }

    #[test]
    fn test_retry_delay_api_error_500() {
        let err = OpenRouterError::ApiError {
            message: "server error".to_string(),
            status_code: Some(500),
        };
        assert_eq!(err.retry_delay(), Some(5));
    }

    #[test]
    fn test_retry_delay_authentication() {
        let err = OpenRouterError::Authentication("invalid".to_string());
        assert_eq!(err.retry_delay(), None);
    }

    // ==================== http_status Tests ====================

    #[test]
    fn test_http_status_authentication() {
        let err = OpenRouterError::Authentication("invalid".to_string());
        assert_eq!(err.http_status(), 401);
    }

    #[test]
    fn test_http_status_rate_limit() {
        let err = OpenRouterError::RateLimit("limited".to_string());
        assert_eq!(err.http_status(), 429);
    }

    #[test]
    fn test_http_status_model_not_found() {
        let err = OpenRouterError::ModelNotFound("test".to_string());
        assert_eq!(err.http_status(), 404);
    }

    #[test]
    fn test_http_status_invalid_request() {
        let err = OpenRouterError::InvalidRequest("bad".to_string());
        assert_eq!(err.http_status(), 400);
    }

    #[test]
    fn test_http_status_configuration() {
        let err = OpenRouterError::Configuration("missing".to_string());
        assert_eq!(err.http_status(), 400);
    }

    #[test]
    fn test_http_status_unsupported_feature() {
        let err = OpenRouterError::UnsupportedFeature("feature".to_string());
        assert_eq!(err.http_status(), 405);
    }

    #[test]
    fn test_http_status_not_implemented() {
        let err = OpenRouterError::NotImplemented("feature".to_string());
        assert_eq!(err.http_status(), 501);
    }

    #[test]
    fn test_http_status_api_error_with_code() {
        let err = OpenRouterError::ApiError {
            message: "test".to_string(),
            status_code: Some(422),
        };
        assert_eq!(err.http_status(), 422);
    }

    #[test]
    fn test_http_status_network() {
        let err = OpenRouterError::Network("failed".to_string());
        assert_eq!(err.http_status(), 503);
    }

    // ==================== Trait Constructor Tests ====================

    #[test]
    fn test_trait_not_supported() {
        let err = <OpenRouterError as ProviderErrorTrait>::not_supported("feature");
        assert!(matches!(err, OpenRouterError::UnsupportedFeature(_)));
    }

    #[test]
    fn test_trait_authentication_failed() {
        let err = <OpenRouterError as ProviderErrorTrait>::authentication_failed("bad key");
        assert!(matches!(err, OpenRouterError::Authentication(_)));
    }

    #[test]
    fn test_trait_rate_limited_with_retry() {
        let err = <OpenRouterError as ProviderErrorTrait>::rate_limited(Some(30));
        assert!(matches!(err, OpenRouterError::RateLimit(_)));
        assert!(err.to_string().contains("30"));
    }

    #[test]
    fn test_trait_rate_limited_without_retry() {
        let err = <OpenRouterError as ProviderErrorTrait>::rate_limited(None);
        assert!(matches!(err, OpenRouterError::RateLimit(_)));
    }

    #[test]
    fn test_trait_network_error() {
        let err = <OpenRouterError as ProviderErrorTrait>::network_error("connection failed");
        assert!(matches!(err, OpenRouterError::Network(_)));
    }

    #[test]
    fn test_trait_parsing_error() {
        let err = <OpenRouterError as ProviderErrorTrait>::parsing_error("invalid json");
        assert!(matches!(err, OpenRouterError::Parsing(_)));
    }

    #[test]
    fn test_trait_not_implemented() {
        let err = <OpenRouterError as ProviderErrorTrait>::not_implemented("feature");
        assert!(matches!(err, OpenRouterError::NotImplemented(_)));
    }

    // ==================== From Implementations ====================

    #[test]
    fn test_from_serde_json_error() {
        let json_err = serde_json::from_str::<serde_json::Value>("invalid").unwrap_err();
        let err: OpenRouterError = json_err.into();
        assert!(matches!(err, OpenRouterError::Parsing(_)));
    }

    #[test]
    fn test_from_openai_error() {
        let openai_err = OpenAIError::Network("connection failed".to_string());
        let err: OpenRouterError = openai_err.into();
        assert!(matches!(err, OpenRouterError::Transformation(_)));
    }

    #[test]
    fn test_to_litellm_error() {
        let openrouter_err = OpenRouterError::Authentication("invalid key".to_string());
        let litellm_err: LiteLLMError = openrouter_err.into();
        assert!(matches!(litellm_err, LiteLLMError::Provider { .. }));
    }
}
