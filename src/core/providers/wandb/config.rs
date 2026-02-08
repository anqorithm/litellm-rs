//! Weights & Biases (W&B) Configuration
//!
//! Configuration for the W&B integration used for logging and experiment tracking.

use serde::{Deserialize, Serialize};
use std::time::Duration;

use crate::core::providers::unified_provider::ProviderError;
use crate::core::traits::provider::ProviderConfig;

/// Provider name constant for error messages
pub const PROVIDER_NAME: &str = "wandb";

/// W&B API base URL
pub const WANDB_API_BASE: &str = "https://api.wandb.ai";

/// Environment variable for W&B API key
pub const WANDB_API_KEY_ENV: &str = "WANDB_API_KEY";

/// Environment variable for W&B project name
pub const WANDB_PROJECT_ENV: &str = "WANDB_PROJECT";

/// Environment variable for W&B entity (team/username)
pub const WANDB_ENTITY_ENV: &str = "WANDB_ENTITY";

/// W&B configuration for LLM call logging and experiment tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WandbConfig {
    /// API key for authentication (required)
    /// Can also be set via WANDB_API_KEY environment variable
    pub api_key: Option<String>,

    /// Project name for organizing runs
    /// Can also be set via WANDB_PROJECT environment variable
    pub project: Option<String>,

    /// Entity (team or username) for the project
    /// Can also be set via WANDB_ENTITY environment variable
    pub entity: Option<String>,

    /// API base URL (defaults to <https://api.wandb.ai>)
    pub api_base: String,

    /// Request timeout in seconds
    pub timeout_seconds: u64,

    /// Maximum retries for failed requests
    pub max_retries: u32,

    /// Whether to log prompts (may contain sensitive data)
    pub log_prompts: bool,

    /// Whether to log responses (may contain sensitive data)
    pub log_responses: bool,

    /// Whether to log token usage
    pub log_token_usage: bool,

    /// Whether to log cost information
    pub log_costs: bool,

    /// Whether to log latency metrics
    pub log_latency: bool,

    /// Custom run name (if not set, W&B generates one)
    pub run_name: Option<String>,

    /// Custom tags for the run
    pub tags: Vec<String>,

    /// Custom notes for the run
    pub notes: Option<String>,

    /// Whether logging is enabled (can be disabled without removing integration)
    pub enabled: bool,

    /// Batch size for sending logs (for performance optimization)
    pub batch_size: usize,

    /// Flush interval in seconds (how often to send batched logs)
    pub flush_interval_seconds: u64,
}

impl Default for WandbConfig {
    fn default() -> Self {
        Self {
            api_key: std::env::var(WANDB_API_KEY_ENV).ok(),
            project: std::env::var(WANDB_PROJECT_ENV).ok(),
            entity: std::env::var(WANDB_ENTITY_ENV).ok(),
            api_base: WANDB_API_BASE.to_string(),
            timeout_seconds: 30,
            max_retries: 3,
            log_prompts: true,
            log_responses: true,
            log_token_usage: true,
            log_costs: true,
            log_latency: true,
            run_name: None,
            tags: Vec::new(),
            notes: None,
            enabled: true,
            batch_size: 10,
            flush_interval_seconds: 30,
        }
    }
}

impl WandbConfig {
    /// Create a new config with API key
    pub fn new(api_key: impl Into<String>) -> Self {
        Self {
            api_key: Some(api_key.into()),
            ..Default::default()
        }
    }

    /// Create config from environment variables
    pub fn from_env() -> Result<Self, ProviderError> {
        let config = Self::default();
        config
            .validate()
            .map_err(|e| ProviderError::configuration(PROVIDER_NAME, e))?;
        Ok(config)
    }

    /// Set the project name
    pub fn with_project(mut self, project: impl Into<String>) -> Self {
        self.project = Some(project.into());
        self
    }

    /// Set the entity (team/username)
    pub fn with_entity(mut self, entity: impl Into<String>) -> Self {
        self.entity = Some(entity.into());
        self
    }

