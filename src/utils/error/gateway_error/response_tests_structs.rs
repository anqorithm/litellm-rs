use super::*;
use actix_web::http::StatusCode;

// ==================== ErrorDetail Tests ====================

#[test]
fn test_error_detail_creation() {
    let detail = GatewayErrorDetail {
        code: "AUTH_ERROR".to_string(),
        canonical_code: "AUTHENTICATION".to_string(),
        retryable: false,
        message: "Authentication failed".to_string(),
        timestamp: 1704067200,
        request_id: Some("req-12345".to_string()),
    };

    assert_eq!(detail.code, "AUTH_ERROR");
    assert_eq!(detail.canonical_code, "AUTHENTICATION");
    assert!(!detail.retryable);
    assert_eq!(detail.message, "Authentication failed");
    assert_eq!(detail.timestamp, 1704067200);
    assert_eq!(detail.request_id, Some("req-12345".to_string()));
}

#[test]
fn test_error_detail_without_request_id() {
    let detail = GatewayErrorDetail {
        code: "VALIDATION_ERROR".to_string(),
        canonical_code: "INVALID_REQUEST".to_string(),
        retryable: false,
        message: "Invalid input".to_string(),
        timestamp: chrono::Utc::now().timestamp(),
        request_id: None,
    };

    assert!(detail.request_id.is_none());
    assert!(detail.timestamp > 0);
}

#[test]
fn test_error_detail_serialization() {
    let detail = GatewayErrorDetail {
        code: "NOT_FOUND".to_string(),
        canonical_code: "NOT_FOUND".to_string(),
        retryable: false,
        message: "Resource not found".to_string(),
        timestamp: 1704067200,
        request_id: Some("req-abc".to_string()),
    };

    let json = serde_json::to_value(&detail).unwrap();
    assert_eq!(json["code"], "NOT_FOUND");
    assert_eq!(json["canonical_code"], "NOT_FOUND");
    assert_eq!(json["retryable"], false);
    assert_eq!(json["message"], "Resource not found");
    assert_eq!(json["timestamp"], 1704067200);
    assert_eq!(json["request_id"], "req-abc");
}

#[test]
fn test_error_detail_serialization_null_request_id() {
    let detail = GatewayErrorDetail {
        code: "ERROR".to_string(),
        canonical_code: "INTERNAL".to_string(),
        retryable: false,
        message: "Some error".to_string(),
        timestamp: 1704067200,
        request_id: None,
    };

    let json = serde_json::to_value(&detail).unwrap();
    assert!(json["request_id"].is_null());
}

// ==================== ErrorResponse Tests ====================

#[test]
fn test_error_response_creation() {
    let response = GatewayErrorResponse {
        error: GatewayErrorDetail {
            code: "INTERNAL_ERROR".to_string(),
            canonical_code: "INTERNAL".to_string(),
            retryable: false,
            message: "An internal error occurred".to_string(),
            timestamp: 1704067200,
            request_id: None,
        },
    };

    assert_eq!(response.error.code, "INTERNAL_ERROR");
}

#[test]
fn test_error_response_serialization() {
    let response = GatewayErrorResponse {
        error: GatewayErrorDetail {
            code: "BAD_REQUEST".to_string(),
            canonical_code: "INVALID_REQUEST".to_string(),
            retryable: false,
            message: "Invalid parameters".to_string(),
            timestamp: 1704067200,
            request_id: Some("req-xyz".to_string()),
        },
    };

    let json = serde_json::to_value(&response).unwrap();
    assert!(json["error"].is_object());
    assert_eq!(json["error"]["code"], "BAD_REQUEST");
    assert_eq!(json["error"]["canonical_code"], "INVALID_REQUEST");
    assert_eq!(json["error"]["retryable"], false);
    assert_eq!(json["error"]["message"], "Invalid parameters");
}

#[test]
fn test_error_response_json_string() {
    let response = GatewayErrorResponse {
        error: GatewayErrorDetail {
            code: "RATE_LIMIT".to_string(),
            canonical_code: "RATE_LIMITED".to_string(),
            retryable: true,
            message: "Too many requests".to_string(),
            timestamp: 1704067200,
            request_id: None,
        },
    };

    let json_str = serde_json::to_string(&response).unwrap();
    assert!(json_str.contains("RATE_LIMIT"));
    assert!(json_str.contains("Too many requests"));
}

// ==================== Integration Tests ====================

#[test]
fn test_error_response_json_structure() {
    let error = GatewayError::Auth("Invalid credentials".to_string());
    let response = error.error_response();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[test]
fn test_error_detail_timestamp_is_current() {
    let before = chrono::Utc::now().timestamp();
    let detail = GatewayErrorDetail {
        code: "TEST".to_string(),
        canonical_code: "INTERNAL".to_string(),
        retryable: false,
        message: "Test".to_string(),
        timestamp: chrono::Utc::now().timestamp(),
        request_id: None,
    };
    let after = chrono::Utc::now().timestamp();

    assert!(detail.timestamp >= before);
    assert!(detail.timestamp <= after);
}

#[test]
fn test_multiple_error_codes() {
    let error_codes = vec![
        ("CONFIG_ERROR", GatewayError::Config("test".to_string())),
        ("AUTH_ERROR", GatewayError::Auth("test".to_string())),
        (
            "VALIDATION_ERROR",
            GatewayError::Validation("test".to_string()),
        ),
        ("NOT_FOUND", GatewayError::NotFound("test".to_string())),
        ("RATE_LIMIT_EXCEEDED", GatewayError::rate_limit("test")),
    ];

    for (_expected_code, error) in error_codes {
        let response = error.error_response();
        assert!(response.status().is_client_error() || response.status().is_server_error());
    }
}
