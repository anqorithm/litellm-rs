//! Health checking types and data structures

use std::collections::HashMap;
use std::time::Duration;

/// Overall system health status
#[derive(Debug, Clone, serde::Serialize)]
pub struct HealthStatus {
    /// Whether the system is overall healthy
    pub overall_healthy: bool,
    /// Timestamp of last health check
    pub last_check: chrono::DateTime<chrono::Utc>,
    /// Individual component health
    pub components: HashMap<String, ComponentHealth>,
    /// System uptime
    pub uptime_seconds: u64,
    /// Health check summary
    pub summary: HealthSummary,
}

/// Individual component health
#[derive(Debug, Clone, serde::Serialize)]
pub struct ComponentHealth {
    /// Component name
    pub name: String,
    /// Whether the component is healthy
    pub healthy: bool,
    /// Health status message
    pub status: String,
    /// Last check timestamp
    pub last_check: chrono::DateTime<chrono::Utc>,
    /// Response time for health check
    pub response_time_ms: u64,
    /// Error message (if unhealthy)
    pub error: Option<String>,
    /// Additional metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Health check summary
#[derive(Debug, Clone, serde::Serialize)]
pub struct HealthSummary {
    /// Total number of components
    pub total_components: usize,
    /// Number of healthy components
    pub healthy_components: usize,
    /// Number of unhealthy components
    pub unhealthy_components: usize,
    /// Health percentage
    pub health_percentage: f64,
}

/// Health check configuration for a runtime component
#[derive(Debug, Clone)]
pub struct ComponentHealthCheckConfig {
    /// Component name
    pub name: String,
    /// Check interval
    pub interval: Duration,
    /// Timeout for health check
    pub timeout: Duration,
    /// Number of retries
    pub retries: u32,
    /// Whether this component is critical
    pub critical: bool,
}

/// Consolidated health data - single lock for all health-related state
#[derive(Debug)]
pub(super) struct HealthData {
    /// Component health status
    pub components: HashMap<String, ComponentHealth>,
    /// Overall health status
    pub overall: HealthStatus,
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    // ==================== Helper Functions ====================

    fn create_test_component_health(name: &str, healthy: bool) -> ComponentHealth {
        ComponentHealth {
            name: name.to_string(),
            healthy,
            status: if healthy {
                "OK".to_string()
            } else {
                "FAILED".to_string()
            },
            last_check: Utc::now(),
            response_time_ms: if healthy { 50 } else { 5000 },
            error: if healthy {
                None
            } else {
                Some("Connection refused".to_string())
            },
            metadata: HashMap::new(),
        }
    }

    fn create_test_health_summary() -> HealthSummary {
        HealthSummary {
            total_components: 5,
            healthy_components: 4,
            unhealthy_components: 1,
            health_percentage: 80.0,
        }
    }

    fn create_test_health_status() -> HealthStatus {
        let mut components = HashMap::new();
        components.insert(
            "database".to_string(),
            create_test_component_health("database", true),
        );
        components.insert(
            "cache".to_string(),
            create_test_component_health("cache", true),
        );
        components.insert(
            "api".to_string(),
            create_test_component_health("api", false),
        );

        HealthStatus {
            overall_healthy: false,
            last_check: Utc::now(),
            components,
            uptime_seconds: 86400,
            summary: HealthSummary {
                total_components: 3,
                healthy_components: 2,
                unhealthy_components: 1,
                health_percentage: 66.67,
            },
        }
    }

    // ==================== HealthStatus Tests ====================

    #[test]
    fn test_health_status_creation() {
        let status = create_test_health_status();

        assert!(!status.overall_healthy);
        assert_eq!(status.uptime_seconds, 86400);
        assert_eq!(status.components.len(), 3);
    }

    #[test]
    fn test_health_status_all_healthy() {
        let mut components = HashMap::new();
        components.insert("db".to_string(), create_test_component_health("db", true));
        components.insert(
            "cache".to_string(),
            create_test_component_health("cache", true),
        );

        let status = HealthStatus {
            overall_healthy: true,
            last_check: Utc::now(),
            components,
            uptime_seconds: 3600,
            summary: HealthSummary {
                total_components: 2,
                healthy_components: 2,
                unhealthy_components: 0,
                health_percentage: 100.0,
            },
        };

        assert!(status.overall_healthy);
        assert_eq!(status.summary.health_percentage, 100.0);
    }

