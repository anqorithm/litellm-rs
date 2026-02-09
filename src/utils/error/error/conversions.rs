//! Type conversions for GatewayError

use super::types::GatewayError;
use crate::core::a2a::error::A2AError;
use crate::core::mcp::error::McpError;
use crate::core::providers::unified_provider::ProviderError;

// Conversion from unified ProviderError to GatewayError
impl From<ProviderError> for GatewayError {
    fn from(err: ProviderError) -> Self {
        match err {
            ProviderError::Authentication { message, .. } => GatewayError::Auth(message),
            ProviderError::RateLimit { message, .. } => GatewayError::RateLimit(message),
            ProviderError::ModelNotFound { model, .. } => {
                GatewayError::NotFound(format!("Model not found: {}", model))
            }
            ProviderError::InvalidRequest { message, .. } => GatewayError::BadRequest(message),
            ProviderError::Network { message, .. } => GatewayError::network(message),
            ProviderError::ProviderUnavailable { message, .. } => {
                GatewayError::ProviderUnavailable(message)
            }
            ProviderError::NotSupported { feature, provider } => GatewayError::NotImplemented(
                format!("Feature '{}' not supported by {}", feature, provider),
            ),
            ProviderError::NotImplemented { feature, provider } => GatewayError::NotImplemented(
                format!("Feature '{}' not implemented for {}", feature, provider),
            ),
            ProviderError::Configuration { message, .. } => GatewayError::Config(message),
            ProviderError::Serialization { message, .. } => GatewayError::parsing(message),
            ProviderError::Timeout { message, .. } => GatewayError::Timeout(message),
            ProviderError::QuotaExceeded { message, .. } => {
                GatewayError::BadRequest(format!("Quota exceeded: {}", message))
            }
            ProviderError::Other { message, .. } => GatewayError::Internal(message),

            // Enhanced error variants mapping
            ProviderError::ContextLengthExceeded {
                max,
                actual,
                provider,
            } => GatewayError::BadRequest(format!(
                "Context length exceeded for {}: max {} tokens, got {} tokens",
                provider, max, actual
            )),
            ProviderError::ContentFiltered {
                reason, provider, ..
            } => GatewayError::BadRequest(format!(
                "Content filtered by {} safety systems: {}",
                provider, reason
            )),
            ProviderError::ApiError {
                status,
                message,
                provider,
            } => match status {
                401 => GatewayError::Auth(format!("{}: {}", provider, message)),
                404 => GatewayError::NotFound(format!("{}: {}", provider, message)),
                429 => GatewayError::RateLimit(format!("{}: {}", provider, message)),
                400..=499 => GatewayError::BadRequest(format!("{}: {}", provider, message)),
                _ => GatewayError::Internal(format!("{}: {}", provider, message)),
            },
            ProviderError::TokenLimitExceeded { message, provider } => GatewayError::BadRequest(
                format!("Token limit exceeded for {}: {}", provider, message),
            ),
            ProviderError::FeatureDisabled { feature, provider } => GatewayError::NotImplemented(
                format!("Feature '{}' disabled for {}", feature, provider),
            ),
            ProviderError::DeploymentError {
                deployment,
                message,
                provider,
            } => GatewayError::NotFound(format!(
                "Azure deployment '{}' error for {}: {}",
                deployment, provider, message
            )),
            ProviderError::ResponseParsing { message, provider } => GatewayError::parsing(format!(
                "Failed to parse {} response: {}",
                provider, message
            )),
            ProviderError::RoutingError {
                attempted_providers,
                message,
                provider,
            } => GatewayError::ProviderUnavailable(format!(
                "Routing error from {}: tried {:?}, final error: {}",
                provider, attempted_providers, message
            )),
            ProviderError::TransformationError {
                from_format,
                to_format,
                message,
                provider,
            } => GatewayError::Internal(format!(
                "Transformation error for {}: from {} to {}: {}",
                provider, from_format, to_format, message
            )),
            ProviderError::Cancelled {
                provider,
                operation_type,
                ..
            } => GatewayError::BadRequest(format!(
                "Operation {} was cancelled for provider {}",
                operation_type, provider
            )),
            ProviderError::Streaming {
                provider,
                stream_type,
                ..
            } => GatewayError::Internal(format!(
                "Streaming error for provider {} on stream type {}",
                provider, stream_type
            )),
        }
    }
}

// Conversion from A2AError to GatewayError
impl From<A2AError> for GatewayError {
    fn from(err: A2AError) -> Self {
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
                GatewayError::Parsing(format!("A2A serialization error: {}", message))
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
                GatewayError::RateLimit(msg)
            }
            A2AError::AgentBusy {
                agent_name,
                message,
            } => GatewayError::ProviderUnavailable(format!(
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
                GatewayError::Authorization(msg)
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
                GatewayError::Parsing(format!("MCP serialization error: {}", message))
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
                GatewayError::RateLimit(msg)
            }
        }
    }
}


#[cfg(test)]
#[path = "conversions_tests.rs"]
mod tests;
