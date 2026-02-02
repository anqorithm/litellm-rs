//! Core types for the Alerting system

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

/// Error types for alerting operations
#[derive(Debug, Error)]
pub enum AlertError {
    /// Configuration error
    #[error("Configuration error: {0}")]
    Config(String),

    /// Network error
    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),

    /// Serialization error
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    /// Channel error
    #[error("Channel error: {0}")]
    Channel(String),

    /// Rate limited
    #[error("Rate limited: {0}")]
    RateLimited(String),
}

/// Result type for alerting operations
pub type AlertResult<T> = Result<T, AlertError>;

/// Alert severity level
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum AlertLevel {
    /// Debug level - for testing
    Debug,
    /// Info level - informational alerts
    #[default]
    Info,
    /// Warning level - potential issues
    Warning,
    /// Error level - errors requiring attention
    Error,
    /// Critical level - critical issues requiring immediate action
    Critical,
}

impl AlertLevel {
    /// Get emoji for Slack messages
    pub fn emoji(&self) -> &'static str {
        match self {
            AlertLevel::Debug => ":bug:",
            AlertLevel::Info => ":information_source:",
            AlertLevel::Warning => ":warning:",
            AlertLevel::Error => ":x:",
            AlertLevel::Critical => ":rotating_light:",
        }
    }

    /// Get color for Slack attachments
    pub fn color(&self) -> &'static str {
        match self {
            AlertLevel::Debug => "#808080",
            AlertLevel::Info => "#36a64f",
            AlertLevel::Warning => "#ffcc00",
            AlertLevel::Error => "#ff0000",
            AlertLevel::Critical => "#8b0000",
        }
    }

    /// Check if this level should trigger given a minimum level
    pub fn should_alert(&self, min_level: AlertLevel) -> bool {
        self.priority() >= min_level.priority()
    }

    /// Get numeric priority (higher = more important)
    fn priority(&self) -> u8 {
        match self {
            AlertLevel::Debug => 0,
            AlertLevel::Info => 1,
            AlertLevel::Warning => 2,
            AlertLevel::Error => 3,
            AlertLevel::Critical => 4,
        }
    }
}

/// Type of alert
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AlertType {
    /// Budget threshold exceeded
    BudgetExceeded,
    /// Budget warning (approaching limit)
    BudgetWarning,
    /// Error rate exceeded threshold
    ErrorRateHigh,
    /// Latency exceeded threshold
    LatencyHigh,
    /// Provider failure
    ProviderFailure,
    /// Rate limit exceeded
    RateLimitExceeded,
    /// Authentication failure
    AuthFailure,
    /// System health issue
    SystemHealth,
    /// Custom alert
    Custom(String),
}

impl AlertType {
    /// Get default alert level for this type
    pub fn default_level(&self) -> AlertLevel {
        match self {
            AlertType::BudgetExceeded => AlertLevel::Error,
            AlertType::BudgetWarning => AlertLevel::Warning,
            AlertType::ErrorRateHigh => AlertLevel::Error,
            AlertType::LatencyHigh => AlertLevel::Warning,
            AlertType::ProviderFailure => AlertLevel::Error,
            AlertType::RateLimitExceeded => AlertLevel::Warning,
            AlertType::AuthFailure => AlertLevel::Warning,
            AlertType::SystemHealth => AlertLevel::Critical,
            AlertType::Custom(_) => AlertLevel::Info,
        }
    }

    /// Get alert type name
    pub fn name(&self) -> &str {
        match self {
            AlertType::BudgetExceeded => "budget_exceeded",
            AlertType::BudgetWarning => "budget_warning",
            AlertType::ErrorRateHigh => "error_rate_high",
            AlertType::LatencyHigh => "latency_high",
            AlertType::ProviderFailure => "provider_failure",
            AlertType::RateLimitExceeded => "rate_limit_exceeded",
            AlertType::AuthFailure => "auth_failure",
            AlertType::SystemHealth => "system_health",
            AlertType::Custom(name) => name,
        }
    }
}

