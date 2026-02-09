//! Routing strategy implementations
//!
//! This module contains the implementation of 7 routing strategies
//! for selecting deployments.

use super::deployment::{Deployment, DeploymentId};
use dashmap::DashMap;
use rand::Rng;
use std::sync::atomic::{AtomicUsize, Ordering::Relaxed};

/// Trait for routing strategy selection
pub trait StrategySelector {
    /// Select a deployment from candidates using weighted random selection
    fn select_weighted_random(
        &self,
        candidate_ids: &[DeploymentId],
        deployments: &DashMap<DeploymentId, Deployment>,
    ) -> Option<DeploymentId>;

    /// Select a deployment with fewest active requests
    fn select_least_busy(
        &self,
        candidate_ids: &[DeploymentId],
        deployments: &DashMap<DeploymentId, Deployment>,
    ) -> Option<DeploymentId>;

    /// Select a deployment with lowest TPM usage rate
    fn select_lowest_usage(
        &self,
        candidate_ids: &[DeploymentId],
        deployments: &DashMap<DeploymentId, Deployment>,
    ) -> Option<DeploymentId>;

    /// Select a deployment with lowest average latency
    fn select_lowest_latency(
        &self,
        candidate_ids: &[DeploymentId],
        deployments: &DashMap<DeploymentId, Deployment>,
    ) -> Option<DeploymentId>;

    /// Select a deployment with lowest cost (priority)
    fn select_lowest_cost(
        &self,
        candidate_ids: &[DeploymentId],
        deployments: &DashMap<DeploymentId, Deployment>,
    ) -> Option<DeploymentId>;

    /// Select a deployment furthest from rate limits
    fn select_rate_limit_aware(
        &self,
        candidate_ids: &[DeploymentId],
        deployments: &DashMap<DeploymentId, Deployment>,
    ) -> Option<DeploymentId>;

    /// Select a deployment using round-robin
    fn select_round_robin(
        &self,
        model_name: &str,
        candidate_ids: &[DeploymentId],
        round_robin_counters: &DashMap<String, AtomicUsize>,
    ) -> Option<DeploymentId>;
}

/// Weighted random selection (SimpleShuffle)
///
/// Selects a deployment randomly based on weights.
/// Higher weight = higher probability of selection.
/// Returns None if candidate_ids is empty.
pub fn weighted_random(
    candidate_ids: &[DeploymentId],
    deployments: &DashMap<DeploymentId, Deployment>,
) -> Option<DeploymentId> {
    if candidate_ids.is_empty() {
        return None;
    }

    if candidate_ids.len() == 1 {
        return Some(candidate_ids[0].clone());
    }

    // Calculate total weight
    let total_weight: u32 = candidate_ids
        .iter()
        .filter_map(|id| deployments.get(id.as_str()).map(|d| d.config.weight))
        .sum();

    if total_weight == 0 {
        // All weights are 0, fall back to uniform random
        let mut rng = rand::thread_rng();
        let index = rng.gen_range(0..candidate_ids.len());
        return Some(candidate_ids[index].clone());
    }

    // Generate random point in [0, total_weight)
    let mut rng = rand::thread_rng();
    let mut point = rng.gen_range(0..total_weight);

    // Find the deployment corresponding to this point
    for id in candidate_ids {
        if let Some(deployment) = deployments.get(id.as_str()) {
            let weight = deployment.config.weight;
            if point < weight {
                return Some(id.clone());
            }
            point -= weight;
        }
    }

    // Fallback (shouldn't happen)
    Some(candidate_ids[0].clone())
}

/// Select deployment with fewest active requests (LeastBusy)
///
/// Chooses the deployment with the lowest number of currently active requests.
/// In case of tie, selects randomly among tied deployments.
/// Returns None if candidate_ids is empty.
pub fn least_busy(
    candidate_ids: &[DeploymentId],
    deployments: &DashMap<DeploymentId, Deployment>,
) -> Option<DeploymentId> {
    if candidate_ids.is_empty() {
        return None;
    }

    let min_active = candidate_ids
        .iter()
        .filter_map(|id| {
            deployments
                .get(id.as_str())
                .map(|d| d.state.active_requests.load(Relaxed))
        })
        .min()
        .unwrap_or(0);

    // Collect all deployments with min active requests
    let tied: Vec<_> = candidate_ids
        .iter()
        .filter(|id| {
            deployments
                .get(id.as_str())
                .map(|d| d.state.active_requests.load(Relaxed) == min_active)
                .unwrap_or(false)
        })
        .collect();

    if tied.is_empty() {
        return Some(candidate_ids[0].clone());
    }

    // Random selection among tied
    if tied.len() == 1 {
        Some(tied[0].clone())
    } else {
        let mut rng = rand::thread_rng();
        let index = rng.gen_range(0..tied.len());
        Some(tied[index].clone())
    }
}

