//! Core deployment type definitions
//!
//! This module defines the main deployment structure and related types.

use crate::config::models::provider::ProviderConfig;
use crate::core::models::{HealthStatus, Metadata};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

use crate::core::models::deployment::health::DeploymentHealth;
use crate::core::models::deployment::metrics::DeploymentMetrics;

/// Provider deployment configuration
#[derive(Debug, Clone)]
pub struct Deployment {
    /// Deployment metadata
    pub metadata: Metadata,
    /// Deployment configuration
    pub config: ProviderConfig,
    /// Current health status
    pub health: Arc<DeploymentHealth>,
    /// Runtime metrics
    pub metrics: Arc<DeploymentMetrics>,
    /// Deployment state
    pub state: DeploymentState,
    /// Tags for routing
    pub tags: Vec<String>,
    /// Weight for load balancing
    pub weight: f32,
    /// Rate limits
    pub rate_limits: Option<DeploymentRateLimits>,
    /// Cost configuration
    pub cost_config: Option<DeploymentCostConfig>,
}

/// Deployment state
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DeploymentState {
    /// Deployment is active and healthy
    Active,
    /// Deployment is active but degraded
    Degraded,
    /// Deployment is temporarily disabled
    Disabled,
    /// Deployment is draining (no new requests)
    Draining,
    /// Deployment is in maintenance mode
    Maintenance,
    /// Deployment failed health checks
    Failed,
}

/// Deployment rate limits
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeploymentRateLimits {
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
    /// Burst allowance
    pub burst: Option<u32>,
}

/// Deployment cost configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeploymentCostConfig {
    /// Cost per input token
    pub input_cost_per_token: Option<f64>,
    /// Cost per output token
    pub output_cost_per_token: Option<f64>,
    /// Cost per request
    pub cost_per_request: Option<f64>,
    /// Cost per image
    pub cost_per_image: Option<f64>,
    /// Cost per audio second
    pub cost_per_audio_second: Option<f64>,
    /// Currency
    pub currency: String,
    /// Billing model
    pub billing_model: BillingModel,
}

/// Billing model
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BillingModel {
    /// Pay-per-use billing
    PayPerUse,
    /// Subscription billing
    Subscription,
    /// Prepaid billing
    Prepaid,
    /// Free billing
    Free,
}

/// Deployment snapshot for serialization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeploymentSnapshot {
    /// Deployment ID
    pub id: Uuid,
    /// Provider name
    pub name: String,
    /// Provider type
    pub provider_type: String,
    /// Model name
    pub model: String,
    /// Current state
    pub state: DeploymentState,
    /// Health status
    pub health_status: HealthStatus,
    /// Weight
    pub weight: f32,
    /// Tags
    pub tags: Vec<String>,
    /// Metrics snapshot
    pub metrics: DeploymentMetricsSnapshot,
    /// Last updated
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// Deployment metrics snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeploymentMetricsSnapshot {
    /// Total requests
    pub total_requests: u64,
    /// Successful requests
    pub successful_requests: u64,
    /// Failed requests
    pub failed_requests: u64,
    /// Success rate percentage
    pub success_rate: f64,
    /// Total tokens
    pub total_tokens: u64,
    /// Total cost
    pub total_cost: f64,
    /// Active connections
    pub active_connections: u32,
    /// Average response time in milliseconds
    pub avg_response_time: u64,
    /// P95 response time in milliseconds
    pub p95_response_time: u64,
    /// P99 response time in milliseconds
    pub p99_response_time: u64,
    /// Request rate (RPM)
    pub request_rate: u32,
    /// Token rate (TPM)
    pub token_rate: u32,
}

#[cfg(test)]
mod tests {
    use super::*;

    // ==================== DeploymentState Tests ====================

    #[test]
    fn test_deployment_state_active() {
        let state = DeploymentState::Active;
        let json = serde_json::to_string(&state).unwrap();
        assert_eq!(json, "\"active\"");
    }

    #[test]
    fn test_deployment_state_degraded() {
        let state = DeploymentState::Degraded;
        let json = serde_json::to_string(&state).unwrap();
        assert_eq!(json, "\"degraded\"");
    }

    #[test]
    fn test_deployment_state_disabled() {
        let state = DeploymentState::Disabled;
        let json = serde_json::to_string(&state).unwrap();
        assert_eq!(json, "\"disabled\"");
    }

    #[test]
    fn test_deployment_state_draining() {
        let state = DeploymentState::Draining;
        let json = serde_json::to_string(&state).unwrap();
        assert_eq!(json, "\"draining\"");
    }

