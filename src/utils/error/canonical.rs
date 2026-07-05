//! Canonical cross-protocol error classification.
//!
//! This provides a single error code taxonomy and retryable semantics that can
//! be reused by HTTP/OpenAI-compatible, A2A, and MCP layers.

use super::gateway_error::GatewayError;
use crate::core::a2a::error::A2AError;
use crate::core::mcp::error::McpError;
use crate::core::providers::unified_provider::ProviderError;
#[cfg(feature = "gateway")]
use actix_web::http::StatusCode;

/// Canonical error code shared across protocol boundaries.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorCode {
    Authentication,
    Authorization,
    RateLimited,
    QuotaExceeded,
    InvalidRequest,
    NotFound,
    Conflict,
    Timeout,
    Unavailable,
    Network,
    Configuration,
    Parsing,
    NotImplemented,
    Internal,
}

impl ErrorCode {
    /// Stable machine-readable canonical string.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Authentication => "AUTHENTICATION",
            Self::Authorization => "AUTHORIZATION",
            Self::RateLimited => "RATE_LIMITED",
            Self::QuotaExceeded => "QUOTA_EXCEEDED",
            Self::InvalidRequest => "INVALID_REQUEST",
            Self::NotFound => "NOT_FOUND",
            Self::Conflict => "CONFLICT",
            Self::Timeout => "TIMEOUT",
            Self::Unavailable => "UNAVAILABLE",
            Self::Network => "NETWORK",
            Self::Configuration => "CONFIGURATION",
            Self::Parsing => "PARSING",
            Self::NotImplemented => "NOT_IMPLEMENTED",
            Self::Internal => "INTERNAL",
        }
    }

    /// Default retryability for canonical classes.
    pub const fn is_retryable(self) -> bool {
        matches!(
            self,
            Self::RateLimited | Self::Timeout | Self::Unavailable | Self::Network
        )
    }
}

/// Canonical code and retryability mapping.
pub trait CanonicalError {
    fn canonical_code(&self) -> ErrorCode;

    fn canonical_retryable(&self) -> bool {
        self.canonical_code().is_retryable()
    }
}

/// Canonical HTTP facts shared by gateway and OpenAI-compatible adapters.
#[cfg(feature = "gateway")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HttpErrorFacts {
    pub status: StatusCode,
    pub gateway_code: &'static str,
    pub openai_error_type: &'static str,
    pub openai_code: &'static str,
}

#[cfg(feature = "gateway")]
impl HttpErrorFacts {
    fn new(
        status: StatusCode,
        gateway_code: &'static str,
        openai_error_type: &'static str,
        openai_code: &'static str,
    ) -> Self {
        Self {
            status,
            gateway_code,
            openai_error_type,
            openai_code,
        }
    }
}

/// HTTP header facts that are part of canonical error-to-HTTP mapping.
#[cfg(feature = "gateway")]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct HttpHeaderFacts {
    pub retry_after: Option<u64>,
    pub rpm_limit: Option<u32>,
    pub tpm_limit: Option<u32>,
}

