//! Observability types and data structures

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Metric value types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MetricValue {
    Counter(u64),
    Gauge(f64),
    Histogram(Vec<f64>),
    Summary { sum: f64, count: u64 },
}

/// Structured log entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    /// Timestamp
    pub timestamp: DateTime<Utc>,
    /// Log level
    pub level: LogLevel,
    /// Message
    pub message: String,
    /// Request ID
    pub request_id: Option<String>,
    /// User ID
    pub user_id: Option<String>,
    /// Provider
    pub provider: Option<String>,
    /// Model
    pub model: Option<String>,
    /// Duration in milliseconds
    pub duration_ms: Option<u64>,
    /// Token usage
    pub tokens: Option<TokenUsage>,
    /// Cost
    pub cost: Option<f64>,
    /// Error details
    pub error: Option<ErrorDetails>,
    /// Additional fields
    pub fields: HashMap<String, serde_json::Value>,
}

/// Log levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LogLevel {
    Error,
    Warn,
    Info,
    Debug,
    Trace,
}

/// Token usage information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenUsage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

/// Error details for logging
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorDetails {
    pub error_type: String,
    pub error_message: String,
    pub error_code: Option<String>,
    pub stack_trace: Option<String>,
}

/// Alert conditions
#[derive(Debug, Clone)]
pub enum AlertCondition {
    GreaterThan,
    LessThan,
    Equal,
    NotEqual,
    GreaterThanOrEqual,
    LessThanOrEqual,
}

/// Alert severity levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlertSeverity {
    Critical,
    High,
    Medium,
    Low,
    Info,
}

/// Alert state tracking
#[derive(Debug, Clone)]
pub struct AlertState {
    /// Whether alert is currently firing
    pub firing: bool,
    /// When alert started firing
    pub fired_at: Option<DateTime<Utc>>,
    /// Last notification sent
    pub last_notification: Option<DateTime<Utc>>,
    /// Notification count
    pub notification_count: u32,
}

/// Trace span
#[derive(Debug, Clone)]
pub struct TraceSpan {
    /// Span ID
    pub span_id: String,
    /// Parent span ID
    pub parent_id: Option<String>,
    /// Trace ID
    pub trace_id: String,
    /// Operation name
    pub operation: String,
    /// Start time
    pub start_time: std::time::Instant,
    /// End time
    pub end_time: Option<std::time::Instant>,
    /// Tags
    pub tags: HashMap<String, String>,
    /// Logs
    pub logs: Vec<SpanLog>,
}

/// Span log entry
#[derive(Debug, Clone)]
pub struct SpanLog {
    pub timestamp: std::time::Instant,
    pub message: String,
    pub fields: HashMap<String, String>,
}


#[cfg(test)]
#[path = "types_tests.rs"]
mod tests;
