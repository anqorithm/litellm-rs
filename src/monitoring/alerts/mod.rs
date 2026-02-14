//! Alert management system
//!
//! This module provides comprehensive alerting functionality for monitoring events.

mod channels;
mod manager;
mod processing;
#[cfg(test)]
mod tests;
mod types;

// Re-export public types
#[allow(unused_imports)]
pub use channels::{EmailChannel, NotificationChannel, SlackChannel, SmtpConfig};
pub use manager::AlertManager;
#[allow(unused_imports)]
pub use types::{AlertRule, AlertStats, ComparisonOperator};
