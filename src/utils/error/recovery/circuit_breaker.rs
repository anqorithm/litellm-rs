//! Circuit breaker implementation for fault tolerance

use super::types::{CircuitBreakerConfig, CircuitBreakerMetrics, CircuitState};
use crate::utils::error::error::{GatewayError, Result};
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
mod circuit_breaker_tests;
