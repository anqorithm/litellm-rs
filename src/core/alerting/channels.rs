//! Alert channels for sending notifications
//!
//! This module provides various channel implementations for sending alerts.

use async_trait::async_trait;
use reqwest::Client;
use serde::Serialize;
use std::time::Duration;
use tracing::{debug, error, warn};

use super::config::{SlackConfig, WebhookConfig};
use super::types::{Alert, AlertError, AlertResult};
use crate::utils::net::http::create_custom_client;

/// Trait for alert channels
#[async_trait]
pub trait AlertChannel: Send + Sync {
    /// Get the name of this channel
    fn name(&self) -> &str;

    /// Check if this channel is enabled
    fn is_enabled(&self) -> bool;

    /// Send an alert through this channel
    async fn send(&self, alert: &Alert) -> AlertResult<()>;
}

/// Boxed alert channel for dynamic dispatch
pub type BoxedAlertChannel = Box<dyn AlertChannel>;

// ============================================================================
// Slack Channel
// ============================================================================

/// Slack webhook channel
pub struct SlackChannel {
    config: SlackConfig,
    client: Client,
}

impl SlackChannel {
    /// Create a new Slack channel
    pub fn new(config: SlackConfig) -> AlertResult<Self> {
        let client = create_custom_client(Duration::from_millis(config.timeout_ms))
            .map_err(|e| AlertError::Config(format!("Failed to create HTTP client: {}", e)))?;

        Ok(Self { config, client })
    }

    /// Build Slack message payload
    fn build_payload(&self, alert: &Alert) -> SlackPayload {
        let mut attachments = vec![SlackAttachment {
            color: alert.level.color().to_string(),
            title: alert.title.clone(),
            text: alert.message.clone(),
            fields: Vec::new(),
            footer: Some(format!("Alert ID: {}", alert.id)),
            ts: Some(alert.timestamp.timestamp()),
        }];

        // Add metadata fields if enabled
        if self.config.include_metadata && !alert.metadata.is_empty() {
            let fields: Vec<SlackField> = alert
                .metadata
                .iter()
                .take(10) // Limit fields
                .map(|(k, v)| SlackField {
                    title: k.clone(),
                    value: v.to_string(),
                    short: true,
                })
                .collect();

            if let Some(attachment) = attachments.first_mut() {
                attachment.fields = fields;
            }
        }

        SlackPayload {
            channel: self.config.channel.clone(),
            username: Some(self.config.username.clone()),
            icon_emoji: Some(self.config.icon_emoji.clone()),
            text: format!("{} *{}*", alert.level.emoji(), alert.alert_type.name()),
            attachments,
        }
    }
}

#[async_trait]
impl AlertChannel for SlackChannel {
    fn name(&self) -> &str {
        "slack"
    }

    fn is_enabled(&self) -> bool {
        self.config.enabled
    }

    async fn send(&self, alert: &Alert) -> AlertResult<()> {
        if !self.is_enabled() {
            return Ok(());
        }

        let payload = self.build_payload(alert);

        debug!("Sending alert to Slack: {}", alert.id);

        let response = self
            .client
            .post(&self.config.webhook_url)
            .json(&payload)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            error!("Slack webhook failed: {} - {}", status, body);
            return Err(AlertError::Channel(format!(
                "Slack webhook failed: {} - {}",
                status, body
            )));
        }

        debug!("Alert sent to Slack successfully: {}", alert.id);
        Ok(())
    }
}

/// Slack message payload
#[derive(Debug, Serialize)]
struct SlackPayload {
    #[serde(skip_serializing_if = "Option::is_none")]
    channel: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    username: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    icon_emoji: Option<String>,
    text: String,
    attachments: Vec<SlackAttachment>,
}

/// Slack attachment
#[derive(Debug, Serialize)]
struct SlackAttachment {
    color: String,
    title: String,
    text: String,
    fields: Vec<SlackField>,
    #[serde(skip_serializing_if = "Option::is_none")]
    footer: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    ts: Option<i64>,
}

/// Slack field
#[derive(Debug, Serialize)]
struct SlackField {
    title: String,
    value: String,
    short: bool,
}

// ============================================================================
// Generic Webhook Channel
// ============================================================================

/// Generic webhook channel
pub struct WebhookChannel {
    config: WebhookConfig,
    client: Client,
}

impl WebhookChannel {
    /// Create a new webhook channel
    pub fn new(config: WebhookConfig) -> AlertResult<Self> {
        let client = create_custom_client(Duration::from_millis(config.timeout_ms))
            .map_err(|e| AlertError::Config(format!("Failed to create HTTP client: {}", e)))?;

        Ok(Self { config, client })
    }

    /// Send with retries
    async fn send_with_retries(&self, alert: &Alert) -> AlertResult<()> {
        let mut last_error = None;

        for attempt in 0..=self.config.retries {
            if attempt > 0 {
                // Exponential backoff
                let delay = Duration::from_millis(100 * 2u64.pow(attempt - 1));
                tokio::time::sleep(delay).await;
                warn!(
                    "Retrying webhook {} (attempt {})",
                    self.config.name, attempt
                );
            }

            match self.do_send(alert).await {
                Ok(()) => return Ok(()),
                Err(e) => {
                    last_error = Some(e);
                }
            }
        }

        Err(last_error.unwrap_or_else(|| AlertError::Channel("Unknown error".to_string())))
    }

