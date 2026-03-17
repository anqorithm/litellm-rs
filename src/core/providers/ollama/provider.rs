//! Main Ollama Provider Implementation
//!
//! Implements the LLMProvider trait for Ollama's local inference server.

use std::collections::HashMap;
use std::sync::Arc;

use super::config::OllamaConfig;
use super::model_info::{OllamaModelInfo, OllamaShowResponse, OllamaTagsResponse, get_model_info};
use crate::core::providers::base::{GlobalPoolManager, HttpMethod, header};
use crate::core::providers::unified_provider::ProviderError;
use crate::core::traits::{
    provider::ProviderConfig as _, provider::llm_provider::trait_definition::LLMProvider,
};
use crate::core::types::{
    chat::ChatMessage,
    chat::ChatRequest,
    message::MessageContent,
    message::MessageRole,
    model::ModelInfo,
    model::ProviderCapability,
    responses::{ChatChoice, ChatResponse, FinishReason, Usage},
    tools::FunctionCall,
    tools::ToolCall,
};

/// Static capabilities for Ollama provider
const OLLAMA_CAPABILITIES: &[ProviderCapability] = &[
    ProviderCapability::ChatCompletion,
    ProviderCapability::ChatCompletionStream,
    ProviderCapability::Embeddings,
    ProviderCapability::ToolCalling,
];

/// Ollama provider implementation
#[derive(Debug, Clone)]
pub struct OllamaProvider {
    config: OllamaConfig,
    pool_manager: Arc<GlobalPoolManager>,
    models: Vec<ModelInfo>,
}

impl OllamaProvider {
    /// Create a new Ollama provider instance
    pub async fn new(config: OllamaConfig) -> Result<Self, ProviderError> {
        // Validate configuration
        config
            .validate()
            .map_err(|e| ProviderError::configuration("ollama", e))?;

        // Create pool manager
        let pool_manager = Arc::new(GlobalPoolManager::new().map_err(|e| {
            ProviderError::configuration("ollama", format!("Failed to create pool manager: {}", e))
        })?);

        // Initialize with empty models (will be populated on first list_models call)
        let models = Vec::new();

        Ok(Self {
            config,
            pool_manager,
            models,
        })
    }

    /// Create provider with custom API base
    pub async fn with_base_url(base_url: impl Into<String>) -> Result<Self, ProviderError> {
        let config = OllamaConfig {
            api_base: Some(base_url.into()),
            ..Default::default()
        };
        Self::new(config).await
    }

    /// Create provider with default configuration (localhost:11434)
    pub async fn default_local() -> Result<Self, ProviderError> {
        Self::new(OllamaConfig::default()).await
    }

    /// Execute an HTTP request
    async fn execute_request(
        &self,
        url: &str,
        method: HttpMethod,
        body: Option<serde_json::Value>,
    ) -> Result<serde_json::Value, ProviderError> {
        let mut headers = Vec::with_capacity(2);

        // Add auth header if API key is set
        if let Some(api_key) = &self.config.get_api_key() {
            headers.push(header("Authorization", format!("Bearer {}", api_key)));
        }
        headers.push(header("Content-Type", "application/json".to_string()));

        let response = self
            .pool_manager
            .execute_request(url, method, headers, body)
            .await
            .map_err(|e| {
                let error_msg = e.to_string();
                if error_msg.contains("Connection refused") || error_msg.contains("connect error") {
                    ProviderError::network(
                        "ollama",
                        format!(
                            "Failed to connect to Ollama server at {}. Is Ollama running?",
                            self.config.get_api_base()
                        ),
                    )
                } else if error_msg.contains("timed out") || error_msg.contains("timeout") {
                    ProviderError::Timeout {
                        provider: "ollama",
                        message: error_msg,
                    }
                } else {
                    ProviderError::network("ollama", error_msg)
                }
            })?;

        let response_bytes = response
            .bytes()
            .await
            .map_err(|e| ProviderError::network("ollama", e.to_string()))?;

        serde_json::from_slice(&response_bytes).map_err(|e| {
            ProviderError::api_error("ollama", 500, format!("Failed to parse response: {}", e))
        })
    }

    /// List available models from Ollama server
    pub async fn list_models(&self) -> Result<Vec<OllamaModelInfo>, ProviderError> {
        let url = self.config.get_tags_endpoint();
        let response = self.execute_request(&url, HttpMethod::GET, None).await?;

        let tags: OllamaTagsResponse = serde_json::from_value(response).map_err(|e| {
            ProviderError::api_error("ollama", 500, format!("Failed to parse models list: {}", e))
        })?;

        Ok(tags.models.into_iter().map(|m| m.into()).collect())
    }

    /// Get detailed model information
    pub async fn show_model(&self, model: &str) -> Result<OllamaShowResponse, ProviderError> {
        let url = self.config.get_show_endpoint();
        let body = serde_json::json!({ "name": model });

        let response = self
            .execute_request(&url, HttpMethod::POST, Some(body))
            .await?;

        serde_json::from_value(response).map_err(|e| {
            ProviderError::api_error("ollama", 500, format!("Failed to parse model info: {}", e))
        })
    }