/// An alert to be sent
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Alert {
    /// Unique alert ID
    pub id: String,
    /// Alert level
    pub level: AlertLevel,
    /// Alert type
    pub alert_type: AlertType,
    /// Alert title
    pub title: String,
    /// Alert message
    pub message: String,
    /// Timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// Source of the alert
    #[serde(default)]
    pub source: Option<String>,
    /// Additional metadata
    #[serde(default)]
    pub metadata: HashMap<String, serde_json::Value>,
    /// Tags for filtering
    #[serde(default)]
    pub tags: Vec<String>,
}

impl Alert {
    /// Create a new alert
    pub fn new(level: AlertLevel, title: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            level,
            alert_type: AlertType::Custom("custom".to_string()),
            title: title.into(),
            message: message.into(),
            timestamp: chrono::Utc::now(),
            source: None,
            metadata: HashMap::new(),
            tags: Vec::new(),
        }
    }

    /// Create a budget exceeded alert
    pub fn budget_exceeded(
        budget_name: impl Into<String>,
        current: f64,
        limit: f64,
    ) -> Self {
        let budget_name = budget_name.into();
        Self::new(
            AlertLevel::Error,
            format!("Budget Exceeded: {}", budget_name),
            format!(
                "Budget '{}' has exceeded its limit. Current: ${:.2}, Limit: ${:.2}",
                budget_name, current, limit
            ),
        )
        .with_type(AlertType::BudgetExceeded)
        .with_metadata("budget_name", serde_json::json!(budget_name))
        .with_metadata("current", serde_json::json!(current))
        .with_metadata("limit", serde_json::json!(limit))
    }

    /// Create a budget warning alert
    pub fn budget_warning(
        budget_name: impl Into<String>,
        current: f64,
        limit: f64,
        percentage: f64,
    ) -> Self {
        let budget_name = budget_name.into();
        Self::new(
            AlertLevel::Warning,
            format!("Budget Warning: {}", budget_name),
            format!(
                "Budget '{}' is at {:.1}% of limit. Current: ${:.2}, Limit: ${:.2}",
                budget_name, percentage * 100.0, current, limit
            ),
        )
        .with_type(AlertType::BudgetWarning)
        .with_metadata("budget_name", serde_json::json!(budget_name))
        .with_metadata("current", serde_json::json!(current))
        .with_metadata("limit", serde_json::json!(limit))
        .with_metadata("percentage", serde_json::json!(percentage))
    }

    /// Create an error rate alert
    pub fn error_rate_high(
        service: impl Into<String>,
        error_rate: f64,
        threshold: f64,
    ) -> Self {
        let service = service.into();
        Self::new(
            AlertLevel::Error,
            format!("High Error Rate: {}", service),
            format!(
                "Service '{}' error rate is {:.1}%, exceeding threshold of {:.1}%",
                service, error_rate * 100.0, threshold * 100.0
            ),
        )
        .with_type(AlertType::ErrorRateHigh)
        .with_metadata("service", serde_json::json!(service))
        .with_metadata("error_rate", serde_json::json!(error_rate))
        .with_metadata("threshold", serde_json::json!(threshold))
    }

    /// Create a provider failure alert
    pub fn provider_failure(provider: impl Into<String>, error: impl Into<String>) -> Self {
        let provider = provider.into();
        Self::new(
            AlertLevel::Error,
            format!("Provider Failure: {}", provider),
            format!("Provider '{}' failed: {}", provider, error.into()),
        )
        .with_type(AlertType::ProviderFailure)
        .with_metadata("provider", serde_json::json!(provider))
    }

    /// Set alert type
    pub fn with_type(mut self, alert_type: AlertType) -> Self {
        self.alert_type = alert_type;
        self
    }

    /// Set source
    pub fn with_source(mut self, source: impl Into<String>) -> Self {
        self.source = Some(source.into());
        self
    }

    /// Add metadata
    pub fn with_metadata(mut self, key: impl Into<String>, value: serde_json::Value) -> Self {
        self.metadata.insert(key.into(), value);
        self
    }

    /// Add tag
    pub fn with_tag(mut self, tag: impl Into<String>) -> Self {
        self.tags.push(tag.into());
        self
    }

    /// Convert to JSON
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_alert_level_priority() {
        assert!(AlertLevel::Error.should_alert(AlertLevel::Warning));
        assert!(AlertLevel::Warning.should_alert(AlertLevel::Warning));
        assert!(!AlertLevel::Info.should_alert(AlertLevel::Warning));
        assert!(AlertLevel::Critical.should_alert(AlertLevel::Error));
    }

    #[test]
    fn test_alert_level_emoji() {
        assert_eq!(AlertLevel::Warning.emoji(), ":warning:");
        assert_eq!(AlertLevel::Error.emoji(), ":x:");
        assert_eq!(AlertLevel::Critical.emoji(), ":rotating_light:");
    }

    #[test]
    fn test_alert_level_color() {
        assert_eq!(AlertLevel::Warning.color(), "#ffcc00");
        assert_eq!(AlertLevel::Error.color(), "#ff0000");
    }

    #[test]
    fn test_alert_type_default_level() {
        assert_eq!(AlertType::BudgetExceeded.default_level(), AlertLevel::Error);
        assert_eq!(AlertType::BudgetWarning.default_level(), AlertLevel::Warning);
        assert_eq!(AlertType::SystemHealth.default_level(), AlertLevel::Critical);
    }

    #[test]
    fn test_alert_creation() {
        let alert = Alert::new(AlertLevel::Warning, "Test Alert", "This is a test");
        assert_eq!(alert.level, AlertLevel::Warning);
        assert_eq!(alert.title, "Test Alert");
        assert_eq!(alert.message, "This is a test");
    }

    #[test]
    fn test_budget_exceeded_alert() {
        let alert = Alert::budget_exceeded("monthly", 150.0, 100.0);
        assert_eq!(alert.level, AlertLevel::Error);
        assert_eq!(alert.alert_type, AlertType::BudgetExceeded);
        assert!(alert.metadata.contains_key("current"));
        assert!(alert.metadata.contains_key("limit"));
    }

    #[test]
    fn test_budget_warning_alert() {
        let alert = Alert::budget_warning("monthly", 80.0, 100.0, 0.8);
        assert_eq!(alert.level, AlertLevel::Warning);
        assert_eq!(alert.alert_type, AlertType::BudgetWarning);
        assert!(alert.message.contains("80.0%"));
    }

    #[test]
    fn test_error_rate_alert() {
        let alert = Alert::error_rate_high("api-gateway", 0.15, 0.05);
        assert_eq!(alert.level, AlertLevel::Error);
        assert_eq!(alert.alert_type, AlertType::ErrorRateHigh);
    }

    #[test]
    fn test_provider_failure_alert() {
        let alert = Alert::provider_failure("openai", "Connection timeout");
        assert_eq!(alert.level, AlertLevel::Error);
        assert_eq!(alert.alert_type, AlertType::ProviderFailure);
    }

    #[test]
    fn test_alert_builder() {
        let alert = Alert::new(AlertLevel::Info, "Test", "Message")
            .with_type(AlertType::Custom("test".to_string()))
            .with_source("test-service")
            .with_metadata("key", serde_json::json!("value"))
            .with_tag("production");

        assert_eq!(alert.source, Some("test-service".to_string()));
        assert!(alert.metadata.contains_key("key"));
        assert!(alert.tags.contains(&"production".to_string()));
    }

    #[test]
    fn test_alert_serialization() {
        let alert = Alert::new(AlertLevel::Warning, "Test", "Message");
        let json = alert.to_json().unwrap();

        assert!(json.contains("warning"));
        assert!(json.contains("Test"));

        let deserialized: Alert = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.level, alert.level);
    }
}
