//! Type conversions for GatewayError

use super::types::GatewayError;
use crate::core::a2a::error::A2AError;
use crate::core::a2a::message::A2AResponseError;
use crate::core::mcp::error::McpError;
use crate::core::mcp::protocol::JsonRpcError;
use crate::core::providers::unified_provider::ProviderError;

// Conversion from unified ProviderError to GatewayError.
// The ResponseError impl in response.rs handles all ProviderError variants
// via GatewayError::Provider, so no destructuring is needed here.
impl From<ProviderError> for GatewayError {
    fn from(err: ProviderError) -> Self {
        GatewayError::Provider(err)
    }
}

// Conversion from A2AError to GatewayError
impl From<A2AError> for GatewayError {
    fn from(err: A2AError) -> Self {
        // Keep protocol mapping in the runtime path so canonical A2A mapping is exercised.
        // GatewayError message text remains unchanged for backward compatibility.
        let _protocol_error = A2AResponseError::from_a2a_error(&err);

        match err {
            A2AError::AgentNotFound { agent_name } => {
                GatewayError::NotFound(format!("A2A agent not found: {}", agent_name))
            }
            A2AError::AgentAlreadyExists { agent_name } => {
                GatewayError::Conflict(format!("A2A agent already exists: {}", agent_name))
            }
            A2AError::ConnectionError {
                agent_name,
                message,
            } => GatewayError::Network(format!(
                "A2A connection error to agent '{}': {}",
                agent_name, message
            )),
            A2AError::AuthenticationError {
                agent_name,
                message,
            } => GatewayError::Auth(format!(
                "A2A authentication failed for agent '{}': {}",
                agent_name, message
            )),
            A2AError::TaskNotFound {
                agent_name,
                task_id,
            } => GatewayError::NotFound(format!(
                "A2A task '{}' not found on agent '{}'",
                task_id, agent_name
            )),
            A2AError::TaskFailed {
                agent_name,
                task_id,
                message,
            } => GatewayError::Internal(format!(
                "A2A task '{}' failed on agent '{}': {}",
                task_id, agent_name, message
            )),
            A2AError::ProtocolError { message } => {
                GatewayError::BadRequest(format!("A2A protocol error: {}", message))
            }
            A2AError::InvalidRequest { message } => {
                GatewayError::BadRequest(format!("Invalid A2A request: {}", message))
            }
            A2AError::Timeout {
                agent_name,
                timeout_ms,
            } => GatewayError::Timeout(format!(
                "A2A timeout waiting for agent '{}' ({}ms)",
                agent_name, timeout_ms
            )),
            A2AError::ConfigurationError { message } => {
                GatewayError::Config(format!("A2A configuration error: {}", message))
            }
            A2AError::SerializationError { message } => {
                GatewayError::Validation(format!("A2A serialization error: {}", message))
            }
            A2AError::UnsupportedProvider { provider } => {
                GatewayError::NotImplemented(format!("A2A provider not supported: {}", provider))
            }
            A2AError::RateLimitExceeded {
                agent_name,
                retry_after_ms,
            } => {
                let msg = if let Some(ms) = retry_after_ms {
                    format!(
                        "A2A rate limit exceeded for agent '{}', retry after {}ms",
                        agent_name, ms
                    )
                } else {
                    format!("A2A rate limit exceeded for agent '{}'", agent_name)
                };
                GatewayError::RateLimit {
                    message: msg,
                    retry_after: None,
                    rpm_limit: None,
                    tpm_limit: None,
                }
            }
            A2AError::AgentBusy {
                agent_name,
                message,
            } => GatewayError::Unavailable(format!(
                "A2A agent '{}' is busy: {}",
                agent_name, message
            )),
            A2AError::ContentBlocked { agent_name, reason } => GatewayError::BadRequest(format!(
                "A2A content blocked by agent '{}': {}",
                agent_name, reason
            )),
        }
    }
}

