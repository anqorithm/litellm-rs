//! GitHub Copilot Provider Configuration
//!
//! Configuration for GitHub Copilot API access including authentication and model settings.

use crate::core::traits::provider::ProviderConfig;
use serde::{Deserialize, Serialize};

/// Default API base URL for GitHub Copilot
pub const GITHUB_COPILOT_API_BASE: &str = "https://api.githubcopilot.com";

/// Copilot version for headers
pub const COPILOT_VERSION: &str = "0.26.7";

/// Editor plugin version for headers
pub const EDITOR_PLUGIN_VERSION: &str = "copilot-chat/0.26.7";

/// User agent for headers
pub const USER_AGENT: &str = "GitHubCopilotChat/0.26.7";

/// API version for headers
pub const API_VERSION: &str = "2025-04-01";

/// GitHub Copilot provider configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubCopilotConfig {
    /// Token directory for storing access tokens
    pub token_dir: Option<String>,

    /// Access token file name
    pub access_token_file: Option<String>,

    /// API key file name
    pub api_key_file: Option<String>,

    /// API base URL (default: <https://api.githubcopilot.com>)
    pub api_base: Option<String>,

    /// Request timeout in seconds
    #[serde(default = "default_timeout")]
    pub timeout: u64,

    /// Maximum number of retries for failed requests
    #[serde(default = "default_max_retries")]
    pub max_retries: u32,

    /// Whether to disable converting system messages to assistant messages
    #[serde(default)]
    pub disable_system_to_assistant: bool,

    /// Whether to enable debug logging
    #[serde(default)]
    pub debug: bool,
}

impl Default for GitHubCopilotConfig {
    fn default() -> Self {
        Self {
            token_dir: None,
            access_token_file: None,
            api_key_file: None,
            api_base: None,
            timeout: default_timeout(),
            max_retries: default_max_retries(),
            disable_system_to_assistant: false,
            debug: false,
        }
    }
}

impl ProviderConfig for GitHubCopilotConfig {
    fn validate(&self) -> Result<(), String> {
        // Validation is done during authentication
        // No API key is needed upfront - it's obtained via OAuth

        // Validate timeout
        if self.timeout == 0 {
            return Err("Timeout must be greater than 0".to_string());
        }

        Ok(())
    }

    fn api_key(&self) -> Option<&str> {
        // API key is obtained dynamically via OAuth
        None
    }

    fn api_base(&self) -> Option<&str> {
        self.api_base.as_deref()
    }

    fn timeout(&self) -> std::time::Duration {
        std::time::Duration::from_secs(self.timeout)
    }

    fn max_retries(&self) -> u32 {
        self.max_retries
    }
}

impl GitHubCopilotConfig {
    /// Get token directory with environment variable fallback
    pub fn get_token_dir(&self) -> String {
        self.token_dir
            .clone()
            .or_else(|| std::env::var("GITHUB_COPILOT_TOKEN_DIR").ok())
            .unwrap_or_else(|| {
                let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
                format!("{}/.config/litellm/github_copilot", home)
            })
    }

    /// Get access token file name with environment variable fallback
    pub fn get_access_token_file(&self) -> String {
        self.access_token_file
            .clone()
            .or_else(|| std::env::var("GITHUB_COPILOT_ACCESS_TOKEN_FILE").ok())
            .unwrap_or_else(|| "access-token".to_string())
    }

    /// Get API key file name with environment variable fallback
    pub fn get_api_key_file(&self) -> String {
        self.api_key_file
            .clone()
            .or_else(|| std::env::var("GITHUB_COPILOT_API_KEY_FILE").ok())
            .unwrap_or_else(|| "api-key.json".to_string())
    }

    /// Get API base with environment variable fallback
    pub fn get_api_base(&self) -> String {
        self.api_base
            .clone()
            .or_else(|| std::env::var("GITHUB_COPILOT_API_BASE").ok())
            .unwrap_or_else(|| GITHUB_COPILOT_API_BASE.to_string())
    }
}

