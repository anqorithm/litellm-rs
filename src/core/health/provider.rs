//! Provider health tracking
//!
//! This module provides health tracking for individual providers including
//! health history, metrics calculation, and routing weights.

use super::types::{HealthCheckResult, HealthStatus};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};

/// Provider health information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderHealth {
    /// Provider identifier
    pub provider_id: String,
    /// Current health status
    pub status: HealthStatus,
    /// Last health check result
    pub last_check: Option<HealthCheckResult>,
    /// Health check history (last N checks) - uses VecDeque for O(1) pop_front
    pub history: VecDeque<HealthCheckResult>,
    /// Average response time over recent checks
    pub avg_response_time_ms: f64,
    /// Success rate over recent checks
    pub success_rate: f64,
    /// Number of consecutive failures
    pub consecutive_failures: u32,
    /// When the provider was last healthy
    pub last_healthy: Option<chrono::DateTime<chrono::Utc>>,
    /// Custom health metrics
    pub metrics: HashMap<String, f64>,
}

impl ProviderHealth {
    /// Create new provider health tracking
    pub fn new(provider_id: String) -> Self {
        Self {
            provider_id,
            status: HealthStatus::Healthy,
            last_check: None,
            history: VecDeque::new(),
            avg_response_time_ms: 0.0,
            success_rate: 100.0,
            consecutive_failures: 0,
            last_healthy: Some(chrono::Utc::now()),
            metrics: HashMap::new(),
        }
    }

    /// Update with new health check result
    pub fn update(&mut self, result: HealthCheckResult) {
        // Update status
        self.status = result.status.clone();

        // Update consecutive failures
        if result.status == HealthStatus::Healthy {
            self.consecutive_failures = 0;
            self.last_healthy = Some(result.timestamp);
        } else {
            self.consecutive_failures += 1;
        }

        // Add to history (keep last 50 results)
        self.history.push_back(result.clone());
        if self.history.len() > 50 {
            self.history.pop_front();
        }

        // Update last check
        self.last_check = Some(result);

        // Recalculate metrics
        self.recalculate_metrics();
    }

    /// Recalculate aggregate metrics
    fn recalculate_metrics(&mut self) {
        if self.history.is_empty() {
            return;
        }

        // Calculate average response time
        let total_time: u64 = self.history.iter().map(|h| h.response_time_ms).sum();
        self.avg_response_time_ms = total_time as f64 / self.history.len() as f64;

        // Calculate success rate
        let successful_checks = self
            .history
            .iter()
            .filter(|h| h.status == HealthStatus::Healthy || h.status == HealthStatus::Degraded)
            .count();
        self.success_rate = (successful_checks as f64 / self.history.len() as f64) * 100.0;
    }

    /// Check if provider should be considered available for routing
    pub fn is_available(&self) -> bool {
        self.status.allows_requests() && self.consecutive_failures < 5
    }

    /// Get routing weight based on health
    pub fn routing_weight(&self) -> f64 {
        if !self.is_available() {
            return 0.0;
        }

        let status_weight = self.status.score() as f64 / 100.0;
        let success_weight = self.success_rate / 100.0;
        let latency_weight = if self.avg_response_time_ms > 0.0 {
            1.0 / (1.0 + self.avg_response_time_ms / 1000.0)
        } else {
            1.0
        };

        (status_weight + success_weight + latency_weight) / 3.0
    }
}

/// System health aggregator
pub struct SystemHealth {
    provider_health: HashMap<String, ProviderHealth>,
}

impl SystemHealth {
    /// Create system health snapshot
    pub fn new(provider_health: HashMap<String, ProviderHealth>) -> Self {
        Self { provider_health }
    }

    /// Get overall system health status
    pub fn overall_status(&self) -> HealthStatus {
        if self.provider_health.is_empty() {
            return HealthStatus::Down;
        }

        let total_providers = self.provider_health.len();
        let healthy_providers = self
            .provider_health
            .values()
            .filter(|h| h.status == HealthStatus::Healthy)
            .count();
        let available_providers = self
            .provider_health
            .values()
            .filter(|h| h.is_available())
            .count();

        if available_providers == 0 {
            HealthStatus::Down
        } else if healthy_providers == total_providers {
            HealthStatus::Healthy
        } else if available_providers >= total_providers / 2 {
            HealthStatus::Degraded
        } else {
            HealthStatus::Unhealthy
        }
    }

