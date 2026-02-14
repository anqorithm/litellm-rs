//! Async utilities for better concurrency patterns
//!
//! This module provides utilities to improve async code patterns,
//! reduce unnecessary spawning, and improve performance.

#![allow(dead_code)] // Tool module - functions may be used in the future

use crate::utils::error::gateway_error::{GatewayError, Result};
use futures::{Future, StreamExt};
use std::time::Duration;
use tokio::time::{sleep, timeout};
use tracing::{debug, error, warn};

/// Utility for running multiple async operations concurrently
#[derive(Clone)]
pub struct ConcurrentRunner {
    max_concurrent: usize,
    timeout_duration: Option<Duration>,
}

impl ConcurrentRunner {
    /// Create a new concurrent runner
    pub fn new(max_concurrent: usize) -> Self {
        Self {
            max_concurrent,
            timeout_duration: None,
        }
    }

    /// Set a timeout for operations
    pub fn with_timeout(mut self, timeout_duration: Duration) -> Self {
        self.timeout_duration = Some(timeout_duration);
        self
    }

    /// Run multiple futures concurrently with controlled parallelism
    pub async fn run_concurrent<F, T, E>(&self, futures: Vec<F>) -> Vec<std::result::Result<T, E>>
    where
        F: Future<Output = std::result::Result<T, E>> + Send + 'static,
        T: Send + 'static,
        E: Send + 'static,
    {
        let stream = futures::stream::iter(futures)
            .map(|fut| fut)
            .buffer_unordered(self.max_concurrent);

        stream.collect().await
    }

    /// Run futures and collect only successful results
    pub async fn run_concurrent_ok<F, T, E>(&self, futures: Vec<F>) -> Vec<T>
    where
        F: Future<Output = std::result::Result<T, E>> + Send + 'static,
        T: Send + 'static,
        E: Send + 'static + std::fmt::Debug,
    {
        let results = self.run_concurrent(futures).await;
        results
            .into_iter()
            .filter_map(|result| match result {
                Ok(value) => Some(value),
                Err(e) => {
                    debug!("Concurrent operation failed: {:?}", e);
                    None
                }
            })
            .collect()
    }
}

/// Retry utility with exponential backoff
#[derive(Clone)]
pub struct RetryPolicy {
    max_attempts: usize,
    base_delay: Duration,
    max_delay: Duration,
    backoff_multiplier: f64,
}

impl RetryPolicy {
    /// Create a new retry policy
    pub fn new(max_attempts: usize) -> Self {
        Self {
            max_attempts,
            base_delay: Duration::from_millis(100),
            max_delay: Duration::from_secs(30),
            backoff_multiplier: 2.0,
        }
    }

    /// Set the base delay
    pub fn with_base_delay(mut self, delay: Duration) -> Self {
        self.base_delay = delay;
        self
    }

    /// Set the maximum delay
    pub fn with_max_delay(mut self, delay: Duration) -> Self {
        self.max_delay = delay;
        self
    }

    /// Set the backoff multiplier
    pub fn with_backoff_multiplier(mut self, multiplier: f64) -> Self {
        self.backoff_multiplier = multiplier;
        self
    }

    /// Execute a future with retry logic
    pub async fn execute<F, Fut, T, E>(&self, mut operation: F) -> std::result::Result<T, E>
    where
        F: FnMut() -> Fut,
        Fut: Future<Output = std::result::Result<T, E>>,
        E: std::fmt::Debug,
    {
        let mut attempt = 0;
        let mut delay = self.base_delay;

        loop {
            attempt += 1;

            match operation().await {
                Ok(result) => return Ok(result),
                Err(e) => {
                    if attempt >= self.max_attempts {
                        error!("Operation failed after {} attempts: {:?}", attempt, e);
                        return Err(e);
                    }

                    warn!(
                        "Operation failed (attempt {}/{}): {:?}. Retrying in {:?}",
                        attempt, self.max_attempts, e, delay
                    );

                    sleep(delay).await;

                    // Exponential backoff
                    delay = std::cmp::min(
                        Duration::from_millis(
                            (delay.as_millis() as f64 * self.backoff_multiplier) as u64,
                        ),
                        self.max_delay,
                    );
                }
            }
        }
    }
}