/// Get default headers for GitHub Copilot API
pub fn get_copilot_default_headers(api_key: &str) -> std::collections::HashMap<String, String> {
    let mut headers = std::collections::HashMap::new();
    headers.insert("Authorization".to_string(), format!("Bearer {}", api_key));
    headers.insert("Content-Type".to_string(), "application/json".to_string());
    headers.insert(
        "copilot-integration-id".to_string(),
        "vscode-chat".to_string(),
    );
    headers.insert("editor-version".to_string(), "vscode/1.95.0".to_string());
    headers.insert(
        "editor-plugin-version".to_string(),
        EDITOR_PLUGIN_VERSION.to_string(),
    );
    headers.insert("user-agent".to_string(), USER_AGENT.to_string());
    headers.insert(
        "openai-intent".to_string(),
        "conversation-panel".to_string(),
    );
    headers.insert("x-github-api-version".to_string(), API_VERSION.to_string());
    headers.insert("x-request-id".to_string(), uuid::Uuid::new_v4().to_string());
    headers.insert(
        "x-vscode-user-agent-library-version".to_string(),
        "electron-fetch".to_string(),
    );
    headers
}

fn default_timeout() -> u64 {
    60
}

fn default_max_retries() -> u32 {
    3
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_github_copilot_config_default() {
        let config = GitHubCopilotConfig::default();
        assert!(config.token_dir.is_none());
        assert!(config.access_token_file.is_none());
        assert!(config.api_key_file.is_none());
        assert!(config.api_base.is_none());
        assert_eq!(config.timeout, 60);
        assert_eq!(config.max_retries, 3);
        assert!(!config.disable_system_to_assistant);
        assert!(!config.debug);
    }

    #[test]
    fn test_github_copilot_config_get_api_base_default() {
        let config = GitHubCopilotConfig::default();
        assert_eq!(config.get_api_base(), "https://api.githubcopilot.com");
    }

    #[test]
    fn test_github_copilot_config_get_api_base_custom() {
        let config = GitHubCopilotConfig {
            api_base: Some("https://custom.copilot.com".to_string()),
            ..Default::default()
        };
        assert_eq!(config.get_api_base(), "https://custom.copilot.com");
    }

    #[test]
    fn test_github_copilot_config_provider_config_trait() {
        let config = GitHubCopilotConfig {
            api_base: Some("https://custom.copilot.com".to_string()),
            timeout: 60,
            max_retries: 5,
            ..Default::default()
        };

        assert!(config.api_key().is_none()); // No static API key
        assert_eq!(config.api_base(), Some("https://custom.copilot.com"));
        assert_eq!(config.timeout(), std::time::Duration::from_secs(60));
        assert_eq!(config.max_retries(), 5);
    }

    #[test]
    fn test_github_copilot_config_validation() {
        let config = GitHubCopilotConfig::default();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_github_copilot_config_validation_zero_timeout() {
        let config = GitHubCopilotConfig {
            timeout: 0,
            ..Default::default()
        };
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_github_copilot_config_serialization() {
        let config = GitHubCopilotConfig {
            token_dir: Some("/custom/path".to_string()),
            api_base: Some("https://custom.copilot.com".to_string()),
            timeout: 45,
            max_retries: 2,
            disable_system_to_assistant: true,
            debug: true,
            ..Default::default()
        };

        let json = serde_json::to_value(&config).unwrap();
        assert_eq!(json["token_dir"], "/custom/path");
        assert_eq!(json["api_base"], "https://custom.copilot.com");
        assert_eq!(json["timeout"], 45);
        assert_eq!(json["max_retries"], 2);
        assert!(json["disable_system_to_assistant"].as_bool().unwrap());
        assert!(json["debug"].as_bool().unwrap());
    }

    #[test]
    fn test_github_copilot_config_deserialization() {
        let json = r#"{
            "api_base": "https://custom.copilot.com",
            "timeout": 60,
            "debug": true
        }"#;

        let config: GitHubCopilotConfig = serde_json::from_str(json).unwrap();
        assert_eq!(
            config.api_base,
            Some("https://custom.copilot.com".to_string())
        );
        assert_eq!(config.timeout, 60);
        assert!(config.debug);
    }

    #[test]
    fn test_get_copilot_default_headers() {
        let headers = get_copilot_default_headers("test-api-key");

        assert!(
            headers
                .get("Authorization")
                .unwrap()
                .contains("Bearer test-api-key")
        );
        assert_eq!(headers.get("Content-Type").unwrap(), "application/json");
        assert_eq!(
            headers.get("copilot-integration-id").unwrap(),
            "vscode-chat"
        );
        assert!(headers.contains_key("x-request-id"));
    }
}
