//! Main Triton Provider Implementation
//!
//! Implements the LLMProvider trait for NVIDIA Triton Inference Server.

use std::collections::HashMap;
use std::sync::Arc;
use tracing::debug;

use super::config::TritonConfig;
use super::error::TritonError;
use super::models::{
    ModelMetadataResponse, TritonInferRequest, TritonInferResponse, TritonModelInfo, TritonTensor,
};
use crate::core::providers::base::{
    GlobalPoolManager, HeaderPair, HttpMethod, header, header_owned,
};
use crate::core::traits::{
    provider::ProviderConfig, provider::llm_provider::trait_definition::LLMProvider,
};
use crate::core::types::{
    chat::ChatMessage,
    chat::ChatRequest,
    message::MessageContent,
    message::MessageRole,
    model::ModelInfo,
    model::ProviderCapability,
    responses::{ChatChoice, ChatResponse, FinishReason, Usage},
};

const PROVIDER_NAME: &str = "triton";

/// Static capabilities for Triton provider
const TRITON_CAPABILITIES: &[ProviderCapability] = &[ProviderCapability::ChatCompletion];

/// Triton provider implementation
#[derive(Debug, Clone)]
pub struct TritonProvider {
    config: TritonConfig,
    pool_manager: Arc<GlobalPoolManager>,
    models: Vec<ModelInfo>,
}

impl TritonProvider {
    /// Create a new Triton provider instance
    pub async fn new(config: TritonConfig) -> Result<Self, TritonError> {
        // Validate configuration
        config
            .validate()
            .map_err(|e| TritonError::configuration(PROVIDER_NAME, e))?;

        // Create pool manager
        let pool_manager = Arc::new(GlobalPoolManager::new().map_err(|e| {
            TritonError::configuration(
                PROVIDER_NAME,
                format!("Failed to create pool manager: {}", e),
            )
        })?);

        // Initialize with empty models list - will be populated from server
        let models = Vec::new();

        Ok(Self {
            config,
            pool_manager,
            models,
        })
    }

    /// Create provider with server URL only
    pub async fn with_server_url(server_url: impl Into<String>) -> Result<Self, TritonError> {
        let config = TritonConfig::new(server_url);
        Self::new(config).await
    }

    /// Create provider from environment variables
    pub async fn from_env() -> Result<Self, TritonError> {
        let config = TritonConfig::default();
        Self::new(config).await
    }

    /// Get the base URL for API requests
    fn get_base_url(&self) -> String {
        self.config.get_server_url()
    }

    /// Build the model endpoint URL
    fn get_model_url(&self, model: &str, version: Option<&str>) -> String {
        let base = self.get_base_url();
        match version {
            Some(v) => format!("{}/v2/models/{}/versions/{}", base, model, v),
            None => format!("{}/v2/models/{}", base, model),
        }
    }

    /// Build default headers for requests
    fn build_headers(&self) -> Vec<HeaderPair> {
        let mut headers = vec![header("Content-Type", "application/json".to_string())];

        // Add custom headers from config
        for (key, value) in &self.config.headers {
            headers.push(header_owned(key.clone(), value.clone()));
        }

        headers
    }

    /// Check if the Triton server is healthy
    pub async fn is_healthy(&self) -> bool {
        let url = format!("{}/v2/health/ready", self.get_base_url());
        let headers = self.build_headers();

        match self
            .pool_manager
            .execute_request(&url, HttpMethod::GET, headers, None::<serde_json::Value>)
            .await
        {
            Ok(response) => response.status().is_success(),
            Err(_) => false,
        }
    }

    /// Check if a specific model is ready
    pub async fn is_model_ready(&self, model: &str) -> Result<bool, TritonError> {
        let url = format!("{}/v2/models/{}/ready", self.get_base_url(), model);
        let headers = self.build_headers();

        match self
            .pool_manager
            .execute_request(&url, HttpMethod::GET, headers, None::<serde_json::Value>)
            .await
        {
            Ok(response) => Ok(response.status().is_success()),
            Err(e) => Err(TritonError::network(PROVIDER_NAME, e.to_string())),
        }
    }

    /// Get model metadata from Triton server
    pub async fn get_model_metadata(
        &self,
        model: &str,
    ) -> Result<ModelMetadataResponse, TritonError> {
        let url = self.get_model_url(model, self.config.get_model_version().as_deref());
        let headers = self.build_headers();

        let response = self
            .pool_manager
            .execute_request(&url, HttpMethod::GET, headers, None::<serde_json::Value>)
            .await
            .map_err(|e| TritonError::network(PROVIDER_NAME, e.to_string()))?;

        if !response.status().is_success() {
            let status = response.status().as_u16();
            return Err(self.map_http_error(
                status,
                &format!("Failed to get model metadata for {}", model),
            ));
        }

        let bytes = response
            .bytes()
            .await
            .map_err(|e| TritonError::network(PROVIDER_NAME, e.to_string()))?;

        serde_json::from_slice(&bytes).map_err(|e| {
            TritonError::response_parsing(
                PROVIDER_NAME,
                format!("Failed to parse model metadata: {}", e),
            )
        })
    }

    /// Get detailed model info from Triton server
    pub async fn get_triton_model_info(&self, model: &str) -> Result<TritonModelInfo, TritonError> {
        let metadata = self.get_model_metadata(model).await?;

        Ok(TritonModelInfo {
            name: metadata.name,
            version: metadata.versions.first().cloned(),
            state: Some("READY".to_string()),
            platform: metadata.platform,
            max_batch_size: None,
            inputs: metadata
                .inputs
                .into_iter()
                .map(|t| super::models::TensorInfo {
                    name: t.name,
                    datatype: t.datatype,
                    shape: t.shape,
                })
                .collect(),
            outputs: metadata
                .outputs
                .into_iter()
                .map(|t| super::models::TensorInfo {
                    name: t.name,
                    datatype: t.datatype,
                    shape: t.shape,
                })
                .collect(),
            parameters: HashMap::new(),
        })
    }

