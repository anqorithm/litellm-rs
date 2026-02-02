//! Core types for the Audit Logging system

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

/// Error types for audit operations
#[derive(Debug, Error)]
pub enum AuditError {
    /// Configuration error
    #[error("Configuration error: {0}")]
    Config(String),

    /// IO error
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// Serialization error
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    /// Channel error
    #[error("Channel error: {0}")]
    Channel(String),

    /// Output error
    #[error("Output error: {0}")]
    Output(String),
}

/// Result type for audit operations
pub type AuditResult<T> = Result<T, AuditError>;

/// Log level for audit events
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum LogLevel {
    /// Debug level - verbose logging
    Debug,
    /// Info level - standard logging
    #[default]
    Info,
    /// Warning level - potential issues
    Warn,
    /// Error level - errors and failures
    Error,
    /// Critical level - critical failures
    Critical,
}

impl LogLevel {
    /// Check if this level should be logged given a minimum level
    pub fn should_log(&self, min_level: LogLevel) -> bool {
        self.priority() >= min_level.priority()
    }

    /// Get numeric priority (higher = more important)
    fn priority(&self) -> u8 {
        match self {
            LogLevel::Debug => 0,
            LogLevel::Info => 1,
            LogLevel::Warn => 2,
            LogLevel::Error => 3,
            LogLevel::Critical => 4,
        }
    }
}

/// Request log entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestLog {
    /// Unique request ID
    pub request_id: String,
    /// HTTP method
    pub method: String,
    /// Request path
    pub path: String,
    /// Query parameters
    #[serde(default)]
    pub query_params: HashMap<String, String>,
    /// Request headers (filtered)
    #[serde(default)]
    pub headers: HashMap<String, String>,
    /// Request body (may be truncated or redacted)
    #[serde(default)]
    pub body: Option<String>,
    /// Body size in bytes
    pub body_size: usize,
    /// Client IP address
    #[serde(default)]
    pub client_ip: Option<String>,
    /// User agent
    #[serde(default)]
    pub user_agent: Option<String>,
    /// Timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl RequestLog {
    /// Create a new request log
    pub fn new(request_id: impl Into<String>, method: impl Into<String>, path: impl Into<String>) -> Self {
        Self {
            request_id: request_id.into(),
            method: method.into(),
            path: path.into(),
            query_params: HashMap::new(),
            headers: HashMap::new(),
            body: None,
            body_size: 0,
            client_ip: None,
            user_agent: None,
            timestamp: chrono::Utc::now(),
        }
    }

    /// Set client IP
    pub fn with_client_ip(mut self, ip: impl Into<String>) -> Self {
        self.client_ip = Some(ip.into());
        self
    }

    /// Set user agent
    pub fn with_user_agent(mut self, ua: impl Into<String>) -> Self {
        self.user_agent = Some(ua.into());
        self
    }

    /// Set body
    pub fn with_body(mut self, body: impl Into<String>, size: usize) -> Self {
        self.body = Some(body.into());
        self.body_size = size;
        self
    }

    /// Add header
    pub fn with_header(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.headers.insert(key.into(), value.into());
        self
    }
}

/// Response log entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseLog {
    /// Request ID this response belongs to
    pub request_id: String,
    /// HTTP status code
    pub status_code: u16,
    /// Response headers (filtered)
    #[serde(default)]
    pub headers: HashMap<String, String>,
    /// Response body (may be truncated or redacted)
    #[serde(default)]
    pub body: Option<String>,
    /// Body size in bytes
    pub body_size: usize,
    /// Duration in milliseconds
    pub duration_ms: u64,
    /// Timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl ResponseLog {
    /// Create a new response log
    pub fn new(request_id: impl Into<String>, status_code: u16, duration_ms: u64) -> Self {
        Self {
            request_id: request_id.into(),
            status_code,
            headers: HashMap::new(),
            body: None,
            body_size: 0,
            duration_ms,
            timestamp: chrono::Utc::now(),
        }
    }

    /// Set body
    pub fn with_body(mut self, body: impl Into<String>, size: usize) -> Self {
        self.body = Some(body.into());
        self.body_size = size;
        self
    }

    /// Add header
    pub fn with_header(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.headers.insert(key.into(), value.into());
        self
    }
}

/// User action types
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UserAction {
    /// User logged in
    Login,
    /// User logged out
    Logout,
    /// User authentication failed
    AuthFailed,
    /// API key created
    ApiKeyCreated,
    /// API key revoked
    ApiKeyRevoked,
    /// Settings changed
    SettingsChanged,
    /// Team member added
    TeamMemberAdded,
    /// Team member removed
    TeamMemberRemoved,
    /// Budget updated
    BudgetUpdated,
    /// Model accessed
    ModelAccessed,
    /// Custom action
    Custom(String),
}

impl UserAction {
    /// Get action name as string
    pub fn as_str(&self) -> &str {
        match self {
            UserAction::Login => "login",
            UserAction::Logout => "logout",
            UserAction::AuthFailed => "auth_failed",
            UserAction::ApiKeyCreated => "api_key_created",
            UserAction::ApiKeyRevoked => "api_key_revoked",
            UserAction::SettingsChanged => "settings_changed",
            UserAction::TeamMemberAdded => "team_member_added",
            UserAction::TeamMemberRemoved => "team_member_removed",
            UserAction::BudgetUpdated => "budget_updated",
            UserAction::ModelAccessed => "model_accessed",
            UserAction::Custom(s) => s,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_log_level_priority() {
        assert!(LogLevel::Error.should_log(LogLevel::Info));
        assert!(LogLevel::Info.should_log(LogLevel::Info));
        assert!(!LogLevel::Debug.should_log(LogLevel::Info));
        assert!(LogLevel::Critical.should_log(LogLevel::Error));
    }

    #[test]
    fn test_request_log() {
        let log = RequestLog::new("req-123", "POST", "/v1/chat/completions")
            .with_client_ip("192.168.1.1")
            .with_user_agent("test-agent")
            .with_body("{}", 2)
            .with_header("Content-Type", "application/json");

        assert_eq!(log.request_id, "req-123");
        assert_eq!(log.method, "POST");
        assert_eq!(log.path, "/v1/chat/completions");
        assert_eq!(log.client_ip, Some("192.168.1.1".to_string()));
        assert_eq!(log.body_size, 2);
    }

    #[test]
    fn test_response_log() {
        let log = ResponseLog::new("req-123", 200, 150)
            .with_body("{\"result\": \"ok\"}", 16)
            .with_header("Content-Type", "application/json");

        assert_eq!(log.request_id, "req-123");
        assert_eq!(log.status_code, 200);
        assert_eq!(log.duration_ms, 150);
        assert_eq!(log.body_size, 16);
    }

    #[test]
    fn test_user_action() {
        assert_eq!(UserAction::Login.as_str(), "login");
        assert_eq!(UserAction::Custom("test".to_string()).as_str(), "test");
    }

    #[test]
    fn test_serialization() {
        let log = RequestLog::new("req-123", "GET", "/health");
        let json = serde_json::to_string(&log).unwrap();
        assert!(json.contains("req-123"));

        let response = ResponseLog::new("req-123", 200, 50);
        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("200"));
    }
}
