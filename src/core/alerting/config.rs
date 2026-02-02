//! Configuration for the Alerting system

use serde::{Deserialize, Serialize};
use std::collections::HashSet;

use super::types::AlertLevel;

/// Main configuration for the alerting system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertConfig {
    /// Whether alerting is enabled
    #[serde(default)]
    pub enabled: bool,

    /// Minimum alert level to send
    #[serde(default)]
    pub min_level: AlertLevel,

    /// Slack configuration
    #[serde(default)]
    pub slack: Option<SlackConfig>,

    /// Webhook configurations
    #[serde(default)]
    pub webhooks: Vec<WebhookConfig>,

    /// Enable budget alerts
    #[serde(default = "default_true")]
    pub budget_alerts: bool,

    /// Budget warning threshold (0.0 - 1.0)
    #[serde(default = "default_budget_warning_threshold")]
    pub budget_warning_threshold: f64,

    /// Enable error rate alerts
    #[serde(default = "default_true")]
    pub error_rate_alerts: bool,

    /// Error rate threshold (0.0 - 1.0)
    #[serde(default = "default_error_rate_threshold")]
    pub error_rate_threshold: f64,

    /// Enable latency alerts
    #[serde(default)]
    pub latency_alerts: bool,

    /// Latency threshold in milliseconds
    #[serde(default = "default_latency_threshold")]
    pub latency_threshold_ms: u64,

    /// Rate limit for alerts (max alerts per minute)
    #[serde(default = "default_rate_limit")]
    pub rate_limit_per_minute: u32,

    /// Cooldown period in seconds (prevent duplicate alerts)
    #[serde(default = "default_cooldown")]
    pub cooldown_seconds: u64,

    /// Alert types to suppress
    #[serde(default)]
    pub suppressed_types: HashSet<String>,
}

fn default_true() -> bool {
    true
}

fn default_budget_warning_threshold() -> f64 {
    0.8 // 80%
}

fn default_error_rate_threshold() -> f64 {
    0.05 // 5%
}

fn default_latency_threshold() -> u64 {
    5000 // 5 seconds
}

fn default_rate_limit() -> u32 {
    60 // 60 alerts per minute
}

fn default_cooldown() -> u64 {
    300 // 5 minutes
}

impl Default for AlertConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            min_level: AlertLevel::Warning,
            slack: None,
            webhooks: Vec::new(),
            budget_alerts: true,
            budget_warning_threshold: default_budget_warning_threshold(),
            error_rate_alerts: true,
            error_rate_threshold: default_error_rate_threshold(),
            latency_alerts: false,
            latency_threshold_ms: default_latency_threshold(),
            rate_limit_per_minute: default_rate_limit(),
            cooldown_seconds: default_cooldown(),
            suppressed_types: HashSet::new(),
        }
    }
}

impl AlertConfig {
    /// Create a new alert config
    pub fn new() -> Self {
        Self::default()
    }

    /// Enable alerting
    pub fn enable(mut self) -> Self {
        self.enabled = true;
        self
    }

    /// Set minimum alert level
    pub fn with_min_level(mut self, level: AlertLevel) -> Self {
        self.min_level = level;
        self
    }

    /// Configure Slack
    pub fn with_slack(mut self, config: SlackConfig) -> Self {
        self.slack = Some(config);
        self
    }

    /// Add a webhook
    pub fn with_webhook(mut self, config: WebhookConfig) -> Self {
        self.webhooks.push(config);
        self
    }

    /// Set budget warning threshold
    pub fn with_budget_warning_threshold(mut self, threshold: f64) -> Self {
        self.budget_warning_threshold = threshold.clamp(0.0, 1.0);
        self
    }

    /// Set error rate threshold
    pub fn with_error_rate_threshold(mut self, threshold: f64) -> Self {
        self.error_rate_threshold = threshold.clamp(0.0, 1.0);
        self
    }

