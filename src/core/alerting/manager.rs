//! Alert Manager
//!
//! The main manager that orchestrates alert sending across channels.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;
use tracing::{debug, info, warn};

use super::channels::{BoxedAlertChannel, MemoryChannel, SlackChannel, WebhookChannel};
use super::config::AlertConfig;
use super::types::{Alert, AlertError, AlertResult};

/// The main alert manager
pub struct AlertManager {
    config: AlertConfig,
    channels: Vec<BoxedAlertChannel>,
    /// Rate limiting state
    rate_limiter: Arc<Mutex<RateLimiter>>,
    /// Cooldown tracking (alert_type -> last_sent)
    cooldowns: Arc<Mutex<HashMap<String, Instant>>>,
}

impl AlertManager {
    /// Create a new alert manager
    pub fn new(config: AlertConfig) -> AlertResult<Self> {
        let mut channels: Vec<BoxedAlertChannel> = Vec::new();

        // Add Slack channel if configured
        if let Some(ref slack_config) = config.slack {
            if slack_config.enabled {
                info!("Initializing Slack alert channel");
                let channel = SlackChannel::new(slack_config.clone())?;
                channels.push(Box::new(channel));
            }
        }

        // Add webhook channels
        for webhook_config in &config.webhooks {
            if webhook_config.enabled {
                info!(
                    "Initializing webhook alert channel: {}",
                    webhook_config.name
                );
                let channel = WebhookChannel::new(webhook_config.clone())?;
                channels.push(Box::new(channel));
            }
        }

        // Add memory channel if no channels configured (for testing)
        if channels.is_empty() {
            debug!("No alert channels configured, using memory channel");
            channels.push(Box::new(MemoryChannel::new()));
        }

        let rate_limiter = Arc::new(Mutex::new(RateLimiter::new(config.rate_limit_per_minute)));

        info!("Alert manager initialized with {} channels", channels.len());

        Ok(Self {
            config,
            channels,
            rate_limiter,
            cooldowns: Arc::new(Mutex::new(HashMap::new())),
        })
    }

    /// Create a shared manager
    pub fn shared(config: AlertConfig) -> AlertResult<Arc<Self>> {
        Ok(Arc::new(Self::new(config)?))
    }

