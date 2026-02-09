use super::*;

// ==================== RoutingStrategyConfig Tests ====================

#[test]
fn test_routing_strategy_round_robin_serialization() {
    let strategy = RoutingStrategyConfig::RoundRobin;
    let json = serde_json::to_string(&strategy).unwrap();
    assert!(json.contains("round_robin"));
}

#[test]
fn test_routing_strategy_round_robin_deserialization() {
    let json = r#"{"type": "round_robin"}"#;
    let strategy: RoutingStrategyConfig = serde_json::from_str(json).unwrap();
    assert!(matches!(strategy, RoutingStrategyConfig::RoundRobin));
}

#[test]
fn test_routing_strategy_least_loaded_serialization() {
    let strategy = RoutingStrategyConfig::LeastLoaded;
    let json = serde_json::to_string(&strategy).unwrap();
    assert!(json.contains("least_loaded"));
}

#[test]
fn test_routing_strategy_least_loaded_deserialization() {
    let json = r#"{"type": "least_loaded"}"#;
    let strategy: RoutingStrategyConfig = serde_json::from_str(json).unwrap();
    assert!(matches!(strategy, RoutingStrategyConfig::LeastLoaded));
}

#[test]
fn test_routing_strategy_cost_optimized_serialization() {
    let strategy = RoutingStrategyConfig::CostOptimized {
        performance_weight: 0.5,
    };
    let json = serde_json::to_string(&strategy).unwrap();
    assert!(json.contains("cost_optimized"));
    assert!(json.contains("0.5"));
}

#[test]
fn test_routing_strategy_cost_optimized_deserialization() {
    let json = r#"{"type": "cost_optimized", "performance_weight": 0.7}"#;
    let strategy: RoutingStrategyConfig = serde_json::from_str(json).unwrap();
    match strategy {
        RoutingStrategyConfig::CostOptimized { performance_weight } => {
            assert!((performance_weight - 0.7).abs() < f32::EPSILON);
        }
        _ => panic!("Expected CostOptimized"),
    }
}

#[test]
fn test_routing_strategy_latency_based_serialization() {
    let strategy = RoutingStrategyConfig::LatencyBased {
        latency_threshold_ms: 100,
    };
    let json = serde_json::to_string(&strategy).unwrap();
    assert!(json.contains("latency_based"));
    assert!(json.contains("100"));
}

#[test]
fn test_routing_strategy_latency_based_deserialization() {
    let json = r#"{"type": "latency_based", "latency_threshold_ms": 200}"#;
    let strategy: RoutingStrategyConfig = serde_json::from_str(json).unwrap();
    match strategy {
        RoutingStrategyConfig::LatencyBased {
            latency_threshold_ms,
        } => {
            assert_eq!(latency_threshold_ms, 200);
        }
        _ => panic!("Expected LatencyBased"),
    }
}

#[test]
fn test_routing_strategy_tag_based_serialization() {
    let strategy = RoutingStrategyConfig::TagBased {
        selectors: vec![TagSelector {
            key: "region".to_string(),
            value: "us-east".to_string(),
            operator: TagOperator::Eq,
        }],
    };
    let json = serde_json::to_string(&strategy).unwrap();
    assert!(json.contains("tag_based"));
    assert!(json.contains("region"));
    assert!(json.contains("us-east"));
}

#[test]
fn test_routing_strategy_tag_based_deserialization() {
    let json = r#"{"type": "tag_based", "selectors": [{"key": "env", "value": "prod", "operator": "eq"}]}"#;
    let strategy: RoutingStrategyConfig = serde_json::from_str(json).unwrap();
    match strategy {
        RoutingStrategyConfig::TagBased { selectors } => {
            assert_eq!(selectors.len(), 1);
            assert_eq!(selectors[0].key, "env");
            assert_eq!(selectors[0].value, "prod");
        }
        _ => panic!("Expected TagBased"),
    }
}

#[test]
fn test_routing_strategy_custom_serialization() {
    let strategy = RoutingStrategyConfig::Custom {
        class: "my.custom.Router".to_string(),
        config: serde_json::json!({"param": "value"}),
    };
    let json = serde_json::to_string(&strategy).unwrap();
    assert!(json.contains("custom"));
    assert!(json.contains("my.custom.Router"));
}

