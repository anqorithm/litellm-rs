//! Tests for error recovery and resilience utilities

#[cfg(test)]
use super::{
    circuit_breaker::CircuitBreaker,
    resilience::{Bulkhead, TimeoutWrapper},
    retry::RetryPolicy,
    types::{CircuitBreakerConfig, CircuitState, RetryConfig},
};
use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};
use std::time::Duration;

#[tokio::test]
async fn test_circuit_breaker_success() {
    let config = CircuitBreakerConfig::default();
    let breaker = CircuitBreaker::new(config);

    let result = breaker.call(async { Ok::<i32, &str>(42) }).await;
    assert!(result.is_ok());
    assert_eq!(breaker.state(), CircuitState::Closed);
}

#[tokio::test]
async fn test_circuit_breaker_failure() {
    let config = CircuitBreakerConfig {
        failure_threshold: 2,
        min_requests: 2,
        ..Default::default()
    };

    let breaker = CircuitBreaker::new(config);

    // First failure
    let _ = breaker.call(async { Err::<i32, &str>("error") }).await;
    assert_eq!(breaker.state(), CircuitState::Closed);

    // Second failure should open circuit
    let _ = breaker.call(async { Err::<i32, &str>("error") }).await;
    assert_eq!(breaker.state(), CircuitState::Open);

    // Next call should be rejected
    let result = breaker.call(async { Ok::<i32, &str>(42) }).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_retry_policy() {
    let config = RetryConfig {
        max_attempts: 3,
        base_delay: Duration::from_millis(1),
        ..Default::default()
    };
    let policy = RetryPolicy::new(config);

    let counter = Arc::new(AtomicU32::new(0));
    let counter_clone = counter.clone();

    let result = policy
        .call(|| {
            let counter = counter_clone.clone();
            async move {
                let count = counter.fetch_add(1, Ordering::Relaxed);
                if count < 2 { Err("not yet") } else { Ok(42) }
            }
        })
        .await;

    assert_eq!(result, Ok(42));
    assert_eq!(counter.load(Ordering::Relaxed), 3);
}

#[tokio::test]
async fn test_timeout_wrapper() {
    let wrapper = TimeoutWrapper::new(Duration::from_millis(10));

    // Fast operation should succeed
    let result = wrapper.call(async { 42 }).await;
    assert!(result.is_ok());

    // Slow operation should timeout
    let result = wrapper
        .call(async {
            tokio::time::sleep(Duration::from_millis(20)).await;
            42
        })
        .await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_bulkhead() {
    let bulkhead = Bulkhead::new("test".to_string(), 2);

    assert_eq!(bulkhead.available_permits(), 2);

    let result = bulkhead.call(async { Ok(42) }).await;
    assert!(result.is_ok());

    assert_eq!(bulkhead.available_permits(), 2); // Permit should be released
}
