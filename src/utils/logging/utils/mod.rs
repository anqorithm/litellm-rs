//! Logging utilities module
//!
//! This module provides comprehensive logging utilities organized into submodules:
//! - `types`: Core types (LogLevel, LogEntry)
//! - `utils`: Utility functions for logging control and formatting
//! - `file_logging`: File-based logging functionality
//! - `request_logging`: Request/response logging
//! - `sanitization`: Data sanitization and masking
//! - `logger`: Logger struct for structured logging

pub mod core;
pub mod file_logging;
pub mod logger;
pub mod request_logging;
pub mod sanitization;
#[cfg(test)]
mod tests;
pub mod types;

use self::core::LoggingUtils as CoreLoggingUtils;
use crate::core::providers::unified_provider::ProviderError;
use file_logging::FileLogging;
use request_logging::RequestLogging;
use sanitization::Sanitization;
use types::{LogEntry, LogLevel};

use std::collections::HashMap;
use std::fs::File;
use std::io::BufWriter;
use std::sync::{Arc, Mutex};

/// Unified LoggingUtils facade that combines all logging functionality
///
/// This struct provides a unified interface to all logging utilities,
/// maintaining backward compatibility with the original API.
pub struct LoggingUtils;

impl LoggingUtils {
    // Verbose control methods
    pub fn print_verbose(message: &str, logger_only: bool, log_level: LogLevel) {
        CoreLoggingUtils::print_verbose(message, logger_only, log_level);
    }

    pub fn is_verbose_enabled() -> bool {
        CoreLoggingUtils::is_verbose_enabled()
    }

    pub fn set_verbose(enabled: bool) {
        CoreLoggingUtils::set_verbose(enabled);
    }

    // ID generation methods
    pub fn get_logging_id() -> String {
        CoreLoggingUtils::get_logging_id()
    }

    pub fn get_logging_id_with_timestamp(start_time: chrono::DateTime<chrono::Utc>) -> String {
        CoreLoggingUtils::get_logging_id_with_timestamp(start_time)
    }

    // Logger initialization
    pub fn init_logger(log_level: Option<LogLevel>) -> Result<(), ProviderError> {
        CoreLoggingUtils::init_logger(log_level)
    }

    // File logging methods
    pub fn setup_file_logging(
        log_file_path: &str,
    ) -> Result<Arc<Mutex<BufWriter<File>>>, ProviderError> {
        FileLogging::setup_file_logging(log_file_path)
    }

    pub fn log_to_file(
        writer: &Arc<Mutex<BufWriter<File>>>,
        entry: &LogEntry,
    ) -> Result<(), ProviderError> {
        FileLogging::log_to_file(writer, entry)
    }

    // Request logging methods
    pub fn log_request_start(request_id: &str, model: &str, provider: &str, endpoint: &str) {
        RequestLogging::log_request_start(request_id, model, provider, endpoint);
    }

    pub fn log_request_end(
        request_id: &str,
        status_code: Option<u16>,
        duration_ms: u64,
        token_usage: Option<(usize, usize)>,
    ) {
        RequestLogging::log_request_end(request_id, status_code, duration_ms, token_usage);
    }

    pub fn log_error(
        request_id: Option<&str>,
        error: &ProviderError,
        context: Option<HashMap<String, String>>,
    ) {
        RequestLogging::log_error(request_id, error, context);
    }

    pub fn log_performance_metrics(
        operation: &str,
        duration: std::time::Duration,
        metadata: Option<HashMap<String, String>>,
    ) {
        RequestLogging::log_performance_metrics(operation, duration, metadata);
    }

    // Sanitization methods
    pub fn sanitize_log_data(data: &str) -> String {
        Sanitization::sanitize_log_data(data)
    }

    pub fn mask_sensitive_data(input: &str) -> String {
        Sanitization::mask_sensitive_data(input)
    }

    // Utility methods
    pub fn create_structured_log(
        level: LogLevel,
        message: &str,
        module: Option<&str>,
        metadata: Option<HashMap<String, serde_json::Value>>,
    ) -> LogEntry {
        CoreLoggingUtils::create_structured_log(level, message, module, metadata)
    }

    pub fn get_log_level_from_env() -> LogLevel {
        CoreLoggingUtils::get_log_level_from_env()
    }

    pub fn should_log_at_level(current_level: &LogLevel, target_level: &LogLevel) -> bool {
        CoreLoggingUtils::should_log_at_level(current_level, target_level)
    }

    pub fn format_duration(duration: std::time::Duration) -> String {
        CoreLoggingUtils::format_duration(duration)
    }
}
