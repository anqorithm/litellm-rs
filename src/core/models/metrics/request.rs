//! Request metrics models

use super::super::Metadata;
use super::cache::CacheMetrics;
use super::cost::CostInfo;
use super::error::ErrorInfo;
use super::token::TokenUsage;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Request metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestMetrics {
    /// Metrics metadata
    #[serde(flatten)]
    pub metadata: Metadata,
    /// Request ID
    pub request_id: String,
    /// User ID
    pub user_id: Option<Uuid>,
    /// Team ID
    pub team_id: Option<Uuid>,
    /// API Key ID
    pub api_key_id: Option<Uuid>,
    /// Model used
    pub model: String,
    /// Provider used
    pub provider: String,
    /// Request type
    pub request_type: String,
    /// Request status
    pub status: RequestStatus,
    /// HTTP status code
    pub status_code: u16,
    /// Request timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// Response time in milliseconds
    pub response_time_ms: u64,
    /// Queue time in milliseconds
    pub queue_time_ms: u64,
    /// Provider response time in milliseconds
    pub provider_time_ms: u64,
    /// Token usage
    pub token_usage: TokenUsage,
    /// Cost information
    pub cost: CostInfo,
    /// Error information
    pub error: Option<ErrorInfo>,
    /// Cache information
    pub cache: CacheMetrics,
    /// Additional metadata
    pub extra: HashMap<String, serde_json::Value>,
}

/// Request status
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RequestStatus {
    /// Request completed successfully
    Success,
    /// Request failed with error
    Error,
    /// Request timed out
    Timeout,
    /// Request hit rate limit
    RateLimit,
    /// Request exceeded quota
    QuotaExceeded,
    /// Request was cancelled
    Cancelled,
}

impl RequestMetrics {
    /// Create new request metrics
    pub fn new(request_id: String, model: String, provider: String, request_type: String) -> Self {
        Self {
            metadata: Metadata::new(),
            request_id,
            user_id: None,
            team_id: None,
            api_key_id: None,
            model,
            provider,
            request_type,
            status: RequestStatus::Success,
            status_code: 200,
            timestamp: chrono::Utc::now(),
            response_time_ms: 0,
            queue_time_ms: 0,
            provider_time_ms: 0,
            token_usage: TokenUsage::default(),
            cost: CostInfo::default(),
            error: None,
            cache: CacheMetrics::default(),
            extra: HashMap::new(),
        }
    }

    /// Set user context
    pub fn with_user(mut self, user_id: Uuid, team_id: Option<Uuid>) -> Self {
        self.user_id = Some(user_id);
        self.team_id = team_id;
        self
    }

    /// Set API key context
    pub fn with_api_key(mut self, api_key_id: Uuid) -> Self {
        self.api_key_id = Some(api_key_id);
        self
    }

    /// Set timing information
    pub fn with_timing(
        mut self,
        response_time_ms: u64,
        queue_time_ms: u64,
        provider_time_ms: u64,
    ) -> Self {
        self.response_time_ms = response_time_ms;
        self.queue_time_ms = queue_time_ms;
        self.provider_time_ms = provider_time_ms;
        self
    }

    /// Set token usage
    pub fn with_tokens(mut self, input_tokens: u32, output_tokens: u32) -> Self {
        self.token_usage.input_tokens = input_tokens;
        self.token_usage.output_tokens = output_tokens;
        self.token_usage.total_tokens = input_tokens + output_tokens;
        self
    }

    /// Set cost information
    pub fn with_cost(mut self, input_cost: f64, output_cost: f64, currency: String) -> Self {
        self.cost.input_cost = input_cost;
        self.cost.output_cost = output_cost;
        self.cost.total_cost = input_cost + output_cost;
        self.cost.currency = currency;
        self
    }

    /// Set error information
    pub fn with_error(mut self, error: ErrorInfo) -> Self {
        self.status = RequestStatus::Error;
        self.error = Some(error);
        self
    }

