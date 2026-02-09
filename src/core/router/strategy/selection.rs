//! Provider selection methods for different routing strategies

use super::types::RoutingData;
use crate::core::types::context::RequestContext;
use crate::utils::error::error::{GatewayError, Result};
use parking_lot::RwLock;
use std::sync::atomic::{AtomicUsize, Ordering};
use tracing::debug;

/// Selection methods for strategy executor
pub(super) struct SelectionMethods;

impl SelectionMethods {
    /// Round-robin provider selection
    pub fn select_round_robin(providers: &[String], counter: &AtomicUsize) -> Result<String> {
        if providers.is_empty() {
            return Err(GatewayError::Validation(
                "No providers available for round-robin selection".to_string(),
            ));
        }
        let index = counter.fetch_add(1, Ordering::Relaxed) % providers.len();
        debug!(
            "Round-robin selected provider at index {}: {}",
            index, providers[index]
        );
        Ok(providers[index].clone())
    }

    /// Select provider with least latency
    pub fn select_least_latency(
        providers: &[String],
        routing_data: &RwLock<RoutingData>,
    ) -> Result<String> {
        if providers.is_empty() {
            return Err(GatewayError::Validation(
                "No providers available for latency-based selection".to_string(),
            ));
        }
        let data = routing_data.read();

        let mut best_provider = &providers[0];
        let mut best_latency = f64::MAX;

        for provider in providers {
            let latency = data.latencies.get(provider).copied().unwrap_or(f64::MAX);
            if latency < best_latency {
                best_latency = latency;
                best_provider = provider;
            }
        }

        debug!(
            "Least latency selected provider: {} ({}ms)",
            best_provider, best_latency
        );
        Ok(best_provider.clone())
    }

    /// Select provider with least cost
    pub fn select_least_cost(
        providers: &[String],
        model: &str,
        routing_data: &RwLock<RoutingData>,
    ) -> Result<String> {
        if providers.is_empty() {
            return Err(GatewayError::Validation(
                "No providers available for cost-based selection".to_string(),
            ));
        }
        let data = routing_data.read();

        let mut best_provider = &providers[0];
        let mut best_cost = f64::MAX;

        // Pre-allocate buffer for cost key to avoid repeated allocations in loop
        let mut cost_key = String::with_capacity(64);
        for provider in providers {
            cost_key.clear();
            cost_key.push_str(provider);
            cost_key.push(':');
            cost_key.push_str(model);

            let cost = data.costs.get(&cost_key).copied().unwrap_or(f64::MAX);
            if cost < best_cost {
                best_cost = cost;
                best_provider = provider;
            }
        }

        debug!(
            "Least cost selected provider: {} (${:.4})",
            best_provider, best_cost
        );
        Ok(best_provider.clone())
    }

    /// Random provider selection
    pub fn select_random(providers: &[String]) -> Result<String> {
        if providers.is_empty() {
            return Err(GatewayError::Validation(
                "No providers available for random selection".to_string(),
            ));
        }
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let index = rng.gen_range(0..providers.len());
        debug!(
            "Random selected provider at index {}: {}",
            index, providers[index]
        );
        Ok(providers[index].clone())
    }

    /// Weighted provider selection
    pub fn select_weighted(
        providers: &[String],
        routing_data: &RwLock<RoutingData>,
        counter: &AtomicUsize,
    ) -> Result<String> {
        if providers.is_empty() {
            return Err(GatewayError::Validation(
                "No providers available for weighted selection".to_string(),
            ));
        }
        // Collect weights and calculate total within lock scope
        let (total_weight, weights): (f64, Vec<f64>) = {
            let data = routing_data.read();
            let mut weights = Vec::with_capacity(providers.len());
            let mut total = 0.0;

            for provider in providers {
                let weight = data.weights.get(provider).copied().unwrap_or(1.0);
                total += weight;
                weights.push(weight);
            }

            (total, weights)
        }; // Lock released here

        if total_weight <= 0.0 {
            return Self::select_round_robin(providers, counter);
        }

        // Generate random number
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let mut random = rng.gen_range(0.0..1.0) * total_weight;

        // Select provider based on weight
        for (idx, weight) in weights.iter().enumerate() {
            random -= weight;
            if random <= 0.0 {
                let provider = &providers[idx];
                debug!(
                    "Weighted selected provider: {} (weight: {})",
                    provider, weight
                );
                return Ok(provider.clone());
            }
        }

        // Fallback to first provider
        Ok(providers[0].clone())
    }

    /// Priority-based provider selection
    pub fn select_priority(
        providers: &[String],
        routing_data: &RwLock<RoutingData>,
    ) -> Result<String> {
        if providers.is_empty() {
            return Err(GatewayError::Validation(
                "No providers available for priority-based selection".to_string(),
            ));
        }
        let data = routing_data.read();

        let mut best_provider = &providers[0];
        let mut best_priority = 0u32;

        for provider in providers {
            let priority = data.priorities.get(provider).copied().unwrap_or(0);
            if priority > best_priority {
                best_priority = priority;
                best_provider = provider;
            }
        }

        debug!(
            "Priority selected provider: {} (priority: {})",
            best_provider, best_priority
        );
        Ok(best_provider.clone())
    }

