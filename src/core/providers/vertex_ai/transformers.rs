//! Request/Response transformers for Vertex AI models

use crate::ProviderError;
use crate::core::types::responses::FinishReason;
use crate::core::types::{
    chat::ChatMessage,
    chat::ChatRequest,
    message::MessageContent,
    message::MessageRole,
    responses::{ChatChoice, ChatResponse, Usage},
};
use serde_json::{Value, json};

use super::{
    common_utils::{Content, FunctionDeclaration, GenerationConfig, Part, Tool, convert_role},
    models::VertexAIModel,
};

/// Transformer for Gemini models
#[derive(Debug, Clone, Default)]
pub struct GeminiTransformer;

impl GeminiTransformer {
    pub fn new() -> Self {
        Self
    }

    /// Transform chat request to Gemini format
    pub fn transform_chat_request(
        &self,
        request: &ChatRequest,
        _model: &VertexAIModel,
    ) -> Result<Value, ProviderError> {
        let mut contents = Vec::new();
        let mut system_instruction = None;

        // Process messages
        for message in &request.messages {
            match message.role {
                MessageRole::System => {
                    // Gemini uses system instruction separately
                    if let Some(ref content) = message.content {
                        system_instruction = Some(self.message_content_to_parts(content)?);
                    }
                }
                _ => {
                    let role = convert_role(&message.role.to_string());
                    let parts = if let Some(ref content) = message.content {
                        self.message_content_to_parts(content)?
                    } else {
                        vec![]
                    };

                    contents.push(Content { role, parts });
                }
            }
        }

        // Build generation config
        let mut generation_config = GenerationConfig {
            temperature: request.temperature,
            top_p: request.top_p,
            top_k: None,
            max_output_tokens: request.max_tokens.map(|v| v as i32),
            stop_sequences: request.stop.clone(),
            response_mime_type: None,
            response_schema: None,
        };

        // Handle JSON mode / response format
        if let Some(ref format) = request.response_format {
            if format.response_type == Some("json_object".to_string()) {
                generation_config.response_mime_type = Some("application/json".to_string());
                if let Some(ref schema) = format.json_schema {
                    generation_config.response_schema = Some(serde_json::to_value(schema)?);
                }
            }
        }

        // Handle tools/functions
        let tools = if let Some(ref tools) = request.tools {
            Some(vec![Tool {
                function_declarations: tools
                    .iter()
                    .map(|tool| FunctionDeclaration {
                        name: tool.function.name.clone(),
                        description: tool.function.description.clone().unwrap_or_default(),
                        parameters: tool.function.parameters.clone().unwrap_or(json!({})),
                    })
                    .collect(),
            }])
        } else {
            None
        };

        // Build request body
        let mut body = json!({
            "contents": contents,
            "generationConfig": generation_config,
        });

        if let Some(system) = system_instruction {
            body["systemInstruction"] = json!({
                "parts": system
            });
        }

        if let Some(tools) = tools {
            body["tools"] = serde_json::to_value(tools)?;
        }

        Ok(body)
    }