#[test]
fn test_routing_strategy_custom_deserialization() {
    let json = r#"{"type": "custom", "class": "router.Custom", "config": {"key": 123}}"#;
    let strategy: RoutingStrategyConfig = serde_json::from_str(json).unwrap();
    match strategy {
        RoutingStrategyConfig::Custom { class, config } => {
            assert_eq!(class, "router.Custom");
            assert_eq!(config["key"], 123);
        }
        _ => panic!("Expected Custom"),
    }
}

// ==================== TagSelector Tests ====================

#[test]
fn test_tag_selector_creation() {
    let selector = TagSelector {
        key: "tier".to_string(),
        value: "premium".to_string(),
        operator: TagOperator::Eq,
    };
    assert_eq!(selector.key, "tier");
    assert_eq!(selector.value, "premium");
}

#[test]
fn test_tag_selector_serialization() {
    let selector = TagSelector {
        key: "env".to_string(),
        value: "prod".to_string(),
        operator: TagOperator::Ne,
    };
    let json = serde_json::to_string(&selector).unwrap();
    assert!(json.contains("env"));
    assert!(json.contains("prod"));
    assert!(json.contains("ne"));
}

#[test]
fn test_tag_selector_deserialization() {
    let json = r#"{"key": "region", "value": "eu-west", "operator": "in"}"#;
    let selector: TagSelector = serde_json::from_str(json).unwrap();
    assert_eq!(selector.key, "region");
    assert_eq!(selector.value, "eu-west");
    assert!(matches!(selector.operator, TagOperator::In));
}

#[test]
fn test_tag_selector_default_operator() {
    let json = r#"{"key": "test", "value": "val"}"#;
    let selector: TagSelector = serde_json::from_str(json).unwrap();
    assert!(matches!(selector.operator, TagOperator::Eq));
}

#[test]
fn test_tag_selector_with_wildcard_value() {
    let selector = TagSelector {
        key: "model".to_string(),
        value: "gpt-*".to_string(),
        operator: TagOperator::Eq,
    };
    assert!(selector.value.contains("*"));
}

// ==================== TagOperator Tests ====================

#[test]
fn test_tag_operator_eq_serialization() {
    let op = TagOperator::Eq;
    let json = serde_json::to_string(&op).unwrap();
    assert_eq!(json, "\"eq\"");
}

#[test]
fn test_tag_operator_ne_serialization() {
    let op = TagOperator::Ne;
    let json = serde_json::to_string(&op).unwrap();
    assert_eq!(json, "\"ne\"");
}

#[test]
fn test_tag_operator_in_serialization() {
    let op = TagOperator::In;
    let json = serde_json::to_string(&op).unwrap();
    assert_eq!(json, "\"in\"");
}

#[test]
fn test_tag_operator_notin_serialization() {
    let op = TagOperator::NotIn;
    let json = serde_json::to_string(&op).unwrap();
    assert_eq!(json, "\"notin\"");
}

#[test]
fn test_tag_operator_exists_serialization() {
    let op = TagOperator::Exists;
    let json = serde_json::to_string(&op).unwrap();
    assert_eq!(json, "\"exists\"");
}

#[test]
fn test_tag_operator_notexists_serialization() {
    let op = TagOperator::NotExists;
    let json = serde_json::to_string(&op).unwrap();
    assert_eq!(json, "\"notexists\"");
}

#[test]
fn test_tag_operator_default() {
    let op = TagOperator::default();
    assert!(matches!(op, TagOperator::Eq));
}

#[test]
fn test_tag_operator_all_variants_deserialize() {
    let operators = ["eq", "ne", "in", "notin", "exists", "notexists"];
    for op in operators {
        let json = format!("\"{}\"", op);
        let _: TagOperator = serde_json::from_str(&json).unwrap();
    }
}

// ==================== CircuitBreakerConfig Tests ====================

#[test]
fn test_circuit_breaker_default() {
    let config = CircuitBreakerConfig::default();
    assert_eq!(config.failure_threshold, 5);
    assert_eq!(config.recovery_timeout_seconds, 60);
    assert_eq!(config.half_open_max_requests, 3);
    assert!(config.enabled);
}

#[test]
fn test_circuit_breaker_serialization() {
    let config = CircuitBreakerConfig {
        failure_threshold: 10,
        recovery_timeout_seconds: 120,
        half_open_max_requests: 5,
        enabled: false,
    };
    let json = serde_json::to_string(&config).unwrap();
    assert!(json.contains("10"));
    assert!(json.contains("120"));
    assert!(json.contains("5"));
    assert!(json.contains("false"));
}

