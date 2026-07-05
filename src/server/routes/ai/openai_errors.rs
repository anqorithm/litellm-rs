//! OpenAI-compatible error response helpers for `/v1/*` endpoints.

use crate::core::providers::ProviderError;
use crate::utils::error::canonical::{gateway_http_error_facts, gateway_http_header_facts};
use crate::utils::error::gateway_error::GatewayError;
use actix_web::http::StatusCode;
use actix_web::{HttpResponse, http::header};
use serde::{Deserialize, Serialize};

#[derive(Serialize)]
struct OpenAiErrorResponse {
    error: OpenAiErrorDetail,
}

#[derive(Serialize)]
struct OpenAiErrorDetail {
    message: String,
    #[serde(rename = "type")]
    error_type: String,
    param: Option<String>,
    code: Option<String>,
}

struct OpenAiErrorSpec {
    status: StatusCode,
    message: String,
    error_type: String,
    param: Option<String>,
    code: Option<String>,
}

#[derive(Deserialize)]
struct UpstreamErrorEnvelope {
    error: UpstreamErrorDetail,
}

#[derive(Deserialize)]
struct UpstreamErrorDetail {
    message: Option<String>,
    #[serde(rename = "type")]
    error_type: Option<String>,
    param: Option<String>,
    code: Option<serde_json::Value>,
}

pub(crate) fn validation_error(message: impl Into<String>) -> HttpResponse {
    build_response(spec(
        StatusCode::BAD_REQUEST,
        message.into(),
        "invalid_request_error",
        "invalid_request",
    ))
}

pub(crate) fn unauthorized_error(message: impl Into<String>) -> HttpResponse {
    build_response(spec(
        StatusCode::UNAUTHORIZED,
        message.into(),
        "authentication_error",
        "authentication_error",
    ))
}

pub(crate) fn gateway_error_response(error: &GatewayError) -> HttpResponse {
    let spec = gateway_error_spec(error);
    let mut builder = HttpResponse::build(spec.status);

    if let Some(facts) = gateway_http_header_facts(error) {
        if let Some(secs) = facts.retry_after {
            builder.insert_header((header::RETRY_AFTER, secs.to_string()));
        }
        if let Some(rpm) = facts.rpm_limit {
            builder.insert_header(("X-RateLimit-Limit-Requests", rpm.to_string()));
        }
        if let Some(tpm) = facts.tpm_limit {
            builder.insert_header(("X-RateLimit-Limit-Tokens", tpm.to_string()));
        }
    }

    builder.json(response_body(
        spec.message,
        spec.error_type,
        spec.param,
        spec.code,
    ))
}

fn build_response(spec: OpenAiErrorSpec) -> HttpResponse {
    HttpResponse::build(spec.status).json(response_body(
        spec.message,
        spec.error_type,
        spec.param,
        spec.code,
    ))
}

fn response_body(
    message: String,
    error_type: String,
    param: Option<String>,
    code: Option<String>,
) -> OpenAiErrorResponse {
    OpenAiErrorResponse {
        error: OpenAiErrorDetail {
            message,
            error_type,
            param,
            code,
        },
    }
}

fn gateway_error_spec(error: &GatewayError) -> OpenAiErrorSpec {
    let facts = gateway_http_error_facts(error);
    let message = match error {
        GatewayError::Provider(ProviderError::ApiError { message, .. }) => message.clone(),
        _ => error.to_string(),
    };
    let mut spec = spec(
        facts.status,
        message,
        facts.openai_error_type,
        facts.openai_code,
    );

    if let GatewayError::Provider(ProviderError::InvalidRequest { message, .. }) = error
        && super::spend::is_model_not_priced_message(message)
    {
        spec.code = Some("model_not_priced".to_string());
    }

    if let GatewayError::Provider(ProviderError::ApiError { .. }) = error {
        apply_upstream_error_detail(&mut spec);
    }

    spec
}

