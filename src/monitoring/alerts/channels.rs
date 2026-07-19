//! Notification channel implementations

use crate::core::net::ProviderEndpointPolicy;
use crate::monitoring::types::{Alert, AlertSeverity};
use crate::utils::error::gateway_error::{GatewayError, Result};
use crate::utils::net::http::ProviderHttpClient;
use std::time::Duration;
use tracing::warn;

pub(super) const SLACK_WEBHOOK_TIMEOUT: Duration = Duration::from_secs(120);
const INVALID_SLACK_WEBHOOK_URL: &str =
    "Slack webhook URL is invalid or disallowed by outbound policy";

/// Notification channel trait
#[async_trait::async_trait]
pub trait NotificationChannel: Send + Sync + std::fmt::Debug {
    /// Send a notification
    async fn send(&self, alert: &Alert) -> Result<()>;

    /// Get channel name
    fn name(&self) -> &str;

    /// Check if channel supports severity level
    fn supports_severity(&self, severity: AlertSeverity) -> bool;
}

/// Slack notification channel
pub struct SlackChannel {
    webhook_url: String,
    client: ProviderHttpClient,
    channel: Option<String>,
    username: Option<String>,
    min_severity: AlertSeverity,
}

impl std::fmt::Debug for SlackChannel {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("SlackChannel { webhook_url: [REDACTED], .. }")
    }
}

/// Email notification channel
#[derive(Debug)]
pub struct EmailChannel {
    _smtp_config: SmtpConfig,
    _recipients: Vec<String>,
    min_severity: AlertSeverity,
}

/// SMTP configuration
#[derive(Debug, Clone)]
pub struct SmtpConfig {
    pub server: String,
    pub port: u16,
    pub username: String,
    pub password: String,
    pub from_address: String,
}

impl SlackChannel {
    /// Create a new Slack notification channel
    pub fn new(
        webhook_url: String,
        channel: Option<String>,
        username: Option<String>,
        min_severity: AlertSeverity,
    ) -> Result<Self> {
        let client = ProviderHttpClient::no_redirect(
            ProviderEndpointPolicy::public_only(),
            SLACK_WEBHOOK_TIMEOUT,
        )
        .map_err(|_| {
            GatewayError::Network("Failed to create policy-bound Slack webhook client".to_string())
        })?;
        Self::with_client(webhook_url, channel, username, min_severity, client)
    }

    fn with_client(
        webhook_url: String,
        channel: Option<String>,
        username: Option<String>,
        min_severity: AlertSeverity,
        client: ProviderHttpClient,
    ) -> Result<Self> {
        let url = reqwest::Url::parse(&webhook_url)
            .map_err(|_| GatewayError::Validation(INVALID_SLACK_WEBHOOK_URL.to_string()))?;
        if !matches!(url.scheme(), "http" | "https")
            || ProviderEndpointPolicy::public_only()
                .validate_url_without_resolution(&url)
                .is_err()
        {
            return Err(GatewayError::Validation(
                INVALID_SLACK_WEBHOOK_URL.to_string(),
            ));
        }

        Ok(Self {
            webhook_url: url.to_string(),
            client,
            channel,
            username,
            min_severity,
        })
    }

    #[cfg(test)]
    pub(super) fn with_client_for_test(
        webhook_url: String,
        channel: Option<String>,
        username: Option<String>,
        min_severity: AlertSeverity,
        client: ProviderHttpClient,
    ) -> Result<Self> {
        Self::with_client(webhook_url, channel, username, min_severity, client)
    }
}