    /// Get system health metrics
    pub fn metrics(&self) -> HashMap<String, f64> {
        let mut metrics = HashMap::new();

        if !self.provider_health.is_empty() {
            let total = self.provider_health.len() as f64;
            let healthy = self
                .provider_health
                .values()
                .filter(|h| h.status == HealthStatus::Healthy)
                .count() as f64;
            let available = self
                .provider_health
                .values()
                .filter(|h| h.is_available())
                .count() as f64;

            metrics.insert("total_providers".to_string(), total);
            metrics.insert("healthy_providers".to_string(), healthy);
            metrics.insert("available_providers".to_string(), available);
            metrics.insert("health_percentage".to_string(), (healthy / total) * 100.0);
            metrics.insert(
                "availability_percentage".to_string(),
                (available / total) * 100.0,
            );

            // Average response time across all providers
            let avg_response_time: f64 = self
                .provider_health
                .values()
                .map(|h| h.avg_response_time_ms)
                .sum::<f64>()
                / total;
            metrics.insert("avg_response_time_ms".to_string(), avg_response_time);
        }

        metrics
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ==================== Helper Functions ====================

    fn create_test_health_result(status: HealthStatus, response_time_ms: u64) -> HealthCheckResult {
        HealthCheckResult {
            status,
            timestamp: chrono::Utc::now(),
            response_time_ms,
            details: None,
            error: None,
            metrics: HashMap::new(),
        }
    }

    // ==================== ProviderHealth Tests ====================

    #[test]
    fn test_provider_health_new() {
        let health = ProviderHealth::new("openai".to_string());

        assert_eq!(health.provider_id, "openai");
        assert_eq!(health.status, HealthStatus::Healthy);
        assert!(health.last_check.is_none());
        assert!(health.history.is_empty());
        assert_eq!(health.avg_response_time_ms, 0.0);
        assert_eq!(health.success_rate, 100.0);
        assert_eq!(health.consecutive_failures, 0);
        assert!(health.last_healthy.is_some());
    }

    #[test]
    fn test_provider_health_update_healthy() {
        let mut health = ProviderHealth::new("anthropic".to_string());
        let result = create_test_health_result(HealthStatus::Healthy, 150);

        health.update(result.clone());

        assert_eq!(health.status, HealthStatus::Healthy);
        assert_eq!(health.consecutive_failures, 0);
        assert!(health.last_check.is_some());
        assert_eq!(health.history.len(), 1);
    }

    #[test]
    fn test_provider_health_update_unhealthy() {
        let mut health = ProviderHealth::new("test".to_string());
        let result = create_test_health_result(HealthStatus::Unhealthy, 500);

        health.update(result);

        assert_eq!(health.status, HealthStatus::Unhealthy);
        assert_eq!(health.consecutive_failures, 1);
    }

    #[test]
    fn test_provider_health_consecutive_failures() {
        let mut health = ProviderHealth::new("test".to_string());

        // Add 3 failures
        for _ in 0..3 {
            let result = create_test_health_result(HealthStatus::Unhealthy, 0);
            health.update(result);
        }

        assert_eq!(health.consecutive_failures, 3);

        // Add a healthy result - should reset
        let healthy = create_test_health_result(HealthStatus::Healthy, 100);
        health.update(healthy);

        assert_eq!(health.consecutive_failures, 0);
    }

    #[test]
    fn test_provider_health_history_limit() {
        let mut health = ProviderHealth::new("test".to_string());

        // Add more than 50 results
        for i in 0..60 {
            let result = create_test_health_result(HealthStatus::Healthy, i as u64);
            health.update(result);
        }

        // Should only keep last 50
        assert_eq!(health.history.len(), 50);
    }

    #[test]
    fn test_provider_health_avg_response_time() {
        let mut health = ProviderHealth::new("test".to_string());

        // Add results with known response times
        health.update(create_test_health_result(HealthStatus::Healthy, 100));
        health.update(create_test_health_result(HealthStatus::Healthy, 200));
        health.update(create_test_health_result(HealthStatus::Healthy, 300));

        // Average should be (100 + 200 + 300) / 3 = 200
        assert!((health.avg_response_time_ms - 200.0).abs() < 0.01);
    }

    #[test]
    fn test_provider_health_success_rate() {
        let mut health = ProviderHealth::new("test".to_string());

        // Add 3 healthy, 1 unhealthy
        health.update(create_test_health_result(HealthStatus::Healthy, 100));
        health.update(create_test_health_result(HealthStatus::Healthy, 100));
        health.update(create_test_health_result(HealthStatus::Healthy, 100));
        health.update(create_test_health_result(HealthStatus::Unhealthy, 100));

        // Success rate should be 75%
        assert!((health.success_rate - 75.0).abs() < 0.01);
    }

    #[test]
    fn test_provider_health_success_rate_includes_degraded() {
        let mut health = ProviderHealth::new("test".to_string());

        // Add healthy and degraded results
        health.update(create_test_health_result(HealthStatus::Healthy, 100));
        health.update(create_test_health_result(HealthStatus::Degraded, 100));

        // Both should count as successful
        assert!((health.success_rate - 100.0).abs() < 0.01);
    }

    #[test]
    fn test_provider_health_is_available_healthy() {
        let health = ProviderHealth::new("test".to_string());
        assert!(health.is_available());
    }

    #[test]
    fn test_provider_health_is_available_too_many_failures() {
        let mut health = ProviderHealth::new("test".to_string());

        // Add 5 failures
        for _ in 0..5 {
            health.update(create_test_health_result(HealthStatus::Unhealthy, 0));
        }

        assert!(!health.is_available());
    }

    #[test]
    fn test_provider_health_is_available_down_status() {
        let mut health = ProviderHealth::new("test".to_string());
        health.update(create_test_health_result(HealthStatus::Down, 0));

        assert!(!health.is_available());
    }

    #[test]
    fn test_provider_health_routing_weight_healthy() {
        let health = ProviderHealth::new("test".to_string());
        let weight = health.routing_weight();

        // Should be positive for healthy provider
        assert!(weight > 0.0);
        assert!(weight <= 1.0);
    }

    #[test]
    fn test_provider_health_routing_weight_unavailable() {
        let mut health = ProviderHealth::new("test".to_string());

        // Make unavailable
        for _ in 0..5 {
            health.update(create_test_health_result(HealthStatus::Unhealthy, 0));
        }

        assert_eq!(health.routing_weight(), 0.0);
    }

    #[test]
    fn test_provider_health_routing_weight_with_latency() {
        let mut health1 = ProviderHealth::new("fast".to_string());
        let mut health2 = ProviderHealth::new("slow".to_string());

        // Fast provider
        health1.update(create_test_health_result(HealthStatus::Healthy, 50));

        // Slow provider
        health2.update(create_test_health_result(HealthStatus::Healthy, 5000));

        // Fast should have higher weight
        assert!(health1.routing_weight() > health2.routing_weight());
    }

    // ==================== SystemHealth Tests ====================

    #[test]
    fn test_system_health_new() {
        let providers = HashMap::new();
        let system_health = SystemHealth::new(providers);

        assert!(system_health.provider_health.is_empty());
    }

    #[test]
    fn test_system_health_overall_status_empty() {
        let system_health = SystemHealth::new(HashMap::new());
        assert_eq!(system_health.overall_status(), HealthStatus::Down);
    }

    #[test]
    fn test_system_health_overall_status_all_healthy() {
        let mut providers = HashMap::new();
        providers.insert(
            "openai".to_string(),
            ProviderHealth::new("openai".to_string()),
        );
        providers.insert(
            "anthropic".to_string(),
            ProviderHealth::new("anthropic".to_string()),
        );

        let system_health = SystemHealth::new(providers);
        assert_eq!(system_health.overall_status(), HealthStatus::Healthy);
    }

    #[test]
    fn test_system_health_overall_status_degraded() {
        let mut providers = HashMap::new();

        let mut healthy = ProviderHealth::new("healthy".to_string());
        healthy.update(create_test_health_result(HealthStatus::Healthy, 100));
        providers.insert("healthy".to_string(), healthy);

        let mut unhealthy = ProviderHealth::new("unhealthy".to_string());
        unhealthy.update(create_test_health_result(HealthStatus::Unhealthy, 100));
        providers.insert("unhealthy".to_string(), unhealthy);

        let system_health = SystemHealth::new(providers);
        // 1 healthy, 1 unhealthy = 50% available = degraded
        assert_eq!(system_health.overall_status(), HealthStatus::Degraded);
    }

    #[test]
    fn test_system_health_overall_status_down() {
        let mut providers = HashMap::new();

        let mut down1 = ProviderHealth::new("down1".to_string());
        for _ in 0..5 {
            down1.update(create_test_health_result(HealthStatus::Unhealthy, 0));
        }
        providers.insert("down1".to_string(), down1);

        let mut down2 = ProviderHealth::new("down2".to_string());
        for _ in 0..5 {
            down2.update(create_test_health_result(HealthStatus::Unhealthy, 0));
        }
        providers.insert("down2".to_string(), down2);

        let system_health = SystemHealth::new(providers);
        assert_eq!(system_health.overall_status(), HealthStatus::Down);
    }

    #[test]
    fn test_system_health_metrics_empty() {
        let system_health = SystemHealth::new(HashMap::new());
        let metrics = system_health.metrics();

        assert!(metrics.is_empty());
    }

    #[test]
    fn test_system_health_metrics() {
        let mut providers = HashMap::new();

        let mut p1 = ProviderHealth::new("p1".to_string());
        p1.update(create_test_health_result(HealthStatus::Healthy, 100));
        providers.insert("p1".to_string(), p1);

        let mut p2 = ProviderHealth::new("p2".to_string());
        p2.update(create_test_health_result(HealthStatus::Healthy, 200));
        providers.insert("p2".to_string(), p2);

        let system_health = SystemHealth::new(providers);
        let metrics = system_health.metrics();

        assert_eq!(metrics.get("total_providers"), Some(&2.0));
        assert_eq!(metrics.get("healthy_providers"), Some(&2.0));
        assert_eq!(metrics.get("available_providers"), Some(&2.0));
        assert_eq!(metrics.get("health_percentage"), Some(&100.0));
        assert_eq!(metrics.get("availability_percentage"), Some(&100.0));

        // Average of 100 and 200 = 150
        let avg = metrics.get("avg_response_time_ms").unwrap();
        assert!((avg - 150.0).abs() < 0.01);
    }

    #[test]
    fn test_system_health_metrics_partial_health() {
        let mut providers = HashMap::new();

        let mut healthy = ProviderHealth::new("healthy".to_string());
        healthy.update(create_test_health_result(HealthStatus::Healthy, 100));
        providers.insert("healthy".to_string(), healthy);

        let mut unhealthy = ProviderHealth::new("unhealthy".to_string());
        unhealthy.update(create_test_health_result(HealthStatus::Unhealthy, 100));
        providers.insert("unhealthy".to_string(), unhealthy);

        let system_health = SystemHealth::new(providers);
        let metrics = system_health.metrics();

        assert_eq!(metrics.get("total_providers"), Some(&2.0));
        assert_eq!(metrics.get("healthy_providers"), Some(&1.0));
        assert_eq!(metrics.get("health_percentage"), Some(&50.0));
    }

    // ==================== Edge Cases ====================

    #[test]
    fn test_provider_health_empty_history_metrics() {
        let health = ProviderHealth::new("test".to_string());
        // Should not panic with empty history
        assert_eq!(health.avg_response_time_ms, 0.0);
        assert_eq!(health.success_rate, 100.0);
    }

    #[test]
    fn test_provider_health_single_result() {
        let mut health = ProviderHealth::new("test".to_string());
        health.update(create_test_health_result(HealthStatus::Healthy, 250));

        assert_eq!(health.history.len(), 1);
        assert!((health.avg_response_time_ms - 250.0).abs() < 0.01);
        assert!((health.success_rate - 100.0).abs() < 0.01);
    }

    #[test]
    fn test_provider_health_clone() {
        let mut health = ProviderHealth::new("test".to_string());
        health.update(create_test_health_result(HealthStatus::Healthy, 100));

        let cloned = health.clone();
        assert_eq!(cloned.provider_id, health.provider_id);
        assert_eq!(cloned.status, health.status);
        assert_eq!(cloned.history.len(), health.history.len());
    }

    #[test]
    fn test_provider_health_last_healthy_updated() {
        let mut health = ProviderHealth::new("test".to_string());
        let _initial_last_healthy = health.last_healthy;

        // Add unhealthy - should not update last_healthy
        health.update(create_test_health_result(HealthStatus::Unhealthy, 0));
        // last_healthy should remain as initial (when provider was created as healthy)

        // Now add healthy - should update last_healthy
        let before_update = chrono::Utc::now();
        health.update(create_test_health_result(HealthStatus::Healthy, 100));
        let after_update = chrono::Utc::now();

        let last_healthy = health.last_healthy.unwrap();
        assert!(last_healthy >= before_update);
        assert!(last_healthy <= after_update);
    }

    #[test]
    fn test_system_health_single_provider() {
        let mut providers = HashMap::new();
        providers.insert(
            "single".to_string(),
            ProviderHealth::new("single".to_string()),
        );

        let system_health = SystemHealth::new(providers);
        assert_eq!(system_health.overall_status(), HealthStatus::Healthy);

        let metrics = system_health.metrics();
        assert_eq!(metrics.get("total_providers"), Some(&1.0));
    }

    #[test]
    fn test_routing_weight_zero_response_time() {
        let health = ProviderHealth::new("test".to_string());
        // With 0 avg_response_time_ms, latency_weight should be 1.0
        let weight = health.routing_weight();
        assert!(weight > 0.0);
    }
}