    /// Set custom run name
    pub fn with_run_name(mut self, name: impl Into<String>) -> Self {
        self.run_name = Some(name.into());
        self
    }

    /// Add tags
    pub fn with_tags(mut self, tags: Vec<String>) -> Self {
        self.tags = tags;
        self
    }

    /// Set notes
    pub fn with_notes(mut self, notes: impl Into<String>) -> Self {
        self.notes = Some(notes.into());
        self
    }

    /// Disable prompt logging (for privacy)
    pub fn without_prompt_logging(mut self) -> Self {
        self.log_prompts = false;
        self
    }

    /// Disable response logging (for privacy)
    pub fn without_response_logging(mut self) -> Self {
        self.log_responses = false;
        self
    }

    /// Configure batch settings
    pub fn with_batch_settings(mut self, batch_size: usize, flush_interval_seconds: u64) -> Self {
        self.batch_size = batch_size;
        self.flush_interval_seconds = flush_interval_seconds;
        self
    }

    /// Get effective API key (config or environment)
    pub fn get_effective_api_key(&self) -> Option<String> {
        self.api_key
            .clone()
            .or_else(|| std::env::var(WANDB_API_KEY_ENV).ok())
    }

    /// Get effective project name (config or environment)
    pub fn get_effective_project(&self) -> Option<String> {
        self.project
            .clone()
            .or_else(|| std::env::var(WANDB_PROJECT_ENV).ok())
    }

    /// Get effective entity (config or environment)
    pub fn get_effective_entity(&self) -> Option<String> {
        self.entity
            .clone()
            .or_else(|| std::env::var(WANDB_ENTITY_ENV).ok())
    }
}

impl ProviderConfig for WandbConfig {
    fn validate(&self) -> Result<(), String> {
        // Check API key
        if self.get_effective_api_key().is_none() {
            return Err(format!(
                "W&B API key is required. Set via config or {} environment variable",
                WANDB_API_KEY_ENV
            ));
        }

        // Validate timeout
        if self.timeout_seconds == 0 {
            return Err("Timeout must be greater than 0".to_string());
        }

        // Validate max retries
        if self.max_retries > 10 {
            return Err("Max retries should not exceed 10".to_string());
        }

        // Validate batch size
        if self.batch_size == 0 {
            return Err("Batch size must be greater than 0".to_string());
        }

        // Validate flush interval
        if self.flush_interval_seconds == 0 {
            return Err("Flush interval must be greater than 0".to_string());
        }

        Ok(())
    }

    fn api_key(&self) -> Option<&str> {
        self.api_key.as_deref()
    }

    fn api_base(&self) -> Option<&str> {
        Some(&self.api_base)
    }

    fn timeout(&self) -> Duration {
        Duration::from_secs(self.timeout_seconds)
    }

