//! Configuration loading utilities
//!
//! This module provides utilities for loading configuration from various sources.

use super::models::*;
use crate::utils::error::error::{GatewayError, Result};
use std::env;
use std::collections::HashMap;
use tracing::{debug, warn};

impl GatewayConfig {
    /// Load configuration from environment variables
    pub fn from_env() -> Result<Self> {
        debug!("Loading configuration from environment variables");
        
        let mut config = Self;
        
        // Server configuration
        if let Ok(host) = env::var("GATEWAY_HOST") {
            config.server.host = host;
        }
        if let Ok(port) = env::var("GATEWAY_PORT") {
            config.server.port = port.parse()
                .map_err(|e| GatewayError::Config(format!("Invalid port: {}", e)))?;
        }
        if let Ok(workers) = env::var("GATEWAY_WORKERS") {
            config.server.workers = Some(workers.parse()
                .map_err(|e| GatewayError::Config(format!("Invalid workers count: {}", e)))?);
        }
        if let Ok(timeout) = env::var("GATEWAY_TIMEOUT") {
            config.server.timeout = timeout.parse()
                .map_err(|e| GatewayError::Config(format!("Invalid timeout: {}", e)))?;
        }
        
        // Database configuration
        if let Ok(db_url) = env::var("DATABASE_URL") {
            config.storage.database.url = db_url;
        }
        if let Ok(max_conn) = env::var("DATABASE_MAX_CONNECTIONS") {
            config.storage.database.max_connections = max_conn.parse()
                .map_err(|e| GatewayError::Config(format!("Invalid max connections: {}", e)))?;
        }
        
        // Redis configuration
        if let Ok(redis_url) = env::var("REDIS_URL") {
            config.storage.redis.url = redis_url;
        }
        if let Ok(redis_cluster) = env::var("REDIS_CLUSTER") {
            config.storage.redis.cluster = redis_cluster.parse()
                .map_err(|e| GatewayError::Config(format!("Invalid redis cluster flag: {}", e)))?;
        }
        
        // Auth configuration
        if let Ok(jwt_secret) = env::var("JWT_SECRET") {
            config.auth.jwt_secret = jwt_secret;
        }
        if let Ok(jwt_exp) = env::var("JWT_EXPIRATION") {
            config.auth.jwt_expiration = jwt_exp.parse()
                .map_err(|e| GatewayError::Config(format!("Invalid JWT expiration: {}", e)))?;
        }
        
        // Monitoring configuration
        if let Ok(metrics_port) = env::var("METRICS_PORT") {
            config.monitoring.metrics.port = metrics_port.parse()
                .map_err(|e| GatewayError::Config(format!("Invalid metrics port: {}", e)))?;
        }
        if let Ok(jaeger_endpoint) = env::var("JAEGER_ENDPOINT") {
            config.monitoring.tracing.jaeger_endpoint = Some(jaeger_endpoint);
            config.monitoring.tracing.enabled = true;
        }
        
        // Load providers from environment
        config.providers = load_providers_from_env()?;
        
        debug!("Configuration loaded from environment variables");
        Ok(config)
    }
}

/// Load provider configurations from environment variables
fn load_providers_from_env() -> Result<Vec<ProviderConfig>> {
    let mut providers = Vec::new();
    
    // Look for provider configurations in environment variables
    // Format: PROVIDER_<NAME>_<FIELD>=value
    let mut provider_configs: HashMap<String, HashMap<String, String>> = HashMap::new();
    
    for (key, value) in env::vars() {
        if key.starts_with("PROVIDER_") {
            let parts: Vec<&str> = key.splitn(3, '_').collect();
            if parts.len() == 3 {
                let provider_name = parts[1].to_lowercase();
                let field_name = parts[2].to_lowercase();
                
                provider_configs
                    .entry(provider_name)
                    .or_insert_with(HashMap::new)
                    .insert(field_name, value);
            }
        }
    }
    
    // Convert to ProviderConfig structs
    for (name, fields) in provider_configs {
        let provider_type = fields.get("type")
            .ok_or_else(|| GatewayError::Config(format!("Provider {} missing type", name)))?
            .clone();
        
        let api_key = fields.get("api_key")
            .ok_or_else(|| GatewayError::Config(format!("Provider {} missing api_key", name)))?
            .clone();
        
        let provider = ProviderConfig {
            name: name.clone(),
            provider_type,
            api_key,
            api_base: fields.get("api_base").cloned(),
            api_version: fields.get("api_version").cloned(),
            timeout: fields.get("timeout").and_then(|t| t.parse().ok()),
            max_retries: fields.get("max_retries")
                .and_then(|r| r.parse().ok())
                .unwrap_or(3),
            weight: fields.get("weight")
                .and_then(|w| w.parse().ok())
                .unwrap_or(1.0),
            tags: fields.get("tags")
                .map(|t| t.split(',').map(|s| s.trim().to_string()).collect())
                .unwrap_or_default(),
            headers: HashMap::new(),
            rate_limits: None,
            cost: None,
        };
        
        providers.push(provider);
    }
    
    if providers.is_empty() {
        warn!("No providers configured in environment variables");
    } else {
        debug!("Loaded {} providers from environment", providers.len());
    }
    
    Ok(providers)
}

