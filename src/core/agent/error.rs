//! Agent Coordinator Error Types
//!
//! Defines error types for agent coordination operations.

use std::fmt;

use super::types::AgentId;

/// Result type for agent operations
pub type AgentResult<T> = Result<T, AgentError>;

/// Agent-specific errors
#[derive(Debug, Clone)]
pub enum AgentError {
    /// Agent not found
    NotFound { agent_id: AgentId },

    /// Agent already exists
    AlreadyExists { agent_id: AgentId },

    /// Agent execution failed
    ExecutionFailed { agent_id: AgentId, message: String },

    /// Agent was cancelled
    Cancelled { agent_id: AgentId },

    /// Agent timed out
    Timeout { agent_id: AgentId, timeout_ms: u64 },

    /// Invalid state transition
    InvalidStateTransition {
        agent_id: AgentId,
        from: String,
        to: String,
    },

    /// Spawn error
    SpawnError { message: String },

    /// Internal error
    Internal { message: String },
}

impl fmt::Display for AgentError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AgentError::NotFound { agent_id } => {
                write!(f, "Agent not found: {}", agent_id)
            }
            AgentError::AlreadyExists { agent_id } => {
                write!(f, "Agent already exists: {}", agent_id)
            }
            AgentError::ExecutionFailed { agent_id, message } => {
                write!(f, "Agent '{}' execution failed: {}", agent_id, message)
            }
            AgentError::Cancelled { agent_id } => {
                write!(f, "Agent '{}' was cancelled", agent_id)
            }
            AgentError::Timeout {
                agent_id,
                timeout_ms,
            } => {
                write!(f, "Agent '{}' timed out after {}ms", agent_id, timeout_ms)
            }
            AgentError::InvalidStateTransition { agent_id, from, to } => {
                write!(
                    f,
                    "Invalid state transition for agent '{}': {} -> {}",
                    agent_id, from, to
                )
            }
            AgentError::SpawnError { message } => {
                write!(f, "Failed to spawn agent: {}", message)
            }
            AgentError::Internal { message } => {
                write!(f, "Internal agent error: {}", message)
            }
        }
    }
}

impl std::error::Error for AgentError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_not_found_display() {
        let err = AgentError::NotFound {
            agent_id: AgentId::new(),
        };
        assert!(err.to_string().contains("not found"));
    }

    #[test]
    fn test_already_exists_display() {
        let err = AgentError::AlreadyExists {
            agent_id: AgentId::new(),
        };
        assert!(err.to_string().contains("already exists"));
    }

    #[test]
    fn test_execution_failed_display() {
        let err = AgentError::ExecutionFailed {
            agent_id: AgentId::new(),
            message: "task panicked".to_string(),
        };
        let msg = err.to_string();
        assert!(msg.contains("execution failed"));
        assert!(msg.contains("task panicked"));
    }

    #[test]
    fn test_cancelled_display() {
        let err = AgentError::Cancelled {
            agent_id: AgentId::new(),
        };
        assert!(err.to_string().contains("cancelled"));
    }

    #[test]
    fn test_timeout_display() {
        let err = AgentError::Timeout {
            agent_id: AgentId::new(),
            timeout_ms: 5000,
        };
        let msg = err.to_string();
        assert!(msg.contains("timed out"));
        assert!(msg.contains("5000ms"));
    }

    #[test]
    fn test_invalid_state_transition_display() {
        let err = AgentError::InvalidStateTransition {
            agent_id: AgentId::new(),
            from: "Idle".to_string(),
            to: "Completed".to_string(),
        };
        let msg = err.to_string();
        assert!(msg.contains("Invalid state transition"));
        assert!(msg.contains("Idle"));
        assert!(msg.contains("Completed"));
    }

    #[test]
    fn test_spawn_error_display() {
        let err = AgentError::SpawnError {
            message: "thread pool exhausted".to_string(),
        };
        assert!(err.to_string().contains("thread pool exhausted"));
    }

    #[test]
    fn test_internal_error_display() {
        let err = AgentError::Internal {
            message: "unexpected state".to_string(),
        };
        assert!(err.to_string().contains("unexpected state"));
    }

    #[test]
    fn test_error_is_error_trait() {
        let err: Box<dyn std::error::Error> = Box::new(AgentError::NotFound {
            agent_id: AgentId::new(),
        });
        assert!(!err.to_string().is_empty());
    }

    #[test]
    fn test_error_clone() {
        let err = AgentError::ExecutionFailed {
            agent_id: AgentId::new(),
            message: "test".to_string(),
        };
        let cloned = err.clone();
        assert!(cloned.to_string().contains("test"));
    }

    #[test]
    fn test_agent_result_ok() {
        let result: AgentResult<i32> = Ok(42);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 42);
    }

    #[test]
    fn test_agent_result_err() {
        let result: AgentResult<i32> = Err(AgentError::NotFound {
            agent_id: AgentId::new(),
        });
        assert!(result.is_err());
    }
}