    #[test]
    fn test_deployment_state_maintenance() {
        let state = DeploymentState::Maintenance;
        let json = serde_json::to_string(&state).unwrap();
        assert_eq!(json, "\"maintenance\"");
    }

    #[test]
    fn test_deployment_state_failed() {
        let state = DeploymentState::Failed;
        let json = serde_json::to_string(&state).unwrap();
        assert_eq!(json, "\"failed\"");
    }

    #[test]
    fn test_deployment_state_deserialize() {
        let state: DeploymentState = serde_json::from_str("\"active\"").unwrap();
        assert!(matches!(state, DeploymentState::Active));

        let state: DeploymentState = serde_json::from_str("\"failed\"").unwrap();
        assert!(matches!(state, DeploymentState::Failed));
    }

    #[test]
    fn test_deployment_state_clone() {
        let original = DeploymentState::Degraded;
        let cloned = original.clone();
        let json1 = serde_json::to_string(&original).unwrap();
        let json2 = serde_json::to_string(&cloned).unwrap();
        assert_eq!(json1, json2);
    }

    // ==================== DeploymentRateLimits Tests ====================

    #[test]
    fn test_deployment_rate_limits_full() {
        let limits = DeploymentRateLimits {
            rpm: Some(100),
            tpm: Some(10000),
            rpd: Some(1000),
            tpd: Some(100000),
            concurrent: Some(10),
            burst: Some(20),
        };

        assert_eq!(limits.rpm, Some(100));
        assert_eq!(limits.tpm, Some(10000));
        assert_eq!(limits.burst, Some(20));
    }

    #[test]
    fn test_deployment_rate_limits_partial() {
        let limits = DeploymentRateLimits {
            rpm: Some(50),
            tpm: None,
            rpd: None,
            tpd: None,
            concurrent: Some(5),
            burst: None,
        };

        assert_eq!(limits.rpm, Some(50));
        assert!(limits.tpm.is_none());
        assert!(limits.burst.is_none());
    }

    #[test]
    fn test_deployment_rate_limits_serialize() {
        let limits = DeploymentRateLimits {
            rpm: Some(200),
            tpm: Some(50000),
            rpd: None,
            tpd: None,
            concurrent: Some(20),
            burst: Some(50),
        };

        let json = serde_json::to_string(&limits).unwrap();
        assert!(json.contains("\"rpm\":200"));
        assert!(json.contains("\"tpm\":50000"));
        assert!(json.contains("\"burst\":50"));
    }

    #[test]
    fn test_deployment_rate_limits_deserialize() {
        let json = r#"{"rpm":100,"tpm":5000,"concurrent":10}"#;
        let limits: DeploymentRateLimits = serde_json::from_str(json).unwrap();
        assert_eq!(limits.rpm, Some(100));
        assert_eq!(limits.concurrent, Some(10));
    }

    #[test]
    fn test_deployment_rate_limits_clone() {
        let original = DeploymentRateLimits {
            rpm: Some(100),
            tpm: Some(10000),
            rpd: None,
            tpd: None,
            concurrent: None,
            burst: None,
        };

        let cloned = original.clone();
        assert_eq!(original.rpm, cloned.rpm);
        assert_eq!(original.tpm, cloned.tpm);
    }

    // ==================== BillingModel Tests ====================

    #[test]
    fn test_billing_model_pay_per_use() {
        let model = BillingModel::PayPerUse;
        let json = serde_json::to_string(&model).unwrap();
        assert_eq!(json, "\"pay_per_use\"");
    }

    #[test]
    fn test_billing_model_subscription() {
        let model = BillingModel::Subscription;
        let json = serde_json::to_string(&model).unwrap();
        assert_eq!(json, "\"subscription\"");
    }

    #[test]
    fn test_billing_model_prepaid() {
        let model = BillingModel::Prepaid;
        let json = serde_json::to_string(&model).unwrap();
        assert_eq!(json, "\"prepaid\"");
    }

    #[test]
    fn test_billing_model_free() {
        let model = BillingModel::Free;
        let json = serde_json::to_string(&model).unwrap();
        assert_eq!(json, "\"free\"");
    }

    #[test]
    fn test_billing_model_deserialize() {
        let model: BillingModel = serde_json::from_str("\"pay_per_use\"").unwrap();
        assert!(matches!(model, BillingModel::PayPerUse));

        let model: BillingModel = serde_json::from_str("\"subscription\"").unwrap();
        assert!(matches!(model, BillingModel::Subscription));
    }

    // ==================== DeploymentCostConfig Tests ====================

