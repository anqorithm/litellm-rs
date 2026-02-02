//! Audit events

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

use super::types::{LogLevel, RequestLog, ResponseLog, UserAction};

/// Type of audit event
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EventType {
    /// Request started
    RequestStarted,
    /// Request completed
    RequestCompleted,
    /// Request failed
    RequestFailed,
    /// User action
    UserAction,
    /// System event
    System,
    /// Security event
    Security,
    /// Error event
    Error,
}

/// An audit event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEvent {
    /// Unique event ID
    pub id: String,
    /// Event type
    pub event_type: EventType,
    /// Log level
    pub level: LogLevel,
    /// Timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// Request ID (if applicable)
    #[serde(default)]
    pub request_id: Option<String>,
    /// User ID (if applicable)
    #[serde(default)]
    pub user_id: Option<String>,
    /// API key ID (if applicable)
    #[serde(default)]
    pub api_key_id: Option<String>,
    /// Team ID (if applicable)
    #[serde(default)]
    pub team_id: Option<String>,
    /// Event message
    pub message: String,
    /// Request log (if applicable)
    #[serde(default)]
    pub request: Option<RequestLog>,
    /// Response log (if applicable)
    #[serde(default)]
    pub response: Option<ResponseLog>,
    /// User action (if applicable)
    #[serde(default)]
    pub action: Option<UserAction>,
    /// Additional metadata
    #[serde(default)]
    pub metadata: HashMap<String, serde_json::Value>,
    /// Source of the event
    #[serde(default)]
    pub source: Option<String>,
}

