//! Core data models for the Gateway
//!
//! This module defines all the core data structures used throughout the gateway.

pub mod deployment;
pub mod metrics;
pub mod openai;
pub mod request;
pub mod response;
pub mod team;
pub mod user;

// No top-level team/user re-exports: use explicit module paths from call sites.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Common metadata for all models
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Metadata {
    /// Unique identifier
    pub id: Uuid,
    /// Creation timestamp
    pub created_at: chrono::DateTime<chrono::Utc>,
    /// Last update timestamp
    pub updated_at: chrono::DateTime<chrono::Utc>,
    /// Version for optimistic locking
    pub version: i64,
    /// Additional metadata
    #[serde(default)]
    pub extra: HashMap<String, serde_json::Value>,
}

impl Default for Metadata {
    fn default() -> Self {
        let now = chrono::Utc::now();
        Self {
            id: Uuid::new_v4(),
            created_at: now,
            updated_at: now,
            version: 1,
            extra: HashMap::new(),
        }
    }
}

impl Metadata {
    /// Create new metadata
    pub fn new() -> Self {
        Self::default()
    }

    /// Update the timestamp and increment version
    pub fn touch(&mut self) {
        self.updated_at = chrono::Utc::now();
        self.version += 1;
    }

    /// Set extra metadata
    pub fn set_extra<K: Into<String>, V: Into<serde_json::Value>>(&mut self, key: K, value: V) {
        self.extra.insert(key.into(), value.into());
    }

    /// Get extra metadata
    pub fn get_extra(&self, key: &str) -> Option<&serde_json::Value> {
        self.extra.get(key)
    }
}

/// API Key information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiKey {
    /// Metadata
    #[serde(flatten)]
    pub metadata: Metadata,
    /// Key name/description
    pub name: String,
    /// Hashed key value
    pub key_hash: String,
    /// Key prefix for identification
    pub key_prefix: String,
    /// Associated user ID
    pub user_id: Option<Uuid>,
    /// Associated team ID
    pub team_id: Option<Uuid>,
    /// Permissions
    pub permissions: Vec<String>,
    /// Rate limits
    pub rate_limits: Option<RateLimits>,
    /// Expiration date
    pub expires_at: Option<chrono::DateTime<chrono::Utc>>,
    /// Whether the key is active
    pub is_active: bool,
    /// Last used timestamp
    pub last_used_at: Option<chrono::DateTime<chrono::Utc>>,
    /// Usage statistics
    pub usage_stats: UsageStats,
}

/// Rate limits for API keys
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimits {
    /// Requests per minute
    pub rpm: Option<u32>,
    /// Tokens per minute
    pub tpm: Option<u32>,
    /// Requests per day
    pub rpd: Option<u32>,
    /// Tokens per day
    pub tpd: Option<u32>,
    /// Concurrent requests
    pub concurrent: Option<u32>,
}

/// Usage statistics
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UsageStats {
    /// Total requests
    pub total_requests: u64,
    /// Total tokens
    pub total_tokens: u64,
    /// Total cost
    pub total_cost: f64,
    /// Requests today
    pub requests_today: u32,
    /// Tokens today
    pub tokens_today: u32,
    /// Cost today
    pub cost_today: f64,
    /// Last reset date
    pub last_reset: chrono::DateTime<chrono::Utc>,
}

/// Model information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelInfo {
    /// Model ID
    pub id: String,
    /// Model name
    pub name: String,
    /// Provider
    pub provider: String,
    /// Model type (chat, completion, embedding, etc.)
    pub model_type: ModelType,
    /// Context window size
    pub context_window: Option<u32>,
    /// Maximum output tokens
    pub max_output_tokens: Option<u32>,
    /// Input cost per token
    pub input_cost_per_token: Option<f64>,
    /// Output cost per token
    pub output_cost_per_token: Option<f64>,
    /// Supported features
    pub features: Vec<String>,
    /// Model description
    pub description: Option<String>,
    /// Whether the model is available
    pub is_available: bool,
}

/// Model types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ModelType {
    /// Chat completion models
    Chat,
    /// Text completion models
    Completion,
    /// Text embedding models
    Embedding,
    /// Image generation models
    ImageGeneration,
    /// Audio transcription models
    AudioTranscription,
    /// Audio translation models
    AudioTranslation,
    /// Content moderation models
    Moderation,
    /// Fine-tuning models
    FineTuning,
    /// Document reranking models
    Rerank,
}

/// Canonical request context type used across the gateway.
pub type RequestContext = crate::core::types::context::RequestContext;

/// Provider health information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderHealth {
    /// Provider name
    pub provider: String,
    /// Health status
    pub status: HealthStatus,
    /// Last check timestamp
    pub last_check: chrono::DateTime<chrono::Utc>,
    /// Response time in milliseconds
    pub response_time_ms: Option<u64>,
    /// Error message if unhealthy
    pub error_message: Option<String>,
    /// Success rate (0.0 to 1.0)
    pub success_rate: f64,
    /// Total requests in the last period
    pub total_requests: u64,
    /// Failed requests in the last period
    pub failed_requests: u64,
}

/// Health status enum
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum HealthStatus {
    /// Service is operating normally
    Healthy,
    /// Service is experiencing minor issues
    Degraded,
    /// Service is not functioning properly
    Unhealthy,
    /// Health status cannot be determined
    Unknown,
}

/// Provider registry health summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderRegistryHealth {
    /// Total number of providers
    pub total_count: usize,
    /// Number of healthy providers
    pub healthy_count: usize,
    /// Number of degraded providers
    pub degraded_count: usize,
    /// Number of unhealthy providers
    pub unhealthy_count: usize,
    /// Individual provider health
    pub providers: Vec<ProviderHealth>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metadata_creation() {
        let metadata = Metadata::new();
        assert_eq!(metadata.version, 1);
        assert!(metadata.created_at <= chrono::Utc::now());
    }

    #[test]
    fn test_metadata_touch() {
        let mut metadata = Metadata::new();
        let original_version = metadata.version;
        let original_updated = metadata.updated_at;

        std::thread::sleep(std::time::Duration::from_millis(1));
        metadata.touch();

        assert_eq!(metadata.version, original_version + 1);
        assert!(metadata.updated_at > original_updated);
    }

    #[test]
    fn test_request_context_creation() {
        let context = RequestContext::new();
        assert!(!context.request_id.is_empty());
        assert!(context.user_id.is_none());
    }

    #[test]
    fn test_request_context_builder() {
        let user_id = Uuid::new_v4();
        let team_id = Uuid::new_v4();

        let context = RequestContext::new()
            .with_user(user_id, Some(team_id))
            .with_client_info(
                Some("127.0.0.1".to_string()),
                Some("test-agent".to_string()),
            )
            .add_header("X-Custom", "value");

        assert_eq!(context.user_id, Some(user_id.to_string()));
        assert_eq!(context.team_id(), Some(team_id));
        assert_eq!(context.client_ip, Some("127.0.0.1".to_string()));
        assert_eq!(context.headers.get("X-Custom"), Some(&"value".to_string()));
    }
}
