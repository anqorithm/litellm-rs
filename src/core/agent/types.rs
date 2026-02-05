//! Agent Coordinator Types
//!
//! Core types for agent coordination including AgentId and AgentState.

use std::fmt;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Unique identifier for an agent
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AgentId(Uuid);

impl AgentId {
    /// Create a new random AgentId
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    /// Create an AgentId from a UUID
    pub fn from_uuid(uuid: Uuid) -> Self {
        Self(uuid)
    }

    /// Parse an AgentId from a string
    pub fn parse(s: &str) -> Result<Self, uuid::Error> {
        Ok(Self(Uuid::parse_str(s)?))
    }

    /// Get the inner UUID
    pub fn as_uuid(&self) -> &Uuid {
        &self.0
    }

    /// Convert to string representation
    pub fn to_string_repr(&self) -> String {
        self.0.to_string()
    }
}

impl Default for AgentId {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for AgentId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<Uuid> for AgentId {
    fn from(uuid: Uuid) -> Self {
        Self(uuid)
    }
}

/// Agent execution state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AgentState {
    /// Agent is idle and ready to run
    #[default]
    Idle,
    /// Agent is currently running
    Running,
    /// Agent completed successfully
    Completed,
    /// Agent failed with an error
    Failed,
    /// Agent was cancelled
    Cancelled,
}

impl AgentState {
    /// Check if the state is terminal (no further transitions expected)
    pub fn is_terminal(&self) -> bool {
        matches!(self, Self::Completed | Self::Failed | Self::Cancelled)
    }

    /// Check if the agent is currently active
    pub fn is_active(&self) -> bool {
        matches!(self, Self::Running)
    }

    /// Check if the agent can be started
    pub fn can_start(&self) -> bool {
        matches!(self, Self::Idle)
    }

    /// Check if the agent can be cancelled
    pub fn can_cancel(&self) -> bool {
        matches!(self, Self::Idle | Self::Running)
    }
}

impl fmt::Display for AgentState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Idle => write!(f, "Idle"),
            Self::Running => write!(f, "Running"),
            Self::Completed => write!(f, "Completed"),
            Self::Failed => write!(f, "Failed"),
            Self::Cancelled => write!(f, "Cancelled"),
        }
    }
}

/// Metadata about an agent's execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentMetadata {
    /// Agent identifier
    pub id: AgentId,
    /// Optional name for the agent
    pub name: Option<String>,
    /// Current state
    pub state: AgentState,
    /// When the agent was created
    pub created_at: DateTime<Utc>,
    /// When the agent started running
    pub started_at: Option<DateTime<Utc>>,
    /// When the agent finished (completed, failed, or cancelled)
    pub finished_at: Option<DateTime<Utc>>,
    /// Error message if failed
    pub error: Option<String>,
}

impl AgentMetadata {
    /// Create new metadata for an agent
    pub fn new(id: AgentId) -> Self {
        Self {
            id,
            name: None,
            state: AgentState::Idle,
            created_at: Utc::now(),
            started_at: None,
            finished_at: None,
            error: None,
        }
    }

    /// Create metadata with a name
    pub fn with_name(id: AgentId, name: impl Into<String>) -> Self {
        Self {
            id,
            name: Some(name.into()),
            state: AgentState::Idle,
            created_at: Utc::now(),
            started_at: None,
            finished_at: None,
            error: None,
        }
    }

    /// Get execution duration if available
    pub fn duration(&self) -> Option<chrono::Duration> {
        match (self.started_at, self.finished_at) {
            (Some(start), Some(end)) => Some(end - start),
            (Some(start), None) if self.state == AgentState::Running => Some(Utc::now() - start),
            _ => None,
        }
    }
}

/// Statistics about the coordinator
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CoordinatorStats {
    /// Total agents created
    pub total_created: usize,
    /// Currently running agents
    pub running: usize,
    /// Completed agents
    pub completed: usize,
    /// Failed agents
    pub failed: usize,
    /// Cancelled agents
    pub cancelled: usize,
    /// Idle agents
    pub idle: usize,
}

impl CoordinatorStats {
    /// Get total active agents (idle + running)
    pub fn active(&self) -> usize {
        self.idle + self.running
    }