    /// A/B test provider selection
    pub fn select_ab_test(providers: &[String], split_ratio: f64) -> Result<String> {
        if providers.is_empty() {
            return Err(GatewayError::Validation(
                "No providers available for A/B test selection".to_string(),
            ));
        }
        if providers.len() < 2 {
            return Ok(providers[0].clone());
        }

        use rand::Rng;
        let mut rng = rand::thread_rng();
        let random = rng.gen_range(0.0..1.0);

        let selected = if random < split_ratio {
            &providers[0]
        } else {
            &providers[1]
        };

        debug!(
            "A/B test selected provider: {} (ratio: {}, random: {})",
            selected, split_ratio, random
        );
        Ok(selected.clone())
    }

    /// Custom strategy selection
    pub fn select_custom(
        providers: &[String],
        logic: &str,
        context: &RequestContext,
        counter: &AtomicUsize,
    ) -> Result<String> {
        if providers.is_empty() {
            return Err(GatewayError::Validation(
                "No providers available for custom selection".to_string(),
            ));
        }

        let logic = logic.trim();
        if logic.is_empty() {
            return Self::select_round_robin(providers, counter);
        }

        // Simple form: direct provider name
        if !logic.contains("->") {
            if let Some(selected) = select_named_provider(providers, logic) {
                debug!("Custom selected provider by name: {}", selected);
                return Ok(selected);
            }
            return Self::select_round_robin(providers, counter);
        }

        // Rule-based form: "key=value->provider; ...; default->provider"
        let mut default_provider: Option<String> = None;
        for rule in logic.split(';') {
            let rule = rule.trim();
            if rule.is_empty() {
                continue;
            }

            let Some((condition, target)) = rule.split_once("->") else {
                continue;
            };
            let condition = condition.trim();
            let target = target.trim();
            if target.is_empty() {
                continue;
            }

            if condition.eq_ignore_ascii_case("default") {
                default_provider = select_named_provider(providers, target);
                continue;
            }

            let Some((key, value)) = condition.split_once('=') else {
                continue;
            };
            if matches_custom_condition(key.trim(), value.trim(), context) {
                if let Some(selected) = select_named_provider(providers, target) {
                    debug!(
                        "Custom selected provider: {} (rule: {} -> {})",
                        selected, condition, target
                    );
                    return Ok(selected);
                }
            }
        }

        if let Some(selected) = default_provider {
            debug!("Custom selected provider via default rule: {}", selected);
            return Ok(selected);
        }

        Self::select_round_robin(providers, counter)
    }

    /// Usage-based provider selection (lowest TPM/RPM usage)
    pub fn select_usage_based(
        providers: &[String],
        routing_data: &RwLock<RoutingData>,
    ) -> Result<String> {
        if providers.is_empty() {
            return Err(GatewayError::Validation(
                "No providers available for usage-based selection".to_string(),
            ));
        }
        let data = routing_data.read();

        let mut best_provider = &providers[0];
        let mut best_usage_pct = f64::MAX;

        for provider in providers {
            let usage_pct = data
                .usage
                .get(provider)
                .map(|u| u.usage_percentage())
                .unwrap_or(0.0); // No usage data = 0% usage

            if usage_pct < best_usage_pct {
                best_usage_pct = usage_pct;
                best_provider = provider;
            }
        }

        debug!(
            "Usage-based selected provider: {} (usage: {:.1}%)",
            best_provider,
            best_usage_pct * 100.0
        );
        Ok(best_provider.clone())
    }

    /// Least-busy provider selection (fewest active requests)
    pub fn select_least_busy(
        providers: &[String],
        routing_data: &RwLock<RoutingData>,
    ) -> Result<String> {
        if providers.is_empty() {
            return Err(GatewayError::Validation(
                "No providers available for least-busy selection".to_string(),
            ));
        }
        let data = routing_data.read();

        let mut best_provider = &providers[0];
        let mut least_active = usize::MAX;

        for provider in providers {
            let active = data
                .usage
                .get(provider)
                .map(|u| u.active_requests)
                .unwrap_or(0); // No usage data = 0 active requests

            if active < least_active {
                least_active = active;
                best_provider = provider;
            }
        }

        debug!(
            "Least-busy selected provider: {} (active requests: {})",
            best_provider, least_active
        );
        Ok(best_provider.clone())
    }
}

fn select_named_provider(providers: &[String], name: &str) -> Option<String> {
    providers.iter().find(|p| p.as_str() == name).cloned()
}

fn matches_custom_condition(key: &str, value: &str, context: &RequestContext) -> bool {
    if let Some(header_key) = key.strip_prefix("header:") {
        return context
            .headers
            .iter()
            .find(|(k, _)| k.eq_ignore_ascii_case(header_key))
            .map(|(_, v)| v == value)
            .unwrap_or(false);
    }

    if let Some(meta_key) = key.strip_prefix("meta:") {
        return context
            .metadata
            .get(meta_key)
            .map(|v| match v {
                serde_json::Value::String(s) => s == value,
                serde_json::Value::Number(n) => n.to_string() == value,
                serde_json::Value::Bool(b) => b.to_string() == value,
                _ => false,
            })
            .unwrap_or(false);
    }

    match key {
        "user_id" => context.user_id.as_deref() == Some(value),
        "client_ip" => context.client_ip.as_deref() == Some(value),
        "request_id" => context.request_id == value,
        "trace_id" => context.trace_id.as_deref() == Some(value),
        "span_id" => context.span_id.as_deref() == Some(value),
        _ => false,
    }
}

// ==================== Unit Tests ====================

#[cfg(test)]
#[path = "selection_tests.rs"]
mod tests;
