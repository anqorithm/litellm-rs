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
pub struct TransformChatRequest {
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
        request: &TransformChatRequest,
        provider_type: &ProviderType,
        provider_config: &HashMap<String, Value>,
    ) -> ProviderResult<TransformResult<ProviderRequest>>;

    /// Transform provider response back to OpenAI format
    async fn transform_chat_response(
        &self,
        response: &ProviderResponse,
        provider_type: &ProviderType,
        original_request: &TransformChatRequest,
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
        request: &TransformChatRequest,
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
        request: &TransformChatRequest,
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
        original_request: &TransformChatRequest,
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
        request: &TransformChatRequest,
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
mod tests {
    use super::*;
    use serde_json::json;

    // ==================== TransformChatRequest Tests ====================

    #[test]
    fn test_chat_request_serialization_full() {
        let request = TransformChatRequest {
            model: "gpt-4".to_string(),
            messages: vec![ChatMessage {
                role: "user".to_string(),
                content: Some(json!("Hello")),
                name: None,
                function_call: None,
                tool_calls: None,
                tool_call_id: None,
            }],
            temperature: Some(0.7),
            max_tokens: Some(1000),
            stream: Some(false),
            functions: None,
            function_call: None,
            tools: None,
            tool_choice: None,
            top_p: Some(0.9),
            presence_penalty: Some(0.0),
            frequency_penalty: Some(0.0),
            stop: Some(vec!["END".to_string()]),
            response_format: None,
            seed: Some(42),
            logit_bias: None,
            user: Some("test-user".to_string()),
            extra_headers: None,
            extra_body: None,
            thinking: None,
        };

        let json = serde_json::to_value(&request).unwrap();
        assert_eq!(json["model"], "gpt-4");
        assert_eq!(json["temperature"], 0.7);
        assert_eq!(json["max_tokens"], 1000);
        assert_eq!(json["seed"], 42);
    }

    #[test]
    fn test_chat_request_minimal() {
        let request = TransformChatRequest {
            model: "gpt-3.5-turbo".to_string(),
            messages: vec![],
            temperature: None,
            max_tokens: None,
            stream: None,
            functions: None,
            function_call: None,
            tools: None,
            tool_choice: None,
            top_p: None,
            presence_penalty: None,
            frequency_penalty: None,
            stop: None,
            response_format: None,
            seed: None,
            logit_bias: None,
            user: None,
            extra_headers: None,
            extra_body: None,
            thinking: None,
        };

        let json = serde_json::to_value(&request).unwrap();
        assert_eq!(json["model"], "gpt-3.5-turbo");
        assert!(json["temperature"].is_null());
    }

    // ==================== ChatMessage Tests ====================

    #[test]
    fn test_chat_message_user() {
        let message = ChatMessage {
            role: "user".to_string(),
            content: Some(json!("Hello, world!")),
            name: None,
            function_call: None,
            tool_calls: None,
            tool_call_id: None,
        };

        let json = serde_json::to_value(&message).unwrap();
        assert_eq!(json["role"], "user");
        assert_eq!(json["content"], "Hello, world!");
    }

    #[test]
    fn test_chat_message_assistant_with_tool_calls() {
        let message = ChatMessage {
            role: "assistant".to_string(),
            content: None,
            name: None,
            function_call: None,
            tool_calls: Some(vec![ToolCall {
                id: "call_abc123".to_string(),
                tool_type: "function".to_string(),
                function: FunctionCallResponse {
                    name: "get_weather".to_string(),
                    arguments: r#"{"location": "NYC"}"#.to_string(),
                },
            }]),
            tool_call_id: None,
        };

        let json = serde_json::to_value(&message).unwrap();
        assert_eq!(json["role"], "assistant");
        assert_eq!(json["tool_calls"][0]["id"], "call_abc123");
        assert_eq!(json["tool_calls"][0]["function"]["name"], "get_weather");
    }

    #[test]
    fn test_chat_message_tool_response() {
        let message = ChatMessage {
            role: "tool".to_string(),
            content: Some(json!("Weather: Sunny, 72°F")),
            name: None,
            function_call: None,
            tool_calls: None,
            tool_call_id: Some("call_abc123".to_string()),
        };

        let json = serde_json::to_value(&message).unwrap();
        assert_eq!(json["role"], "tool");
        assert_eq!(json["tool_call_id"], "call_abc123");
    }

    // ==================== Function Tests ====================

    #[test]
    fn test_function_serialization() {
        let function = Function {
            name: "get_weather".to_string(),
            description: Some("Get weather information".to_string()),
            parameters: json!({
                "type": "object",
                "properties": {
                    "location": {"type": "string"}
                }
            }),
        };

        let json = serde_json::to_value(&function).unwrap();
        assert_eq!(json["name"], "get_weather");
        assert_eq!(json["description"], "Get weather information");
    }

    // ==================== Tool Tests ====================

    #[test]
    fn test_tool_serialization() {
        let tool = Tool {
            tool_type: "function".to_string(),
            function: Function {
                name: "search".to_string(),
                description: Some("Search the web".to_string()),
                parameters: json!({"type": "object"}),
            },
        };

        let json = serde_json::to_value(&tool).unwrap();
        assert_eq!(json["type"], "function");
        assert_eq!(json["function"]["name"], "search");
    }

    // ==================== ToolCall Tests ====================

    #[test]
    fn test_tool_call_serialization() {
        let tool_call = ToolCall {
            id: "call_123".to_string(),
            tool_type: "function".to_string(),
            function: FunctionCallResponse {
                name: "calculate".to_string(),
                arguments: r#"{"a": 1, "b": 2}"#.to_string(),
            },
        };

        let json = serde_json::to_value(&tool_call).unwrap();
        assert_eq!(json["id"], "call_123");
        assert_eq!(json["type"], "function");
        assert_eq!(json["function"]["name"], "calculate");
    }

    // ==================== ResponseFormat Tests ====================

    #[test]
    fn test_response_format_json() {
        let format = ResponseFormat {
            format_type: "json_object".to_string(),
        };

        let json = serde_json::to_value(&format).unwrap();
        assert_eq!(json["type"], "json_object");
    }

    #[test]
    fn test_response_format_text() {
        let format = ResponseFormat {
            format_type: "text".to_string(),
        };

        let json = serde_json::to_value(&format).unwrap();
        assert_eq!(json["type"], "text");
    }

    // ==================== ChatResponse Tests ====================

    #[test]
    fn test_chat_response_serialization() {
        let response = ChatResponse {
            id: "chatcmpl-123".to_string(),
            object: "chat.completion".to_string(),
            created: 1699472400,
            model: "gpt-4".to_string(),
            choices: vec![Choice {
                index: 0,
                message: ChatMessage {
                    role: "assistant".to_string(),
                    content: Some(json!("Hello!")),
                    name: None,
                    function_call: None,
                    tool_calls: None,
                    tool_call_id: None,
                },
                finish_reason: Some("stop".to_string()),
                logprobs: None,
            }],
            usage: Some(Usage {
                prompt_tokens: 10,
                completion_tokens: 5,
                total_tokens: 15,
            }),
            system_fingerprint: Some("fp_abc123".to_string()),
        };

        let json = serde_json::to_value(&response).unwrap();
        assert_eq!(json["id"], "chatcmpl-123");
        assert_eq!(json["choices"][0]["message"]["content"], "Hello!");
        assert_eq!(json["usage"]["total_tokens"], 15);
    }

    // ==================== Usage Tests ====================

    #[test]
    fn test_usage_serialization() {
        let usage = Usage {
            prompt_tokens: 100,
            completion_tokens: 50,
            total_tokens: 150,
        };

        let json = serde_json::to_value(&usage).unwrap();
        assert_eq!(json["prompt_tokens"], 100);
        assert_eq!(json["completion_tokens"], 50);
        assert_eq!(json["total_tokens"], 150);
    }

    // ==================== EmbeddingRequest Tests ====================

    #[test]
    fn test_embedding_request_string_input() {
        let request = EmbeddingRequest {
            model: "text-embedding-ada-002".to_string(),
            input: EmbeddingInput::String("Hello world".to_string()),
            encoding_format: None,
            dimensions: None,
            user: None,
        };

        let json = serde_json::to_value(&request).unwrap();
        assert_eq!(json["model"], "text-embedding-ada-002");
        assert_eq!(json["input"], "Hello world");
    }

    #[test]
    fn test_embedding_request_array_input() {
        let request = EmbeddingRequest {
            model: "text-embedding-3-small".to_string(),
            input: EmbeddingInput::Strings(vec!["Hello".to_string(), "World".to_string()]),
            encoding_format: Some("float".to_string()),
            dimensions: Some(256),
            user: None,
        };

        let json = serde_json::to_value(&request).unwrap();
        assert_eq!(json["dimensions"], 256);
    }

    #[test]
    fn test_embedding_request_token_input() {
        let request = EmbeddingRequest {
            model: "text-embedding-ada-002".to_string(),
            input: EmbeddingInput::Tokens(vec![1, 2, 3, 4]),
            encoding_format: None,
            dimensions: None,
            user: None,
        };

        let json = serde_json::to_value(&request).unwrap();
        assert!(json["input"].is_array());
    }

    // ==================== EmbeddingResponse Tests ====================

    #[test]
    fn test_embedding_response_serialization() {
        let response = EmbeddingResponse {
            object: "list".to_string(),
            data: vec![EmbeddingData {
                object: "embedding".to_string(),
                embedding: vec![0.1, 0.2, 0.3],
                index: 0,
            }],
            model: "text-embedding-ada-002".to_string(),
            usage: Usage {
                prompt_tokens: 5,
                completion_tokens: 0,
                total_tokens: 5,
            },
        };

        let json = serde_json::to_value(&response).unwrap();
        assert_eq!(json["object"], "list");
        assert_eq!(json["data"][0]["embedding"][0], 0.1);
    }

    // ==================== ProviderRequest Tests ====================

    #[test]
    fn test_provider_request_serialization() {
        let request = ProviderRequest {
            endpoint: "/v1/chat/completions".to_string(),
            method: "POST".to_string(),
            headers: HashMap::from([("Content-Type".to_string(), "application/json".to_string())]),
            body: json!({"model": "gpt-4"}),
            query_params: HashMap::new(),
        };

        let json = serde_json::to_value(&request).unwrap();
        assert_eq!(json["endpoint"], "/v1/chat/completions");
        assert_eq!(json["method"], "POST");
    }

    // ==================== ProviderResponse Tests ====================

    #[test]
    fn test_provider_response_serialization() {
        let response = ProviderResponse {
            status_code: 200,
            headers: HashMap::from([("content-type".to_string(), "application/json".to_string())]),
            body: json!({"id": "test"}),
            latency_ms: 150.5,
        };

        let json = serde_json::to_value(&response).unwrap();
        assert_eq!(json["status_code"], 200);
        assert_eq!(json["latency_ms"], 150.5);
    }

    // ==================== TransformMetadata Tests ====================

    #[test]
    fn test_transform_metadata_serialization() {
        let metadata = TransformMetadata {
            provider_type: ProviderType::Anthropic,
            original_model: "gpt-4".to_string(),
            transformed_model: "claude-3-sonnet".to_string(),
            transformations_applied: vec!["message_transform".to_string()],
            warnings: vec!["Some features not supported".to_string()],
            cost_estimate: Some(0.01),
        };

        let json = serde_json::to_value(&metadata).unwrap();
        assert_eq!(json["original_model"], "gpt-4");
        assert_eq!(json["transformed_model"], "claude-3-sonnet");
        assert_eq!(json["cost_estimate"], 0.01);
    }

    // ==================== ModelMapping Tests ====================

    #[test]
    fn test_model_mapping_serialization() {
        let mapping = ModelMapping {
            provider_model: "claude-3-sonnet-20240229".to_string(),
            openai_equivalent: "gpt-4".to_string(),
            capabilities: vec!["chat".to_string(), "vision".to_string()],
            parameter_mappings: HashMap::from([(
                "max_tokens".to_string(),
                "max_tokens".to_string(),
            )]),
        };

        let json = serde_json::to_value(&mapping).unwrap();
        assert_eq!(json["provider_model"], "claude-3-sonnet-20240229");
        assert_eq!(json["capabilities"][0], "chat");
    }

    // ==================== TransformContext Tests ====================

    #[test]
    fn test_transform_context_creation() {
        let context = TransformContext {
            provider_type: ProviderType::VertexAI,
            original_model: "gpt-4".to_string(),
            target_model: "gemini-pro".to_string(),
            config: HashMap::new(),
            metadata: HashMap::from([("request_id".to_string(), "req-123".to_string())]),
        };

        assert_eq!(context.original_model, "gpt-4");
        assert_eq!(context.target_model, "gemini-pro");
        assert_eq!(
            context.metadata.get("request_id"),
            Some(&"req-123".to_string())
        );
    }

    // ==================== DefaultTransformEngine Tests ====================

    #[test]
    fn test_default_transform_engine_new() {
        let engine = DefaultTransformEngine::new();

        // Should have initialized default pipelines
        let anthropic_transforms = engine.get_supported_transformations(&ProviderType::Anthropic);
        assert!(!anthropic_transforms.is_empty());

        let vertexai_transforms = engine.get_supported_transformations(&ProviderType::VertexAI);
        assert!(!vertexai_transforms.is_empty());
    }

    #[test]
    fn test_default_transform_engine_model_mapping_anthropic() {
        let engine = DefaultTransformEngine::new();

        // Claude model should pass through
        let mapped = engine.map_model_name("claude-3-opus", &ProviderType::Anthropic);
        assert_eq!(mapped, "claude-3-opus");

        // Non-Claude model should get default
        let mapped = engine.map_model_name("gpt-4", &ProviderType::Anthropic);
        assert_eq!(mapped, "claude-3-sonnet-20240229");
    }

    #[test]
    fn test_default_transform_engine_model_mapping_vertexai() {
        let engine = DefaultTransformEngine::new();

        // Gemini model should pass through
        let mapped = engine.map_model_name("gemini-1.5-pro", &ProviderType::VertexAI);
        assert_eq!(mapped, "gemini-1.5-pro");

        // Non-Gemini model should get default
        let mapped = engine.map_model_name("gpt-4", &ProviderType::VertexAI);
        assert_eq!(mapped, "gemini-1.0-pro");
    }

    #[test]
    fn test_default_transform_engine_model_mapping_other() {
        let engine = DefaultTransformEngine::new();

        // Other providers should pass model through unchanged
        let mapped = engine.map_model_name("custom-model", &ProviderType::OpenAI);
        assert_eq!(mapped, "custom-model");
    }

    #[tokio::test]
    async fn test_validate_request_compatibility_anthropic() {
        let engine = DefaultTransformEngine::new();

        let request = TransformChatRequest {
            model: "claude-3".to_string(),
            messages: vec![],
            temperature: None,
            max_tokens: None,
            stream: None,
            functions: Some(vec![]), // Anthropic doesn't support functions
            function_call: None,
            tools: None,
            tool_choice: None,
            top_p: None,
            presence_penalty: None,
            frequency_penalty: None,
            stop: None,
            response_format: None,
            seed: None,
            logit_bias: Some(HashMap::new()), // Also not supported
            user: None,
            extra_headers: None,
            extra_body: None,
            thinking: None,
        };

        let issues = engine
            .validate_request_compatibility(&request, &ProviderType::Anthropic)
            .await
            .unwrap();
        assert!(issues.iter().any(|i| i.contains("Functions")));
        assert!(issues.iter().any(|i| i.contains("Logit bias")));
    }

    #[tokio::test]
    async fn test_validate_request_compatibility_vertexai() {
        let engine = DefaultTransformEngine::new();

        let request = TransformChatRequest {
            model: "gemini-pro".to_string(),
            messages: vec![],
            temperature: None,
            max_tokens: None,
            stream: None,
            functions: Some(vec![]),
            function_call: None,
            tools: None,
            tool_choice: None,
            top_p: None,
            presence_penalty: None,
            frequency_penalty: None,
            stop: None,
            response_format: None,
            seed: None,
            logit_bias: None,
            user: None,
            extra_headers: None,
            extra_body: None,
            thinking: None,
        };

        let issues = engine
            .validate_request_compatibility(&request, &ProviderType::VertexAI)
            .await
            .unwrap();
        assert!(issues.iter().any(|i| i.contains("Function calling")));
    }

    #[tokio::test]
    async fn test_validate_request_compatibility_no_issues() {
        let engine = DefaultTransformEngine::new();

        let request = TransformChatRequest {
            model: "gpt-4".to_string(),
            messages: vec![],
            temperature: None,
            max_tokens: None,
            stream: None,
            functions: None,
            function_call: None,
            tools: None,
            tool_choice: None,
            top_p: None,
            presence_penalty: None,
            frequency_penalty: None,
            stop: None,
            response_format: None,
            seed: None,
            logit_bias: None,
            user: None,
            extra_headers: None,
            extra_body: None,
            thinking: None,
        };

        let issues = engine
            .validate_request_compatibility(&request, &ProviderType::OpenAI)
            .await
            .unwrap();
        assert!(issues.is_empty());
    }

    // ==================== Transform Trait Tests ====================

    #[tokio::test]
    async fn test_anthropic_message_transform_name() {
        let transform = AnthropicMessageTransform::new();
        assert_eq!(transform.name(), "anthropic_message_transform");
    }

    #[tokio::test]
    async fn test_anthropic_parameter_transform_name() {
        let transform = AnthropicParameterTransform::new();
        assert_eq!(transform.name(), "anthropic_parameter_transform");
    }

    #[tokio::test]
    async fn test_google_message_transform_name() {
        let transform = GoogleMessageTransform::new();
        assert_eq!(transform.name(), "google_message_transform");
    }

    #[tokio::test]
    async fn test_google_parameter_transform_name() {
        let transform = GoogleParameterTransform::new();
        assert_eq!(transform.name(), "google_parameter_transform");
    }

    #[tokio::test]
    async fn test_transform_passthrough() {
        let transform = AnthropicMessageTransform::new();
        let context = TransformContext {
            provider_type: ProviderType::Anthropic,
            original_model: "gpt-4".to_string(),
            target_model: "claude-3".to_string(),
            config: HashMap::new(),
            metadata: HashMap::new(),
        };

        let input = json!({"messages": [{"role": "user", "content": "Hello"}]});
        let result = transform
            .transform_request(input.clone(), &context)
            .await
            .unwrap();

        // Current implementation is passthrough
        assert_eq!(result, input);
    }

    // ==================== Clone and Debug Tests ====================

    #[test]
    fn test_chat_request_clone() {
        let request = TransformChatRequest {
            model: "gpt-4".to_string(),
            messages: vec![],
            temperature: Some(0.5),
            max_tokens: None,
            stream: None,
            functions: None,
            function_call: None,
            tools: None,
            tool_choice: None,
            top_p: None,
            presence_penalty: None,
            frequency_penalty: None,
            stop: None,
            response_format: None,
            seed: None,
            logit_bias: None,
            user: None,
            extra_headers: None,
            extra_body: None,
            thinking: None,
        };

        let cloned = request.clone();
        assert_eq!(request.model, cloned.model);
        assert_eq!(request.temperature, cloned.temperature);
    }

    #[test]
    fn test_chat_response_debug() {
        let response = ChatResponse {
            id: "test-id".to_string(),
            object: "chat.completion".to_string(),
            created: 12345,
            model: "gpt-4".to_string(),
            choices: vec![],
            usage: None,
            system_fingerprint: None,
        };

        let debug = format!("{:?}", response);
        assert!(debug.contains("ChatResponse"));
        assert!(debug.contains("test-id"));
    }

    #[test]
    fn test_transform_context_clone() {
        let context = TransformContext {
            provider_type: ProviderType::OpenAI,
            original_model: "model-a".to_string(),
            target_model: "model-b".to_string(),
            config: HashMap::new(),
            metadata: HashMap::new(),
        };

        let cloned = context.clone();
        assert_eq!(context.original_model, cloned.original_model);
    }
}
