//! Gradient AI Provider Configuration
//!
//! Configuration for Gradient AI API access including authentication and model settings.

use crate::core::traits::provider::ProviderConfig;
use serde::{Deserialize, Serialize};

/// Default API base URL for Gradient AI Serverless
pub const GRADIENT_AI_SERVERLESS_ENDPOINT: &str = "https://inference.do-ai.run";

/// Retrieval method options for Gradient AI
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
#[derive(Default)]
pub enum RetrievalMethod {
    Rewrite,
    StepBack,
    SubQueries,
    #[default]
    None,
}

/// Knowledge base filter for Gradient AI
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KBFilter {
    /// Filter key
    pub key: String,
    /// Filter value
    pub value: serde_json::Value,
    /// Filter operation (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub operation: Option<String>,
}

/// Gradient AI provider configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GradientAIConfig {
    /// API key for Gradient AI authentication
    pub api_key: Option<String>,

    /// API base URL (default: <https://inference.do-ai.run>)
    pub api_base: Option<String>,

    /// Agent endpoint URL (for agent-specific deployments)
    pub agent_endpoint: Option<String>,

    /// Request timeout in seconds
    #[serde(default = "default_timeout")]
    pub timeout: u64,

    /// Maximum number of retries for failed requests
    #[serde(default = "default_max_retries")]
    pub max_retries: u32,

    /// Whether to enable debug logging
    #[serde(default)]
    pub debug: bool,

    // Gradient AI specific parameters
    /// Number of knowledge base chunks to retrieve
    #[serde(skip_serializing_if = "Option::is_none")]
    pub k: Option<i32>,

    /// Knowledge base filters
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kb_filters: Option<Vec<KBFilter>>,

    /// Filter KB content by query metadata
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filter_kb_content_by_query_metadata: Option<bool>,

    /// Instruction override for the model
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instruction_override: Option<String>,

    /// Include functions info in response
    #[serde(skip_serializing_if = "Option::is_none")]
    pub include_functions_info: Option<bool>,

    /// Include retrieval info in response
    #[serde(skip_serializing_if = "Option::is_none")]
    pub include_retrieval_info: Option<bool>,

    /// Include guardrails info in response
    #[serde(skip_serializing_if = "Option::is_none")]
    pub include_guardrails_info: Option<bool>,

    /// Provide citations in response
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provide_citations: Option<bool>,

    /// Retrieval method for knowledge base queries
    #[serde(skip_serializing_if = "Option::is_none")]
    pub retrieval_method: Option<RetrievalMethod>,
}

impl Default for GradientAIConfig {
    fn default() -> Self {
        Self {
            api_key: None,
            api_base: None,
            agent_endpoint: None,
            timeout: default_timeout(),
            max_retries: default_max_retries(),
            debug: false,
            k: None,
            kb_filters: None,
            filter_kb_content_by_query_metadata: None,
            instruction_override: None,
            include_functions_info: None,
            include_retrieval_info: None,
            include_guardrails_info: None,
            provide_citations: None,
            retrieval_method: None,
        }
    }
}

impl ProviderConfig for GradientAIConfig {
    fn validate(&self) -> Result<(), String> {
        // API key can come from environment variable
        if self.api_key.is_none() && std::env::var("GRADIENT_AI_API_KEY").is_err() {
            return Err(
                "Gradient AI API key not provided and GRADIENT_AI_API_KEY environment variable not set"
                    .to_string(),
            );
        }

        // Validate timeout
        if self.timeout == 0 {
            return Err("Timeout must be greater than 0".to_string());
        }

        Ok(())
    }