    /// Set latency threshold
    pub fn with_latency_threshold(mut self, ms: u64) -> Self {
        self.latency_threshold_ms = ms;
        self.latency_alerts = true;
        self
    }

    /// Set rate limit
    pub fn with_rate_limit(mut self, per_minute: u32) -> Self {
        self.rate_limit_per_minute = per_minute;
        self
    }

    /// Set cooldown period
    pub fn with_cooldown(mut self, seconds: u64) -> Self {
        self.cooldown_seconds = seconds;
        self
    }

    /// Suppress an alert type
    pub fn suppress_type(mut self, alert_type: impl Into<String>) -> Self {
        self.suppressed_types.insert(alert_type.into());
        self
    }

    /// Check if an alert type is suppressed
    pub fn is_suppressed(&self, alert_type: &str) -> bool {
        self.suppressed_types.contains(alert_type)
    }

    /// Validate the configuration
    pub fn validate(&self) -> Result<(), String> {
        if self.enabled && self.slack.is_none() && self.webhooks.is_empty() {
            return Err("Alerting enabled but no channels configured".to_string());
        }
        Ok(())
    }
}

/// Slack configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlackConfig {
    /// Whether Slack is enabled
    #[serde(default = "default_true")]
    pub enabled: bool,

    /// Webhook URL
    pub webhook_url: String,

    /// Default channel (optional, uses webhook default)
    #[serde(default)]
    pub channel: Option<String>,

    /// Bot username
    #[serde(default = "default_slack_username")]
    pub username: String,

    /// Bot icon emoji
    #[serde(default = "default_slack_icon")]
    pub icon_emoji: String,

    /// Include metadata in messages
    #[serde(default = "default_true")]
    pub include_metadata: bool,

    /// Timeout in milliseconds
    #[serde(default = "default_timeout")]
    pub timeout_ms: u64,
}

fn default_slack_username() -> String {
    "LiteLLM Gateway".to_string()
}

fn default_slack_icon() -> String {
    ":robot_face:".to_string()
}

fn default_timeout() -> u64 {
    5000
}

impl SlackConfig {
    /// Create a new Slack config
    pub fn new(webhook_url: impl Into<String>) -> Self {
        Self {
            enabled: true,
            webhook_url: webhook_url.into(),
            channel: None,
            username: default_slack_username(),
            icon_emoji: default_slack_icon(),
            include_metadata: true,
            timeout_ms: default_timeout(),
        }
    }

    /// Set channel
    pub fn with_channel(mut self, channel: impl Into<String>) -> Self {
        self.channel = Some(channel.into());
        self
    }

    /// Set username
    pub fn with_username(mut self, username: impl Into<String>) -> Self {
        self.username = username.into();
        self
    }

    /// Set icon emoji
    pub fn with_icon(mut self, emoji: impl Into<String>) -> Self {
        self.icon_emoji = emoji.into();
        self
    }
}

/// Generic webhook configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookConfig {
    /// Webhook name
    pub name: String,

    /// Whether this webhook is enabled
    #[serde(default = "default_true")]
    pub enabled: bool,

    /// Webhook URL
    pub url: String,

    /// HTTP method (POST, PUT)
    #[serde(default = "default_method")]
    pub method: String,

    /// Custom headers
    #[serde(default)]
    pub headers: std::collections::HashMap<String, String>,

    /// Timeout in milliseconds
    #[serde(default = "default_timeout")]
    pub timeout_ms: u64,

    /// Retry count
    #[serde(default = "default_retries")]
    pub retries: u32,

    /// Alert levels to send to this webhook
    #[serde(default)]
    pub levels: HashSet<AlertLevel>,
}

fn default_method() -> String {
    "POST".to_string()
}

fn default_retries() -> u32 {
    3
}

