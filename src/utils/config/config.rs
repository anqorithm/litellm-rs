//! Configuration utilities for the Gateway
//!
//! This module provides utilities for configuration management and environment handling.

#![allow(dead_code)]

use crate::utils::error::{GatewayError, Result};
use std::sync::LazyLock;
use regex::Regex;
use std::collections::HashMap;
use std::env;
use std::path::Path;

/// Environment variable utilities
pub struct EnvUtils;

impl EnvUtils {
    /// Get environment variable with default value
    pub fn get_env_or_default(key: &str, default: &str) -> String {
        env::var(key).unwrap_or_else(|_| default.to_string())
    }

    /// Get required environment variable
    pub fn get_required_env(key: &str) -> Result<String> {
        env::var(key).map_err(|_| {
            GatewayError::Config(format!("Required environment variable {} not found", key))
        })
    }

    /// Get environment variable as integer
    pub fn get_env_as_int(key: &str, default: i32) -> Result<i32> {
        match env::var(key) {
            Ok(value) => value.parse().map_err(|e| {
                GatewayError::Config(format!("Invalid integer value for {}: {}", key, e))
            }),
            Err(_) => Ok(default),
        }
    }

    /// Get environment variable as boolean
    pub fn get_env_as_bool(key: &str, default: bool) -> bool {
        match env::var(key) {
            Ok(value) => {
                matches!(value.to_lowercase().as_str(), "true" | "1" | "yes" | "on")
            }
            Err(_) => default,
        }
    }

    /// Get environment variable as float
    pub fn get_env_as_float(key: &str, default: f64) -> Result<f64> {
        match env::var(key) {
            Ok(value) => value.parse().map_err(|e| {
                GatewayError::Config(format!("Invalid float value for {}: {}", key, e))
            }),
            Err(_) => Ok(default),
        }
    }

    /// Get environment variable as list (comma-separated)
    pub fn get_env_as_list(key: &str, default: Vec<String>) -> Vec<String> {
        match env::var(key) {
            Ok(value) => value
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect(),
            Err(_) => default,
        }
    }

    /// Check if running in development mode
    pub fn is_development() -> bool {
        Self::get_env_or_default("ENVIRONMENT", "development") == "development"
    }

    /// Check if running in production mode
    pub fn is_production() -> bool {
        Self::get_env_or_default("ENVIRONMENT", "development") == "production"
    }

    /// Get all environment variables with a prefix
    pub fn get_env_with_prefix(prefix: &str) -> HashMap<String, String> {
        env::vars()
            .filter(|(key, _)| key.starts_with(prefix))
            .map(|(key, value)| (key[prefix.len()..].to_string(), value))
            .collect()
    }

    /// Set environment variable (for testing)
    #[cfg(test)]
    pub fn set_env(key: &str, value: &str) {
        unsafe {
            env::set_var(key, value);
        }
    }

    /// Remove environment variable (for testing)
    #[cfg(test)]
    pub fn remove_env(key: &str) {
        unsafe {
            env::remove_var(key);
        }
    }
}

/// Configuration file utilities
pub struct ConfigFileUtils;

impl ConfigFileUtils {
    /// Check if file exists
    pub fn file_exists<P: AsRef<Path>>(path: P) -> bool {
        path.as_ref().exists()
    }

