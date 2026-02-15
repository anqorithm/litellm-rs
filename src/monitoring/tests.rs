//! Tests for monitoring module

#[cfg(test)]
use super::types::*;
use std::collections::HashMap;

// ==================== Alert Tests ====================

#[test]
fn test_alert_creation() {
    let alert = Alert {
        id: "test-alert".to_string(),
        severity: AlertSeverity::Warning,
        title: "Test Alert".to_string(),
        description: "This is a test alert".to_string(),
        timestamp: chrono::Utc::now(),
        source: "test".to_string(),
        metadata: serde_json::json!({"test": true}),
        resolved: false,
    };

    assert_eq!(alert.severity, AlertSeverity::Warning);
    assert!(!alert.resolved);
}

#[test]
fn test_alert_all_severity_levels() {
    let severities = [
        AlertSeverity::Info,
        AlertSeverity::Warning,
        AlertSeverity::Critical,
        AlertSeverity::Emergency,
    ];

    for severity in severities {
        let alert = Alert {
            id: format!("alert-{:?}", severity),
            severity,
            title: "Test".to_string(),
            description: "Test".to_string(),
            timestamp: chrono::Utc::now(),
            source: "test".to_string(),
            metadata: serde_json::json!({}),
            resolved: false,
        };
        assert_eq!(alert.severity, severity);
    }
}

#[test]
fn test_alert_resolved_state() {
    let mut alert = Alert {
        id: "test-alert".to_string(),
        severity: AlertSeverity::Warning,
        title: "Test".to_string(),
        description: "Test".to_string(),
        timestamp: chrono::Utc::now(),
        source: "test".to_string(),
        metadata: serde_json::json!({}),
        resolved: false,
    };

    assert!(!alert.resolved);
    alert.resolved = true;
    assert!(alert.resolved);
}

#[test]
fn test_alert_with_metadata() {
    let alert = Alert {
        id: "test-alert".to_string(),
        severity: AlertSeverity::Critical,
        title: "High CPU".to_string(),
        description: "CPU usage exceeded threshold".to_string(),
        timestamp: chrono::Utc::now(),
        source: "system-monitor".to_string(),
        metadata: serde_json::json!({
            "cpu_usage": 95.5,
            "threshold": 90.0,
            "host": "server-01"
        }),
        resolved: false,
    };

    assert_eq!(alert.metadata["cpu_usage"], 95.5);
    assert_eq!(alert.metadata["threshold"], 90.0);
    assert_eq!(alert.metadata["host"], "server-01");
}

// ==================== AlertSeverity Tests ====================

#[test]
fn test_alert_severity_display() {
    assert_eq!(format!("{}", AlertSeverity::Info), "INFO");
    assert_eq!(format!("{}", AlertSeverity::Warning), "WARNING");
    assert_eq!(format!("{}", AlertSeverity::Critical), "CRITICAL");
    assert_eq!(format!("{}", AlertSeverity::Emergency), "EMERGENCY");
}

#[test]
fn test_alert_severity_equality() {
    assert_eq!(AlertSeverity::Info, AlertSeverity::Info);
    assert_ne!(AlertSeverity::Info, AlertSeverity::Warning);
    assert_ne!(AlertSeverity::Warning, AlertSeverity::Critical);
}

#[test]
fn test_alert_severity_clone() {
    let severity = AlertSeverity::Critical;
    let cloned = severity;
    assert_eq!(severity, cloned);
}

// ==================== SystemMetrics Tests ====================