impl WebhookConfig {
    /// Create a new webhook config
    pub fn new(name: impl Into<String>, url: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            enabled: true,
            url: url.into(),
            method: default_method(),
            headers: std::collections::HashMap::new(),
            timeout_ms: default_timeout(),
            retries: default_retries(),
            levels: HashSet::new(),
        }
    }

    /// Add header
    pub fn with_header(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.headers.insert(key.into(), value.into());
        self
    }

    /// Set timeout
    pub fn with_timeout(mut self, ms: u64) -> Self {
        self.timeout_ms = ms;
        self
    }

    /// Set retries
    pub fn with_retries(mut self, count: u32) -> Self {
        self.retries = count;
        self
    }

    /// Filter by alert levels
    pub fn with_levels(mut self, levels: impl IntoIterator<Item = AlertLevel>) -> Self {
        self.levels = levels.into_iter().collect();
        self
    }

    /// Check if this webhook should receive an alert level
    pub fn accepts_level(&self, level: AlertLevel) -> bool {
        self.levels.is_empty() || self.levels.contains(&level)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_alert_config_default() {
        let config = AlertConfig::default();
        assert!(!config.enabled);
        assert!(config.budget_alerts);
        assert!(config.error_rate_alerts);
        assert_eq!(config.budget_warning_threshold, 0.8);
    }

    #[test]
    fn test_alert_config_builder() {
        let config = AlertConfig::new()
            .enable()
            .with_min_level(AlertLevel::Error)
            .with_slack(SlackConfig::new("https://hooks.slack.com/test"))
            .with_budget_warning_threshold(0.9)
            .with_error_rate_threshold(0.1)
            .with_latency_threshold(10000)
            .with_rate_limit(30)
            .with_cooldown(600)
            .suppress_type("budget_warning");

        assert!(config.enabled);
        assert_eq!(config.min_level, AlertLevel::Error);
        assert!(config.slack.is_some());
        assert_eq!(config.budget_warning_threshold, 0.9);
        assert!(config.latency_alerts);
        assert!(config.is_suppressed("budget_warning"));
    }

    #[test]
    fn test_slack_config() {
        let config = SlackConfig::new("https://hooks.slack.com/test")
            .with_channel("#alerts")
            .with_username("TestBot")
            .with_icon(":bell:");

        assert!(config.enabled);
        assert_eq!(config.channel, Some("#alerts".to_string()));
        assert_eq!(config.username, "TestBot");
        assert_eq!(config.icon_emoji, ":bell:");
    }

    #[test]
    fn test_webhook_config() {
        let config = WebhookConfig::new("pagerduty", "https://events.pagerduty.com/v2/enqueue")
            .with_header("Authorization", "Bearer token")
            .with_timeout(10000)
            .with_retries(5)
            .with_levels([AlertLevel::Error, AlertLevel::Critical]);

        assert!(config.enabled);
        assert!(config.headers.contains_key("Authorization"));
        assert_eq!(config.timeout_ms, 10000);
        assert_eq!(config.retries, 5);
        assert!(config.accepts_level(AlertLevel::Error));
        assert!(config.accepts_level(AlertLevel::Critical));
        assert!(!config.accepts_level(AlertLevel::Warning));
    }

    #[test]
    fn test_webhook_accepts_all_levels() {
        let config = WebhookConfig::new("all", "https://example.com");
        // Empty levels means accept all
        assert!(config.accepts_level(AlertLevel::Debug));
        assert!(config.accepts_level(AlertLevel::Critical));
    }

    #[test]
    fn test_config_validation() {
        let valid_config = AlertConfig::new()
            .enable()
            .with_slack(SlackConfig::new("https://hooks.slack.com/test"));
        assert!(valid_config.validate().is_ok());

        let invalid_config = AlertConfig::new().enable();
        assert!(invalid_config.validate().is_err());
    }

    #[test]
    fn test_config_serialization() {
        let config = AlertConfig::new()
            .enable()
            .with_slack(SlackConfig::new("https://hooks.slack.com/test"));

        let json = serde_json::to_string(&config).unwrap();
        let deserialized: AlertConfig = serde_json::from_str(&json).unwrap();

        assert_eq!(config.enabled, deserialized.enabled);
        assert!(deserialized.slack.is_some());
    }
}