/// Map any gateway error to the canonical HTTP status and adapter codes.
#[cfg(feature = "gateway")]
pub fn gateway_http_error_facts(error: &GatewayError) -> HttpErrorFacts {
    match error {
        GatewayError::Config(_) => http_facts(
            StatusCode::INTERNAL_SERVER_ERROR,
            "CONFIG_ERROR",
            "server_error",
            "internal_error",
        ),
        GatewayError::Storage(_) => http_facts(
            StatusCode::SERVICE_UNAVAILABLE,
            "STORAGE_ERROR",
            "server_error",
            "service_unavailable",
        ),
        GatewayError::HttpClient(_) => http_facts(
            StatusCode::BAD_GATEWAY,
            "HTTP_CLIENT_ERROR",
            "server_error",
            "network_error",
        ),
        GatewayError::Serialization(_) => http_facts(
            StatusCode::BAD_REQUEST,
            "SERIALIZATION_ERROR",
            "invalid_request_error",
            "invalid_request",
        ),
        GatewayError::Io(_) => http_facts(
            StatusCode::INTERNAL_SERVER_ERROR,
            "IO_ERROR",
            "server_error",
            "internal_error",
        ),
        GatewayError::Auth(_) => http_facts(
            StatusCode::UNAUTHORIZED,
            "AUTH_ERROR",
            "authentication_error",
            "authentication_error",
        ),
        GatewayError::Provider(provider_error) => provider_http_error_facts(provider_error),
        GatewayError::RateLimit { .. } => http_facts(
            StatusCode::TOO_MANY_REQUESTS,
            "RATE_LIMIT_EXCEEDED",
            "rate_limit_error",
            "rate_limit_exceeded",
        ),
        GatewayError::Validation(_) => http_facts(
            StatusCode::BAD_REQUEST,
            "VALIDATION_ERROR",
            "invalid_request_error",
            "invalid_request",
        ),
        GatewayError::Timeout(_) => http_facts(
            StatusCode::REQUEST_TIMEOUT,
            "TIMEOUT",
            "server_error",
            "timeout",
        ),
        GatewayError::NotFound(_) => http_facts(
            StatusCode::NOT_FOUND,
            "NOT_FOUND",
            "invalid_request_error",
            "not_found",
        ),
        GatewayError::Conflict(_) => http_facts(
            StatusCode::CONFLICT,
            "CONFLICT",
            "invalid_request_error",
            "conflict",
        ),
        GatewayError::BadRequest(_) => http_facts(
            StatusCode::BAD_REQUEST,
            "BAD_REQUEST",
            "invalid_request_error",
            "invalid_request",
        ),
        GatewayError::Internal(_) => http_facts(
            StatusCode::INTERNAL_SERVER_ERROR,
            "INTERNAL_ERROR",
            "server_error",
            "internal_error",
        ),
        GatewayError::Unavailable(_) => http_facts(
            StatusCode::SERVICE_UNAVAILABLE,
            "SERVICE_UNAVAILABLE",
            "server_error",
            "service_unavailable",
        ),
        GatewayError::Network(_) => http_facts(
            StatusCode::BAD_GATEWAY,
            "NETWORK_ERROR",
            "server_error",
            "network_error",
        ),
        GatewayError::Forbidden(_) => http_facts(
            StatusCode::FORBIDDEN,
            "FORBIDDEN",
            "permission_error",
            "permission_denied",
        ),
        GatewayError::NotImplemented(_) => http_facts(
            StatusCode::NOT_IMPLEMENTED,
            "NOT_IMPLEMENTED",
            "invalid_request_error",
            "not_implemented",
        ),
    }
}

