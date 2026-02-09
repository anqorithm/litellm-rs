//! Webhook type definitions
//!
//! This module contains all webhook-related types, enums, and data structures.

use crate::core::models::RequestContext;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Webhook event types
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum WebhookEventType {
    /// Request started
    RequestStarted,
    /// Request completed successfully
    RequestCompleted,
    /// Request failed
    RequestFailed,
    /// Rate limit exceeded
    RateLimitExceeded,
    /// Cost threshold exceeded
    CostThresholdExceeded,
    /// Provider health changed
    ProviderHealthChanged,
    /// Cache hit/miss
    CacheEvent,
    /// Batch completed
    BatchCompleted,
    /// Batch failed
    BatchFailed,
    /// User created
    UserCreated,
    /// User updated
    UserUpdated,
    /// API key created
    ApiKeyCreated,
    /// API key revoked
    ApiKeyRevoked,
    /// Budget threshold reached
    BudgetThresholdReached,
    /// Security alert
    SecurityAlert,
    /// Custom event
    Custom(String),
}

/// Webhook payload
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookPayload {
    /// Event type
    pub event_type: WebhookEventType,
    /// Event timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// Request context
    pub request_context: Option<RequestContext>,
    /// Event data
    pub data: serde_json::Value,
    /// Event metadata
    pub metadata: HashMap<String, String>,
}

/// Webhook configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookConfig {
    /// Webhook URL
    pub url: String,
    /// Events to subscribe to
    pub events: Vec<WebhookEventType>,
    /// HTTP headers to include
    pub headers: HashMap<String, String>,
    /// Webhook secret for signature verification
    pub secret: Option<String>,
    /// Timeout for webhook requests
    pub timeout_seconds: u64,
    /// Maximum retry attempts
    pub max_retries: u32,
    /// Retry delay in seconds
    pub retry_delay_seconds: u64,
    /// Whether webhook is enabled
    pub enabled: bool,
}

impl Default for WebhookConfig {
    fn default() -> Self {
        Self {
            url: String::new(),
            events: vec![],
            headers: HashMap::new(),
            secret: None,
            timeout_seconds: 30,
            max_retries: 3,
            retry_delay_seconds: 5,
            enabled: true,
        }
    }
}

/// Webhook delivery status
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum WebhookDeliveryStatus {
    /// Pending delivery
    Pending,
    /// Successfully delivered
    Delivered,
    /// Failed to deliver
    Failed,
    /// Retrying delivery
    Retrying,
}

/// Webhook delivery record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookDelivery {
    /// Delivery ID
    pub id: String,
    /// Webhook configuration ID
    pub webhook_id: String,
    /// Event payload
    pub payload: WebhookPayload,
    /// Delivery status
    pub status: WebhookDeliveryStatus,
    /// HTTP response status code
    pub response_status: Option<u16>,
    /// Response body
    pub response_body: Option<String>,
    /// Number of attempts
    pub attempts: u32,
    /// Created timestamp
    pub created_at: chrono::DateTime<chrono::Utc>,
    /// Last attempt timestamp
    pub last_attempt_at: Option<chrono::DateTime<chrono::Utc>>,
    /// Next retry timestamp
    pub next_retry_at: Option<chrono::DateTime<chrono::Utc>>,
}

/// Webhook statistics
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct WebhookStats {
    /// Total events sent
    pub total_events: u64,
    /// Successful deliveries
    pub successful_deliveries: u64,
    /// Failed deliveries
    pub failed_deliveries: u64,
    /// Average delivery time in milliseconds
    pub avg_delivery_time_ms: f64,
    /// Events by type
    pub events_by_type: HashMap<String, u64>,
}

/// Consolidated webhook data - single lock for all webhook-related state
#[derive(Debug, Default)]
pub(super) struct WebhookData {
    /// Registered webhooks
    pub webhooks: HashMap<String, WebhookConfig>,
    /// Delivery queue
    pub delivery_queue: Vec<WebhookDelivery>,
    /// Webhook statistics
    pub stats: WebhookStats,
}


#[cfg(test)]
#[path = "types_tests.rs"]
mod tests;
