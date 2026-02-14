//! Performance logging utilities

use crate::utils::logging::enhanced::async_logger::async_logger;
use crate::utils::logging::enhanced::types::HttpRequestMetrics;
use std::collections::HashMap;
use tracing::Level;

/// Performance logging utilities
#[allow(dead_code)]
pub struct PerformanceLogger;

#[allow(dead_code)]
impl PerformanceLogger {
    /// Log request performance metrics
    pub fn log_request_metrics(metrics: HttpRequestMetrics) {
        let mut fields = HashMap::new();
        fields.insert(
            "method".to_string(),
            serde_json::Value::String(metrics.method.clone()),
        );
        fields.insert(
            "path".to_string(),
            serde_json::Value::String(metrics.path.clone()),
        );
        fields.insert(
            "status_code".to_string(),
            serde_json::Value::Number(metrics.status_code.into()),
        );
        fields.insert(
            "duration_ms".to_string(),
            serde_json::Value::Number(metrics.duration_ms.into()),
        );
        fields.insert(
            "request_size".to_string(),
            serde_json::Value::Number(metrics.request_size.into()),
        );
        fields.insert(
            "response_size".to_string(),
            serde_json::Value::Number(metrics.response_size.into()),
        );

        let message = format!(
            "{} {} {} {}ms",
            metrics.method, metrics.path, metrics.status_code, metrics.duration_ms
        );

        // Use different log levels based on performance
        let level = if metrics.duration_ms > 5000 {
            Level::WARN // Very slow requests
        } else if metrics.duration_ms > 1000 {
            Level::INFO // Slow requests
        } else {
            Level::DEBUG // Normal requests
        };

        if let Some(logger) = async_logger() {
            logger.log_structured(
                level,
                "performance",
                &message,
                fields,
                metrics.request_id,
                metrics.user_id,
            );
        }
    }

    /// Log provider performance metrics
    pub fn log_provider_metrics(
        provider: &str,
        model: &str,
        duration_ms: u64,
        token_count: Option<u32>,
        success: bool,
        error: Option<&str>,
    ) {
        let mut fields = HashMap::new();
        fields.insert(
            "provider".to_string(),
            serde_json::Value::String(provider.to_string()),
        );
        fields.insert(
            "model".to_string(),
            serde_json::Value::String(model.to_string()),
        );
        fields.insert(
            "duration_ms".to_string(),
            serde_json::Value::Number(duration_ms.into()),
        );
        fields.insert("success".to_string(), serde_json::Value::Bool(success));

        if let Some(tokens) = token_count {
            fields.insert(
                "token_count".to_string(),
                serde_json::Value::Number(tokens.into()),
            );
        }

        if let Some(err) = error {
            fields.insert(
                "error".to_string(),
                serde_json::Value::String(err.to_string()),
            );
        }

        let level = if success { Level::DEBUG } else { Level::WARN };
        let message = format!(
            "Provider {} {} {}ms {}",
            provider,
            model,
            duration_ms,
            if success { "success" } else { "failed" }
        );

        if let Some(logger) = async_logger() {
            logger.log_structured(level, "performance", &message, fields, None, None);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::logging::enhanced::types::HttpRequestMetrics;
    use uuid::Uuid;

    fn create_test_request_metrics(duration_ms: u64, status_code: u16) -> HttpRequestMetrics {
        HttpRequestMetrics {
            method: "GET".to_string(),
            path: "/api/test".to_string(),
            status_code,
            duration_ms,
            request_size: 1024,
            response_size: 2048,
            user_id: Some(Uuid::new_v4()),
            request_id: Some("test-request-123".to_string()),
        }
    }

    #[test]
    fn test_request_metrics_creation() {
        let metrics = create_test_request_metrics(100, 200);
        assert_eq!(metrics.method, "GET");
        assert_eq!(metrics.path, "/api/test");
        assert_eq!(metrics.status_code, 200);
        assert_eq!(metrics.duration_ms, 100);
    }

    #[test]
    fn test_request_metrics_with_optional_fields() {
        let mut metrics = create_test_request_metrics(100, 200);
        assert!(metrics.user_id.is_some());
        assert!(metrics.request_id.is_some());

        metrics.user_id = None;
        metrics.request_id = None;
        assert!(metrics.user_id.is_none());
        assert!(metrics.request_id.is_none());
    }

    #[test]
    fn test_request_metrics_various_status_codes() {
        let success = create_test_request_metrics(100, 200);
        assert_eq!(success.status_code, 200);

        let error = create_test_request_metrics(100, 500);
        assert_eq!(error.status_code, 500);

        let not_found = create_test_request_metrics(100, 404);
        assert_eq!(not_found.status_code, 404);
    }

    #[test]
    fn test_request_metrics_various_durations() {
        let fast = create_test_request_metrics(10, 200);
        assert!(fast.duration_ms < 100);

        let slow = create_test_request_metrics(2000, 200);
        assert!(slow.duration_ms > 1000);

        let very_slow = create_test_request_metrics(6000, 200);
        assert!(very_slow.duration_ms > 5000);
    }

    #[test]
    fn test_performance_logger_is_zero_sized() {
        // PerformanceLogger is a unit struct with no fields
        // Verify it has zero size
        assert_eq!(std::mem::size_of::<PerformanceLogger>(), 0);
    }
}
