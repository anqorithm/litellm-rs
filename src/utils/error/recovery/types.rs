//! Types and configurations for error recovery patterns

use std::time::Duration;

/// Circuit breaker state
#[derive(Debug, Clone, PartialEq)]
pub enum CircuitState {
    /// Circuit is closed, requests flow normally
    Closed,
    /// Circuit is open, requests are rejected
    Open,
    /// Circuit is half-open, allowing test requests
    HalfOpen,
}

/// Circuit breaker configuration
#[derive(Debug, Clone)]
pub struct CircuitBreakerConfig {
    /// Failure threshold to open circuit
    pub failure_threshold: u32,
    /// Success threshold to close circuit from half-open
    pub success_threshold: u32,
    /// Minimum requests before considering failure rate
    pub min_requests: u32,
    /// Timeout before transitioning from open to half-open
    pub timeout: Duration,
    /// Window size for failure rate calculation
    pub window_size: Duration,
}

impl Default for CircuitBreakerConfig {
    fn default() -> Self {
        Self {
            failure_threshold: 5,
            success_threshold: 3,
            min_requests: 10,
            timeout: Duration::from_secs(60),
            window_size: Duration::from_secs(60),
        }
    }
}

/// Circuit breaker metrics
#[derive(Debug, Clone)]
pub struct CircuitBreakerMetrics {
    /// Current circuit breaker state
    pub state: CircuitState,
    /// Number of consecutive failures
    pub failure_count: u32,
    /// Number of consecutive successes
    pub success_count: u32,
    /// Total number of requests processed
    pub request_count: u32,
}

#[cfg(test)]
mod tests {
    use super::*;

    // ==================== CircuitState Tests ====================

    #[test]
    fn test_circuit_state_closed() {
        let state = CircuitState::Closed;
        assert_eq!(state, CircuitState::Closed);
    }

    #[test]
    fn test_circuit_state_open() {
        let state = CircuitState::Open;
        assert_eq!(state, CircuitState::Open);
    }

    #[test]
    fn test_circuit_state_half_open() {
        let state = CircuitState::HalfOpen;
        assert_eq!(state, CircuitState::HalfOpen);
    }

    #[test]
    fn test_circuit_state_clone() {
        let state = CircuitState::HalfOpen;
        let cloned = state.clone();
        assert_eq!(state, cloned);
    }

    // ==================== CircuitBreakerConfig Default Tests ====================

    #[test]
    fn test_circuit_breaker_config_default() {
        let config = CircuitBreakerConfig::default();
        assert_eq!(config.failure_threshold, 5);
        assert_eq!(config.success_threshold, 3);
        assert_eq!(config.min_requests, 10);
        assert_eq!(config.timeout, Duration::from_secs(60));
        assert_eq!(config.window_size, Duration::from_secs(60));
    }

    #[test]
    fn test_circuit_breaker_config_custom() {
        let config = CircuitBreakerConfig {
            failure_threshold: 10,
            success_threshold: 5,
            min_requests: 20,
            timeout: Duration::from_secs(120),
            window_size: Duration::from_secs(300),
        };
        assert_eq!(config.failure_threshold, 10);
        assert_eq!(config.success_threshold, 5);
        assert_eq!(config.min_requests, 20);
    }

    #[test]
    fn test_circuit_breaker_config_clone() {
        let config = CircuitBreakerConfig::default();
        let cloned = config.clone();
        assert_eq!(config.failure_threshold, cloned.failure_threshold);
        assert_eq!(config.timeout, cloned.timeout);
    }

    // ==================== CircuitBreakerMetrics Tests ====================

    #[test]
    fn test_circuit_breaker_metrics_closed() {
        let metrics = CircuitBreakerMetrics {
            state: CircuitState::Closed,
            failure_count: 0,
            success_count: 10,
            request_count: 100,
        };
        assert_eq!(metrics.state, CircuitState::Closed);
        assert_eq!(metrics.failure_count, 0);
        assert_eq!(metrics.request_count, 100);
    }

    #[test]
    fn test_circuit_breaker_metrics_open() {
        let metrics = CircuitBreakerMetrics {
            state: CircuitState::Open,
            failure_count: 5,
            success_count: 0,
            request_count: 50,
        };
        assert_eq!(metrics.state, CircuitState::Open);
        assert_eq!(metrics.failure_count, 5);
    }

    #[test]
    fn test_circuit_breaker_metrics_half_open() {
        let metrics = CircuitBreakerMetrics {
            state: CircuitState::HalfOpen,
            failure_count: 3,
            success_count: 2,
            request_count: 5,
        };
        assert_eq!(metrics.state, CircuitState::HalfOpen);
        assert_eq!(metrics.success_count, 2);
    }

    #[test]
    fn test_circuit_breaker_metrics_clone() {
        let metrics = CircuitBreakerMetrics {
            state: CircuitState::Closed,
            failure_count: 1,
            success_count: 5,
            request_count: 10,
        };
        let cloned = metrics.clone();
        assert_eq!(metrics.state, cloned.state);
        assert_eq!(metrics.failure_count, cloned.failure_count);
    }
}