#[test]
fn test_system_metrics_structure() {
    let metrics = SystemMetrics {
        timestamp: chrono::Utc::now(),
        requests: MonitoringRequestMetrics {
            total_requests: 1000,
            requests_per_second: 10.5,
            avg_response_time_ms: 150.0,
            p95_response_time_ms: 300.0,
            p99_response_time_ms: 500.0,
            success_rate: 99.5,
            status_codes: std::collections::HashMap::new(),
            endpoints: std::collections::HashMap::new(),
        },
        providers: ProviderMetrics {
            total_provider_requests: 800,
            provider_success_rates: std::collections::HashMap::new(),
            provider_response_times: std::collections::HashMap::new(),
            provider_errors: std::collections::HashMap::new(),
            provider_usage: std::collections::HashMap::new(),
            token_usage: std::collections::HashMap::new(),
            costs: std::collections::HashMap::new(),
        },
        system: SystemResourceMetrics {
            cpu_usage: 45.2,
            memory_usage: 1024 * 1024 * 512, // 512MB
            memory_usage_percent: 25.0,
            disk_usage: 1024 * 1024 * 1024 * 10, // 10GB
            disk_usage_percent: 50.0,
            network_bytes_in: 1024 * 1024,
            network_bytes_out: 1024 * 512,
            active_connections: 100,
            database_connections: 10,
            redis_connections: 5,
        },
        errors: ErrorMetrics {
            total_errors: 5,
            error_rate: 0.1,
            error_types: std::collections::HashMap::new(),
            error_endpoints: std::collections::HashMap::new(),
            critical_errors: 1,
            warnings: 4,
        },
        performance: PerformanceMetrics {
            cache_hit_rate: 85.5,
            cache_miss_rate: 14.5,
            avg_db_query_time_ms: 25.0,
            queue_depth: 0,
            throughput: 10.5,
            latency_percentiles: LatencyPercentiles {
                p50: 100.0,
                p90: 200.0,
                p95: 300.0,
                p99: 500.0,
                p999: 800.0,
            },
        },
    };

    assert_eq!(metrics.requests.total_requests, 1000);
    assert_eq!(metrics.system.cpu_usage, 45.2);
    assert_eq!(metrics.errors.critical_errors, 1);
}

// ==================== RequestMetrics Tests ====================

#[test]
fn test_request_metrics() {
    let mut status_codes = HashMap::new();
    status_codes.insert(200_u16, 950_u64);
    status_codes.insert(400_u16, 30_u64);
    status_codes.insert(500_u16, 20_u64);

    let mut endpoints = HashMap::new();
    endpoints.insert("/v1/chat/completions".to_string(), 800_u64);
    endpoints.insert("/v1/embeddings".to_string(), 200_u64);

    let metrics = MonitoringRequestMetrics {
        total_requests: 1000,
        requests_per_second: 10.0,
        avg_response_time_ms: 100.0,
        p95_response_time_ms: 250.0,
        p99_response_time_ms: 400.0,
        success_rate: 95.0,
        status_codes,
        endpoints,
    };

    assert_eq!(metrics.total_requests, 1000);
    assert_eq!(metrics.status_codes.get(&200), Some(&950));
    assert_eq!(metrics.endpoints.get("/v1/chat/completions"), Some(&800));
}

#[test]
fn test_request_metrics_empty() {
    let metrics = MonitoringRequestMetrics {
        total_requests: 0,
        requests_per_second: 0.0,
        avg_response_time_ms: 0.0,
        p95_response_time_ms: 0.0,
        p99_response_time_ms: 0.0,
        success_rate: 100.0,
        status_codes: HashMap::new(),
        endpoints: HashMap::new(),
    };

    assert_eq!(metrics.total_requests, 0);
    assert!(metrics.status_codes.is_empty());
    assert!(metrics.endpoints.is_empty());
}

// ==================== ProviderMetrics Tests ====================

#[test]
fn test_provider_metrics() {
    let mut provider_success_rates = HashMap::new();
    provider_success_rates.insert("openai".to_string(), 99.5);
    provider_success_rates.insert("anthropic".to_string(), 99.8);

    let mut token_usage = HashMap::new();
    token_usage.insert("openai".to_string(), 1000000_u64);
    token_usage.insert("anthropic".to_string(), 500000_u64);

    let mut costs = HashMap::new();
    costs.insert("openai".to_string(), 150.0);
    costs.insert("anthropic".to_string(), 100.0);

    let metrics = ProviderMetrics {
        total_provider_requests: 5000,
        provider_success_rates,
        provider_response_times: HashMap::new(),
        provider_errors: HashMap::new(),
        provider_usage: HashMap::new(),
        token_usage,
        costs,
    };

    assert_eq!(metrics.total_provider_requests, 5000);
    assert_eq!(metrics.provider_success_rates.get("openai"), Some(&99.5));
    assert_eq!(metrics.token_usage.get("openai"), Some(&1000000));
    assert_eq!(metrics.costs.get("anthropic"), Some(&100.0));
}

// ==================== SystemResourceMetrics Tests ====================

