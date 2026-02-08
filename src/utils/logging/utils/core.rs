use super::types::{LogEntry, LogLevel};
use crate::core::providers::unified_provider::ProviderError;
use std::collections::HashMap;
use std::env;
use tracing::{Level, debug, error, info, warn};
use uuid::Uuid;

pub struct LoggingUtils;

impl LoggingUtils {
    pub fn print_verbose(message: &str, logger_only: bool, log_level: LogLevel) {
        match log_level {
            LogLevel::Debug => debug!("{}", message),
            LogLevel::Info => info!("{}", message),
            LogLevel::Warn => warn!("{}", message),
            LogLevel::Error => error!("{}", message),
        }

        if !logger_only && Self::is_verbose_enabled() {
            // For verbose mode, also print to stdout for immediate feedback
            println!("{}", message);
        }
    }

    pub fn is_verbose_enabled() -> bool {
        env::var("LITELLM_VERBOSE")
            .map(|v| v.to_lowercase())
            .map(|v| v == "true" || v == "1")
            .unwrap_or(false)
    }

    pub fn set_verbose(enabled: bool) {
        unsafe {
            env::set_var("LITELLM_VERBOSE", if enabled { "true" } else { "false" });
        }
    }

    pub fn get_logging_id() -> String {
        Uuid::new_v4().to_string()
    }

    pub fn get_logging_id_with_timestamp(start_time: chrono::DateTime<chrono::Utc>) -> String {
        let timestamp = start_time.timestamp();
        let uuid = Uuid::new_v4();
        format!("{}-{}", timestamp, uuid)
    }

    pub fn init_logger(log_level: Option<LogLevel>) -> Result<(), ProviderError> {
        let level = log_level.unwrap_or(LogLevel::Info);

        tracing_subscriber::fmt()
            .with_max_level(Level::from(level))
            .with_target(false)
            .with_thread_ids(true)
            .with_file(true)
            .with_line_number(true)
            .init();

        Ok(())
    }

    pub fn create_structured_log(
        level: LogLevel,
        message: &str,
        module: Option<&str>,
        metadata: Option<HashMap<String, serde_json::Value>>,
    ) -> LogEntry {
        let mut entry = LogEntry::new(level, message.to_string());

        if let Some(mod_name) = module {
            entry = entry.with_module(mod_name.to_string());
        }

        if let Some(meta) = metadata {
            for (key, value) in meta {
                entry = entry.with_metadata(key, value);
            }
        }

        entry
    }

    pub fn get_log_level_from_env() -> LogLevel {
        env::var("LITELLM_LOG_LEVEL")
            .unwrap_or_else(|_| "INFO".to_string())
            .parse()
            .unwrap_or(LogLevel::Info)
    }

    pub fn should_log_at_level(current_level: &LogLevel, target_level: &LogLevel) -> bool {
        let current_priority = match current_level {
            LogLevel::Debug => 0,
            LogLevel::Info => 1,
            LogLevel::Warn => 2,
            LogLevel::Error => 3,
        };

        let target_priority = match target_level {
            LogLevel::Debug => 0,
            LogLevel::Info => 1,
            LogLevel::Warn => 2,
            LogLevel::Error => 3,
        };

        target_priority >= current_priority
    }

    pub fn format_duration(duration: std::time::Duration) -> String {
        let total_ms = duration.as_millis();

        if total_ms < 1000 {
            format!("{}ms", total_ms)
        } else if total_ms < 60_000 {
            format!("{:.2}s", total_ms as f64 / 1000.0)
        } else {
            let minutes = total_ms / 60_000;
            let seconds = (total_ms % 60_000) as f64 / 1000.0;
            format!("{}m {:.2}s", minutes, seconds)
        }
    }
}
