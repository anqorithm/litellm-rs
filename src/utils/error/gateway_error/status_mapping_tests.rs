//! Exhaustive variant->HTTP status mapping tests for GatewayError.
//!
//! These tests serve as a regression guard for the 29->15 variant consolidation
//! (commit aea82974). Any change to the HTTP status mapping of any variant will
//! cause a test here to fail explicitly.

use super::*;
use crate::core::providers::unified_provider::ProviderError;
use actix_web::http::StatusCode;

// ==================== Exhaustive Variant->Status Mapping Table ====================

/// Authoritative table: every GatewayError variant mapped to its expected HTTP status.
/// Covers all 18 enum arms (15 semantic categories + HttpClient/Serialization/Io).
#[test]
fn test_all_variants_exhaustive_status_mapping() {
    struct Case {
        label: &'static str,
        expected: u16,
        actual: u16,
    }

    let req_err = reqwest::Client::new()
        .get("not-a-valid-url")
        .build()
        .unwrap_err();

    let cases = [
        Case {
            label: "Config",
            expected: 500,
            actual: GatewayError::Config("e".into())
                .error_response()
                .status()
                .as_u16(),
        },
        Case {
            label: "Storage",
            expected: 503,
            actual: GatewayError::Storage("e".into())
                .error_response()
                .status()
                .as_u16(),
        },
        Case {
            label: "HttpClient",
            expected: 502,
            actual: GatewayError::HttpClient(req_err)
                .error_response()
                .status()
                .as_u16(),
        },
        Case {
            label: "Serialization",
            expected: 400,
            actual: GatewayError::Serialization("e".into())
                .error_response()
                .status()
                .as_u16(),
        },
        Case {
            label: "Io",
            expected: 500,
            actual: GatewayError::Io(std::io::Error::new(
                std::io::ErrorKind::PermissionDenied,
                "e",
            ))
            .error_response()
            .status()
            .as_u16(),
        },
        Case {
            label: "Auth",
            expected: 401,
            actual: GatewayError::Auth("e".into())
                .error_response()
                .status()
                .as_u16(),
        },
        Case {
            label: "Provider(Other)",
            expected: 502,
            actual: GatewayError::Provider(ProviderError::Other {
                provider: "x",
                message: "e".into(),
            })
            .error_response()
            .status()
            .as_u16(),
        },
        Case {
            label: "RateLimit",
            expected: 429,
            actual: GatewayError::RateLimit {
                message: "e".into(),
                retry_after: None,
                rpm_limit: None,
                tpm_limit: None,
            }
            .error_response()
            .status()
            .as_u16(),
        },
        Case {
            label: "Validation",
            expected: 400,
            actual: GatewayError::Validation("e".into())
                .error_response()
                .status()
                .as_u16(),
        },
        Case {
            label: "Timeout",
            expected: 408,
            actual: GatewayError::Timeout("e".into())
                .error_response()
                .status()
                .as_u16(),
        },
        Case {
            label: "NotFound",
            expected: 404,
            actual: GatewayError::NotFound("e".into())
                .error_response()
                .status()
                .as_u16(),
        },
        Case {
            label: "Conflict",
            expected: 409,
            actual: GatewayError::Conflict("e".into())
                .error_response()
                .status()
                .as_u16(),
        },
        Case {
            label: "BadRequest",
            expected: 400,
            actual: GatewayError::BadRequest("e".into())
                .error_response()
                .status()
                .as_u16(),
        },
        Case {
            label: "Internal",
            expected: 500,
            actual: GatewayError::Internal("e".into())
                .error_response()
                .status()
                .as_u16(),
        },
        Case {
            label: "Unavailable",
            expected: 503,
            actual: GatewayError::Unavailable("e".into())
                .error_response()
                .status()
                .as_u16(),
        },
        Case {
            label: "Network",
            expected: 502,
            actual: GatewayError::Network("e".into())
                .error_response()
                .status()
                .as_u16(),
        },
        Case {
            label: "Forbidden",
            expected: 403,
            actual: GatewayError::Forbidden("e".into())
                .error_response()
                .status()
                .as_u16(),
        },
        Case {
            label: "NotImplemented",
            expected: 501,
            actual: GatewayError::NotImplemented("e".into())
                .error_response()
                .status()
                .as_u16(),
        },
    ];

    for c in &cases {
        assert_eq!(
            c.actual, c.expected,
            "GatewayError::{} should map to HTTP {}",
            c.label, c.expected
        );
    }
}

// ==================== 29->15 Consolidation Regression ====================
//
// Old variant name  ->  New GatewayError variant  ->  HTTP status
// ------------------------------------------------------------------
// Auth / Unauthorized / Jwt          -> Auth          -> 401
// Database / Cache / Redis / VectorDb -> Storage      -> 503
// Parsing / InvalidRequest           -> Validation    -> 400
// Serialization / Json / Yaml        -> Serialization -> 400
// ProviderUnavailable / CircuitBreakerOpen / NoHealthyProviders -> Unavailable -> 503
// ExternalService / Network / WebSocket -> Network    -> 502
// HttpClient                         -> HttpClient    -> 502
// Io / FileStorage                   -> Io            -> 500
// Crypto / Encryption                -> Auth          -> 401

#[test]
fn test_consolidation_old_auth_variant_maps_to_auth_401() {
    let error = GatewayError::Auth("Invalid credentials".into());
    assert_eq!(error.error_response().status(), StatusCode::UNAUTHORIZED);
}