    /// Set cache information
    pub fn with_cache(mut self, cache: CacheMetrics) -> Self {
        self.cache = cache;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_metrics() -> RequestMetrics {
        RequestMetrics::new(
            "req-123".to_string(),
            "gpt-4".to_string(),
            "openai".to_string(),
            "chat_completion".to_string(),
        )
    }

    // ==================== RequestMetrics Creation Tests ====================

    #[test]
    fn test_request_metrics_creation() {
        let metrics = create_test_metrics();

        assert_eq!(metrics.request_id, "req-123");
        assert_eq!(metrics.model, "gpt-4");
        assert_eq!(metrics.provider, "openai");
        assert!(matches!(metrics.status, RequestStatus::Success));
    }

    #[test]
    fn test_request_metrics_default_values() {
        let metrics = create_test_metrics();

        assert!(metrics.user_id.is_none());
        assert!(metrics.team_id.is_none());
        assert!(metrics.api_key_id.is_none());
        assert_eq!(metrics.status_code, 200);
        assert_eq!(metrics.response_time_ms, 0);
        assert_eq!(metrics.queue_time_ms, 0);
        assert_eq!(metrics.provider_time_ms, 0);
        assert!(metrics.error.is_none());
        assert!(metrics.extra.is_empty());
    }

    // ==================== Builder Pattern Tests ====================

    #[test]
    fn test_with_user() {
        let user_id = Uuid::new_v4();
        let team_id = Uuid::new_v4();
        let metrics = create_test_metrics().with_user(user_id, Some(team_id));

        assert_eq!(metrics.user_id, Some(user_id));
        assert_eq!(metrics.team_id, Some(team_id));
    }

    #[test]
    fn test_with_user_no_team() {
        let user_id = Uuid::new_v4();
        let metrics = create_test_metrics().with_user(user_id, None);

        assert_eq!(metrics.user_id, Some(user_id));
        assert!(metrics.team_id.is_none());
    }

    #[test]
    fn test_with_api_key() {
        let api_key_id = Uuid::new_v4();
        let metrics = create_test_metrics().with_api_key(api_key_id);

        assert_eq!(metrics.api_key_id, Some(api_key_id));
    }

    #[test]
    fn test_with_timing() {
        let metrics = create_test_metrics().with_timing(100, 10, 80);

        assert_eq!(metrics.response_time_ms, 100);
        assert_eq!(metrics.queue_time_ms, 10);
        assert_eq!(metrics.provider_time_ms, 80);
    }

    #[test]
    fn test_with_timing_zero_values() {
        let metrics = create_test_metrics().with_timing(0, 0, 0);

        assert_eq!(metrics.response_time_ms, 0);
        assert_eq!(metrics.queue_time_ms, 0);
        assert_eq!(metrics.provider_time_ms, 0);
    }

    #[test]
    fn test_with_tokens() {
        let metrics = create_test_metrics().with_tokens(1000, 500);

        assert_eq!(metrics.token_usage.input_tokens, 1000);
        assert_eq!(metrics.token_usage.output_tokens, 500);
        assert_eq!(metrics.token_usage.total_tokens, 1500);
    }

    #[test]
    fn test_with_cost() {
        let metrics = create_test_metrics().with_cost(0.01, 0.02, "USD".to_string());

        assert_eq!(metrics.cost.input_cost, 0.01);
        assert_eq!(metrics.cost.output_cost, 0.02);
        assert_eq!(metrics.cost.total_cost, 0.03);
        assert_eq!(metrics.cost.currency, "USD");
    }

    #[test]
    fn test_with_error() {
        let error = ErrorInfo {
            code: "rate_limit".to_string(),
            message: "Too many requests".to_string(),
            error_type: "rate_limit_error".to_string(),
            provider_code: None,
            stack_trace: None,
        };
        let metrics = create_test_metrics().with_error(error);

        assert!(matches!(metrics.status, RequestStatus::Error));
        assert!(metrics.error.is_some());
        assert_eq!(metrics.error.as_ref().unwrap().code, "rate_limit");
    }

    #[test]
    fn test_with_cache() {
        let cache = CacheMetrics {
            hit: true,
            ..Default::default()
        };
        let metrics = create_test_metrics().with_cache(cache);

        assert!(metrics.cache.hit);
    }

    // ==================== Builder Chaining Tests ====================

    #[test]
    fn test_builder_chain() {
        let user_id = Uuid::new_v4();
        let api_key_id = Uuid::new_v4();

        let metrics = create_test_metrics()
            .with_user(user_id, None)
            .with_api_key(api_key_id)
            .with_timing(150, 20, 120)
            .with_tokens(2000, 1000)
            .with_cost(0.05, 0.10, "EUR".to_string());

        assert_eq!(metrics.user_id, Some(user_id));
        assert_eq!(metrics.api_key_id, Some(api_key_id));
        assert_eq!(metrics.response_time_ms, 150);
        assert_eq!(metrics.token_usage.total_tokens, 3000);
        assert!((metrics.cost.total_cost - 0.15).abs() < 1e-10);
    }

    // ==================== RequestStatus Tests ====================

    #[test]
    fn test_request_status_success() {
        let metrics = create_test_metrics();
        assert!(matches!(metrics.status, RequestStatus::Success));
    }

    #[test]
    fn test_request_status_error() {
        let error = ErrorInfo {
            code: "error".to_string(),
            message: "Error".to_string(),
            error_type: "generic_error".to_string(),
            provider_code: None,
            stack_trace: None,
        };
        let metrics = create_test_metrics().with_error(error);
        assert!(matches!(metrics.status, RequestStatus::Error));
    }

    // ==================== Serialization Tests ====================

    #[test]
    fn test_request_status_serialization() {
        let status = RequestStatus::RateLimit;
        let json = serde_json::to_value(&status).unwrap();
        assert_eq!(json, "rate_limit");
    }

    #[test]
    fn test_request_status_deserialization() {
        let json = "\"quota_exceeded\"";
        let status: RequestStatus = serde_json::from_str(json).unwrap();
        assert!(matches!(status, RequestStatus::QuotaExceeded));
    }

    #[test]
    fn test_request_metrics_serialization() {
        let metrics = create_test_metrics();
        let json = serde_json::to_value(&metrics).unwrap();

        assert_eq!(json["request_id"], "req-123");
        assert_eq!(json["model"], "gpt-4");
        assert_eq!(json["provider"], "openai");
    }

    // ==================== Clone Tests ====================

    #[test]
    fn test_request_metrics_clone() {
        let original = create_test_metrics().with_tokens(100, 50);
        let cloned = original.clone();

        assert_eq!(original.request_id, cloned.request_id);
        assert_eq!(
            original.token_usage.input_tokens,
            cloned.token_usage.input_tokens
        );
    }

    // ==================== Extra Metadata Tests ====================

    #[test]
    fn test_extra_metadata() {
        let mut metrics = create_test_metrics();
        metrics
            .extra
            .insert("custom_key".to_string(), serde_json::json!("custom_value"));
        metrics
            .extra
            .insert("numeric".to_string(), serde_json::json!(42));

        assert_eq!(metrics.extra.len(), 2);
        assert_eq!(metrics.extra.get("custom_key").unwrap(), "custom_value");
    }
}