    /// Convert message content to Gemini parts
    fn message_content_to_parts(
        &self,
        content: &MessageContent,
    ) -> Result<Vec<Part>, ProviderError> {
        match content {
            MessageContent::Text(text) => Ok(vec![Part::Text { text: text.clone() }]),
            MessageContent::Parts(parts) => {
                parts.iter().map(|part| {
                    match part {
                        crate::core::types::content::ContentPart::Text { text } => {
                            Ok(Part::Text { text: text.clone() })
                        }
                        crate::core::types::content::ContentPart::Image { image_url, source: _source, detail: _detail } => {
                            // Parse image URL - could be base64 or URL
                            if let Some(url) = &image_url.as_ref().map(|u| &u.url) {
                                if let Some(base64_data) = url.strip_prefix("data:") {
                                    let parts: Vec<&str> = base64_data.splitn(2, ',').collect();
                                    if parts.len() == 2 {
                                        let mime_type = parts[0].replace(";base64", "");
                                        Ok(Part::InlineData {
                                            inline_data: super::common_utils::InlineData {
                                                mime_type,
                                                data: parts[1].to_string(),
                                            }
                                        })
                                    } else {
                                        Err(ProviderError::invalid_request("vertex_ai", "Invalid base64 image"))
                                    }
                                } else {
                                    // File URL
                                    Ok(Part::FileData {
                                        file_data: super::common_utils::FileData {
                                            mime_type: "image/jpeg".to_string(), // Default
                                            file_uri: url.to_string(),
                                        }
                                    })
                                }
                            } else {
                                Err(ProviderError::invalid_request("vertex_ai", "Missing image URL"))
                            }
                        }
                        crate::core::types::content::ContentPart::ImageUrl { image_url } => {
                            // Handle ImageUrl variant
                            if let Some(base64_data) = image_url.url.strip_prefix("data:") {
                                let parts: Vec<&str> = base64_data.splitn(2, ',').collect();
                                if parts.len() == 2 {
                                    let mime_type = parts[0].replace(";base64", "");
                                    Ok(Part::InlineData {
                                        inline_data: crate::core::providers::vertex_ai::common_utils::InlineData {
                                            mime_type,
                                            data: parts[1].to_string(),
                                        },
                                    })
                                } else {
                                    Err(ProviderError::invalid_request("vertex_ai", "Invalid base64 format"))
                                }
                            } else {
                                Err(ProviderError::invalid_request("vertex_ai", "Only base64 images supported"))
                            }
                        }
                        crate::core::types::content::ContentPart::Audio { audio: _audio } => {
                            // Vertex AI doesn't directly support audio in chat completions
                            // This would need to be handled via separate audio APIs
                            Err(ProviderError::invalid_request("vertex_ai", "Audio content not supported in chat completions"))
                        }
                        crate::core::types::content::ContentPart::Document { .. } => {
                            Err(ProviderError::invalid_request("vertex_ai", "Document content not supported"))
                        }
                        crate::core::types::content::ContentPart::ToolResult { .. } => {
                            Err(ProviderError::invalid_request("vertex_ai", "ToolResult should be handled separately"))
                        }
                        crate::core::types::content::ContentPart::ToolUse { .. } => {
                            Err(ProviderError::invalid_request("vertex_ai", "ToolUse should be handled separately"))
                        }
                    }
                }).collect()
            }
        }
    }

    /// Transform Gemini response to standard format
    pub fn transform_chat_response(
        &self,
        response: Value,
        model: &VertexAIModel,
    ) -> Result<ChatResponse, ProviderError> {
        let candidates = response["candidates"]
            .as_array()
            .ok_or_else(|| ProviderError::response_parsing("vertex_ai", "Missing candidates"))?;

        if candidates.is_empty() {
            return Err(ProviderError::response_parsing(
                "vertex_ai",
                "No candidates in response",
            ));
        }

        let candidate = &candidates[0];
        let content = &candidate["content"];

        // Extract text from parts
        let mut text_parts = Vec::new();
        if let Some(parts) = content["parts"].as_array() {
            for part in parts {
                if let Some(text) = part["text"].as_str() {
                    text_parts.push(text.to_string());
                }
            }
        }

        let message_content = if text_parts.is_empty() {
            None
        } else {
            Some(MessageContent::Text(text_parts.join("")))
        };

        // Parse finish reason
        let finish_reason = candidate["finishReason"]
            .as_str()
            .map(|reason| match reason {
                "STOP" => FinishReason::Stop,
                "MAX_TOKENS" => FinishReason::Length,
                "SAFETY" => FinishReason::ContentFilter,
                "RECITATION" => FinishReason::ContentFilter,
                _ => FinishReason::Stop,
            });

        // Parse usage
        let usage = response.get("usageMetadata").map(|usage_metadata| Usage {
            prompt_tokens: usage_metadata["promptTokenCount"].as_u64().unwrap_or(0) as u32,
            completion_tokens: usage_metadata["candidatesTokenCount"].as_u64().unwrap_or(0) as u32,
            total_tokens: usage_metadata["totalTokenCount"].as_u64().unwrap_or(0) as u32,
            prompt_tokens_details: None,
            completion_tokens_details: None,
            thinking_usage: None,
        });

        Ok(ChatResponse {
            id: uuid::Uuid::new_v4().to_string(),
            object: "chat.completion".to_string(),
            created: chrono::Utc::now().timestamp(),
            model: model.model_id(),
            choices: vec![ChatChoice {
                index: 0,
                message: ChatMessage {
                    role: MessageRole::Assistant,
                    content: message_content,
                    thinking: None,
                    name: None,
                    tool_calls: None,
                    function_call: None,
                    tool_call_id: None,
                },
                finish_reason,
                logprobs: None,
            }],
            usage,
            system_fingerprint: None,
        })
    }
}

