//! Circuit breaker implementation for fault tolerance

use super::types::{CircuitBreakerConfig, CircuitBreakerMetrics, CircuitState};
use crate::utils::error::gateway_error::{GatewayError, Result};
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Instant;
#[allow(unused_imports)]
use tracing::{debug, warn};

/// Circuit breaker implementation
pub struct CircuitBreaker {
    config: CircuitBreakerConfig,
    state: Arc<Mutex<CircuitState>>,
    failure_count: AtomicU32,
    success_count: AtomicU32,
    last_failure_time: Arc<Mutex<Option<Instant>>>,
    request_count: AtomicU32,
    window_start: Arc<Mutex<Instant>>,
}

impl CircuitBreaker {
    /// Create a new circuit breaker
    pub fn new(config: CircuitBreakerConfig) -> Self {
        Self {
            config,
            state: Arc::new(Mutex::new(CircuitState::Closed)),
            failure_count: AtomicU32::new(0),
            success_count: AtomicU32::new(0),
            last_failure_time: Arc::new(Mutex::new(None)),
            request_count: AtomicU32::new(0),
            window_start: Arc::new(Mutex::new(Instant::now())),
        }
    }

    /// Execute a function with circuit breaker protection
    pub async fn call<F, R, E>(&self, f: F) -> Result<R>
    where
        F: std::future::Future<Output = std::result::Result<R, E>>,
        E: std::fmt::Display + std::fmt::Debug,
    {
        // Check if circuit should allow the request
        if !self.can_execute().await {
            return Err(GatewayError::ProviderUnavailable(
                "Circuit breaker is open".to_string(),
            ));
        }

        self.request_count.fetch_add(1, Ordering::Relaxed);

        match f.await {
            Ok(result) => {
                self.on_success().await;
                Ok(result)
            }
            Err(error) => {
                self.on_failure().await;
                Err(GatewayError::External(format!(
                    "Circuit breaker protected call failed: {}",
                    error
                )))
            }
        }
    }

    /// Check if the circuit breaker allows execution
    async fn can_execute(&self) -> bool {
        // Use unwrap_or_else to handle poisoned mutex gracefully
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());

