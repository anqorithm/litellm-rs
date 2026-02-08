//! Core LoadBalancer struct and basic methods
//!
//! **DEPRECATED**: This module is part of the legacy load balancer system.
//! For new code, use `crate::core::router::UnifiedRouter` instead.

use super::deployment_info::DeploymentInfo;
use super::fallback_config::FallbackConfig;
use crate::core::providers::Provider;
use crate::core::router::health::HealthChecker;
use crate::core::router::strategy::executor::StrategyExecutor;
use crate::core::router::strategy::types::RoutingStrategy;
use crate::utils::error::error::Result;
use dashmap::DashMap;
use std::sync::Arc;
use tracing::info;

/// Load balancer for intelligent provider selection
///
/// **DEPRECATED**: Use `crate::core::router::UnifiedRouter` for new code.
pub struct LoadBalancer {
    /// Available providers
    pub(crate) providers: DashMap<String, Provider>,
    /// Deployment information for each provider
    pub(crate) deployments: DashMap<String, DeploymentInfo>,
    /// Strategy executor
    pub(crate) strategy: Arc<StrategyExecutor>,
    /// Health checker
    pub(crate) health_checker: Option<Arc<HealthChecker>>,
    /// Provider model support cache
    pub(crate) model_support_cache: DashMap<String, Arc<Vec<String>>>,
    /// Error-specific fallback configuration
    pub(crate) fallback_config: FallbackConfig,
}

impl LoadBalancer {
    /// Create a new load balancer
    pub async fn new(strategy: RoutingStrategy) -> Result<Self> {
        info!("Creating load balancer with strategy: {:?}", strategy);

        let strategy_executor = Arc::new(StrategyExecutor::new(strategy).await?);

        Ok(Self {
            providers: DashMap::new(),
            deployments: DashMap::new(),
            strategy: strategy_executor,
            health_checker: None,
            model_support_cache: DashMap::new(),
            fallback_config: FallbackConfig::default(),
        })
    }

    /// Create a new load balancer with fallback configuration
    pub async fn with_fallbacks(
        strategy: RoutingStrategy,
        fallback_config: FallbackConfig,
    ) -> Result<Self> {
        info!(
            "Creating load balancer with strategy: {:?} and fallback config",
            strategy
        );

        let strategy_executor = Arc::new(StrategyExecutor::new(strategy).await?);

        Ok(Self {
            providers: DashMap::new(),
            deployments: DashMap::new(),
            strategy: strategy_executor,
            health_checker: None,
            model_support_cache: DashMap::new(),
            fallback_config,
        })
    }

    /// Set fallback configuration
    pub fn set_fallback_config(&mut self, config: FallbackConfig) {
        self.fallback_config = config;
        info!("Updated fallback configuration");
    }

    /// Get fallback configuration
    pub fn fallback_config(&self) -> &FallbackConfig {
        &self.fallback_config
    }

    /// Set health checker
    pub async fn set_health_checker(&mut self, health_checker: Arc<HealthChecker>) {
        self.health_checker = Some(health_checker);
        info!("Health checker attached to load balancer");
    }

    /// Update deployment info for an existing provider
    pub fn update_deployment_info(&self, name: &str, deployment_info: DeploymentInfo) {
        self.deployments.insert(name.to_string(), deployment_info);
        tracing::debug!("Updated deployment info for provider {}", name);
    }

    /// Get deployment info for a provider
    pub fn get_deployment_info(&self, name: &str) -> Option<DeploymentInfo> {
        self.deployments
            .get(name)
            .map(|entry| entry.value().clone())
    }

    /// Update provider weight for weighted routing
    pub async fn update_provider_weight(&self, provider: &str, weight: f64) -> Result<()> {
        self.strategy.update_weight(provider, weight).await
    }

    /// Update provider latency for latency-based routing
    pub async fn update_provider_latency(&self, provider: &str, latency: f64) -> Result<()> {
        self.strategy.update_latency(provider, latency).await
    }

    /// Update provider cost for cost-based routing
    pub async fn update_provider_cost(&self, provider: &str, model: &str, cost: f64) -> Result<()> {
        self.strategy.update_cost(provider, model, cost).await
    }

    /// Update provider priority for priority-based routing
    pub async fn update_provider_priority(&self, provider: &str, priority: u32) -> Result<()> {
        self.strategy.update_priority(provider, priority).await
    }
}

/// Load balancer statistics
#[derive(Debug, Clone)]
pub struct LoadBalancerStats {
    /// Total number of providers
    pub total_providers: usize,
    /// Number of healthy providers
    pub healthy_providers: usize,
    /// Number of cached model mappings
    pub cached_models: usize,
}