    #[test]
    fn test_health_status_clone() {
        let status = create_test_health_status();
        let cloned = status.clone();

        assert_eq!(cloned.overall_healthy, status.overall_healthy);
        assert_eq!(cloned.uptime_seconds, status.uptime_seconds);
        assert_eq!(cloned.components.len(), status.components.len());
    }

    #[test]
    fn test_health_status_debug() {
        let status = create_test_health_status();
        let debug_str = format!("{:?}", status);

        assert!(debug_str.contains("HealthStatus"));
        assert!(debug_str.contains("overall_healthy"));
    }

    #[test]
    fn test_health_status_serialization() {
        let status = create_test_health_status();
        let json = serde_json::to_string(&status).unwrap();

        assert!(json.contains("overall_healthy"));
        assert!(json.contains("uptime_seconds"));
        assert!(json.contains("86400"));
    }

    #[test]
    fn test_health_status_empty_components() {
        let status = HealthStatus {
            overall_healthy: true,
            last_check: Utc::now(),
            components: HashMap::new(),
            uptime_seconds: 0,
            summary: HealthSummary {
                total_components: 0,
                healthy_components: 0,
                unhealthy_components: 0,
                health_percentage: 100.0,
            },
        };

        assert!(status.components.is_empty());
        assert_eq!(status.summary.total_components, 0);
    }

    // ==================== ComponentHealth Tests ====================

    #[test]
    fn test_component_health_healthy() {
        let component = create_test_component_health("database", true);

        assert_eq!(component.name, "database");
        assert!(component.healthy);
        assert_eq!(component.status, "OK");
        assert!(component.error.is_none());
        assert_eq!(component.response_time_ms, 50);
    }

    #[test]
    fn test_component_health_unhealthy() {
        let component = create_test_component_health("redis", false);

        assert_eq!(component.name, "redis");
        assert!(!component.healthy);
        assert_eq!(component.status, "FAILED");
        assert!(component.error.is_some());
        assert_eq!(component.error.as_ref().unwrap(), "Connection refused");
    }

    #[test]
    fn test_component_health_with_metadata() {
        let mut metadata = HashMap::new();
        metadata.insert("version".to_string(), serde_json::json!("1.0.0"));
        metadata.insert("connections".to_string(), serde_json::json!(10));

        let component = ComponentHealth {
            name: "database".to_string(),
            healthy: true,
            status: "Running".to_string(),
            last_check: Utc::now(),
            response_time_ms: 25,
            error: None,
            metadata,
        };

        assert_eq!(component.metadata.len(), 2);
        assert_eq!(
            component.metadata.get("version"),
            Some(&serde_json::json!("1.0.0"))
        );
    }

    #[test]
    fn test_component_health_clone() {
        let component = create_test_component_health("api", true);
        let cloned = component.clone();

        assert_eq!(cloned.name, component.name);
        assert_eq!(cloned.healthy, component.healthy);
        assert_eq!(cloned.response_time_ms, component.response_time_ms);
    }

    #[test]
    fn test_component_health_debug() {
        let component = create_test_component_health("cache", true);
        let debug_str = format!("{:?}", component);

        assert!(debug_str.contains("ComponentHealth"));
        assert!(debug_str.contains("cache"));
    }

    #[test]
    fn test_component_health_serialization() {
        let component = create_test_component_health("storage", true);
        let json = serde_json::to_string(&component).unwrap();

        assert!(json.contains("storage"));
        assert!(json.contains("healthy"));
        assert!(json.contains("true"));
    }

    #[test]
    fn test_component_health_high_response_time() {
        let component = ComponentHealth {
            name: "slow_service".to_string(),
            healthy: true,
            status: "Degraded".to_string(),
            last_check: Utc::now(),
            response_time_ms: 5000,
            error: None,
            metadata: HashMap::new(),
        };

        assert!(component.healthy);
        assert!(component.response_time_ms > 1000);
    }

    // ==================== HealthSummary Tests ====================