        match *state {
            CircuitState::Closed => true,
            CircuitState::Open => {
                // Check if timeout has passed
                if let Some(last_failure) = *self
                    .last_failure_time
                    .lock()
                    .unwrap_or_else(|poisoned| poisoned.into_inner())
                {
                    if last_failure.elapsed() >= self.config.timeout {
                        debug!("Circuit breaker transitioning from Open to HalfOpen");
                        *state = CircuitState::HalfOpen;
                        self.success_count.store(0, Ordering::Relaxed);
                        true
                    } else {
                        false
                    }
                } else {
                    false
                }
            }
            CircuitState::HalfOpen => true,
        }
    }

    /// Handle successful request
    async fn on_success(&self) {
        let success_count = self.success_count.fetch_add(1, Ordering::Relaxed) + 1;

        let mut state = self
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if *state == CircuitState::HalfOpen && success_count >= self.config.success_threshold {
            debug!("Circuit breaker transitioning from HalfOpen to Closed");
            *state = CircuitState::Closed;
            self.failure_count.store(0, Ordering::Relaxed);
            self.success_count.store(0, Ordering::Relaxed);
        }
    }

    /// Handle failed request
    async fn on_failure(&self) {
        let failure_count = self.failure_count.fetch_add(1, Ordering::Relaxed) + 1;
        let request_count = self.request_count.load(Ordering::Relaxed);

        *self
            .last_failure_time
            .lock()
            .unwrap_or_else(|p| p.into_inner()) = Some(Instant::now());

        let mut state = self
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());

        // Update window if needed
        {
            let mut window_start = self.window_start.lock().unwrap_or_else(|p| p.into_inner());
            if window_start.elapsed() >= self.config.window_size {
                *window_start = Instant::now();
                self.failure_count.store(1, Ordering::Relaxed);
                self.request_count.store(1, Ordering::Relaxed);
                return;
            }
        }

        // Check if we should open the circuit
        if request_count >= self.config.min_requests
            && failure_count >= self.config.failure_threshold
            && *state != CircuitState::Open
        {
            warn!(
                "Circuit breaker opening due to {} failures out of {} requests",
                failure_count, request_count
            );
            *state = CircuitState::Open;
        }

        // Always open from half-open on failure
        if *state == CircuitState::HalfOpen {
            debug!("Circuit breaker transitioning from HalfOpen to Open due to failure");
            *state = CircuitState::Open;
        }
    }

    /// Get current circuit breaker state
    pub fn state(&self) -> CircuitState {
        self.state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .clone()
    }

    /// Get current metrics
    pub fn metrics(&self) -> CircuitBreakerMetrics {
        CircuitBreakerMetrics {
            state: self.state(),
            failure_count: self.failure_count.load(Ordering::Relaxed),
            success_count: self.success_count.load(Ordering::Relaxed),
            request_count: self.request_count.load(Ordering::Relaxed),
        }
    }

    /// Reset the circuit breaker
    pub fn reset(&self) {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        *state = CircuitState::Closed;
        self.failure_count.store(0, Ordering::Relaxed);
        self.success_count.store(0, Ordering::Relaxed);
        self.request_count.store(0, Ordering::Relaxed);
        *self
            .last_failure_time
            .lock()
            .unwrap_or_else(|p| p.into_inner()) = None;
        *self.window_start.lock().unwrap_or_else(|p| p.into_inner()) = Instant::now();
        debug!("Circuit breaker reset");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    fn default_config() -> CircuitBreakerConfig {
        CircuitBreakerConfig {
            failure_threshold: 3,
            success_threshold: 2,
            min_requests: 5,
            timeout: Duration::from_millis(100),
            window_size: Duration::from_secs(60),
        }
    }

    // ==================== Creation Tests ====================

    #[test]
    fn test_circuit_breaker_new() {
        let config = default_config();
        let cb = CircuitBreaker::new(config);

        assert_eq!(cb.state(), CircuitState::Closed);
    }

    #[test]
    fn test_circuit_breaker_initial_state_is_closed() {
        let cb = CircuitBreaker::new(default_config());
        assert_eq!(cb.state(), CircuitState::Closed);
    }

    #[test]
    fn test_circuit_breaker_initial_metrics() {
        let cb = CircuitBreaker::new(default_config());
        let metrics = cb.metrics();

        assert_eq!(metrics.state, CircuitState::Closed);
        assert_eq!(metrics.failure_count, 0);
        assert_eq!(metrics.success_count, 0);
        assert_eq!(metrics.request_count, 0);
    }

    // ==================== State Tests ====================

    #[test]
    fn test_circuit_breaker_state_returns_current_state() {
        let cb = CircuitBreaker::new(default_config());
        let state = cb.state();
        assert_eq!(state, CircuitState::Closed);
    }

    // ==================== Metrics Tests ====================

    #[test]
    fn test_circuit_breaker_metrics_structure() {
        let cb = CircuitBreaker::new(default_config());
        let metrics = cb.metrics();

        // Verify metrics struct fields
        assert_eq!(metrics.state, CircuitState::Closed);
        assert_eq!(metrics.failure_count, 0);
        assert_eq!(metrics.success_count, 0);
        assert_eq!(metrics.request_count, 0);
    }

    // ==================== Reset Tests ====================

    #[test]
    fn test_circuit_breaker_reset() {
        let cb = CircuitBreaker::new(default_config());

        // Manually increment counters (accessing internal state)
        cb.failure_count.store(5, Ordering::Relaxed);
        cb.success_count.store(3, Ordering::Relaxed);
        cb.request_count.store(10, Ordering::Relaxed);

        // Reset
        cb.reset();

        let metrics = cb.metrics();
        assert_eq!(metrics.state, CircuitState::Closed);
        assert_eq!(metrics.failure_count, 0);
        assert_eq!(metrics.success_count, 0);
        assert_eq!(metrics.request_count, 0);
    }

    #[test]
    fn test_circuit_breaker_reset_clears_last_failure_time() {
        let cb = CircuitBreaker::new(default_config());

        // Set last failure time
        *cb.last_failure_time.lock().unwrap() = Some(Instant::now());

        // Reset
        cb.reset();

        assert!(cb.last_failure_time.lock().unwrap().is_none());
    }

    #[test]
    fn test_circuit_breaker_reset_resets_window_start() {
        let cb = CircuitBreaker::new(default_config());

        let before = *cb.window_start.lock().unwrap();
        std::thread::sleep(Duration::from_millis(10));

        cb.reset();

        let after = *cb.window_start.lock().unwrap();
        assert!(after > before);
    }

    // ==================== Config Tests ====================

    #[test]
    fn test_circuit_breaker_with_custom_config() {
        let config = CircuitBreakerConfig {
            failure_threshold: 10,
            success_threshold: 5,
            min_requests: 20,
            timeout: Duration::from_secs(120),
            window_size: Duration::from_secs(300),
        };

        let cb = CircuitBreaker::new(config);
        assert_eq!(cb.config.failure_threshold, 10);
        assert_eq!(cb.config.success_threshold, 5);
    }

    #[test]
    fn test_circuit_breaker_with_zero_threshold() {
        let config = CircuitBreakerConfig {
            failure_threshold: 0,
            success_threshold: 0,
            min_requests: 0,
            timeout: Duration::from_millis(1),
            window_size: Duration::from_millis(1),
        };

        let cb = CircuitBreaker::new(config);
        assert_eq!(cb.state(), CircuitState::Closed);
    }

    // ==================== Async Call Tests ====================

    #[tokio::test]
    async fn test_circuit_breaker_call_success() {
        let cb = CircuitBreaker::new(default_config());

        let result = cb.call(async { Ok::<_, String>("success") }).await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "success");
    }

    #[tokio::test]
    async fn test_circuit_breaker_call_increments_request_count() {
        let cb = CircuitBreaker::new(default_config());

        let _ = cb.call(async { Ok::<_, String>("test") }).await;

        let metrics = cb.metrics();
        assert_eq!(metrics.request_count, 1);
    }

    #[tokio::test]
    async fn test_circuit_breaker_call_increments_success_count() {
        let cb = CircuitBreaker::new(default_config());

        let _ = cb.call(async { Ok::<_, String>("test") }).await;

        let metrics = cb.metrics();
        assert_eq!(metrics.success_count, 1);
    }

    #[tokio::test]
    async fn test_circuit_breaker_call_failure() {
        let cb = CircuitBreaker::new(default_config());

        let result: Result<()> = cb.call(async { Err::<(), _>("failure") }).await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_circuit_breaker_call_failure_increments_failure_count() {
        let cb = CircuitBreaker::new(default_config());

        let _: Result<()> = cb.call(async { Err::<(), _>("failure") }).await;

        let metrics = cb.metrics();
        assert_eq!(metrics.failure_count, 1);
    }

    #[tokio::test]
    async fn test_circuit_breaker_multiple_successes() {
        let cb = CircuitBreaker::new(default_config());

        for _ in 0..5 {
            let _ = cb.call(async { Ok::<_, String>("success") }).await;
        }

        let metrics = cb.metrics();
        assert_eq!(metrics.request_count, 5);
        assert_eq!(metrics.success_count, 5);
        assert_eq!(metrics.failure_count, 0);
    }

    #[tokio::test]
    async fn test_circuit_breaker_mixed_results() {
        let cb = CircuitBreaker::new(default_config());

        // Two successes
        let _ = cb.call(async { Ok::<_, String>("success") }).await;
        let _ = cb.call(async { Ok::<_, String>("success") }).await;

        // One failure
        let _: Result<()> = cb.call(async { Err::<(), _>("fail") }).await;

        let metrics = cb.metrics();
        assert_eq!(metrics.request_count, 3);
        assert!(metrics.failure_count >= 1);
    }

    // ==================== State Transition Tests ====================

    #[tokio::test]
    async fn test_circuit_breaker_opens_after_threshold() {
        let config = CircuitBreakerConfig {
            failure_threshold: 3,
            success_threshold: 2,
            min_requests: 3, // Lower min_requests so circuit opens sooner
            timeout: Duration::from_millis(100),
            window_size: Duration::from_secs(60),
        };
        let cb = CircuitBreaker::new(config);

        // Cause enough failures to open circuit
        for _ in 0..5 {
            let _: Result<()> = cb.call(async { Err::<(), _>("fail") }).await;
        }

        assert_eq!(cb.state(), CircuitState::Open);
    }

    #[tokio::test]
    async fn test_circuit_breaker_open_rejects_requests() {
        let config = CircuitBreakerConfig {
            failure_threshold: 2,
            success_threshold: 2,
            min_requests: 2,
            timeout: Duration::from_secs(10), // Long timeout so it stays open
            window_size: Duration::from_secs(60),
        };
        let cb = CircuitBreaker::new(config);

        // Open the circuit
        for _ in 0..5 {
            let _: Result<()> = cb.call(async { Err::<(), _>("fail") }).await;
        }

        assert_eq!(cb.state(), CircuitState::Open);

        // Next request should be rejected
        let result: Result<()> = cb.call(async { Ok::<(), String>(()) }).await;
        assert!(result.is_err());

        if let Err(GatewayError::ProviderUnavailable(msg)) = result {
            assert!(msg.contains("Circuit breaker is open"));
        } else {
            panic!("Expected ProviderUnavailable error");
        }
    }

    #[tokio::test]
    async fn test_circuit_breaker_transitions_to_half_open() {
        let config = CircuitBreakerConfig {
            failure_threshold: 2,
            success_threshold: 2,
            min_requests: 2,
            timeout: Duration::from_millis(50), // Short timeout
            window_size: Duration::from_secs(60),
        };
        let cb = CircuitBreaker::new(config);

        // Open the circuit
        for _ in 0..3 {
            let _: Result<()> = cb.call(async { Err::<(), _>("fail") }).await;
        }

        assert_eq!(cb.state(), CircuitState::Open);

        // Wait for timeout
        tokio::time::sleep(Duration::from_millis(100)).await;

        // Next call should transition to half-open
        let _ = cb.call(async { Ok::<_, String>("success") }).await;

        // State should be HalfOpen or Closed depending on success
        let state = cb.state();
        assert!(state == CircuitState::HalfOpen || state == CircuitState::Closed);
    }

    #[tokio::test]
    async fn test_circuit_breaker_closes_after_success_threshold() {
        let config = CircuitBreakerConfig {
            failure_threshold: 2,
            success_threshold: 2,
            min_requests: 2,
            timeout: Duration::from_millis(10),
            window_size: Duration::from_secs(60),
        };
        let cb = CircuitBreaker::new(config);

        // Open the circuit
        for _ in 0..3 {
            let _: Result<()> = cb.call(async { Err::<(), _>("fail") }).await;
        }

        // Wait for timeout to transition to half-open
        tokio::time::sleep(Duration::from_millis(50)).await;

        // Successful calls in half-open state
        for _ in 0..3 {
            let _ = cb.call(async { Ok::<_, String>("success") }).await;
        }

        assert_eq!(cb.state(), CircuitState::Closed);
    }

    #[tokio::test]
    async fn test_circuit_breaker_half_open_failure_reopens() {
        let config = CircuitBreakerConfig {
            failure_threshold: 2,
            success_threshold: 2,
            min_requests: 2,
            timeout: Duration::from_millis(10),
            window_size: Duration::from_secs(60),
        };
        let cb = CircuitBreaker::new(config);

        // Open the circuit
        for _ in 0..3 {
            let _: Result<()> = cb.call(async { Err::<(), _>("fail") }).await;
        }
        assert_eq!(cb.state(), CircuitState::Open);

        // Wait for timeout
        tokio::time::sleep(Duration::from_millis(50)).await;

        // This will transition to half-open and then fail
        let _: Result<()> = cb.call(async { Err::<(), _>("fail again") }).await;

        assert_eq!(cb.state(), CircuitState::Open);
    }

    // ==================== Concurrent Access Tests ====================

    #[tokio::test]
    async fn test_circuit_breaker_concurrent_calls() {
        let cb = Arc::new(CircuitBreaker::new(default_config()));

        let mut handles = vec![];

        for _ in 0..10 {
            let cb_clone = cb.clone();
            let handle = tokio::spawn(async move {
                let _ = cb_clone.call(async { Ok::<_, String>("success") }).await;
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.await.unwrap();
        }

        let metrics = cb.metrics();
        assert_eq!(metrics.request_count, 10);
    }

    #[tokio::test]
    async fn test_circuit_breaker_concurrent_state_access() {
        let cb = Arc::new(CircuitBreaker::new(default_config()));

        let mut handles = vec![];

        for _ in 0..5 {
            let cb_clone = cb.clone();
            let handle = tokio::spawn(async move { cb_clone.state() });
            handles.push(handle);
        }

        for handle in handles {
            let state = handle.await.unwrap();
            assert_eq!(state, CircuitState::Closed);
        }
    }

    #[tokio::test]
    async fn test_circuit_breaker_concurrent_reset() {
        let cb = Arc::new(CircuitBreaker::new(default_config()));

        // Add some state
        cb.failure_count.store(5, Ordering::Relaxed);

        let mut handles = vec![];
        for _ in 0..3 {
            let cb_clone = cb.clone();
            let handle = tokio::spawn(async move {
                cb_clone.reset();
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.await.unwrap();
        }

        // State should be reset
        assert_eq!(cb.state(), CircuitState::Closed);
        assert_eq!(cb.metrics().failure_count, 0);
    }

    // ==================== Error Type Tests ====================

    #[tokio::test]
    async fn test_circuit_breaker_preserves_error_message() {
        let cb = CircuitBreaker::new(default_config());

        let result: Result<()> = cb
            .call(async { Err::<(), _>("specific error message") })
            .await;

        if let Err(GatewayError::External(msg)) = result {
            assert!(msg.contains("specific error message"));
        } else {
            panic!("Expected External error");
        }
    }

    #[tokio::test]
    async fn test_circuit_breaker_open_error_type() {
        let config = CircuitBreakerConfig {
            failure_threshold: 1,
            success_threshold: 1,
            min_requests: 1,
            timeout: Duration::from_secs(60),
            window_size: Duration::from_secs(60),
        };
        let cb = CircuitBreaker::new(config);

        // Open the circuit
        let _: Result<()> = cb.call(async { Err::<(), _>("fail") }).await;
        let _: Result<()> = cb.call(async { Err::<(), _>("fail") }).await;

        // Next request should fail with ProviderUnavailable
        let result: Result<()> = cb.call(async { Ok::<(), String>(()) }).await;

        assert!(matches!(result, Err(GatewayError::ProviderUnavailable(_))));
    }

    // ==================== Window Size Tests ====================

    #[tokio::test]
    async fn test_circuit_breaker_window_resets_after_expiry() {
        let config = CircuitBreakerConfig {
            failure_threshold: 5,
            success_threshold: 2,
            min_requests: 10,
            timeout: Duration::from_secs(60),
            window_size: Duration::from_millis(50), // Very short window
        };
        let cb = CircuitBreaker::new(config);

        // Add some failures
        for _ in 0..3 {
            let _: Result<()> = cb.call(async { Err::<(), _>("fail") }).await;
        }

        // Wait for window to expire
        tokio::time::sleep(Duration::from_millis(100)).await;

        // This failure should reset window
        let _: Result<()> = cb.call(async { Err::<(), _>("fail") }).await;

        // Failure count should be reset to 1
        let metrics = cb.metrics();
        assert_eq!(metrics.failure_count, 1);
    }

    // ==================== Edge Cases ====================

    #[test]
    fn test_circuit_breaker_debug() {
        let cb = CircuitBreaker::new(default_config());
        // Just verify it doesn't panic
        let _ = cb.state();
        let _ = cb.metrics();
    }

    #[tokio::test]
    async fn test_circuit_breaker_many_failures_then_recovery() {
        let config = CircuitBreakerConfig {
            failure_threshold: 2,
            success_threshold: 2,
            min_requests: 2,
            timeout: Duration::from_millis(10),
            window_size: Duration::from_secs(60),
        };
        let cb = CircuitBreaker::new(config);

        // Many failures
        for _ in 0..10 {
            let _: Result<()> = cb.call(async { Err::<(), _>("fail") }).await;
        }

        assert_eq!(cb.state(), CircuitState::Open);

        // Wait and recover
        tokio::time::sleep(Duration::from_millis(50)).await;

        for _ in 0..5 {
            let _ = cb.call(async { Ok::<_, String>("ok") }).await;
        }

        assert_eq!(cb.state(), CircuitState::Closed);
    }
}
