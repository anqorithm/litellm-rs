//! Error response types
//!
//! Re-exports the canonical error types from `core::types::responses::error`.

pub use crate::core::types::responses::error::{ApiError, ErrorResponse};

/// Type alias for backward compatibility.
/// `ErrorDetail` is the same as [`ApiError`].
pub type ErrorDetail = ApiError;

#[cfg(test)]
mod tests {
    use super::*;

    // ==================== ErrorDetail Tests ====================

    #[test]
    fn test_error_detail_basic() {
        let detail = ErrorDetail {
            message: "Invalid API key".to_string(),
            error_type: "invalid_request_error".to_string(),
            code: None,
            param: None,
        };

        assert_eq!(detail.message, "Invalid API key");
        assert_eq!(detail.error_type, "invalid_request_error");
        assert!(detail.code.is_none());
        assert!(detail.param.is_none());
    }

    #[test]
    fn test_error_detail_with_code() {
        let detail = ErrorDetail {
            message: "Rate limit exceeded".to_string(),
            error_type: "rate_limit_error".to_string(),
            code: Some("rate_limit_exceeded".to_string()),
            param: None,
        };

        assert_eq!(detail.code, Some("rate_limit_exceeded".to_string()));
    }

    #[test]
    fn test_error_detail_with_param() {
        let detail = ErrorDetail {
            message: "Invalid value for 'model'".to_string(),
            error_type: "invalid_request_error".to_string(),
            code: Some("invalid_value".to_string()),
            param: Some("model".to_string()),
        };

        assert_eq!(detail.param, Some("model".to_string()));
    }

    #[test]
    fn test_error_detail_full() {
        let detail = ErrorDetail {
            message: "The model 'gpt-5' does not exist".to_string(),
            error_type: "invalid_request_error".to_string(),
            code: Some("model_not_found".to_string()),
            param: Some("model".to_string()),
        };

        assert_eq!(detail.message, "The model 'gpt-5' does not exist");
        assert_eq!(detail.error_type, "invalid_request_error");
        assert_eq!(detail.code, Some("model_not_found".to_string()));
        assert_eq!(detail.param, Some("model".to_string()));
    }

    #[test]
    fn test_error_detail_clone() {
        let detail = ErrorDetail {
            message: "Test error".to_string(),
            error_type: "test_error".to_string(),
            code: Some("test_code".to_string()),
            param: Some("test_param".to_string()),
        };

        let cloned = detail.clone();
        assert_eq!(detail.message, cloned.message);
        assert_eq!(detail.error_type, cloned.error_type);
        assert_eq!(detail.code, cloned.code);
        assert_eq!(detail.param, cloned.param);
    }

    #[test]
    fn test_error_detail_debug() {
        let detail = ErrorDetail {
            message: "Debug test".to_string(),
            error_type: "debug_error".to_string(),
            code: None,
            param: None,
        };

        let debug_str = format!("{:?}", detail);
        // ErrorDetail is a type alias for ApiError, so Debug prints "ApiError"
        assert!(debug_str.contains("ApiError"));
        assert!(debug_str.contains("Debug test"));
    }

    #[test]
    fn test_error_detail_serialization() {
        let detail = ErrorDetail {
            message: "Serialization test".to_string(),
            error_type: "test_error".to_string(),
            code: Some("test_code".to_string()),
            param: None,
        };

        let json = serde_json::to_value(&detail).unwrap();
        assert_eq!(json["message"], "Serialization test");
        assert_eq!(json["type"], "test_error"); // Note: renamed from error_type
        assert_eq!(json["code"], "test_code");
        // With skip_serializing_if = "Option::is_none", None fields are omitted entirely
        assert!(json.get("param").is_none());
    }

    #[test]
    fn test_error_detail_deserialization() {
        let json = r#"{
            "message": "Deserialization test",
            "type": "test_error",
            "code": "test_code",
            "param": "test_param"
        }"#;

