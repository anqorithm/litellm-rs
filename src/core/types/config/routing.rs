//! Routing configuration types

use super::defaults::*;
use super::health::HealthCheckConfig;
use serde::{Deserialize, Serialize};

/// Routing configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutingConfig {
    /// Routing strategy
    pub strategy: RoutingStrategyConfig,
    /// Health check configuration
    pub health_check: HealthCheckConfig,
    /// Circuit breaker configuration
    pub circuit_breaker: CircuitBreakerConfig,
    /// Load balancer configuration
    #[serde(skip_serializing_if = "Option::is_none")]
    pub load_balancer: Option<LoadBalancerConfig>,
}

/// Routing strategy configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum RoutingStrategyConfig {
    /// Round robin strategy
    #[serde(rename = "round_robin")]
    RoundRobin,
    /// Least load strategy
    #[serde(rename = "least_loaded")]
    LeastLoaded,
    /// Cost optimization strategy
    #[serde(rename = "cost_optimized")]
    CostOptimized { performance_weight: f32 },
    /// Latency optimization strategy
    #[serde(rename = "latency_based")]
    LatencyBased { latency_threshold_ms: u64 },
    /// Tag-based routing strategy
    #[serde(rename = "tag_based")]
    TagBased { selectors: Vec<TagSelector> },
    /// Custom strategy
    #[serde(rename = "custom")]
    Custom {
        class: String,
        config: serde_json::Value,
    },
}

/// Tag selector
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TagSelector {
    /// Tag key
    pub key: String,
    /// Tag value (supports wildcards)
    pub value: String,
    /// Operator
    #[serde(default)]
    pub operator: TagOperator,
}

/// Tag operator
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TagOperator {
    #[default]
    Eq,
    Ne,
    In,
    NotIn,
    Exists,
    NotExists,
}

/// Circuit breaker configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CircuitBreakerConfig {
    /// Failure threshold
    #[serde(default = "default_failure_threshold")]
    pub failure_threshold: u32,
    /// Recovery timeout (seconds)
    #[serde(default = "default_recovery_timeout")]
    pub recovery_timeout_seconds: u64,
    /// Half-open max requests
    #[serde(default = "default_half_open_requests")]
    pub half_open_max_requests: u32,
    /// Enabled
    #[serde(default = "default_true")]
    pub enabled: bool,
}

impl Default for CircuitBreakerConfig {
    fn default() -> Self {
        Self {
            failure_threshold: default_failure_threshold(),
            recovery_timeout_seconds: default_recovery_timeout(),
            half_open_max_requests: default_half_open_requests(),
            enabled: true,
        }
    }
}

/// Load balancer configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoadBalancerConfig {
    /// Algorithm type
    pub algorithm: LoadBalancerAlgorithm,
    /// Session affinity configuration
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_affinity: Option<SessionAffinityConfig>,
}

/// Load balancer algorithm
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LoadBalancerAlgorithm {
    RoundRobin,
    WeightedRoundRobin,
    LeastConnections,
    ConsistentHash,
}

/// Session affinity configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionAffinityConfig {
    /// Affinity type
    pub affinity_type: SessionAffinityType,
    /// Timeout (seconds)
    #[serde(default = "default_session_timeout")]
    pub timeout_seconds: u64,
}

/// Session affinity type
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SessionAffinityType {
    ClientIp,
    UserId,
    CustomHeader { header_name: String },
}

#[cfg(test)]
#[path = "routing_tests.rs"]
mod tests;
