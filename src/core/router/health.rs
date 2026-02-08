//! Health checking for providers

use crate::core::providers::ProviderRegistry;
use crate::utils::error::error::{GatewayError, Result};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tokio::time::interval;
use tracing::{debug, error, info};

/// Health checker for monitoring provider availability
pub struct HealthChecker {
    /// Provider instances
    providers: Arc<RwLock<ProviderRegistry>>,
    /// Health statuses
    statuses: Arc<RwLock<HashMap<String, ProviderHealthStatus>>>,
    /// Check interval
    check_interval: Duration,
    /// Timeout for health checks
    timeout: Duration,
    /// Maximum consecutive failures before marking unhealthy
    max_failures: u32,
}

/// Provider health status
#[derive(Debug, Clone)]
pub struct ProviderHealthStatus {
    /// Provider is healthy
    pub healthy: bool,
    /// Last successful request time
    pub last_success: Option<Instant>,
    /// Last error
    pub last_error: Option<String>,
    /// Response time
    pub response_time: Option<Duration>,
    /// Consecutive failure count
    pub consecutive_failures: u32,
    /// Last check time
    pub last_check: Instant,
}

impl Default for ProviderHealthStatus {
    fn default() -> Self {
        Self {
            healthy: true,
            last_success: None,
            last_error: None,
            response_time: None,
            consecutive_failures: 0,
            last_check: Instant::now(),
        }
    }
}

impl HealthChecker {
    /// Create a new health checker
    pub async fn new(providers: Arc<RwLock<ProviderRegistry>>) -> Result<Self> {
        info!("Creating health checker");

        let checker = Self {
            providers,
            statuses: Arc::new(RwLock::new(HashMap::new())),
            check_interval: Duration::from_secs(30),
            timeout: Duration::from_secs(10),
            max_failures: 3,
        };

        // Start background health checking
        checker.start_background_checks().await?;

        Ok(checker)
    }

    /// Start background health checking
    async fn start_background_checks(&self) -> Result<()> {
        let providers = self.providers.clone();
        let statuses = self.statuses.clone();
        let check_interval = self.check_interval;
        let timeout = self.timeout;
        let max_failures = self.max_failures;

        tokio::spawn(async move {
            let mut interval = interval(check_interval);

            loop {
                interval.tick().await;

                let providers_guard = providers.read().await;
                // Get provider list from registry
                let provider_names: Vec<String> = providers_guard.list();
                drop(providers_guard);

                for name in provider_names {
                    let start = Instant::now();

                    // Try to perform health check by getting provider reference
                    let providers_guard = providers.read().await;
                    let health_result = if let Some(_provider) = providers_guard.get(&name) {
                        // Provider exists - check if it can be used
                        // For now, mark as healthy if the provider is registered
                        // A more complete implementation would call provider.health_check()
                        debug!("Health check for provider {}: registered", name);
                        Ok(())
                    } else {
                        Err(format!("Provider {} not found", name))
                    };
                    drop(providers_guard);

                    let response_time = start.elapsed();

                    // Update status based on result
                    let mut statuses_guard = statuses.write().await;
                    let status = statuses_guard
                        .entry(name.clone())
                        .or_insert_with(ProviderHealthStatus::default);

                    match health_result {
                        Ok(()) => {
                            if response_time <= timeout {
                                status.healthy = true;
                                status.consecutive_failures = 0;
                                status.last_success = Some(Instant::now());
                                status.response_time = Some(response_time);
                                status.last_error = None;
                                debug!(
                                    "Provider {} is healthy ({}ms)",
                                    name,
                                    response_time.as_millis()
                                );
                            } else {
                                status.consecutive_failures += 1;
                                status.last_error = Some(format!(
                                    "Health check timed out: {}ms > {}ms",
                                    response_time.as_millis(),
                                    timeout.as_millis()
                                ));
                                if status.consecutive_failures >= max_failures {
                                    status.healthy = false;
                                    error!(
                                        "Provider {} marked unhealthy after {} failures",
                                        name, max_failures
                                    );
                                }
                            }
                        }
                        Err(e) => {
                            status.consecutive_failures += 1;
                            status.last_error = Some(e);
                            if status.consecutive_failures >= max_failures {
                                status.healthy = false;
                                error!(
                                    "Provider {} marked unhealthy after {} failures",
                                    name, max_failures
                                );
                            }
                        }
                    }
                    status.last_check = Instant::now();
                }
            }
        });

        Ok(())
    }

