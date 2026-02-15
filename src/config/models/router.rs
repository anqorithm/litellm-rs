//! Router configuration

use super::*;
use serde::{Deserialize, Serialize};

/// Gateway router configuration (YAML config model)
///
/// This is the YAML-deserialized router config for the gateway config file.
/// For the runtime router config used by the actual router, see
/// [`crate::core::router::config::RouterConfig`].
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GatewayRouterConfig {
    /// Routing strategy
    #[serde(default)]
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

/// Routing strategy configuration
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum RoutingStrategyConfig {
    /// Round-robin routing
    #[default]
    RoundRobin,
    /// Least latency routing
    LeastLatency,
    /// Least cost routing
    LeastCost,
    /// Random routing
    Random,
    /// Weighted routing
    Weighted {
        /// Provider weights
        weights: std::collections::HashMap<String, f64>,
    },
    /// Priority-based routing
    Priority {
        /// Provider priorities
        priorities: std::collections::HashMap<String, u32>,
    },
    /// A/B testing
    ABTest {
        /// Traffic split ratio
        split_ratio: f64,
    },
    /// Custom strategy
    Custom {
        /// Custom logic identifier
        logic: String,
    },
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

fn default_true() -> bool {
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    // ==================== RoutingStrategyConfig Tests ====================

    #[test]
    fn test_routing_strategy_default() {
        let strategy = RoutingStrategyConfig::default();
        matches!(strategy, RoutingStrategyConfig::RoundRobin);
    }

    #[test]
    fn test_routing_strategy_round_robin_serialization() {
        let strategy = RoutingStrategyConfig::RoundRobin;
        let json = serde_json::to_value(&strategy).unwrap();
        assert_eq!(json["type"], "round_robin");
    }

    #[test]
    fn test_routing_strategy_least_latency_serialization() {
        let strategy = RoutingStrategyConfig::LeastLatency;
        let json = serde_json::to_value(&strategy).unwrap();
        assert_eq!(json["type"], "least_latency");
    }

    #[test]
    fn test_routing_strategy_least_cost_serialization() {
        let strategy = RoutingStrategyConfig::LeastCost;
        let json = serde_json::to_value(&strategy).unwrap();
        assert_eq!(json["type"], "least_cost");
    }

    #[test]
    fn test_routing_strategy_random_serialization() {
        let strategy = RoutingStrategyConfig::Random;
        let json = serde_json::to_value(&strategy).unwrap();
        assert_eq!(json["type"], "random");
    }

    #[test]
    fn test_routing_strategy_weighted_serialization() {
        let mut weights = std::collections::HashMap::new();
        weights.insert("provider1".to_string(), 0.7);
        weights.insert("provider2".to_string(), 0.3);
        let strategy = RoutingStrategyConfig::Weighted { weights };
        let json = serde_json::to_value(&strategy).unwrap();
        assert_eq!(json["type"], "weighted");
        assert!(json["weights"].is_object());
    }

    #[test]
    fn test_routing_strategy_priority_serialization() {
        let mut priorities = std::collections::HashMap::new();
        priorities.insert("primary".to_string(), 1);
        priorities.insert("backup".to_string(), 2);
        let strategy = RoutingStrategyConfig::Priority { priorities };
        let json = serde_json::to_value(&strategy).unwrap();
        assert_eq!(json["type"], "priority");
    }

    #[test]
    fn test_routing_strategy_ab_test_serialization() {
        let strategy = RoutingStrategyConfig::ABTest { split_ratio: 0.5 };
        let json = serde_json::to_value(&strategy).unwrap();
        assert_eq!(json["type"], "a_b_test");
        assert!((json["split_ratio"].as_f64().unwrap() - 0.5).abs() < 0.001);
    }

    #[test]
    fn test_routing_strategy_custom_serialization() {
        let strategy = RoutingStrategyConfig::Custom {
            logic: "my_custom_logic".to_string(),
        };
        let json = serde_json::to_value(&strategy).unwrap();
        assert_eq!(json["type"], "custom");
        assert_eq!(json["logic"], "my_custom_logic");
    }

    #[test]
    fn test_routing_strategy_deserialization() {
        let json = r#"{"type": "least_latency"}"#;
        let strategy: RoutingStrategyConfig = serde_json::from_str(json).unwrap();
        matches!(strategy, RoutingStrategyConfig::LeastLatency);
    }

    #[test]
    fn test_routing_strategy_weighted_deserialization() {
        let json = r#"{"type": "weighted", "weights": {"a": 0.6, "b": 0.4}}"#;
        let strategy: RoutingStrategyConfig = serde_json::from_str(json).unwrap();
        match strategy {
            RoutingStrategyConfig::Weighted { weights } => {
                assert_eq!(weights.len(), 2);
            }
            _ => panic!("Expected Weighted variant"),
        }
    }

    #[test]
    fn test_routing_strategy_clone() {
        let strategy = RoutingStrategyConfig::LeastCost;
        let cloned = strategy.clone();
        matches!(cloned, RoutingStrategyConfig::LeastCost);
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
            strategy: RoutingStrategyConfig::LeastLatency,
            circuit_breaker: CircuitBreakerConfig::default(),
            load_balancer: LoadBalancerConfig::default(),
        };
        matches!(config.strategy, RoutingStrategyConfig::LeastLatency);
    }

    #[test]
    fn test_router_config_serialization() {
        let config = GatewayRouterConfig {
            strategy: RoutingStrategyConfig::LeastCost,
            circuit_breaker: CircuitBreakerConfig::default(),
            load_balancer: LoadBalancerConfig::default(),
        };
        let json = serde_json::to_value(&config).unwrap();
        assert_eq!(json["strategy"]["type"], "least_cost");
    }

    #[test]
    fn test_router_config_deserialization() {
        let json = r#"{
            "strategy": {"type": "random"},
            "circuit_breaker": {"failure_threshold": 5, "recovery_timeout": 120, "min_requests": 10, "success_threshold": 3},
            "load_balancer": {"health_check_enabled": true, "sticky_sessions": false, "session_timeout": 3600}
        }"#;
        let config: GatewayRouterConfig = serde_json::from_str(json).unwrap();
        matches!(config.strategy, RoutingStrategyConfig::Random);
    }

    #[test]
    fn test_router_config_merge() {
        let base = GatewayRouterConfig::default();
        let other = GatewayRouterConfig {
            strategy: RoutingStrategyConfig::LeastLatency,
            circuit_breaker: CircuitBreakerConfig::default(),
            load_balancer: LoadBalancerConfig::default(),
        };
        let merged = base.merge(other);
        matches!(merged.strategy, RoutingStrategyConfig::LeastLatency);
    }

    #[test]
    fn test_router_config_clone() {
        let config = GatewayRouterConfig::default();
        let cloned = config.clone();
        matches!(cloned.strategy, RoutingStrategyConfig::RoundRobin);
    }
}
