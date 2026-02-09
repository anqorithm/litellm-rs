use super::*;

// ==================== ProviderHealthStatus Tests ====================

#[test]
fn test_provider_health_status_default() {
    let status = ProviderHealthStatus::default();
    assert!(status.healthy);
    assert!(status.last_success.is_none());
    assert!(status.last_error.is_none());
    assert!(status.response_time.is_none());
    assert_eq!(status.consecutive_failures, 0);
}

#[test]
fn test_provider_health_status_clone() {
    let status = ProviderHealthStatus {
        healthy: false,
        consecutive_failures: 5,
        last_error: Some("Connection refused".to_string()),
        response_time: Some(Duration::from_millis(100)),
        ..Default::default()
    };

    let cloned = status.clone();
    assert!(!cloned.healthy);
    assert_eq!(cloned.consecutive_failures, 5);
    assert_eq!(cloned.last_error, Some("Connection refused".to_string()));
    assert_eq!(cloned.response_time, Some(Duration::from_millis(100)));
}

#[test]
fn test_provider_health_status_debug() {
    let status = ProviderHealthStatus::default();
    let debug = format!("{:?}", status);
    assert!(debug.contains("ProviderHealthStatus"));
    assert!(debug.contains("healthy"));
    assert!(debug.contains("consecutive_failures"));
}

#[test]
fn test_provider_health_status_with_success() {
    let status = ProviderHealthStatus {
        healthy: true,
        last_success: Some(Instant::now()),
        response_time: Some(Duration::from_millis(50)),
        consecutive_failures: 0,
        ..Default::default()
    };

    assert!(status.healthy);
    assert!(status.last_success.is_some());
    assert_eq!(status.response_time, Some(Duration::from_millis(50)));
}

#[test]
fn test_provider_health_status_with_error() {
    let status = ProviderHealthStatus {
        healthy: false,
        last_error: Some("Timeout".to_string()),
        consecutive_failures: 3,
        ..Default::default()
    };

    assert!(!status.healthy);
    assert_eq!(status.last_error, Some("Timeout".to_string()));
    assert_eq!(status.consecutive_failures, 3);
}

#[test]
fn test_provider_health_status_reset_after_success() {
    // Start with a status that has failures
    let mut status = ProviderHealthStatus {
        consecutive_failures: 2,
        last_error: Some("Previous error".to_string()),
        ..Default::default()
    };

    // Simulate success - this resets the counters
    status.healthy = true;
    status.consecutive_failures = 0;
    status.last_success = Some(Instant::now());
    status.last_error = None;
    status.response_time = Some(Duration::from_millis(25));

    assert!(status.healthy);
    assert_eq!(status.consecutive_failures, 0);
    assert!(status.last_error.is_none());
}

// ==================== RouterHealthStatus Tests ====================

#[test]
fn test_router_health_status_debug() {
    let status = RouterHealthStatus {
        healthy: true,
        providers: HashMap::new(),
        last_check: Instant::now(),
    };
    let debug = format!("{:?}", status);
    assert!(debug.contains("RouterHealthStatus"));
    assert!(debug.contains("healthy"));
}

#[test]
fn test_router_health_status_clone() {
    let mut providers = HashMap::new();
    providers.insert("openai".to_string(), ProviderHealthStatus::default());

    let status = RouterHealthStatus {
        healthy: true,
        providers,
        last_check: Instant::now(),
    };

    let cloned = status.clone();
    assert!(cloned.healthy);
    assert!(cloned.providers.contains_key("openai"));
}

#[test]
fn test_router_health_status_empty_providers() {
    let status = RouterHealthStatus {
        healthy: false,
        providers: HashMap::new(),
        last_check: Instant::now(),
    };

    assert!(!status.healthy);
    assert!(status.providers.is_empty());
}

