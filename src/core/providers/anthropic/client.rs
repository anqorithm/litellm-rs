//! Anthropic Client
//!
//! Error handling

use std::time::Duration;

use reqwest::{Client, ClientBuilder, Response};
use serde_json::{Value, json};
use tokio::time::timeout;

use crate::core::providers::unified_provider::ProviderError;
use crate::core::types::{
    chat::ChatMessage, chat::ChatRequest,
    content::ContentPart,
    message::MessageRole,
    responses::{ChatChoice, ChatResponse, Usage},
};

use super::config::AnthropicConfig;
use super::error::{
    anthropic_api_error, anthropic_auth_error, anthropic_network_error, anthropic_parse_error,
    anthropic_rate_limit_error,
};
use super::models::{ModelFeature, get_anthropic_registry};

/// Anthropic API client
#[derive(Debug, Clone)]
pub struct AnthropicClient {
    config: AnthropicConfig,
    http_client: Client,
}

impl AnthropicClient {
    /// Create
    pub fn new(config: AnthropicConfig) -> Result<Self, ProviderError> {
        let mut builder = ClientBuilder::new()
            .timeout(Duration::from_secs(config.request_timeout))
            .connect_timeout(Duration::from_secs(config.connect_timeout));

        // Configuration
        if let Some(proxy_url) = &config.proxy_url {
            let proxy = reqwest::Proxy::all(proxy_url)
                .map_err(|e| anthropic_network_error(format!("Invalid proxy URL: {}", e)))?;
            builder = builder.proxy(proxy);
        }

        let http_client = builder
            .build()
            .map_err(|e| anthropic_network_error(format!("Failed to create HTTP client: {}", e)))?;

        Ok(Self {
            config,
            http_client,
        })
    }

    /// Request
    pub async fn chat(&self, request: ChatRequest) -> Result<ChatResponse, ProviderError> {
        // Request
        let anthropic_request = self.transform_chat_request(&request)?;

        // Request
        let response = self.send_request("/v1/messages", anthropic_request).await?;

        // Response
        self.transform_chat_response(response)
    }

    /// Request
    pub async fn chat_stream(
        &self,
        request: ChatRequest,
    ) -> Result<reqwest::Response, ProviderError> {
        // Request
        let mut anthropic_request = self.transform_chat_request(&request)?;
        anthropic_request["stream"] = json!(true);

        // Request
        self.send_stream_request("/v1/messages", anthropic_request)
            .await
    }

    /// Request
    async fn send_request(&self, endpoint: &str, body: Value) -> Result<Value, ProviderError> {
        let url = format!("{}{}", self.config.base_url.trim_end_matches('/'), endpoint);
        let headers = self.build_headers();

        let response = timeout(
            Duration::from_secs(self.config.request_timeout),
            self.http_client
                .post(&url)
                .json(&body)
                .headers(headers)
                .send(),
        )
        .await
        .map_err(|_| anthropic_network_error("Request timeout"))?
        .map_err(|e| anthropic_network_error(format!("Network error: {}", e)))?;

        self.handle_response(response).await
    }

    /// Request
    async fn send_stream_request(
        &self,
        endpoint: &str,
        body: Value,
    ) -> Result<Response, ProviderError> {
        let url = format!("{}{}", self.config.base_url.trim_end_matches('/'), endpoint);
        let headers = self.build_headers();

        let response = timeout(
            Duration::from_secs(self.config.request_timeout),
            self.http_client
                .post(&url)
                .json(&body)
                .headers(headers)
                .send(),
        )
        .await
        .map_err(|_| anthropic_network_error("Request timeout"))?
        .map_err(|e| anthropic_network_error(format!("Network error: {}", e)))?;

        // Check
        if !response.status().is_success() {
            let status = response.status().as_u16();
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Failed to read error response".to_string());
            return Err(self.map_http_error(status, &error_text));
        }

        Ok(response)
    }

