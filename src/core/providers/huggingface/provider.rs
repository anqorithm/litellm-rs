//! HuggingFace Provider Implementation
//!
//! Main provider implementation integrating HuggingFace Hub capabilities.

use serde_json::{Value, json};
use std::collections::HashMap;
use tracing::warn;

use crate::core::providers::base::{
    BaseConfig, BaseHttpClient, HttpErrorMapper, apply_headers, header,
};
use crate::core::traits::{
    error_mapper::trait_def::ErrorMapper, provider::ProviderConfig,
    provider::llm_provider::trait_definition::LLMProvider,
};
use crate::core::types::{chat::ChatRequest, model::ModelInfo, model::ProviderCapability};

use super::config::{HF_HUB_URL, HuggingFaceConfig};
use super::embedding::HuggingFaceEmbeddingHandler;
use super::error::{HuggingFaceError, parse_hf_error_response};
use super::models::get_default_models;

// Static capabilities
const HUGGINGFACE_CAPABILITIES: &[ProviderCapability] = &[
    ProviderCapability::ChatCompletion,
    ProviderCapability::ChatCompletionStream,
    ProviderCapability::ToolCalling,
    ProviderCapability::Embeddings,
];

/// HuggingFace error mapper
pub struct HuggingFaceErrorMapper;

impl ErrorMapper<HuggingFaceError> for HuggingFaceErrorMapper {
    fn map_http_error(&self, status_code: u16, response_body: &str) -> HuggingFaceError {
        parse_hf_error_response(status_code, response_body)
    }

    fn map_json_error(&self, error_response: &Value) -> HuggingFaceError {
        HttpErrorMapper::parse_json_error("huggingface", error_response)
    }

    fn map_network_error(&self, error: &dyn std::error::Error) -> HuggingFaceError {
        HuggingFaceError::network("huggingface", error.to_string())
    }

    fn map_parsing_error(&self, error: &dyn std::error::Error) -> HuggingFaceError {
        HuggingFaceError::response_parsing("huggingface", error.to_string())
    }

    fn map_timeout_error(&self, timeout_duration: std::time::Duration) -> HuggingFaceError {
        HuggingFaceError::timeout(
            "huggingface",
            format!("Request timed out after {:?}", timeout_duration),
        )
    }
}

/// HuggingFace provider implementation
#[derive(Debug, Clone)]
pub struct HuggingFaceProvider {
    config: HuggingFaceConfig,
    base_client: BaseHttpClient,
    embedding_handler: HuggingFaceEmbeddingHandler,
    models: Vec<ModelInfo>,
}

impl HuggingFaceProvider {
    /// Create a new HuggingFace provider instance
    pub async fn new(config: HuggingFaceConfig) -> Result<Self, HuggingFaceError> {
        // Validate configuration
        config
            .validate()
            .map_err(|e| HuggingFaceError::configuration("huggingface", e))?;

        // Create base HTTP client
        let base_config = BaseConfig {
            api_key: Some(config.api_key.clone()),
            api_base: config.api_base.clone(),
            timeout: config.timeout_seconds,
            max_retries: config.max_retries,
            headers: HashMap::new(),
            organization: None,
            api_version: None,
        };

        let base_client = BaseHttpClient::new(base_config)?;
        let embedding_handler = HuggingFaceEmbeddingHandler::new(config.clone());
        let models = get_default_models();

        Ok(Self {
            config,
            base_client,
            embedding_handler,
            models,
        })
    }

    /// Create provider with API key only
    pub async fn with_api_key(api_key: impl Into<String>) -> Result<Self, HuggingFaceError> {
        let config = HuggingFaceConfig::new(api_key);
        Self::new(config).await
    }

    /// Get the config
    pub fn config(&self) -> &HuggingFaceConfig {
        &self.config
    }