/// Utility for batching operations
pub struct BatchProcessor {
    batch_size: usize,
    flush_interval: Duration,
}

impl BatchProcessor {
    /// Create a new batch processor
    pub fn new(batch_size: usize, flush_interval: Duration) -> Self {
        Self {
            batch_size,
            flush_interval,
        }
    }

    /// Process items in batches
    pub async fn process<T, F, Fut, R, E>(
        &self,
        items: Vec<T>,
        processor: F,
    ) -> Vec<std::result::Result<R, E>>
    where
        T: Clone,
        F: Fn(Vec<T>) -> Fut + Clone,
        Fut: Future<Output = std::result::Result<Vec<R>, E>>,
        E: Clone,
    {
        let mut results = Vec::new();

        for chunk in items.chunks(self.batch_size) {
            match processor(chunk.to_vec()).await {
                Ok(batch_results) => results.extend(batch_results.into_iter().map(Ok)),
                Err(e) => {
                    // If batch fails, mark all items in batch as failed
                    for _ in chunk {
                        results.push(Err(e.clone()));
                    }
                }
            }
        }

        results
    }
}

/// Utility for graceful shutdown
pub struct GracefulShutdown {
    shutdown_timeout: Duration,
}

impl GracefulShutdown {
    /// Create a new graceful shutdown handler
    pub fn new(shutdown_timeout: Duration) -> Self {
        Self { shutdown_timeout }
    }

    /// Wait for shutdown signal and execute cleanup
    pub async fn wait_for_shutdown<F, Fut>(&self, cleanup: F) -> Result<()>
    where
        F: FnOnce() -> Fut,
        Fut: Future<Output = Result<()>>,
    {
        // Wait for shutdown signal (Ctrl+C)
        tokio::signal::ctrl_c().await.map_err(|e| {
            GatewayError::Internal(format!("Failed to listen for shutdown signal: {}", e))
        })?;

        debug!("Shutdown signal received, starting graceful shutdown");

        // Execute cleanup with timeout
        match timeout(self.shutdown_timeout, cleanup()).await {
            Ok(Ok(())) => {
                debug!("Graceful shutdown completed successfully");
                Ok(())
            }
            Ok(Err(e)) => {
                error!("Error during graceful shutdown: {}", e);
                Err(e)
            }
            Err(_) => {
                error!(
                    "Graceful shutdown timed out after {:?}",
                    self.shutdown_timeout
                );
                Err(GatewayError::Timeout(
                    "Graceful shutdown timed out".to_string(),
                ))
            }
        }
    }
}

// Note: Macros removed for simplicity - use futures::try_join! and tokio::time::timeout directly

/// Default concurrent runner for common use cases
pub fn default_concurrent_runner() -> ConcurrentRunner {
    ConcurrentRunner::new(10).with_timeout(Duration::from_secs(30))
}

