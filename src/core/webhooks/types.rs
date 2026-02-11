//! Webhook type definitions
//!
//! This module contains all webhook-related types, enums, and data structures.

use crate::core::types::context::RequestContext;
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
mod tests {
    use super::*;
    use chrono::Utc;

    // ==================== WebhookEventType Tests ====================

    #[test]
    fn test_webhook_event_type_request_lifecycle() {
        let started = WebhookEventType::RequestStarted;
        let completed = WebhookEventType::RequestCompleted;
        let failed = WebhookEventType::RequestFailed;

        // Test inequality
        assert_ne!(started, completed);
        assert_ne!(completed, failed);

        // Test equality
        assert_eq!(started, WebhookEventType::RequestStarted);
    }

    #[test]
    fn test_webhook_event_type_rate_and_cost() {
        let rate_limit = WebhookEventType::RateLimitExceeded;
        let cost = WebhookEventType::CostThresholdExceeded;

        assert_ne!(rate_limit, cost);
    }

    #[test]
    fn test_webhook_event_type_custom() {
        let custom1 = WebhookEventType::Custom("my-event".to_string());
        let custom2 = WebhookEventType::Custom("my-event".to_string());
        let custom3 = WebhookEventType::Custom("other-event".to_string());

        assert_eq!(custom1, custom2);
        assert_ne!(custom1, custom3);
    }

    #[test]
    fn test_webhook_event_type_all_variants() {
        let variants = vec![
            WebhookEventType::RequestStarted,
            WebhookEventType::RequestCompleted,
            WebhookEventType::RequestFailed,
            WebhookEventType::RateLimitExceeded,
            WebhookEventType::CostThresholdExceeded,
            WebhookEventType::ProviderHealthChanged,
            WebhookEventType::CacheEvent,
            WebhookEventType::BatchCompleted,
            WebhookEventType::BatchFailed,
            WebhookEventType::UserCreated,
            WebhookEventType::UserUpdated,
            WebhookEventType::ApiKeyCreated,
            WebhookEventType::ApiKeyRevoked,
            WebhookEventType::BudgetThresholdReached,
            WebhookEventType::SecurityAlert,
            WebhookEventType::Custom("test".to_string()),
        ];

        // All variants can be cloned
        for variant in &variants {
            let _ = variant.clone();
        }
    }

    #[test]
    fn test_webhook_event_type_serialization() {
        let event = WebhookEventType::RequestCompleted;
        let json = serde_json::to_value(&event).unwrap();
        assert_eq!(json, "RequestCompleted");
    }

    #[test]
    fn test_webhook_event_type_custom_serialization() {
        let event = WebhookEventType::Custom("custom-event".to_string());
        let json = serde_json::to_value(&event).unwrap();
        assert!(json.is_object());
        assert_eq!(json["Custom"], "custom-event");
    }

    #[test]
    fn test_webhook_event_type_deserialization() {
        let json = "\"RequestStarted\"";
        let event: WebhookEventType = serde_json::from_str(json).unwrap();
        assert_eq!(event, WebhookEventType::RequestStarted);
    }

    #[test]
    fn test_webhook_event_type_hash() {
        use std::collections::HashSet;

        let mut set = HashSet::new();
        set.insert(WebhookEventType::RequestStarted);
        set.insert(WebhookEventType::RequestCompleted);
        set.insert(WebhookEventType::RequestStarted); // duplicate

        assert_eq!(set.len(), 2);
    }

    // ==================== WebhookPayload Tests ====================

    #[test]
    fn test_webhook_payload_creation() {
        let payload = WebhookPayload {
            event_type: WebhookEventType::RequestCompleted,
            timestamp: Utc::now(),
            request_context: None,
            data: serde_json::json!({"status": "success"}),
            metadata: HashMap::new(),
        };

        assert_eq!(payload.event_type, WebhookEventType::RequestCompleted);
        assert!(payload.request_context.is_none());
    }

