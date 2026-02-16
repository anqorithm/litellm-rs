//! A2A Error types
//!
//! Defines error types for A2A protocol operations.

use crate::{impl_from_reqwest_error, impl_from_serde_error};
use std::fmt;

/// Result type for A2A operations
pub type A2AResult<T> = Result<T, A2AError>;

/// A2A-specific errors
#[derive(Debug, Clone)]
pub enum A2AError {
    /// Agent not found
    AgentNotFound { agent_name: String },

    /// Agent already registered
    AgentAlreadyExists { agent_name: String },

    /// Connection error
    ConnectionError { agent_name: String, message: String },

    /// Authentication error
    AuthenticationError { agent_name: String, message: String },

    /// Task not found
    TaskNotFound { agent_name: String, task_id: String },

    /// Task failed
    TaskFailed {
        agent_name: String,
        task_id: String,
        message: String,
    },

    /// Protocol error (invalid JSON-RPC message)
    ProtocolError { message: String },

    /// Invalid request
    InvalidRequest { message: String },

    /// Timeout error
    Timeout { agent_name: String, timeout_ms: u64 },

    /// Configuration error
    ConfigurationError { message: String },

    /// Serialization error
    SerializationError { message: String },

    /// Provider not supported
    UnsupportedProvider { provider: String },

    /// Rate limit exceeded
    RateLimitExceeded {
        agent_name: String,
        retry_after_ms: Option<u64>,
    },

    /// Agent busy
    AgentBusy { agent_name: String, message: String },

    /// Content blocked by moderation
    ContentBlocked { agent_name: String, reason: String },
}

impl fmt::Display for A2AError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            A2AError::AgentNotFound { agent_name } => {
                write!(f, "Agent not found: {}", agent_name)
            }
            A2AError::AgentAlreadyExists { agent_name } => {
                write!(f, "Agent already registered: {}", agent_name)
            }
            A2AError::ConnectionError {
                agent_name,
                message,
            } => {
                write!(f, "Connection error to agent '{}': {}", agent_name, message)
            }
            A2AError::AuthenticationError {
                agent_name,
                message,
            } => {
                write!(
                    f,
                    "Authentication failed for agent '{}': {}",
                    agent_name, message
                )
            }
            A2AError::TaskNotFound {
                agent_name,
                task_id,
            } => {
                write!(f, "Task '{}' not found on agent '{}'", task_id, agent_name)
            }
            A2AError::TaskFailed {
                agent_name,
                task_id,
                message,
            } => {
                write!(
                    f,
                    "Task '{}' failed on agent '{}': {}",
                    task_id, agent_name, message
                )
            }
            A2AError::ProtocolError { message } => {
                write!(f, "A2A protocol error: {}", message)
            }
            A2AError::InvalidRequest { message } => {
                write!(f, "Invalid A2A request: {}", message)
            }
            A2AError::Timeout {
                agent_name,
                timeout_ms,
            } => {
                write!(
                    f,
                    "Timeout waiting for agent '{}' ({}ms)",
                    agent_name, timeout_ms
                )
            }
            A2AError::ConfigurationError { message } => {
                write!(f, "A2A configuration error: {}", message)
            }
            A2AError::SerializationError { message } => {
                write!(f, "A2A serialization error: {}", message)
            }
            A2AError::UnsupportedProvider { provider } => {
                write!(f, "Unsupported agent provider: {}", provider)
            }
            A2AError::RateLimitExceeded {
                agent_name,
                retry_after_ms,
            } => {
                if let Some(ms) = retry_after_ms {
                    write!(
                        f,
                        "Rate limit exceeded for agent '{}', retry after {}ms",
                        agent_name, ms
                    )
                } else {
                    write!(f, "Rate limit exceeded for agent '{}'", agent_name)
                }
            }
            A2AError::AgentBusy {
                agent_name,
                message,
            } => {
                write!(f, "Agent '{}' is busy: {}", agent_name, message)
            }
            A2AError::ContentBlocked { agent_name, reason } => {
                write!(f, "Content blocked by agent '{}': {}", agent_name, reason)
            }
        }
    }
}

impl std::error::Error for A2AError {}

