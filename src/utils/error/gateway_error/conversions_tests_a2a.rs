use super::*;

#[test]
fn test_a2a_agent_not_found_conversion() {
    let a2a_err = A2AError::AgentNotFound {
        agent_name: "my-agent".to_string(),
    };
    let gateway_err: GatewayError = a2a_err.into();
    match gateway_err {
        GatewayError::NotFound(msg) => {
            assert!(msg.contains("A2A"));
            assert!(msg.contains("my-agent"));
        }
        _ => panic!("Expected NotFound error"),
    }
}

#[test]
fn test_a2a_agent_already_exists_conversion() {
    let a2a_err = A2AError::AgentAlreadyExists {
        agent_name: "existing-agent".to_string(),
    };
    let gateway_err: GatewayError = a2a_err.into();
    match gateway_err {
        GatewayError::Conflict(msg) => {
            assert!(msg.contains("A2A"));
            assert!(msg.contains("existing-agent"));
        }
        _ => panic!("Expected Conflict error"),
    }
}

#[test]
fn test_a2a_connection_error_conversion() {
    let a2a_err = A2AError::ConnectionError {
        agent_name: "remote-agent".to_string(),
        message: "Connection refused".to_string(),
    };
    let gateway_err: GatewayError = a2a_err.into();
    match gateway_err {
        GatewayError::Network(msg) => {
            assert!(msg.contains("A2A"));
            assert!(msg.contains("remote-agent"));
            assert!(msg.contains("Connection refused"));
        }
        _ => panic!("Expected Network error"),
    }
}

#[test]
fn test_a2a_authentication_error_conversion() {
    let a2a_err = A2AError::AuthenticationError {
        agent_name: "secure-agent".to_string(),
        message: "Invalid token".to_string(),
    };
    let gateway_err: GatewayError = a2a_err.into();
    match gateway_err {
        GatewayError::Auth(msg) => {
            assert!(msg.contains("A2A"));
            assert!(msg.contains("secure-agent"));
        }
        _ => panic!("Expected Auth error"),
    }
}

#[test]
fn test_a2a_task_not_found_conversion() {
    let a2a_err = A2AError::TaskNotFound {
        agent_name: "agent".to_string(),
        task_id: "task-456".to_string(),
    };
    let gateway_err: GatewayError = a2a_err.into();
    match gateway_err {
        GatewayError::NotFound(msg) => {
            assert!(msg.contains("A2A"));
            assert!(msg.contains("task-456"));
        }
        _ => panic!("Expected NotFound error"),
    }
}

#[test]
fn test_a2a_task_failed_conversion() {
    let a2a_err = A2AError::TaskFailed {
        agent_name: "agent".to_string(),
        task_id: "task-123".to_string(),
        message: "Something went wrong".to_string(),
    };
    let gateway_err: GatewayError = a2a_err.into();
    match gateway_err {
        GatewayError::Internal(msg) => {
            assert!(msg.contains("A2A"));
            assert!(msg.contains("task-123"));
            assert!(msg.contains("Something went wrong"));
        }
        _ => panic!("Expected Internal error"),
    }
}

#[test]
fn test_a2a_protocol_error_conversion() {
    let a2a_err = A2AError::ProtocolError {
        message: "Invalid JSON-RPC".to_string(),
    };
    let gateway_err: GatewayError = a2a_err.into();
    match gateway_err {
        GatewayError::BadRequest(msg) => {
            assert!(msg.contains("A2A"));
            assert!(msg.contains("protocol error"));
        }
        _ => panic!("Expected BadRequest error"),
    }
}

#[test]
fn test_a2a_timeout_conversion() {
    let a2a_err = A2AError::Timeout {
        agent_name: "slow-agent".to_string(),
        timeout_ms: 30000,
    };
    let gateway_err: GatewayError = a2a_err.into();
    match gateway_err {
        GatewayError::Timeout(msg) => {
            assert!(msg.contains("A2A"));
            assert!(msg.contains("slow-agent"));
            assert!(msg.contains("30000"));
        }
        _ => panic!("Expected Timeout error"),
    }
}