#[test]
fn test_consolidation_old_jwt_variant_maps_to_auth_401() {
    use jsonwebtoken::{errors::Error as JwtError, errors::ErrorKind};
    let jwt_err = JwtError::from(ErrorKind::ExpiredSignature);
    let error: GatewayError = jwt_err.into();
    assert!(matches!(error, GatewayError::Auth(_)));
    assert_eq!(error.error_response().status(), StatusCode::UNAUTHORIZED);
}

#[test]
fn test_consolidation_old_database_variant_maps_to_storage_503() {
    let error = GatewayError::Storage("Database connection failed".into());
    assert_eq!(
        error.error_response().status(),
        StatusCode::SERVICE_UNAVAILABLE
    );
}

#[test]
fn test_consolidation_old_redis_variant_maps_to_storage_503() {
    let error = GatewayError::Storage("Redis error: connection refused".into());
    assert_eq!(
        error.error_response().status(),
        StatusCode::SERVICE_UNAVAILABLE
    );
}

#[test]
fn test_consolidation_old_parse_variant_maps_to_validation_400() {
    let error = GatewayError::Validation("Parse error at line 5".into());
    assert_eq!(error.error_response().status(), StatusCode::BAD_REQUEST);
}

#[test]
fn test_consolidation_old_serialization_variant_maps_to_serialization_400() {
    let error = GatewayError::Serialization("JSON parse failed".into());
    assert_eq!(error.error_response().status(), StatusCode::BAD_REQUEST);
}

#[test]
fn test_consolidation_old_provider_unavailable_maps_to_unavailable_503() {
    let error = GatewayError::Unavailable("Circuit breaker open".into());
    assert_eq!(
        error.error_response().status(),
        StatusCode::SERVICE_UNAVAILABLE
    );
}

#[test]
fn test_consolidation_old_external_service_maps_to_network_502() {
    let error = GatewayError::Network("External service unreachable".into());
    assert_eq!(error.error_response().status(), StatusCode::BAD_GATEWAY);
}

#[test]
fn test_consolidation_old_http_client_maps_to_http_client_502() {
    let req_err = reqwest::Client::new()
        .get("not-a-valid-url")
        .build()
        .unwrap_err();
    let error = GatewayError::HttpClient(req_err);
    assert_eq!(error.error_response().status(), StatusCode::BAD_GATEWAY);
}

#[test]
fn test_consolidation_old_io_variant_maps_to_io_500() {
    let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file missing");
    let error = GatewayError::Io(io_err);
    assert_eq!(
        error.error_response().status(),
        StatusCode::INTERNAL_SERVER_ERROR
    );
}

#[test]
fn test_consolidation_old_crypto_variant_maps_to_auth_401() {
    let error = GatewayError::Auth("Encryption failed: invalid key".into());
    assert_eq!(error.error_response().status(), StatusCode::UNAUTHORIZED);
}

#[test]
fn test_consolidation_rate_limit_preserves_header_metadata() {
    let error = GatewayError::RateLimit {
        message: "100 RPM exceeded".into(),
        retry_after: Some(30),
        rpm_limit: Some(100),
        tpm_limit: Some(40000),
    };
    let response = error.error_response();
    assert_eq!(response.status(), StatusCode::TOO_MANY_REQUESTS);
    assert_eq!(
        response
            .headers()
            .get("Retry-After")
            .and_then(|v| v.to_str().ok()),
        Some("30"),
        "Retry-After header"
    );
    assert_eq!(
        response
            .headers()
            .get("X-RateLimit-Limit-Requests")
            .and_then(|v| v.to_str().ok()),
        Some("100"),
        "X-RateLimit-Limit-Requests header"
    );
    assert_eq!(
        response
            .headers()
            .get("X-RateLimit-Limit-Tokens")
            .and_then(|v| v.to_str().ok()),
        Some("40000"),
        "X-RateLimit-Limit-Tokens header"
    );
}

// ==================== User-Friendly Error Message Tests ====================

#[test]
fn test_auth_error_message_is_user_friendly() {
    let error = GatewayError::Auth("Invalid API key".into());
    let msg = error.to_string();
    assert!(msg.starts_with("Authentication error:"), "Got: {}", msg);
}

#[test]
fn test_not_found_error_message_includes_resource() {
    let error = GatewayError::NotFound("Model 'gpt-5' not found".into());
    let msg = error.to_string();
    assert!(msg.starts_with("Not found:"), "Got: {}", msg);
    assert!(msg.contains("gpt-5"));
}

#[test]
fn test_rate_limit_error_message_includes_context() {
    let error = GatewayError::RateLimit {
        message: "100 requests/minute exceeded".into(),
        retry_after: Some(60),
        rpm_limit: Some(100),
        tpm_limit: None,
    };
    let msg = error.to_string();
    assert!(msg.contains("Rate limit exceeded"), "Got: {}", msg);
    assert!(msg.contains("100 requests/minute exceeded"));
}

#[test]
fn test_validation_error_message_is_descriptive() {
    let error = GatewayError::Validation("'model' field is required".into());
    let msg = error.to_string();
    assert!(msg.starts_with("Validation error:"), "Got: {}", msg);
    assert!(msg.contains("model"));
}

#[test]
fn test_unavailable_error_message_is_user_friendly() {
    let error = GatewayError::Unavailable("OpenAI is temporarily unavailable".into());
    let msg = error.to_string();
    assert!(msg.starts_with("Service unavailable:"), "Got: {}", msg);
    assert!(msg.contains("OpenAI"));
}
