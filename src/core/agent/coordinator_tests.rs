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