#[test]
fn test_router_health_status_with_mixed_providers() {
    let mut providers = HashMap::new();

    let healthy_provider = ProviderHealthStatus {
        healthy: true,
        ..Default::default()
    };

    let unhealthy_provider = ProviderHealthStatus {
        healthy: false,
        ..Default::default()
    };

    providers.insert("openai".to_string(), healthy_provider);
    providers.insert("anthropic".to_string(), unhealthy_provider);

    let status = RouterHealthStatus {
        healthy: true, // At least one provider is healthy
        providers,
        last_check: Instant::now(),
    };

    assert!(status.healthy);
    assert_eq!(status.providers.len(), 2);
    assert!(status.providers.get("openai").unwrap().healthy);
    assert!(!status.providers.get("anthropic").unwrap().healthy);
}

// ==================== Health Status Calculation Tests ====================

#[test]
fn test_overall_health_any_healthy() {
    let mut providers = HashMap::new();

    let status1 = ProviderHealthStatus {
        healthy: false,
        ..Default::default()
    };

    let status2 = ProviderHealthStatus {
        healthy: true,
        ..Default::default()
    };

    let status3 = ProviderHealthStatus {
        healthy: false,
        ..Default::default()
    };

    providers.insert("p1".to_string(), status1);
    providers.insert("p2".to_string(), status2);
    providers.insert("p3".to_string(), status3);

    // At least one provider is healthy
    let overall_healthy = providers.values().any(|s| s.healthy);
    assert!(overall_healthy);
}

#[test]
fn test_overall_health_all_unhealthy() {
    let mut providers = HashMap::new();

    let status1 = ProviderHealthStatus {
        healthy: false,
        ..Default::default()
    };

    let status2 = ProviderHealthStatus {
        healthy: false,
        ..Default::default()
    };

    providers.insert("p1".to_string(), status1);
    providers.insert("p2".to_string(), status2);

    // No provider is healthy
    let overall_healthy = providers.values().any(|s| s.healthy);
    assert!(!overall_healthy);
}

#[test]
fn test_overall_health_all_healthy() {
    let mut providers = HashMap::new();

    let status1 = ProviderHealthStatus {
        healthy: true,
        ..Default::default()
    };

    let status2 = ProviderHealthStatus {
        healthy: true,
        ..Default::default()
    };

    providers.insert("p1".to_string(), status1);
    providers.insert("p2".to_string(), status2);

    let overall_healthy = providers.values().any(|s| s.healthy);
    assert!(overall_healthy);

    let all_healthy = providers.values().all(|s| s.healthy);
    assert!(all_healthy);
}

// ==================== Failure Counting Tests ====================

#[test]
fn test_consecutive_failures_increment() {
    let mut status = ProviderHealthStatus::default();
    assert_eq!(status.consecutive_failures, 0);

    status.consecutive_failures += 1;
    assert_eq!(status.consecutive_failures, 1);

    status.consecutive_failures += 1;
    assert_eq!(status.consecutive_failures, 2);

    status.consecutive_failures += 1;
    assert_eq!(status.consecutive_failures, 3);
}

#[test]
fn test_consecutive_failures_threshold() {
    let max_failures = 3u32;

    // Below threshold - still healthy
    let status_below = ProviderHealthStatus {
        consecutive_failures: 2,
        healthy: true,
        ..Default::default()
    };
    assert!(status_below.consecutive_failures < max_failures);
    assert!(status_below.healthy);

    // At threshold - should be unhealthy
    let status_at = ProviderHealthStatus {
        consecutive_failures: 3,
        healthy: false,
        ..Default::default()
    };
    assert!(status_at.consecutive_failures >= max_failures);
    assert!(!status_at.healthy);
}

#[test]
fn test_failure_reset_on_success() {
    // Start with accumulated failures (still healthy, not yet at threshold)
    let mut status = ProviderHealthStatus {
        consecutive_failures: 2,
        last_error: Some("Error".to_string()),
        healthy: true,
        ..Default::default()
    };

    // Success resets counters
    status.consecutive_failures = 0;
    status.last_success = Some(Instant::now());
    status.last_error = None;

    assert_eq!(status.consecutive_failures, 0);
    assert!(status.last_error.is_none());
    assert!(status.last_success.is_some());
}

