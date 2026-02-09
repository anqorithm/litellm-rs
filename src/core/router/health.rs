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
mod health_tests;
