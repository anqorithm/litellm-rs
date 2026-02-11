//! Core types for logging system

use serde::Serialize;
use std::collections::HashMap;
use uuid::Uuid;

/// Log entry for async processing
#[derive(Debug, Clone, Serialize)]
pub struct AsyncLogRecord {
    /// Timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// Log level
    pub level: String,
    /// Logger name/component
    pub logger: String,
    /// Log message
    pub message: String,
    /// Structured fields
    pub fields: HashMap<String, serde_json::Value>,
    /// Request ID for correlation
    pub request_id: Option<String>,
    /// User ID if available
    pub user_id: Option<Uuid>,
    /// Trace ID for distributed tracing
    pub trace_id: Option<String>,
}

/// Async logger configuration
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct AsyncLoggerConfig {
    /// Buffer size for log entries
    pub buffer_size: usize,
    /// Whether to drop logs on buffer overflow
    pub drop_on_overflow: bool,
    /// Sampling rate for high-frequency logs (0.0 to 1.0)
    pub sample_rate: f64,
    /// Maximum log message length
    pub max_message_length: usize,
}

impl Default for AsyncLoggerConfig {
    fn default() -> Self {
        Self {
            buffer_size: 10000,
            drop_on_overflow: false,
            sample_rate: 1.0,
            max_message_length: 1024,
        }
    }
}

/// Request metrics for performance logging
#[derive(Debug)]
pub struct RequestMetrics {
    /// HTTP method (GET, POST, etc.)
    pub method: String,
    /// Request path
    pub path: String,
    /// HTTP status code
    pub status_code: u16,
    /// Request duration in milliseconds
    pub duration_ms: u64,
    /// Request size in bytes
    pub request_size: u64,
    /// Response size in bytes
    pub response_size: u64,
    /// Optional user ID
    pub user_id: Option<Uuid>,
    /// Optional request ID for tracing
    pub request_id: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    // ==================== AsyncLogRecord Tests ====================

    #[test]
    fn test_log_entry_creation() {
        let entry = AsyncLogRecord {
            timestamp: Utc::now(),
            level: "INFO".to_string(),
            logger: "gateway".to_string(),
            message: "Request processed".to_string(),
            fields: HashMap::new(),
            request_id: Some("req-123".to_string()),
            user_id: Some(Uuid::new_v4()),
            trace_id: Some("trace-456".to_string()),
        };

        assert_eq!(entry.level, "INFO");
        assert_eq!(entry.logger, "gateway");
        assert!(entry.request_id.is_some());
    }

    #[test]
    fn test_log_entry_minimal() {
        let entry = AsyncLogRecord {
            timestamp: Utc::now(),
            level: "DEBUG".to_string(),
            logger: "app".to_string(),
            message: "Debug message".to_string(),
            fields: HashMap::new(),
            request_id: None,
            user_id: None,
            trace_id: None,
        };

        assert!(entry.request_id.is_none());
        assert!(entry.user_id.is_none());
        assert!(entry.trace_id.is_none());
    }

    #[test]
    fn test_log_entry_with_fields() {
        let mut fields = HashMap::new();
        fields.insert("user_agent".to_string(), serde_json::json!("Mozilla/5.0"));
        fields.insert("ip_address".to_string(), serde_json::json!("192.168.1.1"));
        fields.insert("response_time".to_string(), serde_json::json!(150));

        let entry = AsyncLogRecord {
            timestamp: Utc::now(),
            level: "INFO".to_string(),
            logger: "http".to_string(),
            message: "HTTP request".to_string(),
            fields,
            request_id: None,
            user_id: None,
            trace_id: None,
        };

        assert_eq!(entry.fields.len(), 3);
        assert!(entry.fields.contains_key("user_agent"));
    }

    #[test]
    fn test_log_entry_clone() {
        let entry = AsyncLogRecord {
            timestamp: Utc::now(),
            level: "ERROR".to_string(),
            logger: "error_handler".to_string(),
            message: "An error occurred".to_string(),
            fields: HashMap::new(),
            request_id: Some("req-789".to_string()),
            user_id: None,
            trace_id: None,
        };

        let cloned = entry.clone();
        assert_eq!(cloned.level, entry.level);
        assert_eq!(cloned.message, entry.message);
    }

    #[test]
    fn test_log_entry_debug() {
        let entry = AsyncLogRecord {
            timestamp: Utc::now(),
            level: "WARN".to_string(),
            logger: "test".to_string(),
            message: "Warning".to_string(),
            fields: HashMap::new(),
            request_id: None,
            user_id: None,
            trace_id: None,
        };

        let debug_str = format!("{:?}", entry);
        assert!(debug_str.contains("AsyncLogRecord"));
        assert!(debug_str.contains("WARN"));
    }

