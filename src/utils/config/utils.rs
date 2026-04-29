use crate::core::providers::unified_provider::ProviderError;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ConfigManager {
    pub config_path: Option<String>,
    pub env_vars: HashMap<String, String>,
}

pub struct ConfigUtils;

impl ConfigUtils {
    pub fn read_config_args(
        config_path: &str,
    ) -> Result<HashMap<String, serde_yml::Value>, ProviderError> {
        if !Path::new(config_path).exists() {
            return Err(ProviderError::InvalidRequest {
                provider: "config",
                message: format!("Config file not found: {}", config_path),
            });
        }

        let content =
            fs::read_to_string(config_path).map_err(|e| ProviderError::InvalidRequest {
                provider: "config",
                message: format!("Failed to read config file: {}", e),
            })?;

        let config: HashMap<String, serde_yml::Value> =
            serde_yml::from_str(&content).map_err(|e| ProviderError::InvalidRequest {
                provider: "config",
                message: format!("Failed to parse YAML config: {}", e),
            })?;

        Ok(config)
    }

    pub fn get_env_var(key: &str) -> Option<String> {
        env::var(key).ok().filter(|v| !v.is_empty())
    }

    pub fn get_env_var_with_default(key: &str, default_value: &str) -> String {
        Self::get_env_var(key).unwrap_or_else(|| default_value.to_string())
    }

    pub fn set_env_var(key: &str, value: &str) {
        unsafe {
            env::set_var(key, value);
        }
    }

    pub fn load_dotenv() -> Result<(), ProviderError> {
        dotenvy::dotenv().map_err(|e| ProviderError::InvalidRequest {
            provider: "config",
            message: format!("Failed to load .env file: {}", e),
        })?;
        Ok(())
    }

    pub fn get_bool_config(key: &str, default: bool) -> bool {
        if let Ok(value) = env::var(key) {
            match value.to_lowercase().as_str() {
                "true" | "1" | "yes" | "on" => true,
                "false" | "0" | "no" | "off" => false,
                _ => default,
            }
        } else {
            default
        }
    }

    pub fn get_numeric_config<T>(key: &str, default: T) -> T
    where
        T: std::str::FromStr + Clone,
    {
        if let Ok(value) = env::var(key) {
            value.parse().unwrap_or(default)
        } else {
            default
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigDefaults {
    pub timeout: u64,
    pub max_retries: u32,
    pub retry_delay: u64,
    pub max_tokens: Option<u32>,
    pub temperature: Option<f32>,
}

impl Default for ConfigDefaults {
    fn default() -> Self {
        Self {
            timeout: 60,
            max_retries: 3,
            retry_delay: 1000,
            max_tokens: None,
            temperature: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_env_var_with_default() {
        unsafe {
            env::set_var("TEST_VAR", "test_value");
        }
        assert_eq!(
            ConfigUtils::get_env_var_with_default("TEST_VAR", "default"),
            "test_value"
        );
        assert_eq!(
            ConfigUtils::get_env_var_with_default("NONEXISTENT_VAR", "default"),
            "default"
        );
        unsafe {
            env::remove_var("TEST_VAR");
        }
    }

    #[test]
    fn test_get_bool_config() {
        unsafe {
            env::set_var("BOOL_TRUE", "true");
            env::set_var("BOOL_FALSE", "false");
            env::set_var("BOOL_INVALID", "invalid");
        }

        assert!(ConfigUtils::get_bool_config("BOOL_TRUE", false));
        assert!(!ConfigUtils::get_bool_config("BOOL_FALSE", true));
        assert!(ConfigUtils::get_bool_config("BOOL_INVALID", true));
        assert!(!ConfigUtils::get_bool_config("NONEXISTENT", false));

        unsafe {
            env::remove_var("BOOL_TRUE");
            env::remove_var("BOOL_FALSE");
            env::remove_var("BOOL_INVALID");
        }
    }
}