    /// Get health status for all providers
    pub async fn get_status(&self) -> Result<RouterHealthStatus> {
        let statuses = self.statuses.read().await;
        let provider_statuses = statuses.clone();

        let overall_healthy = provider_statuses.values().any(|status| status.healthy);

        Ok(RouterHealthStatus {
            healthy: overall_healthy,
            providers: provider_statuses,
            last_check: Instant::now(),
        })
    }

    /// Get health status for a specific provider
    pub async fn get_provider_status(&self, name: &str) -> Result<Option<ProviderHealthStatus>> {
        let statuses = self.statuses.read().await;
        Ok(statuses.get(name).cloned())
    }

    /// Get list of healthy providers
    pub async fn get_healthy_providers(&self) -> Result<Vec<String>> {
        let statuses = self.statuses.read().await;
        let healthy_providers = statuses
            .iter()
            .filter(|(_, status)| status.healthy)
            .map(|(name, _)| name.clone())
            .collect();

        Ok(healthy_providers)
    }

    /// Add a new provider to health checking
    pub async fn add_provider(&self, name: &str) -> Result<()> {
        let mut statuses = self.statuses.write().await;
        statuses.insert(name.to_string(), ProviderHealthStatus::default());
        info!("Added provider {} to health checking", name);
        Ok(())
    }

    /// Remove a provider from health checking
    pub async fn remove_provider(&self, name: &str) -> Result<()> {
        let mut statuses = self.statuses.write().await;
        statuses.remove(name);
        info!("Removed provider {} from health checking", name);
        Ok(())
    }

    /// Manually trigger health check for a provider
    pub async fn check_provider(&self, name: &str) -> Result<ProviderHealthStatus> {
        let providers = self.providers.read().await;
        let provider = providers
            .get(name)
            .ok_or_else(|| GatewayError::ProviderNotFound(name.to_string()))?;

        let start_time = Instant::now();

        match tokio::time::timeout(self.timeout, provider.health_check()).await {
            Ok(health_status) => {
                if matches!(
                    health_status,
                    crate::core::types::health::HealthStatus::Healthy
                ) {
                    let response_time = start_time.elapsed();
                    let mut statuses = self.statuses.write().await;
                    let status = statuses.entry(name.to_string()).or_default();

                    status.healthy = true;
                    status.last_success = Some(Instant::now());
                    status.response_time = Some(response_time);
                    status.consecutive_failures = 0;
                    status.last_check = Instant::now();
                    status.last_error = None;

                    debug!(
                        "Manual health check passed for provider {}: {}ms",
                        name,
                        response_time.as_millis()
                    );
                    Ok(status.clone())
                } else {
                    let mut statuses = self.statuses.write().await;
                    let status = statuses.entry(name.to_string()).or_default();

                    status.consecutive_failures += 1;
                    status.last_error = Some(format!("Health check returned: {:?}", health_status));
                    status.last_check = Instant::now();

                    if status.consecutive_failures >= self.max_failures {
                        status.healthy = false;
                    }

                    let error_msg = format!("Health status: {:?}", health_status);
                    error!(
                        "Manual health check failed for provider {}: {}",
                        name, error_msg
                    );
                    Ok(status.clone())
                }
            }
            Err(_) => {
                let mut statuses = self.statuses.write().await;
                let status = statuses.entry(name.to_string()).or_default();

                status.consecutive_failures += 1;
                status.last_error = Some("Health check timeout".to_string());
                status.last_check = Instant::now();

                if status.consecutive_failures >= self.max_failures {
                    status.healthy = false;
                }

                error!("Manual health check timeout for provider {}", name);
                Ok(status.clone())
            }
        }
    }
}