impl_from_serde_error!(A2AError, |e| A2AError::SerializationError {
    message: e.to_string(),
});

impl_from_reqwest_error!(A2AError,
    timeout => |_e| A2AError::Timeout {
        agent_name: "unknown".to_string(),
        timeout_ms: 0,
    },
    connect => |e| A2AError::ConnectionError {
        agent_name: "unknown".to_string(),
        message: e.to_string(),
    },
    other => |e| A2AError::ProtocolError {
        message: e.to_string(),
    }
);

#[cfg(test)]
mod tests {
    use super::*;

    // ==================== Display Tests for All Variants ====================

    #[test]
    fn test_agent_not_found_display() {
        let err = A2AError::AgentNotFound {
            agent_name: "my-agent".to_string(),
        };
        assert!(err.to_string().contains("my-agent"));
        assert!(err.to_string().contains("not found"));
    }

    #[test]
    fn test_agent_already_exists_display() {
        let err = A2AError::AgentAlreadyExists {
            agent_name: "existing-agent".to_string(),
        };
        let msg = err.to_string();
        assert!(msg.contains("existing-agent"));
        assert!(msg.contains("already registered"));
    }

    #[test]
    fn test_connection_error_display() {
        let err = A2AError::ConnectionError {
            agent_name: "remote-agent".to_string(),
            message: "Connection refused".to_string(),
        };
        let msg = err.to_string();
        assert!(msg.contains("remote-agent"));
        assert!(msg.contains("Connection refused"));
        assert!(msg.contains("Connection error"));
    }

    #[test]
    fn test_authentication_error_display() {
        let err = A2AError::AuthenticationError {
            agent_name: "secure-agent".to_string(),
            message: "Invalid token".to_string(),
        };
        let msg = err.to_string();
        assert!(msg.contains("secure-agent"));
        assert!(msg.contains("Invalid token"));
        assert!(msg.contains("Authentication failed"));
    }

    #[test]
    fn test_task_not_found_display() {
        let err = A2AError::TaskNotFound {
            agent_name: "agent".to_string(),
            task_id: "task-456".to_string(),
        };
        let msg = err.to_string();
        assert!(msg.contains("agent"));
        assert!(msg.contains("task-456"));
        assert!(msg.contains("not found"));
    }

    #[test]
    fn test_task_failed_display() {
        let err = A2AError::TaskFailed {
            agent_name: "agent".to_string(),
            task_id: "task-123".to_string(),
            message: "Something went wrong".to_string(),
        };
        let msg = err.to_string();
        assert!(msg.contains("task-123"));
        assert!(msg.contains("Something went wrong"));
    }

    #[test]
    fn test_protocol_error_display() {
        let err = A2AError::ProtocolError {
            message: "Invalid JSON-RPC version".to_string(),
        };
        let msg = err.to_string();
        assert!(msg.contains("Invalid JSON-RPC version"));
        assert!(msg.contains("protocol error"));
    }

    #[test]
    fn test_invalid_request_display() {
        let err = A2AError::InvalidRequest {
            message: "Missing required field".to_string(),
        };
        let msg = err.to_string();
        assert!(msg.contains("Missing required field"));
        assert!(msg.contains("Invalid"));
    }

    #[test]
    fn test_timeout_display() {
        let err = A2AError::Timeout {
            agent_name: "slow-agent".to_string(),
            timeout_ms: 30000,
        };
        let msg = err.to_string();
        assert!(msg.contains("slow-agent"));
        assert!(msg.contains("30000"));
        assert!(msg.contains("Timeout"));
    }

    #[test]
    fn test_configuration_error_display() {
        let err = A2AError::ConfigurationError {
            message: "Missing endpoint URL".to_string(),
        };
        let msg = err.to_string();
        assert!(msg.contains("Missing endpoint URL"));
        assert!(msg.contains("configuration error"));
    }

    #[test]
    fn test_serialization_error_display() {
        let err = A2AError::SerializationError {
            message: "Invalid UTF-8".to_string(),
        };
        let msg = err.to_string();
        assert!(msg.contains("Invalid UTF-8"));
        assert!(msg.contains("serialization error"));
    }