    #[test]
    fn test_health_summary_creation() {
        let summary = create_test_health_summary();

        assert_eq!(summary.total_components, 5);
        assert_eq!(summary.healthy_components, 4);
        assert_eq!(summary.unhealthy_components, 1);
        assert!((summary.health_percentage - 80.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_health_summary_all_healthy() {
        let summary = HealthSummary {
            total_components: 10,
            healthy_components: 10,
            unhealthy_components: 0,
            health_percentage: 100.0,
        };

        assert_eq!(summary.healthy_components, summary.total_components);
        assert_eq!(summary.unhealthy_components, 0);
    }

    #[test]
    fn test_health_summary_all_unhealthy() {
        let summary = HealthSummary {
            total_components: 5,
            healthy_components: 0,
            unhealthy_components: 5,
            health_percentage: 0.0,
        };

        assert_eq!(summary.healthy_components, 0);
        assert_eq!(summary.unhealthy_components, summary.total_components);
    }

    #[test]
    fn test_health_summary_clone() {
        let summary = create_test_health_summary();
        let cloned = summary.clone();

        assert_eq!(cloned.total_components, summary.total_components);
        assert_eq!(cloned.health_percentage, summary.health_percentage);
    }

    #[test]
    fn test_health_summary_debug() {
        let summary = create_test_health_summary();
        let debug_str = format!("{:?}", summary);

        assert!(debug_str.contains("HealthSummary"));
        assert!(debug_str.contains("total_components"));
    }

    #[test]
    fn test_health_summary_serialization() {
        let summary = create_test_health_summary();
        let json = serde_json::to_string(&summary).unwrap();

        assert!(json.contains("total_components"));
        assert!(json.contains("health_percentage"));
        assert!(json.contains("80"));
    }

    #[test]
    fn test_health_summary_consistency() {
        let summary = HealthSummary {
            total_components: 8,
            healthy_components: 6,
            unhealthy_components: 2,
            health_percentage: 75.0,
        };

        assert_eq!(
            summary.healthy_components + summary.unhealthy_components,
            summary.total_components
        );
    }

    // ==================== ComponentHealthCheckConfig Tests ====================

    #[test]
    fn test_health_check_config_creation() {
        let config = ComponentHealthCheckConfig {
            name: "database".to_string(),
            interval: Duration::from_secs(30),
            timeout: Duration::from_secs(5),
            retries: 3,
            critical: true,
        };

        assert_eq!(config.name, "database");
        assert_eq!(config.interval, Duration::from_secs(30));
        assert_eq!(config.timeout, Duration::from_secs(5));
        assert_eq!(config.retries, 3);
        assert!(config.critical);
    }

    #[test]
    fn test_health_check_config_non_critical() {
        let config = ComponentHealthCheckConfig {
            name: "metrics".to_string(),
            interval: Duration::from_secs(60),
            timeout: Duration::from_secs(10),
            retries: 1,
            critical: false,
        };

        assert!(!config.critical);
        assert_eq!(config.retries, 1);
    }

    #[test]
    fn test_health_check_config_clone() {
        let config = ComponentHealthCheckConfig {
            name: "api".to_string(),
            interval: Duration::from_secs(15),
            timeout: Duration::from_secs(3),
            retries: 2,
            critical: true,
        };

        let cloned = config.clone();
        assert_eq!(cloned.name, config.name);
        assert_eq!(cloned.interval, config.interval);
        assert_eq!(cloned.critical, config.critical);
    }

    #[test]
    fn test_health_check_config_debug() {
        let config = ComponentHealthCheckConfig {
            name: "cache".to_string(),
            interval: Duration::from_secs(20),
            timeout: Duration::from_secs(2),
            retries: 3,
            critical: false,
        };

        let debug_str = format!("{:?}", config);
        assert!(debug_str.contains("ComponentHealthCheckConfig"));
        assert!(debug_str.contains("cache"));
    }

    #[test]
    fn test_health_check_config_aggressive() {
        let config = ComponentHealthCheckConfig {
            name: "critical_service".to_string(),
            interval: Duration::from_secs(5),
            timeout: Duration::from_secs(1),
            retries: 5,
            critical: true,
        };

        assert!(config.interval < Duration::from_secs(10));
        assert!(config.retries > 3);
    }

    #[test]
    fn test_health_check_config_relaxed() {
        let config = ComponentHealthCheckConfig {
            name: "background_job".to_string(),
            interval: Duration::from_secs(300),
            timeout: Duration::from_secs(30),
            retries: 1,
            critical: false,
        };

        assert!(config.interval >= Duration::from_secs(60));
        assert!(!config.critical);
    }

    // ==================== HealthData Tests ====================

    #[test]
    fn test_health_data_creation() {
        let mut components = HashMap::new();
        components.insert("db".to_string(), create_test_component_health("db", true));

        let data = HealthData {
            components: components.clone(),
            overall: create_test_health_status(),
        };

        assert_eq!(data.components.len(), 1);
        assert!(!data.overall.overall_healthy);
    }

    #[test]
    fn test_health_data_debug() {
        let data = HealthData {
            components: HashMap::new(),
            overall: create_test_health_status(),
        };

        let debug_str = format!("{:?}", data);
        assert!(debug_str.contains("HealthData"));
    }

    // ==================== Integration Tests ====================

    #[test]
    fn test_full_health_check_workflow() {
        // Simulate health check results
        let mut components = HashMap::new();

        let component_names = ["database", "cache", "api", "storage", "queue"];
        let health_states = [true, true, false, true, true];

        for (name, healthy) in component_names.iter().zip(health_states.iter()) {
            components.insert(
                name.to_string(),
                create_test_component_health(name, *healthy),
            );
        }

        let healthy_count = health_states.iter().filter(|&&h| h).count();
        let unhealthy_count = health_states.len() - healthy_count;

        let status = HealthStatus {
            overall_healthy: unhealthy_count == 0,
            last_check: Utc::now(),
            components,
            uptime_seconds: 3600,
            summary: HealthSummary {
                total_components: health_states.len(),
                healthy_components: healthy_count,
                unhealthy_components: unhealthy_count,
                health_percentage: (healthy_count as f64 / health_states.len() as f64) * 100.0,
            },
        };

        assert!(!status.overall_healthy);
        assert_eq!(status.summary.healthy_components, 4);
        assert_eq!(status.summary.unhealthy_components, 1);
        assert!((status.summary.health_percentage - 80.0).abs() < 0.01);
    }

    #[test]
    fn test_component_health_by_response_time() {
        let components = [
            create_test_component_health("fast", true),
            ComponentHealth {
                name: "slow".to_string(),
                healthy: true,
                status: "OK".to_string(),
                last_check: Utc::now(),
                response_time_ms: 500,
                error: None,
                metadata: HashMap::new(),
            },
            ComponentHealth {
                name: "very_slow".to_string(),
                healthy: true,
                status: "Degraded".to_string(),
                last_check: Utc::now(),
                response_time_ms: 2000,
                error: None,
                metadata: HashMap::new(),
            },
        ];

        let avg_response_time: u64 =
            components.iter().map(|c| c.response_time_ms).sum::<u64>() / components.len() as u64;

        assert!(avg_response_time > 100);
    }

    #[test]
    fn test_critical_vs_non_critical_components() {
        let configs = [
            ComponentHealthCheckConfig {
                name: "database".to_string(),
                interval: Duration::from_secs(10),
                timeout: Duration::from_secs(2),
                retries: 3,
                critical: true,
            },
            ComponentHealthCheckConfig {
                name: "cache".to_string(),
                interval: Duration::from_secs(30),
                timeout: Duration::from_secs(5),
                retries: 2,
                critical: true,
            },
            ComponentHealthCheckConfig {
                name: "metrics".to_string(),
                interval: Duration::from_secs(60),
                timeout: Duration::from_secs(10),
                retries: 1,
                critical: false,
            },
        ];

        let critical_count = configs.iter().filter(|c| c.critical).count();
        let non_critical_count = configs.len() - critical_count;

        assert_eq!(critical_count, 2);
        assert_eq!(non_critical_count, 1);
    }

    #[test]
    fn test_health_status_json_structure() {
        let status = create_test_health_status();
        let json = serde_json::to_string_pretty(&status).unwrap();

        // Verify JSON structure
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

        assert!(parsed.get("overall_healthy").is_some());
        assert!(parsed.get("uptime_seconds").is_some());
        assert!(parsed.get("components").is_some());
        assert!(parsed.get("summary").is_some());
    }

    #[test]
    fn test_uptime_calculation() {
        let uptimes_seconds = [0, 60, 3600, 86400, 604800];
        let expected_descriptions = ["0 seconds", "1 minute", "1 hour", "1 day", "1 week"];

        for (uptime, desc) in uptimes_seconds.iter().zip(expected_descriptions.iter()) {
            let status = HealthStatus {
                overall_healthy: true,
                last_check: Utc::now(),
                components: HashMap::new(),
                uptime_seconds: *uptime,
                summary: HealthSummary {
                    total_components: 0,
                    healthy_components: 0,
                    unhealthy_components: 0,
                    health_percentage: 100.0,
                },
            };

            assert_eq!(status.uptime_seconds, *uptime);
            // Description is for documentation purposes
            let _ = desc;
        }
    }
}
