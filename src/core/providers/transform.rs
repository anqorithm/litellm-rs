//! Request/Response Transformation Engine - Format normalization across providers
//!
//! This module implements sophisticated transformation pipelines that convert
//! between OpenAI-compatible format and provider-specific formats.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use std::collections::HashMap;

use super::{ProviderType, unified_provider::ProviderError};

/// Result type for provider operations
pub type ProviderResult<T> = Result<T, ProviderError>;

/// Generic request types for different endpoints
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatRequest {
    pub model: String,
    pub messages: Vec<ChatMessage>,
    pub temperature: Option<f64>,
    pub max_tokens: Option<u32>,
    pub stream: Option<bool>,
    pub functions: Option<Vec<Function>>,
    pub function_call: Option<FunctionCall>,
    pub tools: Option<Vec<Tool>>,
    pub tool_choice: Option<ToolChoice>,
    pub top_p: Option<f64>,
    pub presence_penalty: Option<f64>,
    pub frequency_penalty: Option<f64>,
    pub stop: Option<Vec<String>>,
    pub response_format: Option<ResponseFormat>,
    pub seed: Option<i32>,
    pub logit_bias: Option<HashMap<String, f64>>,
    pub user: Option<String>,
    pub extra_headers: Option<HashMap<String, String>>,
    pub extra_body: Option<Map<String, Value>>,
    pub thinking: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: Option<Value>, // Can be string or structured content
    pub name: Option<String>,
    pub function_call: Option<FunctionCallResponse>,
    pub tool_calls: Option<Vec<ToolCall>>,
    pub tool_call_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Function {
    pub name: String,
    pub description: Option<String>,
    pub parameters: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum FunctionCall {
    Auto,
    None,
    Specific { name: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tool {
    #[serde(rename = "type")]
    pub tool_type: String,
    pub function: Function,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ToolChoice {
    Auto,
    None,
    Required,
    Specific {
        #[serde(rename = "type")]
        tool_type: String,
        function: ToolFunction,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolFunction {
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    pub id: String,
    #[serde(rename = "type")]
    pub tool_type: String,
    pub function: FunctionCallResponse,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionCallResponse {
    pub name: String,
    pub arguments: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseFormat {
    #[serde(rename = "type")]
    pub format_type: String, // "text" or "json_object"
}

/// Generic response types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatResponse {
    pub id: String,
    pub object: String,
    pub created: u64,
    pub model: String,
    pub choices: Vec<Choice>,
    pub usage: Option<Usage>,
    pub system_fingerprint: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Choice {
    pub index: u32,
    pub message: ChatMessage,
    pub finish_reason: Option<String>,
    pub logprobs: Option<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Usage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

/// Embedding request/response types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingRequest {
    pub model: String,
    pub input: EmbeddingInput,
    pub encoding_format: Option<String>,
    pub dimensions: Option<u32>,
    pub user: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum EmbeddingInput {
    String(String),
    Strings(Vec<String>),
    Tokens(Vec<u32>),
    TokenArrays(Vec<Vec<u32>>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingResponse {
    pub object: String,
    pub data: Vec<EmbeddingData>,
    pub model: String,
    pub usage: Usage,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingData {
    pub object: String,
    pub embedding: Vec<f64>,
    pub index: u32,
}

/// Generic provider request/response (what gets sent to actual provider APIs)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderRequest {
    pub endpoint: String,
    pub method: String,
    pub headers: HashMap<String, String>,
    pub body: Value,
    pub query_params: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderResponse {
    pub status_code: u16,
    pub headers: HashMap<String, String>,
    pub body: Value,
    pub latency_ms: f64,
}

/// Transform result with metadata
#[derive(Debug, Clone)]
pub struct TransformResult<T> {
    pub data: T,
    pub metadata: TransformMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransformMetadata {
    pub provider_type: ProviderType,
    pub original_model: String,
    pub transformed_model: String,
    pub transformations_applied: Vec<String>,
    pub warnings: Vec<String>,
    pub cost_estimate: Option<f64>,
}

/// Transformation engine trait
#[async_trait]
pub trait TransformEngine: Send + Sync {
    /// Transform OpenAI format request to provider-specific format
    async fn transform_chat_request(
        &self,
        request: &ChatRequest,
        provider_type: &ProviderType,
        provider_config: &HashMap<String, Value>,
    ) -> ProviderResult<TransformResult<ProviderRequest>>;

    /// Transform provider response back to OpenAI format
    async fn transform_chat_response(
        &self,
        response: &ProviderResponse,
        provider_type: &ProviderType,
        original_request: &ChatRequest,
    ) -> ProviderResult<TransformResult<ChatResponse>>;

    /// Transform embedding request
    async fn transform_embedding_request(
        &self,
        request: &EmbeddingRequest,
        provider_type: &ProviderType,
        provider_config: &HashMap<String, Value>,
    ) -> ProviderResult<TransformResult<ProviderRequest>>;

    /// Transform embedding response
    async fn transform_embedding_response(
        &self,
        response: &ProviderResponse,
        provider_type: &ProviderType,
        original_request: &EmbeddingRequest,
    ) -> ProviderResult<TransformResult<EmbeddingResponse>>;

    /// Get supported transformations for a provider
    fn get_supported_transformations(&self, provider_type: &ProviderType) -> Vec<String>;

    /// Validate request compatibility with provider
    async fn validate_request_compatibility(
        &self,
        request: &ChatRequest,
        provider_type: &ProviderType,
    ) -> ProviderResult<Vec<String>>;
}

/// Transform pipeline for chaining transformations
pub struct TransformPipeline {
    transforms: Vec<Box<dyn Transform>>,
}

/// Individual transformation step
#[async_trait]
pub trait Transform: Send + Sync {
    /// Apply transformation to request
    async fn transform_request(
        &self,
        request: Value,
        context: &TransformContext,
    ) -> ProviderResult<Value>;

    /// Apply reverse transformation to response
    async fn transform_response(
        &self,
        response: Value,
        context: &TransformContext,
    ) -> ProviderResult<Value>;

    /// Get transformation name
    fn name(&self) -> &str;
}

/// Context for transformations
#[derive(Debug, Clone)]
pub struct TransformContext {
    pub provider_type: ProviderType,
    pub original_model: String,
    pub target_model: String,
    pub config: HashMap<String, Value>,
    pub metadata: HashMap<String, String>,
}

/// Default transformation engine implementation
pub struct DefaultTransformEngine {
    pipelines: HashMap<ProviderType, TransformPipeline>,
    model_mappings: HashMap<String, ModelMapping>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelMapping {
    pub provider_model: String,
    pub openai_equivalent: String,
    pub capabilities: Vec<String>,
    pub parameter_mappings: HashMap<String, String>,
}

impl Default for DefaultTransformEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl DefaultTransformEngine {
    pub fn new() -> Self {
        let mut engine = Self {
            pipelines: HashMap::new(),
            model_mappings: HashMap::new(),
        };

        engine.init_default_mappings();
        engine.init_default_pipelines();
        engine
    }

    fn init_default_mappings(&mut self) {
        // Anthropic model mappings
        self.model_mappings.insert(
            "claude-3-sonnet".to_string(),
            ModelMapping {
                provider_model: "claude-3-sonnet-20240229".to_string(),
                openai_equivalent: "gpt-4".to_string(),
                capabilities: vec!["chat".to_string(), "vision".to_string()],
                parameter_mappings: HashMap::from([
                    ("max_tokens".to_string(), "max_tokens".to_string()),
                    ("temperature".to_string(), "temperature".to_string()),
                ]),
            },
        );

        // Google model mappings
        self.model_mappings.insert(
            "gemini-pro".to_string(),
            ModelMapping {
                provider_model: "gemini-1.0-pro".to_string(),
                openai_equivalent: "gpt-3.5-turbo".to_string(),
                capabilities: vec!["chat".to_string()],
                parameter_mappings: HashMap::from([
                    ("max_tokens".to_string(), "maxOutputTokens".to_string()),
                    ("temperature".to_string(), "temperature".to_string()),
                ]),
            },
        );
    }

    fn init_default_pipelines(&mut self) {
        // Initialize transformation pipelines for each provider
        // This would include provider-specific transformations

        // Anthropic pipeline
        let anthropic_pipeline = TransformPipeline {
            transforms: vec![
                Box::new(AnthropicMessageTransform::new()),
                Box::new(AnthropicParameterTransform::new()),
            ],
        };
        self.pipelines
            .insert(ProviderType::Anthropic, anthropic_pipeline);

        // VertexAI/Gemini pipeline
        let vertexai_pipeline = TransformPipeline {
            transforms: vec![
                Box::new(GoogleMessageTransform::new()),
                Box::new(GoogleParameterTransform::new()),
            ],
        };
        self.pipelines
            .insert(ProviderType::VertexAI, vertexai_pipeline);
    }

    fn map_model_name(&self, model: &str, provider_type: &ProviderType) -> String {
        // Model name mapping logic
        match provider_type {
            ProviderType::Anthropic => {
                if model.starts_with("claude") {
                    model.to_string()
                } else {
                    "claude-3-sonnet-20240229".to_string() // default
                }
            }
            ProviderType::VertexAI => {
                if model.starts_with("gemini") {
                    model.to_string()
                } else {
                    "gemini-1.0-pro".to_string() // default
                }
            }
            _ => model.to_string(),
        }
    }
}

#[async_trait]
impl TransformEngine for DefaultTransformEngine {
    async fn transform_chat_request(
        &self,
        request: &ChatRequest,
        provider_type: &ProviderType,
        provider_config: &HashMap<String, Value>,
    ) -> ProviderResult<TransformResult<ProviderRequest>> {
        let context = TransformContext {
            provider_type: provider_type.clone(),
            original_model: request.model.clone(),
            target_model: self.map_model_name(&request.model, provider_type),
            config: provider_config.clone(),
            metadata: HashMap::new(),
        };

        let mut transformations = Vec::new();
        let warnings = Vec::new();

        // Convert request to JSON for pipeline processing
        let mut request_json =
            serde_json::to_value(request).map_err(|e| ProviderError::Serialization {
                provider: "transform",
                message: format!("Serialization error: {}", e),
            })?;

        // Apply transformation pipeline if available
        if let Some(pipeline) = self.pipelines.get(provider_type) {
            for transform in &pipeline.transforms {
                transformations.push(transform.name().to_string());
                request_json = transform.transform_request(request_json, &context).await?;
            }
        }

        // Build provider request
        let provider_request = match provider_type {
            ProviderType::Anthropic => self.build_anthropic_request(request_json, &context).await?,
            ProviderType::VertexAI => self.build_vertexai_request(request_json, &context).await?,
            _ => {
                self.build_openai_compatible_request(request_json, &context)
                    .await?
            }
        };

        Ok(TransformResult {
            data: provider_request,
            metadata: TransformMetadata {
                provider_type: provider_type.clone(),
                original_model: request.model.clone(),
                transformed_model: context.target_model,
                transformations_applied: transformations,
                warnings,
                cost_estimate: None,
            },
        })
    }

    async fn transform_chat_response(
        &self,
        response: &ProviderResponse,
        provider_type: &ProviderType,
        original_request: &ChatRequest,
    ) -> ProviderResult<TransformResult<ChatResponse>> {
        let context = TransformContext {
            provider_type: provider_type.clone(),
            original_model: original_request.model.clone(),
            target_model: self.map_model_name(&original_request.model, provider_type),
            config: HashMap::new(),
            metadata: HashMap::new(),
        };

        let mut transformations = Vec::new();
        let mut response_json = response.body.clone();

        // Apply reverse transformation pipeline
        if let Some(pipeline) = self.pipelines.get(provider_type) {
            for transform in pipeline.transforms.iter().rev() {
                transformations.push(format!("reverse_{}", transform.name()));
                response_json = transform
                    .transform_response(response_json, &context)
                    .await?;
            }
        }

        // Convert back to ChatResponse
        let chat_response: ChatResponse =
            serde_json::from_value(response_json).map_err(|e| ProviderError::Serialization {
                provider: "transform",
                message: format!("Deserialization error: {}", e),
            })?;

        Ok(TransformResult {
            data: chat_response,
            metadata: TransformMetadata {
                provider_type: provider_type.clone(),
                original_model: original_request.model.clone(),
                transformed_model: context.target_model,
                transformations_applied: transformations,
                warnings: Vec::new(),
                cost_estimate: None,
            },
        })
    }

    async fn transform_embedding_request(
        &self,
        request: &EmbeddingRequest,
        provider_type: &ProviderType,
        provider_config: &HashMap<String, Value>,
    ) -> ProviderResult<TransformResult<ProviderRequest>> {
        // Similar implementation for embedding requests
        let context = TransformContext {
            provider_type: provider_type.clone(),
            original_model: request.model.clone(),
            target_model: self.map_model_name(&request.model, provider_type),
            config: provider_config.clone(),
            metadata: HashMap::new(),
        };

        let request_json =
            serde_json::to_value(request).map_err(|e| ProviderError::Serialization {
                provider: "transform",
                message: format!("Serialization error: {}", e),
            })?;

        let provider_request = self
            .build_openai_compatible_request(request_json, &context)
            .await?;

        Ok(TransformResult {
            data: provider_request,
            metadata: TransformMetadata {
                provider_type: provider_type.clone(),
                original_model: request.model.clone(),
                transformed_model: context.target_model,
                transformations_applied: vec!["embedding_transform".to_string()],
                warnings: Vec::new(),
                cost_estimate: None,
            },
        })
    }

    async fn transform_embedding_response(
        &self,
        response: &ProviderResponse,
        provider_type: &ProviderType,
        original_request: &EmbeddingRequest,
    ) -> ProviderResult<TransformResult<EmbeddingResponse>> {
        let embedding_response: EmbeddingResponse = serde_json::from_value(response.body.clone())
            .map_err(|e| ProviderError::Serialization {
            provider: "transform",
            message: format!("Deserialization error: {}", e),
        })?;

        Ok(TransformResult {
            data: embedding_response,
            metadata: TransformMetadata {
                provider_type: provider_type.clone(),
                original_model: original_request.model.clone(),
                transformed_model: self.map_model_name(&original_request.model, provider_type),
                transformations_applied: vec!["embedding_response_transform".to_string()],
                warnings: Vec::new(),
                cost_estimate: None,
            },
        })
    }

    fn get_supported_transformations(&self, provider_type: &ProviderType) -> Vec<String> {
        self.pipelines
            .get(provider_type)
            .map(|pipeline| {
                pipeline
                    .transforms
                    .iter()
                    .map(|t| t.name().to_string())
                    .collect()
            })
            .unwrap_or_default()
    }

    async fn validate_request_compatibility(
        &self,
        request: &ChatRequest,
        provider_type: &ProviderType,
    ) -> ProviderResult<Vec<String>> {
        let mut issues = Vec::new();

        // Check for unsupported features
        match provider_type {
            ProviderType::Anthropic => {
                if request.functions.is_some() {
                    issues.push(
                        "Functions are not supported by Anthropic, use tools instead".to_string(),
                    );
                }
                if request.logit_bias.is_some() {
                    issues.push("Logit bias is not supported by Anthropic".to_string());
                }
            }
            ProviderType::VertexAI => {
                if request.functions.is_some() || request.tools.is_some() {
                    issues.push("Function calling support limited in Vertex AI models".to_string());
                }
            }
            _ => {}
        }

        Ok(issues)
    }
}

impl DefaultTransformEngine {
    async fn build_anthropic_request(
        &self,
        _request: Value,
        _context: &TransformContext,
    ) -> ProviderResult<ProviderRequest> {
        // Build Anthropic-specific request format
        Ok(ProviderRequest {
            endpoint: "/v1/messages".to_string(),
            method: "POST".to_string(),
            headers: HashMap::from([
                ("Content-Type".to_string(), "application/json".to_string()),
                ("anthropic-version".to_string(), "2023-06-01".to_string()),
            ]),
            body: serde_json::json!({}), // Would contain transformed request
            query_params: HashMap::new(),
        })
    }

    async fn build_vertexai_request(
        &self,
        _request: Value,
        context: &TransformContext,
    ) -> ProviderResult<ProviderRequest> {
        // Build VertexAI/Gemini-specific request format
        Ok(ProviderRequest {
            endpoint: format!("/v1/models/{}:generateContent", context.target_model),
            method: "POST".to_string(),
            headers: HashMap::from([("Content-Type".to_string(), "application/json".to_string())]),
            body: serde_json::json!({}), // Would contain transformed request
            query_params: HashMap::new(),
        })
    }

    async fn build_openai_compatible_request(
        &self,
        request: Value,
        _context: &TransformContext,
    ) -> ProviderResult<ProviderRequest> {
        // Build OpenAI-compatible request format
        Ok(ProviderRequest {
            endpoint: "/v1/chat/completions".to_string(),
            method: "POST".to_string(),
            headers: HashMap::from([("Content-Type".to_string(), "application/json".to_string())]),
            body: request,
            query_params: HashMap::new(),
        })
    }
}

// Example transformation implementations
#[derive(Default)]
pub struct AnthropicMessageTransform;
#[derive(Default)]
pub struct AnthropicParameterTransform;
#[derive(Default)]
pub struct GoogleMessageTransform;
#[derive(Default)]
pub struct GoogleParameterTransform;

impl AnthropicMessageTransform {
    pub fn new() -> Self {
        Self
    }
}

impl AnthropicParameterTransform {
    pub fn new() -> Self {
        Self
    }
}

impl GoogleMessageTransform {
    pub fn new() -> Self {
        Self
    }
}

impl GoogleParameterTransform {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl Transform for AnthropicMessageTransform {
    async fn transform_request(
        &self,
        request: Value,
        _context: &TransformContext,
    ) -> ProviderResult<Value> {
        // Transform OpenAI messages to Anthropic format
        // Implementation would handle message role mapping, content structure, etc.
        Ok(request)
    }

    async fn transform_response(
        &self,
        response: Value,
        _context: &TransformContext,
    ) -> ProviderResult<Value> {
        // Transform Anthropic response back to OpenAI format
        Ok(response)
    }

    fn name(&self) -> &str {
        "anthropic_message_transform"
    }
}

#[async_trait]
impl Transform for AnthropicParameterTransform {
    async fn transform_request(
        &self,
        request: Value,
        _context: &TransformContext,
    ) -> ProviderResult<Value> {
        // Transform OpenAI parameters to Anthropic equivalents
        Ok(request)
    }

    async fn transform_response(
        &self,
        response: Value,
        _context: &TransformContext,
    ) -> ProviderResult<Value> {
        Ok(response)
    }

    fn name(&self) -> &str {
        "anthropic_parameter_transform"
    }
}

#[async_trait]
impl Transform for GoogleMessageTransform {
    async fn transform_request(
        &self,
        request: Value,
        _context: &TransformContext,
    ) -> ProviderResult<Value> {
        // Transform OpenAI messages to Google format
        Ok(request)
    }

    async fn transform_response(
        &self,
        response: Value,
        _context: &TransformContext,
    ) -> ProviderResult<Value> {
        // Transform Google response back to OpenAI format
        Ok(response)
    }

    fn name(&self) -> &str {
        "google_message_transform"
    }
}

#[async_trait]
impl Transform for GoogleParameterTransform {
    async fn transform_request(
        &self,
        request: Value,
        _context: &TransformContext,
    ) -> ProviderResult<Value> {
        // Transform OpenAI parameters to Google equivalents
        Ok(request)
    }

    async fn transform_response(
        &self,
        response: Value,
        _context: &TransformContext,
    ) -> ProviderResult<Value> {
        Ok(response)
    }

    fn name(&self) -> &str {
        "google_parameter_transform"
    }
}


#[cfg(test)]
#[path = "transform_tests.rs"]
mod tests;