#[test]
fn test_circuit_breaker_deserialization() {
    let json = r#"{"failure_threshold": 7, "recovery_timeout_seconds": 90, "half_open_max_requests": 4, "enabled": true}"#;
    let config: CircuitBreakerConfig = serde_json::from_str(json).unwrap();
    assert_eq!(config.failure_threshold, 7);
    assert_eq!(config.recovery_timeout_seconds, 90);
    assert_eq!(config.half_open_max_requests, 4);
    assert!(config.enabled);
}

#[test]
fn test_circuit_breaker_deserialization_with_defaults() {
    let json = r#"{}"#;
    let config: CircuitBreakerConfig = serde_json::from_str(json).unwrap();
    assert_eq!(config.failure_threshold, 5);
    assert_eq!(config.recovery_timeout_seconds, 60);
    assert_eq!(config.half_open_max_requests, 3);
    assert!(config.enabled);
}

#[test]
fn test_circuit_breaker_partial_deserialization() {
    let json = r#"{"failure_threshold": 15}"#;
    let config: CircuitBreakerConfig = serde_json::from_str(json).unwrap();
    assert_eq!(config.failure_threshold, 15);
    assert_eq!(config.recovery_timeout_seconds, 60);
}

// ==================== LoadBalancerConfig Tests ====================

#[test]
fn test_load_balancer_config_creation() {
    let config = LoadBalancerConfig {
        algorithm: LoadBalancerAlgorithm::RoundRobin,
        session_affinity: None,
    };
    assert!(matches!(
        config.algorithm,
        LoadBalancerAlgorithm::RoundRobin
    ));
    assert!(config.session_affinity.is_none());
}

#[test]
fn test_load_balancer_config_with_session_affinity() {
    let config = LoadBalancerConfig {
        algorithm: LoadBalancerAlgorithm::ConsistentHash,
        session_affinity: Some(SessionAffinityConfig {
            affinity_type: SessionAffinityType::ClientIp,
            timeout_seconds: 7200,
        }),
    };
    assert!(config.session_affinity.is_some());
}

#[test]
fn test_load_balancer_config_serialization() {
    let config = LoadBalancerConfig {
        algorithm: LoadBalancerAlgorithm::WeightedRoundRobin,
        session_affinity: None,
    };
    let json = serde_json::to_string(&config).unwrap();
    assert!(json.contains("weighted_round_robin"));
    assert!(!json.contains("session_affinity"));
}

#[test]
fn test_load_balancer_config_deserialization() {
    let json = r#"{"algorithm": "least_connections"}"#;
    let config: LoadBalancerConfig = serde_json::from_str(json).unwrap();
    assert!(matches!(
        config.algorithm,
        LoadBalancerAlgorithm::LeastConnections
    ));
}

// ==================== LoadBalancerAlgorithm Tests ====================

#[test]
fn test_load_balancer_algorithm_round_robin() {
    let algo = LoadBalancerAlgorithm::RoundRobin;
    let json = serde_json::to_string(&algo).unwrap();
    assert_eq!(json, "\"round_robin\"");
}

#[test]
fn test_load_balancer_algorithm_weighted_round_robin() {
    let algo = LoadBalancerAlgorithm::WeightedRoundRobin;
    let json = serde_json::to_string(&algo).unwrap();
    assert_eq!(json, "\"weighted_round_robin\"");
}

#[test]
fn test_load_balancer_algorithm_least_connections() {
    let algo = LoadBalancerAlgorithm::LeastConnections;
    let json = serde_json::to_string(&algo).unwrap();
    assert_eq!(json, "\"least_connections\"");
}

#[test]
fn test_load_balancer_algorithm_consistent_hash() {
    let algo = LoadBalancerAlgorithm::ConsistentHash;
    let json = serde_json::to_string(&algo).unwrap();
    assert_eq!(json, "\"consistent_hash\"");
}

#[test]
fn test_load_balancer_algorithm_all_variants_deserialize() {
    let algorithms = [
        "round_robin",
        "weighted_round_robin",
        "least_connections",
        "consistent_hash",
    ];
    for algo in algorithms {
        let json = format!("\"{}\"", algo);
        let _: LoadBalancerAlgorithm = serde_json::from_str(&json).unwrap();
    }
}

// ==================== SessionAffinityConfig Tests ====================

#[test]
fn test_session_affinity_config_creation() {
    let config = SessionAffinityConfig {
        affinity_type: SessionAffinityType::ClientIp,
        timeout_seconds: 3600,
    };
    assert_eq!(config.timeout_seconds, 3600);
}