#[async_trait::async_trait]
impl NotificationChannel for SlackChannel {
    async fn send(&self, alert: &Alert) -> Result<()> {
        let color = match alert.severity {
            AlertSeverity::Info => "#36a64f",      // Green
            AlertSeverity::Warning => "#ff9500",   // Orange
            AlertSeverity::Critical => "#ff0000",  // Red
            AlertSeverity::Emergency => "#8b0000", // Dark Red
        };

        let payload = serde_json::json!({
            "username": self.username.as_deref().unwrap_or("Gateway Alert"),
            "channel": self.channel,
            "attachments": [{
                "color": color,
                "title": alert.title,
                "text": alert.description,
                "fields": [
                    {
                        "title": "Severity",
                        "value": format!("{:?}", alert.severity),
                        "short": true
                    },
                    {
                        "title": "Source",
                        "value": alert.source,
                        "short": true
                    },
                    {
                        "title": "Time",
                        "value": alert.timestamp.format("%Y-%m-%d %H:%M:%S UTC").to_string(),
                        "short": true
                    }
                ],
                "footer": "Gateway Monitoring",
                "ts": alert.timestamp.timestamp()
            }]
        });

        let request = self.client.post(&self.webhook_url).map_err(|_| {
            GatewayError::Network("Slack webhook rejected by outbound endpoint policy".to_string())
        })?;
        let response = request.json(&payload).send().await.map_err(|error| {
            let message = if ProviderHttpClient::request_error_is_endpoint_policy(&error) {
                "Slack webhook rejected by outbound endpoint policy"
            } else {
                "Failed to send Slack notification"
            };
            GatewayError::Network(message.to_string())
        })?;

        if !response.status().is_success() {
            return Err(GatewayError::Internal(format!(
                "Slack webhook returned status: {}",
                response.status()
            )));
        }

        Ok(())
    }

    fn name(&self) -> &str {
        "slack"
    }

    fn supports_severity(&self, severity: AlertSeverity) -> bool {
        severity as u8 >= self.min_severity as u8
    }
}

impl EmailChannel {
    /// Create a new email notification channel
    pub fn new(
        smtp_config: SmtpConfig,
        recipients: Vec<String>,
        min_severity: AlertSeverity,
    ) -> Self {
        Self {
            _smtp_config: smtp_config,
            _recipients: recipients,
            min_severity,
        }
    }
}

#[async_trait::async_trait]
impl NotificationChannel for EmailChannel {
    async fn send(&self, _alert: &Alert) -> Result<()> {
        // NOTE: email sending not yet implemented
        // This would use an SMTP library to send emails
        warn!("Email notifications not implemented yet");
        Ok(())
    }

    fn name(&self) -> &str {
        "email"
    }