impl AuditEvent {
    /// Create a new audit event
    pub fn new(event_type: EventType, message: impl Into<String>) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            event_type,
            level: LogLevel::Info,
            timestamp: chrono::Utc::now(),
            request_id: None,
            user_id: None,
            api_key_id: None,
            team_id: None,
            message: message.into(),
            request: None,
            response: None,
            action: None,
            metadata: HashMap::new(),
            source: None,
        }
    }

    /// Create a request started event
    pub fn request_started(request_id: impl Into<String>, path: impl Into<String>) -> Self {
        let request_id = request_id.into();
        Self::new(
            EventType::RequestStarted,
            format!("Request started: {}", path.into()),
        )
        .with_request_id(&request_id)
    }

    /// Create a request completed event
    pub fn request_completed(
        request_id: impl Into<String>,
        status_code: u16,
        duration_ms: u64,
    ) -> Self {
        let request_id = request_id.into();
        Self::new(
            EventType::RequestCompleted,
            format!(
                "Request completed: status={}, duration={}ms",
                status_code, duration_ms
            ),
        )
        .with_request_id(&request_id)
        .with_metadata("status_code", serde_json::json!(status_code))
        .with_metadata("duration_ms", serde_json::json!(duration_ms))
    }

    /// Create a request failed event
    pub fn request_failed(request_id: impl Into<String>, error: impl Into<String>) -> Self {
        let request_id = request_id.into();
        Self::new(EventType::RequestFailed, format!("Request failed: {}", error.into()))
            .with_request_id(&request_id)
            .with_level(LogLevel::Error)
    }

    /// Create a user action event
    pub fn user_action(user_id: impl Into<String>, action: UserAction) -> Self {
        let user_id = user_id.into();
        Self::new(
            EventType::UserAction,
            format!("User action: {}", action.as_str()),
        )
        .with_user_id(&user_id)
        .with_action(action)
    }

    /// Create a security event
    pub fn security(message: impl Into<String>) -> Self {
        Self::new(EventType::Security, message).with_level(LogLevel::Warn)
    }

    /// Create a system event
    pub fn system(message: impl Into<String>) -> Self {
        Self::new(EventType::System, message)
    }

    /// Create an error event
    pub fn error(message: impl Into<String>) -> Self {
        Self::new(EventType::Error, message).with_level(LogLevel::Error)
    }

    /// Set log level
    pub fn with_level(mut self, level: LogLevel) -> Self {
        self.level = level;
        self
    }

    /// Set request ID
    pub fn with_request_id(mut self, request_id: &str) -> Self {
        self.request_id = Some(request_id.to_string());
        self
    }

    /// Set user ID
    pub fn with_user_id(mut self, user_id: &str) -> Self {
        self.user_id = Some(user_id.to_string());
        self
    }

    /// Set API key ID
    pub fn with_api_key_id(mut self, api_key_id: &str) -> Self {
        self.api_key_id = Some(api_key_id.to_string());
        self
    }

    /// Set team ID
    pub fn with_team_id(mut self, team_id: &str) -> Self {
        self.team_id = Some(team_id.to_string());
        self
    }

    /// Set request log
    pub fn with_request(mut self, request: RequestLog) -> Self {
        self.request = Some(request);
        self
    }

    /// Set response log
    pub fn with_response(mut self, response: ResponseLog) -> Self {
        self.response = Some(response);
        self
    }

    /// Set user action
    pub fn with_action(mut self, action: UserAction) -> Self {
        self.action = Some(action);
        self
    }

    /// Add metadata
    pub fn with_metadata(mut self, key: impl Into<String>, value: serde_json::Value) -> Self {
        self.metadata.insert(key.into(), value);
        self
    }

    /// Set source
    pub fn with_source(mut self, source: impl Into<String>) -> Self {
        self.source = Some(source.into());
        self
    }

    /// Convert to JSON string
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }

    /// Convert to pretty JSON string
    pub fn to_json_pretty(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_creation() {
        let event = AuditEvent::new(EventType::System, "Test event");
        assert_eq!(event.event_type, EventType::System);
        assert_eq!(event.message, "Test event");
        assert_eq!(event.level, LogLevel::Info);
    }

    #[test]
    fn test_request_started() {
        let event = AuditEvent::request_started("req-123", "/v1/chat/completions");
        assert_eq!(event.event_type, EventType::RequestStarted);
        assert_eq!(event.request_id, Some("req-123".to_string()));
    }

    #[test]
    fn test_request_completed() {
        let event = AuditEvent::request_completed("req-123", 200, 150);
        assert_eq!(event.event_type, EventType::RequestCompleted);
        assert_eq!(event.metadata.get("status_code"), Some(&serde_json::json!(200)));
        assert_eq!(event.metadata.get("duration_ms"), Some(&serde_json::json!(150)));
    }

    #[test]
    fn test_request_failed() {
        let event = AuditEvent::request_failed("req-123", "Connection timeout");
        assert_eq!(event.event_type, EventType::RequestFailed);
        assert_eq!(event.level, LogLevel::Error);
    }

    #[test]
    fn test_user_action() {
        let event = AuditEvent::user_action("user-123", UserAction::Login);
        assert_eq!(event.event_type, EventType::UserAction);
        assert_eq!(event.user_id, Some("user-123".to_string()));
        assert_eq!(event.action, Some(UserAction::Login));
    }

    #[test]
    fn test_security_event() {
        let event = AuditEvent::security("Suspicious activity detected");
        assert_eq!(event.event_type, EventType::Security);
        assert_eq!(event.level, LogLevel::Warn);
    }

    #[test]
    fn test_event_builder() {
        let event = AuditEvent::new(EventType::System, "Test")
            .with_level(LogLevel::Debug)
            .with_request_id("req-123")
            .with_user_id("user-456")
            .with_api_key_id("key-789")
            .with_team_id("team-abc")
            .with_source("test-service")
            .with_metadata("custom", serde_json::json!("value"));

        assert_eq!(event.level, LogLevel::Debug);
        assert_eq!(event.request_id, Some("req-123".to_string()));
        assert_eq!(event.user_id, Some("user-456".to_string()));
        assert_eq!(event.api_key_id, Some("key-789".to_string()));
        assert_eq!(event.team_id, Some("team-abc".to_string()));
        assert_eq!(event.source, Some("test-service".to_string()));
        assert!(event.metadata.contains_key("custom"));
    }

    #[test]
    fn test_event_serialization() {
        let event = AuditEvent::request_started("req-123", "/test");
        let json = event.to_json().unwrap();

        assert!(json.contains("req-123"));
        assert!(json.contains("request_started"));

        let deserialized: AuditEvent = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.request_id, event.request_id);
    }

    #[test]
    fn test_event_with_request_log() {
        let request = RequestLog::new("req-123", "POST", "/v1/chat/completions");
        let event = AuditEvent::request_started("req-123", "/v1/chat/completions")
            .with_request(request);

        assert!(event.request.is_some());
        assert_eq!(event.request.unwrap().method, "POST");
    }

    #[test]
    fn test_event_with_response_log() {
        let response = ResponseLog::new("req-123", 200, 100);
        let event = AuditEvent::request_completed("req-123", 200, 100)
            .with_response(response);

        assert!(event.response.is_some());
        assert_eq!(event.response.unwrap().status_code, 200);
    }
}
