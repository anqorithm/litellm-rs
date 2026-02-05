//! Agent Coordinator Module
//!
//! This module provides agent coordination functionality for managing multiple
//! concurrent agent lifecycles with support for cancellation, timeout control,
//! and state management.
//!
//! # Overview
//!
//! The Agent Coordinator is inspired by Crush's Coordinator design and provides:
//!
//! - Unique agent identification via `AgentId`
//! - State tracking with `AgentState` (Idle, Running, Completed, Failed, Cancelled)
//! - Concurrent execution management
//! - Cancellation support (single or all agents)
//! - Timeout control
//! - Statistics and cleanup
//!
//! # Usage
//!
//! ```rust,no_run
//! # use litellm_rs::core::agent::{DefaultCoordinator, AgentCoordinator, AgentState};
//! # use std::time::Duration;
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Create a coordinator
//! let coordinator = DefaultCoordinator::new();
//!
//! // Spawn an agent
//! let agent_id = coordinator.spawn(async {
//!     // Agent work here
//!     42
//! }).await?;
//!
//! // Check state
//! let state = coordinator.state(agent_id).await?;
//! println!("Agent state: {}", state);
//!
//! // Wait for completion with timeout
//! let final_state = coordinator.wait(agent_id, Some(Duration::from_secs(30))).await?;
//!
//! // Get statistics
//! let stats = coordinator.stats().await;
//! println!("Completed: {}, Failed: {}", stats.completed, stats.failed);
//! # Ok(())
//! # }
//! ```
//!
//! # Cancellation
//!
//! ```rust,no_run
//! # use litellm_rs::core::agent::{DefaultCoordinator, AgentCoordinator};
//! # use std::time::Duration;
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let coordinator = DefaultCoordinator::new();
//!
//! // Spawn a long-running agent
//! let agent_id = coordinator.spawn(async {
//!     tokio::time::sleep(Duration::from_secs(60)).await;
//! }).await?;
//!
//! // Cancel it
//! coordinator.cancel(agent_id).await?;
//!
//! // Or cancel all agents
//! let cancelled_count = coordinator.cancel_all().await;
//! # Ok(())
//! # }
//! ```

pub mod coordinator;
pub mod error;
pub mod types;

// Re-export commonly used types
pub use coordinator::{AgentCoordinator, BoxedTask, CoordinatorHandle, DefaultCoordinator};
pub use error::{AgentError, AgentResult};
pub use types::{AgentId, AgentMetadata, AgentState, CoordinatorStats};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_module_exports() {
        // Verify all public types are accessible
        let _ = AgentState::Idle;
        let _ = AgentId::new();
    }

    #[tokio::test]
    async fn test_coordinator_integration() {
        let coord = DefaultCoordinator::new();

        // Spawn a simple task
        let id = coord.spawn(async { 42 }).await.unwrap();

        // Wait for completion
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;

        // Verify state
        let state = coord.state(id).await.unwrap();
        assert_eq!(state, AgentState::Completed);

        // Verify stats
        let stats = coord.stats().await;
        assert_eq!(stats.completed, 1);
    }
}