        let detail: ErrorDetail = serde_json::from_str(json).unwrap();
        assert_eq!(detail.message, "Deserialization test");
        assert_eq!(detail.error_type, "test_error");
        assert_eq!(detail.code, Some("test_code".to_string()));
        assert_eq!(detail.param, Some("test_param".to_string()));
    }

    #[test]
    fn test_error_detail_deserialization_minimal() {
        let json = r#"{
            "message": "Minimal error",
            "type": "minimal_error"
        }"#;

        let detail: ErrorDetail = serde_json::from_str(json).unwrap();
        assert_eq!(detail.message, "Minimal error");
        assert_eq!(detail.error_type, "minimal_error");
        assert!(detail.code.is_none());
        assert!(detail.param.is_none());
    }

    // ==================== ErrorResponse Tests ====================

    #[test]
    fn test_error_response_basic() {
        let response = ErrorResponse {
            error: ErrorDetail {
                message: "Test error".to_string(),
                error_type: "test_error".to_string(),
                code: None,
                param: None,
            },
        };

        assert_eq!(response.error.message, "Test error");
        assert_eq!(response.error.error_type, "test_error");
    }

    #[test]
    fn test_error_response_clone() {
        let response = ErrorResponse {
            error: ErrorDetail {
                message: "Clone test".to_string(),
                error_type: "clone_error".to_string(),
                code: Some("clone_code".to_string()),
                param: None,
            },
        };

        let cloned = response.clone();
        assert_eq!(response.error.message, cloned.error.message);
        assert_eq!(response.error.code, cloned.error.code);
    }

    #[test]
    fn test_error_response_debug() {
        let response = ErrorResponse {
            error: ErrorDetail {
                message: "Debug test".to_string(),
                error_type: "debug_error".to_string(),
                code: None,
                param: None,
            },
        };

        let debug_str = format!("{:?}", response);
        assert!(debug_str.contains("ErrorResponse"));
        // ErrorDetail is a type alias for ApiError, so Debug prints "ApiError"
        assert!(debug_str.contains("ApiError"));
    }

    #[test]
    fn test_error_response_serialization() {
        let response = ErrorResponse {
            error: ErrorDetail {
                message: "API error".to_string(),
                error_type: "api_error".to_string(),
                code: Some("500".to_string()),
                param: None,
            },
        };

        let json = serde_json::to_value(&response).unwrap();
        assert!(json["error"].is_object());
        assert_eq!(json["error"]["message"], "API error");
        assert_eq!(json["error"]["type"], "api_error");
        assert_eq!(json["error"]["code"], "500");
    }

    #[test]
    fn test_error_response_deserialization() {
        let json = r#"{
            "error": {
                "message": "Deserialization test",
                "type": "test_error",
                "code": "test_code",
                "param": null
            }
        }"#;

        let response: ErrorResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.error.message, "Deserialization test");
        assert_eq!(response.error.error_type, "test_error");
        assert_eq!(response.error.code, Some("test_code".to_string()));
        assert!(response.error.param.is_none());
    }

    #[test]
    fn test_error_response_openai_format() {
        // Test compatibility with OpenAI API error format
        let json = r#"{
            "error": {
                "message": "You exceeded your current quota, please check your plan and billing details.",
                "type": "insufficient_quota",
                "param": null,
                "code": "insufficient_quota"
            }
        }"#;

        let response: ErrorResponse = serde_json::from_str(json).unwrap();
        assert!(response.error.message.contains("quota"));
        assert_eq!(response.error.error_type, "insufficient_quota");
        assert_eq!(response.error.code, Some("insufficient_quota".to_string()));
    }

    #[test]
    fn test_error_response_invalid_request() {
        let response = ErrorResponse {
            error: ErrorDetail {
                message: "Invalid value for 'temperature': expected a number between 0 and 2"
                    .to_string(),
                error_type: "invalid_request_error".to_string(),
                code: Some("invalid_value".to_string()),
                param: Some("temperature".to_string()),
            },
        };

        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("temperature"));
        assert!(json.contains("invalid_value"));
    }

    #[test]
    fn test_error_response_authentication_error() {
        let response = ErrorResponse {
            error: ErrorDetail {
                message: "Incorrect API key provided".to_string(),
                error_type: "invalid_api_key".to_string(),
                code: None,
                param: None,
            },
        };

        assert_eq!(response.error.error_type, "invalid_api_key");
    }

    #[test]
    fn test_error_response_rate_limit() {
        let response = ErrorResponse {
            error: ErrorDetail {
                message: "Rate limit reached for requests".to_string(),
                error_type: "rate_limit_exceeded".to_string(),
                code: Some("rate_limit_exceeded".to_string()),
                param: None,
            },
        };

        assert_eq!(response.error.error_type, "rate_limit_exceeded");
        assert_eq!(response.error.code, Some("rate_limit_exceeded".to_string()));
    }

    // ==================== Edge Cases ====================

    #[test]
    fn test_error_detail_empty_strings() {
        let detail = ErrorDetail {
            message: "".to_string(),
            error_type: "".to_string(),
            code: Some("".to_string()),
            param: Some("".to_string()),
        };

        assert!(detail.message.is_empty());
        assert!(detail.error_type.is_empty());
    }

    #[test]
    fn test_error_detail_special_characters() {
        let detail = ErrorDetail {
            message: "Error with special chars: <>&\"'".to_string(),
            error_type: "special_error".to_string(),
            code: None,
            param: None,
        };

        let json = serde_json::to_string(&detail).unwrap();
        let deserialized: ErrorDetail = serde_json::from_str(&json).unwrap();
        assert_eq!(detail.message, deserialized.message);
    }

    #[test]
    fn test_error_detail_unicode() {
        let detail = ErrorDetail {
            message: "错误信息 🚨 エラー".to_string(),
            error_type: "unicode_error".to_string(),
            code: None,
            param: None,
        };

        let json = serde_json::to_string(&detail).unwrap();
        let deserialized: ErrorDetail = serde_json::from_str(&json).unwrap();
        assert_eq!(detail.message, deserialized.message);
    }

    #[test]
    fn test_error_response_roundtrip() {
        let response = ErrorResponse {
            error: ErrorDetail {
                message: "Roundtrip test".to_string(),
                error_type: "roundtrip_error".to_string(),
                code: Some("roundtrip_code".to_string()),
                param: Some("roundtrip_param".to_string()),
            },
        };

        let json = serde_json::to_string(&response).unwrap();
        let deserialized: ErrorResponse = serde_json::from_str(&json).unwrap();

        assert_eq!(response.error.message, deserialized.error.message);
        assert_eq!(response.error.error_type, deserialized.error.error_type);
        assert_eq!(response.error.code, deserialized.error.code);
        assert_eq!(response.error.param, deserialized.error.param);
    }
}