    /// Get total finished agents
    pub fn finished(&self) -> usize {
        self.completed + self.failed + self.cancelled
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ==================== AgentId Tests ====================

    #[test]
    fn test_agent_id_new() {
        let id1 = AgentId::new();
        let id2 = AgentId::new();
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_agent_id_default() {
        let id = AgentId::default();
        assert!(!id.to_string().is_empty());
    }

    #[test]
    fn test_agent_id_from_uuid() {
        let uuid = Uuid::new_v4();
        let id = AgentId::from_uuid(uuid);
        assert_eq!(id.as_uuid(), &uuid);
    }

    #[test]
    fn test_agent_id_parse() {
        let uuid_str = "550e8400-e29b-41d4-a716-446655440000";
        let id = AgentId::parse(uuid_str).unwrap();
        assert_eq!(id.to_string(), uuid_str);
    }

    #[test]
    fn test_agent_id_parse_invalid() {
        let result = AgentId::parse("invalid");
        assert!(result.is_err());
    }

    #[test]
    fn test_agent_id_display() {
        let id = AgentId::new();
        let display = format!("{}", id);
        assert!(!display.is_empty());
        assert!(display.contains('-')); // UUID format
    }

    #[test]
    fn test_agent_id_clone() {
        let id = AgentId::new();
        let cloned = id;
        assert_eq!(id, cloned);
    }

    #[test]
    fn test_agent_id_hash() {
        use std::collections::HashSet;
        let mut set = HashSet::new();
        let id = AgentId::new();
        set.insert(id);
        assert!(set.contains(&id));
    }

    #[test]
    fn test_agent_id_serialize() {
        let id = AgentId::new();
        let json = serde_json::to_string(&id).unwrap();
        let deserialized: AgentId = serde_json::from_str(&json).unwrap();
        assert_eq!(id, deserialized);
    }

    // ==================== AgentState Tests ====================

    #[test]
    fn test_agent_state_default() {
        assert_eq!(AgentState::default(), AgentState::Idle);
    }

    #[test]
    fn test_agent_state_is_terminal() {
        assert!(!AgentState::Idle.is_terminal());
        assert!(!AgentState::Running.is_terminal());
        assert!(AgentState::Completed.is_terminal());
        assert!(AgentState::Failed.is_terminal());
        assert!(AgentState::Cancelled.is_terminal());
    }

    #[test]
    fn test_agent_state_is_active() {
        assert!(!AgentState::Idle.is_active());
        assert!(AgentState::Running.is_active());
        assert!(!AgentState::Completed.is_active());
        assert!(!AgentState::Failed.is_active());
        assert!(!AgentState::Cancelled.is_active());
    }

    #[test]
    fn test_agent_state_can_start() {
        assert!(AgentState::Idle.can_start());
        assert!(!AgentState::Running.can_start());
        assert!(!AgentState::Completed.can_start());
        assert!(!AgentState::Failed.can_start());
        assert!(!AgentState::Cancelled.can_start());
    }

    #[test]
    fn test_agent_state_can_cancel() {
        assert!(AgentState::Idle.can_cancel());
        assert!(AgentState::Running.can_cancel());
        assert!(!AgentState::Completed.can_cancel());
        assert!(!AgentState::Failed.can_cancel());
        assert!(!AgentState::Cancelled.can_cancel());
    }

    #[test]
    fn test_agent_state_display() {
        assert_eq!(AgentState::Idle.to_string(), "Idle");
        assert_eq!(AgentState::Running.to_string(), "Running");
        assert_eq!(AgentState::Completed.to_string(), "Completed");
        assert_eq!(AgentState::Failed.to_string(), "Failed");
        assert_eq!(AgentState::Cancelled.to_string(), "Cancelled");
    }

    #[test]
    fn test_agent_state_serialize() {
        let state = AgentState::Running;
        let json = serde_json::to_string(&state).unwrap();
        assert_eq!(json, "\"running\"");

        let deserialized: AgentState = serde_json::from_str(&json).unwrap();
        assert_eq!(state, deserialized);
    }

    // ==================== AgentMetadata Tests ====================

    #[test]
    fn test_agent_metadata_new() {
        let id = AgentId::new();
        let meta = AgentMetadata::new(id);
        assert_eq!(meta.id, id);
        assert_eq!(meta.state, AgentState::Idle);
        assert!(meta.name.is_none());
        assert!(meta.started_at.is_none());
        assert!(meta.finished_at.is_none());
        assert!(meta.error.is_none());
    }

    #[test]
    fn test_agent_metadata_with_name() {
        let id = AgentId::new();
        let meta = AgentMetadata::with_name(id, "test-agent");
        assert_eq!(meta.name, Some("test-agent".to_string()));
    }

    #[test]
    fn test_agent_metadata_duration_not_started() {
        let meta = AgentMetadata::new(AgentId::new());
        assert!(meta.duration().is_none());
    }

    #[test]
    fn test_agent_metadata_duration_completed() {
        let mut meta = AgentMetadata::new(AgentId::new());
        meta.started_at = Some(Utc::now() - chrono::Duration::seconds(10));
        meta.finished_at = Some(Utc::now());
        meta.state = AgentState::Completed;

        let duration = meta.duration().unwrap();
        assert!(duration.num_seconds() >= 9 && duration.num_seconds() <= 11);
    }

    // ==================== CoordinatorStats Tests ====================

    #[test]
    fn test_coordinator_stats_default() {
        let stats = CoordinatorStats::default();
        assert_eq!(stats.total_created, 0);
        assert_eq!(stats.running, 0);
        assert_eq!(stats.completed, 0);
        assert_eq!(stats.failed, 0);
        assert_eq!(stats.cancelled, 0);
        assert_eq!(stats.idle, 0);
    }

    #[test]
    fn test_coordinator_stats_active() {
        let stats = CoordinatorStats {
            idle: 5,
            running: 3,
            ..Default::default()
        };
        assert_eq!(stats.active(), 8);
    }

    #[test]
    fn test_coordinator_stats_finished() {
        let stats = CoordinatorStats {
            completed: 10,
            failed: 2,
            cancelled: 1,
            ..Default::default()
        };
        assert_eq!(stats.finished(), 13);
    }

    #[test]
    fn test_coordinator_stats_serialize() {
        let stats = CoordinatorStats {
            total_created: 100,
            running: 5,
            completed: 80,
            failed: 10,
            cancelled: 5,
            idle: 0,
        };
        let json = serde_json::to_string(&stats).unwrap();
        let deserialized: CoordinatorStats = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.total_created, 100);
        assert_eq!(deserialized.running, 5);
    }
}