    #[test]
    fn test_webhook_payload_with_metadata() {
        let mut metadata = HashMap::new();
        metadata.insert("user_id".to_string(), "user-123".to_string());
        metadata.insert("request_id".to_string(), "req-456".to_string());

        let payload = WebhookPayload {
            event_type: WebhookEventType::CostThresholdExceeded,
            timestamp: Utc::now(),
            request_context: None,
            data: serde_json::json!({"cost": 100.0, "threshold": 50.0}),
            metadata,
        };

        assert_eq!(payload.metadata.len(), 2);
        assert_eq!(
            payload.metadata.get("user_id"),
            Some(&"user-123".to_string())
        );
    }

    #[test]
    fn test_webhook_payload_clone() {
        let payload = WebhookPayload {
            event_type: WebhookEventType::CacheEvent,
            timestamp: Utc::now(),
            request_context: None,
            data: serde_json::json!({}),
            metadata: HashMap::new(),
        };

        let cloned = payload.clone();
        assert_eq!(payload.event_type, cloned.event_type);
    }

    #[test]
    fn test_webhook_payload_serialization() {
        let payload = WebhookPayload {
            event_type: WebhookEventType::RequestStarted,
            timestamp: Utc::now(),
            request_context: None,
            data: serde_json::json!({"model": "gpt-4"}),
            metadata: HashMap::new(),
        };

        let json = serde_json::to_value(&payload).unwrap();
        assert_eq!(json["event_type"], "RequestStarted");
        assert_eq!(json["data"]["model"], "gpt-4");
    }

    // ==================== WebhookConfig Tests ====================

    #[test]
    fn test_webhook_config_default() {
        let config = WebhookConfig::default();

        assert!(config.url.is_empty());
        assert!(config.events.is_empty());
        assert!(config.headers.is_empty());
        assert!(config.secret.is_none());
        assert_eq!(config.timeout_seconds, 30);
        assert_eq!(config.max_retries, 3);
        assert_eq!(config.retry_delay_seconds, 5);
        assert!(config.enabled);
    }

    #[test]
    fn test_webhook_config_custom() {
        let mut headers = HashMap::new();
        headers.insert("Authorization".to_string(), "Bearer token123".to_string());

        let config = WebhookConfig {
            url: "https://example.com/webhook".to_string(),
            events: vec![
                WebhookEventType::RequestCompleted,
                WebhookEventType::RequestFailed,
            ],
            headers,
            secret: Some("my-secret".to_string()),
            timeout_seconds: 60,
            max_retries: 5,
            retry_delay_seconds: 10,
            enabled: true,
        };

        assert_eq!(config.url, "https://example.com/webhook");
        assert_eq!(config.events.len(), 2);
        assert!(config.headers.contains_key("Authorization"));
        assert_eq!(config.secret, Some("my-secret".to_string()));
    }

    #[test]
    fn test_webhook_config_disabled() {
        let config = WebhookConfig {
            enabled: false,
            ..Default::default()
        };

        assert!(!config.enabled);
    }

    #[test]
    fn test_webhook_config_clone() {
        let config = WebhookConfig {
            url: "https://test.com".to_string(),
            events: vec![WebhookEventType::UserCreated],
            ..Default::default()
        };

        let cloned = config.clone();
        assert_eq!(config.url, cloned.url);
        assert_eq!(config.events.len(), cloned.events.len());
    }

    #[test]
    fn test_webhook_config_serialization() {
        let config = WebhookConfig {
            url: "https://api.example.com/hooks".to_string(),
            events: vec![WebhookEventType::SecurityAlert],
            headers: HashMap::new(),
            secret: Some("secret123".to_string()),
            timeout_seconds: 45,
            max_retries: 2,
            retry_delay_seconds: 3,
            enabled: true,
        };

        let json = serde_json::to_value(&config).unwrap();
        assert_eq!(json["url"], "https://api.example.com/hooks");
        assert_eq!(json["timeout_seconds"], 45);
        assert_eq!(json["secret"], "secret123");
    }

