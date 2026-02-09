//! Structured logging utilities
//!
//! This module provides structured logging capabilities with consistent
//! formatting and contextual information for better observability.

use crate::utils::data::types::{ApiKey, ModelName, RequestId, UserId};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Duration, Instant};
use tracing::{debug, error, info, warn};
use uuid::Uuid;

/// Log context for structured logging
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogContext {
    /// Request ID for tracing
    pub request_id: Option<RequestId>,
    /// User ID if available
    pub user_id: Option<UserId>,
    /// Organization ID if available
    pub organization_id: Option<Uuid>,
    /// API key (truncated for security)
    pub api_key: Option<String>,
    /// Model being used
    pub model: Option<ModelName>,
    /// Provider being used
    pub provider: Option<String>,
    /// Additional custom fields
    pub extra: HashMap<String, serde_json::Value>,
}

impl LogContext {
    /// Create a new log context
    pub fn new() -> Self {
        Self {
            request_id: None,
            user_id: None,
            organization_id: None,
            api_key: None,
            model: None,
            provider: None,
            extra: HashMap::new(),
        }
    }

    /// Set request ID
    pub fn with_request_id(mut self, request_id: RequestId) -> Self {
        self.request_id = Some(request_id);
        self
    }

    /// Set user ID
    pub fn with_user_id(mut self, user_id: UserId) -> Self {
        self.user_id = Some(user_id);
        self
    }

    /// Set organization ID
    pub fn with_organization_id(mut self, org_id: Uuid) -> Self {
        self.organization_id = Some(org_id);
        self
    }

    /// Set API key (will be truncated for security)
    pub fn with_api_key(mut self, api_key: &ApiKey) -> Self {
        self.api_key = Some(api_key.as_display_str());
        self
    }

    /// Set model
    pub fn with_model(mut self, model: ModelName) -> Self {
        self.model = Some(model);
        self
    }

    /// Set provider
    pub fn with_provider(mut self, provider: String) -> Self {
        self.provider = Some(provider);
        self
    }

    /// Add custom field
    pub fn with_field<T: Serialize>(mut self, key: &str, value: T) -> Self {
        if let Ok(json_value) = serde_json::to_value(value) {
            self.extra.insert(key.to_string(), json_value);
        }
        self
    }

    /// Get context fields as a formatted string for logging
    pub fn context_fields(&self) -> String {
        let mut fields = Vec::new();

        if let Some(request_id) = &self.request_id {
            fields.push(format!("request_id={}", request_id.as_str()));
        }
        if let Some(user_id) = &self.user_id {
            fields.push(format!("user_id={}", user_id));
        }
        if let Some(model) = &self.model {
            fields.push(format!("model={}", model.as_str()));
        }
        if let Some(provider) = &self.provider {
            fields.push(format!("provider={}", provider));
        }

        fields.join(", ")
    }
}

impl Default for LogContext {
    fn default() -> Self {
        Self::new()
    }
}

/// Performance metrics for logging
#[derive(Debug, Clone, Serialize)]
pub struct PerformanceMetrics {
    /// Duration of the operation
    pub duration_ms: u64,
    /// Memory usage in bytes
    pub memory_bytes: Option<u64>,
    /// Number of database queries
    pub db_queries: Option<u32>,
    /// Cache hit/miss information
    pub cache_hits: Option<u32>,
    /// Cache misses count
    /// Number of cache misses during the operation
    pub cache_misses: Option<u32>,
    /// Token usage
    pub tokens_used: Option<u32>,
    /// Cost in USD
    pub cost_usd: Option<f64>,
}

impl PerformanceMetrics {
    /// Create new performance metrics
    pub fn new(duration: Duration) -> Self {
        Self {
            duration_ms: duration.as_millis() as u64,
            memory_bytes: None,
            db_queries: None,
            cache_hits: None,
            cache_misses: None,
            tokens_used: None,
            cost_usd: None,
        }
    }

    /// Set memory usage
    pub fn with_memory(mut self, bytes: u64) -> Self {
        self.memory_bytes = Some(bytes);
        self
    }

    /// Set database query count
    pub fn with_db_queries(mut self, count: u32) -> Self {
        self.db_queries = Some(count);
        self
    }