    /// Request
    fn build_headers(&self) -> reqwest::header::HeaderMap {
        let mut headers = reqwest::header::HeaderMap::new();

        // Authentication header
        if let Some(ref api_key) = self.config.api_key {
            if let Ok(api_key_header) = api_key.parse() {
                headers.insert("x-api-key", api_key_header);
            }
        }

        // Version header
        if let Ok(version_header) = self.config.api_version.parse() {
            headers.insert("anthropic-version", version_header);
        }

        // Content type - static string guaranteed to be valid
        headers.insert(
            "Content-Type",
            "application/json"
                .parse()
                .expect("static header value is valid"),
        );

        // User agent - static string guaranteed to be valid
        headers.insert(
            "User-Agent",
            "LiteLLM-Rust/1.0"
                .parse()
                .expect("static header value is valid"),
        );

        // Custom headers
        for (key, value) in &self.config.custom_headers {
            if let (Ok(header_name), Ok(header_value)) = (
                key.parse::<reqwest::header::HeaderName>(),
                value.parse::<reqwest::header::HeaderValue>(),
            ) {
                headers.insert(header_name, header_value);
            }
        }

        headers
    }

    /// Handle
    async fn handle_response(&self, response: Response) -> Result<Value, ProviderError> {
        let status = response.status().as_u16();
        let response_text = response
            .text()
            .await
            .map_err(|e| anthropic_network_error(format!("Failed to read response: {}", e)))?;

        if status != 200 {
            return Err(self.map_http_error(status, &response_text));
        }

        serde_json::from_str(&response_text)
            .map_err(|e| anthropic_parse_error(format!("Failed to parse JSON: {}", e)))
    }

    /// Error
    fn map_http_error(&self, status: u16, body: &str) -> ProviderError {
        match status {
            400 => anthropic_api_error(400, format!("Bad request: {}", body)),
            401 => anthropic_auth_error("Invalid or missing API key"),
            403 => anthropic_auth_error("Forbidden: insufficient permissions"),
            404 => anthropic_api_error(404, "Model or endpoint not found"),
            429 => {
                let retry_after = self.extract_retry_after(body);
                anthropic_rate_limit_error(retry_after)
            }
            500..=599 => anthropic_api_error(status, format!("Server error: {}", body)),
            _ => anthropic_api_error(status, body),
        }
    }

    /// Extract retry-after value
    fn extract_retry_after(&self, body: &str) -> Option<u64> {
        if let Ok(json) = serde_json::from_str::<Value>(body) {
            if let Some(retry_after) = json.get("retry_after") {
                return retry_after.as_u64();
            }

            if let Some(error) = json.get("error") {
                if let Some(retry_after) = error.get("retry_after") {
                    return retry_after.as_u64();
                }
            }
        }
        None
    }

    /// Request
    fn transform_chat_request(&self, request: &ChatRequest) -> Result<Value, ProviderError> {
        let registry = get_anthropic_registry();

        // Check
        let model_spec = registry.get_model_spec(&request.model).ok_or_else(|| {
            anthropic_api_error(400, format!("Unsupported model: {}", request.model))
        })?;

        // Separate system messages from user messages
        let (system_message, messages) = self.separate_system_messages(&request.messages)?;

        // Transform message format
        let anthropic_messages = self.transform_messages(messages, model_spec)?;

        // Request
        let mut anthropic_request = json!({
            "model": request.model,
            "max_tokens": request.max_tokens.unwrap_or(4096),
            "messages": anthropic_messages,
        });

        // Add system message
        if let Some(system) = system_message {
            anthropic_request["system"] = json!(system);
        }

        // Add optional parameters
        if let Some(temperature) = request.temperature {
            anthropic_request["temperature"] = json!(temperature);
        }

        if let Some(top_p) = request.top_p {
            anthropic_request["top_p"] = json!(top_p);
        }

        if let Some(stop) = &request.stop {
            anthropic_request["stop_sequences"] = json!(stop);
        }

        // Add tool support
        if let Some(tools) = &request.tools {
            if model_spec.features.contains(&ModelFeature::ToolCalling) {
                let anthropic_tools = self.transform_tools(tools)?;
                anthropic_request["tools"] = json!(anthropic_tools);

                // Add tool_choice
                if let Some(tool_choice) = &request.tool_choice {
                    anthropic_request["tool_choice"] = self.transform_tool_choice(tool_choice)?;
                }
            }
        }

        Ok(anthropic_request)
    }

