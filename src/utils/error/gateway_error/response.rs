//! HTTP response handling for errors

use super::http_mapping::{HttpErrorHeaders, gateway_http_error_facts};
use super::types::GatewayError;
use crate::utils::error::canonical::CanonicalError;
use actix_web::{HttpResponse, HttpResponseBuilder, ResponseError};
use std::future::Future;

tokio::task_local! {
    static CURRENT_REQUEST_ID: String;
}

pub async fn with_error_response_request_id<F>(request_id: String, future: F) -> F::Output
where
    F: Future,
{
    CURRENT_REQUEST_ID.scope(request_id, future).await
}

pub fn current_error_response_request_id() -> Option<String> {
    CURRENT_REQUEST_ID.try_with(Clone::clone).ok()
}

fn insert_error_headers(builder: &mut HttpResponseBuilder, headers: HttpErrorHeaders) {
    if let Some(secs) = headers.retry_after {
        builder.insert_header(("Retry-After", secs.to_string()));
    }
    if let Some(rpm) = headers.rpm_limit {
        builder.insert_header(("X-RateLimit-Limit-Requests", rpm.to_string()));
    }
    if let Some(tpm) = headers.tpm_limit {
        builder.insert_header(("X-RateLimit-Limit-Tokens", tpm.to_string()));
    }
}

impl GatewayError {
    pub fn error_response_with_request_id(&self, request_id: Option<String>) -> HttpResponse {
        let facts = gateway_http_error_facts(self);
        let message = self.to_string();

        let canonical_code = self.canonical_code().as_str().to_string();
        let retryable = self.canonical_retryable();

        let error_response = GatewayErrorResponse {
            error: GatewayErrorDetail {
                code: facts.gateway_code.to_string(),
                canonical_code,
                retryable,
                message,
                timestamp: chrono::Utc::now().timestamp(),
                request_id,
            },
        };

        let mut builder = HttpResponse::build(facts.status);
        insert_error_headers(&mut builder, facts.headers);

        builder.json(error_response)
    }
}

impl ResponseError for GatewayError {
    fn error_response(&self) -> HttpResponse {
        self.error_response_with_request_id(current_error_response_request_id())
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