    /// Perform the actual send
    async fn do_send(&self, alert: &Alert) -> AlertResult<()> {
        let mut request = match self.config.method.to_uppercase().as_str() {
            "PUT" => self.client.put(&self.config.url),
            _ => self.client.post(&self.config.url),
        };

        // Add custom headers
        for (key, value) in &self.config.headers {
            request = request.header(key, value);
        }

        let response = request
            .header("Content-Type", "application/json")
            .json(alert)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(AlertError::Channel(format!(
                "Webhook {} failed: {} - {}",
                self.config.name, status, body
            )));
        }

        Ok(())
    }
}

#[async_trait]
impl AlertChannel for WebhookChannel {
    fn name(&self) -> &str {
        &self.config.name
    }

    fn is_enabled(&self) -> bool {
        self.config.enabled
    }

    async fn send(&self, alert: &Alert) -> AlertResult<()> {
        if !self.is_enabled() {
            return Ok(());
        }

        // Check if this webhook accepts this alert level
        if !self.config.accepts_level(alert.level) {
            debug!(
                "Webhook {} does not accept level {:?}",
                self.config.name, alert.level
            );
            return Ok(());
        }

        debug!(
            "Sending alert to webhook {}: {}",
            self.config.name, alert.id
        );

        self.send_with_retries(alert).await?;

        debug!(
            "Alert sent to webhook {} successfully: {}",
            self.config.name, alert.id
        );
        Ok(())
    }
}

// ============================================================================
// Null Channel (for testing)
// ============================================================================

/// Null channel that discards all alerts
pub struct NullChannel;

#[async_trait]
impl AlertChannel for NullChannel {
    fn name(&self) -> &str {
        "null"
    }

    fn is_enabled(&self) -> bool {
        true
    }

    async fn send(&self, _alert: &Alert) -> AlertResult<()> {
        Ok(())
    }
}

// ============================================================================
// Memory Channel (for testing)
// ============================================================================

/// In-memory channel for testing
pub struct MemoryChannel {
    alerts: std::sync::Arc<tokio::sync::Mutex<Vec<Alert>>>,
}

impl MemoryChannel {
    /// Create a new memory channel
    pub fn new() -> Self {
        Self {
            alerts: std::sync::Arc::new(tokio::sync::Mutex::new(Vec::new())),
        }
    }

    /// Get all stored alerts
    pub async fn alerts(&self) -> Vec<Alert> {
        let alerts = self.alerts.lock().await;
        alerts.clone()
    }

    /// Get alert count
    pub async fn count(&self) -> usize {
        let alerts = self.alerts.lock().await;
        alerts.len()
    }

    /// Clear all alerts
    pub async fn clear(&self) {
        let mut alerts = self.alerts.lock().await;
        alerts.clear();
    }
}

impl Default for MemoryChannel {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl AlertChannel for MemoryChannel {
    fn name(&self) -> &str {
        "memory"
    }

    fn is_enabled(&self) -> bool {
        true
    }

    async fn send(&self, alert: &Alert) -> AlertResult<()> {
        let mut alerts = self.alerts.lock().await;
        alerts.push(alert.clone());
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::alerting::types::AlertLevel;

    #[test]
    fn test_slack_channel_creation() {
        let config = SlackConfig::new("https://hooks.slack.com/test");
        let channel = SlackChannel::new(config).unwrap();
        assert_eq!(channel.name(), "slack");
        assert!(channel.is_enabled());
    }

    #[test]
    fn test_slack_payload_building() {
        let config = SlackConfig::new("https://hooks.slack.com/test");
        let channel = SlackChannel::new(config).unwrap();

        let alert = Alert::new(AlertLevel::Warning, "Test Alert", "Test message");
        let payload = channel.build_payload(&alert);

        assert!(payload.text.contains(":warning:"));
        assert_eq!(payload.attachments.len(), 1);
        assert_eq!(payload.attachments[0].color, "#ffcc00");
    }

    #[test]
    fn test_webhook_channel_creation() {
        let config = WebhookConfig::new("test", "https://example.com/webhook");
        let channel = WebhookChannel::new(config).unwrap();
        assert_eq!(channel.name(), "test");
        assert!(channel.is_enabled());
    }

    #[tokio::test]
    async fn test_memory_channel() {
        let channel = MemoryChannel::new();

        let alert = Alert::new(AlertLevel::Info, "Test", "Message");
        channel.send(&alert).await.unwrap();

        assert_eq!(channel.count().await, 1);

        let alerts = channel.alerts().await;
        assert_eq!(alerts[0].title, "Test");

        channel.clear().await;
        assert_eq!(channel.count().await, 0);
    }

    #[tokio::test]
    async fn test_null_channel() {
        let channel = NullChannel;
        let alert = Alert::new(AlertLevel::Info, "Test", "Message");
        channel.send(&alert).await.unwrap();
    }
}