    #[test]
    fn test_log_entry_serialization() {
        let entry = AsyncLogRecord {
            timestamp: Utc::now(),
            level: "INFO".to_string(),
            logger: "api".to_string(),
            message: "API call".to_string(),
            fields: HashMap::new(),
            request_id: Some("req-abc".to_string()),
            user_id: None,
            trace_id: None,
        };

        let json = serde_json::to_value(&entry).unwrap();
        assert_eq!(json["level"], "INFO");
        assert_eq!(json["logger"], "api");
    }

    #[test]
    fn test_log_entry_different_levels() {
        let levels = vec!["TRACE", "DEBUG", "INFO", "WARN", "ERROR", "FATAL"];

        for level in levels {
            let entry = AsyncLogRecord {
                timestamp: Utc::now(),
                level: level.to_string(),
                logger: "test".to_string(),
                message: format!("{} message", level),
                fields: HashMap::new(),
                request_id: None,
                user_id: None,
                trace_id: None,
            };

            assert_eq!(entry.level, level);
        }
    }

    // ==================== AsyncLoggerConfig Tests ====================

    #[test]
    fn test_async_logger_config_default() {
        let config = AsyncLoggerConfig::default();

        assert_eq!(config.buffer_size, 10000);
        assert!(!config.drop_on_overflow);
        assert!((config.sample_rate - 1.0).abs() < f64::EPSILON);
        assert_eq!(config.max_message_length, 1024);
    }

    #[test]
    fn test_async_logger_config_custom() {
        let config = AsyncLoggerConfig {
            buffer_size: 5000,
            drop_on_overflow: true,
            sample_rate: 0.5,
            max_message_length: 2048,
        };

        assert_eq!(config.buffer_size, 5000);
        assert!(config.drop_on_overflow);
        assert!((config.sample_rate - 0.5).abs() < f64::EPSILON);
    }

    #[test]
    fn test_async_logger_config_clone() {
        let config = AsyncLoggerConfig::default();
        let cloned = config.clone();

        assert_eq!(cloned.buffer_size, config.buffer_size);
        assert_eq!(cloned.sample_rate, config.sample_rate);
    }

    #[test]
    fn test_async_logger_config_debug() {
        let config = AsyncLoggerConfig::default();
        let debug_str = format!("{:?}", config);

        assert!(debug_str.contains("AsyncLoggerConfig"));
        assert!(debug_str.contains("buffer_size"));
    }

    #[test]
    fn test_async_logger_config_low_sample_rate() {
        let config = AsyncLoggerConfig {
            sample_rate: 0.1,
            ..AsyncLoggerConfig::default()
        };

        assert!(config.sample_rate < 0.5);
    }

    #[test]
    fn test_async_logger_config_small_buffer() {
        let config = AsyncLoggerConfig {
            buffer_size: 100,
            drop_on_overflow: true,
            ..AsyncLoggerConfig::default()
        };

        assert_eq!(config.buffer_size, 100);
        assert!(config.drop_on_overflow);
    }

    // ==================== RequestMetrics Tests ====================

    #[test]
    fn test_request_metrics_creation() {
        let metrics = RequestMetrics {
            method: "GET".to_string(),
            path: "/api/v1/users".to_string(),
            status_code: 200,
            duration_ms: 150,
            request_size: 256,
            response_size: 1024,
            user_id: Some(Uuid::new_v4()),
            request_id: Some("req-123".to_string()),
        };

        assert_eq!(metrics.method, "GET");
        assert_eq!(metrics.status_code, 200);
        assert_eq!(metrics.duration_ms, 150);
    }

    #[test]
    fn test_request_metrics_minimal() {
        let metrics = RequestMetrics {
            method: "POST".to_string(),
            path: "/".to_string(),
            status_code: 201,
            duration_ms: 50,
            request_size: 100,
            response_size: 50,
            user_id: None,
            request_id: None,
        };

        assert!(metrics.user_id.is_none());
        assert!(metrics.request_id.is_none());
    }

    #[test]
    fn test_request_metrics_debug() {
        let metrics = RequestMetrics {
            method: "PUT".to_string(),
            path: "/api/v1/resource".to_string(),
            status_code: 200,
            duration_ms: 100,
            request_size: 500,
            response_size: 200,
            user_id: None,
            request_id: None,
        };

        let debug_str = format!("{:?}", metrics);
        assert!(debug_str.contains("RequestMetrics"));
        assert!(debug_str.contains("PUT"));
    }

    #[test]
    fn test_request_metrics_different_methods() {
        let methods = vec!["GET", "POST", "PUT", "PATCH", "DELETE", "HEAD", "OPTIONS"];

        for method in methods {
            let metrics = RequestMetrics {
                method: method.to_string(),
                path: "/test".to_string(),
                status_code: 200,
                duration_ms: 10,
                request_size: 0,
                response_size: 0,
                user_id: None,
                request_id: None,
            };

            assert_eq!(metrics.method, method);
        }
    }