/// Map provider errors to canonical HTTP status and adapter codes.
#[cfg(feature = "gateway")]
pub fn provider_http_error_facts(error: &ProviderError) -> HttpErrorFacts {
    match error {
        ProviderError::Authentication { .. } => http_facts(
            StatusCode::UNAUTHORIZED,
            "PROVIDER_AUTH_ERROR",
            "authentication_error",
            "authentication_error",
        ),
        ProviderError::RateLimit { .. } => http_facts(
            StatusCode::TOO_MANY_REQUESTS,
            "PROVIDER_RATE_LIMIT",
            "rate_limit_error",
            "rate_limit_exceeded",
        ),
        ProviderError::QuotaExceeded { .. } => http_facts(
            StatusCode::PAYMENT_REQUIRED,
            "PROVIDER_QUOTA_EXCEEDED",
            "insufficient_quota",
            "insufficient_quota",
        ),
        ProviderError::ModelNotFound { .. } => http_facts(
            StatusCode::NOT_FOUND,
            "MODEL_NOT_FOUND",
            "invalid_request_error",
            "model_not_found",
        ),
        ProviderError::InvalidRequest { .. } => http_facts(
            StatusCode::BAD_REQUEST,
            "INVALID_REQUEST",
            "invalid_request_error",
            "invalid_request",
        ),
        ProviderError::Network { .. } => http_facts(
            StatusCode::BAD_GATEWAY,
            "PROVIDER_NETWORK_ERROR",
            "server_error",
            "provider_network_error",
        ),
        ProviderError::ProviderUnavailable { .. } => http_facts(
            StatusCode::SERVICE_UNAVAILABLE,
            "PROVIDER_UNAVAILABLE",
            "server_error",
            "provider_unavailable",
        ),
        ProviderError::NotSupported { .. }
        | ProviderError::NotImplemented { .. }
        | ProviderError::FeatureDisabled { .. } => http_facts(
            StatusCode::NOT_IMPLEMENTED,
            "PROVIDER_NOT_IMPLEMENTED",
            "invalid_request_error",
            "not_supported",
        ),
        ProviderError::Configuration { .. }
        | ProviderError::Serialization { .. }
        | ProviderError::TransformationError { .. } => http_facts(
            StatusCode::INTERNAL_SERVER_ERROR,
            "PROVIDER_INTERNAL_ERROR",
            "server_error",
            "internal_error",
        ),
        ProviderError::Timeout { .. } => http_facts(
            StatusCode::GATEWAY_TIMEOUT,
            "PROVIDER_TIMEOUT",
            "server_error",
            "timeout",
        ),
        ProviderError::ContextLengthExceeded { .. } => http_facts(
            StatusCode::BAD_REQUEST,
            "PROVIDER_REQUEST_ERROR",
            "invalid_request_error",
            "context_length_exceeded",
        ),
        ProviderError::ContentFiltered { .. } => http_facts(
            StatusCode::BAD_REQUEST,
            "PROVIDER_REQUEST_ERROR",
            "invalid_request_error",
            "content_filter",
        ),
        ProviderError::ApiError { status, .. } => provider_api_error_http_facts(*status),
        ProviderError::TokenLimitExceeded { .. } => http_facts(
            StatusCode::BAD_REQUEST,
            "PROVIDER_REQUEST_ERROR",
            "invalid_request_error",
            "token_limit_exceeded",
        ),
        ProviderError::DeploymentError { .. } => http_facts(
            StatusCode::NOT_FOUND,
            "DEPLOYMENT_NOT_FOUND",
            "invalid_request_error",
            "deployment_not_found",
        ),
        ProviderError::ResponseParsing { .. } | ProviderError::Streaming { .. } => http_facts(
            StatusCode::BAD_GATEWAY,
            "PROVIDER_RESPONSE_ERROR",
            "server_error",
            "provider_response_error",
        ),
        ProviderError::RoutingError { .. } => http_facts(
            StatusCode::SERVICE_UNAVAILABLE,
            "PROVIDER_ROUTING_ERROR",
            "server_error",
            "provider_routing_error",
        ),
        ProviderError::Cancelled { .. } => http_facts(
            client_closed_request(),
            "PROVIDER_CANCELLED",
            "server_error",
            "cancelled",
        ),
        ProviderError::Other { .. } => http_facts(
            StatusCode::BAD_GATEWAY,
            "PROVIDER_ERROR",
            "server_error",
            "provider_error",
        ),
    }
}

#[cfg(feature = "gateway")]
pub fn gateway_http_header_facts(error: &GatewayError) -> Option<HttpHeaderFacts> {
    match error {
        GatewayError::RateLimit {
            retry_after,
            rpm_limit,
            tpm_limit,
            ..
        } => Some(HttpHeaderFacts {
            retry_after: *retry_after,
            rpm_limit: *rpm_limit,
            tpm_limit: *tpm_limit,
        }),
        GatewayError::Provider(ProviderError::RateLimit {
            retry_after,
            rpm_limit,
            tpm_limit,
            ..
        }) => Some(HttpHeaderFacts {
            retry_after: *retry_after,
            rpm_limit: *rpm_limit,
            tpm_limit: *tpm_limit,
        }),
        _ => None,
    }
}