fn apply_upstream_error_detail(spec: &mut OpenAiErrorSpec) {
    if let Some(upstream) = parse_upstream_error_detail(&spec.message) {
        if let Some(message) = upstream.message {
            spec.message = message;
        }
        if let Some(error_type) = upstream.error_type {
            spec.error_type = error_type;
        }
        if let Some(param) = upstream.param {
            spec.param = Some(param);
        }
        if let Some(code) = upstream.code.and_then(error_code_to_string) {
            spec.code = Some(code);
        }
    }
}

fn parse_upstream_error_detail(message: &str) -> Option<UpstreamErrorDetail> {
    serde_json::from_str::<UpstreamErrorEnvelope>(message)
        .ok()
        .map(|envelope| envelope.error)
}

fn error_code_to_string(value: serde_json::Value) -> Option<String> {
    match value {
        serde_json::Value::String(value) if !value.is_empty() => Some(value),
        serde_json::Value::Number(value) => Some(value.to_string()),
        _ => None,
    }
}

fn spec(
    status: StatusCode,
    message: String,
    error_type: &'static str,
    code: &'static str,
) -> OpenAiErrorSpec {
    OpenAiErrorSpec {
        status,
        message,
        error_type: error_type.to_string(),
        param: None,
        code: Some(code.to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::ResponseError;
    use actix_web::body::to_bytes;
    use serde_json::Value;

    #[actix_web::test]
    async fn validation_error_uses_openai_shape() {
        let response = validation_error("model must not be empty");

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        let body = to_json(response).await;
        assert_eq!(body["error"]["message"], "model must not be empty");
        assert_eq!(body["error"]["type"], "invalid_request_error");
        assert_eq!(body["error"]["param"], Value::Null);
        assert_eq!(body["error"]["code"], "invalid_request");
        assert!(body.get("success").is_none());
    }

    #[actix_web::test]
    async fn config_error_remains_internal_server_error() {
        let error = GatewayError::Config("Invalid config".to_string());

        let response = gateway_error_response(&error);

        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
        let body = to_json(response).await;
        assert_eq!(
            body["error"]["message"],
            "Configuration error: Invalid config"
        );
        assert_eq!(body["error"]["type"], "server_error");
        assert_eq!(body["error"]["code"], "internal_error");
    }

    #[actix_web::test]
    async fn provider_rate_limit_uses_openai_shape_and_retry_after() {
        let error = GatewayError::Provider(ProviderError::RateLimit {
            provider: "openai",
            message: "Rate limit exceeded".to_string(),
            retry_after: Some(2),
            rpm_limit: Some(120),
            tpm_limit: Some(60000),
            current_usage: None,
        });

        let response = gateway_error_response(&error);

        assert_eq!(response.status(), StatusCode::TOO_MANY_REQUESTS);
        assert_eq!(response.headers().get(header::RETRY_AFTER).unwrap(), "2");
        assert_eq!(
            response
                .headers()
                .get("X-RateLimit-Limit-Requests")
                .and_then(|value| value.to_str().ok()),
            Some("120")
        );
        assert_eq!(
            response
                .headers()
                .get("X-RateLimit-Limit-Tokens")
                .and_then(|value| value.to_str().ok()),
            Some("60000")
        );
        let body = to_json(response).await;
        assert_eq!(body["error"]["type"], "rate_limit_error");
        assert_eq!(body["error"]["code"], "rate_limit_exceeded");
        assert!(
            body["error"]["message"]
                .as_str()
                .unwrap()
                .contains("Rate limit exceeded")
        );
        assert!(body["error"]["retryable"].is_null());
    }

    #[actix_web::test]
    async fn provider_rate_limit_without_metadata_does_not_fake_openai_headers() {
        let error = GatewayError::Provider(ProviderError::RateLimit {
            provider: "openai",
            message: "Rate limit exceeded".to_string(),
            retry_after: None,
            rpm_limit: None,
            tpm_limit: None,
            current_usage: None,
        });

        let response = gateway_error_response(&error);

        assert_eq!(response.status(), StatusCode::TOO_MANY_REQUESTS);
        assert!(response.headers().get(header::RETRY_AFTER).is_none());
        assert!(
            response
                .headers()
                .get("X-RateLimit-Limit-Requests")
                .is_none()
        );
        assert!(response.headers().get("X-RateLimit-Limit-Tokens").is_none());
    }

    #[actix_web::test]
    async fn provider_timeout_http_mapping_lives_at_openai_adapter_boundary() {
        let error = GatewayError::Provider(ProviderError::timeout("openai", "upstream timed out"));

        let response = gateway_error_response(&error);

        assert_eq!(response.status(), StatusCode::GATEWAY_TIMEOUT);
        let body = to_json(response).await;
        assert_eq!(body["error"]["type"], "server_error");
        assert_eq!(body["error"]["code"], "timeout");
    }

    #[actix_web::test]
    async fn openai_and_gateway_adapters_share_canonical_statuses() {
        let cases = vec![
            GatewayError::Validation("bad request".to_string()),
            GatewayError::Auth("bad token".to_string()),
            GatewayError::Forbidden("denied".to_string()),
            GatewayError::Provider(ProviderError::rate_limit("openai", Some(30))),
            GatewayError::Provider(ProviderError::timeout("openai", "upstream timed out")),
            GatewayError::Provider(ProviderError::provider_unavailable("openai", "down")),
            GatewayError::Provider(ProviderError::api_error("openai", 409, "conflict")),
            GatewayError::Provider(ProviderError::cancelled(
                "openai",
                "chat",
                Some("client disconnected".to_string()),
            )),
        ];

        for error in cases {
            let gateway_status = error.error_response().status();
            let openai_status = gateway_error_response(&error).status();

            assert_eq!(openai_status, gateway_status, "status drift for {error:?}");
        }
    }

    #[actix_web::test]
    async fn provider_cancelled_uses_single_canonical_status() {
        let error = GatewayError::Provider(ProviderError::cancelled(
            "openai",
            "chat",
            Some("client disconnected".to_string()),
        ));

        let response = gateway_error_response(&error);

        assert_eq!(response.status().as_u16(), 499);
        let body = to_json(response).await;
        assert_eq!(body["error"]["type"], "server_error");
        assert_eq!(body["error"]["code"], "cancelled");
    }

    #[actix_web::test]
    async fn model_not_priced_uses_specific_openai_error_code() {
        let error = GatewayError::Provider(super::super::spend::model_not_priced_error(
            "openai",
            "missing-model",
            "missing pricing",
        ));

        let response = gateway_error_response(&error);

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        let body = to_json(response).await;
        assert_eq!(body["error"]["type"], "invalid_request_error");
        assert_eq!(body["error"]["code"], "model_not_priced");
    }

    #[actix_web::test]
    async fn provider_api_error_preserves_upstream_openai_error_fields() {
        let upstream = serde_json::json!({
            "error": {
                "message": "context window exceeded",
                "type": "invalid_request_error",
                "param": "messages",
                "code": "context_length_exceeded"
            }
        });
        let error = GatewayError::Provider(ProviderError::api_error(
            "openai",
            400,
            upstream.to_string(),
        ));

        let response = gateway_error_response(&error);

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        let body = to_json(response).await;
        assert_eq!(body["error"]["message"], "context window exceeded");
        assert_eq!(body["error"]["type"], "invalid_request_error");
        assert_eq!(body["error"]["param"], "messages");
        assert_eq!(body["error"]["code"], "context_length_exceeded");
    }

    async fn to_json(response: HttpResponse) -> Value {
        let body = to_bytes(response.into_body()).await.unwrap();
        serde_json::from_slice(&body).unwrap()
    }
}