    #[test]
    fn test_unsupported_provider_display() {
        let err = A2AError::UnsupportedProvider {
            provider: "unknown-provider".to_string(),
        };
        let msg = err.to_string();
        assert!(msg.contains("unknown-provider"));
        assert!(msg.contains("Unsupported"));
    }

    #[test]
    fn test_rate_limit_with_retry() {
        let err = A2AError::RateLimitExceeded {
            agent_name: "agent".to_string(),
            retry_after_ms: Some(5000),
        };
        assert!(err.to_string().contains("5000ms"));
    }

    #[test]
    fn test_rate_limit_without_retry() {
        let err = A2AError::RateLimitExceeded {
            agent_name: "agent".to_string(),
            retry_after_ms: None,
        };
        let msg = err.to_string();
        assert!(msg.contains("agent"));
        assert!(msg.contains("Rate limit exceeded"));
        assert!(!msg.contains("retry after"));
    }

    #[test]
    fn test_agent_busy_display() {
        let err = A2AError::AgentBusy {
            agent_name: "busy-agent".to_string(),
            message: "Processing another request".to_string(),
        };
        let msg = err.to_string();
        assert!(msg.contains("busy-agent"));
        assert!(msg.contains("busy"));
        assert!(msg.contains("Processing another request"));
    }

    #[test]
    fn test_content_blocked_display() {
        let err = A2AError::ContentBlocked {
            agent_name: "safe-agent".to_string(),
            reason: "Harmful content detected".to_string(),
        };
        let msg = err.to_string();
        assert!(msg.contains("safe-agent"));
        assert!(msg.contains("Harmful content detected"));
        assert!(msg.contains("blocked"));
    }

    // ==================== Error Trait Tests ====================

    #[test]
    fn test_error_is_error_trait() {
        let err: Box<dyn std::error::Error> = Box::new(A2AError::AgentNotFound {
            agent_name: "test".to_string(),
        });
        assert!(!err.to_string().is_empty());
    }

    #[test]
    fn test_all_variants_implement_error() {
        let errors: Vec<Box<dyn std::error::Error>> = vec![
            Box::new(A2AError::AgentNotFound {
                agent_name: "a".to_string(),
            }),
            Box::new(A2AError::AgentAlreadyExists {
                agent_name: "a".to_string(),
            }),
            Box::new(A2AError::ConnectionError {
                agent_name: "a".to_string(),
                message: "m".to_string(),
            }),
            Box::new(A2AError::AuthenticationError {
                agent_name: "a".to_string(),
                message: "m".to_string(),
            }),
            Box::new(A2AError::TaskNotFound {
                agent_name: "a".to_string(),
                task_id: "t".to_string(),
            }),
            Box::new(A2AError::TaskFailed {
                agent_name: "a".to_string(),
                task_id: "t".to_string(),
                message: "m".to_string(),
            }),
            Box::new(A2AError::ProtocolError {
                message: "m".to_string(),
            }),
            Box::new(A2AError::InvalidRequest {
                message: "m".to_string(),
            }),
            Box::new(A2AError::Timeout {
                agent_name: "a".to_string(),
                timeout_ms: 1000,
            }),
            Box::new(A2AError::ConfigurationError {
                message: "m".to_string(),
            }),
            Box::new(A2AError::SerializationError {
                message: "m".to_string(),
            }),
            Box::new(A2AError::UnsupportedProvider {
                provider: "p".to_string(),
            }),
            Box::new(A2AError::RateLimitExceeded {
                agent_name: "a".to_string(),
                retry_after_ms: None,
            }),
            Box::new(A2AError::AgentBusy {
                agent_name: "a".to_string(),
                message: "m".to_string(),
            }),
            Box::new(A2AError::ContentBlocked {
                agent_name: "a".to_string(),
                reason: "r".to_string(),
            }),
        ];

        for err in errors {
            assert!(!err.to_string().is_empty());
        }
    }

    // ==================== From Trait Tests ====================

    #[test]
    fn test_from_serde_json_error() {
        let json_err = serde_json::from_str::<serde_json::Value>("invalid json").unwrap_err();
        let err: A2AError = json_err.into();
        match err {
            A2AError::SerializationError { message } => {
                assert!(!message.is_empty());
            }
            _ => panic!("Expected SerializationError"),
        }
    }

