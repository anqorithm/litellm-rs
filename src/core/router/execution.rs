//! Execution flow for router operations
//!
//! This module contains the execution logic for running operations
//! with retry and fallback support.

use super::config::RouterConfig;
use super::deployment::DeploymentId;
use super::error::{CooldownReason, RouterError};
use super::fallback::{ExecutionResult, FallbackType};
use crate::core::providers::unified_provider::ProviderError;
use std::time::Duration;

/// Check if an error is retryable
///
/// Determines whether a request should be retried based on the error type.
pub fn is_retryable_error(error: &ProviderError) -> bool {
    matches!(
        error,
        ProviderError::RateLimit { .. }
            | ProviderError::Timeout { .. }
            | ProviderError::ProviderUnavailable { .. }
            | ProviderError::Network { .. }
    )
}

/// Calculate retry delay using exponential backoff
///
/// Implements exponential backoff with a maximum delay cap.
/// The formula is: `base * 2^(attempt - 1)`, capped at 30 seconds.
pub fn calculate_retry_delay(config: &RouterConfig, attempt: u32) -> Duration {
    let base = config.retry_after_secs.max(1);
    let delay = base * (2_u64.pow(attempt.saturating_sub(1)));
    Duration::from_secs(delay.min(30)) // Cap at 30 seconds
}

/// Infer fallback type from a ProviderError
///
/// Analyzes the error to determine which type of fallback should be used.
pub fn infer_fallback_type(error: &ProviderError) -> FallbackType {
    match error {
        // Context length exceeded -> use context window fallback
        ProviderError::ContextLengthExceeded { .. } => FallbackType::ContextWindow,

        // Content filtered -> use content policy fallback
        ProviderError::ContentFiltered { .. } => FallbackType::ContentPolicy,

        // Rate limit -> use rate limit fallback
        ProviderError::RateLimit { .. } => FallbackType::RateLimit,

        // All other errors -> use general fallback
        _ => FallbackType::General,
    }
}

/// Infer cooldown reason from a ProviderError
///
/// Maps provider error types to cooldown reasons based on the error characteristics.
pub fn infer_cooldown_reason(error: &ProviderError) -> CooldownReason {
    match error {
        // Rate limit errors
        ProviderError::RateLimit { .. } => CooldownReason::RateLimit,

        // Authentication errors
        ProviderError::Authentication { .. } => CooldownReason::AuthError,

        // Model/deployment not found
        ProviderError::ModelNotFound { .. } | ProviderError::DeploymentError { .. } => {
            CooldownReason::NotFound
        }

        // Timeout errors
        ProviderError::Timeout { .. } => CooldownReason::Timeout,

        // API errors - map based on status code
        ProviderError::ApiError { status, .. } => match *status {
            401 => CooldownReason::AuthError,
            404 => CooldownReason::NotFound,
            408 => CooldownReason::Timeout,
            429 => CooldownReason::RateLimit,
            _ => CooldownReason::ConsecutiveFailures,
        },

        // All other errors are treated as consecutive failures
        _ => CooldownReason::ConsecutiveFailures,
    }
}

/// Convert RouterError to ProviderError for consistency
pub fn router_error_to_provider_error(err: RouterError) -> ProviderError {
    match err {
        RouterError::ModelNotFound(msg) => ProviderError::model_not_found("router", msg),
        RouterError::NoAvailableDeployment(msg) => ProviderError::ProviderUnavailable {
            provider: "router",
            message: format!("No available deployment: {}", msg),
        },
        RouterError::AllDeploymentsInCooldown(msg) => ProviderError::ProviderUnavailable {
            provider: "router",
            message: format!("All deployments in cooldown: {}", msg),
        },
        RouterError::DeploymentNotFound(msg) => ProviderError::DeploymentError {
            provider: "router",
            deployment: msg.clone(),
            message: "Deployment not found".to_string(),
        },
        RouterError::RateLimitExceeded(_msg) => ProviderError::rate_limit("router", Some(60)),
        RouterError::AliasCycle(msg) => ProviderError::Other {
            provider: "router",
            message: format!("Circular alias detected: {}", msg),
        },
        RouterError::FallbackCycle(msg) => ProviderError::Other {
            provider: "router",
            message: format!("Circular fallback chain detected: {}", msg),
        },
    }
}

/// Convert final ProviderError back to RouterError
pub fn provider_error_to_router_error(err: ProviderError, model_name: &str) -> RouterError {
    match err {
        ProviderError::ModelNotFound { model, .. } => RouterError::ModelNotFound(model),
        ProviderError::RateLimit { .. } => RouterError::RateLimitExceeded(model_name.to_string()),
        _ => RouterError::NoAvailableDeployment(format!("{}: {}", model_name, err)),
    }
}

/// Build execution result from successful execution
pub fn build_execution_result<T>(
    result: T,
    deployment_id: DeploymentId,
    attempts: u32,
    model_used: String,
    used_fallback: bool,
    latency_us: u64,
) -> ExecutionResult<T> {
    ExecutionResult {
        result,
        deployment_id,
        attempts,
        model_used,
        used_fallback,
        latency_us,
    }
}
