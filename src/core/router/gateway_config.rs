//! Gateway configuration integration
//!
//! This module contains the from_gateway_config method for creating
//! a Router from gateway configuration.

use super::config::RouterConfig;
use super::deployment::{Deployment, DeploymentConfig};
use super::error::RouterError;
use super::unified::Router;
use crate::config::models::provider::ProviderConfig;
use crate::config::models::router::GatewayRouterConfig;
use crate::core::providers::{Provider, create_provider};
use tracing::warn;

/// Build runtime router config from gateway YAML router config.
///
/// Note: `GatewayRouterConfig` fields `circuit_breaker.min_requests`,
/// `circuit_breaker.success_threshold`, `load_balancer.sticky_sessions`, and
/// `load_balancer.session_timeout` are currently not mapped because runtime
/// `RouterConfig` has no corresponding fields yet.
pub fn runtime_router_config_from_gateway(config: &GatewayRouterConfig) -> RouterConfig {
    let has_unmapped_fields = config.circuit_breaker.min_requests != 10
        || config.circuit_breaker.success_threshold != 3
        || config.load_balancer.sticky_sessions
        || config.load_balancer.session_timeout != 3600;
    if has_unmapped_fields {
        warn!(
            min_requests = config.circuit_breaker.min_requests,
            success_threshold = config.circuit_breaker.success_threshold,
            sticky_sessions = config.load_balancer.sticky_sessions,
            session_timeout = config.load_balancer.session_timeout,
            "Gateway router config contains fields not mapped to runtime RouterConfig yet; values are ignored"
        );
    }

    RouterConfig {
        routing_strategy: config.strategy,
        // Gateway circuit-breaker thresholds are the closest semantic mapping here.
        allowed_fails: config.circuit_breaker.failure_threshold,
        cooldown_time_secs: config.circuit_breaker.recovery_timeout,
        enable_pre_call_checks: config.load_balancer.health_check_enabled,
        ..RouterConfig::default()
    }
}

impl Router {
    /// Create a Router from gateway configuration
    ///
    /// This method initializes a Router with deployments created from provider configurations.
    /// Each provider in the config becomes a deployment in the router.
    pub async fn from_gateway_config(
        providers: &[ProviderConfig],
        router_config: Option<RouterConfig>,
    ) -> Result<Self, RouterError> {
        let config = router_config.unwrap_or_default();
        let router = Self::new(config);

        for provider_config in providers {
            if !provider_config.enabled {
                continue;
            }

            // Create provider instance via the single canonical factory.
            let provider = create_provider(provider_config.clone())
                .await
                .map_err(|e| {
                    RouterError::DeploymentNotFound(format!(
                        "Failed to create provider {}: {}",
                        provider_config.name, e
                    ))
                })?;

            // Determine which models this deployment serves
            let models: Vec<String> = if !provider_config.models.is_empty() {
                provider_config.models.clone()
            } else {
                provider
                    .list_models()
                    .iter()
                    .map(|m| m.id.clone())
                    .collect()
            };

            // Create deployments
            if models.is_empty() {
                // Create a single deployment with provider name
                let deployment = create_deployment_from_config(
                    &provider_config.name,
                    provider.clone(),
                    &provider_config.name,
                    provider_config,
                );
                router.add_deployment(deployment);
            } else {
                // Create one deployment per model
                for model in models {
                    let deployment_id = format!("{}-{}", provider_config.name, model);
                    let deployment = create_deployment_from_config(
                        &deployment_id,
                        provider.clone(),
                        &model,
                        provider_config,
                    );
                    router.add_deployment(deployment);
                }
            }
        }

        Ok(router)
    }
}

/// Helper function to create deployment from provider config
fn create_deployment_from_config(
    deployment_id: &str,
    provider: Provider,
    model: &str,
    config: &ProviderConfig,
) -> Deployment {
    let deployment_config = DeploymentConfig {
        tpm_limit: if config.tpm > 0 {
            Some(config.tpm as u64)
        } else {
            None
        },
        rpm_limit: if config.rpm > 0 {
            Some(config.rpm as u64)
        } else {
            None
        },
        max_parallel_requests: if config.max_concurrent_requests > 0 {
            Some(config.max_concurrent_requests)
        } else {
            None
        },
        weight: (config.weight.max(1.0)).round() as u32,
        timeout_secs: config.timeout,
        priority: 0,
    };

    Deployment::new(
        deployment_id.to_string(),
        provider,
        model.to_string(),
        model.to_string(),
    )
    .with_config(deployment_config)
    .with_tags(config.tags.clone())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::models::router::{
        CircuitBreakerConfig, GatewayRouterConfig, LoadBalancerConfig, RoutingStrategyConfig,
    };

    #[test]
    fn test_runtime_router_config_from_gateway_round_robin() {
        let gateway = GatewayRouterConfig::default();
        let runtime = runtime_router_config_from_gateway(&gateway);
        assert_eq!(
            runtime.routing_strategy,
            super::super::config::RoutingStrategy::RoundRobin
        );
    }

    #[test]
    fn test_runtime_router_config_from_gateway_strategy_mapping() {
        let gateway = GatewayRouterConfig {
            strategy: RoutingStrategyConfig::LatencyBased,
            circuit_breaker: CircuitBreakerConfig::default(),
            load_balancer: LoadBalancerConfig::default(),
        };
        let runtime = runtime_router_config_from_gateway(&gateway);
        assert_eq!(
            runtime.routing_strategy,
            super::super::config::RoutingStrategy::LatencyBased
        );
    }

    #[test]
    fn test_runtime_router_config_from_gateway_circuit_breaker_mapping() {
        let gateway = GatewayRouterConfig {
            strategy: RoutingStrategyConfig::RoundRobin,
            circuit_breaker: CircuitBreakerConfig {
                failure_threshold: 8,
                recovery_timeout: 45,
                min_requests: 10,
                success_threshold: 3,
            },
            load_balancer: LoadBalancerConfig::default(),
        };
        let runtime = runtime_router_config_from_gateway(&gateway);
        assert_eq!(runtime.allowed_fails, 8);
        assert_eq!(runtime.cooldown_time_secs, 45);
    }
}
