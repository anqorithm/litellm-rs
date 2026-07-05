//! Canonical HTTP mapping for gateway/provider errors.

use super::types::GatewayError;
use crate::core::providers::unified_provider::{
    ProviderHttpErrorFacts, provider_http_error_facts as provider_core_http_error_facts,
};
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
        GatewayError::Provider(provider_error) => {
            provider_facts_to_http_error_facts(provider_core_http_error_facts(provider_error))
        }
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

const fn facts(
    status: StatusCode,
    gateway_code: &'static str,
    openai_error_type: &'static str,
    openai_code: &'static str,
) -> HttpErrorFacts {
    HttpErrorFacts::new(status, gateway_code, openai_error_type, openai_code)
}

fn provider_facts_to_http_error_facts(core_facts: ProviderHttpErrorFacts) -> HttpErrorFacts {
    HttpErrorFacts::new(
        status_from_u16(core_facts.status),
        core_facts.gateway_code,
        core_facts.openai_error_type,
        core_facts.openai_code,
    )
    .with_headers(HttpErrorHeaders {
        retry_after: core_facts.headers.retry_after,
        rpm_limit: core_facts.headers.rpm_limit,
        tpm_limit: core_facts.headers.tpm_limit,
    })
}

fn status_from_u16(status: u16) -> StatusCode {
    StatusCode::from_u16(status).unwrap_or(StatusCode::BAD_GATEWAY)
}