    /// Separate system messages from user messages
    fn separate_system_messages(
        &self,
        messages: &[ChatMessage],
    ) -> Result<(Option<String>, Vec<ChatMessage>), ProviderError> {
        let mut system_parts = Vec::new();
        let mut user_messages = Vec::new();

        for message in messages {
            match message.role {
                MessageRole::System => {
                    if let Some(content) = &message.content {
                        match content {
                            crate::core::types::message::MessageContent::Text(text) => {
                                system_parts.push(text.clone());
                            }
                            crate::core::types::message::MessageContent::Parts(parts) => {
                                for part in parts {
                                    if let ContentPart::Text { text } = part {
                                        system_parts.push(text.clone());
                                    }
                                }
                            }
                        }
                    }
                }
                _ => {
                    user_messages.push(message.clone());
                }
            }
        }

        let system_message = if system_parts.is_empty() {
            None
        } else {
            Some(system_parts.join("\n"))
        };

        Ok((system_message, user_messages))
    }

    /// Transform messages to Anthropic format
    fn transform_messages(
        &self,
        messages: Vec<ChatMessage>,
        model_spec: &super::models::ModelSpec,
    ) -> Result<Vec<Value>, ProviderError> {
        let mut anthropic_messages = Vec::new();

        for message in messages {
            let role = match message.role {
                MessageRole::User => "user",
                MessageRole::Assistant => "assistant",
                MessageRole::Tool => "user",     // Response
                MessageRole::Function => "user", // Response
                MessageRole::System => continue, // Already handled
            };

            let content = if let Some(content) = message.content {
                match content {
                    crate::core::types::message::MessageContent::Text(text) => {
                        json!(text)
                    }
                    crate::core::types::message::MessageContent::Parts(parts) => {
                        let mut anthropic_parts = Vec::new();

                        for part in parts {
                            match part {
                                ContentPart::Text { text } => {
                                    anthropic_parts.push(json!({
                                        "type": "text",
                                        "text": text
                                    }));
                                }
                                ContentPart::ImageUrl { image_url } => {
                                    if model_spec
                                        .features
                                        .contains(&ModelFeature::MultimodalSupport)
                                    {
                                        // Handle
                                        if image_url.url.starts_with("data:") {
                                            // Base64 format image
                                            let parts: Vec<&str> =
                                                image_url.url.split(',').collect();
                                            if parts.len() == 2 {
                                                let media_type = parts[0]
                                                    .strip_prefix("data:")
                                                    .and_then(|s| s.split(';').next())
                                                    .unwrap_or("image/jpeg");

                                                anthropic_parts.push(json!({
                                                    "type": "image",
                                                    "source": {
                                                        "type": "base64",
                                                        "media_type": media_type,
                                                        "data": parts[1]
                                                    }
                                                }));
                                            }
                                        } else {
                                            // URL format image - requires download and conversion
                                            // TODO: implement URL image download and conversion
                                            return Err(anthropic_api_error(
                                                400,
                                                "URL images not yet supported, use base64 format",
                                            ));
                                        }
                                    }
                                }
                                ContentPart::Document { source, .. } => {
                                    if model_spec
                                        .features
                                        .contains(&ModelFeature::MultimodalSupport)
                                    {
                                        anthropic_parts.push(json!({
                                            "type": "document",
                                            "source": {
                                                "type": "base64",
                                                "media_type": source.media_type,
                                                "data": source.data
                                            }
                                        }));
                                    }
                                }
                                _ => {
                                    // Other content types not yet supported
                                }
                            }
                        }

                        json!(anthropic_parts)
                    }
                }
            } else {
                json!("")
            };

            let mut anthropic_message = json!({
                "role": role,
                "content": content
            });

            // Add tool_call
            if let Some(tool_calls) = &message.tool_calls {
                let mut anthropic_tool_calls = Vec::new();
                for tool_call in tool_calls {
                    anthropic_tool_calls.push(json!({
                        "type": "tool_use",
                        "id": tool_call.id,
                        "name": tool_call.function.name,
                        "input": serde_json::from_str::<Value>(&tool_call.function.arguments)
                            .unwrap_or(json!({}))
                    }));
                }
                anthropic_message["content"] = json!(anthropic_tool_calls);
            }

            anthropic_messages.push(anthropic_message);
        }

        Ok(anthropic_messages)
    }

