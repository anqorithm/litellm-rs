//! Router configuration

use super::*;
use serde::{Deserialize, Serialize};

/// Canonical routing strategy type shared with runtime router config.
pub type RoutingStrategyConfig = crate::core::router::config::RoutingStrategy;

/// Gateway router configuration (YAML config model)
///
/// This is the YAML-deserialized router config for the gateway config file.
/// For the runtime router config used by the actual router, see
/// [`crate::core::router::config::RouterConfig`].
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GatewayRouterConfig {
    /// Routing strategy
    #[serde(default = "default_gateway_routing_strategy")]
    pub strategy: RoutingStrategyConfig,
    /// Circuit breaker configuration
    #[serde(default)]
    pub circuit_breaker: CircuitBreakerConfig,
    /// Load balancer configuration
    #[serde(default)]
    pub load_balancer: LoadBalancerConfig,
}

impl GatewayRouterConfig {
    /// Merge router configurations
    pub fn merge(mut self, other: Self) -> Self {
        self.strategy = other.strategy;
        self.circuit_breaker = self.circuit_breaker.merge(other.circuit_breaker);
        self.load_balancer = self.load_balancer.merge(other.load_balancer);
        self
    }
}

impl Default for GatewayRouterConfig {
    fn default() -> Self {
        Self {
            strategy: default_gateway_routing_strategy(),
            circuit_breaker: CircuitBreakerConfig::default(),
            load_balancer: LoadBalancerConfig::default(),
        }
    }
}

fn default_gateway_routing_strategy() -> RoutingStrategyConfig {
    RoutingStrategyConfig::RoundRobin
}

/// Circuit breaker configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CircuitBreakerConfig {
    /// Failure threshold
    #[serde(default = "default_failure_threshold")]
    pub failure_threshold: u32,
    /// Recovery timeout in seconds
    #[serde(default = "default_recovery_timeout")]
    pub recovery_timeout: u64,
    /// Minimum requests before circuit breaker activates
    #[serde(default = "default_min_requests")]
    pub min_requests: u32,
    /// Success threshold for half-open state
    #[serde(default = "default_success_threshold")]
    pub success_threshold: u32,
}

impl Default for CircuitBreakerConfig {
    fn default() -> Self {
        Self {
            failure_threshold: default_failure_threshold(),
            recovery_timeout: default_recovery_timeout(),
            min_requests: default_min_requests(),
            success_threshold: 3,
        }
    }
}

impl CircuitBreakerConfig {
    /// Merge circuit breaker configurations
    pub fn merge(mut self, other: Self) -> Self {
        if other.failure_threshold != default_failure_threshold() {
            self.failure_threshold = other.failure_threshold;
        }
        if other.recovery_timeout != default_recovery_timeout() {
            self.recovery_timeout = other.recovery_timeout;
        }
        if other.min_requests != default_min_requests() {
            self.min_requests = other.min_requests;
        }
        if other.success_threshold != 3 {
            self.success_threshold = other.success_threshold;
        }
        self
    }
}

/// Load balancer configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoadBalancerConfig {
    /// Health check enabled
    #[serde(default = "default_true")]
    pub health_check_enabled: bool,
    /// Sticky sessions enabled
    #[serde(default)]
    pub sticky_sessions: bool,
    /// Session timeout in seconds
    #[serde(default = "default_session_timeout")]
    pub session_timeout: u64,
}

impl Default for LoadBalancerConfig {
    fn default() -> Self {
        Self {
            health_check_enabled: true,
            sticky_sessions: false,
            session_timeout: 3600,
        }
    }
}

impl LoadBalancerConfig {
    /// Merge load balancer configurations
    pub fn merge(mut self, other: Self) -> Self {
        if !other.health_check_enabled {
            self.health_check_enabled = other.health_check_enabled;
        }
        if other.sticky_sessions {
            self.sticky_sessions = other.sticky_sessions;
        }
        if other.session_timeout != 3600 {
            self.session_timeout = other.session_timeout;
        }
        self
    }
}

fn default_success_threshold() -> u32 {
    3
}

fn default_session_timeout() -> u64 {
    3600
}