#[cfg(feature = "gateway")]
fn provider_api_error_http_facts(status: u16) -> HttpErrorFacts {
    let status_code = StatusCode::from_u16(status).unwrap_or(StatusCode::BAD_GATEWAY);
    match status {
        400 => http_facts(
            status_code,
            "PROVIDER_API_ERROR",
            "invalid_request_error",
            "invalid_request",
        ),
        401 => http_facts(
            status_code,
            "PROVIDER_API_ERROR",
            "authentication_error",
            "authentication_error",
        ),
        403 => http_facts(
            status_code,
            "PROVIDER_API_ERROR",
            "permission_error",
            "permission_denied",
        ),
        404 => http_facts(
            status_code,
            "PROVIDER_API_ERROR",
            "invalid_request_error",
            "not_found",
        ),
        408 => http_facts(status_code, "PROVIDER_API_ERROR", "server_error", "timeout"),
        409 => http_facts(
            status_code,
            "PROVIDER_API_ERROR",
            "invalid_request_error",
            "conflict",
        ),
        429 => http_facts(
            status_code,
            "PROVIDER_API_ERROR",
            "rate_limit_error",
            "rate_limit_exceeded",
        ),
        500..=599 => http_facts(
            status_code,
            "PROVIDER_API_ERROR",
            "server_error",
            "provider_api_error",
        ),
        _ => http_facts(
            status_code,
            "PROVIDER_API_ERROR",
            "server_error",
            "provider_api_error",
        ),
    }
}

#[cfg(feature = "gateway")]
fn http_facts(
    status: StatusCode,
    gateway_code: &'static str,
    openai_error_type: &'static str,
    openai_code: &'static str,
) -> HttpErrorFacts {
    HttpErrorFacts::new(status, gateway_code, openai_error_type, openai_code)
}

#[cfg(feature = "gateway")]
fn client_closed_request() -> StatusCode {
    StatusCode::from_u16(499).unwrap_or(StatusCode::BAD_REQUEST)
}

impl CanonicalError for ProviderError {
    fn canonical_code(&self) -> ErrorCode {
        match self {
            ProviderError::Authentication { .. } => ErrorCode::Authentication,
            ProviderError::RateLimit { .. } => ErrorCode::RateLimited,
            ProviderError::QuotaExceeded { .. } => ErrorCode::QuotaExceeded,
            ProviderError::ModelNotFound { .. } | ProviderError::DeploymentError { .. } => {
                ErrorCode::NotFound
            }
            ProviderError::InvalidRequest { .. }
            | ProviderError::ContextLengthExceeded { .. }
            | ProviderError::ContentFiltered { .. }
            | ProviderError::TokenLimitExceeded { .. }
            | ProviderError::FeatureDisabled { .. }
            | ProviderError::Cancelled { .. } => ErrorCode::InvalidRequest,
            ProviderError::Network { .. } => ErrorCode::Network,
            ProviderError::ProviderUnavailable { .. } | ProviderError::RoutingError { .. } => {
                ErrorCode::Unavailable
            }
            ProviderError::NotSupported { .. } | ProviderError::NotImplemented { .. } => {
                ErrorCode::NotImplemented
            }
            ProviderError::Configuration { .. } => ErrorCode::Configuration,
            ProviderError::Serialization { .. }
            | ProviderError::ResponseParsing { .. }
            | ProviderError::TransformationError { .. } => ErrorCode::Parsing,
            ProviderError::Timeout { .. } => ErrorCode::Timeout,
            ProviderError::ApiError { status, .. } => match *status {
                401 => ErrorCode::Authentication,
                403 => ErrorCode::Authorization,
                404 => ErrorCode::NotFound,
                408 | 504 => ErrorCode::Timeout,
                409 => ErrorCode::Conflict,
                429 => ErrorCode::RateLimited,
                400..=499 => ErrorCode::InvalidRequest,
                500..=599 => ErrorCode::Unavailable,
                _ => ErrorCode::Internal,
            },
            ProviderError::Streaming { .. } | ProviderError::Other { .. } => ErrorCode::Internal,
        }
    }