    fn api_key(&self) -> Option<&str> {
        self.api_key.as_deref()
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

impl GradientAIConfig {
    /// Get API key with environment variable fallback
    pub fn get_api_key(&self) -> Option<String> {
        self.api_key
            .clone()
            .or_else(|| std::env::var("GRADIENT_AI_API_KEY").ok())
    }

    /// Get API base with environment variable fallback
    pub fn get_api_base(&self) -> String {
        self.api_base
            .clone()
            .or_else(|| std::env::var("GRADIENT_AI_API_BASE").ok())
            .unwrap_or_else(|| GRADIENT_AI_SERVERLESS_ENDPOINT.to_string())
    }

    /// Get agent endpoint with environment variable fallback
    pub fn get_agent_endpoint(&self) -> Option<String> {
        self.agent_endpoint
            .clone()
            .or_else(|| std::env::var("GRADIENT_AI_AGENT_ENDPOINT").ok())
    }

    /// Get the complete URL for chat completions
    ///
    /// Uses agent endpoint if available, otherwise uses default serverless endpoint
    pub fn get_complete_url(&self) -> String {
        let agent_endpoint = self.get_agent_endpoint();
        let api_base = self.api_base.clone().unwrap_or_default();

        if !api_base.is_empty() && api_base != GRADIENT_AI_SERVERLESS_ENDPOINT {
            format!("{}/api/v1/chat/completions", api_base)
        } else if let Some(endpoint) = agent_endpoint {
            if !endpoint.is_empty() && endpoint != GRADIENT_AI_SERVERLESS_ENDPOINT {
                format!("{}/api/v1/chat/completions", endpoint)
            } else {
                format!("{}/v1/chat/completions", GRADIENT_AI_SERVERLESS_ENDPOINT)
            }
        } else {
            format!("{}/v1/chat/completions", GRADIENT_AI_SERVERLESS_ENDPOINT)
        }
    }
}

fn default_timeout() -> u64 {
    30
}

fn default_max_retries() -> u32 {
    3
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gradient_ai_config_default() {
        let config = GradientAIConfig::default();
        assert!(config.api_key.is_none());
        assert!(config.api_base.is_none());
        assert!(config.agent_endpoint.is_none());
        assert_eq!(config.timeout, 30);
        assert_eq!(config.max_retries, 3);
        assert!(!config.debug);
        assert!(config.k.is_none());
        assert!(config.kb_filters.is_none());
        assert!(config.retrieval_method.is_none());
    }

    #[test]
    fn test_gradient_ai_config_get_api_base_default() {
        let config = GradientAIConfig::default();
        assert_eq!(config.get_api_base(), GRADIENT_AI_SERVERLESS_ENDPOINT);
    }

    #[test]
    fn test_gradient_ai_config_get_api_base_custom() {
        let config = GradientAIConfig {
            api_base: Some("https://custom.gradient.ai".to_string()),
            ..Default::default()
        };
        assert_eq!(config.get_api_base(), "https://custom.gradient.ai");
    }

    #[test]
    fn test_gradient_ai_config_get_api_key() {
        let config = GradientAIConfig {
            api_key: Some("test-key".to_string()),
            ..Default::default()
        };
        assert_eq!(config.get_api_key(), Some("test-key".to_string()));
    }

    #[test]
    fn test_get_complete_url_default() {
        let config = GradientAIConfig::default();
        let url = config.get_complete_url();
        assert_eq!(
            url,
            format!("{}/v1/chat/completions", GRADIENT_AI_SERVERLESS_ENDPOINT)
        );
    }

    #[test]
    fn test_get_complete_url_with_api_base() {
        let config = GradientAIConfig {
            api_base: Some("https://custom.gradient.ai".to_string()),
            ..Default::default()
        };
        let url = config.get_complete_url();
        assert_eq!(url, "https://custom.gradient.ai/api/v1/chat/completions");
    }

    #[test]
    fn test_get_complete_url_with_agent_endpoint() {
        let config = GradientAIConfig {
            agent_endpoint: Some("https://agent.gradient.ai".to_string()),
            ..Default::default()
        };
        let url = config.get_complete_url();
        assert_eq!(url, "https://agent.gradient.ai/api/v1/chat/completions");
    }

    #[test]
    fn test_gradient_ai_config_provider_config_trait() {
        let config = GradientAIConfig {
            api_key: Some("test-key".to_string()),
            api_base: Some("https://custom.gradient.ai".to_string()),
            timeout: 60,
            max_retries: 5,
            ..Default::default()
        };

        assert_eq!(config.api_key(), Some("test-key"));
        assert_eq!(config.api_base(), Some("https://custom.gradient.ai"));
        assert_eq!(config.timeout(), std::time::Duration::from_secs(60));
        assert_eq!(config.max_retries(), 5);
    }

    #[test]
    fn test_gradient_ai_config_validation_with_key() {
        let config = GradientAIConfig {
            api_key: Some("test-key".to_string()),
            ..Default::default()
        };
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_gradient_ai_config_validation_zero_timeout() {
        let config = GradientAIConfig {
            api_key: Some("test-key".to_string()),
            timeout: 0,
            ..Default::default()
        };
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_retrieval_method_serialization() {
        let method = RetrievalMethod::SubQueries;
        let json = serde_json::to_string(&method).unwrap();
        assert_eq!(json, "\"sub_queries\"");

        let method = RetrievalMethod::StepBack;
        let json = serde_json::to_string(&method).unwrap();
        assert_eq!(json, "\"step_back\"");
    }

    #[test]
    fn test_retrieval_method_deserialization() {
        let method: RetrievalMethod = serde_json::from_str("\"rewrite\"").unwrap();
        assert_eq!(method, RetrievalMethod::Rewrite);

        let method: RetrievalMethod = serde_json::from_str("\"none\"").unwrap();
        assert_eq!(method, RetrievalMethod::None);
    }

    #[test]
    fn test_kb_filter_serialization() {
        let filter = KBFilter {
            key: "category".to_string(),
            value: serde_json::json!("tech"),
            operation: Some("eq".to_string()),
        };

        let json = serde_json::to_value(&filter).unwrap();
        assert_eq!(json["key"], "category");
        assert_eq!(json["value"], "tech");
        assert_eq!(json["operation"], "eq");
    }

    #[test]
    fn test_gradient_ai_config_serialization() {
        let config = GradientAIConfig {
            api_key: Some("test-key".to_string()),
            api_base: Some("https://custom.gradient.ai".to_string()),
            timeout: 45,
            max_retries: 2,
            debug: true,
            k: Some(5),
            retrieval_method: Some(RetrievalMethod::SubQueries),
            ..Default::default()
        };

        let json = serde_json::to_value(&config).unwrap();
        assert_eq!(json["api_key"], "test-key");
        assert_eq!(json["api_base"], "https://custom.gradient.ai");
        assert_eq!(json["timeout"], 45);
        assert_eq!(json["max_retries"], 2);
        assert_eq!(json["debug"], true);
        assert_eq!(json["k"], 5);
        assert_eq!(json["retrieval_method"], "sub_queries");
    }

    #[test]
    fn test_gradient_ai_config_deserialization() {
        let json = r#"{
            "api_key": "test-key",
            "timeout": 60,
            "debug": true,
            "k": 10,
            "retrieval_method": "rewrite"
        }"#;

        let config: GradientAIConfig = serde_json::from_str(json).unwrap();
        assert_eq!(config.api_key, Some("test-key".to_string()));
        assert_eq!(config.timeout, 60);
        assert!(config.debug);
        assert_eq!(config.k, Some(10));
        assert_eq!(config.retrieval_method, Some(RetrievalMethod::Rewrite));
    }

    #[test]
    fn test_gradient_ai_specific_params() {
        let config = GradientAIConfig {
            api_key: Some("test-key".to_string()),
            k: Some(5),
            filter_kb_content_by_query_metadata: Some(true),
            instruction_override: Some("Custom instructions".to_string()),
            include_functions_info: Some(true),
            include_retrieval_info: Some(true),
            include_guardrails_info: Some(false),
            provide_citations: Some(true),
            retrieval_method: Some(RetrievalMethod::SubQueries),
            ..Default::default()
        };

        assert_eq!(config.k, Some(5));
        assert_eq!(config.filter_kb_content_by_query_metadata, Some(true));
        assert_eq!(
            config.instruction_override,
            Some("Custom instructions".to_string())
        );
        assert_eq!(config.include_functions_info, Some(true));
        assert_eq!(config.include_retrieval_info, Some(true));
        assert_eq!(config.include_guardrails_info, Some(false));
        assert_eq!(config.provide_citations, Some(true));
        assert_eq!(config.retrieval_method, Some(RetrievalMethod::SubQueries));
    }
}