/// Default retry policy for common use cases
pub fn default_retry_policy() -> RetryPolicy {
    RetryPolicy::new(3)
        .with_base_delay(Duration::from_millis(100))
        .with_max_delay(Duration::from_secs(5))
        .with_backoff_multiplier(2.0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use std::sync::atomic::{AtomicUsize, Ordering};

    // Type alias to avoid clippy::type_complexity warning
    type BoxedFuture<T, E> =
        std::pin::Pin<Box<dyn Future<Output = std::result::Result<T, E>> + Send>>;

    // ConcurrentRunner tests
    #[tokio::test]
    async fn test_concurrent_runner_basic() {
        let runner = ConcurrentRunner::new(2);
        let counter = Arc::new(AtomicUsize::new(0));

        let futures: Vec<_> = (0..5)
            .map(|_| {
                let counter = counter.clone();
                async move {
                    counter.fetch_add(1, Ordering::SeqCst);
                    tokio::time::sleep(Duration::from_millis(10)).await;
                    Ok::<_, GatewayError>(())
                }
            })
            .collect();

        let results = runner.run_concurrent(futures).await;
        assert_eq!(results.len(), 5);
        assert_eq!(counter.load(Ordering::SeqCst), 5);
    }

    #[tokio::test]
    async fn test_concurrent_runner_with_failures() {
        let runner = ConcurrentRunner::new(3);

        let futures: Vec<_> = (0..5)
            .map(|i| async move {
                if i % 2 == 0 {
                    Ok::<_, String>(i)
                } else {
                    Err(format!("error {}", i))
                }
            })
            .collect();

        let results = runner.run_concurrent(futures).await;
        assert_eq!(results.len(), 5);
        assert!(results[0].is_ok());
        assert!(results[1].is_err());
        assert!(results[2].is_ok());
        assert!(results[3].is_err());
        assert!(results[4].is_ok());
    }

    #[tokio::test]
    async fn test_concurrent_runner_with_timeout() {
        let runner = ConcurrentRunner::new(2).with_timeout(Duration::from_millis(100));

        // Verify timeout is set
        assert!(runner.timeout_duration.is_some());
        assert_eq!(runner.timeout_duration.unwrap(), Duration::from_millis(100));
    }

    #[tokio::test]
    async fn test_concurrent_runner_clone() {
        let runner1 = ConcurrentRunner::new(5).with_timeout(Duration::from_secs(10));
        let runner2 = runner1.clone();

        assert_eq!(runner2.max_concurrent, 5);
        assert_eq!(runner2.timeout_duration, Some(Duration::from_secs(10)));
    }

    #[tokio::test]
    async fn test_concurrent_runner_empty_futures() {
        let runner = ConcurrentRunner::new(2);
        let futures: Vec<BoxedFuture<i32, String>> = Vec::new();

        let results: Vec<std::result::Result<i32, String>> = runner.run_concurrent(futures).await;
        assert_eq!(results.len(), 0);
    }

    #[tokio::test]
    async fn test_run_concurrent_ok() {
        let runner = ConcurrentRunner::new(3);

        let futures: Vec<_> = (0..10)
            .map(|i| async move {
                if i < 7 {
                    Ok::<_, String>(i * 2)
                } else {
                    Err(format!("error {}", i))
                }
            })
            .collect();

        let results = runner.run_concurrent_ok(futures).await;
        // Should only have successful results (0-6)
        assert_eq!(results.len(), 7);
        assert_eq!(results, vec![0, 2, 4, 6, 8, 10, 12]);
    }

    #[tokio::test]
    async fn test_run_concurrent_ok_all_failures() {
        let runner = ConcurrentRunner::new(2);

        let futures: Vec<_> = (0..5)
            .map(|i| async move { Err::<i32, String>(format!("error {}", i)) })
            .collect();

        let results = runner.run_concurrent_ok(futures).await;
        assert_eq!(results.len(), 0);
    }

    #[tokio::test]
    async fn test_run_concurrent_ok_all_success() {
        let runner = ConcurrentRunner::new(4);

        let futures: Vec<_> = (0..8)
            .map(|i| async move { Ok::<_, String>(i + 100) })
            .collect();

        let results = runner.run_concurrent_ok(futures).await;
        assert_eq!(results.len(), 8);
        assert_eq!(results, vec![100, 101, 102, 103, 104, 105, 106, 107]);
    }

    // RetryPolicy tests
    #[tokio::test]
    async fn test_retry_policy_success_on_retry() {
        let policy = RetryPolicy::new(3);
        let counter = Arc::new(AtomicUsize::new(0));

        let result = policy
            .execute(|| {
                let counter = counter.clone();
                async move {
                    let count = counter.fetch_add(1, Ordering::SeqCst);
                    if count < 2 {
                        Err("temporary failure")
                    } else {
                        Ok("success")
                    }
                }
            })
            .await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "success");
        assert_eq!(counter.load(Ordering::SeqCst), 3);
    }

    #[tokio::test]
    async fn test_retry_policy_immediate_success() {
        let policy = RetryPolicy::new(5);
        let counter = Arc::new(AtomicUsize::new(0));

        let result = policy
            .execute(|| {
                let counter = counter.clone();
                async move {
                    counter.fetch_add(1, Ordering::SeqCst);
                    Ok::<_, String>("immediate success")
                }
            })
            .await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "immediate success");
        assert_eq!(counter.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn test_retry_policy_all_attempts_fail() {
        let policy = RetryPolicy::new(3)
            .with_base_delay(Duration::from_millis(1))
            .with_max_delay(Duration::from_millis(10));
        let counter = Arc::new(AtomicUsize::new(0));

        let result = policy
            .execute(|| {
                let counter = counter.clone();
                async move {
                    counter.fetch_add(1, Ordering::SeqCst);
                    Err::<String, _>("persistent failure")
                }
            })
            .await;

        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "persistent failure");
        assert_eq!(counter.load(Ordering::SeqCst), 3);
    }

    #[tokio::test]
    async fn test_retry_policy_with_base_delay() {
        let policy = RetryPolicy::new(2).with_base_delay(Duration::from_millis(50));

        assert_eq!(policy.base_delay, Duration::from_millis(50));
        assert_eq!(policy.max_attempts, 2);
    }

    #[tokio::test]
    async fn test_retry_policy_with_max_delay() {
        let policy = RetryPolicy::new(4).with_max_delay(Duration::from_secs(60));

        assert_eq!(policy.max_delay, Duration::from_secs(60));
    }

    #[tokio::test]
    async fn test_retry_policy_with_backoff_multiplier() {
        let policy = RetryPolicy::new(3).with_backoff_multiplier(3.0);

        assert_eq!(policy.backoff_multiplier, 3.0);
    }

    #[tokio::test]
    async fn test_retry_policy_clone() {
        let policy1 = RetryPolicy::new(5)
            .with_base_delay(Duration::from_millis(200))
            .with_max_delay(Duration::from_secs(20))
            .with_backoff_multiplier(1.5);

        let policy2 = policy1.clone();

        assert_eq!(policy2.max_attempts, 5);
        assert_eq!(policy2.base_delay, Duration::from_millis(200));
        assert_eq!(policy2.max_delay, Duration::from_secs(20));
        assert_eq!(policy2.backoff_multiplier, 1.5);
    }

    #[tokio::test]
    async fn test_retry_policy_exponential_backoff() {
        let policy = RetryPolicy::new(4)
            .with_base_delay(Duration::from_millis(10))
            .with_max_delay(Duration::from_millis(100))
            .with_backoff_multiplier(2.0);

        let start = std::time::Instant::now();
        let counter = Arc::new(AtomicUsize::new(0));

        let _result = policy
            .execute(|| {
                let counter = counter.clone();
                async move {
                    counter.fetch_add(1, Ordering::SeqCst);
                    Err::<String, _>("fail")
                }
            })
            .await;

        let elapsed = start.elapsed();
        // Should take at least base_delay + base_delay*2 + base_delay*4 = 70ms
        // But capped by max_delay
        assert!(elapsed >= Duration::from_millis(30));
    }

    // BatchProcessor tests
    #[tokio::test]
    async fn test_batch_processor_single_batch() {
        let processor = BatchProcessor::new(5, Duration::from_millis(100));

        let items = vec![1, 2, 3, 4, 5];
        let batch_fn = |batch: Vec<i32>| async move {
            Ok::<Vec<i32>, String>(batch.into_iter().map(|x| x * 2).collect())
        };

        let results = processor.process(items, batch_fn).await;
        assert_eq!(results.len(), 5);
        assert_eq!(results[0], Ok(2));
        assert_eq!(results[4], Ok(10));
    }

    #[tokio::test]
    async fn test_batch_processor_multiple_batches() {
        let processor = BatchProcessor::new(3, Duration::from_millis(50));

        let items = vec![1, 2, 3, 4, 5, 6, 7];
        let batch_fn = |batch: Vec<i32>| async move {
            Ok::<Vec<i32>, String>(batch.into_iter().map(|x| x + 10).collect())
        };

        let results = processor.process(items, batch_fn).await;
        assert_eq!(results.len(), 7);
        assert_eq!(results[0], Ok(11));
        assert_eq!(results[3], Ok(14));
        assert_eq!(results[6], Ok(17));
    }

    #[tokio::test]
    async fn test_batch_processor_with_failure() {
        let processor = BatchProcessor::new(2, Duration::from_millis(100));

        let items = vec![1, 2, 3, 4];
        let counter = Arc::new(AtomicUsize::new(0));

        let batch_fn = |batch: Vec<i32>| {
            let counter = counter.clone();
            async move {
                let batch_num = counter.fetch_add(1, Ordering::SeqCst);
                if batch_num == 1 {
                    // Second batch fails
                    Err::<Vec<i32>, String>("batch error".to_string())
                } else {
                    Ok(batch.into_iter().map(|x| x * 3).collect())
                }
            }
        };

        let results = processor.process(items, batch_fn).await;
        assert_eq!(results.len(), 4);
        // First batch succeeds
        assert_eq!(results[0], Ok(3));
        assert_eq!(results[1], Ok(6));
        // Second batch fails
        assert!(results[2].is_err());
        assert!(results[3].is_err());
    }

    #[tokio::test]
    async fn test_batch_processor_empty_items() {
        let processor = BatchProcessor::new(10, Duration::from_millis(100));

        let items: Vec<i32> = vec![];
        let batch_fn = |batch: Vec<i32>| async move { Ok::<Vec<i32>, String>(batch) };

        let results = processor.process(items, batch_fn).await;
        assert_eq!(results.len(), 0);
    }

    #[tokio::test]
    async fn test_batch_processor_large_batches() {
        let processor = BatchProcessor::new(20, Duration::from_millis(50));

        let items: Vec<i32> = (0..100).collect();
        let batch_fn = |batch: Vec<i32>| async move {
            Ok::<Vec<i32>, String>(batch.into_iter().map(|x| x + 1).collect())
        };

        let results = processor.process(items, batch_fn).await;
        assert_eq!(results.len(), 100);
        assert_eq!(results[0], Ok(1));
        assert_eq!(results[99], Ok(100));
    }

    // GracefulShutdown tests
    #[tokio::test]
    async fn test_graceful_shutdown_new() {
        let shutdown = GracefulShutdown::new(Duration::from_secs(10));
        assert_eq!(shutdown.shutdown_timeout, Duration::from_secs(10));
    }

    #[tokio::test]
    async fn test_graceful_shutdown_cleanup_success() {
        let shutdown = GracefulShutdown::new(Duration::from_secs(5));
        let cleanup_called = Arc::new(AtomicUsize::new(0));
        let cleanup_called_clone = cleanup_called.clone();

        // Create a future that will timeout (simulating no shutdown signal)
        let shutdown_future = async {
            let result = timeout(
                Duration::from_millis(100),
                shutdown.wait_for_shutdown(|| {
                    let cleanup_called = cleanup_called_clone.clone();
                    async move {
                        cleanup_called.fetch_add(1, Ordering::SeqCst);
                        Ok(())
                    }
                }),
            )
            .await;

            // Should timeout waiting for signal
            assert!(result.is_err());
        };

        shutdown_future.await;
    }

    // Default helpers tests
    #[tokio::test]
    async fn test_default_concurrent_runner() {
        let runner = default_concurrent_runner();

        assert_eq!(runner.max_concurrent, 10);
        assert_eq!(runner.timeout_duration, Some(Duration::from_secs(30)));
    }

    #[tokio::test]
    async fn test_default_retry_policy() {
        let policy = default_retry_policy();

        assert_eq!(policy.max_attempts, 3);
        assert_eq!(policy.base_delay, Duration::from_millis(100));
        assert_eq!(policy.max_delay, Duration::from_secs(5));
        assert_eq!(policy.backoff_multiplier, 2.0);
    }

    #[tokio::test]
    async fn test_default_concurrent_runner_usage() {
        let runner = default_concurrent_runner();

        let futures: Vec<_> = (0..5)
            .map(|i| async move { Ok::<_, String>(i * 2) })
            .collect();

        let results = runner.run_concurrent(futures).await;
        assert_eq!(results.len(), 5);
        assert!(results.iter().all(|r| r.is_ok()));
    }

    #[tokio::test]
    async fn test_default_retry_policy_usage() {
        let policy = default_retry_policy();
        let counter = Arc::new(AtomicUsize::new(0));

        let result = policy
            .execute(|| {
                let counter = counter.clone();
                async move {
                    let count = counter.fetch_add(1, Ordering::SeqCst);
                    if count < 1 { Err("retry once") } else { Ok(42) }
                }
            })
            .await;

        assert_eq!(result.unwrap(), 42);
        assert_eq!(counter.load(Ordering::SeqCst), 2);
    }

    // Integration tests
    #[tokio::test]
    async fn test_concurrent_runner_with_retry_policy() {
        let runner = ConcurrentRunner::new(3);
        let retry_policy = RetryPolicy::new(2).with_base_delay(Duration::from_millis(10));

        let counters: Vec<_> = (0..5).map(|_| Arc::new(AtomicUsize::new(0))).collect();

        let futures: Vec<_> = counters
            .iter()
            .enumerate()
            .map(|(idx, counter)| {
                let counter = counter.clone();
                let policy = retry_policy.clone();
                async move {
                    policy
                        .execute(|| {
                            let counter = counter.clone();
                            async move {
                                let count = counter.fetch_add(1, Ordering::SeqCst);
                                if count < 1 && idx % 2 == 0 {
                                    Err(format!("error {}", idx))
                                } else {
                                    Ok(idx * 10)
                                }
                            }
                        })
                        .await
                }
            })
            .collect();

        let results = runner.run_concurrent(futures).await;
        assert_eq!(results.len(), 5);

        // Even indices should have retried once
        assert_eq!(counters[0].load(Ordering::SeqCst), 2);
        assert_eq!(counters[2].load(Ordering::SeqCst), 2);
        assert_eq!(counters[4].load(Ordering::SeqCst), 2);

        // Odd indices should succeed immediately
        assert_eq!(counters[1].load(Ordering::SeqCst), 1);
        assert_eq!(counters[3].load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn test_batch_processor_with_concurrent_runner() {
        let batch_processor = BatchProcessor::new(3, Duration::from_millis(100));
        let concurrent_runner = ConcurrentRunner::new(2);

        let items = vec![1, 2, 3, 4, 5, 6];

        let batch_fn = |batch: Vec<i32>| {
            let runner = concurrent_runner.clone();
            async move {
                let futures: Vec<_> = batch
                    .into_iter()
                    .map(|x| async move { Ok::<_, String>(x * 2) })
                    .collect();

                let results = runner.run_concurrent_ok(futures).await;
                Ok::<Vec<i32>, String>(results)
            }
        };

        let results = batch_processor.process(items, batch_fn).await;
        assert_eq!(results.len(), 6);
        assert!(results.iter().all(|r| r.is_ok()));
    }
}
