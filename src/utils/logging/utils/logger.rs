use super::core::LoggingUtils;
use super::file_logging::FileLogging;
use super::types::LogLevel;
use crate::core::providers::unified_provider::ProviderError;
use std::collections::HashMap;
use std::fs::File;
use std::io::BufWriter;
use std::sync::{Arc, Mutex};

pub struct Logger {
    level: LogLevel,
    file_writer: Option<Arc<Mutex<BufWriter<File>>>>,
}

impl Logger {
    pub fn new(level: LogLevel) -> Self {
        Self {
            level,
            file_writer: None,
        }
    }

    pub fn with_file_output(mut self, file_path: &str) -> Result<Self, ProviderError> {
        self.file_writer = Some(FileLogging::setup_file_logging(file_path)?);
        Ok(self)
    }

    pub fn log(&self, level: LogLevel, message: &str, context: Option<HashMap<String, String>>) {
        if !LoggingUtils::should_log_at_level(&self.level, &level) {
            return;
        }

        let mut metadata = HashMap::new();
        if let Some(ctx) = context {
            for (key, value) in ctx {
                metadata.insert(key, serde_json::Value::String(value));
            }
        }

        let entry =
            LoggingUtils::create_structured_log(level.clone(), message, None, Some(metadata));

        LoggingUtils::print_verbose(message, false, level);

        if let Some(writer) = &self.file_writer {
            let _ = FileLogging::log_to_file(writer, &entry);
        }
    }

    pub fn debug(&self, message: &str) {
        self.log(LogLevel::Debug, message, None);
    }

    pub fn info(&self, message: &str) {
        self.log(LogLevel::Info, message, None);
    }

    pub fn warn(&self, message: &str) {
        self.log(LogLevel::Warn, message, None);
    }

    pub fn error(&self, message: &str) {
        self.log(LogLevel::Error, message, None);
    }
}