#[test]
fn test_system_resource_metrics() {
    let metrics = SystemResourceMetrics {
        cpu_usage: 75.5,
        memory_usage: 8 * 1024 * 1024 * 1024, // 8GB
        memory_usage_percent: 50.0,
        disk_usage: 100 * 1024 * 1024 * 1024, // 100GB
        disk_usage_percent: 40.0,
        network_bytes_in: 1024 * 1024 * 100, // 100MB
        network_bytes_out: 1024 * 1024 * 50, // 50MB
        active_connections: 500,
        database_connections: 50,
        redis_connections: 20,
    };

    assert_eq!(metrics.cpu_usage, 75.5);
    assert_eq!(metrics.memory_usage_percent, 50.0);
    assert_eq!(metrics.active_connections, 500);
}

#[test]
fn test_system_resource_metrics_idle() {
    let metrics = SystemResourceMetrics {
        cpu_usage: 1.0,
        memory_usage: 1024 * 1024 * 100, // 100MB
        memory_usage_percent: 1.0,
        disk_usage: 1024 * 1024 * 1024, // 1GB
        disk_usage_percent: 5.0,
        network_bytes_in: 0,
        network_bytes_out: 0,
        active_connections: 1,
        database_connections: 1,
        redis_connections: 0,
    };

    assert!(metrics.cpu_usage < 5.0);
    assert_eq!(metrics.network_bytes_in, 0);
}

// ==================== ErrorMetrics Tests ====================

#[test]
fn test_error_metrics() {
    let mut error_types = HashMap::new();
    error_types.insert("timeout".to_string(), 10_u64);
    error_types.insert("rate_limit".to_string(), 5_u64);
    error_types.insert("internal".to_string(), 2_u64);

    let mut error_endpoints = HashMap::new();
    error_endpoints.insert("/v1/chat/completions".to_string(), 15_u64);
    error_endpoints.insert("/v1/embeddings".to_string(), 2_u64);

    let metrics = ErrorMetrics {
        total_errors: 17,
        error_rate: 0.5,
        error_types,
        error_endpoints,
        critical_errors: 2,
        warnings: 5,
    };

    assert_eq!(metrics.total_errors, 17);
    assert_eq!(metrics.error_types.get("timeout"), Some(&10));
    assert_eq!(metrics.critical_errors, 2);
}

#[test]
fn test_error_metrics_no_errors() {
    let metrics = ErrorMetrics {
        total_errors: 0,
        error_rate: 0.0,
        error_types: HashMap::new(),
        error_endpoints: HashMap::new(),
        critical_errors: 0,
        warnings: 0,
    };

    assert_eq!(metrics.total_errors, 0);
    assert_eq!(metrics.error_rate, 0.0);
}

// ==================== PerformanceMetrics Tests ====================

#[test]
fn test_performance_metrics() {
    let metrics = PerformanceMetrics {
        cache_hit_rate: 90.0,
        cache_miss_rate: 10.0,
        avg_db_query_time_ms: 15.0,
        queue_depth: 5,
        throughput: 100.0,
        latency_percentiles: LatencyPercentiles {
            p50: 50.0,
            p90: 150.0,
            p95: 200.0,
            p99: 350.0,
            p999: 500.0,
        },
    };

    assert_eq!(metrics.cache_hit_rate, 90.0);
    assert_eq!(metrics.throughput, 100.0);
    assert_eq!(metrics.latency_percentiles.p50, 50.0);
}

#[test]
fn test_performance_metrics_cache_rates_sum() {
    let metrics = PerformanceMetrics {
        cache_hit_rate: 85.5,
        cache_miss_rate: 14.5,
        avg_db_query_time_ms: 20.0,
        queue_depth: 0,
        throughput: 50.0,
        latency_percentiles: LatencyPercentiles {
            p50: 100.0,
            p90: 200.0,
            p95: 300.0,
            p99: 500.0,
            p999: 800.0,
        },
    };

    // Cache hit + miss should equal 100%
    let sum = metrics.cache_hit_rate + metrics.cache_miss_rate;
    assert!((sum - 100.0).abs() < f64::EPSILON);
}

// ==================== LatencyPercentiles Tests ====================

#[test]
fn test_latency_percentiles() {
    let percentiles = LatencyPercentiles {
        p50: 100.0,
        p90: 200.0,
        p95: 300.0,
        p99: 500.0,
        p999: 800.0,
    };

    assert_eq!(percentiles.p50, 100.0);
    assert!(percentiles.p90 > percentiles.p50);
    assert!(percentiles.p95 > percentiles.p90);
    assert!(percentiles.p99 > percentiles.p95);
    assert!(percentiles.p999 > percentiles.p99);
}