    /// Get file extension
    pub fn get_file_extension<P: AsRef<Path>>(path: P) -> Option<String> {
        path.as_ref()
            .extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| ext.to_lowercase())
    }

    /// Read file content
    pub async fn read_file<P: AsRef<Path>>(path: P) -> Result<String> {
        tokio::fs::read_to_string(path)
            .await
            .map_err(|e| GatewayError::Config(format!("Failed to read file: {}", e)))
    }

    /// Write file content
    pub async fn write_file<P: AsRef<Path>>(path: P, content: &str) -> Result<()> {
        // Create parent directories if they don't exist
        if let Some(parent) = path.as_ref().parent() {
            tokio::fs::create_dir_all(parent).await.map_err(|e| {
                GatewayError::Config(format!("Failed to create directories: {}", e))
            })?;
        }

        tokio::fs::write(path, content)
            .await
            .map_err(|e| GatewayError::Config(format!("Failed to write file: {}", e)))
    }

    /// Parse YAML file
    pub async fn parse_yaml_file<T, P>(path: P) -> Result<T>
    where
        T: serde::de::DeserializeOwned,
        P: AsRef<Path>,
    {
        let content = Self::read_file(path).await?;
        serde_yaml::from_str(&content)
            .map_err(|e| GatewayError::Config(format!("Failed to parse YAML: {}", e)))
    }

    /// Parse JSON file
    pub async fn parse_json_file<T, P>(path: P) -> Result<T>
    where
        T: serde::de::DeserializeOwned,
        P: AsRef<Path>,
    {
        let content = Self::read_file(path).await?;
        serde_json::from_str(&content)
            .map_err(|e| GatewayError::Config(format!("Failed to parse JSON: {}", e)))
    }

    /// Write YAML file
    pub async fn write_yaml_file<T, P>(path: P, data: &T) -> Result<()>
    where
        T: serde::Serialize,
        P: AsRef<Path>,
    {
        let content = serde_yaml::to_string(data)
            .map_err(|e| GatewayError::Config(format!("Failed to serialize YAML: {}", e)))?;
        Self::write_file(path, &content).await
    }

    /// Write JSON file
    pub async fn write_json_file<T, P>(path: P, data: &T) -> Result<()>
    where
        T: serde::Serialize,
        P: AsRef<Path>,
    {
        let content = serde_json::to_string_pretty(data)
            .map_err(|e| GatewayError::Config(format!("Failed to serialize JSON: {}", e)))?;
        Self::write_file(path, &content).await
    }

    /// Find configuration file in multiple locations
    pub fn find_config_file(filename: &str) -> Option<std::path::PathBuf> {
        let search_paths = [
            std::path::PathBuf::from(filename),
            std::path::PathBuf::from(format!("config/{}", filename)),
            std::path::PathBuf::from(format!("./config/{}", filename)),
            std::path::PathBuf::from(format!("/etc/gateway/{}", filename)),
            std::path::PathBuf::from(format!("~/.config/gateway/{}", filename)),
        ];

        for path in &search_paths {
            if path.exists() {
                return Some(path.clone());
            }
        }

        None
    }
}

/// Configuration validation utilities
pub struct ConfigValidator;

impl ConfigValidator {
    /// Validate URL format
    pub fn validate_url(url: &str) -> Result<()> {
        url::Url::parse(url)
            .map_err(|e| GatewayError::Validation(format!("Invalid URL format: {}", e)))?;
        Ok(())
    }

