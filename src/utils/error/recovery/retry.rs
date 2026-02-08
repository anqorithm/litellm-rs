//! Retry mechanism with exponential backoff

use super::types::RetryConfig;
use std::time::Duration;
use tracing::{debug, error};

/// Retry mechanism with exponential backoff
pub struct RetryPolicy {
    config: RetryConfig,
}

impl RetryPolicy {
    /// Create a new retry policy
    pub fn new(config: RetryConfig) -> Self {
        Self { config }
    }

    /// Execute a function with retry logic
    pub async fn call<F, Fut, R, E>(&self, mut f: F) -> std::result::Result<R, E>
    where
        F: FnMut() -> Fut,
        Fut: std::future::Future<Output = std::result::Result<R, E>>,
        E: std::fmt::Display + std::fmt::Debug,
    {
        let mut attempt = 0;
        let mut delay = self.config.base_delay;

        loop {
            attempt += 1;

            match f().await {
                Ok(result) => {
                    if attempt > 1 {
                        debug!("Retry succeeded on attempt {}", attempt);
                    }
                    return Ok(result);
                }
                Err(error) => {
                    if attempt >= self.config.max_attempts {
                        error!("Retry failed after {} attempts: {}", attempt, error);
                        return Err(error);
                    }

                    debug!(
                        "Attempt {} failed: {}, retrying in {:?}",
                        attempt, error, delay
                    );

                    // Sleep with optional jitter
                    let actual_delay = if self.config.jitter {
                        let jitter_factor = 0.1;
                        let jitter = delay.as_millis() as f64
                            * jitter_factor
                            * (rand::random::<f64>() - 0.5);
                        Duration::from_millis((delay.as_millis() as f64 + jitter) as u64)
                    } else {
                        delay
                    };

                    tokio::time::sleep(actual_delay).await;

                    // Calculate next delay with exponential backoff
                    delay = std::cmp::min(
                        Duration::from_millis(
                            (delay.as_millis() as f64 * self.config.backoff_multiplier) as u64,
                        ),
                        self.config.max_delay,
                    );
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use std::sync::atomic::{AtomicUsize, Ordering};

    fn default_config() -> RetryConfig {
        RetryConfig {
            max_attempts: 3,
            base_delay: Duration::from_millis(10),
            max_delay: Duration::from_millis(100),
            backoff_multiplier: 2.0,
            jitter: false,
        }
    }

    // ==================== RetryPolicy Creation Tests ====================

    #[test]
    fn test_retry_policy_creation() {
        let config = default_config();
        let policy = RetryPolicy::new(config.clone());
        assert_eq!(policy.config.max_attempts, 3);
        assert_eq!(policy.config.base_delay, Duration::from_millis(10));
    }

    // ==================== RetryPolicy Success Tests ====================

    #[tokio::test]
    async fn test_retry_success_first_attempt() {
        let policy = RetryPolicy::new(default_config());
        let counter = Arc::new(AtomicUsize::new(0));
        let c = counter.clone();

        let result: std::result::Result<i32, String> = policy
            .call(|| {
                let c = c.clone();
                async move {
                    c.fetch_add(1, Ordering::SeqCst);
                    Ok(42)
                }
            })
            .await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 42);
        assert_eq!(counter.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn test_retry_success_second_attempt() {
        let policy = RetryPolicy::new(default_config());
        let counter = Arc::new(AtomicUsize::new(0));
        let c = counter.clone();

        let result: std::result::Result<i32, String> = policy
            .call(|| {
                let c = c.clone();
                async move {
                    let attempt = c.fetch_add(1, Ordering::SeqCst) + 1;
                    if attempt < 2 {
                        Err("fail".to_string())
                    } else {
                        Ok(42)
                    }
                }
            })
            .await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 42);
        assert_eq!(counter.load(Ordering::SeqCst), 2);
    }

    #[tokio::test]
    async fn test_retry_success_third_attempt() {
        let policy = RetryPolicy::new(default_config());
        let counter = Arc::new(AtomicUsize::new(0));
        let c = counter.clone();

        let result: std::result::Result<i32, String> = policy
            .call(|| {
                let c = c.clone();
                async move {
                    let attempt = c.fetch_add(1, Ordering::SeqCst) + 1;
                    if attempt < 3 {
                        Err("fail".to_string())
                    } else {
                        Ok(42)
                    }
                }
            })
            .await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 42);
        assert_eq!(counter.load(Ordering::SeqCst), 3);
    }

    // ==================== RetryPolicy Failure Tests ====================

    #[tokio::test]
    async fn test_retry_all_attempts_fail() {
        let policy = RetryPolicy::new(default_config());
        let counter = Arc::new(AtomicUsize::new(0));
        let c = counter.clone();

        let result: std::result::Result<i32, String> = policy
            .call(|| {
                let c = c.clone();
                async move {
                    c.fetch_add(1, Ordering::SeqCst);
                    Err("always fail".to_string())
                }
            })
            .await;

        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "always fail");
        assert_eq!(counter.load(Ordering::SeqCst), 3); // max_attempts = 3
    }

    #[tokio::test]
    async fn test_retry_single_attempt() {
        let config = RetryConfig {
            max_attempts: 1,
            ..default_config()
        };
        let policy = RetryPolicy::new(config);
        let counter = Arc::new(AtomicUsize::new(0));
        let c = counter.clone();

        let result: std::result::Result<i32, String> = policy
            .call(|| {
                let c = c.clone();
                async move {
                    c.fetch_add(1, Ordering::SeqCst);
                    Err("fail".to_string())
                }
            })
            .await;

        assert!(result.is_err());
        assert_eq!(counter.load(Ordering::SeqCst), 1);
    }

    // ==================== RetryPolicy Backoff Tests ====================

    #[tokio::test]
    async fn test_retry_with_backoff() {
        let config = RetryConfig {
            max_attempts: 3,
            base_delay: Duration::from_millis(10),
            max_delay: Duration::from_millis(1000),
            backoff_multiplier: 2.0,
            jitter: false,
        };
        let policy = RetryPolicy::new(config);

        let start = std::time::Instant::now();

        let result: std::result::Result<i32, String> =
            policy.call(|| async { Err("fail".to_string()) }).await;

        let elapsed = start.elapsed();

        assert!(result.is_err());
        // Should have waited at least 10ms + 20ms = 30ms (base + backoff)
        assert!(elapsed >= Duration::from_millis(25));
    }

    #[tokio::test]
    async fn test_retry_max_delay_cap() {
        let config = RetryConfig {
            max_attempts: 5,
            base_delay: Duration::from_millis(50),
            max_delay: Duration::from_millis(60),
            backoff_multiplier: 10.0,
            jitter: false,
        };
        let policy = RetryPolicy::new(config);

        let start = std::time::Instant::now();

        let _: std::result::Result<i32, String> =
            policy.call(|| async { Err("fail".to_string()) }).await;

        let elapsed = start.elapsed();

        // With max_delay cap at 60ms, delays should be limited
        // Without cap: 50 + 500 + 5000 + 50000 = huge
        // With cap: 50 + 60 + 60 + 60 = 230ms
        assert!(elapsed < Duration::from_secs(1));
    }

    // ==================== RetryPolicy Jitter Tests ====================

    #[tokio::test]
    async fn test_retry_with_jitter() {
        let config = RetryConfig {
            max_attempts: 2,
            base_delay: Duration::from_millis(50),
            max_delay: Duration::from_millis(200),
            backoff_multiplier: 2.0,
            jitter: true,
        };
        let policy = RetryPolicy::new(config);

        let result: std::result::Result<i32, String> =
            policy.call(|| async { Err("fail".to_string()) }).await;

        assert!(result.is_err());
    }

    // ==================== RetryPolicy Config Tests ====================

    #[tokio::test]
    async fn test_retry_many_attempts() {
        let config = RetryConfig {
            max_attempts: 10,
            base_delay: Duration::from_millis(1),
            max_delay: Duration::from_millis(5),
            backoff_multiplier: 1.5,
            jitter: false,
        };
        let policy = RetryPolicy::new(config);
        let counter = Arc::new(AtomicUsize::new(0));
        let c = counter.clone();

        let result: std::result::Result<i32, String> = policy
            .call(|| {
                let c = c.clone();
                async move {
                    c.fetch_add(1, Ordering::SeqCst);
                    Err("fail".to_string())
                }
            })
            .await;

        assert!(result.is_err());
        assert_eq!(counter.load(Ordering::SeqCst), 10);
    }
}
