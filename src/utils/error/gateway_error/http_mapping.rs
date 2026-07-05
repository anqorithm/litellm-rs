//! Canonical HTTP mapping for gateway/provider errors.

use super::types::GatewayError;
use crate::core::providers::unified_provider::ProviderError;
use actix_web::http::StatusCode;

/// Optional HTTP headers carried by an error mapping.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct HttpErrorHeaders {
    pub retry_after: Option<u64>,
    pub rpm_limit: Option<u32>,
    pub tpm_limit: Option<u32>,
}

/// Protocol-neutral HTTP facts shared by response adapters.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HttpErrorFacts {
    pub status: StatusCode,
    pub gateway_code: &'static str,
    pub openai_error_type: &'static str,
    pub openai_code: &'static str,
    pub headers: HttpErrorHeaders,
}

impl HttpErrorFacts {
    const fn new(
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
            headers: HttpErrorHeaders {
                retry_after: None,
                rpm_limit: None,
                tpm_limit: None,
            },
        }
    }

    const fn with_headers(mut self, headers: HttpErrorHeaders) -> Self {
        self.headers = headers;
        self
    }
}

pub fn gateway_http_error_facts(error: &GatewayError) -> HttpErrorFacts {
    match error {
        GatewayError::Config(_) => facts(
            StatusCode::INTERNAL_SERVER_ERROR,
            "CONFIG_ERROR",
            "server_error",
            "internal_error",
        ),
        GatewayError::Storage(_) => facts(
            StatusCode::SERVICE_UNAVAILABLE,
            "STORAGE_ERROR",
            "server_error",
            "service_unavailable",
        ),
        GatewayError::Auth(_) => facts(
            StatusCode::UNAUTHORIZED,
            "AUTH_ERROR",
            "authentication_error",
            "authentication_error",
        ),
        GatewayError::Forbidden(_) => facts(
            StatusCode::FORBIDDEN,
            "FORBIDDEN",
            "permission_error",
            "permission_denied",
        ),
        GatewayError::Provider(provider_error) => provider_http_error_facts(provider_error),
        GatewayError::RateLimit {
            retry_after,
            rpm_limit,
            tpm_limit,
            ..
        } => facts(
            StatusCode::TOO_MANY_REQUESTS,
            "RATE_LIMIT_EXCEEDED",
            "rate_limit_error",
            "rate_limit_exceeded",
        )
        .with_headers(HttpErrorHeaders {
            retry_after: *retry_after,
            rpm_limit: *rpm_limit,
            tpm_limit: *tpm_limit,
        }),
        GatewayError::Validation(_) => facts(
            StatusCode::BAD_REQUEST,
            "VALIDATION_ERROR",
            "invalid_request_error",
            "invalid_request",
        ),
        GatewayError::NotFound(_) => facts(
            StatusCode::NOT_FOUND,
            "NOT_FOUND",
            "invalid_request_error",
            "not_found",
        ),
        GatewayError::Conflict(_) => facts(
            StatusCode::CONFLICT,
            "CONFLICT",
            "invalid_request_error",
            "conflict",
        ),
        GatewayError::BadRequest(_) => facts(
            StatusCode::BAD_REQUEST,
            "BAD_REQUEST",
            "invalid_request_error",
            "invalid_request",
        ),
        GatewayError::Timeout(_) => facts(
            StatusCode::REQUEST_TIMEOUT,
            "TIMEOUT",
            "server_error",
            "timeout",
        ),
        GatewayError::Unavailable(_) => facts(
            StatusCode::SERVICE_UNAVAILABLE,
            "SERVICE_UNAVAILABLE",
            "server_error",
            "service_unavailable",
        ),
        GatewayError::Network(_) => facts(
            StatusCode::BAD_GATEWAY,
            "NETWORK_ERROR",
            "server_error",
            "network_error",
        ),
        GatewayError::Internal(_) => facts(
            StatusCode::INTERNAL_SERVER_ERROR,
            "INTERNAL_ERROR",
            "server_error",
            "internal_error",
        ),
        GatewayError::NotImplemented(_) => facts(
            StatusCode::NOT_IMPLEMENTED,
            "NOT_IMPLEMENTED",
            "invalid_request_error",
            "not_implemented",
        ),
        GatewayError::Serialization(_) => facts(
            StatusCode::BAD_REQUEST,
            "SERIALIZATION_ERROR",
            "invalid_request_error",
            "invalid_request",
        ),
        GatewayError::HttpClient(_) => facts(
            StatusCode::BAD_GATEWAY,
            "HTTP_CLIENT_ERROR",
            "server_error",
            "network_error",
        ),
        GatewayError::Io(_) => facts(
            StatusCode::INTERNAL_SERVER_ERROR,
            "IO_ERROR",
            "server_error",
            "internal_error",
        ),
    }
}

