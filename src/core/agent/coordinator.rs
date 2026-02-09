//! Agent Coordinator
//!
//! Provides the `AgentCoordinator` trait and `DefaultCoordinator` implementation
//! for managing multiple agent lifecycles with support for cancellation, timeout,
//! and concurrent execution.

use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use chrono::Utc;
use dashmap::DashMap;
use tokio::sync::broadcast;
use tokio::task::JoinHandle;

use super::error::{AgentError, AgentResult};
use super::types::{AgentId, AgentMetadata, AgentState, CoordinatorStats};

/// Type alias for boxed async task
pub type BoxedTask = Pin<Box<dyn Future<Output = ()> + Send + 'static>>;

/// Agent Coordinator trait
///
/// Defines the interface for managing agent lifecycles.
/// This trait is dyn-compatible for use with trait objects.
#[async_trait]
pub trait AgentCoordinator: Send + Sync {
    /// Spawn a new agent with the given boxed task
    async fn spawn_boxed(&self, task: BoxedTask) -> AgentResult<AgentId>;

    /// Spawn a named agent with a boxed task
    async fn spawn_named_boxed(&self, name: String, task: BoxedTask) -> AgentResult<AgentId>;

    /// Cancel a specific agent
    async fn cancel(&self, agent_id: AgentId) -> AgentResult<()>;

    /// Cancel all running agents
    async fn cancel_all(&self) -> usize;

    /// Get the state of an agent
    async fn state(&self, agent_id: AgentId) -> AgentResult<AgentState>;

    /// Get metadata for an agent
    async fn metadata(&self, agent_id: AgentId) -> AgentResult<AgentMetadata>;

    /// List all agent IDs
    async fn list(&self) -> Vec<AgentId>;

    /// List agents by state
    async fn list_by_state(&self, state: AgentState) -> Vec<AgentId>;

    /// Wait for an agent to complete with optional timeout
    async fn wait(&self, agent_id: AgentId, timeout: Option<Duration>) -> AgentResult<AgentState>;

    /// Wait for all agents to complete
    async fn wait_all(&self, timeout: Option<Duration>) -> Vec<(AgentId, AgentResult<AgentState>)>;

    /// Get coordinator statistics
    async fn stats(&self) -> CoordinatorStats;

    /// Remove completed/failed/cancelled agents from tracking
    async fn cleanup(&self) -> usize;
}

/// Internal agent entry
struct AgentEntry {
    /// Agent metadata
    metadata: AgentMetadata,
    /// Cancellation sender
    cancel_tx: broadcast::Sender<()>,
    /// Task handle (if running)
    handle: Option<JoinHandle<()>>,
}

/// Shared agent registry type
type AgentRegistry = Arc<DashMap<AgentId, AgentEntry>>;

/// Default implementation of AgentCoordinator
pub struct DefaultCoordinator {
    /// Agent registry (wrapped in Arc for sharing with spawned tasks)
    agents: AgentRegistry,
    /// Default timeout for operations
    default_timeout: Option<Duration>,
}

impl Default for DefaultCoordinator {
    fn default() -> Self {
        Self::new()
    }
}

impl DefaultCoordinator {
    /// Create a new coordinator
    pub fn new() -> Self {
        Self {
            agents: Arc::new(DashMap::new()),
            default_timeout: None,
        }
    }

    /// Create a coordinator with a default timeout
    pub fn with_timeout(timeout: Duration) -> Self {
        Self {
            agents: Arc::new(DashMap::new()),
            default_timeout: Some(timeout),
        }
    }

    /// Get the default timeout
    pub fn default_timeout(&self) -> Option<Duration> {
        self.default_timeout
    }

    /// Spawn a new agent with the given task (generic version)
    pub async fn spawn<F, T>(&self, task: F) -> AgentResult<AgentId>
    where
        F: Future<Output = T> + Send + 'static,
        T: Send + 'static,
    {
        let id = AgentId::new();
        self.spawn_internal(
            id,
            None,
            Box::pin(async move {
                let _ = task.await;
            }),
        )
    }

    /// Spawn a named agent (generic version)
    pub async fn spawn_named<F, T>(&self, name: impl Into<String>, task: F) -> AgentResult<AgentId>
    where
        F: Future<Output = T> + Send + 'static,
        T: Send + 'static,
    {
        let id = AgentId::new();
        self.spawn_internal(
            id,
            Some(name.into()),
            Box::pin(async move {
                let _ = task.await;
            }),
        )
    }

    /// Internal method to spawn an agent
    fn spawn_internal(
        &self,
        id: AgentId,
        name: Option<String>,
        task: BoxedTask,
    ) -> AgentResult<AgentId> {
        // Create cancellation channel
        let (cancel_tx, mut cancel_rx) = broadcast::channel::<()>(1);

        // Create metadata
        let mut metadata = match name {
            Some(n) => AgentMetadata::with_name(id, n),
            None => AgentMetadata::new(id),
        };
        metadata.state = AgentState::Running;
        metadata.started_at = Some(Utc::now());

        // Clone Arc for the spawned task
        let agents = Arc::clone(&self.agents);
        let agent_id = id;

        // Spawn the task
        let handle = tokio::spawn(async move {
            tokio::select! {
                biased;

                _ = cancel_rx.recv() => {
                    // Task was cancelled
                    if let Some(mut entry) = agents.get_mut(&agent_id) {
                        entry.metadata.state = AgentState::Cancelled;
                        entry.metadata.finished_at = Some(Utc::now());
                    }
                }

                _ = task => {
                    // Task completed
                    if let Some(mut entry) = agents.get_mut(&agent_id) {
                        entry.metadata.state = AgentState::Completed;
                        entry.metadata.finished_at = Some(Utc::now());
                    }
                }
            }
        });

        // Store the entry
        let entry = AgentEntry {
            metadata,
            cancel_tx,
            handle: Some(handle),
        };

        self.agents.insert(id, entry);

        Ok(id)
    }