    #[test]
    fn test_request_metrics_different_status_codes() {
        let status_codes = vec![200, 201, 204, 301, 400, 401, 403, 404, 500, 502, 503];

        for code in status_codes {
            let metrics = RequestMetrics {
                method: "GET".to_string(),
                path: "/test".to_string(),
                status_code: code,
                duration_ms: 10,
                request_size: 0,
                response_size: 0,
                user_id: None,
                request_id: None,
            };

            assert_eq!(metrics.status_code, code);
        }
    }

    #[test]
    fn test_request_metrics_large_sizes() {
        let metrics = RequestMetrics {
            method: "POST".to_string(),
            path: "/upload".to_string(),
            status_code: 200,
            duration_ms: 5000,
            request_size: 10_000_000, // 10MB
            response_size: 1_000,
            user_id: None,
            request_id: None,
        };

        assert_eq!(metrics.request_size, 10_000_000);
    }

    #[test]
    fn test_request_metrics_slow_request() {
        let metrics = RequestMetrics {
            method: "GET".to_string(),
            path: "/slow-endpoint".to_string(),
            status_code: 200,
            duration_ms: 30_000, // 30 seconds
            request_size: 100,
            response_size: 1000,
            user_id: None,
            request_id: None,
        };

        assert!(metrics.duration_ms > 10_000);
    }

    // ==================== Integration Tests ====================

    #[test]
    fn test_log_entry_for_request() {
        let request_id = "req-12345";
        let user_id = Uuid::new_v4();

        let entry = AsyncLogRecord {
            timestamp: Utc::now(),
            level: "INFO".to_string(),
            logger: "http".to_string(),
            message: "Request completed".to_string(),
            fields: {
                let mut f = HashMap::new();
                f.insert("status".to_string(), serde_json::json!(200));
                f.insert("duration_ms".to_string(), serde_json::json!(150));
                f
            },
            request_id: Some(request_id.to_string()),
            user_id: Some(user_id),
            trace_id: None,
        };

        assert_eq!(entry.request_id.as_ref().unwrap(), request_id);
        assert_eq!(entry.user_id.unwrap(), user_id);
    }

    #[test]
    fn test_sampling_decision_simulation() {
        let config = AsyncLoggerConfig {
            sample_rate: 0.1,
            ..AsyncLoggerConfig::default()
        };

        // Simulate sampling decision
        let should_log = |rate: f64| -> bool {
            // In reality, this would use rand::random::<f64>() < rate
            rate >= 1.0 // Always log if rate is 1.0
        };

        assert!(!should_log(config.sample_rate));
        assert!(should_log(1.0));
    }

    #[test]
    fn test_message_truncation_simulation() {
        let config = AsyncLoggerConfig {
            max_message_length: 100,
            ..AsyncLoggerConfig::default()
        };

        let long_message = "A".repeat(200);
        let truncated = if long_message.len() > config.max_message_length {
            format!("{}...", &long_message[..config.max_message_length - 3])
        } else {
            long_message.clone()
        };

        assert_eq!(truncated.len(), config.max_message_length);
        assert!(truncated.ends_with("..."));
    }

    #[test]
    fn test_request_metrics_is_success() {
        let success_metrics = RequestMetrics {
            method: "GET".to_string(),
            path: "/api".to_string(),
            status_code: 200,
            duration_ms: 100,
            request_size: 0,
            response_size: 500,
            user_id: None,
            request_id: None,
        };

        let error_metrics = RequestMetrics {
            method: "GET".to_string(),
            path: "/api".to_string(),
            status_code: 500,
            duration_ms: 50,
            request_size: 0,
            response_size: 100,
            user_id: None,
            request_id: None,
        };

        let is_success = |code: u16| (200..400).contains(&code);

        assert!(is_success(success_metrics.status_code));
        assert!(!is_success(error_metrics.status_code));
    }

    #[test]
    fn test_request_metrics_is_error() {
        let client_error = RequestMetrics {
            method: "POST".to_string(),
            path: "/api".to_string(),
            status_code: 400,
            duration_ms: 10,
            request_size: 100,
            response_size: 50,
            user_id: None,
            request_id: None,
        };

        let server_error = RequestMetrics {
            method: "POST".to_string(),
            path: "/api".to_string(),
            status_code: 503,
            duration_ms: 100,
            request_size: 100,
            response_size: 50,
            user_id: None,
            request_id: None,
        };

        let is_client_error = |code: u16| (400..500).contains(&code);
        let is_server_error = |code: u16| code >= 500;

        assert!(is_client_error(client_error.status_code));
        assert!(is_server_error(server_error.status_code));
    }
}
