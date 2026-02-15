//! Health checking tests

use std::collections::HashMap;
use std::time::Duration;

use super::types::{ComponentHealth, ComponentHealthCheckConfig, HealthSummary};

#[test]
fn test_component_health_creation() {
    let health = ComponentHealth {
        name: "test_component".to_string(),
        healthy: true,
        status: "healthy".to_string(),
        last_check: chrono::Utc::now(),
        response_time_ms: 50,
        error: None,
        metadata: HashMap::new(),
    };

    assert!(health.healthy);
    assert_eq!(health.name, "test_component");
    assert_eq!(health.response_time_ms, 50);
}

#[test]
fn test_health_summary_calculation() {
    let summary = HealthSummary {
        total_components: 5,
        healthy_components: 4,
        unhealthy_components: 1,
        health_percentage: 80.0,
    };

    assert_eq!(summary.total_components, 5);
    assert_eq!(summary.healthy_components, 4);
    assert_eq!(summary.health_percentage, 80.0);
}

#[test]
fn test_health_check_config() {
    let config = ComponentHealthCheckConfig {
        name: "database".to_string(),
        interval: Duration::from_secs(30),
        timeout: Duration::from_secs(5),
        retries: 3,
        critical: true,
    };

    assert_eq!(config.name, "database");
    assert!(config.critical);
    assert_eq!(config.retries, 3);
}