    /// Update agent state to failed
    fn mark_failed(&self, agent_id: AgentId, error: &str) {
        if let Some(mut entry) = self.agents.get_mut(&agent_id) {
            entry.metadata.state = AgentState::Failed;
            entry.metadata.finished_at = Some(Utc::now());
            entry.metadata.error = Some(error.to_string());
        }
    }
}

#[async_trait]
impl AgentCoordinator for DefaultCoordinator {
    async fn spawn_boxed(&self, task: BoxedTask) -> AgentResult<AgentId> {
        let id = AgentId::new();
        self.spawn_internal(id, None, task)
    }

    async fn spawn_named_boxed(&self, name: String, task: BoxedTask) -> AgentResult<AgentId> {
        let id = AgentId::new();
        self.spawn_internal(id, Some(name), task)
    }

    async fn cancel(&self, agent_id: AgentId) -> AgentResult<()> {
        let entry = self
            .agents
            .get(&agent_id)
            .ok_or(AgentError::NotFound { agent_id })?;

        if !entry.metadata.state.can_cancel() {
            return Err(AgentError::InvalidStateTransition {
                agent_id,
                from: entry.metadata.state.to_string(),
                to: "Cancelled".to_string(),
            });
        }

        // Send cancellation signal
        let _ = entry.cancel_tx.send(());

        Ok(())
    }

    async fn cancel_all(&self) -> usize {
        let mut cancelled = 0;

        for entry in self.agents.iter() {
            if entry.metadata.state.can_cancel() {
                let _ = entry.cancel_tx.send(());
                cancelled += 1;
            }
        }

        cancelled
    }

    async fn state(&self, agent_id: AgentId) -> AgentResult<AgentState> {
        self.agents
            .get(&agent_id)
            .map(|e| e.metadata.state)
            .ok_or(AgentError::NotFound { agent_id })
    }

    async fn metadata(&self, agent_id: AgentId) -> AgentResult<AgentMetadata> {
        self.agents
            .get(&agent_id)
            .map(|e| e.metadata.clone())
            .ok_or(AgentError::NotFound { agent_id })
    }

    async fn list(&self) -> Vec<AgentId> {
        self.agents.iter().map(|e| *e.key()).collect()
    }

    async fn list_by_state(&self, state: AgentState) -> Vec<AgentId> {
        self.agents
            .iter()
            .filter(|e| e.metadata.state == state)
            .map(|e| *e.key())
            .collect()
    }

    async fn wait(&self, agent_id: AgentId, timeout: Option<Duration>) -> AgentResult<AgentState> {
        let timeout = timeout.or(self.default_timeout);

        // Get the handle
        let handle = {
            let mut entry = self
                .agents
                .get_mut(&agent_id)
                .ok_or(AgentError::NotFound { agent_id })?;

            // If already terminal, return immediately
            if entry.metadata.state.is_terminal() {
                return Ok(entry.metadata.state);
            }

            entry.handle.take()
        };

        if let Some(handle) = handle {
            let result = match timeout {
                Some(t) => {
                    match tokio::time::timeout(t, handle).await {
                        Ok(Ok(())) => Ok(()),
                        Ok(Err(e)) => {
                            self.mark_failed(agent_id, &e.to_string());
                            Err(AgentError::ExecutionFailed {
                                agent_id,
                                message: e.to_string(),
                            })
                        }
                        Err(_) => {
                            // Timeout - cancel the agent
                            if let Some(entry) = self.agents.get(&agent_id) {
                                let _ = entry.cancel_tx.send(());
                            }
                            self.mark_failed(agent_id, "timeout");
                            Err(AgentError::Timeout {
                                agent_id,
                                timeout_ms: t.as_millis() as u64,
                            })
                        }
                    }
                }
                None => match handle.await {
                    Ok(()) => Ok(()),
                    Err(e) => {
                        self.mark_failed(agent_id, &e.to_string());
                        Err(AgentError::ExecutionFailed {
                            agent_id,
                            message: e.to_string(),
                        })
                    }
                },
            };

            result?;
        }

        // Return final state
        self.state(agent_id).await
    }

    async fn wait_all(&self, timeout: Option<Duration>) -> Vec<(AgentId, AgentResult<AgentState>)> {
        let ids: Vec<AgentId> = self.list().await;
        let mut results = Vec::with_capacity(ids.len());

        for id in ids {
            let result = self.wait(id, timeout).await;
            results.push((id, result));
        }

        results
    }

    async fn stats(&self) -> CoordinatorStats {
        let mut stats = CoordinatorStats {
            total_created: self.agents.len(),
            ..Default::default()
        };

        for entry in self.agents.iter() {
            match entry.metadata.state {
                AgentState::Idle => stats.idle += 1,
                AgentState::Running => stats.running += 1,
                AgentState::Completed => stats.completed += 1,
                AgentState::Failed => stats.failed += 1,
                AgentState::Cancelled => stats.cancelled += 1,
            }
        }

        stats
    }

    async fn cleanup(&self) -> usize {
        let terminal_ids: Vec<AgentId> = self
            .agents
            .iter()
            .filter(|e| e.metadata.state.is_terminal())
            .map(|e| *e.key())
            .collect();

        let count = terminal_ids.len();

        for id in terminal_ids {
            self.agents.remove(&id);
        }

        count
    }
}

/// Thread-safe coordinator handle
pub type CoordinatorHandle = Arc<dyn AgentCoordinator>;

#[cfg(test)]
#[path = "coordinator_tests.rs"]
mod tests;