    /// Transform a chat request to HuggingFace/OpenAI-compatible format
    fn transform_chat_request(&self, request: &ChatRequest, mapped_model: &str) -> Value {
        let mut body = json!({
            "model": mapped_model,
            "messages": request.messages,
        });

        // Add optional parameters
        if let Some(temp) = request.temperature {
            // HuggingFace requires temperature > 0
            let temp_value = if temp <= 0.0 { 0.01 } else { temp };
            body["temperature"] = json!(temp_value);
        }

        if let Some(max_tokens) = request.max_tokens {
            // HuggingFace uses max_new_tokens
            let max_value = if max_tokens == 0 { 1 } else { max_tokens };
            body["max_tokens"] = json!(max_value);
        }

        if let Some(max_completion_tokens) = request.max_completion_tokens {
            let max_value = if max_completion_tokens == 0 {
                1
            } else {
                max_completion_tokens
            };
            body["max_tokens"] = json!(max_value);
        }

        if let Some(top_p) = request.top_p {
            body["top_p"] = json!(top_p);
        }

        if let Some(stop) = &request.stop {
            body["stop"] = json!(stop);
        }

        if let Some(n) = request.n {
            body["n"] = json!(n);
        }

        if let Some(seed) = request.seed {
            body["seed"] = json!(seed);
        }

        if let Some(user) = &request.user {
            body["user"] = json!(user);
        }

        if let Some(tools) = &request.tools {
            body["tools"] = json!(tools);
        }

        if let Some(tool_choice) = &request.tool_choice {
            body["tool_choice"] = json!(tool_choice);
        }

        if let Some(response_format) = &request.response_format {
            body["response_format"] = json!(response_format);
        }

        body
    }

    /// Fetch provider mapping from HuggingFace Hub API
    async fn fetch_provider_mapping(
        &self,
        model: &str,
    ) -> Result<HashMap<String, Value>, HuggingFaceError> {
        let url = format!("{}/api/models/{}", HF_HUB_URL, model);

        let headers = vec![
            header("Authorization", format!("Bearer {}", self.config.api_key)),
            header("Accept", "application/json".to_string()),
        ];

        let response = apply_headers(self.base_client.inner().get(&url), headers)
            .query(&[("expand", "inferenceProviderMapping")])
            .send()
            .await
            .map_err(|e| HuggingFaceError::huggingface_network_error(e.to_string()))?;

        if !response.status().is_success() {
            let status = response.status().as_u16();
            let body = response.text().await.unwrap_or_default();
            return Err(parse_hf_error_response(status, &body));
        }

        let data: Value = response
            .json()
            .await
            .map_err(|e| HuggingFaceError::huggingface_response_parsing(e.to_string()))?;

        if let Some(mapping) = data.get("inferenceProviderMapping")
            && let Some(obj) = mapping.as_object()
        {
            return Ok(obj.clone().into_iter().collect());
        }

        Ok(HashMap::new())
    }

    /// Get the mapped model ID for a specific provider
    async fn get_mapped_model(
        &self,
        model: &str,
        provider: &str,
    ) -> Result<String, HuggingFaceError> {
        let mapping = self.fetch_provider_mapping(model).await?;

        if let Some(provider_info) = mapping.get(provider) {
            if let Some(status) = provider_info.get("status").and_then(|s| s.as_str())
                && status == "staging"
            {
                warn!(
                    "Model {} is in staging mode for provider {}. Meant for test purposes only.",
                    model, provider
                );
            }

            if let Some(provider_id) = provider_info.get("providerId").and_then(|p| p.as_str()) {
                return Ok(provider_id.to_string());
            }
        }

        // Check if provider is available
        if mapping.is_empty() || !mapping.contains_key(provider) {
            return Err(HuggingFaceError::huggingface_provider_not_found(
                model, provider,
            ));
        }

        Ok(model.to_string())
    }

    /// Determine if this is a TGI/custom endpoint request
    fn is_custom_endpoint(&self) -> bool {
        self.config.api_base.is_some()
    }
}