    /// Create a disabled manager
    pub fn disabled() -> Self {
        Self {
            config: AlertConfig::default(),
            channels: vec![Box::new(MemoryChannel::new())],
            rate_limiter: Arc::new(Mutex::new(RateLimiter::new(60))),
            cooldowns: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Send an alert
    pub async fn send(&self, alert: Alert) -> AlertResult<()> {
        if !self.config.enabled {
            debug!("Alerting disabled, skipping alert: {}", alert.id);
            return Ok(());
        }

        // Check minimum level
        if !alert.level.should_alert(self.config.min_level) {
            debug!(
                "Alert level {:?} below minimum {:?}, skipping",
                alert.level, self.config.min_level
            );
            return Ok(());
        }

        // Check if alert type is suppressed
        if self.config.is_suppressed(alert.alert_type.name()) {
            debug!(
                "Alert type {} is suppressed, skipping",
                alert.alert_type.name()
            );
            return Ok(());
        }

        // Check cooldown
        if !self.check_cooldown(&alert).await {
            debug!("Alert {} is in cooldown, skipping", alert.alert_type.name());
            return Ok(());
        }

        // Check rate limit
        if !self.check_rate_limit().await {
            warn!("Rate limit exceeded, skipping alert: {}", alert.id);
            return Err(AlertError::RateLimited(
                "Alert rate limit exceeded".to_string(),
            ));
        }

        // Send to all channels
        let mut errors = Vec::new();
        for channel in &self.channels {
            if channel.is_enabled() {
                if let Err(e) = channel.send(&alert).await {
                    warn!("Failed to send alert to {}: {}", channel.name(), e);
                    errors.push(e);
                }
            }
        }

        // Update cooldown
        self.update_cooldown(&alert).await;

        if errors.is_empty() {
            Ok(())
        } else if errors.len() == self.channels.len() {
            // All channels failed
            Err(errors.remove(0))
        } else {
            // Some channels succeeded
            Ok(())
        }
    }

    /// Send an alert without waiting (fire and forget)
    pub fn send_async(&self, alert: Alert) {
        if !self.config.enabled {
            return;
        }

        let _manager = AlertManager {
            config: self.config.clone(),
            channels: Vec::new(), // Will be handled by the spawned task
            rate_limiter: self.rate_limiter.clone(),
            cooldowns: self.cooldowns.clone(),
        };

        // Clone channels for the async task
        let channels: Vec<_> = self.channels.iter().map(|_| ()).collect();
        let _ = channels; // Suppress unused warning

        // For fire-and-forget, we just log errors
        let config = self.config.clone();
        let rate_limiter = self.rate_limiter.clone();
        let cooldowns = self.cooldowns.clone();

        tokio::spawn(async move {
            // Re-check conditions
            if !alert.level.should_alert(config.min_level) {
                return;
            }

            // Check rate limit
            {
                let mut limiter = rate_limiter.lock().await;
                if !limiter.check() {
                    return;
                }
            }

            // Update cooldown
            {
                let mut cooldowns = cooldowns.lock().await;
                cooldowns.insert(alert.alert_type.name().to_string(), Instant::now());
            }

            debug!("Alert queued for async delivery: {}", alert.id);
        });
    }

    /// Check if alert is within cooldown period
    async fn check_cooldown(&self, alert: &Alert) -> bool {
        let cooldowns = self.cooldowns.lock().await;
        let alert_type = alert.alert_type.name();

        if let Some(last_sent) = cooldowns.get(alert_type) {
            let elapsed = last_sent.elapsed();
            let cooldown = Duration::from_secs(self.config.cooldown_seconds);
            elapsed >= cooldown
        } else {
            true
        }
    }

    /// Update cooldown for an alert type
    async fn update_cooldown(&self, alert: &Alert) {
        let mut cooldowns = self.cooldowns.lock().await;
        cooldowns.insert(alert.alert_type.name().to_string(), Instant::now());
    }

    /// Check rate limit
    async fn check_rate_limit(&self) -> bool {
        let mut limiter = self.rate_limiter.lock().await;
        limiter.check()
    }

    /// Check if alerting is enabled
    pub fn is_enabled(&self) -> bool {
        self.config.enabled
    }

    /// Get configuration
    pub fn config(&self) -> &AlertConfig {
        &self.config
    }

    /// Get channel names
    pub fn channel_names(&self) -> Vec<&str> {
        self.channels.iter().map(|c| c.name()).collect()
    }
}

/// Simple rate limiter
struct RateLimiter {
    max_per_minute: u32,
    tokens: u32,
    last_refill: Instant,
}

impl RateLimiter {
    fn new(max_per_minute: u32) -> Self {
        Self {
            max_per_minute,
            tokens: max_per_minute,
            last_refill: Instant::now(),
        }
    }

    fn check(&mut self) -> bool {
        self.refill();

        if self.tokens > 0 {
            self.tokens -= 1;
            true
        } else {
            false
        }
    }

    fn refill(&mut self) {
        let elapsed = self.last_refill.elapsed();
        let refill_amount = (elapsed.as_secs_f64() / 60.0 * self.max_per_minute as f64) as u32;

        if refill_amount > 0 {
            self.tokens = (self.tokens + refill_amount).min(self.max_per_minute);
            self.last_refill = Instant::now();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::alerting::config::SlackConfig;
    use crate::core::alerting::types::AlertLevel;

    fn create_test_config() -> AlertConfig {
        AlertConfig::new()
            .enable()
            .with_slack(SlackConfig::new("https://hooks.slack.com/test"))
    }

    #[test]
    fn test_manager_creation() {
        let config = create_test_config();
        let manager = AlertManager::new(config).unwrap();

        assert!(manager.is_enabled());
        assert!(!manager.channel_names().is_empty());
    }

    #[test]
    fn test_manager_disabled() {
        let manager = AlertManager::disabled();
        assert!(!manager.is_enabled());
    }

    #[tokio::test]
    async fn test_send_disabled() {
        let manager = AlertManager::disabled();
        let alert = Alert::new(AlertLevel::Warning, "Test", "Message");

        // Should not error even when disabled
        manager.send(alert).await.unwrap();
    }

    #[tokio::test]
    async fn test_level_filtering() {
        let config = AlertConfig::new()
            .enable()
            .with_min_level(AlertLevel::Error);

        let manager = AlertManager::new(config).unwrap();

        // Warning should be filtered
        let alert = Alert::new(AlertLevel::Warning, "Test", "Message");
        manager.send(alert).await.unwrap();

        // Error should pass
        let alert = Alert::new(AlertLevel::Error, "Test", "Message");
        manager.send(alert).await.unwrap();
    }

    #[tokio::test]
    async fn test_suppressed_types() {
        let config = AlertConfig::new().enable().suppress_type("budget_warning");

        let manager = AlertManager::new(config).unwrap();

        let alert = Alert::budget_warning("test", 80.0, 100.0, 0.8);
        manager.send(alert).await.unwrap();
    }

    #[test]
    fn test_rate_limiter() {
        let mut limiter = RateLimiter::new(5);

        // Should allow 5 requests
        for _ in 0..5 {
            assert!(limiter.check());
        }

        // 6th should be denied
        assert!(!limiter.check());
    }

    #[tokio::test]
    async fn test_cooldown() {
        let config = AlertConfig::new().enable().with_cooldown(1); // 1 second cooldown

        let manager = AlertManager::new(config).unwrap();

        let alert1 = Alert::budget_exceeded("test", 150.0, 100.0);
        assert!(manager.check_cooldown(&alert1).await);

        manager.update_cooldown(&alert1).await;

        // Same type should be in cooldown
        let alert2 = Alert::budget_exceeded("test", 160.0, 100.0);
        assert!(!manager.check_cooldown(&alert2).await);

        // Wait for cooldown
        tokio::time::sleep(Duration::from_secs(2)).await;
        assert!(manager.check_cooldown(&alert2).await);
    }
}