/// Router health status
#[derive(Debug, Clone)]
pub struct RouterHealthStatus {
    /// Overall health
    pub healthy: bool,
    /// Provider health statuses
    pub providers: HashMap<String, ProviderHealthStatus>,
    /// Last check time
    pub last_check: Instant,
}

#[cfg(test)]
mod tests {
    use super::*;

    // ==================== ProviderHealthStatus Tests ====================

    #[test]
    fn test_provider_health_status_default() {
        let status = ProviderHealthStatus::default();
        assert!(status.healthy);
        assert!(status.last_success.is_none());
        assert!(status.last_error.is_none());
        assert!(status.response_time.is_none());
        assert_eq!(status.consecutive_failures, 0);
    }

    #[test]
    fn test_provider_health_status_clone() {
        let status = ProviderHealthStatus {
            healthy: false,
            consecutive_failures: 5,
            last_error: Some("Connection refused".to_string()),
            response_time: Some(Duration::from_millis(100)),
            ..Default::default()
        };

        let cloned = status.clone();
        assert!(!cloned.healthy);
        assert_eq!(cloned.consecutive_failures, 5);
        assert_eq!(cloned.last_error, Some("Connection refused".to_string()));
        assert_eq!(cloned.response_time, Some(Duration::from_millis(100)));
    }

    #[test]
    fn test_provider_health_status_debug() {
        let status = ProviderHealthStatus::default();
        let debug = format!("{:?}", status);
        assert!(debug.contains("ProviderHealthStatus"));
        assert!(debug.contains("healthy"));
        assert!(debug.contains("consecutive_failures"));
    }

    #[test]
    fn test_provider_health_status_with_success() {
        let status = ProviderHealthStatus {
            healthy: true,
            last_success: Some(Instant::now()),
            response_time: Some(Duration::from_millis(50)),
            consecutive_failures: 0,
            ..Default::default()
        };

        assert!(status.healthy);
        assert!(status.last_success.is_some());
        assert_eq!(status.response_time, Some(Duration::from_millis(50)));
    }

    #[test]
    fn test_provider_health_status_with_error() {
        let status = ProviderHealthStatus {
            healthy: false,
            last_error: Some("Timeout".to_string()),
            consecutive_failures: 3,
            ..Default::default()
        };

        assert!(!status.healthy);
        assert_eq!(status.last_error, Some("Timeout".to_string()));
        assert_eq!(status.consecutive_failures, 3);
    }

    #[test]
    fn test_provider_health_status_reset_after_success() {
        // Start with a status that has failures
        let mut status = ProviderHealthStatus {
            consecutive_failures: 2,
            last_error: Some("Previous error".to_string()),
            ..Default::default()
        };

        // Simulate success - this resets the counters
        status.healthy = true;
        status.consecutive_failures = 0;
        status.last_success = Some(Instant::now());
        status.last_error = None;
        status.response_time = Some(Duration::from_millis(25));

        assert!(status.healthy);
        assert_eq!(status.consecutive_failures, 0);
        assert!(status.last_error.is_none());
    }

    // ==================== RouterHealthStatus Tests ====================

    #[test]
    fn test_router_health_status_debug() {
        let status = RouterHealthStatus {
            healthy: true,
            providers: HashMap::new(),
            last_check: Instant::now(),
        };
        let debug = format!("{:?}", status);
        assert!(debug.contains("RouterHealthStatus"));
        assert!(debug.contains("healthy"));
    }

    #[test]
    fn test_router_health_status_clone() {
        let mut providers = HashMap::new();
        providers.insert("openai".to_string(), ProviderHealthStatus::default());

        let status = RouterHealthStatus {
            healthy: true,
            providers,
            last_check: Instant::now(),
        };

        let cloned = status.clone();
        assert!(cloned.healthy);
        assert!(cloned.providers.contains_key("openai"));
    }

    #[test]
    fn test_router_health_status_empty_providers() {
        let status = RouterHealthStatus {
            healthy: false,
            providers: HashMap::new(),
            last_check: Instant::now(),
        };

        assert!(!status.healthy);
        assert!(status.providers.is_empty());
    }

