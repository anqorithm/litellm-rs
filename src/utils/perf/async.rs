//! Async utilities for better concurrency patterns
//!
//! This module provides utilities to improve async code patterns,
//! reduce unnecessary spawning, and improve performance.

use crate::utils::error::error::{GatewayError, Result};
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
    _flush_interval: Duration,
}

impl BatchProcessor {
    /// Create a new batch processor
    pub fn new(batch_size: usize, flush_interval: Duration) -> Self {
        Self {
            batch_size,
            _flush_interval: flush_interval,
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
#[path = "async_tests.rs"]
mod tests;