    // ==================== WebhookDeliveryStatus Tests ====================

    #[test]
    fn test_webhook_delivery_status_variants() {
        assert_eq!(
            WebhookDeliveryStatus::Pending,
            WebhookDeliveryStatus::Pending
        );
        assert_ne!(
            WebhookDeliveryStatus::Pending,
            WebhookDeliveryStatus::Delivered
        );
        assert_ne!(
            WebhookDeliveryStatus::Failed,
            WebhookDeliveryStatus::Retrying
        );
    }

    #[test]
    fn test_webhook_delivery_status_serialization() {
        let status = WebhookDeliveryStatus::Delivered;
        let json = serde_json::to_value(&status).unwrap();
        assert_eq!(json, "Delivered");
    }

    #[test]
    fn test_webhook_delivery_status_deserialization() {
        let json = "\"Failed\"";
        let status: WebhookDeliveryStatus = serde_json::from_str(json).unwrap();
        assert_eq!(status, WebhookDeliveryStatus::Failed);
    }

    // ==================== WebhookDelivery Tests ====================

    #[test]
    fn test_webhook_delivery_creation() {
        let now = Utc::now();
        let delivery = WebhookDelivery {
            id: "delivery-123".to_string(),
            webhook_id: "webhook-456".to_string(),
            payload: WebhookPayload {
                event_type: WebhookEventType::RequestCompleted,
                timestamp: now,
                request_context: None,
                data: serde_json::json!({}),
                metadata: HashMap::new(),
            },
            status: WebhookDeliveryStatus::Pending,
            response_status: None,
            response_body: None,
            attempts: 0,
            created_at: now,
            last_attempt_at: None,
            next_retry_at: None,
        };

        assert_eq!(delivery.id, "delivery-123");
        assert_eq!(delivery.status, WebhookDeliveryStatus::Pending);
        assert_eq!(delivery.attempts, 0);
    }

    #[test]
    fn test_webhook_delivery_after_attempts() {
        let now = Utc::now();
        let delivery = WebhookDelivery {
            id: "delivery-789".to_string(),
            webhook_id: "webhook-001".to_string(),
            payload: WebhookPayload {
                event_type: WebhookEventType::RequestFailed,
                timestamp: now,
                request_context: None,
                data: serde_json::json!({"error": "timeout"}),
                metadata: HashMap::new(),
            },
            status: WebhookDeliveryStatus::Retrying,
            response_status: Some(500),
            response_body: Some("Internal Server Error".to_string()),
            attempts: 2,
            created_at: now - chrono::Duration::minutes(5),
            last_attempt_at: Some(now),
            next_retry_at: Some(now + chrono::Duration::seconds(10)),
        };

        assert_eq!(delivery.attempts, 2);
        assert_eq!(delivery.response_status, Some(500));
        assert!(delivery.last_attempt_at.is_some());
        assert!(delivery.next_retry_at.is_some());
    }

    #[test]
    fn test_webhook_delivery_success() {
        let now = Utc::now();
        let delivery = WebhookDelivery {
            id: "success-delivery".to_string(),
            webhook_id: "webhook-success".to_string(),
            payload: WebhookPayload {
                event_type: WebhookEventType::BatchCompleted,
                timestamp: now,
                request_context: None,
                data: serde_json::json!({"batch_id": "batch-123"}),
                metadata: HashMap::new(),
            },
            status: WebhookDeliveryStatus::Delivered,
            response_status: Some(200),
            response_body: Some("OK".to_string()),
            attempts: 1,
            created_at: now,
            last_attempt_at: Some(now),
            next_retry_at: None,
        };

        assert_eq!(delivery.status, WebhookDeliveryStatus::Delivered);
        assert_eq!(delivery.response_status, Some(200));
    }

