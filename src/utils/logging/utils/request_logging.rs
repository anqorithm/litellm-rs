use super::core::LoggingUtils;
use super::types::{LogEntry, LogLevel};
use crate::core::providers::unified_provider::ProviderError;
use std::collections::HashMap;
use tracing::{error, info};

pub struct RequestLogging;

impl RequestLogging {
    pub fn log_request_start(request_id: &str, model: &str, provider: &str, endpoint: &str) {
        let entry = LogEntry::new(LogLevel::Info, "Request started".to_string())
            .with_request_id(request_id.to_string())
            .with_metadata(
                "model".to_string(),
                serde_json::Value::String(model.to_string()),
            )
            .with_metadata(
                "provider".to_string(),
                serde_json::Value::String(provider.to_string()),
            )
            .with_metadata(
                "endpoint".to_string(),
                serde_json::Value::String(endpoint.to_string()),
            );

        info!("{}", serde_json::to_string(&entry).unwrap_or_default());
    }

    pub fn log_request_end(
        request_id: &str,
        status_code: Option<u16>,
        duration_ms: u64,
        token_usage: Option<(usize, usize)>,
    ) {
        let mut entry = LogEntry::new(LogLevel::Info, "Request completed".to_string())
            .with_request_id(request_id.to_string())
            .with_metadata(
                "duration_ms".to_string(),
                serde_json::Value::Number(duration_ms.into()),
            );

        if let Some(status) = status_code {
            entry = entry.with_metadata(
                "status_code".to_string(),
                serde_json::Value::Number(status.into()),
            );
        }

        if let Some((input_tokens, output_tokens)) = token_usage {
            entry = entry
                .with_metadata(
                    "input_tokens".to_string(),
                    serde_json::Value::Number(input_tokens.into()),
                )
                .with_metadata(
                    "output_tokens".to_string(),
                    serde_json::Value::Number(output_tokens.into()),
                );
        }

        info!("{}", serde_json::to_string(&entry).unwrap_or_default());
    }

    pub fn log_error(
        request_id: Option<&str>,
        error: &ProviderError,
        context: Option<HashMap<String, String>>,
    ) {
        let mut entry = LogEntry::new(LogLevel::Error, error.to_string());

        if let Some(id) = request_id {
            entry = entry.with_request_id(id.to_string());
        }

        if let Some(ctx) = context {
            for (key, value) in ctx {
                entry = entry.with_metadata(key, serde_json::Value::String(value));
            }
        }

        error!("{}", serde_json::to_string(&entry).unwrap_or_default());
    }

    pub fn log_performance_metrics(
        operation: &str,
        duration: std::time::Duration,
        metadata: Option<HashMap<String, String>>,
    ) {
        let mut entry = LogEntry::new(LogLevel::Info, format!("Performance: {}", operation))
            .with_metadata(
                "duration".to_string(),
                serde_json::Value::String(LoggingUtils::format_duration(duration)),
            )
            .with_metadata(
                "duration_ms".to_string(),
                serde_json::Value::Number(serde_json::Number::from(duration.as_millis() as u64)),
            );

        if let Some(meta) = metadata {
            for (key, value) in meta {
                entry = entry.with_metadata(key, serde_json::Value::String(value));
            }
        }

        info!("{}", serde_json::to_string(&entry).unwrap_or_default());
    }
}