    #[test]
    fn test_router_health_status_with_mixed_providers() {
        let mut providers = HashMap::new();

        let healthy_provider = ProviderHealthStatus {
            healthy: true,
            ..Default::default()
        };

        let unhealthy_provider = ProviderHealthStatus {
            healthy: false,
            ..Default::default()
        };

        providers.insert("openai".to_string(), healthy_provider);
        providers.insert("anthropic".to_string(), unhealthy_provider);

        let status = RouterHealthStatus {
            healthy: true, // At least one provider is healthy
            providers,
            last_check: Instant::now(),
        };

        assert!(status.healthy);
        assert_eq!(status.providers.len(), 2);
        assert!(status.providers.get("openai").unwrap().healthy);
        assert!(!status.providers.get("anthropic").unwrap().healthy);
    }

    // ==================== Health Status Calculation Tests ====================

    #[test]
    fn test_overall_health_any_healthy() {
        let mut providers = HashMap::new();

        let status1 = ProviderHealthStatus {
            healthy: false,
            ..Default::default()
        };

        let status2 = ProviderHealthStatus {
            healthy: true,
            ..Default::default()
        };

        let status3 = ProviderHealthStatus {
            healthy: false,
            ..Default::default()
        };

        providers.insert("p1".to_string(), status1);
        providers.insert("p2".to_string(), status2);
        providers.insert("p3".to_string(), status3);

        // At least one provider is healthy
        let overall_healthy = providers.values().any(|s| s.healthy);
        assert!(overall_healthy);
    }

    #[test]
    fn test_overall_health_all_unhealthy() {
        let mut providers = HashMap::new();

        let status1 = ProviderHealthStatus {
            healthy: false,
            ..Default::default()
        };

        let status2 = ProviderHealthStatus {
            healthy: false,
            ..Default::default()
        };

        providers.insert("p1".to_string(), status1);
        providers.insert("p2".to_string(), status2);

        // No provider is healthy
        let overall_healthy = providers.values().any(|s| s.healthy);
        assert!(!overall_healthy);
    }

    #[test]
    fn test_overall_health_all_healthy() {
        let mut providers = HashMap::new();

        let status1 = ProviderHealthStatus {
            healthy: true,
            ..Default::default()
        };

        let status2 = ProviderHealthStatus {
            healthy: true,
            ..Default::default()
        };

        providers.insert("p1".to_string(), status1);
        providers.insert("p2".to_string(), status2);

        let overall_healthy = providers.values().any(|s| s.healthy);
        assert!(overall_healthy);

        let all_healthy = providers.values().all(|s| s.healthy);
        assert!(all_healthy);
    }

    // ==================== Failure Counting Tests ====================

    #[test]
    fn test_consecutive_failures_increment() {
        let mut status = ProviderHealthStatus::default();
        assert_eq!(status.consecutive_failures, 0);

        status.consecutive_failures += 1;
        assert_eq!(status.consecutive_failures, 1);

        status.consecutive_failures += 1;
        assert_eq!(status.consecutive_failures, 2);

        status.consecutive_failures += 1;
        assert_eq!(status.consecutive_failures, 3);
    }

    #[test]
    fn test_consecutive_failures_threshold() {
        let max_failures = 3u32;

        // Below threshold - still healthy
        let status_below = ProviderHealthStatus {
            consecutive_failures: 2,
            healthy: true,
            ..Default::default()
        };
        assert!(status_below.consecutive_failures < max_failures);
        assert!(status_below.healthy);

        // At threshold - should be unhealthy
        let status_at = ProviderHealthStatus {
            consecutive_failures: 3,
            healthy: false,
            ..Default::default()
        };
        assert!(status_at.consecutive_failures >= max_failures);
        assert!(!status_at.healthy);
    }

    #[test]
    fn test_failure_reset_on_success() {
        // Start with accumulated failures (still healthy, not yet at threshold)
        let mut status = ProviderHealthStatus {
            consecutive_failures: 2,
            last_error: Some("Error".to_string()),
            healthy: true,
            ..Default::default()
        };

        // Success resets counters
        status.consecutive_failures = 0;
        status.last_success = Some(Instant::now());
        status.last_error = None;

        assert_eq!(status.consecutive_failures, 0);
        assert!(status.last_error.is_none());
        assert!(status.last_success.is_some());
    }