    /// Validate email format
    pub fn validate_email(email: &str) -> Result<()> {
        static EMAIL_REGEX: LazyLock<Regex> =
            LazyLock::new(|| Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$").unwrap());

        if !EMAIL_REGEX.is_match(email) {
            return Err(GatewayError::Validation("Invalid email format".to_string()));
        }
        Ok(())
    }

    /// Validate port number
    pub fn validate_port(port: u16) -> Result<()> {
        if port == 0 {
            return Err(GatewayError::Validation("Port cannot be 0".to_string()));
        }
        Ok(())
    }

    /// Validate positive integer
    pub fn validate_positive_int(value: i32, field_name: &str) -> Result<()> {
        if value <= 0 {
            return Err(GatewayError::Validation(format!(
                "{} must be positive",
                field_name
            )));
        }
        Ok(())
    }

    /// Validate range
    pub fn validate_range<T>(value: T, min: T, max: T, field_name: &str) -> Result<()>
    where
        T: PartialOrd + std::fmt::Display,
    {
        if value < min || value > max {
            return Err(GatewayError::Validation(format!(
                "{} must be between {} and {}",
                field_name, min, max
            )));
        }
        Ok(())
    }

    /// Validate string length
    pub fn validate_string_length(
        value: &str,
        min: usize,
        max: usize,
        field_name: &str,
    ) -> Result<()> {
        let len = value.len();
        if len < min || len > max {
            return Err(GatewayError::Validation(format!(
                "{} length must be between {} and {} characters",
                field_name, min, max
            )));
        }
        Ok(())
    }

    /// Validate required field
    pub fn validate_required<T>(value: &Option<T>, field_name: &str) -> Result<()> {
        if value.is_none() {
            return Err(GatewayError::Validation(format!(
                "{} is required",
                field_name
            )));
        }
        Ok(())
    }

    /// Validate non-empty string
    pub fn validate_non_empty(value: &str, field_name: &str) -> Result<()> {
        if value.trim().is_empty() {
            return Err(GatewayError::Validation(format!(
                "{} cannot be empty",
                field_name
            )));
        }
        Ok(())
    }

    /// Validate alphanumeric string
    pub fn validate_alphanumeric(value: &str, field_name: &str) -> Result<()> {
        if !value
            .chars()
            .all(|c| c.is_alphanumeric() || c == '_' || c == '-')
        {
            return Err(GatewayError::Validation(format!(
                "{} can only contain alphanumeric characters, underscores, and hyphens",
                field_name
            )));
        }
        Ok(())
    }

    /// Validate JSON format
    pub fn validate_json(value: &str) -> Result<()> {
        serde_json::from_str::<serde_json::Value>(value)
            .map_err(|e| GatewayError::Validation(format!("Invalid JSON format: {}", e)))?;
        Ok(())
    }

    /// Validate duration string (e.g., "30s", "5m", "1h")
    pub fn validate_duration_string(value: &str) -> Result<std::time::Duration> {
        static DURATION_REGEX: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^(\d+)(s|m|h|d)$").unwrap());

        if let Some(captures) = DURATION_REGEX.captures(value) {
            let number: u64 = captures[1]
                .parse()
                .map_err(|e| GatewayError::Validation(format!("Invalid duration number: {}", e)))?;

            let unit = &captures[2];
            let duration = match unit {
                "s" => std::time::Duration::from_secs(number),
                "m" => std::time::Duration::from_secs(number * 60),
                "h" => std::time::Duration::from_secs(number * 3600),
                "d" => std::time::Duration::from_secs(number * 86400),
                _ => {
                    return Err(GatewayError::Validation(
                        "Invalid duration unit".to_string(),
                    ));
                }
            };

            Ok(duration)
        } else {
            Err(GatewayError::Validation(
                "Invalid duration format. Use format like '30s', '5m', '1h', '1d'".to_string(),
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    // ==================== EnvUtils Tests ====================

    #[test]
    fn test_env_utils_get_or_default() {
        EnvUtils::set_env("TEST_VAR_1", "test_value");
        assert_eq!(
            EnvUtils::get_env_or_default("TEST_VAR_1", "default"),
            "test_value"
        );
        assert_eq!(
            EnvUtils::get_env_or_default("NON_EXISTENT_VAR", "default"),
            "default"
        );
        EnvUtils::remove_env("TEST_VAR_1");
    }

    #[test]
    fn test_env_utils_get_required_env() {
        EnvUtils::set_env("REQUIRED_VAR", "required_value");
        let result = EnvUtils::get_required_env("REQUIRED_VAR");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "required_value");

        let missing = EnvUtils::get_required_env("MISSING_REQUIRED_VAR");
        assert!(missing.is_err());

        EnvUtils::remove_env("REQUIRED_VAR");
    }

    #[test]
    fn test_env_utils_get_as_int() {
        EnvUtils::set_env("INT_VAR", "42");
        assert_eq!(EnvUtils::get_env_as_int("INT_VAR", 0).unwrap(), 42);

        EnvUtils::set_env("INVALID_INT", "not_a_number");
        assert!(EnvUtils::get_env_as_int("INVALID_INT", 0).is_err());

        assert_eq!(EnvUtils::get_env_as_int("MISSING_INT", 99).unwrap(), 99);

        EnvUtils::remove_env("INT_VAR");
        EnvUtils::remove_env("INVALID_INT");
    }

    #[test]
    fn test_env_utils_get_as_int_negative() {
        EnvUtils::set_env("NEG_INT", "-42");
        assert_eq!(EnvUtils::get_env_as_int("NEG_INT", 0).unwrap(), -42);
        EnvUtils::remove_env("NEG_INT");
    }

    #[test]
    fn test_env_utils_get_as_bool() {
        let true_values = ["true", "1", "yes", "on", "TRUE", "YES", "ON"];
        for (i, val) in true_values.iter().enumerate() {
            let key = format!("BOOL_VAR_{}", i);
            EnvUtils::set_env(&key, val);
            assert!(
                EnvUtils::get_env_as_bool(&key, false),
                "Failed for: {}",
                val
            );
            EnvUtils::remove_env(&key);
        }

        let false_values = ["false", "0", "no", "off", "random"];
        for (i, val) in false_values.iter().enumerate() {
            let key = format!("BOOL_FALSE_VAR_{}", i);
            EnvUtils::set_env(&key, val);
            assert!(
                !EnvUtils::get_env_as_bool(&key, true),
                "Failed for: {}",
                val
            );
            EnvUtils::remove_env(&key);
        }

        assert!(!EnvUtils::get_env_as_bool("MISSING_BOOL", false));
        assert!(EnvUtils::get_env_as_bool("MISSING_BOOL", true));
    }

    #[test]
    fn test_env_utils_get_as_float() {
        EnvUtils::set_env("FLOAT_VAR", "1.234");
        let result = EnvUtils::get_env_as_float("FLOAT_VAR", 0.0).unwrap();
        assert!((result - 1.234).abs() < 0.001);

        EnvUtils::set_env("INVALID_FLOAT", "not_a_float");
        assert!(EnvUtils::get_env_as_float("INVALID_FLOAT", 0.0).is_err());

        let default = EnvUtils::get_env_as_float("MISSING_FLOAT", 1.5).unwrap();
        assert!((default - 1.5).abs() < 0.001);

        EnvUtils::remove_env("FLOAT_VAR");
        EnvUtils::remove_env("INVALID_FLOAT");
    }

    #[test]
    fn test_env_utils_get_as_list() {
        EnvUtils::set_env("LIST_VAR", "a,b,c");
        assert_eq!(
            EnvUtils::get_env_as_list("LIST_VAR", vec![]),
            vec!["a", "b", "c"]
        );

        EnvUtils::set_env("LIST_WITH_SPACES", "a, b, c");
        assert_eq!(
            EnvUtils::get_env_as_list("LIST_WITH_SPACES", vec![]),
            vec!["a", "b", "c"]
        );

        EnvUtils::set_env("SINGLE_ITEM", "single");
        assert_eq!(
            EnvUtils::get_env_as_list("SINGLE_ITEM", vec![]),
            vec!["single"]
        );

        EnvUtils::set_env("EMPTY_ITEMS", "a,,b");
        let result = EnvUtils::get_env_as_list("EMPTY_ITEMS", vec![]);
        assert_eq!(result, vec!["a", "b"]);

        assert_eq!(
            EnvUtils::get_env_as_list("MISSING_LIST", vec!["default".to_string()]),
            vec!["default"]
        );

        EnvUtils::remove_env("LIST_VAR");
        EnvUtils::remove_env("LIST_WITH_SPACES");
        EnvUtils::remove_env("SINGLE_ITEM");
        EnvUtils::remove_env("EMPTY_ITEMS");
    }

    #[test]
    fn test_env_utils_get_with_prefix() {
        EnvUtils::set_env("PREFIX_VAR1", "value1");
        EnvUtils::set_env("PREFIX_VAR2", "value2");
        EnvUtils::set_env("OTHER_VAR", "other");

        let prefixed = EnvUtils::get_env_with_prefix("PREFIX_");
        assert!(prefixed.contains_key("VAR1"));
        assert!(prefixed.contains_key("VAR2"));
        assert!(!prefixed.contains_key("OTHER_VAR"));

        EnvUtils::remove_env("PREFIX_VAR1");
        EnvUtils::remove_env("PREFIX_VAR2");
        EnvUtils::remove_env("OTHER_VAR");
    }

    // ==================== ConfigFileUtils Tests ====================

    #[test]
    fn test_file_exists() {
        assert!(ConfigFileUtils::file_exists("Cargo.toml"));
        assert!(!ConfigFileUtils::file_exists("nonexistent_file.txt"));
    }

    #[test]
    fn test_get_file_extension() {
        assert_eq!(
            ConfigFileUtils::get_file_extension("config.yaml"),
            Some("yaml".to_string())
        );
        assert_eq!(
            ConfigFileUtils::get_file_extension("config.JSON"),
            Some("json".to_string())
        );
        assert_eq!(
            ConfigFileUtils::get_file_extension("config.YML"),
            Some("yml".to_string())
        );
        assert_eq!(ConfigFileUtils::get_file_extension("noextension"), None);
        assert_eq!(
            ConfigFileUtils::get_file_extension("path/to/file.toml"),
            Some("toml".to_string())
        );
    }

    #[test]
    fn test_find_config_file() {
        // This file exists in the repo
        let found = ConfigFileUtils::find_config_file("Cargo.toml");
        assert!(found.is_some());

        let not_found = ConfigFileUtils::find_config_file("nonexistent_config.yaml");
        assert!(not_found.is_none());
    }

    // ==================== ConfigValidator Tests ====================

    #[test]
    fn test_validate_url() {
        assert!(ConfigValidator::validate_url("https://api.openai.com").is_ok());
        assert!(ConfigValidator::validate_url("http://localhost:8080").is_ok());
        assert!(ConfigValidator::validate_url("ftp://files.example.com").is_ok());
        assert!(ConfigValidator::validate_url("invalid-url").is_err());
        assert!(ConfigValidator::validate_url("").is_err());
    }

    #[test]
    fn test_validate_email() {
        assert!(ConfigValidator::validate_email("test@example.com").is_ok());
        assert!(ConfigValidator::validate_email("user.name@domain.co.uk").is_ok());
        assert!(ConfigValidator::validate_email("user+tag@example.com").is_ok());
        assert!(ConfigValidator::validate_email("invalid-email").is_err());
        assert!(ConfigValidator::validate_email("@nodomain.com").is_err());
        assert!(ConfigValidator::validate_email("noat.com").is_err());
    }

    #[test]
    fn test_validate_port() {
        assert!(ConfigValidator::validate_port(8080).is_ok());
        assert!(ConfigValidator::validate_port(1).is_ok());
        assert!(ConfigValidator::validate_port(65535).is_ok());
        assert!(ConfigValidator::validate_port(0).is_err());
    }

    #[test]
    fn test_validate_positive_int() {
        assert!(ConfigValidator::validate_positive_int(1, "count").is_ok());
        assert!(ConfigValidator::validate_positive_int(100, "count").is_ok());
        assert!(ConfigValidator::validate_positive_int(0, "count").is_err());
        assert!(ConfigValidator::validate_positive_int(-1, "count").is_err());
    }

    #[test]
    fn test_validate_range() {
        assert!(ConfigValidator::validate_range(5, 1, 10, "value").is_ok());
        assert!(ConfigValidator::validate_range(1, 1, 10, "value").is_ok());
        assert!(ConfigValidator::validate_range(10, 1, 10, "value").is_ok());
        assert!(ConfigValidator::validate_range(0, 1, 10, "value").is_err());
        assert!(ConfigValidator::validate_range(11, 1, 10, "value").is_err());

        // Test with floats
        assert!(ConfigValidator::validate_range(0.5, 0.0, 1.0, "ratio").is_ok());
        assert!(ConfigValidator::validate_range(1.5, 0.0, 1.0, "ratio").is_err());
    }

    #[test]
    fn test_validate_string_length() {
        assert!(ConfigValidator::validate_string_length("hello", 1, 10, "name").is_ok());
        assert!(ConfigValidator::validate_string_length("a", 1, 10, "name").is_ok());
        assert!(ConfigValidator::validate_string_length("", 1, 10, "name").is_err());
        assert!(
            ConfigValidator::validate_string_length("this is too long", 1, 10, "name").is_err()
        );
    }

    #[test]
    fn test_validate_required() {
        let some_value: Option<i32> = Some(42);
        let none_value: Option<i32> = None;

        assert!(ConfigValidator::validate_required(&some_value, "field").is_ok());
        assert!(ConfigValidator::validate_required(&none_value, "field").is_err());
    }

    #[test]
    fn test_validate_non_empty() {
        assert!(ConfigValidator::validate_non_empty("hello", "field").is_ok());
        assert!(ConfigValidator::validate_non_empty("  content  ", "field").is_ok());
        assert!(ConfigValidator::validate_non_empty("", "field").is_err());
        assert!(ConfigValidator::validate_non_empty("   ", "field").is_err());
    }

    #[test]
    fn test_validate_alphanumeric() {
        assert!(ConfigValidator::validate_alphanumeric("hello123", "id").is_ok());
        assert!(ConfigValidator::validate_alphanumeric("hello_world", "id").is_ok());
        assert!(ConfigValidator::validate_alphanumeric("hello-world", "id").is_ok());
        assert!(ConfigValidator::validate_alphanumeric("hello_world-123", "id").is_ok());
        assert!(ConfigValidator::validate_alphanumeric("hello@world", "id").is_err());
        assert!(ConfigValidator::validate_alphanumeric("hello world", "id").is_err());
        assert!(ConfigValidator::validate_alphanumeric("hello.world", "id").is_err());
    }

    #[test]
    fn test_validate_json() {
        assert!(ConfigValidator::validate_json(r#"{"key": "value"}"#).is_ok());
        assert!(ConfigValidator::validate_json(r#"[1, 2, 3]"#).is_ok());
        assert!(ConfigValidator::validate_json(r#"null"#).is_ok());
        assert!(ConfigValidator::validate_json(r#""string""#).is_ok());
        assert!(ConfigValidator::validate_json(r#"123"#).is_ok());
        assert!(ConfigValidator::validate_json("invalid json").is_err());
        assert!(ConfigValidator::validate_json(r#"{"key": }"#).is_err());
    }

    #[test]
    fn test_validate_duration_string_seconds() {
        let result = ConfigValidator::validate_duration_string("30s").unwrap();
        assert_eq!(result, Duration::from_secs(30));

        let result = ConfigValidator::validate_duration_string("1s").unwrap();
        assert_eq!(result, Duration::from_secs(1));
    }

    #[test]
    fn test_validate_duration_string_minutes() {
        let result = ConfigValidator::validate_duration_string("5m").unwrap();
        assert_eq!(result, Duration::from_secs(300));

        let result = ConfigValidator::validate_duration_string("1m").unwrap();
        assert_eq!(result, Duration::from_secs(60));
    }

    #[test]
    fn test_validate_duration_string_hours() {
        let result = ConfigValidator::validate_duration_string("1h").unwrap();
        assert_eq!(result, Duration::from_secs(3600));

        let result = ConfigValidator::validate_duration_string("24h").unwrap();
        assert_eq!(result, Duration::from_secs(86400));
    }

    #[test]
    fn test_validate_duration_string_days() {
        let result = ConfigValidator::validate_duration_string("1d").unwrap();
        assert_eq!(result, Duration::from_secs(86400));

        let result = ConfigValidator::validate_duration_string("7d").unwrap();
        assert_eq!(result, Duration::from_secs(604800));
    }

    #[test]
    fn test_validate_duration_string_invalid() {
        assert!(ConfigValidator::validate_duration_string("invalid").is_err());
        assert!(ConfigValidator::validate_duration_string("30").is_err());
        assert!(ConfigValidator::validate_duration_string("s30").is_err());
        assert!(ConfigValidator::validate_duration_string("30x").is_err());
        assert!(ConfigValidator::validate_duration_string("").is_err());
    }

    #[test]
    fn test_validate_duration_string_zero() {
        let result = ConfigValidator::validate_duration_string("0s").unwrap();
        assert_eq!(result, Duration::from_secs(0));
    }

    // ==================== ConfigFileUtils Async Tests ====================

    #[tokio::test]
    async fn test_read_file() {
        // Read an existing file
        let result = ConfigFileUtils::read_file("Cargo.toml").await;
        assert!(result.is_ok());
        let content = result.unwrap();
        assert!(content.contains("[package]"));
    }

    #[tokio::test]
    async fn test_read_file_not_found() {
        let result = ConfigFileUtils::read_file("nonexistent_file.txt").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_write_and_read_file() {
        let temp_path = "/tmp/test_config_write.txt";
        let content = "test content\nline 2";

        let write_result = ConfigFileUtils::write_file(temp_path, content).await;
        assert!(write_result.is_ok());

        let read_result = ConfigFileUtils::read_file(temp_path).await;
        assert!(read_result.is_ok());
        assert_eq!(read_result.unwrap(), content);

        // Cleanup
        let _ = tokio::fs::remove_file(temp_path).await;
    }

    #[tokio::test]
    async fn test_write_file_creates_directories() {
        let temp_path = "/tmp/test_nested/dir/config.txt";
        let content = "nested content";

        let write_result = ConfigFileUtils::write_file(temp_path, content).await;
        assert!(write_result.is_ok());

        let read_result = ConfigFileUtils::read_file(temp_path).await;
        assert!(read_result.is_ok());

        // Cleanup
        let _ = tokio::fs::remove_dir_all("/tmp/test_nested").await;
    }

    #[tokio::test]
    async fn test_parse_json_file() {
        let temp_path = "/tmp/test_config.json";
        let json_content = r#"{"name": "test", "value": 42}"#;

        ConfigFileUtils::write_file(temp_path, json_content)
            .await
            .unwrap();

        #[derive(serde::Deserialize)]
        struct TestConfig {
            name: String,
            value: i32,
        }

        let result: Result<TestConfig> = ConfigFileUtils::parse_json_file(temp_path).await;
        assert!(result.is_ok());
        let config = result.unwrap();
        assert_eq!(config.name, "test");
        assert_eq!(config.value, 42);

        // Cleanup
        let _ = tokio::fs::remove_file(temp_path).await;
    }

    #[tokio::test]
    async fn test_parse_yaml_file() {
        let temp_path = "/tmp/test_config.yaml";
        let yaml_content = "name: test\nvalue: 42";

        ConfigFileUtils::write_file(temp_path, yaml_content)
            .await
            .unwrap();

        #[derive(serde::Deserialize)]
        struct TestConfig {
            name: String,
            value: i32,
        }

        let result: Result<TestConfig> = ConfigFileUtils::parse_yaml_file(temp_path).await;
        assert!(result.is_ok());
        let config = result.unwrap();
        assert_eq!(config.name, "test");
        assert_eq!(config.value, 42);

        // Cleanup
        let _ = tokio::fs::remove_file(temp_path).await;
    }

    #[tokio::test]
    async fn test_write_json_file() {
        let temp_path = "/tmp/test_write_config.json";

        #[derive(serde::Serialize)]
        struct TestConfig {
            name: String,
            value: i32,
        }

        let config = TestConfig {
            name: "test".to_string(),
            value: 123,
        };

        let result = ConfigFileUtils::write_json_file(temp_path, &config).await;
        assert!(result.is_ok());

        let content = ConfigFileUtils::read_file(temp_path).await.unwrap();
        assert!(content.contains("\"name\""));
        assert!(content.contains("test"));

        // Cleanup
        let _ = tokio::fs::remove_file(temp_path).await;
    }

    #[tokio::test]
    async fn test_write_yaml_file() {
        let temp_path = "/tmp/test_write_config.yaml";

        #[derive(serde::Serialize)]
        struct TestConfig {
            name: String,
            value: i32,
        }

        let config = TestConfig {
            name: "test".to_string(),
            value: 456,
        };

        let result = ConfigFileUtils::write_yaml_file(temp_path, &config).await;
        assert!(result.is_ok());

        let content = ConfigFileUtils::read_file(temp_path).await.unwrap();
        assert!(content.contains("name:"));
        assert!(content.contains("test"));

        // Cleanup
        let _ = tokio::fs::remove_file(temp_path).await;
    }
}