    fn canonical_retryable(&self) -> bool {
        self.is_retryable()
    }
}

impl CanonicalError for GatewayError {
    fn canonical_code(&self) -> ErrorCode {
        match self {
            GatewayError::Config(_) => ErrorCode::Configuration,
            GatewayError::Auth(_) => ErrorCode::Authentication,
            GatewayError::Forbidden(_) => ErrorCode::Authorization,
            GatewayError::Provider(provider_error) => provider_error.canonical_code(),
            GatewayError::RateLimit { .. } => ErrorCode::RateLimited,
            GatewayError::Validation(_) | GatewayError::BadRequest(_) => ErrorCode::InvalidRequest,
            GatewayError::NotFound(_) => ErrorCode::NotFound,
            GatewayError::Conflict(_) => ErrorCode::Conflict,
            GatewayError::Timeout(_) => ErrorCode::Timeout,
            GatewayError::Unavailable(_) => ErrorCode::Unavailable,
            GatewayError::Network(_) => ErrorCode::Network,
            GatewayError::NotImplemented(_) => ErrorCode::NotImplemented,
            GatewayError::Storage(_)
            | GatewayError::HttpClient(_)
            | GatewayError::Serialization(_)
            | GatewayError::Io(_)
            | GatewayError::Internal(_) => ErrorCode::Internal,
        }
    }

    fn canonical_retryable(&self) -> bool {
        match self {
            GatewayError::Provider(provider_error) => provider_error.canonical_retryable(),
            _ => self.canonical_code().is_retryable(),
        }
    }
}

impl CanonicalError for A2AError {
    fn canonical_code(&self) -> ErrorCode {
        match self {
            A2AError::AgentNotFound { .. } | A2AError::TaskNotFound { .. } => ErrorCode::NotFound,
            A2AError::AgentAlreadyExists { .. } => ErrorCode::Conflict,
            A2AError::ConnectionError { .. } => ErrorCode::Network,
            A2AError::AuthenticationError { .. } => ErrorCode::Authentication,
            A2AError::ProtocolError { .. }
            | A2AError::InvalidRequest { .. }
            | A2AError::ContentBlocked { .. } => ErrorCode::InvalidRequest,
            A2AError::Timeout { .. } => ErrorCode::Timeout,
            A2AError::ConfigurationError { .. } => ErrorCode::Configuration,
            A2AError::SerializationError { .. } => ErrorCode::Parsing,
            A2AError::UnsupportedProvider { .. } => ErrorCode::NotImplemented,
            A2AError::RateLimitExceeded { .. } => ErrorCode::RateLimited,
            A2AError::AgentBusy { .. } => ErrorCode::Unavailable,
            A2AError::TaskFailed { .. } => ErrorCode::Internal,
        }
    }

    fn canonical_retryable(&self) -> bool {
        matches!(
            self,
            A2AError::ConnectionError { .. }
                | A2AError::Timeout { .. }
                | A2AError::RateLimitExceeded { .. }
                | A2AError::AgentBusy { .. }
        )
    }
}