    #[test]
    fn test_webhook_delivery_clone() {
        let now = Utc::now();
        let delivery = WebhookDelivery {
            id: "clone-test".to_string(),
            webhook_id: "webhook-clone".to_string(),
            payload: WebhookPayload {
                event_type: WebhookEventType::CacheEvent,
                timestamp: now,
                request_context: None,
                data: serde_json::json!({}),
                metadata: HashMap::new(),
            },
            status: WebhookDeliveryStatus::Pending,
            response_status: None,
            response_body: None,
            attempts: 0,
            created_at: now,
            last_attempt_at: None,
            next_retry_at: None,
        };

        let cloned = delivery.clone();
        assert_eq!(delivery.id, cloned.id);
        assert_eq!(delivery.status, cloned.status);
    }

    #[test]
    fn test_webhook_delivery_serialization() {
        let now = Utc::now();
        let delivery = WebhookDelivery {
            id: "ser-test".to_string(),
            webhook_id: "webhook-ser".to_string(),
            payload: WebhookPayload {
                event_type: WebhookEventType::UserCreated,
                timestamp: now,
                request_context: None,
                data: serde_json::json!({"user_id": "user-new"}),
                metadata: HashMap::new(),
            },
            status: WebhookDeliveryStatus::Delivered,
            response_status: Some(201),
            response_body: None,
            attempts: 1,
            created_at: now,
            last_attempt_at: Some(now),
            next_retry_at: None,
        };

        let json = serde_json::to_value(&delivery).unwrap();
        assert_eq!(json["id"], "ser-test");
        assert_eq!(json["status"], "Delivered");
        assert_eq!(json["attempts"], 1);
    }

    // ==================== WebhookStats Tests ====================

    #[test]
    fn test_webhook_stats_default() {
        let stats = WebhookStats::default();

        assert_eq!(stats.total_events, 0);
        assert_eq!(stats.successful_deliveries, 0);
        assert_eq!(stats.failed_deliveries, 0);
        assert_eq!(stats.avg_delivery_time_ms, 0.0);
        assert!(stats.events_by_type.is_empty());
    }

    #[test]
    fn test_webhook_stats_with_data() {
        let mut events_by_type = HashMap::new();
        events_by_type.insert("RequestCompleted".to_string(), 100);
        events_by_type.insert("RequestFailed".to_string(), 10);

        let stats = WebhookStats {
            total_events: 110,
            successful_deliveries: 100,
            failed_deliveries: 10,
            avg_delivery_time_ms: 150.5,
            events_by_type,
        };

        assert_eq!(stats.total_events, 110);
        assert_eq!(stats.successful_deliveries, 100);
        assert_eq!(stats.failed_deliveries, 10);
        assert_eq!(stats.avg_delivery_time_ms, 150.5);
        assert_eq!(stats.events_by_type.len(), 2);
    }

    #[test]
    fn test_webhook_stats_clone() {
        let stats = WebhookStats {
            total_events: 50,
            successful_deliveries: 45,
            failed_deliveries: 5,
            avg_delivery_time_ms: 100.0,
            events_by_type: HashMap::new(),
        };

        let cloned = stats.clone();
        assert_eq!(stats.total_events, cloned.total_events);
        assert_eq!(stats.successful_deliveries, cloned.successful_deliveries);
    }

    #[test]
    fn test_webhook_stats_serialization() {
        let mut events_by_type = HashMap::new();
        events_by_type.insert("CacheEvent".to_string(), 50);

        let stats = WebhookStats {
            total_events: 200,
            successful_deliveries: 180,
            failed_deliveries: 20,
            avg_delivery_time_ms: 75.25,
            events_by_type,
        };

        let json = serde_json::to_value(&stats).unwrap();
        assert_eq!(json["total_events"], 200);
        assert_eq!(json["successful_deliveries"], 180);
        assert_eq!(json["avg_delivery_time_ms"], 75.25);
    }

    // ==================== WebhookData Tests ====================

    #[test]
    fn test_webhook_data_default() {
        let data = WebhookData::default();

        assert!(data.webhooks.is_empty());
        assert!(data.delivery_queue.is_empty());
        assert_eq!(data.stats.total_events, 0);
    }