/// Transformer for partner models (Claude, Llama, etc.)
#[derive(Debug, Clone, Default)]
pub struct PartnerModelTransformer;

impl PartnerModelTransformer {
    pub fn new() -> Self {
        Self
    }

    /// Transform chat request for partner models
    pub fn transform_chat_request(
        &self,
        request: &ChatRequest,
        model: &VertexAIModel,
    ) -> Result<Value, ProviderError> {
        // Partner models use different formats based on the provider
        if model.model_id().contains("claude") {
            self.transform_claude_request(request)
        } else if model.model_id().contains("llama") {
            self.transform_llama_request(request)
        } else if model.model_id().contains("jamba") {
            self.transform_jamba_request(request)
        } else {
            // Default format
            self.transform_default_partner_request(request)
        }
    }

    /// Transform request for Claude models
    fn transform_claude_request(&self, request: &ChatRequest) -> Result<Value, ProviderError> {
        let mut messages = Vec::new();
        let mut system_message = None;

        for message in &request.messages {
            match message.role {
                MessageRole::System => {
                    if let Some(ref content) = message.content {
                        system_message = Some(content.to_string());
                    }
                }
                _ => {
                    messages.push(json!({
                        "role": message.role.to_string().to_lowercase(),
                        "content": message.content.as_ref().map(|c| c.to_string()).unwrap_or_default()
                    }));
                }
            }
        }

        let mut body = json!({
            "anthropic_version": "vertex-2023-10-16",
            "messages": messages,
            "max_tokens": request.max_tokens.unwrap_or(4096),
        });

        if let Some(system) = system_message {
            body["system"] = json!(system);
        }

        if let Some(temp) = request.temperature {
            body["temperature"] = json!(temp);
        }

        if let Some(top_p) = request.top_p {
            body["top_p"] = json!(top_p);
        }

        if let Some(stop) = &request.stop {
            body["stop_sequences"] = json!(stop);
        }

        Ok(json!({
            "instances": [body],
            "parameters": {}
        }))
    }

    /// Transform request for Llama models
    fn transform_llama_request(&self, request: &ChatRequest) -> Result<Value, ProviderError> {
        let prompt = self.messages_to_llama_prompt(&request.messages);

        Ok(json!({
            "instances": [{
                "prompt": prompt,
            }],
            "parameters": {
                "temperature": request.temperature.unwrap_or(0.7),
                "maxOutputTokens": request.max_tokens.unwrap_or(2048),
                "topP": request.top_p.unwrap_or(0.9),
            }
        }))
    }

    /// Transform request for Jamba models
    fn transform_jamba_request(&self, request: &ChatRequest) -> Result<Value, ProviderError> {
        let messages: Vec<Value> = request
            .messages
            .iter()
            .map(|msg| {
                json!({
                    "role": msg.role.to_string().to_lowercase(),
                    "content": msg.content.as_ref().map(|c| c.to_string()).unwrap_or_default()
                })
            })
            .collect();

        Ok(json!({
            "instances": [{
                "messages": messages,
            }],
            "parameters": {
                "temperature": request.temperature.unwrap_or(0.7),
                "max_tokens": request.max_tokens.unwrap_or(4096),
                "top_p": request.top_p.unwrap_or(0.9),
            }
        }))
    }

