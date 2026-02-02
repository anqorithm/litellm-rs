//! Integration tests for the Alerting system

use super::*;
use self::channels::MemoryChannel;
use self::config::{AlertConfig, SlackConfig, WebhookConfig};
use self::manager::AlertManager;
use self::types::{Alert, AlertLevel, AlertType};
use std::sync::Arc;

// ============================================================================
// Integration Tests
// ============================================================================

#[tokio::test]
async fn test_full_alert_pipeline() {
    let config = AlertConfig::new().enable();
    let manager = AlertManager::new(config).unwrap();

    // Send various alerts
    manager.send(Alert::new(AlertLevel::Warning, "Test Warning", "Warning message")).await.unwrap();
    manager.send(Alert::budget_exceeded("monthly", 150.0, 100.0)).await.unwrap();
    manager.send(Alert::error_rate_high("api", 0.15, 0.05)).await.unwrap();
    manager.send(Alert::provider_failure("openai", "Timeout")).await.unwrap();
}

#[tokio::test]
async fn test_budget_alerts() {
    let config = AlertConfig::new()
        .enable()
        .with_budget_warning_threshold(0.8);

    let manager = AlertManager::new(config).unwrap();

    // Budget warning at 80%
    let alert = Alert::budget_warning("monthly", 80.0, 100.0, 0.8);
    manager.send(alert).await.unwrap();

    // Budget exceeded
    let alert = Alert::budget_exceeded("monthly", 150.0, 100.0);
    manager.send(alert).await.unwrap();
}

#[tokio::test]
async fn test_error_rate_alerts() {
    let config = AlertConfig::new()
        .enable()
        .with_error_rate_threshold(0.05);

    let manager = AlertManager::new(config).unwrap();

    let alert = Alert::error_rate_high("api-gateway", 0.10, 0.05);
    manager.send(alert).await.unwrap();
}

#[tokio::test]
async fn test_alert_level_filtering() {
    let config = AlertConfig::new()
        .enable()
        .with_min_level(AlertLevel::Error);

    let manager = AlertManager::new(config).unwrap();

    // These should be filtered
    manager.send(Alert::new(AlertLevel::Debug, "Debug", "msg")).await.unwrap();
    manager.send(Alert::new(AlertLevel::Info, "Info", "msg")).await.unwrap();
    manager.send(Alert::new(AlertLevel::Warning, "Warning", "msg")).await.unwrap();

    // These should pass
    manager.send(Alert::new(AlertLevel::Error, "Error", "msg")).await.unwrap();
    manager.send(Alert::new(AlertLevel::Critical, "Critical", "msg")).await.unwrap();
}

#[tokio::test]
async fn test_memory_channel_integration() {
    let channel = Arc::new(MemoryChannel::new());

    // Send multiple alerts
    for i in 0..5 {
        let alert = Alert::new(AlertLevel::Info, format!("Alert {}", i), "Message");
        channel.send(&alert).await.unwrap();
    }

    assert_eq!(channel.count().await, 5);

    let alerts = channel.alerts().await;
    assert!(alerts[0].title.contains("Alert 0"));
    assert!(alerts[4].title.contains("Alert 4"));
}

#[test]
fn test_config_serialization_roundtrip() {
    let config = AlertConfig::new()
        .enable()
        .with_min_level(AlertLevel::Warning)
        .with_slack(SlackConfig::new("https://hooks.slack.com/test"))
        .with_webhook(WebhookConfig::new("pagerduty", "https://events.pagerduty.com"))
        .with_budget_warning_threshold(0.9)
        .with_error_rate_threshold(0.1)
        .with_cooldown(600);

    let json = serde_json::to_string(&config).unwrap();
    let deserialized: AlertConfig = serde_json::from_str(&json).unwrap();

    assert_eq!(config.enabled, deserialized.enabled);
    assert_eq!(config.min_level, deserialized.min_level);
    assert!(deserialized.slack.is_some());
    assert_eq!(deserialized.webhooks.len(), 1);
}

#[test]
fn test_yaml_config() {
    let yaml = r#"
enabled: true
min_level: warning
slack:
  enabled: true
  webhook_url: https://hooks.slack.com/services/xxx
  channel: alerts
  username: LiteLLM Gateway
webhooks:
  - name: pagerduty
    enabled: true
    url: https://events.pagerduty.com/v2/enqueue
    method: POST
    timeout_ms: 10000
    retries: 3
budget_alerts: true
budget_warning_threshold: 0.8
error_rate_alerts: true
error_rate_threshold: 0.05
rate_limit_per_minute: 60
cooldown_seconds: 300
"#;

    let config: AlertConfig = serde_yaml::from_str(yaml).unwrap();
    assert!(config.enabled);
    assert!(config.slack.is_some());
    assert_eq!(config.webhooks.len(), 1);
    assert_eq!(config.budget_warning_threshold, 0.8);
}

// ============================================================================
// Edge Cases
// ============================================================================

#[tokio::test]
async fn test_disabled_manager() {
    let manager = AlertManager::disabled();

    // Should not error
    let alert = Alert::new(AlertLevel::Critical, "Critical", "Message");
    manager.send(alert).await.unwrap();
}

#[tokio::test]
async fn test_suppressed_alert_types() {
    let config = AlertConfig::new()
        .enable()
        .suppress_type("budget_warning")
        .suppress_type("latency_high");

    let manager = AlertManager::new(config).unwrap();

    // These should be suppressed
    manager.send(Alert::budget_warning("test", 80.0, 100.0, 0.8)).await.unwrap();

    // These should pass
    manager.send(Alert::budget_exceeded("test", 150.0, 100.0)).await.unwrap();
}

#[tokio::test]
async fn test_concurrent_alerts() {
    let config = AlertConfig::new().enable();
    let manager = Arc::new(AlertManager::new(config).unwrap());

    let mut handles = Vec::new();

    for i in 0..10 {
        let manager = manager.clone();
        let handle = tokio::spawn(async move {
            for j in 0..10 {
                let alert = Alert::new(
                    AlertLevel::Info,
                    format!("Thread {} Alert {}", i, j),
                    "Message",
                );
                let _ = manager.send(alert).await;
            }
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.await.unwrap();
    }
}

#[test]
fn test_alert_serialization() {
    let alert = Alert::budget_exceeded("monthly", 150.0, 100.0)
        .with_source("budget-service")
        .with_tag("production")
        .with_tag("critical");

    let json = alert.to_json().unwrap();
    let deserialized: Alert = serde_json::from_str(&json).unwrap();

    assert_eq!(alert.level, deserialized.level);
    assert_eq!(alert.alert_type, deserialized.alert_type);
    assert!(deserialized.metadata.contains_key("current"));
    assert!(deserialized.metadata.contains_key("limit"));
}

#[test]
fn test_alert_type_names() {
    assert_eq!(AlertType::BudgetExceeded.name(), "budget_exceeded");
    assert_eq!(AlertType::BudgetWarning.name(), "budget_warning");
    assert_eq!(AlertType::ErrorRateHigh.name(), "error_rate_high");
    assert_eq!(AlertType::ProviderFailure.name(), "provider_failure");
    assert_eq!(AlertType::Custom("test".to_string()).name(), "test");
}

#[test]
fn test_alert_type_default_levels() {
    assert_eq!(AlertType::BudgetExceeded.default_level(), AlertLevel::Error);
    assert_eq!(AlertType::BudgetWarning.default_level(), AlertLevel::Warning);
    assert_eq!(AlertType::SystemHealth.default_level(), AlertLevel::Critical);
}