    /// Build Ollama chat request from ChatRequest
    fn build_chat_request(
        &self,
        request: &ChatRequest,
        stream: bool,
    ) -> Result<serde_json::Value, ProviderError> {
        let mut messages = Vec::new();

        for msg in &request.messages {
            let role = match &msg.role {
                MessageRole::System => "system",
                MessageRole::User => "user",
                MessageRole::Assistant => "assistant",
                MessageRole::Tool => "tool",
                MessageRole::Function => "function",
            };

            let mut message = serde_json::json!({
                "role": role,
            });

            // Handle content
            match &msg.content {
                Some(MessageContent::Text(text)) => {
                    message["content"] = serde_json::json!(text);
                }
                Some(MessageContent::Parts(parts)) => {
                    // Handle multimodal content
                    let mut images = Vec::new();
                    let mut text_parts = Vec::new();

                    for part in parts {
                        match part {
                            crate::core::types::content::ContentPart::Text { text } => {
                                text_parts.push(text.clone());
                            }
                            crate::core::types::content::ContentPart::ImageUrl { image_url } => {
                                // Extract base64 image data from data URL or URL
                                let url = &image_url.url;
                                if url.starts_with("data:") {
                                    // Extract base64 data
                                    if let Some(comma_pos) = url.find(',') {
                                        let base64_data = &url[comma_pos + 1..];
                                        images.push(base64_data.to_string());
                                    }
                                } else {
                                    // Regular URL - Ollama expects base64, so this might not work
                                    images.push(url.clone());
                                }
                            }
                            crate::core::types::content::ContentPart::Image { source, .. } => {
                                // Base64 encoded image
                                images.push(source.data.clone());
                            }
                            // Skip unsupported content types (Audio, Document, ToolResult, ToolUse)
                            _ => {}
                        }
                    }

                    message["content"] = serde_json::json!(text_parts.join("\n"));
                    if !images.is_empty() {
                        message["images"] = serde_json::json!(images);
                    }
                }
                None => {
                    message["content"] = serde_json::json!("");
                }
            }

            // Handle tool calls for assistant messages
            if let Some(tool_calls) = &msg.tool_calls {
                let ollama_tool_calls: Vec<_> = tool_calls
                    .iter()
                    .map(|tc| {
                        serde_json::json!({
                            "function": {
                                "name": tc.function.name,
                                "arguments": tc.function.arguments
                            }
                        })
                    })
                    .collect();
                message["tool_calls"] = serde_json::json!(ollama_tool_calls);
            }

            // Handle tool call id for tool messages
            if msg.role == MessageRole::Tool
                && let Some(name) = &msg.name
            {
                message["name"] = serde_json::json!(name);
            }

            messages.push(message);
        }

        // Build the request body
        let mut body = serde_json::json!({
            "model": request.model.strip_prefix("ollama/").unwrap_or(&request.model),
            "messages": messages,
            "stream": stream,
        });

        // Add options from request parameters
        let mut options = self.config.build_options();
        if let serde_json::Value::Object(ref mut opts) = options {
            if let Some(temp) = request.temperature {
                opts.insert("temperature".to_string(), serde_json::json!(temp));
            }
            if let Some(top_p) = request.top_p {
                opts.insert("top_p".to_string(), serde_json::json!(top_p));
            }
            if let Some(max_tokens) = request.max_tokens {
                opts.insert("num_predict".to_string(), serde_json::json!(max_tokens));
            }
            if let Some(stop) = &request.stop {
                opts.insert("stop".to_string(), serde_json::json!(stop));
            }
            if let Some(freq_penalty) = request.frequency_penalty {
                opts.insert(
                    "frequency_penalty".to_string(),
                    serde_json::json!(freq_penalty),
                );
            }
            if let Some(pres_penalty) = request.presence_penalty {
                opts.insert(
                    "presence_penalty".to_string(),
                    serde_json::json!(pres_penalty),
                );
            }
            if let Some(seed) = request.seed {
                opts.insert("seed".to_string(), serde_json::json!(seed));
            }
        }
        body["options"] = options;

        // Add tools if present
        if let Some(tools) = &request.tools {
            let ollama_tools: Vec<_> = tools
                .iter()
                .map(|t| {
                    serde_json::json!({
                        "type": "function",
                        "function": {
                            "name": t.function.name,
                            "description": t.function.description,
                            "parameters": t.function.parameters
                        }
                    })
                })
                .collect();
            body["tools"] = serde_json::json!(ollama_tools);
        }

        // Add response format if set
        if let Some(format) = &request.response_format
            && format.format_type == "json_object"
        {
            body["format"] = serde_json::json!("json");
        }

        // Add keep_alive if set in config
        if let Some(keep_alive) = &self.config.keep_alive {
            body["keep_alive"] = serde_json::json!(keep_alive);
        }

        Ok(body)
    }

