//! Webhook tests
//!
//! This module contains unit tests for webhook functionality.

#[cfg(test)]
use super::manager::WebhookManager;
use super::types::{WebhookConfig, WebhookEventType, WebhookPayload};
use std::collections::HashMap;

#[tokio::test]
async fn test_webhook_manager_creation() {
    let manager = WebhookManager::new().unwrap();
    let webhooks = manager.list_webhooks().await;
    assert!(webhooks.is_empty());
}

#[tokio::test]
async fn test_webhook_registration() {
    let manager = WebhookManager::new().unwrap();

    let config = WebhookConfig {
        url: "https://example.com/webhook".to_string(),
        events: vec![WebhookEventType::RequestCompleted],
        ..Default::default()
    };

    manager
        .register_webhook("test".to_string(), config)
        .await
        .unwrap();

    let webhooks = manager.list_webhooks().await;
    assert_eq!(webhooks.len(), 1);
    assert!(webhooks.contains_key("test"));
}

#[test]
fn test_webhook_event_types() {
    let event = WebhookEventType::RequestStarted;
    assert_eq!(event, WebhookEventType::RequestStarted);

    let custom_event = WebhookEventType::Custom("my_event".to_string());
    assert_eq!(
        custom_event,
        WebhookEventType::Custom("my_event".to_string())
    );
}

#[test]
fn test_webhook_payload_serialization() {
    let payload = WebhookPayload {
        event_type: WebhookEventType::RequestCompleted,
        timestamp: chrono::Utc::now(),
        request_context: None,
        data: serde_json::json!({"test": "data"}),
        metadata: HashMap::new(),
    };

    let serialized = serde_json::to_string(&payload).unwrap();
    assert!(serialized.contains("RequestCompleted"));
    assert!(serialized.contains("test"));
}
