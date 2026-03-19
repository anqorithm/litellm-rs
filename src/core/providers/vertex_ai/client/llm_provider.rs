//! LLMProvider trait implementation for VertexAIProvider
//!
//! Contains the trait impl and supporting methods (health check, cost calculation,
//! parameter mapping, request/response transformation, error mapper).

use async_trait::async_trait;
use serde_json::Value;
use std::collections::HashMap;

use crate::ProviderError;
use crate::core::{
    traits::{error_mapper::trait_def::ErrorMapper, provider::LLMProvider},
    types::{
        chat::ChatRequest,
        context::RequestContext,
        embedding::EmbeddingRequest,
        health::HealthStatus,
        image::ImageGenerationRequest,
        model::{ModelInfo, ProviderCapability},
        responses::{ChatResponse, EmbeddingResponse, ImageGenerationResponse},
    },
};

use super::{VertexAIErrorMapper, VertexAIProvider};

#[async_trait]
impl LLMProvider for VertexAIProvider {
    fn name(&self) -> &'static str {
        "vertex_ai"
    }

    fn capabilities(&self) -> &'static [crate::core::types::model::ProviderCapability] {
        use crate::core::types::model::ProviderCapability;
        &[
            ProviderCapability::ChatCompletion,
            ProviderCapability::ChatCompletionStream,
            ProviderCapability::Embeddings,
            ProviderCapability::ImageGeneration,
            ProviderCapability::ToolCalling,
        ]
    }

    fn models(&self) -> &[ModelInfo] {
        use std::sync::LazyLock;
        static MODELS: LazyLock<Vec<ModelInfo>> = LazyLock::new(|| {
            vec![
                ModelInfo {
                    id: "gemini-1.5-pro".to_string(),
                    name: "Gemini 1.5 Pro".to_string(),
                    provider: "vertex_ai".to_string(),
                    max_context_length: 2_097_152,
                    max_output_length: Some(8192),
                    supports_streaming: true,
                    supports_tools: true,
                    supports_multimodal: true,
                    input_cost_per_1k_tokens: Some(1.25),
                    output_cost_per_1k_tokens: Some(3.75),
                    currency: "USD".to_string(),
                    capabilities: vec![
                        ProviderCapability::ChatCompletion,
                        ProviderCapability::ChatCompletionStream,
                        ProviderCapability::FunctionCalling,
                        ProviderCapability::ToolCalling,
                    ],
                    created_at: None,
                    updated_at: None,
                    metadata: std::collections::HashMap::new(),
                },
                ModelInfo {
                    id: "gemini-1.5-flash".to_string(),
                    name: "Gemini 1.5 Flash".to_string(),
                    provider: "vertex_ai".to_string(),
                    max_context_length: 1_048_576,
                    max_output_length: Some(8192),
                    supports_streaming: true,
                    supports_tools: true,
                    supports_multimodal: true,
                    input_cost_per_1k_tokens: Some(0.0625),
                    output_cost_per_1k_tokens: Some(0.25),
                    currency: "USD".to_string(),
                    capabilities: vec![
                        ProviderCapability::ChatCompletion,
                        ProviderCapability::ChatCompletionStream,
                        ProviderCapability::FunctionCalling,
                        ProviderCapability::ToolCalling,
                    ],
                    created_at: None,
                    updated_at: None,
                    metadata: std::collections::HashMap::new(),
                },
            ]
        });
        &MODELS
    }

    async fn chat_completion(
        &self,
        request: ChatRequest,
        context: RequestContext,
    ) -> Result<ChatResponse, ProviderError> {
        self.chat_completion_internal(request, context).await
    }

    async fn embeddings(
        &self,
        request: EmbeddingRequest,
        context: RequestContext,
    ) -> Result<EmbeddingResponse, ProviderError> {
        self.embedding_internal(request, context).await
    }

    async fn image_generation(
        &self,
        request: ImageGenerationRequest,
        _context: RequestContext,
    ) -> Result<ImageGenerationResponse, ProviderError> {
        // Use Imagen model for image generation
        let endpoint = "predict";
        let model = "imagegeneration@006";

        let url = format!(
            "https://{}-aiplatform.googleapis.com/v1/projects/{}/locations/{}/publishers/google/models/{}:{}",
            self.config.location, self.config.project_id, self.config.location, model, endpoint
        );

        let body = serde_json::json!({
            "instances": [{
                "prompt": request.prompt
            }],
            "parameters": {
                "sampleCount": request.n.unwrap_or(1),
                "aspectRatio": request.size.as_deref().unwrap_or("1:1"),
            }
        });

        let response = self.make_request(&url, body).await?;
        let response_body: Value = response
            .json()
            .await
            .map_err(|e| ProviderError::response_parsing("vertex_ai", e.to_string()))?;

        let predictions = response_body["predictions"]
            .as_array()
            .ok_or_else(|| ProviderError::response_parsing("vertex_ai", "Missing predictions"))?;

        let image_data = predictions
            .iter()
            .filter_map(|pred| pred["bytesBase64Encoded"].as_str())
            .map(|s| crate::core::types::responses::ImageData {
                url: None,
                b64_json: Some(s.to_string()),
                revised_prompt: None,
            })
            .collect();

        Ok(ImageGenerationResponse {
            created: chrono::Utc::now().timestamp() as u64,
            data: image_data,
        })
    }

    async fn health_check(&self) -> HealthStatus {
        match self.check_health().await {
            Ok(()) => HealthStatus::Healthy,
            Err(_) => HealthStatus::Unhealthy,
        }
    }

    async fn calculate_cost(
        &self,
        model: &str,
        input_tokens: u32,
        output_tokens: u32,
    ) -> Result<f64, ProviderError> {
        // Basic cost calculation for Vertex AI models (per 1M tokens)
        let cost = match model {
            m if m.contains("gemini-pro") => {
                (input_tokens as f64 * 0.0005 + output_tokens as f64 * 0.0015) / 1000.0
            }
            m if m.contains("gemini-1.5-pro") => {
                (input_tokens as f64 * 0.00125 + output_tokens as f64 * 0.00375) / 1000.0
            }
            m if m.contains("gemini-1.5-flash") => {
                (input_tokens as f64 * 0.000075 + output_tokens as f64 * 0.0003) / 1000.0
            }
            _ => 0.0, // Default cost for unknown models
        };
        Ok(cost)
    }

    /// Model
    fn get_supported_openai_params(&self, model: &str) -> &'static [&'static str] {
        // VertexAI supports OpenAI-compatible parameters for Gemini models
        if model.contains("gemini") {
            &[
                "messages",
                "model",
                "max_tokens",
                "temperature",
                "top_p",
                "stop",
                "stream",
                "tools",
                "tool_choice",
                "response_format",
                "user",
                "top_k",
            ]
        } else {
            // Partner models have limited OpenAI compatibility
            &[
                "messages",
                "model",
                "max_tokens",
                "temperature",
                "top_p",
                "stream",
            ]
        }
    }

    /// Map OpenAI format parameters to VertexAI API parameter format
    async fn map_openai_params(
        &self,
        params: HashMap<String, Value>,
        model: &str,
    ) -> std::result::Result<HashMap<String, Value>, ProviderError> {
        let mut vertex_params = HashMap::new();
        let vertex_model = super::super::parse_vertex_model(model);

        // Basic parameter mapping
        if let Some(messages) = params.get("messages") {
            vertex_params.insert("contents".to_string(), messages.clone());
        }

        vertex_params.insert("model".to_string(), Value::String(vertex_model.model_id()));

        // Generation parameter mapping
        let mut generation_config = serde_json::Map::new();

        if let Some(max_tokens) = params.get("max_tokens") {
            generation_config.insert("maxOutputTokens".to_string(), max_tokens.clone());
        }

        if let Some(temperature) = params.get("temperature") {
            generation_config.insert("temperature".to_string(), temperature.clone());
        }

        if let Some(top_p) = params.get("top_p") {
            generation_config.insert("topP".to_string(), top_p.clone());
        }

        if let Some(top_k) = params.get("top_k") {
            generation_config.insert("topK".to_string(), top_k.clone());
        }

        if let Some(stop) = params.get("stop") {
            match stop {
                Value::String(s) => {
                    generation_config.insert(
                        "stopSequences".to_string(),
                        Value::Array(vec![Value::String(s.clone())]),
                    );
                }
                Value::Array(_arr) => {
                    generation_config.insert("stopSequences".to_string(), stop.clone());
                }
                _ => {
                    return Err(ProviderError::invalid_request(
                        "vertex_ai",
                        "stop must be string or array",
                    ));
                }
            }
        }

        if !generation_config.is_empty() {
            vertex_params.insert(
                "generationConfig".to_string(),
                Value::Object(generation_config),
            );
        }

        // tool_callparameter
        if let Some(tools) = params.get("tools") {
            vertex_params.insert("tools".to_string(), tools.clone());
        }

        if let Some(tool_choice) = params.get("tool_choice") {
            vertex_params.insert(
                "toolConfig".to_string(),
                serde_json::json!({
                    "functionCallingConfig": {
                        "mode": match tool_choice.as_str() {
                            Some("auto") => "AUTO",
                            Some("none") => "NONE",
                            _ => "AUTO"
                        }
                    }
                }),
            );
        }

        Ok(vertex_params)
    }

    /// Request
    async fn transform_request(
        &self,
        request: ChatRequest,
        _context: RequestContext,
    ) -> std::result::Result<Value, ProviderError> {
        let mut params = HashMap::new();

        params.insert(
            "messages".to_string(),
            serde_json::to_value(request.messages)
                .map_err(|e| ProviderError::serialization("vertex_ai", e.to_string()))?,
        );
        params.insert("model".to_string(), Value::String(request.model.clone()));

        if let Some(max_tokens) = request.max_tokens {
            params.insert(
                "max_tokens".to_string(),
                Value::Number(serde_json::Number::from(max_tokens)),
            );
        }

        if let Some(temperature) = request.temperature {
            let temp_f64 = temperature as f64;
            params.insert(
                "temperature".to_string(),
                Value::Number(serde_json::Number::from_f64(temp_f64).ok_or_else(|| {
                    ProviderError::invalid_request(
                        "vertex_ai",
                        format!("invalid temperature value: {temp_f64} (NaN and Infinity are not allowed)"),
                    )
                })?),
            );
        }

        if let Some(top_p) = request.top_p {
            let top_p_f64 = top_p as f64;
            params.insert(
                "top_p".to_string(),
                Value::Number(serde_json::Number::from_f64(top_p_f64).ok_or_else(|| {
                    ProviderError::invalid_request(
                        "vertex_ai",
                        format!(
                            "invalid top_p value: {top_p_f64} (NaN and Infinity are not allowed)"
                        ),
                    )
                })?),
            );
        }

        if let Some(stop) = request.stop {
            params.insert(
                "stop".to_string(),
                serde_json::to_value(stop)
                    .map_err(|e| ProviderError::serialization("vertex_ai", e.to_string()))?,
            );
        }

        if request.stream {
            params.insert("stream".to_string(), Value::Bool(true));
        }

        if let Some(tools) = request.tools {
            params.insert(
                "tools".to_string(),
                serde_json::to_value(tools)
                    .map_err(|e| ProviderError::serialization("vertex_ai", e.to_string()))?,
            );
        }

        if let Some(tool_choice) = request.tool_choice {
            params.insert(
                "tool_choice".to_string(),
                serde_json::to_value(tool_choice)
                    .map_err(|e| ProviderError::serialization("vertex_ai", e.to_string()))?,
            );
        }

        let vertex_params = self.map_openai_params(params, &request.model).await?;
        Ok(serde_json::to_value(vertex_params)
            .map_err(|e| ProviderError::serialization("vertex_ai", e.to_string()))?)
    }

    /// Response
    async fn transform_response(
        &self,
        raw_response: &[u8],
        model: &str,
        _request_id: &str,
    ) -> std::result::Result<ChatResponse, ProviderError> {
        let response_str = std::str::from_utf8(raw_response).map_err(|e| {
            ProviderError::response_parsing("vertex_ai", format!("Invalid UTF-8: {}", e))
        })?;

        let response_json: Value = serde_json::from_str(response_str).map_err(|e| {
            ProviderError::response_parsing("vertex_ai", format!("JSON parsing error: {}", e))
        })?;

        // Error
        if let Some(_error) = response_json.get("error") {
            let error_mapper = self.get_error_mapper();
            return Err(error_mapper.map_json_error(&response_json));
        }

        // Response
        let candidates = response_json
            .get("candidates")
            .and_then(|c| c.as_array())
            .ok_or_else(|| {
                ProviderError::response_parsing("vertex_ai", "Missing candidates in response")
            })?;

        if candidates.is_empty() {
            return Err(ProviderError::response_parsing(
                "vertex_ai",
                "No candidates in response",
            ));
        }

        let candidate = &candidates[0];
        let content = candidate
            .get("content")
            .and_then(|c| c.get("parts"))
            .and_then(|p| p.as_array())
            .and_then(|parts| parts.first())
            .and_then(|part| part.get("text"))
            .and_then(|t| t.as_str())
            .unwrap_or_default()
            .to_string();

        // Usage statistics information
        let usage = response_json.get("usageMetadata").map(|usage_json| {
            let input_tokens = usage_json
                .get("promptTokenCount")
                .and_then(|v| v.as_u64())
                .unwrap_or(0) as u32;
            let output_tokens = usage_json
                .get("candidatesTokenCount")
                .and_then(|v| v.as_u64())
                .unwrap_or(0) as u32;
            crate::core::types::responses::Usage {
                prompt_tokens: input_tokens,
                completion_tokens: output_tokens,
                total_tokens: input_tokens + output_tokens,
                prompt_tokens_details: None,
                completion_tokens_details: None,
                thinking_usage: None,
            }
        });

        Ok(ChatResponse {
            id: format!("vertex-ai-{}", uuid::Uuid::new_v4()),
            object: "chat.completion".to_string(),
            created: chrono::Utc::now().timestamp(),
            model: model.to_string(),
            choices: vec![crate::core::types::responses::ChatChoice {
                index: 0,
                message: crate::core::types::chat::ChatMessage {
                    role: crate::core::types::message::MessageRole::Assistant,
                    content: Some(crate::core::types::message::MessageContent::Text(content)),
                    thinking: None,
                    name: None,
                    tool_calls: None, // Handle
                    tool_call_id: None,
                    function_call: None,
                },
                finish_reason: candidate
                    .get("finishReason")
                    .and_then(|r| r.as_str())
                    .map(|reason| match reason {
                        "STOP" => crate::core::types::responses::FinishReason::Stop,
                        "MAX_TOKENS" => crate::core::types::responses::FinishReason::Length,
                        "SAFETY" => crate::core::types::responses::FinishReason::ContentFilter,
                        "RECITATION" => crate::core::types::responses::FinishReason::ContentFilter,
                        _ => crate::core::types::responses::FinishReason::Stop,
                    })
                    .or(Some(crate::core::types::responses::FinishReason::Stop)),
                logprobs: None,
            }],
            usage,
            system_fingerprint: None,
        })
    }

    /// Error
    fn get_error_mapper(&self) -> Box<dyn ErrorMapper<ProviderError>> {
        Box::new(VertexAIErrorMapper)
    }
}

impl VertexAIProvider {
    /// Internal health check
    pub(crate) async fn check_health(&self) -> Result<(), super::super::error::VertexAIError> {
        // Simple health check by calling countTokens
        let url = format!(
            "https://{}-aiplatform.googleapis.com/v1/projects/{}/locations/{}/publishers/google/models/gemini-1.5-flash:countTokens",
            self.config.location, self.config.project_id, self.config.location
        );

        let body = serde_json::json!({
            "contents": [{
                "parts": [{"text": "test"}]
            }]
        });

        self.make_request(&url, body).await?;
        Ok(())
    }
}