    #[test]
    fn test_deployment_cost_config_full() {
        let config = DeploymentCostConfig {
            input_cost_per_token: Some(0.00001),
            output_cost_per_token: Some(0.00003),
            cost_per_request: Some(0.001),
            cost_per_image: Some(0.02),
            cost_per_audio_second: Some(0.006),
            currency: "USD".to_string(),
            billing_model: BillingModel::PayPerUse,
        };

        assert_eq!(config.input_cost_per_token, Some(0.00001));
        assert_eq!(config.output_cost_per_token, Some(0.00003));
        assert_eq!(config.currency, "USD");
    }

    #[test]
    fn test_deployment_cost_config_minimal() {
        let config = DeploymentCostConfig {
            input_cost_per_token: None,
            output_cost_per_token: None,
            cost_per_request: None,
            cost_per_image: None,
            cost_per_audio_second: None,
            currency: "EUR".to_string(),
            billing_model: BillingModel::Free,
        };

        assert!(config.input_cost_per_token.is_none());
        assert_eq!(config.currency, "EUR");
    }

    #[test]
    fn test_deployment_cost_config_serialize() {
        let config = DeploymentCostConfig {
            input_cost_per_token: Some(0.00002),
            output_cost_per_token: Some(0.00006),
            cost_per_request: None,
            cost_per_image: None,
            cost_per_audio_second: None,
            currency: "USD".to_string(),
            billing_model: BillingModel::PayPerUse,
        };

        let json = serde_json::to_string(&config).unwrap();
        assert!(json.contains("USD"));
        assert!(json.contains("pay_per_use"));
    }

    #[test]
    fn test_deployment_cost_config_clone() {
        let original = DeploymentCostConfig {
            input_cost_per_token: Some(0.00001),
            output_cost_per_token: Some(0.00002),
            cost_per_request: None,
            cost_per_image: None,
            cost_per_audio_second: None,
            currency: "USD".to_string(),
            billing_model: BillingModel::Subscription,
        };

        let cloned = original.clone();
        assert_eq!(original.currency, cloned.currency);
    }

    // ==================== DeploymentSnapshot Tests ====================

    #[test]
    fn test_deployment_snapshot_creation() {
        let snapshot = DeploymentSnapshot {
            id: Uuid::new_v4(),
            name: "openai-prod".to_string(),
            provider_type: "openai".to_string(),
            model: "gpt-4".to_string(),
            state: DeploymentState::Active,
            health_status: HealthStatus::Healthy,
            weight: 1.0,
            tags: vec!["production".to_string(), "primary".to_string()],
            metrics: DeploymentMetricsSnapshot {
                total_requests: 1000,
                successful_requests: 990,
                failed_requests: 10,
                success_rate: 99.0,
                total_tokens: 500000,
                total_cost: 50.0,
                active_connections: 5,
                avg_response_time: 200,
                p95_response_time: 500,
                p99_response_time: 800,
                request_rate: 100,
                token_rate: 50000,
            },
            updated_at: chrono::Utc::now(),
        };

        assert_eq!(snapshot.name, "openai-prod");
        assert_eq!(snapshot.model, "gpt-4");
        assert!(matches!(snapshot.state, DeploymentState::Active));
        assert_eq!(snapshot.tags.len(), 2);
    }

    #[test]
    fn test_deployment_snapshot_serialize() {
        let snapshot = DeploymentSnapshot {
            id: Uuid::new_v4(),
            name: "test-deployment".to_string(),
            provider_type: "anthropic".to_string(),
            model: "claude-3-opus".to_string(),
            state: DeploymentState::Degraded,
            health_status: HealthStatus::Degraded,
            weight: 0.5,
            tags: vec![],
            metrics: DeploymentMetricsSnapshot {
                total_requests: 100,
                successful_requests: 95,
                failed_requests: 5,
                success_rate: 95.0,
                total_tokens: 10000,
                total_cost: 5.0,
                active_connections: 2,
                avg_response_time: 300,
                p95_response_time: 600,
                p99_response_time: 900,
                request_rate: 10,
                token_rate: 1000,
            },
            updated_at: chrono::Utc::now(),
        };

        let json = serde_json::to_string(&snapshot).unwrap();
        assert!(json.contains("test-deployment"));
        assert!(json.contains("claude-3-opus"));
        assert!(json.contains("degraded"));
    }