// Conversion from McpError to GatewayError
impl From<McpError> for GatewayError {
    fn from(err: McpError) -> Self {
        // Keep protocol mapping in the runtime path so canonical MCP mapping is exercised.
        // GatewayError message text remains unchanged for backward compatibility.
        let _protocol_error = JsonRpcError::from_mcp_error(&err);

        match err {
            McpError::ServerNotFound { server_name } => {
                GatewayError::NotFound(format!("MCP server not found: {}", server_name))
            }
            McpError::ToolNotFound {
                server_name,
                tool_name,
            } => GatewayError::NotFound(format!(
                "MCP tool '{}' not found on server '{}'",
                tool_name, server_name
            )),
            McpError::ConnectionError {
                server_name,
                message,
            } => GatewayError::Network(format!(
                "MCP connection error to server '{}': {}",
                server_name, message
            )),
            McpError::TransportError { transport, message } => {
                GatewayError::Network(format!("MCP transport error ({}): {}", transport, message))
            }
            McpError::AuthenticationError {
                server_name,
                message,
            } => GatewayError::Auth(format!(
                "MCP authentication failed for server '{}': {}",
                server_name, message
            )),
            McpError::AuthorizationError {
                server_name,
                tool_name,
                message,
            } => {
                let msg = if let Some(tool) = tool_name {
                    format!(
                        "MCP permission denied for tool '{}' on server '{}': {}",
                        tool, server_name, message
                    )
                } else {
                    format!(
                        "MCP permission denied for server '{}': {}",
                        server_name, message
                    )
                };
                GatewayError::Forbidden(msg)
            }
            McpError::ProtocolError { message } => {
                GatewayError::BadRequest(format!("MCP protocol error: {}", message))
            }
            McpError::ToolExecutionError {
                server_name,
                tool_name,
                code,
                message,
            } => GatewayError::Internal(format!(
                "MCP tool execution failed: server='{}', tool='{}', code={}, message='{}'",
                server_name, tool_name, code, message
            )),
            McpError::Timeout {
                server_name,
                timeout_ms,
            } => GatewayError::Timeout(format!(
                "MCP timeout waiting for server '{}' ({}ms)",
                server_name, timeout_ms
            )),
            McpError::ConfigurationError { message } => {
                GatewayError::Config(format!("MCP configuration error: {}", message))
            }
            McpError::SerializationError { message } => {
                GatewayError::Validation(format!("MCP serialization error: {}", message))
            }
            McpError::ServerAlreadyExists { server_name } => {
                GatewayError::Conflict(format!("MCP server already registered: {}", server_name))
            }
            McpError::InvalidUrl { url, message } => {
                GatewayError::BadRequest(format!("Invalid MCP server URL '{}': {}", url, message))
            }
            McpError::RateLimitExceeded {
                server_name,
                retry_after_ms,
            } => {
                let msg = if let Some(ms) = retry_after_ms {
                    format!(
                        "MCP rate limit exceeded for server '{}', retry after {}ms",
                        server_name, ms
                    )
                } else {
                    format!("MCP rate limit exceeded for server '{}'", server_name)
                };
                GatewayError::RateLimit {
                    message: msg,
                    retry_after: None,
                    rpm_limit: None,
                    tpm_limit: None,
                }
            }
            McpError::ValidationError {
                server_name,
                tool_name,
                errors,
            } => GatewayError::Validation(format!(
                "Validation failed for tool '{}' on server '{}': {}",
                tool_name,
                server_name,
                errors.join("; ")
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // All ProviderError variants now wrap into GatewayError::Provider.
    // The ResponseError impl (response.rs) handles HTTP status mapping.

    #[test]
    fn test_provider_error_wraps_in_provider_variant() {
        let provider_err = ProviderError::Authentication {
            provider: "openai",
            message: "Invalid API key".to_string(),
        };
        let gateway_err: GatewayError = provider_err.into();
        assert!(matches!(gateway_err, GatewayError::Provider(_)));
    }

    #[test]
    fn test_provider_error_preserves_inner() {
        let provider_err = ProviderError::RateLimit {
            provider: "anthropic",
            message: "Too many requests".to_string(),
            retry_after: Some(60),
            rpm_limit: None,
            tpm_limit: None,
            current_usage: None,
        };
        let gateway_err: GatewayError = provider_err.into();
        match &gateway_err {
            GatewayError::Provider(inner) => {
                assert!(inner.to_string().contains("Too many requests"));
            }
            _ => panic!("Expected Provider variant"),
        }
    }

    #[test]
    fn test_all_provider_variants_become_provider() {
        let cases: Vec<ProviderError> = vec![
            ProviderError::Authentication {
                provider: "openai",
                message: "bad key".to_string(),
            },
            ProviderError::ModelNotFound {
                provider: "openai",
                model: "gpt-5".to_string(),
            },
            ProviderError::InvalidRequest {
                provider: "openai",
                message: "bad".to_string(),
            },
            ProviderError::Network {
                provider: "openai",
                message: "refused".to_string(),
            },
            ProviderError::Timeout {
                provider: "openai",
                message: "timed out".to_string(),
            },
            ProviderError::Configuration {
                provider: "azure",
                message: "missing key".to_string(),
            },
            ProviderError::Other {
                provider: "unknown",
                message: "unknown".to_string(),
            },
        ];
        for err in cases {
            let gateway_err: GatewayError = err.into();
            assert!(
                matches!(gateway_err, GatewayError::Provider(_)),
                "Expected Provider variant, got: {:?}",
                gateway_err,
            );
        }
    }

    // ==================== A2A Error Conversion Tests ====================

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

    // ==================== MCP Error Conversion Tests ====================

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
}