#[test]
fn test_session_affinity_config_serialization() {
    let config = SessionAffinityConfig {
        affinity_type: SessionAffinityType::UserId,
        timeout_seconds: 7200,
    };
    let json = serde_json::to_string(&config).unwrap();
    assert!(json.contains("user_id"));
    assert!(json.contains("7200"));
}

#[test]
fn test_session_affinity_config_deserialization() {
    let json = r#"{"affinity_type": "client_ip", "timeout_seconds": 1800}"#;
    let config: SessionAffinityConfig = serde_json::from_str(json).unwrap();
    assert!(matches!(
        config.affinity_type,
        SessionAffinityType::ClientIp
    ));
    assert_eq!(config.timeout_seconds, 1800);
}

#[test]
fn test_session_affinity_config_default_timeout() {
    let json = r#"{"affinity_type": "user_id"}"#;
    let config: SessionAffinityConfig = serde_json::from_str(json).unwrap();
    assert_eq!(config.timeout_seconds, 3600);
}

// ==================== SessionAffinityType Tests ====================

#[test]
fn test_session_affinity_type_client_ip() {
    let affinity = SessionAffinityType::ClientIp;
    let json = serde_json::to_string(&affinity).unwrap();
    assert_eq!(json, "\"client_ip\"");
}

#[test]
fn test_session_affinity_type_user_id() {
    let affinity = SessionAffinityType::UserId;
    let json = serde_json::to_string(&affinity).unwrap();
    assert_eq!(json, "\"user_id\"");
}

#[test]
fn test_session_affinity_type_custom_header() {
    let affinity = SessionAffinityType::CustomHeader {
        header_name: "X-Session-Id".to_string(),
    };
    let json = serde_json::to_string(&affinity).unwrap();
    assert!(json.contains("custom_header"));
    assert!(json.contains("X-Session-Id"));
}

#[test]
fn test_session_affinity_type_custom_header_deserialization() {
    let json = r#"{"custom_header": {"header_name": "X-Request-Id"}}"#;
    let affinity: SessionAffinityType = serde_json::from_str(json).unwrap();
    match affinity {
        SessionAffinityType::CustomHeader { header_name } => {
            assert_eq!(header_name, "X-Request-Id");
        }
        _ => panic!("Expected CustomHeader"),
    }
}

// ==================== RoutingConfig Integration Tests ====================

#[test]
fn test_routing_config_serialization() {
    let config = RoutingConfig {
        strategy: RoutingStrategyConfig::RoundRobin,
        health_check: HealthCheckConfig::default(),
        circuit_breaker: CircuitBreakerConfig::default(),
        load_balancer: None,
    };
    let json = serde_json::to_string(&config).unwrap();
    assert!(json.contains("round_robin"));
    assert!(json.contains("health_check"));
    assert!(json.contains("circuit_breaker"));
    assert!(!json.contains("load_balancer"));
}

#[test]
fn test_routing_config_with_load_balancer() {
    let config = RoutingConfig {
        strategy: RoutingStrategyConfig::LeastLoaded,
        health_check: HealthCheckConfig::default(),
        circuit_breaker: CircuitBreakerConfig::default(),
        load_balancer: Some(LoadBalancerConfig {
            algorithm: LoadBalancerAlgorithm::ConsistentHash,
            session_affinity: Some(SessionAffinityConfig {
                affinity_type: SessionAffinityType::UserId,
                timeout_seconds: 3600,
            }),
        }),
    };
    let json = serde_json::to_string(&config).unwrap();
    assert!(json.contains("load_balancer"));
    assert!(json.contains("consistent_hash"));
}

#[test]
fn test_routing_config_complex_deserialization() {
    let json = r#"{
            "strategy": {"type": "cost_optimized", "performance_weight": 0.8},
            "health_check": {"interval_seconds": 60, "enabled": true},
            "circuit_breaker": {"failure_threshold": 10, "enabled": true},
            "load_balancer": {
                "algorithm": "weighted_round_robin",
                "session_affinity": {
                    "affinity_type": "client_ip",
                    "timeout_seconds": 1800
                }
            }
        }"#;
    let config: RoutingConfig = serde_json::from_str(json).unwrap();
    match config.strategy {
        RoutingStrategyConfig::CostOptimized { performance_weight } => {
            assert!((performance_weight - 0.8).abs() < f32::EPSILON);
        }
        _ => panic!("Expected CostOptimized"),
    }
    assert!(config.load_balancer.is_some());
}

// ==================== Debug and Clone Tests ====================