/// Select deployment with lowest TPM usage rate (UsageBased)
///
/// Calculates TPM usage as: (tpm_current / tpm_limit) * 100
/// Deployments without limits are considered at 0% usage.
/// Returns None if candidate_ids is empty.
pub fn lowest_usage(
    candidate_ids: &[DeploymentId],
    deployments: &DashMap<DeploymentId, Deployment>,
) -> Option<DeploymentId> {
    if candidate_ids.is_empty() {
        return None;
    }

    let mut best_id = &candidate_ids[0];
    let mut best_usage_pct = u64::MAX;

    for id in candidate_ids {
        if let Some(deployment) = deployments.get(id.as_str()) {
            let tpm_current = deployment.state.tpm_current.load(Relaxed);
            let usage_pct = match deployment.config.tpm_limit {
                Some(limit) if limit > 0 => (tpm_current * 100) / limit,
                _ => 0, // No limit = 0% usage
            };

            if usage_pct < best_usage_pct {
                best_usage_pct = usage_pct;
                best_id = id;
            }
        }
    }

    Some(best_id.clone())
}

/// Select deployment with lowest average latency (LatencyBased)
///
/// Selects the deployment with the lowest average latency.
/// New deployments (latency = 0) are given a chance by treating them
/// as having average latency.
/// Returns None if candidate_ids is empty.
pub fn lowest_latency(
    candidate_ids: &[DeploymentId],
    deployments: &DashMap<DeploymentId, Deployment>,
) -> Option<DeploymentId> {
    if candidate_ids.is_empty() {
        return None;
    }

    // Calculate average latency across all candidates (for new deployments)
    let latencies: Vec<u64> = candidate_ids
        .iter()
        .filter_map(|id| {
            deployments
                .get(id.as_str())
                .map(|d| d.state.avg_latency_us.load(Relaxed))
        })
        .filter(|&lat| lat > 0)
        .collect();

    let avg_latency = if !latencies.is_empty() {
        latencies.iter().sum::<u64>() / latencies.len() as u64
    } else {
        0
    };

    let mut best_id = &candidate_ids[0];
    let mut best_latency = u64::MAX;

    for id in candidate_ids {
        if let Some(deployment) = deployments.get(id.as_str()) {
            let mut latency = deployment.state.avg_latency_us.load(Relaxed);

            // Treat new deployments (latency = 0) as having average latency
            if latency == 0 {
                latency = avg_latency;
            }

            if latency < best_latency {
                best_latency = latency;
                best_id = id;
            }
        }
    }

    Some(best_id.clone())
}

/// Select deployment with lowest cost (CostBased)
///
/// Currently uses priority as a cost proxy (lower priority = lower cost).
/// Returns None if candidate_ids is empty.
pub fn lowest_cost(
    candidate_ids: &[DeploymentId],
    deployments: &DashMap<DeploymentId, Deployment>,
) -> Option<DeploymentId> {
    if candidate_ids.is_empty() {
        return None;
    }

    let mut best_id = &candidate_ids[0];
    let mut best_priority = u32::MAX;

    for id in candidate_ids {
        if let Some(deployment) = deployments.get(id.as_str()) {
            let priority = deployment.config.priority;
            if priority < best_priority {
                best_priority = priority;
                best_id = id;
            }
        }
    }

    Some(best_id.clone())
}

/// Select deployment that is furthest from rate limits (RateLimitAware)
///
/// Calculates distance from rate limit as: (limit - current) / limit
/// Selects the deployment with maximum distance (most headroom).
/// Returns None if candidate_ids is empty.
pub fn rate_limit_aware(
    candidate_ids: &[DeploymentId],
    deployments: &DashMap<DeploymentId, Deployment>,
) -> Option<DeploymentId> {
    if candidate_ids.is_empty() {
        return None;
    }

    let mut best_id = &candidate_ids[0];
    let mut best_distance: f64 = -1.0;

    for id in candidate_ids {
        if let Some(deployment) = deployments.get(id.as_str()) {
            // Calculate TPM distance
            let tpm_distance = match deployment.config.tpm_limit {
                Some(limit) if limit > 0 => {
                    let current = deployment.state.tpm_current.load(Relaxed);
                    let remaining = limit.saturating_sub(current);
                    remaining as f64 / limit as f64
                }
                _ => 1.0, // No limit = maximum distance
            };

            // Calculate RPM distance
            let rpm_distance = match deployment.config.rpm_limit {
                Some(limit) if limit > 0 => {
                    let current = deployment.state.rpm_current.load(Relaxed);
                    let remaining = limit.saturating_sub(current);
                    remaining as f64 / limit as f64
                }
                _ => 1.0, // No limit = maximum distance
            };

            // Use minimum of TPM and RPM distance (most constrained)
            let distance = tpm_distance.min(rpm_distance);

            if distance > best_distance {
                best_distance = distance;
                best_id = id;
            }
        }
    }

    Some(best_id.clone())
}

/// Round-robin selection (RoundRobin)
///
/// Cycles through deployments in order, using a per-model counter.
/// Returns None if candidate_ids is empty.
pub fn round_robin(
    model_name: &str,
    candidate_ids: &[DeploymentId],
    round_robin_counters: &DashMap<String, AtomicUsize>,
) -> Option<DeploymentId> {
    if candidate_ids.is_empty() {
        return None;
    }

    if candidate_ids.len() == 1 {
        return Some(candidate_ids[0].clone());
    }

    // Get or create counter for this model
    let counter = round_robin_counters
        .entry(model_name.to_string())
        .or_insert_with(|| AtomicUsize::new(0));

    // Fetch and increment counter
    let index = counter.fetch_add(1, Relaxed) % candidate_ids.len();

    Some(candidate_ids[index].clone())
}

// ====================================================================================
// TESTS
// ====================================================================================

#[cfg(test)]
#[path = "strategy_impl_tests.rs"]
mod tests;
