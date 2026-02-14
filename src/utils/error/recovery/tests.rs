//! Tests for error recovery and resilience utilities

#[cfg(test)]
use super::{
    circuit_breaker::CircuitBreaker,
    types::{CircuitBreakerConfig, CircuitState},
};

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