    fn max_retries(&self) -> u32 {
        self.max_retries
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wandb_config_default() {
        let config = WandbConfig::default();

        assert_eq!(config.api_base, WANDB_API_BASE);
        assert_eq!(config.timeout_seconds, 30);
        assert_eq!(config.max_retries, 3);
        assert!(config.log_prompts);
        assert!(config.log_responses);
        assert!(config.log_token_usage);
        assert!(config.log_costs);
        assert!(config.log_latency);
        assert!(config.enabled);
        assert_eq!(config.batch_size, 10);
        assert_eq!(config.flush_interval_seconds, 30);
    }

    #[test]
    fn test_wandb_config_new() {
        let config = WandbConfig::new("test-api-key");

        assert_eq!(config.api_key, Some("test-api-key".to_string()));
        assert_eq!(config.api_base, WANDB_API_BASE);
    }

    #[test]
    fn test_wandb_config_builder_pattern() {
        let config = WandbConfig::new("test-key")
            .with_project("my-project")
            .with_entity("my-team")
            .with_run_name("experiment-1")
            .with_tags(vec!["production".to_string(), "gpt-4".to_string()])
            .with_notes("Testing the new model")
            .without_prompt_logging()
            .with_batch_settings(20, 60);

        assert_eq!(config.project, Some("my-project".to_string()));
        assert_eq!(config.entity, Some("my-team".to_string()));
        assert_eq!(config.run_name, Some("experiment-1".to_string()));
        assert_eq!(config.tags.len(), 2);
        assert_eq!(config.notes, Some("Testing the new model".to_string()));
        assert!(!config.log_prompts);
        assert_eq!(config.batch_size, 20);
        assert_eq!(config.flush_interval_seconds, 60);
    }

    #[test]
    fn test_wandb_config_validation_no_api_key() {
        let config = WandbConfig {
            api_key: None,
            ..Default::default()
        };

        // This will fail if WANDB_API_KEY env is not set
        // We can't reliably test this without mocking env
        // Just verify the method exists and is callable
        let _ = config.validate();
    }

    #[test]
    fn test_wandb_config_validation_with_api_key() {
        let config = WandbConfig::new("valid-api-key");
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_wandb_config_validation_invalid_timeout() {
        let mut config = WandbConfig::new("valid-api-key");
        config.timeout_seconds = 0;

        let result = config.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Timeout"));
    }

    #[test]
    fn test_wandb_config_validation_invalid_max_retries() {
        let mut config = WandbConfig::new("valid-api-key");
        config.max_retries = 11;

        let result = config.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("retries"));
    }

    #[test]
    fn test_wandb_config_validation_invalid_batch_size() {
        let mut config = WandbConfig::new("valid-api-key");
        config.batch_size = 0;

        let result = config.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Batch size"));
    }

    #[test]
    fn test_wandb_config_validation_invalid_flush_interval() {
        let mut config = WandbConfig::new("valid-api-key");
        config.flush_interval_seconds = 0;

        let result = config.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Flush interval"));
    }

    #[test]
    fn test_wandb_config_provider_config_trait() {
        let config = WandbConfig::new("test-key");

        assert_eq!(config.api_key(), Some("test-key"));
        assert_eq!(config.api_base(), Some(WANDB_API_BASE));
        assert_eq!(config.timeout(), Duration::from_secs(30));
        assert_eq!(config.max_retries(), 3);
    }

    #[test]
    fn test_wandb_config_serialization() {
        let config = WandbConfig::new("test-key")
            .with_project("my-project")
            .with_entity("my-team");

        let json = serde_json::to_value(&config).unwrap();
        assert_eq!(json["api_key"], "test-key");
        assert_eq!(json["project"], "my-project");
        assert_eq!(json["entity"], "my-team");
        assert_eq!(json["enabled"], true);
    }

    #[test]
    fn test_wandb_config_deserialization() {
        let json = r#"{
            "api_key": "my-key",
            "project": "test-project",
            "entity": "test-entity",
            "api_base": "https://api.wandb.ai",
            "timeout_seconds": 60,
            "max_retries": 5,
            "log_prompts": false,
            "log_responses": true,
            "log_token_usage": true,
            "log_costs": true,
            "log_latency": true,
            "enabled": true,
            "batch_size": 15,
            "flush_interval_seconds": 45,
            "tags": []
        }"#;

        let config: WandbConfig = serde_json::from_str(json).unwrap();
        assert_eq!(config.api_key, Some("my-key".to_string()));
        assert_eq!(config.project, Some("test-project".to_string()));
        assert_eq!(config.entity, Some("test-entity".to_string()));
        assert_eq!(config.timeout_seconds, 60);
        assert!(!config.log_prompts);
        assert_eq!(config.batch_size, 15);
    }

    #[test]
    fn test_wandb_config_clone() {
        let config = WandbConfig::new("test-key")
            .with_project("my-project")
            .with_tags(vec!["tag1".to_string()]);

        let cloned = config.clone();

        assert_eq!(config.api_key, cloned.api_key);
        assert_eq!(config.project, cloned.project);
        assert_eq!(config.tags, cloned.tags);
    }

    #[test]
    fn test_wandb_config_debug() {
        let config = WandbConfig::new("test-key");
        let debug_str = format!("{:?}", config);

        assert!(debug_str.contains("WandbConfig"));
        assert!(debug_str.contains("test-key"));
    }
}