// ==================== Response Time Tests ====================

#[test]
fn test_response_time_tracking() {
    let mut status = ProviderHealthStatus::default();
    assert!(status.response_time.is_none());

    status.response_time = Some(Duration::from_millis(150));
    assert_eq!(status.response_time, Some(Duration::from_millis(150)));

    // Update response time
    status.response_time = Some(Duration::from_millis(75));
    assert_eq!(status.response_time, Some(Duration::from_millis(75)));
}

#[test]
fn test_response_time_timeout_check() {
    let timeout = Duration::from_secs(10);
    let response_time = Duration::from_millis(500);

    // Fast response - healthy
    assert!(response_time <= timeout);

    let slow_response = Duration::from_secs(15);
    // Slow response - timeout
    assert!(slow_response > timeout);
}

// ==================== Provider Filtering Tests ====================

#[test]
fn test_filter_healthy_providers() {
    let mut providers = HashMap::new();

    let status1 = ProviderHealthStatus {
        healthy: true,
        ..Default::default()
    };

    let status2 = ProviderHealthStatus {
        healthy: false,
        ..Default::default()
    };

    let status3 = ProviderHealthStatus {
        healthy: true,
        ..Default::default()
    };

    providers.insert("openai".to_string(), status1);
    providers.insert("anthropic".to_string(), status2);
    providers.insert("google".to_string(), status3);

    let healthy: Vec<String> = providers
        .iter()
        .filter(|(_, status)| status.healthy)
        .map(|(name, _)| name.clone())
        .collect();

    assert_eq!(healthy.len(), 2);
    assert!(healthy.contains(&"openai".to_string()));
    assert!(healthy.contains(&"google".to_string()));
    assert!(!healthy.contains(&"anthropic".to_string()));
}

#[test]
fn test_filter_unhealthy_providers() {
    let mut providers = HashMap::new();

    let status1 = ProviderHealthStatus {
        healthy: true,
        ..Default::default()
    };

    let status2 = ProviderHealthStatus {
        healthy: false,
        last_error: Some("Rate limited".to_string()),
        ..Default::default()
    };

    providers.insert("openai".to_string(), status1);
    providers.insert("anthropic".to_string(), status2);

    let unhealthy: Vec<String> = providers
        .iter()
        .filter(|(_, status)| !status.healthy)
        .map(|(name, _)| name.clone())
        .collect();

    assert_eq!(unhealthy.len(), 1);
    assert!(unhealthy.contains(&"anthropic".to_string()));
}

// ==================== Edge Cases ====================

#[test]
fn test_empty_provider_map() {
    let providers: HashMap<String, ProviderHealthStatus> = HashMap::new();

    let healthy: Vec<String> = providers
        .iter()
        .filter(|(_, status)| status.healthy)
        .map(|(name, _)| name.clone())
        .collect();

    assert!(healthy.is_empty());

    // Overall health with no providers
    let overall_healthy = providers.values().any(|s| s.healthy);
    assert!(!overall_healthy);
}

#[test]
fn test_status_with_long_error_message() {
    let mut status = ProviderHealthStatus::default();
    let long_error = "a".repeat(10000);
    status.last_error = Some(long_error.clone());

    assert_eq!(status.last_error.as_ref().unwrap().len(), 10000);
}

#[test]
fn test_status_timestamps() {
    let before = Instant::now();
    let status = ProviderHealthStatus::default();
    let after = Instant::now();

    // last_check should be between before and after
    assert!(status.last_check >= before);
    assert!(status.last_check <= after);
}

#[test]
fn test_high_failure_count() {
    let status = ProviderHealthStatus {
        consecutive_failures: u32::MAX,
        ..Default::default()
    };
    assert_eq!(status.consecutive_failures, u32::MAX);
}