impl CanonicalError for McpError {
    fn canonical_code(&self) -> ErrorCode {
        match self {
            McpError::ServerNotFound { .. } | McpError::ToolNotFound { .. } => ErrorCode::NotFound,
            McpError::ConnectionError { .. } | McpError::TransportError { .. } => {
                ErrorCode::Network
            }
            McpError::AuthenticationError { .. } => ErrorCode::Authentication,
            McpError::AuthorizationError { .. } => ErrorCode::Authorization,
            McpError::ProtocolError { .. } | McpError::InvalidUrl { .. } => {
                ErrorCode::InvalidRequest
            }
            McpError::ToolExecutionError { .. } => ErrorCode::Internal,
            McpError::Timeout { .. } => ErrorCode::Timeout,
            McpError::ConfigurationError { .. } => ErrorCode::Configuration,
            McpError::SerializationError { .. } => ErrorCode::Parsing,
            McpError::ServerAlreadyExists { .. } | McpError::ToolDefinitionChanged { .. } => {
                ErrorCode::Conflict
            }
            McpError::RateLimitExceeded { .. } => ErrorCode::RateLimited,
            McpError::ValidationError { .. } => ErrorCode::InvalidRequest,
        }
    }

    fn canonical_retryable(&self) -> bool {
        matches!(
            self,
            McpError::ConnectionError { .. }
                | McpError::TransportError { .. }
                | McpError::Timeout { .. }
                | McpError::RateLimitExceeded { .. }
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_provider_rate_limit_mapping() {
        let err = ProviderError::rate_limit("openai", Some(10));
        assert_eq!(err.canonical_code(), ErrorCode::RateLimited);
        assert!(err.canonical_retryable());
    }

    #[test]
    fn test_provider_auth_mapping() {
        let err = ProviderError::authentication("openai", "bad key");
        assert_eq!(err.canonical_code(), ErrorCode::Authentication);
        assert!(!err.canonical_retryable());
    }

    #[test]
    fn test_gateway_provider_delegates_retryable() {
        let err = GatewayError::Provider(ProviderError::timeout("openai", "timeout"));
        assert_eq!(err.canonical_code(), ErrorCode::Timeout);
        assert!(err.canonical_retryable());
    }

    #[test]
    fn test_gateway_not_found_mapping() {
        let err = GatewayError::NotFound("missing".to_string());
        assert_eq!(err.canonical_code(), ErrorCode::NotFound);
        assert!(!err.canonical_retryable());
    }

    #[cfg(feature = "s3")]
    #[test]
    fn test_gateway_s3_mapping() {
        let err = GatewayError::Storage("bucket error".to_string());
        assert_eq!(err.canonical_code(), ErrorCode::Internal);
        assert!(!err.canonical_retryable());
    }

    #[cfg(feature = "vector-db")]
    #[test]
    fn test_gateway_qdrant_mapping() {
        let err = GatewayError::Storage("connection failed".to_string());
        assert_eq!(err.canonical_code(), ErrorCode::Internal);
        assert!(!err.canonical_retryable());
    }

    #[cfg(feature = "websockets")]
    #[test]
    fn test_gateway_websocket_mapping() {
        let err = GatewayError::Network("connection closed".to_string());
        assert_eq!(err.canonical_code(), ErrorCode::Network);
        assert!(err.canonical_retryable());
    }

    #[test]
    fn test_a2a_busy_mapping() {
        let err = A2AError::AgentBusy {
            agent_name: "agent-1".to_string(),
            message: "overloaded".to_string(),
        };
        assert_eq!(err.canonical_code(), ErrorCode::Unavailable);
        assert!(err.canonical_retryable());
    }

    #[test]
    fn test_mcp_auth_mapping() {
        let err = McpError::AuthenticationError {
            server_name: "s1".to_string(),
            message: "bad token".to_string(),
        };
        assert_eq!(err.canonical_code(), ErrorCode::Authentication);
        assert!(!err.canonical_retryable());
    }

    #[test]
    fn test_error_code_str_values() {
        assert_eq!(ErrorCode::Authentication.as_str(), "AUTHENTICATION");
        assert_eq!(ErrorCode::RateLimited.as_str(), "RATE_LIMITED");
    }
}
