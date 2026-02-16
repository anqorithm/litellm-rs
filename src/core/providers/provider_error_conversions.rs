//! Error Conversions for ProviderError
//!
//! This module contains all `From` implementations for converting various error types
//! into the unified `ProviderError` type.

use super::unified_provider::ProviderError;
use crate::{impl_from_reqwest_error, impl_from_serde_error};

// Convert from common error types
impl_from_reqwest_error!(ProviderError,
    timeout => |e| Self::timeout("unknown", e.to_string()),
    connect => |e| Self::network("unknown", e.to_string()),
    other   => |e| Self::network("unknown", e.to_string())
);

impl_from_serde_error!(ProviderError, |e| Self::serialization(
    "unknown",
    e.to_string()
));

// Convert from provider-specific errors for unified handling
impl From<crate::core::types::errors::OpenAIError> for ProviderError {
    fn from(err: crate::core::types::errors::OpenAIError) -> Self {
        use crate::core::types::errors::OpenAIError;
        match err {
            OpenAIError::Authentication(msg) => Self::authentication("openai", msg),
            OpenAIError::RateLimit(_msg) => Self::rate_limit("openai", Some(60)),
            OpenAIError::InvalidRequest(msg) => Self::invalid_request("openai", msg),
            OpenAIError::Network(msg) => Self::network("openai", msg),
            OpenAIError::Timeout(msg) => Self::timeout("openai", msg),
            OpenAIError::Parsing(msg) => Self::serialization("openai", msg),
            OpenAIError::Streaming(msg) => Self::network("openai", msg),
            OpenAIError::UnsupportedFeature(feature) => Self::not_implemented("openai", feature),
            OpenAIError::NotImplemented(feature) => Self::not_implemented("openai", feature),
            OpenAIError::ModelNotFound { model } => Self::model_not_found("openai", model),
            OpenAIError::ApiError {
                message,
                status_code,
                ..
            } => Self::api_error("openai", status_code.unwrap_or(500), message),
            OpenAIError::Other(msg) => Self::api_error("openai", 500, msg),
        }
    }
}

// AzureError is now a type alias for ProviderError, no conversion needed

// Add more error type conversions for better interoperability
impl From<Box<dyn std::error::Error + Send + Sync>> for ProviderError {
    fn from(err: Box<dyn std::error::Error + Send + Sync>) -> Self {
        Self::network("unknown", format!("{}", err))
    }
}

impl From<String> for ProviderError {
    fn from(err: String) -> Self {
        Self::network("unknown", err)
    }
}

// Provider-specific error conversions for unified error handling
// Note: MoonshotError, MistralError, and LlamaError are now type aliases for ProviderError, so no From impl needed

// OpenRouterError is now a type alias for ProviderError, no conversion needed

// DeepInfraError is now a type alias for ProviderError, no conversion needed

impl From<crate::core::cost::types::CostError> for ProviderError {
    fn from(err: crate::core::cost::types::CostError) -> Self {
        use crate::core::cost::types::CostError;
        match err {
            CostError::ModelNotSupported { model, provider } => Self::model_not_found(
                "cost",
                format!("Model {} not supported for provider {}", model, provider),
            ),
            CostError::ProviderNotSupported { provider } => Self::not_implemented(
                "cost",
                format!("Provider {} does not support cost calculation", provider),
            ),
            CostError::MissingPricing { model } => {
                Self::invalid_request("cost", format!("Missing pricing for model: {}", model))
            }
            CostError::InvalidUsage { message } => Self::invalid_request("cost", message),
            CostError::CalculationError { message } => Self::api_error("cost", 500, message),
            CostError::ConfigError { message } => Self::invalid_request("cost", message),
        }
    }
}

// VertexAIError is now a type alias for ProviderError, no conversion needed

// V0 provider now uses ProviderError directly - no conversion needed

// DeepSeek now uses ProviderError directly - no conversion needed

// Azure AI provider uses ProviderError directly - no conversion needed

// Anthropic provider now uses ProviderError directly - no conversion needed

// ==================== Legacy Methods ====================
// Convenience methods for backward compatibility