#[cfg(test)]
mod tests {
    use super::*;

    // ==================== RoutingStrategyConfig Tests ====================

    #[test]
    fn test_routing_strategy_default() {
        assert_eq!(
            default_gateway_routing_strategy(),
            RoutingStrategyConfig::RoundRobin
        );
    }

    #[test]
    fn test_routing_strategy_round_robin_serialization() {
        let strategy = RoutingStrategyConfig::RoundRobin;
        let json = serde_json::to_value(strategy).unwrap();
        assert_eq!(json, serde_json::json!("round_robin"));
    }

    #[test]
    fn test_routing_strategy_latency_based_serialization() {
        let strategy = RoutingStrategyConfig::LatencyBased;
        let json = serde_json::to_value(strategy).unwrap();
        assert_eq!(json, serde_json::json!("latency_based"));
    }

    #[test]
    fn test_routing_strategy_priority_based_serialization() {
        let strategy = RoutingStrategyConfig::PriorityBased;
        let json = serde_json::to_value(strategy).unwrap();
        assert_eq!(json, serde_json::json!("priority_based"));
    }

    #[test]
    fn test_routing_strategy_simple_shuffle_serialization() {
        let strategy = RoutingStrategyConfig::SimpleShuffle;
        let json = serde_json::to_value(strategy).unwrap();
        assert_eq!(json, serde_json::json!("simple_shuffle"));
    }

    #[test]
    fn test_routing_strategy_deserialization() {
        let json = r#""latency_based""#;
        let strategy: RoutingStrategyConfig = serde_json::from_str(json).unwrap();
        assert_eq!(strategy, RoutingStrategyConfig::LatencyBased);
    }

    #[test]
    fn test_routing_strategy_clone() {
        let strategy = RoutingStrategyConfig::PriorityBased;
        let copied = strategy;
        assert_eq!(copied, RoutingStrategyConfig::PriorityBased);
    }

    // ==================== CircuitBreakerConfig Tests ====================

    #[test]
    fn test_circuit_breaker_config_default() {
        let config = CircuitBreakerConfig::default();
        assert_eq!(config.failure_threshold, 5);
        assert_eq!(config.recovery_timeout, 60);
        assert_eq!(config.min_requests, 10);
        assert_eq!(config.success_threshold, 3);
    }

    #[test]
    fn test_circuit_breaker_config_structure() {
        let config = CircuitBreakerConfig {
            failure_threshold: 5,
            recovery_timeout: 120,
            min_requests: 20,
            success_threshold: 5,
        };
        assert_eq!(config.failure_threshold, 5);
        assert_eq!(config.recovery_timeout, 120);
    }

    #[test]
    fn test_circuit_breaker_config_serialization() {
        let config = CircuitBreakerConfig {
            failure_threshold: 4,
            recovery_timeout: 90,
            min_requests: 15,
            success_threshold: 4,
        };
        let json = serde_json::to_value(&config).unwrap();
        assert_eq!(json["failure_threshold"], 4);
        assert_eq!(json["recovery_timeout"], 90);
    }

    #[test]
    fn test_circuit_breaker_config_deserialization() {
        let json = r#"{"failure_threshold": 2, "recovery_timeout": 30, "min_requests": 5, "success_threshold": 2}"#;
        let config: CircuitBreakerConfig = serde_json::from_str(json).unwrap();
        assert_eq!(config.failure_threshold, 2);
        assert_eq!(config.min_requests, 5);
    }

    #[test]
    fn test_circuit_breaker_config_merge() {
        let base = CircuitBreakerConfig::default();
        let other = CircuitBreakerConfig {
            failure_threshold: 5,
            recovery_timeout: 60,
            min_requests: 10,
            success_threshold: 3,
        };
        let merged = base.merge(other);
        assert_eq!(merged.failure_threshold, 5);
    }

    #[test]
    fn test_circuit_breaker_config_clone() {
        let config = CircuitBreakerConfig::default();
        let cloned = config.clone();
        assert_eq!(config.failure_threshold, cloned.failure_threshold);
    }

    // ==================== LoadBalancerConfig Tests ====================