    // ==================== Clone Tests ====================

    #[test]
    fn test_error_clone() {
        let err = A2AError::AgentNotFound {
            agent_name: "agent".to_string(),
        };
        let cloned = err.clone();
        assert_eq!(err.to_string(), cloned.to_string());
    }

    #[test]
    fn test_clone_all_variants() {
        let errors = vec![
            A2AError::AgentNotFound {
                agent_name: "a".to_string(),
            },
            A2AError::ConnectionError {
                agent_name: "a".to_string(),
                message: "m".to_string(),
            },
            A2AError::TaskFailed {
                agent_name: "a".to_string(),
                task_id: "t".to_string(),
                message: "m".to_string(),
            },
            A2AError::RateLimitExceeded {
                agent_name: "a".to_string(),
                retry_after_ms: Some(1000),
            },
        ];

        for err in errors {
            let cloned = err.clone();
            assert_eq!(err.to_string(), cloned.to_string());
        }
    }

    // ==================== Debug Tests ====================

    #[test]
    fn test_error_debug() {
        let err = A2AError::AgentNotFound {
            agent_name: "agent".to_string(),
        };
        let debug = format!("{:?}", err);
        assert!(debug.contains("AgentNotFound"));
        assert!(debug.contains("agent"));
    }

    #[test]
    fn test_debug_task_failed() {
        let err = A2AError::TaskFailed {
            agent_name: "agent".to_string(),
            task_id: "task-123".to_string(),
            message: "Error message".to_string(),
        };
        let debug = format!("{:?}", err);
        assert!(debug.contains("TaskFailed"));
        assert!(debug.contains("task-123"));
    }

    // ==================== A2AResult Type Alias Tests ====================

    #[test]
    fn test_a2a_result_ok() {
        let result: A2AResult<i32> = Ok(42);
        assert!(result.is_ok());
        assert!(matches!(result, Ok(42)));
    }

    #[test]
    fn test_a2a_result_err() {
        let result: A2AResult<i32> = Err(A2AError::AgentNotFound {
            agent_name: "agent".to_string(),
        });
        assert!(result.is_err());
    }

    #[test]
    fn test_a2a_result_map() {
        let result: A2AResult<i32> = Ok(42);
        let mapped = result.map(|x| x * 2);
        assert_eq!(mapped.unwrap(), 84);
    }

    #[test]
    fn test_a2a_result_and_then() {
        let result: A2AResult<i32> = Ok(42);
        let chained: A2AResult<String> = result.map(|x| x.to_string());
        assert!(matches!(chained, Ok(ref s) if s == "42"));
    }

    // ==================== Edge Case Tests ====================

    #[test]
    fn test_empty_agent_name() {
        let err = A2AError::AgentNotFound {
            agent_name: "".to_string(),
        };
        let msg = err.to_string();
        assert!(msg.contains("not found"));
    }

    #[test]
    fn test_empty_message() {
        let err = A2AError::ProtocolError {
            message: "".to_string(),
        };
        let msg = err.to_string();
        assert!(msg.contains("protocol error"));
    }

    #[test]
    fn test_special_characters_in_names() {
        let err = A2AError::AgentNotFound {
            agent_name: "agent/with/slashes".to_string(),
        };
        let msg = err.to_string();
        assert!(msg.contains("agent/with/slashes"));
    }

    #[test]
    fn test_unicode_in_message() {
        let err = A2AError::TaskFailed {
            agent_name: "agent".to_string(),
            task_id: "task".to_string(),
            message: "错误消息 🚨".to_string(),
        };
        let msg = err.to_string();
        assert!(msg.contains("错误消息"));
        assert!(msg.contains("🚨"));
    }

    #[test]
    fn test_zero_timeout() {
        let err = A2AError::Timeout {
            agent_name: "agent".to_string(),
            timeout_ms: 0,
        };
        let msg = err.to_string();
        assert!(msg.contains("0ms"));
    }

    #[test]
    fn test_large_timeout() {
        let err = A2AError::Timeout {
            agent_name: "agent".to_string(),
            timeout_ms: u64::MAX,
        };
        let msg = err.to_string();
        assert!(msg.contains(&u64::MAX.to_string()));
    }
}
