use super::*;

#[test]
fn test_mcp_server_not_found_conversion() {
    let mcp_err = McpError::ServerNotFound {
        server_name: "github".to_string(),
    };
    let gateway_err: GatewayError = mcp_err.into();
    match gateway_err {
        GatewayError::NotFound(msg) => {
            assert!(msg.contains("MCP"));
            assert!(msg.contains("github"));
        }
        _ => panic!("Expected NotFound error"),
    }
}

#[test]
fn test_mcp_tool_not_found_conversion() {
    let mcp_err = McpError::ToolNotFound {
        server_name: "github".to_string(),
        tool_name: "get_repo".to_string(),
    };
    let gateway_err: GatewayError = mcp_err.into();
    match gateway_err {
        GatewayError::NotFound(msg) => {
            assert!(msg.contains("MCP"));
            assert!(msg.contains("get_repo"));
            assert!(msg.contains("github"));
        }
        _ => panic!("Expected NotFound error"),
    }
}

#[test]
fn test_mcp_connection_error_conversion() {
    let mcp_err = McpError::ConnectionError {
        server_name: "github".to_string(),
        message: "Connection refused".to_string(),
    };
    let gateway_err: GatewayError = mcp_err.into();
    match gateway_err {
        GatewayError::Network(msg) => {
            assert!(msg.contains("MCP"));
            assert!(msg.contains("github"));
        }
        _ => panic!("Expected Network error"),
    }
}

#[test]
fn test_mcp_transport_error_conversion() {
    let mcp_err = McpError::TransportError {
        transport: "http".to_string(),
        message: "Connection reset".to_string(),
    };
    let gateway_err: GatewayError = mcp_err.into();
    match gateway_err {
        GatewayError::Network(msg) => {
            assert!(msg.contains("MCP"));
            assert!(msg.contains("http"));
        }
        _ => panic!("Expected Network error"),
    }
}

#[test]
fn test_mcp_authentication_error_conversion() {
    let mcp_err = McpError::AuthenticationError {
        server_name: "github".to_string(),
        message: "Invalid token".to_string(),
    };
    let gateway_err: GatewayError = mcp_err.into();
    match gateway_err {
        GatewayError::Auth(msg) => {
            assert!(msg.contains("MCP"));
            assert!(msg.contains("github"));
        }
        _ => panic!("Expected Auth error"),
    }
}

#[test]
fn test_mcp_authorization_error_with_tool_conversion() {
    let mcp_err = McpError::AuthorizationError {
        server_name: "github".to_string(),
        tool_name: Some("delete_repo".to_string()),
        message: "Admin required".to_string(),
    };
    let gateway_err: GatewayError = mcp_err.into();
    match gateway_err {
        GatewayError::Forbidden(msg) => {
            assert!(msg.contains("MCP"));
            assert!(msg.contains("delete_repo"));
        }
        _ => panic!("Expected Forbidden error"),
    }
}

#[test]
fn test_mcp_authorization_error_without_tool_conversion() {
    let mcp_err = McpError::AuthorizationError {
        server_name: "github".to_string(),
        tool_name: None,
        message: "Access denied".to_string(),
    };
    let gateway_err: GatewayError = mcp_err.into();
    match gateway_err {
        GatewayError::Forbidden(msg) => {
            assert!(msg.contains("MCP"));
            assert!(msg.contains("github"));
        }
        _ => panic!("Expected Forbidden error"),
    }
}

#[test]
fn test_mcp_protocol_error_conversion() {
    let mcp_err = McpError::ProtocolError {
        message: "Invalid JSON-RPC".to_string(),
    };
    let gateway_err: GatewayError = mcp_err.into();
    match gateway_err {
        GatewayError::BadRequest(msg) => {
            assert!(msg.contains("MCP"));
            assert!(msg.contains("protocol error"));
        }
        _ => panic!("Expected BadRequest error"),
    }
}

#[test]
fn test_mcp_tool_execution_error_conversion() {
    let mcp_err = McpError::ToolExecutionError {
        server_name: "github".to_string(),
        tool_name: "create_issue".to_string(),
        code: -32000,
        message: "Repository not found".to_string(),
    };
    let gateway_err: GatewayError = mcp_err.into();
    match gateway_err {
        GatewayError::Internal(msg) => {
            assert!(msg.contains("MCP"));
            assert!(msg.contains("create_issue"));
            assert!(msg.contains("-32000"));
        }
        _ => panic!("Expected Internal error"),
    }
}

