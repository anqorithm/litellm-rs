//! Webhook event builders
//!
//! This module provides convenience functions for building common webhook events.

use super::types::WebhookEventType;
use crate::core::types::context::RequestContext;

/// Build request started event
pub fn request_started(
    model: &str,
    provider: &str,
    context: RequestContext,
) -> (WebhookEventType, serde_json::Value) {
    (
        WebhookEventType::RequestStarted,
        serde_json::json!({
            "model": model,
            "provider": provider,
            "request_id": context.request_id,
            "user_id": context.user_id,
            "timestamp": chrono::Utc::now()
        }),
    )
}

/// Build request completed event
pub fn request_completed(
    model: &str,
    provider: &str,
    tokens_used: u32,
    cost: f64,
    latency_ms: u64,
    context: RequestContext,
) -> (WebhookEventType, serde_json::Value) {
    (
        WebhookEventType::RequestCompleted,
        serde_json::json!({
            "model": model,
            "provider": provider,
            "tokens_used": tokens_used,
            "cost": cost,
            "latency_ms": latency_ms,
            "request_id": context.request_id,
            "user_id": context.user_id,
            "timestamp": chrono::Utc::now()
        }),
    )
}

/// Build request failed event
pub fn request_failed(
    model: &str,
    provider: &str,
    error: &str,
    context: RequestContext,
) -> (WebhookEventType, serde_json::Value) {
    (
        WebhookEventType::RequestFailed,
        serde_json::json!({
            "model": model,
            "provider": provider,
            "error": error,
            "request_id": context.request_id,
            "user_id": context.user_id,
            "timestamp": chrono::Utc::now()
        }),
    )
}

/// Build cost threshold exceeded event
pub fn cost_threshold_exceeded(
    user_id: &str,
    current_cost: f64,
    threshold: f64,
    period: &str,
) -> (WebhookEventType, serde_json::Value) {
    (
        WebhookEventType::CostThresholdExceeded,
        serde_json::json!({
            "user_id": user_id,
            "current_cost": current_cost,
            "threshold": threshold,
            "period": period,
            "timestamp": chrono::Utc::now()
        }),
    )
}