    fn supports_severity(&self, severity: AlertSeverity) -> bool {
        severity as u8 >= self.min_severity as u8
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use uuid::Uuid;

    fn create_test_alert(severity: AlertSeverity) -> Alert {
        Alert {
            id: Uuid::new_v4().to_string(),
            title: "Test Alert".to_string(),
            description: "This is a test alert".to_string(),
            severity,
            source: "test".to_string(),
            timestamp: Utc::now(),
            metadata: serde_json::json!({}),
            resolved: false,
        }
    }

    // ==================== SmtpConfig Tests ====================

    #[test]
    fn test_smtp_config_creation() {
        let config = SmtpConfig {
            server: "smtp.example.com".to_string(),
            port: 587,
            username: "user@example.com".to_string(),
            password: "secret123".to_string(),
            from_address: "alerts@example.com".to_string(),
        };

        assert_eq!(config.server, "smtp.example.com");
        assert_eq!(config.port, 587);
        assert_eq!(config.username, "user@example.com");
        assert_eq!(config.password, "secret123");
        assert_eq!(config.from_address, "alerts@example.com");
    }

    #[test]
    fn test_smtp_config_clone() {
        let original = SmtpConfig {
            server: "smtp.test.com".to_string(),
            port: 465,
            username: "test".to_string(),
            password: "pass".to_string(),
            from_address: "from@test.com".to_string(),
        };

        let cloned = original.clone();
        assert_eq!(original.server, cloned.server);
        assert_eq!(original.port, cloned.port);
    }

    #[test]
    fn test_smtp_config_common_ports() {
        // Test common SMTP ports
        let ports = [25, 465, 587, 2525];
        for port in ports {
            let config = SmtpConfig {
                server: "smtp.test.com".to_string(),
                port,
                username: "user".to_string(),
                password: "pass".to_string(),
                from_address: "from@test.com".to_string(),
            };
            assert_eq!(config.port, port);
        }
    }

    // ==================== SlackChannel Tests ====================

    #[test]
    fn test_slack_channel_creation() {
        let channel = SlackChannel::new(
            "https://hooks.slack.com/services/xxx".to_string(),
            Some("#alerts".to_string()),
            Some("AlertBot".to_string()),
            AlertSeverity::Warning,
        )
        .unwrap();

        assert_eq!(channel.webhook_url, "https://hooks.slack.com/services/xxx");
        assert_eq!(channel.channel, Some("#alerts".to_string()));
        assert_eq!(channel.username, Some("AlertBot".to_string()));
    }

    #[test]
    fn test_slack_channel_minimal() {
        let channel = SlackChannel::new(
            "https://hooks.slack.com/test".to_string(),
            None,
            None,
            AlertSeverity::Info,
        )
        .unwrap();

        assert!(channel.channel.is_none());
        assert!(channel.username.is_none());
    }

    #[test]
    fn test_slack_channel_name() {
        let channel = SlackChannel::new(
            "https://test.com".to_string(),
            None,
            None,
            AlertSeverity::Info,
        )
        .unwrap();

        assert_eq!(channel.name(), "slack");
    }

    #[test]
    fn test_slack_channel_supports_severity_info() {
        let channel = SlackChannel::new(
            "https://test.com".to_string(),
            None,
            None,
            AlertSeverity::Info,
        )
        .unwrap();

        assert!(channel.supports_severity(AlertSeverity::Info));
        assert!(channel.supports_severity(AlertSeverity::Warning));
        assert!(channel.supports_severity(AlertSeverity::Critical));
        assert!(channel.supports_severity(AlertSeverity::Emergency));
    }

    #[test]
    fn test_slack_channel_supports_severity_warning() {
        let channel = SlackChannel::new(
            "https://test.com".to_string(),
            None,
            None,
            AlertSeverity::Warning,
        )
        .unwrap();

        assert!(!channel.supports_severity(AlertSeverity::Info));
        assert!(channel.supports_severity(AlertSeverity::Warning));
        assert!(channel.supports_severity(AlertSeverity::Critical));
        assert!(channel.supports_severity(AlertSeverity::Emergency));
    }

    #[test]
    fn test_slack_channel_supports_severity_critical() {
        let channel = SlackChannel::new(
            "https://test.com".to_string(),
            None,
            None,
            AlertSeverity::Critical,
        )
        .unwrap();

        assert!(!channel.supports_severity(AlertSeverity::Info));
        assert!(!channel.supports_severity(AlertSeverity::Warning));
        assert!(channel.supports_severity(AlertSeverity::Critical));
        assert!(channel.supports_severity(AlertSeverity::Emergency));
    }

    #[test]
    fn test_slack_channel_supports_severity_emergency() {
        let channel = SlackChannel::new(
            "https://test.com".to_string(),
            None,
            None,
            AlertSeverity::Emergency,
        )
        .unwrap();

        assert!(!channel.supports_severity(AlertSeverity::Info));
        assert!(!channel.supports_severity(AlertSeverity::Warning));
        assert!(!channel.supports_severity(AlertSeverity::Critical));
        assert!(channel.supports_severity(AlertSeverity::Emergency));
    }

    // ==================== EmailChannel Tests ====================

    #[test]
    fn test_email_channel_creation() {
        let smtp_config = SmtpConfig {
            server: "smtp.example.com".to_string(),
            port: 587,
            username: "user".to_string(),
            password: "pass".to_string(),
            from_address: "alerts@example.com".to_string(),
        };

        let channel = EmailChannel::new(
            smtp_config,
            vec![
                "admin@example.com".to_string(),
                "ops@example.com".to_string(),
            ],
            AlertSeverity::Critical,
        );

        assert_eq!(channel._recipients.len(), 2);
    }

    #[test]
    fn test_email_channel_name() {
        let smtp_config = SmtpConfig {
            server: "smtp.test.com".to_string(),
            port: 587,
            username: "user".to_string(),
            password: "pass".to_string(),
            from_address: "from@test.com".to_string(),
        };

        let channel = EmailChannel::new(smtp_config, vec![], AlertSeverity::Info);

        assert_eq!(channel.name(), "email");
    }

    #[test]
    fn test_email_channel_supports_severity() {
        let smtp_config = SmtpConfig {
            server: "smtp.test.com".to_string(),
            port: 587,
            username: "user".to_string(),
            password: "pass".to_string(),
            from_address: "from@test.com".to_string(),
        };

        let channel = EmailChannel::new(smtp_config, vec![], AlertSeverity::Warning);

        assert!(!channel.supports_severity(AlertSeverity::Info));
        assert!(channel.supports_severity(AlertSeverity::Warning));
        assert!(channel.supports_severity(AlertSeverity::Critical));
    }

    #[test]
    fn test_email_channel_empty_recipients() {
        let smtp_config = SmtpConfig {
            server: "smtp.test.com".to_string(),
            port: 587,
            username: "user".to_string(),
            password: "pass".to_string(),
            from_address: "from@test.com".to_string(),
        };

        let channel = EmailChannel::new(smtp_config, vec![], AlertSeverity::Info);

        assert!(channel._recipients.is_empty());
    }

    #[test]
    fn test_email_channel_many_recipients() {
        let smtp_config = SmtpConfig {
            server: "smtp.test.com".to_string(),
            port: 587,
            username: "user".to_string(),
            password: "pass".to_string(),
            from_address: "from@test.com".to_string(),
        };

        let recipients: Vec<String> = (0..10).map(|i| format!("user{}@test.com", i)).collect();
        let channel = EmailChannel::new(smtp_config, recipients, AlertSeverity::Info);

        assert_eq!(channel._recipients.len(), 10);
    }

    #[tokio::test]
    async fn test_email_channel_send() {
        let smtp_config = SmtpConfig {
            server: "smtp.test.com".to_string(),
            port: 587,
            username: "user".to_string(),
            password: "pass".to_string(),
            from_address: "from@test.com".to_string(),
        };

        let channel = EmailChannel::new(
            smtp_config,
            vec!["test@example.com".to_string()],
            AlertSeverity::Info,
        );

        let alert = create_test_alert(AlertSeverity::Warning);

        // Email send is not implemented, but should not panic
        let result = channel.send(&alert).await;
        assert!(result.is_ok());
    }

    // ==================== Alert Creation Tests ====================

    #[test]
    fn test_alert_creation_for_channels() {
        let alert = create_test_alert(AlertSeverity::Critical);

        assert_eq!(alert.title, "Test Alert");
        assert_eq!(alert.source, "test");
        assert!(matches!(alert.severity, AlertSeverity::Critical));
    }

    #[test]
    fn test_alert_with_different_severities() {
        let severities = [
            AlertSeverity::Info,
            AlertSeverity::Warning,
            AlertSeverity::Critical,
            AlertSeverity::Emergency,
        ];

        for severity in severities {
            let alert = create_test_alert(severity);
            assert!(
                matches!(alert.severity, s if std::mem::discriminant(&s) == std::mem::discriminant(&severity))
            );
        }
    }

    // ==================== Channel Debug Tests ====================

    #[test]
    fn test_slack_channel_debug() {
        let channel = SlackChannel::new(
            "https://test.com".to_string(),
            Some("#test".to_string()),
            None,
            AlertSeverity::Info,
        )
        .unwrap();

        let debug_str = format!("{:?}", channel);
        assert!(debug_str.contains("SlackChannel"));
    }

    #[test]
    fn test_email_channel_debug() {
        let smtp_config = SmtpConfig {
            server: "smtp.test.com".to_string(),
            port: 587,
            username: "user".to_string(),
            password: "pass".to_string(),
            from_address: "from@test.com".to_string(),
        };

        let channel = EmailChannel::new(smtp_config, vec![], AlertSeverity::Info);

        let debug_str = format!("{:?}", channel);
        assert!(debug_str.contains("EmailChannel"));
    }

    #[test]
    fn test_smtp_config_debug() {
        let config = SmtpConfig {
            server: "smtp.test.com".to_string(),
            port: 587,
            username: "user".to_string(),
            password: "pass".to_string(),
            from_address: "from@test.com".to_string(),
        };

        let debug_str = format!("{:?}", config);
        assert!(debug_str.contains("SmtpConfig"));
        assert!(debug_str.contains("smtp.test.com"));
    }
}