    /// Execute inference request on Triton server
    async fn infer(
        &self,
        model: &str,
        request: TritonInferRequest,
    ) -> Result<TritonInferResponse, TritonError> {
        let url = format!(
            "{}/infer",
            self.get_model_url(model, self.config.get_model_version().as_deref())
        );
        let headers = self.build_headers();

        debug!("Triton inference request: model={}, url={}", model, url);

        let request_body = serde_json::to_value(&request)
            .map_err(|e| TritonError::invalid_request(PROVIDER_NAME, e.to_string()))?;

        let response = self
            .pool_manager
            .execute_request(&url, HttpMethod::POST, headers, Some(request_body))
            .await
            .map_err(|e| TritonError::network(PROVIDER_NAME, e.to_string()))?;

        if !response.status().is_success() {
            let status = response.status().as_u16();
            let body = response.text().await.unwrap_or_default();
            return Err(self.map_http_error(status, &body));
        }

        let bytes = response
            .bytes()
            .await
            .map_err(|e| TritonError::network(PROVIDER_NAME, e.to_string()))?;

        serde_json::from_slice(&bytes).map_err(|e| {
            TritonError::response_parsing(
                PROVIDER_NAME,
                format!("Failed to parse inference response: {}", e),
            )
        })
    }

    /// Map HTTP error status to ProviderError
    fn map_http_error(&self, status: u16, body: &str) -> TritonError {
        match status {
            400 => TritonError::invalid_request(PROVIDER_NAME, body),
            401 | 403 => TritonError::authentication(PROVIDER_NAME, "Authentication failed"),
            404 => TritonError::model_not_found(PROVIDER_NAME, body),
            408 => TritonError::timeout(PROVIDER_NAME, "Request timeout"),
            429 => TritonError::rate_limit(PROVIDER_NAME, None),
            500..=599 => TritonError::provider_unavailable(PROVIDER_NAME, body),
            _ => TritonError::api_error(PROVIDER_NAME, status, body),
        }
    }

    /// Convert chat messages to Triton inference request
    fn build_inference_request(&self, request: &ChatRequest) -> TritonInferRequest {
        // Serialize messages to a prompt string
        // This is a simple implementation - actual format depends on model
        let prompt = request
            .messages
            .iter()
            .map(|m| {
                let role = format!("{:?}", m.role).to_lowercase();
                format!(
                    "{}: {}",
                    role,
                    m.content
                        .as_ref()
                        .map(|c| c.to_string())
                        .unwrap_or_default()
                )
            })
            .collect::<Vec<_>>()
            .join("\n");

        let mut parameters = HashMap::new();

        // Add generation parameters
        if let Some(temp) = request.temperature {
            parameters.insert("temperature".to_string(), serde_json::json!(temp));
        }
        if let Some(max_tokens) = request.max_tokens {
            parameters.insert("max_tokens".to_string(), serde_json::json!(max_tokens));
        }
        if let Some(top_p) = request.top_p {
            parameters.insert("top_p".to_string(), serde_json::json!(top_p));
        }

        TritonInferRequest {
            id: Some(uuid::Uuid::new_v4().to_string()),
            inputs: vec![TritonTensor {
                name: "text_input".to_string(),
                datatype: "BYTES".to_string(),
                shape: vec![1],
                data: serde_json::json!([prompt]),
                parameters: None,
            }],
            outputs: Some(vec![super::models::TritonOutputRequest {
                name: "text_output".to_string(),
                parameters: None,
            }]),
            parameters: if parameters.is_empty() {
                None
            } else {
                Some(parameters)
            },
        }
    }

    /// Convert Triton response to ChatResponse
    fn build_chat_response(
        &self,
        model: &str,
        response: TritonInferResponse,
        request_id: &str,
    ) -> Result<ChatResponse, TritonError> {
        // Extract text output from response
        let text_output = response
            .outputs
            .iter()
            .find(|o| o.name == "text_output" || o.name.contains("output"))
            .ok_or_else(|| {
                TritonError::response_parsing(PROVIDER_NAME, "No output tensor found in response")
            })?;

        // Parse the output data
        let content = match &text_output.data {
            serde_json::Value::Array(arr) => arr
                .first()
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            serde_json::Value::String(s) => s.clone(),
            _ => text_output.data.to_string(),
        };

        Ok(ChatResponse {
            id: request_id.to_string(),
            object: "chat.completion".to_string(),
            created: chrono::Utc::now().timestamp(),
            model: model.to_string(),
            choices: vec![ChatChoice {
                index: 0,
                message: ChatMessage {
                    role: MessageRole::Assistant,
                    content: Some(MessageContent::Text(content)),
                    thinking: None,
                    name: None,
                    tool_calls: None,
                    tool_call_id: None,
                    function_call: None,
                },
                finish_reason: Some(FinishReason::Stop),
                logprobs: None,
            }],
            usage: Some(Usage {
                prompt_tokens: 0,     // Triton doesn't typically return token counts
                completion_tokens: 0, // Would need tokenizer to calculate
                total_tokens: 0,
                completion_tokens_details: None,
                prompt_tokens_details: None,
                thinking_usage: None,
            }),
            system_fingerprint: None,
        })
    }
}
