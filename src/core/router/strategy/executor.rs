//! Strategy executor for provider selection

use super::selection::SelectionMethods;
use super::types::{ProviderUsage, RoutingData, RoutingStrategy};
use crate::core::types::RequestContext;
use crate::utils::error::{GatewayError, Result};
use parking_lot::RwLock;
use std::sync::atomic::AtomicUsize;
use tracing::{debug, info};

/// Strategy executor for provider selection
pub struct StrategyExecutor {
    /// Current strategy
    strategy: RoutingStrategy,
    /// Round-robin counter
    round_robin_counter: AtomicUsize,
    /// Consolidated routing data - single lock for all related data
    /// This reduces lock contention when multiple strategies are used
    routing_data: RwLock<RoutingData>,
}

impl StrategyExecutor {
    /// Create a new strategy executor
    pub async fn new(strategy: RoutingStrategy) -> Result<Self> {
        info!("Creating strategy executor with strategy: {:?}", strategy);

        Ok(Self {
            strategy,
            round_robin_counter: AtomicUsize::new(0),
            routing_data: RwLock::new(RoutingData::default()),
        })
    }

    /// Select a provider based on the current strategy
    pub async fn select_provider(
        &self,
        available_providers: &[String],
        model: &str,
        context: &RequestContext,
    ) -> Result<String> {
        if available_providers.is_empty() {
            return Err(GatewayError::NoProvidersAvailable(
                "No providers available".to_string(),
            ));
        }

        match &self.strategy {
            RoutingStrategy::RoundRobin => {
                SelectionMethods::select_round_robin(available_providers, &self.round_robin_counter)
            }
            RoutingStrategy::LeastLatency => {
                SelectionMethods::select_least_latency(available_providers, &self.routing_data)
            }
            RoutingStrategy::LeastCost => {
                SelectionMethods::select_least_cost(available_providers, model, &self.routing_data)
            }
            RoutingStrategy::Random => SelectionMethods::select_random(available_providers),
            RoutingStrategy::Weighted => SelectionMethods::select_weighted(
                available_providers,
                &self.routing_data,
                &self.round_robin_counter,
            ),
            RoutingStrategy::Priority => {
                SelectionMethods::select_priority(available_providers, &self.routing_data)
            }
            RoutingStrategy::ABTest { split_ratio } => {
                SelectionMethods::select_ab_test(available_providers, *split_ratio)
            }
            RoutingStrategy::UsageBased => {
                SelectionMethods::select_usage_based(available_providers, &self.routing_data)
            }
            RoutingStrategy::LeastBusy => {
                SelectionMethods::select_least_busy(available_providers, &self.routing_data)
            }
            RoutingStrategy::Custom(logic) => SelectionMethods::select_custom(
                available_providers,
                logic,
                context,
                &self.round_robin_counter,
            ),
        }
    }

    /// Update provider weight
    pub async fn update_weight(&self, provider: &str, weight: f64) -> Result<()> {
        self.routing_data
            .write()
            .weights
            .insert(provider.to_string(), weight);
        debug!("Updated weight for provider {}: {}", provider, weight);
        Ok(())
    }

    /// Update provider latency
    pub async fn update_latency(&self, provider: &str, latency: f64) -> Result<()> {
        self.routing_data
            .write()
            .latencies
            .insert(provider.to_string(), latency);
        debug!("Updated latency for provider {}: {}ms", provider, latency);
        Ok(())
    }

    /// Update provider cost
    pub async fn update_cost(&self, provider: &str, model: &str, cost: f64) -> Result<()> {
        let key = format!("{}:{}", provider, model);
        self.routing_data.write().costs.insert(key, cost);
        debug!(
            "Updated cost for provider {} model {}: ${:.4}",
            provider, model, cost
        );
        Ok(())
    }

    /// Update provider priority
    pub async fn update_priority(&self, provider: &str, priority: u32) -> Result<()> {
        self.routing_data
            .write()
            .priorities
            .insert(provider.to_string(), priority);
        debug!("Updated priority for provider {}: {}", provider, priority);
        Ok(())
    }

    /// Update provider usage metrics
    pub async fn update_usage(&self, provider: &str, usage: ProviderUsage) -> Result<()> {
        let (tpm, rpm, active) = (usage.tpm, usage.rpm, usage.active_requests);
        self.routing_data
            .write()
            .usage
            .insert(provider.to_string(), usage);
        debug!(
            "Updated usage for provider {}: TPM={}, RPM={}, active={}",
            provider, tpm, rpm, active
        );
        Ok(())
    }

    /// Increment active request count for a provider
    pub async fn increment_active_requests(&self, provider: &str) -> Result<()> {
        let mut data = self.routing_data.write();
        let usage = data.usage.entry(provider.to_string()).or_default();
        usage.active_requests += 1;
        debug!(
            "Incremented active requests for {}: now {}",
            provider, usage.active_requests
        );
        Ok(())
    }

    /// Decrement active request count for a provider
    pub async fn decrement_active_requests(&self, provider: &str) -> Result<()> {
        let mut data = self.routing_data.write();
        if let Some(usage) = data.usage.get_mut(provider) {
            usage.active_requests = usage.active_requests.saturating_sub(1);
            debug!(
                "Decremented active requests for {}: now {}",
                provider, usage.active_requests
            );
        }
        Ok(())
    }

    /// Record token usage for a provider (updates TPM tracking)
    pub async fn record_token_usage(&self, provider: &str, tokens: u64) -> Result<()> {
        let mut data = self.routing_data.write();
        let usage = data.usage.entry(provider.to_string()).or_default();
        usage.tpm += tokens;
        usage.rpm += 1;
        debug!(
            "Recorded token usage for {}: +{} tokens (TPM: {}, RPM: {})",
            provider, tokens, usage.tpm, usage.rpm
        );
        Ok(())
    }

    /// Set rate limits for a provider
    pub async fn set_rate_limits(
        &self,
        provider: &str,
        tpm_limit: Option<u64>,
        rpm_limit: Option<u64>,
    ) -> Result<()> {
        let mut data = self.routing_data.write();
        let usage = data.usage.entry(provider.to_string()).or_default();
        usage.tpm_limit = tpm_limit;
        usage.rpm_limit = rpm_limit;
        debug!(
            "Set rate limits for {}: TPM={:?}, RPM={:?}",
            provider, tpm_limit, rpm_limit
        );
        Ok(())
    }

    /// Reset usage counters (typically called at the start of each minute)
    pub async fn reset_usage_counters(&self) -> Result<()> {
        let mut data = self.routing_data.write();
        for usage in data.usage.values_mut() {
            usage.tpm = 0;
            usage.rpm = 0;
        }
        debug!("Reset usage counters for all providers");
        Ok(())
    }

    /// Get current usage for a provider
    pub async fn get_usage(&self, provider: &str) -> Option<ProviderUsage> {
        self.routing_data.read().usage.get(provider).cloned()
    }
}