    #[test]
    fn test_load_balancer_config_default() {
        let config = LoadBalancerConfig::default();
        assert!(config.health_check_enabled);
        assert!(!config.sticky_sessions);
        assert_eq!(config.session_timeout, 3600);
    }

    #[test]
    fn test_load_balancer_config_structure() {
        let config = LoadBalancerConfig {
            health_check_enabled: false,
            sticky_sessions: true,
            session_timeout: 7200,
        };
        assert!(!config.health_check_enabled);
        assert!(config.sticky_sessions);
        assert_eq!(config.session_timeout, 7200);
    }

    #[test]
    fn test_load_balancer_config_serialization() {
        let config = LoadBalancerConfig {
            health_check_enabled: true,
            sticky_sessions: true,
            session_timeout: 1800,
        };
        let json = serde_json::to_value(&config).unwrap();
        assert_eq!(json["health_check_enabled"], true);
        assert_eq!(json["sticky_sessions"], true);
        assert_eq!(json["session_timeout"], 1800);
    }

    #[test]
    fn test_load_balancer_config_deserialization() {
        let json =
            r#"{"health_check_enabled": false, "sticky_sessions": true, "session_timeout": 900}"#;
        let config: LoadBalancerConfig = serde_json::from_str(json).unwrap();
        assert!(!config.health_check_enabled);
        assert!(config.sticky_sessions);
    }

    #[test]
    fn test_load_balancer_config_merge() {
        let base = LoadBalancerConfig::default();
        let other = LoadBalancerConfig {
            health_check_enabled: true,
            sticky_sessions: true,
            session_timeout: 3600,
        };
        let merged = base.merge(other);
        assert!(merged.sticky_sessions);
    }

    #[test]
    fn test_load_balancer_config_clone() {
        let config = LoadBalancerConfig::default();
        let cloned = config.clone();
        assert_eq!(config.health_check_enabled, cloned.health_check_enabled);
    }

    // ==================== GatewayRouterConfig Tests ====================

    #[test]
    fn test_router_config_default() {
        let config = GatewayRouterConfig::default();
        matches!(config.strategy, RoutingStrategyConfig::RoundRobin);
    }

    #[test]
    fn test_router_config_structure() {
        let config = GatewayRouterConfig {
            strategy: RoutingStrategyConfig::LatencyBased,
            circuit_breaker: CircuitBreakerConfig::default(),
            load_balancer: LoadBalancerConfig::default(),
        };
        assert_eq!(config.strategy, RoutingStrategyConfig::LatencyBased);
    }

    #[test]
    fn test_router_config_serialization() {
        let config = GatewayRouterConfig {
            strategy: RoutingStrategyConfig::PriorityBased,
            circuit_breaker: CircuitBreakerConfig::default(),
            load_balancer: LoadBalancerConfig::default(),
        };
        let json = serde_json::to_value(&config).unwrap();
        assert_eq!(json["strategy"], "priority_based");
    }

    #[test]
    fn test_router_config_deserialization() {
        let json = r#"{
            "strategy": "simple_shuffle",
            "circuit_breaker": {"failure_threshold": 5, "recovery_timeout": 120, "min_requests": 10, "success_threshold": 3},
            "load_balancer": {"health_check_enabled": true, "sticky_sessions": false, "session_timeout": 3600}
        }"#;
        let config: GatewayRouterConfig = serde_json::from_str(json).unwrap();
        assert_eq!(config.strategy, RoutingStrategyConfig::SimpleShuffle);
    }

    #[test]
    fn test_router_config_merge() {
        let base = GatewayRouterConfig::default();
        let other = GatewayRouterConfig {
            strategy: RoutingStrategyConfig::LatencyBased,
            circuit_breaker: CircuitBreakerConfig::default(),
            load_balancer: LoadBalancerConfig::default(),
        };
        let merged = base.merge(other);
        assert_eq!(merged.strategy, RoutingStrategyConfig::LatencyBased);
    }

    #[test]
    fn test_router_config_clone() {
        let config = GatewayRouterConfig::default();
        let cloned = config.clone();
        matches!(cloned.strategy, RoutingStrategyConfig::RoundRobin);
    }
}