pub fn provider_http_error_facts(error: &ProviderError) -> HttpErrorFacts {
    match error {
        ProviderError::Authentication { .. } => facts(
            StatusCode::UNAUTHORIZED,
            "PROVIDER_AUTH_ERROR",
            "authentication_error",
            "authentication_error",
        ),
        ProviderError::RateLimit {
            retry_after,
            rpm_limit,
            tpm_limit,
            ..
        } => facts(
            StatusCode::TOO_MANY_REQUESTS,
            "PROVIDER_RATE_LIMIT",
            "rate_limit_error",
            "rate_limit_exceeded",
        )
        .with_headers(HttpErrorHeaders {
            retry_after: *retry_after,
            rpm_limit: *rpm_limit,
            tpm_limit: *tpm_limit,
        }),
        ProviderError::QuotaExceeded { .. } => facts(
            StatusCode::PAYMENT_REQUIRED,
            "PROVIDER_QUOTA_EXCEEDED",
            "insufficient_quota",
            "insufficient_quota",
        ),
        ProviderError::ModelNotFound { .. } => facts(
            StatusCode::NOT_FOUND,
            "MODEL_NOT_FOUND",
            "invalid_request_error",
            "model_not_found",
        ),
        ProviderError::InvalidRequest { message, .. } if is_model_not_priced_message(message) => {
            facts(
                StatusCode::BAD_REQUEST,
                "INVALID_REQUEST",
                "invalid_request_error",
                "model_not_priced",
            )
        }
        ProviderError::InvalidRequest { .. } => facts(
            StatusCode::BAD_REQUEST,
            "INVALID_REQUEST",
            "invalid_request_error",
            "invalid_request",
        ),
        ProviderError::Network { .. } => facts(
            StatusCode::BAD_GATEWAY,
            "PROVIDER_NETWORK_ERROR",
            "server_error",
            "provider_network_error",
        ),
        ProviderError::ProviderUnavailable { .. } => facts(
            StatusCode::SERVICE_UNAVAILABLE,
            "PROVIDER_UNAVAILABLE",
            "server_error",
            "provider_unavailable",
        ),
        ProviderError::NotSupported { .. }
        | ProviderError::NotImplemented { .. }
        | ProviderError::FeatureDisabled { .. } => facts(
            StatusCode::NOT_IMPLEMENTED,
            "PROVIDER_NOT_IMPLEMENTED",
            "invalid_request_error",
            "not_supported",
        ),
        ProviderError::Configuration { .. }
        | ProviderError::Serialization { .. }
        | ProviderError::TransformationError { .. } => facts(
            StatusCode::INTERNAL_SERVER_ERROR,
            "PROVIDER_INTERNAL_ERROR",
            "server_error",
            "internal_error",
        ),
        ProviderError::Timeout { .. } => facts(
            StatusCode::GATEWAY_TIMEOUT,
            "PROVIDER_TIMEOUT",
            "server_error",
            "timeout",
        ),
        ProviderError::ContextLengthExceeded { .. } => facts(
            StatusCode::BAD_REQUEST,
            "PROVIDER_REQUEST_ERROR",
            "invalid_request_error",
            "context_length_exceeded",
        ),
        ProviderError::ContentFiltered { .. } => facts(
            StatusCode::BAD_REQUEST,
            "PROVIDER_REQUEST_ERROR",
            "invalid_request_error",
            "content_filter",
        ),
        ProviderError::ApiError { status, .. } => provider_api_error_facts(*status),
        ProviderError::TokenLimitExceeded { .. } => facts(
            StatusCode::BAD_REQUEST,
            "PROVIDER_REQUEST_ERROR",
            "invalid_request_error",
            "token_limit_exceeded",
        ),
        ProviderError::DeploymentError { .. } => facts(
            StatusCode::NOT_FOUND,
            "DEPLOYMENT_NOT_FOUND",
            "invalid_request_error",
            "deployment_not_found",
        ),
        ProviderError::ResponseParsing { .. } | ProviderError::Streaming { .. } => facts(
            StatusCode::BAD_GATEWAY,
            "PROVIDER_RESPONSE_ERROR",
            "server_error",
            "provider_response_error",
        ),
        ProviderError::RoutingError { .. } => facts(
            StatusCode::SERVICE_UNAVAILABLE,
            "PROVIDER_ROUTING_ERROR",
            "server_error",
            "provider_routing_error",
        ),
        ProviderError::Cancelled { .. } => facts(
            client_closed_request_status(),
            "PROVIDER_CANCELLED",
            "server_error",
            "cancelled",
        ),
        ProviderError::Other { .. } => facts(
            StatusCode::BAD_GATEWAY,
            "PROVIDER_ERROR",
            "server_error",
            "provider_error",
        ),
    }
}

fn provider_api_error_facts(status: u16) -> HttpErrorFacts {
    let status_code = StatusCode::from_u16(status).unwrap_or(StatusCode::BAD_GATEWAY);
    match status {
        400 => facts(
            status_code,
            "PROVIDER_API_ERROR",
            "invalid_request_error",
            "invalid_request",
        ),
        401 => facts(
            status_code,
            "PROVIDER_API_ERROR",
            "authentication_error",
            "authentication_error",
        ),
        403 => facts(
            status_code,
            "PROVIDER_API_ERROR",
            "permission_error",
            "permission_denied",
        ),
        404 => facts(
            status_code,
            "PROVIDER_API_ERROR",
            "invalid_request_error",
            "not_found",
        ),
        408 => facts(status_code, "PROVIDER_API_ERROR", "server_error", "timeout"),
        409 => facts(
            status_code,
            "PROVIDER_API_ERROR",
            "invalid_request_error",
            "conflict",
        ),
        429 => facts(
            status_code,
            "PROVIDER_API_ERROR",
            "rate_limit_error",
            "rate_limit_exceeded",
        ),
        500..=599 => facts(
            status_code,
            "PROVIDER_API_ERROR",
            "server_error",
            "provider_api_error",
        ),
        _ => facts(
            status_code,
            "PROVIDER_API_ERROR",
            "server_error",
            "provider_api_error",
        ),
    }
}

const fn facts(
    status: StatusCode,
    gateway_code: &'static str,
    openai_error_type: &'static str,
    openai_code: &'static str,
) -> HttpErrorFacts {
    HttpErrorFacts::new(status, gateway_code, openai_error_type, openai_code)
}

fn client_closed_request_status() -> StatusCode {
    StatusCode::from_u16(499).unwrap_or(StatusCode::BAD_REQUEST)
}

fn is_model_not_priced_message(message: &str) -> bool {
    message.starts_with("model_not_priced:")
}
