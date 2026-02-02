//! Alerting System
//!
//! A decoupled alerting system for sending notifications via various channels.
//!
//! # Features
//!
//! - **Slack Integration**: Send alerts to Slack channels via webhooks
//! - **Generic Webhooks**: Support for custom webhook endpoints
//! - **Budget Alerts**: Automatic alerts when budgets are exceeded
//! - **Error Rate Alerts**: Alerts when error rates exceed thresholds
//! - **Configurable Thresholds**: Fine-grained control over alert triggers
//!
//! # Architecture
//!
//! The alerting system follows these principles:
//! - **Decoupled**: Each alert channel is independent
//! - **Extensible**: Easy to add new channels via the `AlertChannel` trait
//! - **Rate Limited**: Prevents alert flooding
//! - **Async**: Non-blocking alert delivery
//!
//! # Quick Start
//!
//! ```rust,ignore
//! use litellm_rs::core::alerting::{AlertManager, AlertConfig, Alert, AlertLevel};
//!
//! let config = AlertConfig::default()
//!     .with_slack_webhook("https://hooks.slack.com/services/...")
//!     .enable_budget_alerts(true);
//!
//! let manager = AlertManager::new(config).await?;
//!
//! // Send an alert
//! manager.send(Alert::new(AlertLevel::Warning, "Budget threshold reached")).await;
//! ```

pub mod channels;
pub mod config;
pub mod manager;
pub mod types;

#[cfg(test)]
mod tests;

// Re-export main types
pub use channels::{AlertChannel, SlackChannel, WebhookChannel};
pub use config::{AlertConfig, SlackConfig, WebhookConfig};
pub use manager::AlertManager;
pub use types::{Alert, AlertError, AlertLevel, AlertResult, AlertType};