/// Merge configuration from multiple sources
pub fn merge_configs(base: GatewayConfig, overrides: Vec<GatewayConfig>) -> GatewayConfig {
    overrides.into_iter().fold(base, |acc, config| acc.merge(config))
}

/// Load configuration with precedence: file -> env -> cli args
pub async fn load_config_with_precedence(
    config_file: Option<&str>,
    env_override: bool,
) -> Result<GatewayConfig> {
    let mut configs = Vec::new();
    
    // 1. Load from file if provided
    if let Some(file_path) = config_file {
        match tokio::fs::read_to_string(file_path).await {
            Ok(content) => {
                let file_config: GatewayConfig = serde_yaml::from_str(&content)
                    .map_err(|e| GatewayError::Config(format!("Failed to parse config file: {}", e)))?;
                configs.push(file_config);
                debug!("Loaded configuration from file: {}", file_path);
            }
            Err(e) => {
                warn!("Failed to load config file {}: {}", file_path, e);
            }
        }
    }
    
    // 2. Load from environment if enabled
    if env_override {
        match GatewayConfig::from_env() {
            Ok(env_config) => {
                configs.push(env_config);
                debug!("Loaded configuration from environment variables");
            }
            Err(e) => {
                warn!("Failed to load config from environment: {}", e);
            }
        }
    }
    
    // 3. Start with default config and merge others
    let base_config = GatewayConfig::default();
    let final_config = merge_configs(base_config, configs);
    
    Ok(final_config)
}

/// Expand environment variables in configuration strings
pub fn expand_env_vars(input: &str) -> String {
    let mut result = input.to_string();
    
    // Simple environment variable expansion: ${VAR_NAME} or $VAR_NAME
    for (key, value) in env::vars() {
        let patterns = [
            format!("${{{}}}", key),
            format!("${}", key),
        ];
        
        for pattern in &patterns {
            result = result.replace(pattern, &value);
        }
    }
    
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_expand_env_vars() {
        env::set_var("TEST_VAR", "test_value");
        
        let input = "Database URL: ${TEST_VAR}/database";
        let result = expand_env_vars(input);
        assert_eq!(result, "Database URL: test_value/database");
        
        let input2 = "API Key: $TEST_VAR";
        let result2 = expand_env_vars(input2);
        assert_eq!(result2, "API Key: test_value");
        
        env::remove_var("TEST_VAR");
    }

    #[test]
    fn test_merge_configs() {
        let base = GatewayConfig::default();
        let mut override_config = GatewayConfig::default();
        override_config.server.port = 9000;
        override_config.server.host = "127.0.0.1".to_string();
        
        let merged = merge_configs(base, vec![override_config]);
        
        assert_eq!(merged.server.port, 9000);
        assert_eq!(merged.server.host, "127.0.0.1");
    }

    #[tokio::test]
    async fn test_load_providers_from_env() {
        env::set_var("PROVIDER_OPENAI_TYPE", "openai");
        env::set_var("PROVIDER_OPENAI_API_KEY", "test-key");
        env::set_var("PROVIDER_OPENAI_API_BASE", "https://api.openai.com/v1");
        
        let providers = load_providers_from_env().unwrap();
        assert_eq!(providers.len(), 1);
        assert_eq!(providers[0].name, "openai");
        assert_eq!(providers[0].provider_type, "openai");
        assert_eq!(providers[0].api_key, "test-key");
        
        env::remove_var("PROVIDER_OPENAI_TYPE");
        env::remove_var("PROVIDER_OPENAI_API_KEY");
        env::remove_var("PROVIDER_OPENAI_API_BASE");
    }
}
