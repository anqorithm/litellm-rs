//! Resilience patterns for resource isolation and timeout protection

use crate::utils::error::gateway_error::{GatewayError, Result};
use std::sync::Arc;
use std::time::Duration;
use tracing::debug;

/// Timeout wrapper for async operations
#[allow(dead_code)]
pub struct TimeoutWrapper {
    timeout: Duration,
}

#[allow(dead_code)]
impl TimeoutWrapper {
    /// Create a new timeout wrapper
    pub fn new(timeout: Duration) -> Self {
        Self { timeout }
    }

    /// Execute a function with timeout protection
    pub async fn call<F, R>(&self, f: F) -> Result<R>
    where
        F: std::future::Future<Output = R>,
    {
        match tokio::time::timeout(self.timeout, f).await {
            Ok(result) => Ok(result),
            Err(_) => Err(GatewayError::Timeout(format!(
                "Operation timed out after {:?}",
                self.timeout
            ))),
        }
    }
}

/// Bulkhead pattern for resource isolation
#[allow(dead_code)]
pub struct Bulkhead {
    semaphore: Arc<tokio::sync::Semaphore>,
    name: String,
    max_concurrent: usize,
}

#[allow(dead_code)]
impl Bulkhead {
    /// Create a new bulkhead
    pub fn new(name: String, max_concurrent: usize) -> Self {
        Self {
            semaphore: Arc::new(tokio::sync::Semaphore::new(max_concurrent)),
            name,
            max_concurrent,
        }
    }

    /// Execute a function with bulkhead protection
    pub async fn call<F, R>(&self, f: F) -> Result<R>
    where
        F: std::future::Future<Output = Result<R>>,
    {
        let _permit = self
            .semaphore
            .acquire()
            .await
            .map_err(|e| GatewayError::Internal(format!("Bulkhead acquire failed: {}", e)))?;

        debug!("Bulkhead '{}' acquired permit", self.name);

        let result = f.await;

        debug!("Bulkhead '{}' released permit", self.name);

        result
    }

    /// Get available permits
    pub fn available_permits(&self) -> usize {
        self.semaphore.available_permits()
    }

    /// Get maximum concurrent operations
    pub fn max_concurrent(&self) -> usize {
        self.max_concurrent
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    // ==================== TimeoutWrapper Tests ====================

    #[tokio::test]
    async fn test_timeout_wrapper_success() {
        let wrapper = TimeoutWrapper::new(Duration::from_secs(1));
        let result = wrapper.call(async { 42 }).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 42);
    }

    #[tokio::test]
    async fn test_timeout_wrapper_timeout() {
        let wrapper = TimeoutWrapper::new(Duration::from_millis(50));
        let result = wrapper
            .call(async {
                tokio::time::sleep(Duration::from_millis(200)).await;
                42
            })
            .await;
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, GatewayError::Timeout(_)));
    }

    #[tokio::test]
    async fn test_timeout_wrapper_just_in_time() {
        let wrapper = TimeoutWrapper::new(Duration::from_millis(100));
        let result = wrapper
            .call(async {
                tokio::time::sleep(Duration::from_millis(10)).await;
                "success"
            })
            .await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "success");
    }

    #[tokio::test]
    async fn test_timeout_wrapper_very_short_duration() {
        let wrapper = TimeoutWrapper::new(Duration::from_nanos(1));
        let result = wrapper
            .call(async {
                tokio::time::sleep(Duration::from_millis(100)).await;
                42
            })
            .await;
        // Very short timeout should timeout for long operations
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_timeout_wrapper_error_message() {
        let wrapper = TimeoutWrapper::new(Duration::from_millis(10));
        let result: Result<i32> = wrapper
            .call(async {
                tokio::time::sleep(Duration::from_millis(100)).await;
                42
            })
            .await;

        if let Err(GatewayError::Timeout(msg)) = result {
            assert!(msg.contains("timed out"));
        } else {
            panic!("Expected Timeout error");
        }
    }

    // ==================== Bulkhead Tests ====================

    #[tokio::test]
    async fn test_bulkhead_creation() {
        let bulkhead = Bulkhead::new("test".to_string(), 5);
        assert_eq!(bulkhead.max_concurrent(), 5);
        assert_eq!(bulkhead.available_permits(), 5);
    }

    #[tokio::test]
    async fn test_bulkhead_single_call() {
        let bulkhead = Bulkhead::new("test".to_string(), 3);
        let result = bulkhead.call(async { Ok::<_, GatewayError>(42) }).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 42);
        assert_eq!(bulkhead.available_permits(), 3);
    }

    #[tokio::test]
    async fn test_bulkhead_permits_released() {
        let bulkhead = Arc::new(Bulkhead::new("test".to_string(), 2));

        // First call
        let b1 = bulkhead.clone();
        let result1 = b1.call(async { Ok::<_, GatewayError>("first") }).await;
        assert!(result1.is_ok());

        // Permits should be fully available after call completes
        assert_eq!(bulkhead.available_permits(), 2);
    }

    #[tokio::test]
    async fn test_bulkhead_concurrent_calls() {
        let bulkhead = Arc::new(Bulkhead::new("test".to_string(), 3));
        let counter = Arc::new(AtomicUsize::new(0));

        let mut handles = vec![];

        for _ in 0..3 {
            let b = bulkhead.clone();
            let c = counter.clone();
            handles.push(tokio::spawn(async move {
                b.call(async {
                    c.fetch_add(1, Ordering::SeqCst);
                    tokio::time::sleep(Duration::from_millis(10)).await;
                    Ok::<_, GatewayError>(())
                })
                .await
            }));
        }

        for handle in handles {
            assert!(handle.await.unwrap().is_ok());
        }

        assert_eq!(counter.load(Ordering::SeqCst), 3);
    }

    #[tokio::test]
    async fn test_bulkhead_error_propagation() {
        let bulkhead = Bulkhead::new("test".to_string(), 2);
        let result: Result<i32> = bulkhead
            .call(async { Err(GatewayError::Internal("test error".to_string())) })
            .await;

        assert!(result.is_err());
        if let Err(GatewayError::Internal(msg)) = result {
            assert_eq!(msg, "test error");
        } else {
            panic!("Expected Internal error");
        }
    }

    #[tokio::test]
    async fn test_bulkhead_name() {
        let bulkhead = Bulkhead::new("my-bulkhead".to_string(), 5);
        assert_eq!(bulkhead.name, "my-bulkhead");
    }

    #[tokio::test]
    async fn test_bulkhead_max_concurrent_one() {
        let bulkhead = Bulkhead::new("single".to_string(), 1);
        assert_eq!(bulkhead.max_concurrent(), 1);
        assert_eq!(bulkhead.available_permits(), 1);

        let result = bulkhead.call(async { Ok::<_, GatewayError>(123) }).await;
        assert!(result.is_ok());
        assert_eq!(bulkhead.available_permits(), 1);
    }
}