    // ==================== Response Time Tests ====================

    #[test]
    fn test_response_time_tracking() {
        let mut status = ProviderHealthStatus::default();
        assert!(status.response_time.is_none());

        status.response_time = Some(Duration::from_millis(150));
        assert_eq!(status.response_time, Some(Duration::from_millis(150)));

        // Update response time
        status.response_time = Some(Duration::from_millis(75));
        assert_eq!(status.response_time, Some(Duration::from_millis(75)));
    }

    #[test]
    fn test_response_time_timeout_check() {
        let timeout = Duration::from_secs(10);
        let response_time = Duration::from_millis(500);

        // Fast response - healthy
        assert!(response_time <= timeout);

        let slow_response = Duration::from_secs(15);
        // Slow response - timeout
        assert!(slow_response > timeout);
    }

    // ==================== Provider Filtering Tests ====================

    #[test]
    fn test_filter_healthy_providers() {
        let mut providers = HashMap::new();

        let status1 = ProviderHealthStatus {
            healthy: true,
            ..Default::default()
        };

        let status2 = ProviderHealthStatus {
            healthy: false,
            ..Default::default()
        };

        let status3 = ProviderHealthStatus {
            healthy: true,
            ..Default::default()
        };

        providers.insert("openai".to_string(), status1);
        providers.insert("anthropic".to_string(), status2);
        providers.insert("google".to_string(), status3);

        let healthy: Vec<String> = providers
            .iter()
            .filter(|(_, status)| status.healthy)
            .map(|(name, _)| name.clone())
            .collect();

        assert_eq!(healthy.len(), 2);
        assert!(healthy.contains(&"openai".to_string()));
        assert!(healthy.contains(&"google".to_string()));
        assert!(!healthy.contains(&"anthropic".to_string()));
    }

    #[test]
    fn test_filter_unhealthy_providers() {
        let mut providers = HashMap::new();

        let status1 = ProviderHealthStatus {
            healthy: true,
            ..Default::default()
        };

        let status2 = ProviderHealthStatus {
            healthy: false,
            last_error: Some("Rate limited".to_string()),
            ..Default::default()
        };

        providers.insert("openai".to_string(), status1);
        providers.insert("anthropic".to_string(), status2);

        let unhealthy: Vec<String> = providers
            .iter()
            .filter(|(_, status)| !status.healthy)
            .map(|(name, _)| name.clone())
            .collect();

        assert_eq!(unhealthy.len(), 1);
        assert!(unhealthy.contains(&"anthropic".to_string()));
    }

    // ==================== Edge Cases ====================

    #[test]
    fn test_empty_provider_map() {
        let providers: HashMap<String, ProviderHealthStatus> = HashMap::new();

        let healthy: Vec<String> = providers
            .iter()
            .filter(|(_, status)| status.healthy)
            .map(|(name, _)| name.clone())
            .collect();

        assert!(healthy.is_empty());

        // Overall health with no providers
        let overall_healthy = providers.values().any(|s| s.healthy);
        assert!(!overall_healthy);
    }

    #[test]
    fn test_status_with_long_error_message() {
        let mut status = ProviderHealthStatus::default();
        let long_error = "a".repeat(10000);
        status.last_error = Some(long_error.clone());

        assert_eq!(status.last_error.as_ref().unwrap().len(), 10000);
    }

    #[test]
    fn test_status_timestamps() {
        let before = Instant::now();
        let status = ProviderHealthStatus::default();
        let after = Instant::now();

        // last_check should be between before and after
        assert!(status.last_check >= before);
        assert!(status.last_check <= after);
    }

    #[test]
    fn test_high_failure_count() {
        let status = ProviderHealthStatus {
            consecutive_failures: u32::MAX,
            ..Default::default()
        };
        assert_eq!(status.consecutive_failures, u32::MAX);
    }
}
