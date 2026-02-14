//! Alert system tests

#[cfg(test)]
use super::channels::SlackChannel;
use super::types::{AlertRule, AlertStats, ComparisonOperator};
use crate::monitoring::types::AlertSeverity;
use std::time::Duration;

#[test]
fn test_alert_rule_creation() {
    let rule = AlertRule {
        id: "test-rule".to_string(),
        name: "High CPU Usage".to_string(),
        description: "Alert when CPU usage exceeds 80%".to_string(),
        metric: "cpu_usage".to_string(),
        threshold: 80.0,
        operator: ComparisonOperator::GreaterThan,
        severity: AlertSeverity::Warning,
        interval: Duration::from_secs(60),
        enabled: true,
        channels: vec!["slack".to_string()],
    };

    assert_eq!(rule.name, "High CPU Usage");
    assert_eq!(rule.threshold, 80.0);
    assert!(rule.enabled);
}

#[test]
fn test_comparison_operators() {
    assert_eq!(
        ComparisonOperator::GreaterThan,
        ComparisonOperator::GreaterThan
    );
    assert_ne!(
        ComparisonOperator::GreaterThan,
        ComparisonOperator::LessThan
    );
}

#[test]
fn test_slack_channel_creation() {
    let channel = SlackChannel::new(
        "https://hooks.slack.com/test".to_string(),
        Some("#alerts".to_string()),
        Some("Gateway".to_string()),
        AlertSeverity::Warning,
    );

    use super::channels::NotificationChannel;
    assert_eq!(channel.name(), "slack");
    assert!(channel.supports_severity(AlertSeverity::Critical));
    assert!(!channel.supports_severity(AlertSeverity::Info));
}

#[test]
fn test_alert_stats() {
    let mut stats = AlertStats {
        total_alerts: 10,
        ..Default::default()
    };
    stats.alerts_by_severity.insert("Warning".to_string(), 5);
    stats.alerts_by_severity.insert("Critical".to_string(), 3);

    assert_eq!(stats.total_alerts, 10);
    assert_eq!(stats.alerts_by_severity.get("Warning"), Some(&5));
}