    /// Transform tool definitions
    fn transform_tools(
        &self,
        tools: &[crate::core::types::tools::Tool],
    ) -> Result<Vec<Value>, ProviderError> {
        let mut anthropic_tools = Vec::new();

        for tool in tools {
            anthropic_tools.push(json!({
                "name": tool.function.name,
                "description": tool.function.description.as_ref().unwrap_or(&String::new()),
                "input_schema": tool.function.parameters.as_ref().unwrap_or(&json!({}))
            }));
        }

        Ok(anthropic_tools)
    }

    /// Transform tool choice
    fn transform_tool_choice(
        &self,
        tool_choice: &crate::core::types::tools::ToolChoice,
    ) -> Result<Value, ProviderError> {
        match tool_choice {
            crate::core::types::tools::ToolChoice::String(choice) => match choice.as_str() {
                "auto" => Ok(json!({"type": "auto"})),
                "none" => Ok(json!({"type": "none"})),
                "required" => Ok(json!({"type": "any"})),
                _ => Ok(json!({"type": "auto"})),
            },
            crate::core::types::tools::ToolChoice::Specific { function, .. } => {
                if let Some(func) = function {
                    Ok(json!({
                        "type": "tool",
                        "name": func.name
                    }))
                } else {
                    Ok(json!({"type": "auto"}))
                }
            }
        }
    }

    /// Response
    fn transform_chat_response(&self, response: Value) -> Result<ChatResponse, ProviderError> {
        // Extract basic information
        let id = response
            .get("id")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let model = response
            .get("model")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let created = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64;

        // Handle content
        let content = response
            .get("content")
            .and_then(|v| v.as_array())
            .ok_or_else(|| anthropic_parse_error("Missing or invalid content array"))?;

        let mut message_content = String::new();
        let mut tool_calls = Vec::new();

        for item in content {
            match item.get("type").and_then(|t| t.as_str()) {
                Some("text") => {
                    if let Some(text) = item.get("text").and_then(|t| t.as_str()) {
                        message_content.push_str(text);
                    }
                }
                Some("tool_use") => {
                    if let (Some(id), Some(name), Some(input)) = (
                        item.get("id").and_then(|v| v.as_str()),
                        item.get("name").and_then(|v| v.as_str()),
                        item.get("input"),
                    ) {
                        tool_calls.push(crate::core::types::tools::ToolCall {
                            id: id.to_string(),
                            tool_type: "function".to_string(),
                            function: crate::core::types::tools::FunctionCall {
                                name: name.to_string(),
                                arguments: input.to_string(),
                            },
                        });
                    }
                }
                _ => {}
            }
        }

        // Build message
        let message = ChatMessage {
            role: MessageRole::Assistant,
            content: if message_content.is_empty() {
                None
            } else {
                Some(crate::core::types::message::MessageContent::Text(
                    message_content,
                ))
            },
            thinking: None,
            name: None,
            tool_calls: if tool_calls.is_empty() {
                None
            } else {
                Some(tool_calls)
            },
            tool_call_id: None,
            function_call: None,
        };

        // Build choice
        let choice = ChatChoice {
            index: 0,
            message,
            finish_reason: response
                .get("stop_reason")
                .and_then(|r| r.as_str())
                .map(|reason| match reason {
                    "end_turn" => crate::core::types::responses::FinishReason::Stop,
                    "max_tokens" => crate::core::types::responses::FinishReason::Length,
                    "tool_use" => crate::core::types::responses::FinishReason::ToolCalls,
                    _ => crate::core::types::responses::FinishReason::Stop,
                }),
            logprobs: None,
        };

        // Build usage
        let usage = response.get("usage").map(|usage_data| Usage {
            prompt_tokens: usage_data
                .get("input_tokens")
                .and_then(|v| v.as_u64())
                .unwrap_or(0) as u32,
            completion_tokens: usage_data
                .get("output_tokens")
                .and_then(|v| v.as_u64())
                .unwrap_or(0) as u32,
            total_tokens: (usage_data
                .get("input_tokens")
                .and_then(|v| v.as_u64())
                .unwrap_or(0)
                + usage_data
                    .get("output_tokens")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0)) as u32,
            completion_tokens_details: None,
            prompt_tokens_details: None,
            thinking_usage: None,
        });

        Ok(ChatResponse {
            id,
            object: "chat.completion".to_string(),
            created,
            model,
            choices: vec![choice],
            usage,
            system_fingerprint: None,
        })
    }
}


#[cfg(test)]
#[path = "client_tests.rs"]
mod tests;