#[test]
fn test_a2a_configuration_error_conversion() {
    let a2a_err = A2AError::ConfigurationError {
        message: "Missing endpoint".to_string(),
    };
    let gateway_err: GatewayError = a2a_err.into();
    assert!(matches!(gateway_err, GatewayError::Config(_)));
}

#[test]
fn test_a2a_serialization_error_conversion() {
    let a2a_err = A2AError::SerializationError {
        message: "Invalid UTF-8".to_string(),
    };
    let gateway_err: GatewayError = a2a_err.into();
    match gateway_err {
        GatewayError::Validation(msg) => assert!(msg.contains("A2A")),
        _ => panic!("Expected Parsing error"),
    }
}

#[test]
fn test_a2a_unsupported_provider_conversion() {
    let a2a_err = A2AError::UnsupportedProvider {
        provider: "unknown-provider".to_string(),
    };
    let gateway_err: GatewayError = a2a_err.into();
    match gateway_err {
        GatewayError::NotImplemented(msg) => {
            assert!(msg.contains("A2A"));
            assert!(msg.contains("unknown-provider"));
        }
        _ => panic!("Expected NotImplemented error"),
    }
}

#[test]
fn test_a2a_rate_limit_with_retry_conversion() {
    let a2a_err = A2AError::RateLimitExceeded {
        agent_name: "agent".to_string(),
        retry_after_ms: Some(5000),
    };
    let gateway_err: GatewayError = a2a_err.into();
    match gateway_err {
        GatewayError::RateLimit {
            message: msg,
            retry_after: None,
            rpm_limit: None,
            tpm_limit: None,
        } => {
            assert!(msg.contains("A2A"));
            assert!(msg.contains("5000ms"));
        }
        _ => panic!("Expected RateLimit error"),
    }
}

#[test]
fn test_a2a_conversion_keeps_legacy_message_shape() {
    let a2a_err = A2AError::RateLimitExceeded {
        agent_name: "agent".to_string(),
        retry_after_ms: Some(1200),
    };
    let gateway_err: GatewayError = a2a_err.into();
    match gateway_err {
        GatewayError::RateLimit {
            message: msg,
            retry_after: None,
            rpm_limit: None,
            tpm_limit: None,
        } => {
            assert!(msg.contains("A2A rate limit exceeded"));
            assert!(!msg.contains("protocol_code="));
            assert!(!msg.contains("canonical_code="));
            assert!(!msg.contains("retryable="));
        }
        _ => panic!("Expected RateLimit error"),
    }
}

#[test]
fn test_a2a_rate_limit_without_retry_conversion() {
    let a2a_err = A2AError::RateLimitExceeded {
        agent_name: "agent".to_string(),
        retry_after_ms: None,
    };
    let gateway_err: GatewayError = a2a_err.into();
    match gateway_err {
        GatewayError::RateLimit {
            message: msg,
            retry_after: None,
            rpm_limit: None,
            tpm_limit: None,
        } => {
            assert!(msg.contains("A2A"));
            assert!(!msg.contains("retry after"));
        }
        _ => panic!("Expected RateLimit error"),
    }
}

#[test]
fn test_a2a_agent_busy_conversion() {
    let a2a_err = A2AError::AgentBusy {
        agent_name: "busy-agent".to_string(),
        message: "Processing another request".to_string(),
    };
    let gateway_err: GatewayError = a2a_err.into();
    match gateway_err {
        GatewayError::Unavailable(msg) => {
            assert!(msg.contains("A2A"));
            assert!(msg.contains("busy-agent"));
        }
        _ => panic!("Expected ProviderUnavailable error"),
    }
}

#[test]
fn test_a2a_content_blocked_conversion() {
    let a2a_err = A2AError::ContentBlocked {
        agent_name: "safe-agent".to_string(),
        reason: "Harmful content".to_string(),
    };
    let gateway_err: GatewayError = a2a_err.into();
    match gateway_err {
        GatewayError::BadRequest(msg) => {
            assert!(msg.contains("A2A"));
            assert!(msg.contains("blocked"));
        }
        _ => panic!("Expected BadRequest error"),
    }
}
