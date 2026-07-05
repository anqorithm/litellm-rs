//! HTTP response handling for errors

use super::types::GatewayError;
use crate::utils::error::canonical::{
    CanonicalError, HttpHeaderFacts, gateway_http_error_facts, gateway_http_header_facts,
};
use actix_web::{HttpResponse, HttpResponseBuilder, ResponseError};
use std::future::Future;

tokio::task_local! {
    static CURRENT_GATEWAY_ERROR_REQUEST_ID: String;
}

/// Run a request future with the request ID available to `GatewayError` bodies.
pub async fn with_gateway_error_request_id<F>(request_id: String, future: F) -> F::Output
where
    F: Future,
{
    CURRENT_GATEWAY_ERROR_REQUEST_ID
        .scope(request_id, future)
        .await
}

fn current_gateway_error_request_id() -> Option<String> {
    CURRENT_GATEWAY_ERROR_REQUEST_ID.try_with(Clone::clone).ok()
}

fn insert_http_headers(builder: &mut HttpResponseBuilder, facts: HttpHeaderFacts) {
    if let Some(secs) = facts.retry_after {
        builder.insert_header(("Retry-After", secs.to_string()));
    }
    if let Some(rpm) = facts.rpm_limit {
        builder.insert_header(("X-RateLimit-Limit-Requests", rpm.to_string()));
    }
    if let Some(tpm) = facts.tpm_limit {
        builder.insert_header(("X-RateLimit-Limit-Tokens", tpm.to_string()));
    }
}

impl GatewayError {
    /// Build the gateway JSON error body with a request ID supplied by middleware.
    pub fn error_response_with_request_id(&self, request_id: Option<String>) -> HttpResponse {
        self.error_response_with_optional_request_id(request_id)
    }

    fn error_response_with_optional_request_id(&self, request_id: Option<String>) -> HttpResponse {
        let request_id = request_id.or_else(current_gateway_error_request_id);
        let facts = gateway_http_error_facts(self);
        let canonical_code = self.canonical_code().as_str().to_string();
        let retryable = self.canonical_retryable();

        let error_response = GatewayErrorResponse {
            error: GatewayErrorDetail {
                code: facts.gateway_code.to_string(),
                canonical_code,
                retryable,
                message: self.to_string(),
                timestamp: chrono::Utc::now().timestamp(),
                request_id,
            },
        };

        let mut builder = HttpResponse::build(facts.status);

        if let Some(header_facts) = gateway_http_header_facts(self) {
            insert_http_headers(&mut builder, header_facts);
        }

        builder.json(error_response)
    }
}

impl ResponseError for GatewayError {
    fn error_response(&self) -> HttpResponse {
        self.error_response_with_optional_request_id(None)
    }
}

/// Standard gateway error response format
#[derive(serde::Serialize)]
pub struct GatewayErrorResponse {
    pub error: GatewayErrorDetail,
}

/// Gateway error detail structure
#[derive(serde::Serialize)]
pub struct GatewayErrorDetail {
    pub code: String,
    pub canonical_code: String,
    pub retryable: bool,
    pub message: String,
    pub timestamp: i64,
    pub request_id: Option<String>,
}

#[cfg(test)]
#[path = "response_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "response_consolidation_tests.rs"]
mod consolidation_tests;