#[test]
fn test_routing_strategy_debug() {
    let strategy = RoutingStrategyConfig::RoundRobin;
    let debug = format!("{:?}", strategy);
    assert!(debug.contains("RoundRobin"));
}

#[test]
fn test_routing_strategy_clone() {
    let strategy = RoutingStrategyConfig::CostOptimized {
        performance_weight: 0.5,
    };
    let cloned = strategy.clone();
    match cloned {
        RoutingStrategyConfig::CostOptimized { performance_weight } => {
            assert!((performance_weight - 0.5).abs() < f32::EPSILON);
        }
        _ => panic!("Expected CostOptimized"),
    }
}

#[test]
fn test_tag_selector_clone() {
    let selector = TagSelector {
        key: "test".to_string(),
        value: "value".to_string(),
        operator: TagOperator::In,
    };
    let cloned = selector.clone();
    assert_eq!(cloned.key, "test");
}

#[test]
fn test_circuit_breaker_clone() {
    let config = CircuitBreakerConfig::default();
    let cloned = config.clone();
    assert_eq!(cloned.failure_threshold, config.failure_threshold);
}

#[test]
fn test_load_balancer_config_clone() {
    let config = LoadBalancerConfig {
        algorithm: LoadBalancerAlgorithm::RoundRobin,
        session_affinity: None,
    };
    let cloned = config.clone();
    assert!(matches!(
        cloned.algorithm,
        LoadBalancerAlgorithm::RoundRobin
    ));
}

#[test]
fn test_session_affinity_config_clone() {
    let config = SessionAffinityConfig {
        affinity_type: SessionAffinityType::ClientIp,
        timeout_seconds: 3600,
    };
    let cloned = config.clone();
    assert_eq!(cloned.timeout_seconds, 3600);
}

// ==================== Edge Cases ====================

#[test]
fn test_tag_selector_empty_values() {
    let selector = TagSelector {
        key: "".to_string(),
        value: "".to_string(),
        operator: TagOperator::Eq,
    };
    let json = serde_json::to_string(&selector).unwrap();
    let deserialized: TagSelector = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.key, "");
    assert_eq!(deserialized.value, "");
}

#[test]
fn test_routing_strategy_tag_based_empty_selectors() {
    let strategy = RoutingStrategyConfig::TagBased { selectors: vec![] };
    let json = serde_json::to_string(&strategy).unwrap();
    assert!(json.contains("[]"));
}

#[test]
fn test_routing_strategy_custom_empty_config() {
    let strategy = RoutingStrategyConfig::Custom {
        class: "empty.Router".to_string(),
        config: serde_json::json!({}),
    };
    let json = serde_json::to_string(&strategy).unwrap();
    assert!(json.contains("{}"));
}

#[test]
fn test_circuit_breaker_zero_values() {
    let config = CircuitBreakerConfig {
        failure_threshold: 0,
        recovery_timeout_seconds: 0,
        half_open_max_requests: 0,
        enabled: false,
    };
    assert_eq!(config.failure_threshold, 0);
    assert_eq!(config.recovery_timeout_seconds, 0);
}

#[test]
fn test_session_affinity_zero_timeout() {
    let config = SessionAffinityConfig {
        affinity_type: SessionAffinityType::ClientIp,
        timeout_seconds: 0,
    };
    assert_eq!(config.timeout_seconds, 0);
}

#[test]
fn test_latency_based_zero_threshold() {
    let strategy = RoutingStrategyConfig::LatencyBased {
        latency_threshold_ms: 0,
    };
    match strategy {
        RoutingStrategyConfig::LatencyBased {
            latency_threshold_ms,
        } => {
            assert_eq!(latency_threshold_ms, 0);
        }
        _ => panic!("Expected LatencyBased"),
    }
}

#[test]
fn test_cost_optimized_zero_weight() {
    let strategy = RoutingStrategyConfig::CostOptimized {
        performance_weight: 0.0,
    };
    match strategy {
        RoutingStrategyConfig::CostOptimized { performance_weight } => {
            assert!(performance_weight.abs() < f32::EPSILON);
        }
        _ => panic!("Expected CostOptimized"),
    }
}

#[test]
fn test_cost_optimized_max_weight() {
    let strategy = RoutingStrategyConfig::CostOptimized {
        performance_weight: 1.0,
    };
    match strategy {
        RoutingStrategyConfig::CostOptimized { performance_weight } => {
            assert!((performance_weight - 1.0).abs() < f32::EPSILON);
        }
        _ => panic!("Expected CostOptimized"),
    }
}