impl ProviderError {
    /// Create authentication error (legacy method)
    pub fn authentication_legacy(msg: impl Into<String>) -> Self {
        Self::authentication("unknown", msg)
    }

    /// Create rate limit error (legacy method)
    pub fn rate_limit_legacy(msg: impl Into<String>) -> Self {
        Self::RateLimit {
            provider: "unknown",
            message: msg.into(),
            retry_after: None,
            rpm_limit: None,
            tpm_limit: None,
            current_usage: None,
        }
    }

    /// Create model not found error (legacy method)
    pub fn model_not_found_legacy(msg: impl Into<String>) -> Self {
        Self::ModelNotFound {
            provider: "unknown",
            model: msg.into(),
        }
    }

    /// Create network error (legacy method)
    pub fn network_legacy(msg: impl Into<String>) -> Self {
        Self::network("unknown", msg)
    }

    /// Create generic error (legacy method)
    pub fn generic(err: impl std::error::Error + Send + Sync + 'static) -> Self {
        Self::network("unknown", err.to_string())
    }
}

// ==================== ProviderErrorTrait Implementation ====================

use crate::core::types::errors::ProviderErrorTrait;

impl ProviderErrorTrait for ProviderError {
    fn error_type(&self) -> &'static str {
        match self {
            Self::Authentication { .. } => "authentication",
            Self::RateLimit { .. } => "rate_limit",
            Self::QuotaExceeded { .. } => "quota_exceeded",
            Self::ModelNotFound { .. } => "model_not_found",
            Self::InvalidRequest { .. } => "invalid_request",
            Self::Network { .. } => "network",
            Self::ProviderUnavailable { .. } => "provider_unavailable",
            Self::NotSupported { .. } => "not_supported",
            Self::NotImplemented { .. } => "not_implemented",
            Self::Configuration { .. } => "configuration",
            Self::Serialization { .. } => "serialization",
            Self::Timeout { .. } => "timeout",

            // Enhanced error variants
            Self::ContextLengthExceeded { .. } => "context_length_exceeded",
            Self::ContentFiltered { .. } => "content_filtered",
            Self::ApiError { .. } => "api_error",
            Self::TokenLimitExceeded { .. } => "token_limit_exceeded",
            Self::FeatureDisabled { .. } => "feature_disabled",
            Self::DeploymentError { .. } => "deployment_error",
            Self::ResponseParsing { .. } => "response_parsing",
            Self::RoutingError { .. } => "routing_error",
            Self::TransformationError { .. } => "transformation_error",
            Self::Cancelled { .. } => "cancelled",
            Self::Streaming { .. } => "streaming",

            Self::Other { .. } => "other",
        }
    }

    fn is_retryable(&self) -> bool {
        // Delegate to the main implementation
        ProviderError::is_retryable(self)
    }

    fn retry_delay(&self) -> Option<u64> {
        // Delegate to the main implementation
        ProviderError::retry_delay(self)
    }

    fn http_status(&self) -> u16 {
        // Delegate to the main implementation
        ProviderError::http_status(self)
    }

    fn not_supported(feature: &str) -> Self {
        Self::NotSupported {
            provider: "unknown",
            feature: feature.to_string(),
        }
    }

    fn authentication_failed(reason: &str) -> Self {
        Self::Authentication {
            provider: "unknown",
            message: reason.to_string(),
        }
    }

    fn rate_limited(retry_after: Option<u64>) -> Self {
        Self::RateLimit {
            provider: "unknown",
            message: "Rate limit exceeded".to_string(),
            retry_after,
            rpm_limit: None,
            tpm_limit: None,
            current_usage: None,
        }
    }

    fn network_error(details: &str) -> Self {
        Self::Network {
            provider: "unknown",
            message: details.to_string(),
        }
    }

    fn parsing_error(details: &str) -> Self {
        Self::Serialization {
            provider: "unknown",
            message: details.to_string(),
        }
    }

    fn not_implemented(feature: &str) -> Self {
        Self::NotImplemented {
            provider: "unknown",
            feature: feature.to_string(),
        }
    }
}
