//! Main Voyage AI Provider Implementation
//!
//! Implements the LLMProvider trait for Voyage AI's specialized embedding platform.
//! Voyage AI is focused on high-quality text embeddings for search and retrieval.

use std::collections::HashMap;
use std::sync::Arc;

use super::config::VoyageConfig;
use super::error::VoyageError;
use super::model_info::{get_available_models, get_model_info, supports_custom_dimensions};
use crate::core::providers::base::{GlobalPoolManager, HttpMethod, header};
use crate::core::traits::provider::ProviderConfig as _;
use crate::core::traits::provider::llm_provider::trait_definition::LLMProvider;
use crate::core::types::{
    embedding::EmbeddingInput,
    embedding::EmbeddingRequest,
    model::ModelInfo,
    model::ProviderCapability,
    responses::{EmbeddingData, EmbeddingResponse, Usage},
};

/// Static capabilities for Voyage AI provider
const VOYAGE_CAPABILITIES: &[ProviderCapability] = &[ProviderCapability::Embeddings];

/// Voyage AI provider implementation
#[derive(Debug, Clone)]
pub struct VoyageProvider {
    config: VoyageConfig,
    pool_manager: Arc<GlobalPoolManager>,
    models: Vec<ModelInfo>,
}

impl VoyageProvider {
    /// Create a new Voyage AI provider instance
    pub async fn new(config: VoyageConfig) -> Result<Self, VoyageError> {
        // Validate configuration
        config
            .validate()
            .map_err(|e| VoyageError::configuration("voyage", e))?;

        // Create pool manager
        let pool_manager = Arc::new(GlobalPoolManager::new().map_err(|e| {
            VoyageError::configuration("voyage", format!("Failed to create pool manager: {}", e))
        })?);

        // Build model list from static configuration
        let models = get_available_models()
            .iter()
            .filter_map(|id| get_model_info(id))
            .map(|info| ModelInfo {
                id: info.model_id.to_string(),
                name: info.display_name.to_string(),
                provider: "voyage".to_string(),
                max_context_length: info.max_tokens,
                max_output_length: None,
                supports_streaming: false,
                supports_tools: false,
                supports_multimodal: false,
                input_cost_per_1k_tokens: Some(info.cost_per_million_tokens / 1000.0),
                output_cost_per_1k_tokens: None,
                currency: "USD".to_string(),
                capabilities: vec![ProviderCapability::Embeddings],
                created_at: None,
                updated_at: None,
                metadata: {
                    let mut meta = HashMap::new();
                    meta.insert(
                        "embedding_dimensions".to_string(),
                        serde_json::json!(info.embedding_dimensions),
                    );
                    meta
                },
            })
            .collect();

        Ok(Self {
            config,
            pool_manager,
            models,
        })
    }

    /// Create provider with API key only
    pub async fn with_api_key(api_key: impl Into<String>) -> Result<Self, VoyageError> {
        let config = VoyageConfig::from_env().with_api_key(api_key);
        Self::new(config).await
    }

    /// Transform embedding request to Voyage AI format
    pub(crate) fn transform_embedding_request(
        &self,
        request: &EmbeddingRequest,
    ) -> Result<serde_json::Value, VoyageError> {
        let mut payload = serde_json::json!({
            "model": request.model,
            "input": self.normalize_input(&request.input),
        });

        // Add encoding_format if specified
        if let Some(ref encoding_format) = request.encoding_format {
            payload["encoding_format"] = serde_json::json!(encoding_format);
        }

        // Map OpenAI 'dimensions' to Voyage 'output_dimension'
        if let Some(dimensions) = request.dimensions
            && supports_custom_dimensions(&request.model)
        {
            payload["output_dimension"] = serde_json::json!(dimensions);
        }

        // Add task_type if specified (Voyage-specific parameter)
        if let Some(ref task_type) = request.task_type {
            payload["input_type"] = serde_json::json!(task_type);
        }

        Ok(payload)
    }

    /// Normalize input to array format
    fn normalize_input(&self, input: &EmbeddingInput) -> serde_json::Value {
        match input {
            EmbeddingInput::Text(text) => serde_json::json!([text]),
            EmbeddingInput::Array(arr) => serde_json::json!(arr),
        }
    }

    /// Transform Voyage AI response to standard format
    pub(crate) fn transform_embedding_response(
        &self,
        response: serde_json::Value,
    ) -> Result<EmbeddingResponse, VoyageError> {
        let object = response
            .get("object")
            .and_then(|v| v.as_str())
            .unwrap_or("list")
            .to_string();

        let model = response
            .get("model")
            .and_then(|v| v.as_str())
            .unwrap_or_default()
            .to_string();

        // Parse embeddings data
        let data: Vec<EmbeddingData> = response
            .get("data")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|item| {
                        let index = item.get("index")?.as_i64()? as u32;
                        let embedding = item
                            .get("embedding")?
                            .as_array()?
                            .iter()
                            .filter_map(|v| v.as_f64().map(|f| f as f32))
                            .collect();

                        Some(EmbeddingData {
                            object: "embedding".to_string(),
                            index,
                            embedding,
                        })
                    })
                    .collect()
            })
            .unwrap_or_default();

        // Parse usage - Voyage uses total_tokens only
        let usage = response.get("usage").map(|u| Usage {
            prompt_tokens: u.get("total_tokens").and_then(|v| v.as_u64()).unwrap_or(0) as u32,
            completion_tokens: 0,
            total_tokens: u.get("total_tokens").and_then(|v| v.as_u64()).unwrap_or(0) as u32,
            prompt_tokens_details: None,
            completion_tokens_details: None,
            thinking_usage: None,
        });

        Ok(EmbeddingResponse {
            object,
            data: data.clone(),
            model,
            usage,
            embeddings: Some(data),
        })
    }

    /// Execute an HTTP request
    async fn execute_request(
        &self,
        endpoint: &str,
        body: serde_json::Value,
    ) -> Result<serde_json::Value, VoyageError> {
        let url = if endpoint.starts_with("http") {
            endpoint.to_string()
        } else {
            format!("{}{}", self.config.get_api_base(), endpoint)
        };

        let mut headers = Vec::with_capacity(2);
        if let Some(api_key) = &self.config.get_api_key() {
            headers.push(header("Authorization", format!("Bearer {}", api_key)));
        }
        headers.push(header("Content-Type", "application/json".to_string()));

        let response = self
            .pool_manager
            .execute_request(&url, HttpMethod::POST, headers, Some(body))
            .await
            .map_err(|e| VoyageError::network("voyage", e.to_string()))?;

        let status = response.status();
        let response_bytes = response
            .bytes()
            .await
            .map_err(|e| VoyageError::network("voyage", e.to_string()))?;

        if !status.is_success() {
            let body_str = String::from_utf8_lossy(&response_bytes);
            return Err(VoyageError::api_error(
                "voyage",
                status.as_u16(),
                body_str.to_string(),
            ));
        }

        serde_json::from_slice(&response_bytes).map_err(|e| {
            VoyageError::api_error("voyage", 500, format!("Failed to parse response: {}", e))
        })
    }
}
