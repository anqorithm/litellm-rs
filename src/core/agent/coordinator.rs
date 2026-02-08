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
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use tokio::time::sleep;

    // ==================== DefaultCoordinator Creation Tests ====================

    #[test]
    fn test_coordinator_new() {
        let coord = DefaultCoordinator::new();
        assert!(coord.default_timeout().is_none());
    }

    #[test]
    fn test_coordinator_with_timeout() {
        let coord = DefaultCoordinator::with_timeout(Duration::from_secs(30));
        assert_eq!(coord.default_timeout(), Some(Duration::from_secs(30)));
    }

    #[test]
    fn test_coordinator_default() {
        let coord = DefaultCoordinator::default();
        assert!(coord.default_timeout().is_none());
    }

    // ==================== Spawn Tests ====================

    #[tokio::test]
    async fn test_spawn_simple() {
        let coord = DefaultCoordinator::new();

        let id = coord.spawn(async { 42 }).await.unwrap();

        // Give task time to complete
        sleep(Duration::from_millis(50)).await;

        let state = coord.state(id).await.unwrap();
        assert!(state.is_terminal());
    }

    #[tokio::test]
    async fn test_spawn_named() {
        let coord = DefaultCoordinator::new();

        let id = coord.spawn_named("test-agent", async { 42 }).await.unwrap();

        let meta = coord.metadata(id).await.unwrap();
        assert_eq!(meta.name, Some("test-agent".to_string()));
    }

    #[tokio::test]
    async fn test_spawn_multiple() {
        let coord = DefaultCoordinator::new();

        let id1 = coord.spawn(async { 1 }).await.unwrap();
        let id2 = coord.spawn(async { 2 }).await.unwrap();
        let id3 = coord.spawn(async { 3 }).await.unwrap();

        assert_ne!(id1, id2);
        assert_ne!(id2, id3);

        let list = coord.list().await;
        assert_eq!(list.len(), 3);
    }

    #[tokio::test]
    async fn test_spawn_boxed() {
        let coord = DefaultCoordinator::new();

        let task: BoxedTask = Box::pin(async {});
        let id = coord.spawn_boxed(task).await.unwrap();

        sleep(Duration::from_millis(50)).await;

        let state = coord.state(id).await.unwrap();
        assert_eq!(state, AgentState::Completed);
    }

    // ==================== Cancel Tests ====================

    #[tokio::test]
    async fn test_cancel_running() {
        let coord = DefaultCoordinator::new();

        let id = coord
            .spawn(async {
                sleep(Duration::from_secs(10)).await;
            })
            .await
            .unwrap();

        // Cancel immediately
        coord.cancel(id).await.unwrap();

        // Give time for cancellation to propagate
        sleep(Duration::from_millis(50)).await;

        let state = coord.state(id).await.unwrap();
        assert_eq!(state, AgentState::Cancelled);
    }

    #[tokio::test]
    async fn test_cancel_not_found() {
        let coord = DefaultCoordinator::new();
        let fake_id = AgentId::new();

        let result = coord.cancel(fake_id).await;
        assert!(matches!(result, Err(AgentError::NotFound { .. })));
    }

    #[tokio::test]
    async fn test_cancel_all() {
        let coord = DefaultCoordinator::new();

        // Spawn multiple long-running tasks
        for _ in 0..5 {
            coord
                .spawn(async {
                    sleep(Duration::from_secs(10)).await;
                })
                .await
                .unwrap();
        }

        let cancelled = coord.cancel_all().await;
        assert_eq!(cancelled, 5);

        // Give time for cancellation
        sleep(Duration::from_millis(50)).await;

        let stats = coord.stats().await;
        assert_eq!(stats.cancelled, 5);
    }

    // ==================== State Tests ====================

    #[tokio::test]
    async fn test_state_running() {
        let coord = DefaultCoordinator::new();

        let id = coord
            .spawn(async {
                sleep(Duration::from_secs(10)).await;
            })
            .await
            .unwrap();

        let state = coord.state(id).await.unwrap();
        assert_eq!(state, AgentState::Running);

        // Cleanup
        coord.cancel(id).await.unwrap();
    }

    #[tokio::test]
    async fn test_state_completed() {
        let coord = DefaultCoordinator::new();

        let id = coord.spawn(async { 42 }).await.unwrap();

        // Wait for completion
        sleep(Duration::from_millis(50)).await;

        let state = coord.state(id).await.unwrap();
        assert_eq!(state, AgentState::Completed);
    }

    #[tokio::test]
    async fn test_state_not_found() {
        let coord = DefaultCoordinator::new();
        let fake_id = AgentId::new();

        let result = coord.state(fake_id).await;
        assert!(matches!(result, Err(AgentError::NotFound { .. })));
    }

    // ==================== Metadata Tests ====================

    #[tokio::test]
    async fn test_metadata_timestamps() {
        let coord = DefaultCoordinator::new();

        let id = coord.spawn(async { 42 }).await.unwrap();

        // Wait for completion
        sleep(Duration::from_millis(50)).await;

        let meta = coord.metadata(id).await.unwrap();
        assert!(meta.started_at.is_some());
        assert!(meta.finished_at.is_some());
        assert!(meta.started_at.unwrap() <= meta.finished_at.unwrap());
    }

    // ==================== List Tests ====================

    #[tokio::test]
    async fn test_list_empty() {
        let coord = DefaultCoordinator::new();
        let list = coord.list().await;
        assert!(list.is_empty());
    }

    #[tokio::test]
    async fn test_list_by_state() {
        let coord = DefaultCoordinator::new();

        // Spawn completed task
        let id1 = coord.spawn(async { 1 }).await.unwrap();

        // Spawn long-running task
        let id2 = coord
            .spawn(async {
                sleep(Duration::from_secs(10)).await;
            })
            .await
            .unwrap();

        // Wait for first to complete
        sleep(Duration::from_millis(50)).await;

        let running = coord.list_by_state(AgentState::Running).await;
        let completed = coord.list_by_state(AgentState::Completed).await;

        assert!(running.contains(&id2));
        assert!(completed.contains(&id1));

        // Cleanup
        coord.cancel(id2).await.unwrap();
    }

    // ==================== Wait Tests ====================

    #[tokio::test]
    async fn test_wait_completion() {
        let coord = DefaultCoordinator::new();

        let id = coord
            .spawn(async {
                sleep(Duration::from_millis(50)).await;
                42
            })
            .await
            .unwrap();

        let state = coord.wait(id, None).await.unwrap();
        assert_eq!(state, AgentState::Completed);
    }

    #[tokio::test]
    async fn test_wait_timeout() {
        let coord = DefaultCoordinator::new();

        let id = coord
            .spawn(async {
                sleep(Duration::from_secs(10)).await;
            })
            .await
            .unwrap();

        let result = coord.wait(id, Some(Duration::from_millis(50))).await;
        assert!(matches!(result, Err(AgentError::Timeout { .. })));
    }

    #[tokio::test]
    async fn test_wait_already_completed() {
        let coord = DefaultCoordinator::new();

        let id = coord.spawn(async { 42 }).await.unwrap();

        // Wait for natural completion
        sleep(Duration::from_millis(50)).await;

        // Wait should return immediately
        let state = coord.wait(id, None).await.unwrap();
        assert_eq!(state, AgentState::Completed);
    }

    // ==================== Stats Tests ====================

    #[tokio::test]
    async fn test_stats_empty() {
        let coord = DefaultCoordinator::new();
        let stats = coord.stats().await;

        assert_eq!(stats.total_created, 0);
        assert_eq!(stats.running, 0);
        assert_eq!(stats.completed, 0);
    }

    #[tokio::test]
    async fn test_stats_mixed() {
        let coord = DefaultCoordinator::new();

        // Spawn and complete
        coord.spawn(async { 1 }).await.unwrap();

        // Spawn long-running
        let id = coord
            .spawn(async {
                sleep(Duration::from_secs(10)).await;
            })
            .await
            .unwrap();

        // Wait for first to complete
        sleep(Duration::from_millis(50)).await;

        let stats = coord.stats().await;
        assert_eq!(stats.total_created, 2);
        assert_eq!(stats.completed, 1);
        assert_eq!(stats.running, 1);

        // Cleanup
        coord.cancel(id).await.unwrap();
    }

    // ==================== Cleanup Tests ====================

    #[tokio::test]
    async fn test_cleanup() {
        let coord = DefaultCoordinator::new();

        // Spawn and complete multiple tasks
        for _ in 0..5 {
            coord.spawn(async { 42 }).await.unwrap();
        }

        // Wait for completion
        sleep(Duration::from_millis(50)).await;

        let cleaned = coord.cleanup().await;
        assert_eq!(cleaned, 5);

        let list = coord.list().await;
        assert!(list.is_empty());
    }

    #[tokio::test]
    async fn test_cleanup_preserves_running() {
        let coord = DefaultCoordinator::new();

        // Spawn completed task
        coord.spawn(async { 1 }).await.unwrap();

        // Spawn long-running task
        let running_id = coord
            .spawn(async {
                sleep(Duration::from_secs(10)).await;
            })
            .await
            .unwrap();

        // Wait for first to complete
        sleep(Duration::from_millis(50)).await;

        let cleaned = coord.cleanup().await;
        assert_eq!(cleaned, 1);

        let list = coord.list().await;
        assert_eq!(list.len(), 1);
        assert!(list.contains(&running_id));

        // Cleanup
        coord.cancel(running_id).await.unwrap();
    }

    // ==================== Concurrent Execution Tests ====================

    #[tokio::test]
    async fn test_concurrent_execution() {
        let coord = Arc::new(DefaultCoordinator::new());
        let counter = Arc::new(AtomicUsize::new(0));

        // Spawn 10 concurrent tasks
        let mut ids = Vec::new();
        for _ in 0..10 {
            let c = counter.clone();
            let id = coord
                .spawn(async move {
                    c.fetch_add(1, Ordering::SeqCst);
                })
                .await
                .unwrap();
            ids.push(id);
        }

        // Wait for all to complete
        sleep(Duration::from_millis(100)).await;

        assert_eq!(counter.load(Ordering::SeqCst), 10);

        let stats = coord.stats().await;
        assert_eq!(stats.completed, 10);
    }

    // ==================== Trait Object Tests ====================

    #[tokio::test]
    async fn test_coordinator_as_trait_object() {
        let coord: CoordinatorHandle = Arc::new(DefaultCoordinator::new());

        let task: BoxedTask = Box::pin(async {});
        let id = coord.spawn_boxed(task).await.unwrap();

        sleep(Duration::from_millis(50)).await;

        let state = coord.state(id).await.unwrap();
        assert_eq!(state, AgentState::Completed);
    }
}
