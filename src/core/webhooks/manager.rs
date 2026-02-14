//! Webhook manager implementation
//!
//! This module contains the WebhookManager struct and its core functionality.

use super::types::{
    WebhookConfig, WebhookData, WebhookDelivery, WebhookDeliveryStatus, WebhookEventType,
    WebhookPayload, WebhookStats,
};
use crate::core::types::context::RequestContext;
use crate::utils::error::gateway_error::{GatewayError, Result};
use crate::utils::net::http::create_custom_client;
use reqwest::Client;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tracing::{debug, error, info};
use uuid::Uuid;

/// Webhook manager
pub struct WebhookManager {
    /// HTTP client for webhook requests
    pub(super) client: Client,
    /// Consolidated webhook data - single lock for all related state
    pub(super) data: Arc<RwLock<WebhookData>>,
}

impl WebhookManager {
    /// Create a new webhook manager
    pub fn new() -> Result<Self> {
        let client = create_custom_client(Duration::from_secs(30))
            .map_err(|e| GatewayError::Network(format!("Failed to create HTTP client: {}", e)))?;

        Ok(Self {
            client,
            data: Arc::new(RwLock::new(WebhookData::default())),
        })
    }

    /// Create a new webhook manager with default settings, panics on failure
    /// Use `new()` for fallible construction
    pub fn new_or_default() -> Self {
        Self::new().unwrap_or_else(|e| {
            tracing::error!(
                "Failed to create WebhookManager: {}, using minimal client",
                e
            );
            // Create a minimal client as fallback
            Self {
                client: Client::new(),
                data: Arc::new(RwLock::new(WebhookData::default())),
            }
        })
    }

    /// Register a webhook
    pub async fn register_webhook(&self, id: String, config: WebhookConfig) -> Result<()> {
        info!("Registering webhook: {} -> {}", id, config.url);

        // Validate webhook URL
        if config.url.is_empty() {
            return Err(GatewayError::Validation(
                "Webhook URL cannot be empty".to_string(),
            ));
        }

        if !config.url.starts_with("http://") && !config.url.starts_with("https://") {
            return Err(GatewayError::Validation(
                "Webhook URL must be HTTP or HTTPS".to_string(),
            ));
        }

        let mut data = self.data.write().await;
        data.webhooks.insert(id, config);

        Ok(())
    }

    /// Unregister a webhook
    pub async fn unregister_webhook(&self, id: &str) -> Result<()> {
        info!("Unregistering webhook: {}", id);

        let mut data = self.data.write().await;
        data.webhooks.remove(id);

        Ok(())
    }

    /// Send webhook event
    pub async fn send_event(
        &self,
        event_type: WebhookEventType,
        event_data: serde_json::Value,
        context: Option<RequestContext>,
    ) -> Result<()> {
        let payload = WebhookPayload {
            event_type: event_type.clone(),
            timestamp: chrono::Utc::now(),
            request_context: context,
            data: event_data,
            metadata: HashMap::new(),
        };

        let mut data = self.data.write().await;
        // Pre-allocate with estimated capacity (most events go to few webhooks)
        let mut deliveries = Vec::with_capacity(data.webhooks.len().min(8));

        // Find webhooks subscribed to this event type
        for (webhook_id, config) in data.webhooks.iter() {
            if config.enabled && config.events.contains(&event_type) {
                let delivery = WebhookDelivery {
                    id: Uuid::new_v4().to_string(),
                    webhook_id: webhook_id.clone(),
                    payload: payload.clone(),
                    status: WebhookDeliveryStatus::Pending,
                    response_status: None,
                    response_body: None,
                    attempts: 0,
                    created_at: chrono::Utc::now(),
                    last_attempt_at: None,
                    next_retry_at: Some(chrono::Utc::now()),
                };
                deliveries.push(delivery);
            }
        }

        // Add to delivery queue and update statistics
        let delivery_count = deliveries.len();
        if !deliveries.is_empty() {
            data.delivery_queue.extend(deliveries);
            data.stats.total_events += 1;
            *data
                .stats
                .events_by_type
                .entry(format!("{:?}", event_type))
                .or_insert(0) += 1;
        }

        debug!(
            "Queued {} webhook deliveries for event: {:?}",
            delivery_count, event_type
        );
        Ok(())
    }

    /// Get webhook statistics
    pub async fn get_stats(&self) -> WebhookStats {
        self.data.read().await.stats.clone()
    }

    /// List all registered webhooks
    pub async fn list_webhooks(&self) -> HashMap<String, WebhookConfig> {
        self.data.read().await.webhooks.clone()
    }

    /// Get delivery history
    pub async fn get_delivery_history(&self, limit: Option<usize>) -> Vec<WebhookDelivery> {
        let data = self.data.read().await;
        let limit = limit.unwrap_or(100).min(data.delivery_queue.len());
        // Pre-allocate with exact capacity and collect from reverse iterator
        let mut result = Vec::with_capacity(limit);
        result.extend(data.delivery_queue.iter().rev().take(limit).cloned());
        result
    }

    /// Start background delivery processor
    pub async fn start_delivery_processor(&self) -> Result<()> {
        let manager = self.clone();

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(5));

            loop {
                interval.tick().await;

                if let Err(e) = manager.process_delivery_queue().await {
                    error!("Error processing webhook delivery queue: {}", e);
                }
            }
        });

        info!("Started webhook delivery processor");
        Ok(())
    }
}

impl Clone for WebhookManager {
    fn clone(&self) -> Self {
        Self {
            client: self.client.clone(),
            data: self.data.clone(),
        }
    }
}

impl Default for WebhookManager {
    fn default() -> Self {
        Self::new_or_default()
    }
}