    #[test]
    fn test_deployment_snapshot_clone() {
        let snapshot = DeploymentSnapshot {
            id: Uuid::new_v4(),
            name: "clone-test".to_string(),
            provider_type: "openai".to_string(),
            model: "gpt-3.5-turbo".to_string(),
            state: DeploymentState::Active,
            health_status: HealthStatus::Healthy,
            weight: 1.0,
            tags: vec!["test".to_string()],
            metrics: DeploymentMetricsSnapshot {
                total_requests: 50,
                successful_requests: 50,
                failed_requests: 0,
                success_rate: 100.0,
                total_tokens: 5000,
                total_cost: 1.0,
                active_connections: 1,
                avg_response_time: 100,
                p95_response_time: 200,
                p99_response_time: 300,
                request_rate: 5,
                token_rate: 500,
            },
            updated_at: chrono::Utc::now(),
        };

        let cloned = snapshot.clone();
        assert_eq!(snapshot.name, cloned.name);
        assert_eq!(snapshot.id, cloned.id);
    }

    // ==================== DeploymentMetricsSnapshot Tests ====================

    #[test]
    fn test_deployment_metrics_snapshot_creation() {
        let metrics = DeploymentMetricsSnapshot {
            total_requests: 10000,
            successful_requests: 9900,
            failed_requests: 100,
            success_rate: 99.0,
            total_tokens: 5000000,
            total_cost: 500.0,
            active_connections: 50,
            avg_response_time: 150,
            p95_response_time: 400,
            p99_response_time: 700,
            request_rate: 200,
            token_rate: 100000,
        };

        assert_eq!(metrics.total_requests, 10000);
        assert_eq!(metrics.successful_requests, 9900);
        assert!((metrics.success_rate - 99.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_deployment_metrics_snapshot_zero_values() {
        let metrics = DeploymentMetricsSnapshot {
            total_requests: 0,
            successful_requests: 0,
            failed_requests: 0,
            success_rate: 0.0,
            total_tokens: 0,
            total_cost: 0.0,
            active_connections: 0,
            avg_response_time: 0,
            p95_response_time: 0,
            p99_response_time: 0,
            request_rate: 0,
            token_rate: 0,
        };

        assert_eq!(metrics.total_requests, 0);
        assert!((metrics.success_rate - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_deployment_metrics_snapshot_serialize() {
        let metrics = DeploymentMetricsSnapshot {
            total_requests: 500,
            successful_requests: 480,
            failed_requests: 20,
            success_rate: 96.0,
            total_tokens: 100000,
            total_cost: 10.0,
            active_connections: 10,
            avg_response_time: 200,
            p95_response_time: 500,
            p99_response_time: 800,
            request_rate: 50,
            token_rate: 10000,
        };

        let json = serde_json::to_string(&metrics).unwrap();
        assert!(json.contains("\"total_requests\":500"));
        assert!(json.contains("\"success_rate\":96.0"));
    }

    #[test]
    fn test_deployment_metrics_snapshot_deserialize() {
        let json = r#"{"total_requests":100,"successful_requests":95,"failed_requests":5,"success_rate":95.0,"total_tokens":10000,"total_cost":5.0,"active_connections":3,"avg_response_time":150,"p95_response_time":300,"p99_response_time":500,"request_rate":10,"token_rate":1000}"#;
        let metrics: DeploymentMetricsSnapshot = serde_json::from_str(json).unwrap();

        assert_eq!(metrics.total_requests, 100);
        assert_eq!(metrics.successful_requests, 95);
        assert!((metrics.success_rate - 95.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_deployment_metrics_snapshot_clone() {
        let original = DeploymentMetricsSnapshot {
            total_requests: 1000,
            successful_requests: 1000,
            failed_requests: 0,
            success_rate: 100.0,
            total_tokens: 50000,
            total_cost: 25.0,
            active_connections: 5,
            avg_response_time: 100,
            p95_response_time: 200,
            p99_response_time: 300,
            request_rate: 100,
            token_rate: 5000,
        };

        let cloned = original.clone();
        assert_eq!(original.total_requests, cloned.total_requests);
        assert_eq!(original.total_cost, cloned.total_cost);
    }

    #[test]
    fn test_deployment_metrics_snapshot_high_latency() {
        let metrics = DeploymentMetricsSnapshot {
            total_requests: 100,
            successful_requests: 50,
            failed_requests: 50,
            success_rate: 50.0,
            total_tokens: 10000,
            total_cost: 5.0,
            active_connections: 100,
            avg_response_time: 5000,
            p95_response_time: 10000,
            p99_response_time: 30000,
            request_rate: 1,
            token_rate: 100,
        };

        assert_eq!(metrics.avg_response_time, 5000);
        assert_eq!(metrics.p99_response_time, 30000);
    }
}