    /// Set cache statistics
    pub fn with_cache_stats(mut self, hits: u32, misses: u32) -> Self {
        self.cache_hits = Some(hits);
        self.cache_misses = Some(misses);
        self
    }

    /// Set token usage
    pub fn with_tokens(mut self, tokens: u32) -> Self {
        self.tokens_used = Some(tokens);
        self
    }

    /// Set cost
    pub fn with_cost(mut self, cost: f64) -> Self {
        self.cost_usd = Some(cost);
        self
    }
}

/// Structured logger for consistent logging
pub struct StructuredLogger {
    context: LogContext,
}

impl StructuredLogger {
    /// Create a new structured logger
    pub fn new(context: LogContext) -> Self {
        Self { context }
    }

    /// Log an info message
    pub fn info(&self, message: &str) {
        let context_str = self.context.context_fields();
        info!("{} | {}", message, context_str);
    }

    /// Log a warning message
    pub fn warn(&self, message: &str) {
        let context_str = self.context.context_fields();
        warn!("{} | {}", message, context_str);
    }

    /// Log an error message
    pub fn error(&self, message: &str, error: Option<&dyn std::error::Error>) {
        let context_str = self.context.context_fields();
        if let Some(err) = error {
            error!("{} | {} | error={}", message, context_str, err);
        } else {
            error!("{} | {}", message, context_str);
        }
    }

    /// Log a debug message
    pub fn debug(&self, message: &str) {
        let context_str = self.context.context_fields();
        debug!("{} | {}", message, context_str);
    }

    /// Log performance metrics
    pub fn performance(&self, operation: &str, metrics: PerformanceMetrics) {
        let context_str = self.context.context_fields();
        info!(
            "Performance metrics: operation={}, metrics={:?} | {}",
            operation, metrics, context_str
        );
    }

    /// Log an API request
    pub fn api_request(&self, method: &str, path: &str, status_code: u16, duration: Duration) {
        let context_str = self.context.context_fields();
        info!(
            "API request completed: method={}, path={}, status_code={}, duration_ms={} | {}",
            method,
            path,
            status_code,
            duration.as_millis(),
            context_str
        );
    }

    /// Log a database operation
    pub fn database_operation(
        &self,
        operation: &str,
        table: &str,
        duration: Duration,
        rows_affected: Option<u64>,
    ) {
        let context_str = self.context.context_fields();
        debug!(
            "Database operation completed: operation={}, table={}, duration_ms={}, rows_affected={:?} | {}",
            operation,
            table,
            duration.as_millis(),
            rows_affected,
            context_str
        );
    }

    /// Log a cache operation
    pub fn cache_operation(&self, operation: &str, key: &str, hit: bool, duration: Duration) {
        let context_str = self.context.context_fields();
        debug!(
            "Cache operation completed: operation={}, key={}, hit={}, duration_ms={} | {}",
            operation,
            key,
            hit,
            duration.as_millis(),
            context_str
        );
    }

    /// Log provider interaction
    pub fn provider_interaction(
        &self,
        provider: &str,
        model: &str,
        tokens: Option<u32>,
        cost: Option<f64>,
        duration: Duration,
    ) {
        let context_str = self.context.context_fields();
        info!(
            "Provider interaction completed: provider={}, model={}, tokens={:?}, cost_usd={:?}, duration_ms={} | {}",
            provider,
            model,
            tokens,
            cost,
            duration.as_millis(),
            context_str
        );
    }
}

/// Timer for measuring operation duration
pub struct Timer {
    start: Instant,
    operation: String,
    logger: StructuredLogger,
}

impl Timer {
    /// Start a new timer
    pub fn start(operation: String, logger: StructuredLogger) -> Self {
        Self {
            start: Instant::now(),
            operation,
            logger,
        }
    }

    /// Stop the timer and log the duration
    pub fn stop(self) {
        let duration = self.start.elapsed();
        self.logger
            .performance(&self.operation, PerformanceMetrics::new(duration));
    }

    /// Stop the timer with additional metrics
    pub fn stop_with_metrics(self, metrics: PerformanceMetrics) {
        self.logger.performance(&self.operation, metrics);
    }
}


#[cfg(test)]
#[path = "structured_tests.rs"]
mod tests;