    /// Default partner model request format
    fn transform_default_partner_request(
        &self,
        request: &ChatRequest,
    ) -> Result<Value, ProviderError> {
        let messages: Vec<Value> = request
            .messages
            .iter()
            .map(|msg| {
                json!({
                    "role": msg.role.to_string().to_lowercase(),
                    "content": msg.content.as_ref().map(|c| c.to_string()).unwrap_or_default()
                })
            })
            .collect();

        Ok(json!({
            "instances": [{
                "messages": messages,
            }],
            "parameters": {
                "temperature": request.temperature,
                "maxOutputTokens": request.max_tokens,
                "topP": request.top_p,
            }
        }))
    }

    /// Convert messages to Llama prompt format
    fn messages_to_llama_prompt(&self, messages: &[ChatMessage]) -> String {
        let mut prompt = String::new();

        for message in messages {
            let content = message
                .content
                .as_ref()
                .map(|c| c.to_string())
                .unwrap_or_default();
            match message.role {
                MessageRole::System => {
                    prompt.push_str(&format!("<<SYS>>\n{}\n<</SYS>>\n\n", content));
                }
                MessageRole::User => {
                    prompt.push_str(&format!("[INST] {} [/INST]", content));
                }
                MessageRole::Assistant => {
                    prompt.push_str(&format!(" {}", content));
                }
                _ => {}
            }
        }

        prompt
    }

    /// Transform partner model response to standard format
    pub fn transform_chat_response(
        &self,
        response: Value,
        model: &VertexAIModel,
    ) -> Result<ChatResponse, ProviderError> {
        let predictions = response["predictions"]
            .as_array()
            .ok_or_else(|| ProviderError::response_parsing("vertex_ai", "Missing predictions"))?;

        if predictions.is_empty() {
            return Err(ProviderError::response_parsing(
                "vertex_ai",
                "No predictions in response",
            ));
        }

        let prediction = &predictions[0];

        // Extract content based on model type
        let content = if model.model_id().contains("claude") {
            prediction["content"]
                .as_str()
                .or_else(|| prediction["completion"].as_str())
                .map(|s| s.to_string())
        } else {
            prediction["content"]
                .as_str()
                .or_else(|| prediction["text"].as_str())
                .or_else(|| prediction["output"].as_str())
                .map(|s| s.to_string())
        };

        let message_content = content.map(MessageContent::Text);

        // Try to extract usage if available
        let usage = if let Some(metadata) = response.get("metadata") {
            metadata.get("tokenMetadata").map(|token_metadata| Usage {
                prompt_tokens: token_metadata["inputTokens"]["totalTokens"]
                    .as_u64()
                    .unwrap_or(0) as u32,
                completion_tokens: token_metadata["outputTokens"]["totalTokens"]
                    .as_u64()
                    .unwrap_or(0) as u32,
                total_tokens: 0, // Will be calculated
                prompt_tokens_details: None,
                completion_tokens_details: None,
                thinking_usage: None,
            })
        } else {
            None
        };

        let mut usage = usage.unwrap_or(Usage {
            prompt_tokens: 0,
            completion_tokens: 0,
            total_tokens: 0,
            prompt_tokens_details: None,
            completion_tokens_details: None,
            thinking_usage: None,
        });

        if usage.total_tokens == 0 {
            usage.total_tokens = usage.prompt_tokens + usage.completion_tokens;
        }

        Ok(ChatResponse {
            id: uuid::Uuid::new_v4().to_string(),
            object: "chat.completion".to_string(),
            created: chrono::Utc::now().timestamp(),
            model: model.model_id(),
            choices: vec![ChatChoice {
                index: 0,
                message: ChatMessage {
                    role: MessageRole::Assistant,
                    content: message_content,
                    thinking: None,
                    name: None,
                    tool_calls: None,
                    function_call: None,
                    tool_call_id: None,
                },
                finish_reason: Some(FinishReason::Stop),
                logprobs: None,
            }],
            usage: Some(usage),
            system_fingerprint: None,
        })
    }
}

#[cfg(test)]
#[path = "transformers_tests.rs"]
mod tests;
