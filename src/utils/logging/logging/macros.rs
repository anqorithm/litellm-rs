//! Convenience macros for structured logging

/// Convenience macro for structured logging
#[macro_export]
macro_rules! log_structured {
    ($level:expr, $logger:expr, $message:expr, $($key:expr => $value:expr),*) => {
        {
            let mut fields = std::collections::HashMap::new();
            $(
                fields.insert($key.to_string(), serde_json::to_value($value).unwrap_or(serde_json::Value::Null));
            )*

            if let Some(logger) = $crate::utils::logging::logging::async_logger() {
                logger.log_structured($level, $logger, $message, fields, None, None);
            }
        }
    };
}
