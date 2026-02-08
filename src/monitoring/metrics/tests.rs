//! Tests for metrics module

#[cfg(test)]
mod tests {
    use super::super::collector::MetricsCollector;
    use super::super::helpers::{calculate_average, calculate_percentile};
    use crate::config::models::monitoring::MonitoringConfig;
    use std::collections::VecDeque;

    #[test]
    fn test_calculate_percentile() {
        let values = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        assert_eq!(calculate_percentile(&values, 0.5), 3.0); // 50th percentile
        assert_eq!(calculate_percentile(&values, 0.95), 4.8); // 95th percentile (interpolated)
        assert_eq!(calculate_percentile(&values, 1.0), 5.0); // 100th percentile
        assert_eq!(calculate_percentile(&[], 0.5), 0.0); // empty array
    }

    #[test]
    fn test_calculate_average() {
        let values: VecDeque<f64> = vec![1.0, 2.0, 3.0, 4.0, 5.0].into();
        assert_eq!(calculate_average(&values), 3.0);
        assert_eq!(calculate_average(&VecDeque::new()), 0.0);
    }

    #[tokio::test]
    async fn test_metrics_collector_creation() {
        let config = MonitoringConfig {
            metrics: crate::config::models::monitoring::MetricsConfig {
                enabled: true,
                port: 9090,
                path: "/metrics".to_string(),
            },
            tracing: crate::config::models::monitoring::TracingConfig {
                enabled: false,
                endpoint: None,
                service_name: "test".to_string(),
            },
            health: crate::config::models::monitoring::HealthConfig {
                path: "/health".to_string(),
                detailed: true,
            },
        };

        let collector = MetricsCollector::new(&config).await.unwrap();
        assert!(!collector.is_active());
    }
}