    /// Parse Ollama chat response into ChatResponse
    fn parse_chat_response(
        &self,
        response: serde_json::Value,
        model: &str,
    ) -> Result<ChatResponse, ProviderError> {
        let message = response.get("message").ok_or_else(|| {
            ProviderError::api_error("ollama", 500, "Missing message in response".to_string())
        })?;

        let content = message
            .get("content")
            .and_then(|c| c.as_str())
            .map(|s| s.to_string());

        // Parse thinking content if present
        let thinking = message
            .get("thinking")
            .and_then(|t| t.as_str())
            .map(crate::core::types::thinking::ThinkingContent::text);

        // Parse tool calls if present
        let tool_calls = if let Some(tcs) = message.get("tool_calls").and_then(|v| v.as_array()) {
            let calls: Vec<_> = tcs
                .iter()
                .map(|tc| {
                    let func = tc
                        .get("function")
                        .cloned()
                        .unwrap_or_else(|| serde_json::json!({}));
                    ToolCall {
                        id: format!("call_{}", uuid::Uuid::new_v4()),
                        tool_type: "function".to_string(),
                        function: FunctionCall {
                            name: func
                                .get("name")
                                .and_then(|n| n.as_str())
                                .unwrap_or("")
                                .to_string(),
                            arguments: func
                                .get("arguments")
                                .map(|a| a.to_string())
                                .unwrap_or_default(),
                        },
                    }
                })
                .collect();
            if calls.is_empty() { None } else { Some(calls) }
        } else {
            None
        };

        // Determine finish reason
        let done_reason_str = response
            .get("done_reason")
            .and_then(|r| r.as_str())
            .unwrap_or("stop");
        let finish_reason = match done_reason_str {
            "stop" => FinishReason::Stop,
            "length" => FinishReason::Length,
            "tool_calls" => FinishReason::ToolCalls,
            "content_filter" => FinishReason::ContentFilter,
            "function_call" => FinishReason::FunctionCall,
            _ => FinishReason::Stop,
        };

        // Build usage info
        let usage = Usage {
            prompt_tokens: response
                .get("prompt_eval_count")
                .and_then(|v| v.as_u64())
                .unwrap_or(0) as u32,
            completion_tokens: response
                .get("eval_count")
                .and_then(|v| v.as_u64())
                .unwrap_or(0) as u32,
            total_tokens: response
                .get("prompt_eval_count")
                .and_then(|v| v.as_u64())
                .unwrap_or(0) as u32
                + response
                    .get("eval_count")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0) as u32,
            prompt_tokens_details: None,
            completion_tokens_details: None,
            thinking_usage: None,
        };

        Ok(ChatResponse {
            id: format!("ollama-{}", uuid::Uuid::new_v4()),
            object: "chat.completion".to_string(),
            created: chrono::Utc::now().timestamp(),
            model: format!(
                "ollama/{}",
                response
                    .get("model")
                    .and_then(|m| m.as_str())
                    .unwrap_or(model)
            ),
            system_fingerprint: None,
            choices: vec![ChatChoice {
                index: 0,
                message: ChatMessage {
                    role: MessageRole::Assistant,
                    content: content.map(MessageContent::Text),
                    thinking,
                    tool_calls,
                    function_call: None,
                    name: None,
                    tool_call_id: None,
                },
                finish_reason: Some(finish_reason),
                logprobs: None,
            }],
            usage: Some(usage),
        })
    }
}

// Additional utility methods
impl OllamaProvider {
    /// Check if Ollama server is running
    pub async fn is_server_running(&self) -> bool {
        self.list_models().await.is_ok()
    }

    /// Get model info from server
    pub async fn get_model_info(&self, model: &str) -> Result<OllamaModelInfo, ProviderError> {
        // First try to get detailed info from show endpoint
        match self.show_model(model).await {
            Ok(show_response) => {
                let mut info = get_model_info(model);

                // Enrich with server data
                if let Some(ctx_len) = show_response.get_context_length() {
                    info.max_context_length = Some(ctx_len);
                }
                if show_response.supports_tools() {
                    info.supports_tools = true;
                }
                if let Some(details) = show_response.details {
                    info.family = details.family;
                    info.parameter_size = details.parameter_size;
                    info.quantization = details.quantization_level;
                }

                Ok(info)
            }
            Err(_) => {
                // Fall back to inferred info
                Ok(get_model_info(model))
            }
        }
    }

    /// Refresh model list from server
    pub async fn refresh_models(&mut self) -> Result<(), ProviderError> {
        let ollama_models = self.list_models().await?;

        self.models = ollama_models
            .into_iter()
            .map(|m| ModelInfo {
                id: m.name.clone(),
                name: m.display_name.clone(),
                provider: "ollama".to_string(),
                max_context_length: m.max_context_length.unwrap_or(4096),
                max_output_length: None,
                supports_streaming: true,
                supports_tools: m.supports_tools,
                supports_multimodal: m.supports_multimodal,
                input_cost_per_1k_tokens: Some(0.0), // Ollama is free
                output_cost_per_1k_tokens: Some(0.0), // Ollama is free
                currency: "USD".to_string(),
                capabilities: vec![
                    ProviderCapability::ChatCompletion,
                    ProviderCapability::ChatCompletionStream,
                    ProviderCapability::Embeddings,
                ],
                created_at: None,
                updated_at: None,
                metadata: HashMap::new(),
            })
            .collect();

        Ok(())
    }
}