    // ==================== Integration Tests ====================

    #[test]
    fn test_webhook_workflow() {
        // 1. Create a webhook config
        let config = WebhookConfig {
            url: "https://api.example.com/webhook".to_string(),
            events: vec![
                WebhookEventType::RequestCompleted,
                WebhookEventType::RequestFailed,
            ],
            headers: {
                let mut h = HashMap::new();
                h.insert("X-API-Key".to_string(), "key123".to_string());
                h
            },
            secret: Some("secret".to_string()),
            timeout_seconds: 30,
            max_retries: 3,
            retry_delay_seconds: 5,
            enabled: true,
        };

        assert!(config.enabled);
        assert!(config.events.contains(&WebhookEventType::RequestCompleted));

        // 2. Create a payload
        let now = Utc::now();
        let payload = WebhookPayload {
            event_type: WebhookEventType::RequestCompleted,
            timestamp: now,
            request_context: None,
            data: serde_json::json!({
                "model": "gpt-4",
                "tokens": 500,
                "cost": 0.05
            }),
            metadata: {
                let mut m = HashMap::new();
                m.insert("request_id".to_string(), "req-123".to_string());
                m
            },
        };

        // 3. Create a delivery
        let delivery = WebhookDelivery {
            id: "del-001".to_string(),
            webhook_id: "hook-001".to_string(),
            payload: payload.clone(),
            status: WebhookDeliveryStatus::Pending,
            response_status: None,
            response_body: None,
            attempts: 0,
            created_at: now,
            last_attempt_at: None,
            next_retry_at: None,
        };

        assert_eq!(delivery.status, WebhookDeliveryStatus::Pending);

        // 4. Simulate successful delivery
        let delivered = WebhookDelivery {
            status: WebhookDeliveryStatus::Delivered,
            response_status: Some(200),
            response_body: Some("Received".to_string()),
            attempts: 1,
            last_attempt_at: Some(now),
            ..delivery.clone()
        };

        assert_eq!(delivered.status, WebhookDeliveryStatus::Delivered);
        assert_eq!(delivered.response_status, Some(200));

        // 5. Update stats
        let stats = WebhookStats {
            total_events: 1,
            successful_deliveries: 1,
            failed_deliveries: 0,
            avg_delivery_time_ms: 50.0,
            events_by_type: {
                let mut m = HashMap::new();
                m.insert("RequestCompleted".to_string(), 1);
                m
            },
        };

        assert_eq!(
            stats.total_events,
            stats.successful_deliveries + stats.failed_deliveries
        );
    }

    #[test]
    fn test_webhook_retry_scenario() {
        let now = Utc::now();

        // Initial attempt fails
        let attempt1 = WebhookDelivery {
            id: "del-retry".to_string(),
            webhook_id: "hook-retry".to_string(),
            payload: WebhookPayload {
                event_type: WebhookEventType::BatchFailed,
                timestamp: now,
                request_context: None,
                data: serde_json::json!({"error": "processing failed"}),
                metadata: HashMap::new(),
            },
            status: WebhookDeliveryStatus::Retrying,
            response_status: Some(503),
            response_body: Some("Service Unavailable".to_string()),
            attempts: 1,
            created_at: now,
            last_attempt_at: Some(now),
            next_retry_at: Some(now + chrono::Duration::seconds(5)),
        };

        assert_eq!(attempt1.attempts, 1);
        assert_eq!(attempt1.status, WebhookDeliveryStatus::Retrying);

        // Second attempt succeeds
        let attempt2 = WebhookDelivery {
            status: WebhookDeliveryStatus::Delivered,
            response_status: Some(200),
            response_body: Some("OK".to_string()),
            attempts: 2,
            last_attempt_at: Some(now + chrono::Duration::seconds(5)),
            next_retry_at: None,
            ..attempt1.clone()
        };

        assert_eq!(attempt2.attempts, 2);
        assert_eq!(attempt2.status, WebhookDeliveryStatus::Delivered);
    }
}