#[test]
fn test_latency_percentiles_clone() {
    let percentiles = LatencyPercentiles {
        p50: 50.0,
        p90: 100.0,
        p95: 150.0,
        p99: 200.0,
        p999: 300.0,
    };

    let cloned = percentiles.clone();
    assert_eq!(percentiles.p50, cloned.p50);
    assert_eq!(percentiles.p99, cloned.p99);
}

// ==================== Serialization Tests ====================

#[test]
fn test_alert_serialization() {
    let alert = Alert {
        id: "test-alert".to_string(),
        severity: AlertSeverity::Warning,
        title: "Test".to_string(),
        description: "Test description".to_string(),
        timestamp: chrono::Utc::now(),
        source: "test".to_string(),
        metadata: serde_json::json!({"key": "value"}),
        resolved: false,
    };

    let json = serde_json::to_string(&alert);
    assert!(json.is_ok());
    let json_str = json.unwrap();
    assert!(json_str.contains("test-alert"));
    assert!(json_str.contains("Warning"));
}

#[test]
fn test_request_metrics_serialization() {
    let metrics = MonitoringRequestMetrics {
        total_requests: 100,
        requests_per_second: 5.0,
        avg_response_time_ms: 50.0,
        p95_response_time_ms: 100.0,
        p99_response_time_ms: 150.0,
        success_rate: 99.0,
        status_codes: HashMap::new(),
        endpoints: HashMap::new(),
    };

    let json = serde_json::to_string(&metrics);
    assert!(json.is_ok());
    assert!(json.unwrap().contains("total_requests"));
}

#[test]
fn test_latency_percentiles_serialization() {
    let percentiles = LatencyPercentiles {
        p50: 50.0,
        p90: 100.0,
        p95: 150.0,
        p99: 200.0,
        p999: 300.0,
    };

    let json = serde_json::to_string(&percentiles);
    assert!(json.is_ok());
    let json_str = json.unwrap();
    assert!(json_str.contains("p50"));
    assert!(json_str.contains("p999"));
}

// ==================== Clone Tests ====================

#[test]
fn test_alert_clone() {
    let alert = Alert {
        id: "original".to_string(),
        severity: AlertSeverity::Critical,
        title: "Test".to_string(),
        description: "Test".to_string(),
        timestamp: chrono::Utc::now(),
        source: "test".to_string(),
        metadata: serde_json::json!({}),
        resolved: false,
    };

    let cloned = alert.clone();
    assert_eq!(alert.id, cloned.id);
    assert_eq!(alert.severity, cloned.severity);
}

#[test]
fn test_system_metrics_clone() {
    let metrics = SystemMetrics {
        timestamp: chrono::Utc::now(),
        requests: MonitoringRequestMetrics {
            total_requests: 100,
            requests_per_second: 5.0,
            avg_response_time_ms: 50.0,
            p95_response_time_ms: 100.0,
            p99_response_time_ms: 150.0,
            success_rate: 99.0,
            status_codes: HashMap::new(),
            endpoints: HashMap::new(),
        },
        providers: ProviderMetrics {
            total_provider_requests: 100,
            provider_success_rates: HashMap::new(),
            provider_response_times: HashMap::new(),
            provider_errors: HashMap::new(),
            provider_usage: HashMap::new(),
            token_usage: HashMap::new(),
            costs: HashMap::new(),
        },
        system: SystemResourceMetrics {
            cpu_usage: 50.0,
            memory_usage: 1024 * 1024,
            memory_usage_percent: 10.0,
            disk_usage: 1024 * 1024 * 1024,
            disk_usage_percent: 20.0,
            network_bytes_in: 1024,
            network_bytes_out: 512,
            active_connections: 10,
            database_connections: 5,
            redis_connections: 2,
        },
        errors: ErrorMetrics {
            total_errors: 1,
            error_rate: 0.01,
            error_types: HashMap::new(),
            error_endpoints: HashMap::new(),
            critical_errors: 0,
            warnings: 1,
        },
        performance: PerformanceMetrics {
            cache_hit_rate: 80.0,
            cache_miss_rate: 20.0,
            avg_db_query_time_ms: 10.0,
            queue_depth: 0,
            throughput: 5.0,
            latency_percentiles: LatencyPercentiles {
                p50: 50.0,
                p90: 100.0,
                p95: 150.0,
                p99: 200.0,
                p999: 300.0,
            },
        },
    };

    let cloned = metrics.clone();
    assert_eq!(
        metrics.requests.total_requests,
        cloned.requests.total_requests
    );
    assert_eq!(metrics.system.cpu_usage, cloned.system.cpu_usage);
}