#[test]
fn test_mcp_timeout_conversion() {
    let mcp_err = McpError::Timeout {
        server_name: "slow-server".to_string(),
        timeout_ms: 30000,
    };
    let gateway_err: GatewayError = mcp_err.into();
    match gateway_err {
        GatewayError::Timeout(msg) => {
            assert!(msg.contains("MCP"));
            assert!(msg.contains("slow-server"));
            assert!(msg.contains("30000"));
        }
        _ => panic!("Expected Timeout error"),
    }
}

#[test]
fn test_mcp_configuration_error_conversion() {
    let mcp_err = McpError::ConfigurationError {
        message: "Missing URL".to_string(),
    };
    let gateway_err: GatewayError = mcp_err.into();
    assert!(matches!(gateway_err, GatewayError::Config(_)));
}

#[test]
fn test_mcp_serialization_error_conversion() {
    let mcp_err = McpError::SerializationError {
        message: "Invalid JSON".to_string(),
    };
    let gateway_err: GatewayError = mcp_err.into();
    match gateway_err {
        GatewayError::Validation(msg) => assert!(msg.contains("MCP")),
        _ => panic!("Expected Parsing error"),
    }
}

#[test]
fn test_mcp_server_already_exists_conversion() {
    let mcp_err = McpError::ServerAlreadyExists {
        server_name: "github".to_string(),
    };
    let gateway_err: GatewayError = mcp_err.into();
    match gateway_err {
        GatewayError::Conflict(msg) => {
            assert!(msg.contains("MCP"));
            assert!(msg.contains("github"));
        }
        _ => panic!("Expected Conflict error"),
    }
}

#[test]
fn test_mcp_invalid_url_conversion() {
    let mcp_err = McpError::InvalidUrl {
        url: "not-a-url".to_string(),
        message: "Invalid format".to_string(),
    };
    let gateway_err: GatewayError = mcp_err.into();
    match gateway_err {
        GatewayError::BadRequest(msg) => {
            assert!(msg.contains("MCP"));
            assert!(msg.contains("not-a-url"));
        }
        _ => panic!("Expected BadRequest error"),
    }
}

#[test]
fn test_mcp_rate_limit_with_retry_conversion() {
    let mcp_err = McpError::RateLimitExceeded {
        server_name: "github".to_string(),
        retry_after_ms: Some(5000),
    };
    let gateway_err: GatewayError = mcp_err.into();
    match gateway_err {
        GatewayError::RateLimit {
            message: msg,
            retry_after: None,
            rpm_limit: None,
            tpm_limit: None,
        } => {
            assert!(msg.contains("MCP"));
            assert!(msg.contains("5000ms"));
        }
        _ => panic!("Expected RateLimit error"),
    }
}

#[test]
fn test_mcp_conversion_keeps_legacy_message_shape() {
    let mcp_err = McpError::RateLimitExceeded {
        server_name: "github".to_string(),
        retry_after_ms: Some(800),
    };
    let gateway_err: GatewayError = mcp_err.into();
    match gateway_err {
        GatewayError::RateLimit {
            message: msg,
            retry_after: None,
            rpm_limit: None,
            tpm_limit: None,
        } => {
            assert!(msg.contains("MCP rate limit exceeded"));
            assert!(!msg.contains("protocol_code="));
            assert!(!msg.contains("canonical_code="));
            assert!(!msg.contains("retryable="));
        }
        _ => panic!("Expected RateLimit error"),
    }
}

#[test]
fn test_mcp_rate_limit_without_retry_conversion() {
    let mcp_err = McpError::RateLimitExceeded {
        server_name: "github".to_string(),
        retry_after_ms: None,
    };
    let gateway_err: GatewayError = mcp_err.into();
    match gateway_err {
        GatewayError::RateLimit {
            message: msg,
            retry_after: None,
            rpm_limit: None,
            tpm_limit: None,
        } => {
            assert!(msg.contains("MCP"));
            assert!(!msg.contains("retry after"));
        }
        _ => panic!("Expected RateLimit error"),
    }
}
