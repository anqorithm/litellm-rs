//! Anthropic Client
//!
//! Error handling

use std::time::Duration;

use reqwest::{Client, ClientBuilder, Response};
use serde_json::{Value, json};
use tokio::time::timeout;

use crate::core::providers::base::{
    HeaderPair, apply_headers, header, header_owned, header_static,
};
use crate::core::providers::shared::parse_retry_after_from_body;
use crate::core::providers::unified_provider::ProviderError;
use crate::core::types::{
    chat::ChatMessage,
    chat::ChatRequest,
    content::ContentPart,
    message::MessageRole,
    responses::{ChatChoice, ChatResponse, FinishReason, PromptTokensDetails, Usage},
    thinking::ThinkingContent,
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
        let anthropic_request = self.transform_chat_request(&request)?;
        let mut headers = self.get_request_headers();
        headers.extend(self.compute_beta_headers(&request));
        let response = self
            .send_request("/v1/messages", anthropic_request, headers)
            .await?;
        self.transform_chat_response(response)
    }

    /// Request
    pub async fn chat_stream(
        &self,
        request: ChatRequest,
    ) -> Result<reqwest::Response, ProviderError> {
        let mut anthropic_request = self.transform_chat_request(&request)?;
        anthropic_request["stream"] = json!(true);
        let mut headers = self.get_request_headers();
        headers.extend(self.compute_beta_headers(&request));
        self.send_stream_request("/v1/messages", anthropic_request, headers)
            .await
    }

    /// Request
    async fn send_request(
        &self,
        endpoint: &str,
        body: Value,
        headers: Vec<HeaderPair>,
    ) -> Result<Value, ProviderError> {
        let url = format!("{}{}", self.config.base_url.trim_end_matches('/'), endpoint);

        let response = timeout(
            Duration::from_secs(self.config.request_timeout),
            apply_headers(self.http_client.post(&url).json(&body), headers).send(),
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
        headers: Vec<HeaderPair>,
    ) -> Result<Response, ProviderError> {
        let url = format!("{}{}", self.config.base_url.trim_end_matches('/'), endpoint);

        let response = timeout(
            Duration::from_secs(self.config.request_timeout),
            apply_headers(self.http_client.post(&url).json(&body), headers).send(),
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

    /// Build request headers using the unified HeaderPair pattern.
    pub fn get_request_headers(&self) -> Vec<HeaderPair> {
        let mut headers = Vec::with_capacity(5);

        // Authentication header
        if let Some(ref api_key) = self.config.api_key {
            headers.push(header("x-api-key", api_key.clone()));
        }

        // Version header
        headers.push(header("anthropic-version", self.config.api_version.clone()));

        // Content type and user agent - zero allocation for static values
        headers.push(header_static("Content-Type", "application/json"));
        headers.push(header_static("User-Agent", "LiteLLM-Rust/1.0"));

        // Custom headers
        for (key, value) in &self.config.custom_headers {
            headers.push(header_owned(key.clone(), value.clone()));
        }

        headers
    }

    /// Compute the `anthropic-beta` header values required for the given request.
    ///
    /// Returns an empty Vec when no beta features are needed.
    fn compute_beta_headers(&self, request: &ChatRequest) -> Vec<HeaderPair> {
        let mut features: Vec<String> = Vec::new();

        // Extended / interleaved thinking requires the beta header.
        if request.thinking.as_ref().is_some_and(|t| t.enabled) {
            features.push("interleaved-thinking-2025-05-14".to_string());
        }

        // Computer-use built-in tool requires its own beta header.
        if let Some(arr) = request
            .extra_params
            .get("anthropic_tools")
            .and_then(|v| v.as_array())
        {
            for tool in arr {
                if tool.get("type").and_then(|t| t.as_str()) == Some("computer_20241022") {
                    features.push("computer-use-2024-10-22".to_string());
                    break;
                }
            }
        }

        // Caller-supplied beta flags via extra_params["anthropic_beta"].
        if let Some(beta_val) = request.extra_params.get("anthropic_beta") {
            match beta_val {
                Value::Array(arr) => {
                    for item in arr {
                        if let Some(s) = item.as_str() {
                            let s = s.to_string();
                            if !features.contains(&s) {
                                features.push(s);
                            }
                        }
                    }
                }
                Value::String(s) if !features.contains(s) => {
                    features.push(s.clone());
                }
                _ => {}
            }
        }

        if features.is_empty() {
            return vec![];
        }

        vec![header("anthropic-beta", features.join(","))]
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
                let retry_after = parse_retry_after_from_body(body);
                anthropic_rate_limit_error(retry_after)
            }
            500..=599 => anthropic_api_error(status, format!("Server error: {}", body)),
            _ => anthropic_api_error(status, body),
        }
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
        if let Some(tools) = &request.tools
            && model_spec.features.contains(&ModelFeature::ToolCalling)
        {
            let anthropic_tools = self.transform_tools(tools)?;
            anthropic_request["tools"] = json!(anthropic_tools);

            // Add tool_choice
            if let Some(tool_choice) = &request.tool_choice {
                anthropic_request["tool_choice"] = self.transform_tool_choice(tool_choice)?;
            }
        }

        // Add thinking configuration
        if let Some(thinking) = &request.thinking
            && thinking.enabled
            && model_spec.features.contains(&ModelFeature::ThinkingMode)
        {
            let budget = thinking.budget_tokens.unwrap_or(10_000);
            // Anthropic requires max_tokens > budget_tokens. If the default (4096)
            // is not greater than budget_tokens, raise max_tokens to budget + 1.
            let current_max = request.max_tokens.unwrap_or(4096);
            if current_max <= budget {
                anthropic_request["max_tokens"] = json!(budget + 1);
            }
            anthropic_request["thinking"] = json!({
                "type": "enabled",
                "budget_tokens": budget
            });
        }

        // Structured outputs: pass json_schema response_format to Anthropic.
        if let Some(rf) = &request.response_format
            && rf.format_type == "json_schema"
            && let Some(schema) = &rf.json_schema
        {
            anthropic_request["response_format"] = json!({
                "type": "json_schema",
                "json_schema": schema
            });
        }

        // Anthropic built-in (server-side) tools passed via extra_params.
        // These are appended after any user-defined function tools.
        if let Some(arr) = request
            .extra_params
            .get("anthropic_tools")
            .and_then(|v| v.as_array())
            .filter(|a| !a.is_empty())
        {
            let mut merged: Vec<Value> = anthropic_request
                .get("tools")
                .and_then(|v| v.as_array())
                .cloned()
                .unwrap_or_default();
            merged.extend(arr.iter().cloned());
            anthropic_request["tools"] = json!(merged);
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
                MessageRole::System | MessageRole::Developer => {
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
            if matches!(&message.role, MessageRole::Tool | MessageRole::Function) {
                let tool_use_id = message.tool_call_id.clone().ok_or_else(|| {
                    anthropic_parse_error("Tool/function message missing tool_call_id")
                })?;
                anthropic_messages.push(json!({
                    "role": "user",
                    "content": [{
                        "type": "tool_result",
                        "tool_use_id": tool_use_id,
                        "content": Self::tool_result_content(message.content.clone())
                    }]
                }));
                continue;
            }

            let role = match message.role {
                MessageRole::User => "user",
                MessageRole::Assistant => "assistant",
                MessageRole::Tool | MessageRole::Function => unreachable!(),
                MessageRole::System | MessageRole::Developer => continue, // Already handled
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
                                ContentPart::ImageUrl { image_url }
                                    if model_spec
                                        .features
                                        .contains(&ModelFeature::MultimodalSupport) =>
                                {
                                    // Handle
                                    if image_url.url.starts_with("data:") {
                                        // Base64 format image
                                        let parts: Vec<&str> = image_url.url.split(',').collect();
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
                                        // NOTE: URL image download and conversion not yet implemented
                                        return Err(anthropic_api_error(
                                            400,
                                            "URL images not yet supported, use base64 format",
                                        ));
                                    }
                                }
                                ContentPart::Document { source, .. }
                                    if model_spec
                                        .features
                                        .contains(&ModelFeature::MultimodalSupport) =>
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
                let mut anthropic_content =
                    Self::content_value_to_blocks(&anthropic_message["content"]);
                for tool_call in tool_calls {
                    anthropic_content.push(json!({
                        "type": "tool_use",
                        "id": tool_call.id,
                        "name": tool_call.function.name,
                        "input": serde_json::from_str::<Value>(&tool_call.function.arguments)
                            .unwrap_or(json!({}))
                    }));
                }
                anthropic_message["content"] = json!(anthropic_content);
            }

            anthropic_messages.push(anthropic_message);
        }

        Ok(anthropic_messages)
    }

    fn content_value_to_blocks(content: &Value) -> Vec<Value> {
        if let Some(text) = content.as_str() {
            if text.is_empty() {
                Vec::new()
            } else {
                vec![json!({
                    "type": "text",
                    "text": text,
                })]
            }
        } else {
            content.as_array().cloned().unwrap_or_default()
        }
    }

    fn tool_result_content(content: Option<crate::core::types::message::MessageContent>) -> Value {
        match content {
            Some(crate::core::types::message::MessageContent::Text(text)) => json!(text),
            Some(crate::core::types::message::MessageContent::Parts(parts)) => {
                let text = parts
                    .into_iter()
                    .filter_map(|part| match part {
                        ContentPart::Text { text } => Some(text),
                        _ => None,
                    })
                    .collect::<Vec<_>>()
                    .join("\n");
                json!(text)
            }
            None => json!(""),
        }
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

    fn parse_anthropic_stop_reason(reason: &str) -> FinishReason {
        match reason {
            "end_turn" => FinishReason::Stop,
            "max_tokens" => FinishReason::Length,
            "tool_use" => FinishReason::ToolCalls,
            "stop_sequence" => FinishReason::StopSequence,
            "refusal" => FinishReason::Refusal,
            "pause_turn" => FinishReason::PauseTurn,
            _ => FinishReason::Stop,
        }
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
        let mut thinking_parts = Vec::new();
        let mut thinking_signature = None;
        let mut has_redacted_thinking = false;
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
                Some("thinking") => {
                    if let Some(thinking) = item.get("thinking").and_then(|t| t.as_str()) {
                        thinking_parts.push(thinking.to_string());
                    }
                    if let Some(signature) = item.get("signature").and_then(|s| s.as_str()) {
                        thinking_signature = Some(signature.to_string());
                    }
                }
                Some("redacted_thinking") => {
                    has_redacted_thinking = true;
                }
                Some("refusal") => {
                    if let Some(refusal) = item.get("refusal").and_then(|r| r.as_str()) {
                        message_content.push_str(refusal);
                    }
                }
                _ => {}
            }
        }

        let thinking = if !thinking_parts.is_empty() {
            Some(ThinkingContent::Text {
                text: thinking_parts.join(""),
                signature: thinking_signature,
            })
        } else if has_redacted_thinking {
            Some(ThinkingContent::Redacted { token_count: None })
        } else {
            None
        };

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
            thinking,
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
                .map(Self::parse_anthropic_stop_reason),
            logprobs: None,
        };

        // Build usage
        let usage = response.get("usage").map(|usage_data| {
            let input_tokens = usage_data
                .get("input_tokens")
                .and_then(|v| v.as_u64())
                .unwrap_or(0) as u32;
            let output_tokens = usage_data
                .get("output_tokens")
                .and_then(|v| v.as_u64())
                .unwrap_or(0) as u32;
            let cache_creation_tokens = usage_data
                .get("cache_creation_input_tokens")
                .and_then(|v| v.as_u64())
                .map(|tokens| tokens as u32);
            let cache_read_tokens = usage_data
                .get("cache_read_input_tokens")
                .and_then(|v| v.as_u64())
                .map(|tokens| tokens as u32);

            Usage {
                prompt_tokens: input_tokens,
                completion_tokens: output_tokens,
                total_tokens: input_tokens + output_tokens,
                completion_tokens_details: None,
                prompt_tokens_details: if cache_creation_tokens.is_some()
                    || cache_read_tokens.is_some()
                {
                    Some(PromptTokensDetails {
                        cached_tokens: cache_read_tokens,
                        cache_creation_tokens,
                        cache_read_tokens,
                        audio_tokens: None,
                    })
                } else {
                    None
                },
                thinking_usage: None,
            }
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
mod tests {
    use super::*;
    use crate::core::providers::anthropic::config::AnthropicConfig;
    use crate::core::types::message::MessageContent;

    // ==================== Client Creation Tests ====================

    #[test]
    fn test_client_creation() {
        let config = AnthropicConfig::new_test("test-key");
        let client = AnthropicClient::new(config);
        assert!(client.is_ok());
    }

    #[test]
    fn test_client_creation_with_custom_config() {
        let mut config = AnthropicConfig::new_test("test-key");
        config.request_timeout = 120;
        config.connect_timeout = 30;
        let client = AnthropicClient::new(config);
        assert!(client.is_ok());
    }

    // ==================== Header Building Tests ====================

    /// Helper to check if a header key exists in Vec<HeaderPair>
    fn has_header(headers: &[HeaderPair], key: &str) -> bool {
        headers.iter().any(|(k, _)| k.eq_ignore_ascii_case(key))
    }

    /// Helper to get a header value from Vec<HeaderPair>
    fn get_header<'a>(headers: &'a [HeaderPair], key: &str) -> Option<&'a str> {
        headers
            .iter()
            .find(|(k, _)| k.eq_ignore_ascii_case(key))
            .map(|(_, v)| v.as_ref())
    }

    #[test]
    fn test_header_building() {
        let config = AnthropicConfig::new_test("test-key");
        let client = AnthropicClient::new(config).unwrap();
        let headers = client.get_request_headers();

        // Anthropic uses x-api-key header instead of Authorization
        assert!(has_header(&headers, "x-api-key"));
        assert!(has_header(&headers, "anthropic-version"));
        assert!(has_header(&headers, "Content-Type"));
        assert!(has_header(&headers, "User-Agent"));
    }

    #[test]
    fn test_header_content_type() {
        let config = AnthropicConfig::new_test("test-key");
        let client = AnthropicClient::new(config).unwrap();
        let headers = client.get_request_headers();

        assert_eq!(
            get_header(&headers, "Content-Type").unwrap(),
            "application/json"
        );
    }

    #[test]
    fn test_header_user_agent() {
        let config = AnthropicConfig::new_test("test-key");
        let client = AnthropicClient::new(config).unwrap();
        let headers = client.get_request_headers();

        assert_eq!(
            get_header(&headers, "User-Agent").unwrap(),
            "LiteLLM-Rust/1.0"
        );
    }

    #[test]
    fn test_header_with_custom_headers() {
        let mut config = AnthropicConfig::new_test("test-key");
        config
            .custom_headers
            .insert("X-Custom-Header".to_string(), "custom-value".to_string());
        let client = AnthropicClient::new(config).unwrap();
        let headers = client.get_request_headers();

        assert!(has_header(&headers, "X-Custom-Header"));
    }

    // ==================== Error Mapping Tests ====================

    #[test]
    fn test_map_http_error_400() {
        let config = AnthropicConfig::new_test("test-key");
        let client = AnthropicClient::new(config).unwrap();
        let error = client.map_http_error(400, "invalid request");

        // Should return an API error for 400
        let error_string = format!("{}", error);
        assert!(error_string.contains("400") || error_string.contains("request"));
    }

    #[test]
    fn test_map_http_error_401() {
        let config = AnthropicConfig::new_test("test-key");
        let client = AnthropicClient::new(config).unwrap();
        let error = client.map_http_error(401, "unauthorized");

        // Should return an authentication error
        let error_string = format!("{}", error);
        assert!(error_string.to_lowercase().contains("auth") || error_string.contains("key"));
    }

    #[test]
    fn test_map_http_error_403() {
        let config = AnthropicConfig::new_test("test-key");
        let client = AnthropicClient::new(config).unwrap();
        let error = client.map_http_error(403, "forbidden");

        // Should return an authentication error
        let error_string = format!("{}", error);
        assert!(
            error_string.to_lowercase().contains("forbidden")
                || error_string.to_lowercase().contains("permission")
                || error_string.to_lowercase().contains("auth")
        );
    }

    #[test]
    fn test_map_http_error_404() {
        let config = AnthropicConfig::new_test("test-key");
        let client = AnthropicClient::new(config).unwrap();
        let error = client.map_http_error(404, "not found");

        let error_string = format!("{}", error);
        assert!(error_string.contains("404") || error_string.contains("not found"));
    }

    #[test]
    fn test_map_http_error_429() {
        let config = AnthropicConfig::new_test("test-key");
        let client = AnthropicClient::new(config).unwrap();
        let error = client.map_http_error(429, "rate limited");

        // Should return a rate limit error
        let error_string = format!("{}", error);
        assert!(
            error_string.to_lowercase().contains("rate")
                || error_string.to_lowercase().contains("limit")
        );
    }

    #[test]
    fn test_map_http_error_500() {
        let config = AnthropicConfig::new_test("test-key");
        let client = AnthropicClient::new(config).unwrap();
        let error = client.map_http_error(500, "server error");

        let error_string = format!("{}", error);
        assert!(error_string.contains("500") || error_string.to_lowercase().contains("server"));
    }

    // ==================== Retry-After Extraction Tests ====================

    #[test]
    fn test_extract_retry_after_from_root() {
        let body = r#"{"retry_after": 60}"#;
        let retry = parse_retry_after_from_body(body);
        assert_eq!(retry, Some(60));
    }

    #[test]
    fn test_extract_retry_after_from_error() {
        let body = r#"{"error": {"retry_after": 30}}"#;
        let retry = parse_retry_after_from_body(body);
        assert_eq!(retry, Some(30));
    }

    #[test]
    fn test_extract_retry_after_missing() {
        let body = r#"{"message": "no retry info"}"#;
        let retry = parse_retry_after_from_body(body);
        assert!(retry.is_none());
    }

    #[test]
    fn test_extract_retry_after_invalid_json() {
        let body = "not json";
        let retry = parse_retry_after_from_body(body);
        assert!(retry.is_none());
    }

    // ==================== System Message Separation Tests ====================

    #[test]
    fn test_separate_system_messages_no_system() {
        let config = AnthropicConfig::new_test("test-key");
        let client = AnthropicClient::new(config).unwrap();

        let messages = vec![ChatMessage {
            role: MessageRole::User,
            content: Some(MessageContent::Text("Hello".to_string())),
            name: None,
            tool_calls: None,
            tool_call_id: None,
            function_call: None,
            thinking: None,
        }];

        let (system, user_msgs) = client.separate_system_messages(&messages).unwrap();
        assert!(system.is_none());
        assert_eq!(user_msgs.len(), 1);
    }

    #[test]
    fn test_separate_system_messages_with_system() {
        let config = AnthropicConfig::new_test("test-key");
        let client = AnthropicClient::new(config).unwrap();

        let messages = vec![
            ChatMessage {
                role: MessageRole::System,
                content: Some(MessageContent::Text(
                    "You are a helpful assistant.".to_string(),
                )),
                name: None,
                tool_calls: None,
                tool_call_id: None,
                function_call: None,
                thinking: None,
            },
            ChatMessage {
                role: MessageRole::User,
                content: Some(MessageContent::Text("Hello".to_string())),
                name: None,
                tool_calls: None,
                tool_call_id: None,
                function_call: None,
                thinking: None,
            },
        ];

        let (system, user_msgs) = client.separate_system_messages(&messages).unwrap();
        assert_eq!(system, Some("You are a helpful assistant.".to_string()));
        assert_eq!(user_msgs.len(), 1);
    }

    #[test]
    fn test_separate_system_messages_multiple_system() {
        let config = AnthropicConfig::new_test("test-key");
        let client = AnthropicClient::new(config).unwrap();

        let messages = vec![
            ChatMessage {
                role: MessageRole::System,
                content: Some(MessageContent::Text("Rule 1".to_string())),
                name: None,
                tool_calls: None,
                tool_call_id: None,
                function_call: None,
                thinking: None,
            },
            ChatMessage {
                role: MessageRole::System,
                content: Some(MessageContent::Text("Rule 2".to_string())),
                name: None,
                tool_calls: None,
                tool_call_id: None,
                function_call: None,
                thinking: None,
            },
        ];

        let (system, _) = client.separate_system_messages(&messages).unwrap();
        assert_eq!(system, Some("Rule 1\nRule 2".to_string()));
    }

    #[test]
    fn test_anthropic_transform_messages_preserves_assistant_text_with_tool_use() {
        let config = AnthropicConfig::new_test("test-key");
        let client = AnthropicClient::new(config).unwrap();
        let model_spec = get_anthropic_registry()
            .get_model_spec("claude-3-opus-20240229")
            .unwrap();

        let messages = vec![ChatMessage {
            role: MessageRole::Assistant,
            content: Some(MessageContent::Text("I'll check the weather.".to_string())),
            name: None,
            tool_calls: Some(vec![crate::core::types::tools::ToolCall {
                id: "toolu_123".to_string(),
                tool_type: "function".to_string(),
                function: crate::core::types::tools::FunctionCall {
                    name: "get_weather".to_string(),
                    arguments: r#"{"location":"San Francisco"}"#.to_string(),
                },
            }]),
            tool_call_id: None,
            function_call: None,
            thinking: None,
        }];

        let transformed = client.transform_messages(messages, model_spec).unwrap();
        assert_eq!(transformed[0]["role"], "assistant");
        let content = transformed[0]["content"].as_array().unwrap();
        assert_eq!(content[0]["type"], "text");
        assert_eq!(content[0]["text"], "I'll check the weather.");
        assert_eq!(content[1]["type"], "tool_use");
        assert_eq!(content[1]["id"], "toolu_123");
        assert_eq!(content[1]["name"], "get_weather");
        assert_eq!(content[1]["input"]["location"], "San Francisco");
    }

    #[test]
    fn test_anthropic_transform_messages_tool_role_to_tool_result() {
        let config = AnthropicConfig::new_test("test-key");
        let client = AnthropicClient::new(config).unwrap();
        let model_spec = get_anthropic_registry()
            .get_model_spec("claude-3-opus-20240229")
            .unwrap();

        let messages = vec![ChatMessage {
            role: MessageRole::Tool,
            content: Some(MessageContent::Text(r#"{"temperature":"68F"}"#.to_string())),
            name: None,
            tool_calls: None,
            tool_call_id: Some("toolu_123".to_string()),
            function_call: None,
            thinking: None,
        }];

        let transformed = client.transform_messages(messages, model_spec).unwrap();
        assert_eq!(transformed[0]["role"], "user");
        let content = transformed[0]["content"].as_array().unwrap();
        assert_eq!(content[0]["type"], "tool_result");
        assert_eq!(content[0]["tool_use_id"], "toolu_123");
        assert_eq!(content[0]["content"], r#"{"temperature":"68F"}"#);
    }

    // ==================== Tool Choice Transformation Tests ====================

    #[test]
    fn test_transform_tool_choice_auto() {
        let config = AnthropicConfig::new_test("test-key");
        let client = AnthropicClient::new(config).unwrap();

        let tool_choice = crate::core::types::tools::ToolChoice::String("auto".to_string());
        let result = client.transform_tool_choice(&tool_choice).unwrap();

        assert_eq!(result["type"], "auto");
    }

    #[test]
    fn test_transform_tool_choice_none() {
        let config = AnthropicConfig::new_test("test-key");
        let client = AnthropicClient::new(config).unwrap();

        let tool_choice = crate::core::types::tools::ToolChoice::String("none".to_string());
        let result = client.transform_tool_choice(&tool_choice).unwrap();

        assert_eq!(result["type"], "none");
    }

    #[test]
    fn test_transform_tool_choice_required() {
        let config = AnthropicConfig::new_test("test-key");
        let client = AnthropicClient::new(config).unwrap();

        let tool_choice = crate::core::types::tools::ToolChoice::String("required".to_string());
        let result = client.transform_tool_choice(&tool_choice).unwrap();

        assert_eq!(result["type"], "any");
    }

    // ==================== Tool Transformation Tests ====================

    #[test]
    fn test_transform_tools() {
        let config = AnthropicConfig::new_test("test-key");
        let client = AnthropicClient::new(config).unwrap();

        let tools = vec![crate::core::types::tools::Tool {
            tool_type: crate::core::types::tools::ToolType::Function,
            function: crate::core::types::tools::FunctionDefinition {
                name: "get_weather".to_string(),
                description: Some("Get weather for a location".to_string()),
                parameters: Some(json!({"type": "object"})),
            },
        }];

        let result = client.transform_tools(&tools).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0]["name"], "get_weather");
        assert_eq!(result[0]["description"], "Get weather for a location");
    }

    #[test]
    fn test_transform_tools_empty() {
        let config = AnthropicConfig::new_test("test-key");
        let client = AnthropicClient::new(config).unwrap();

        let tools: Vec<crate::core::types::tools::Tool> = vec![];
        let result = client.transform_tools(&tools).unwrap();
        assert!(result.is_empty());
    }

    // ==================== Chat Response Transformation Tests ====================

    #[test]
    fn test_transform_chat_response_text() {
        let config = AnthropicConfig::new_test("test-key");
        let client = AnthropicClient::new(config).unwrap();

        let response = json!({
            "id": "msg_123",
            "model": "claude-3-opus-20240229",
            "content": [
                {"type": "text", "text": "Hello, world!"}
            ],
            "stop_reason": "end_turn",
            "usage": {
                "input_tokens": 10,
                "output_tokens": 20
            }
        });

        let result = client.transform_chat_response(response).unwrap();
        assert_eq!(result.id, "msg_123");
        assert_eq!(result.model, "claude-3-opus-20240229");
        assert_eq!(result.choices.len(), 1);

        if let Some(MessageContent::Text(text)) = &result.choices.first().unwrap().message.content {
            assert_eq!(text, "Hello, world!");
        } else {
            panic!("Expected text content");
        }
    }

    #[test]
    fn test_transform_chat_response_usage() {
        let config = AnthropicConfig::new_test("test-key");
        let client = AnthropicClient::new(config).unwrap();

        let response = json!({
            "id": "msg_123",
            "model": "claude-3-opus-20240229",
            "content": [{"type": "text", "text": "Hi"}],
            "usage": {
                "input_tokens": 100,
                "output_tokens": 50
            }
        });

        let result = client.transform_chat_response(response).unwrap();
        let usage = result.usage.unwrap();
        assert_eq!(usage.prompt_tokens, 100);
        assert_eq!(usage.completion_tokens, 50);
        assert_eq!(usage.total_tokens, 150);
    }

    #[test]
    fn test_anthropic_usage_cache_details() {
        let config = AnthropicConfig::new_test("test-key");
        let client = AnthropicClient::new(config).unwrap();

        let response = json!({
            "id": "msg_123",
            "model": "claude-3-opus-20240229",
            "content": [{"type": "text", "text": "Hi"}],
            "usage": {
                "input_tokens": 100,
                "cache_creation_input_tokens": 12,
                "cache_read_input_tokens": 34,
                "output_tokens": 50
            }
        });

        let result = client.transform_chat_response(response).unwrap();
        let details = result
            .usage
            .as_ref()
            .and_then(|usage| usage.prompt_tokens_details.as_ref())
            .unwrap();
        assert_eq!(details.cached_tokens, Some(34));
        assert_eq!(details.cache_creation_tokens, Some(12));
        assert_eq!(details.cache_read_tokens, Some(34));
    }

    #[test]
    fn test_anthropic_client_preserves_thinking_blocks() {
        let config = AnthropicConfig::new_test("test-key");
        let client = AnthropicClient::new(config).unwrap();

        let response = json!({
            "id": "msg_123",
            "model": "claude-3-opus-20240229",
            "content": [
                {
                    "type": "thinking",
                    "thinking": "First thought. ",
                    "signature": "sig_123"
                },
                {
                    "type": "thinking",
                    "thinking": "Second thought."
                },
                {"type": "text", "text": "Answer."}
            ],
            "stop_reason": "end_turn"
        });

        let result = client.transform_chat_response(response).unwrap();
        assert_eq!(
            result
                .choices
                .first()
                .unwrap()
                .message
                .thinking
                .as_ref()
                .and_then(|thinking| thinking.as_text()),
            Some("First thought. Second thought.")
        );
        assert_eq!(
            result.choices.first().unwrap().message.thinking.as_ref(),
            Some(&ThinkingContent::Text {
                text: "First thought. Second thought.".to_string(),
                signature: Some("sig_123".to_string()),
            })
        );
    }

    #[test]
    fn test_transform_chat_response_tool_use() {
        let config = AnthropicConfig::new_test("test-key");
        let client = AnthropicClient::new(config).unwrap();

        let response = json!({
            "id": "msg_123",
            "model": "claude-3-opus-20240229",
            "content": [
                {
                    "type": "tool_use",
                    "id": "tool_1",
                    "name": "get_weather",
                    "input": {"location": "San Francisco"}
                }
            ],
            "stop_reason": "tool_use"
        });

        let result = client.transform_chat_response(response).unwrap();
        let tool_calls = result
            .choices
            .first()
            .unwrap()
            .message
            .tool_calls
            .as_ref()
            .unwrap();
        assert_eq!(tool_calls.len(), 1);
        assert_eq!(tool_calls.first().unwrap().id, "tool_1");
        assert_eq!(tool_calls.first().unwrap().function.name, "get_weather");
    }

    #[test]
    fn test_transform_chat_response_finish_reasons() {
        let config = AnthropicConfig::new_test("test-key");
        let client = AnthropicClient::new(config).unwrap();

        // end_turn -> Stop
        let response = json!({
            "id": "msg_123",
            "model": "claude-3-opus-20240229",
            "content": [{"type": "text", "text": "Hi"}],
            "stop_reason": "end_turn"
        });
        let result = client.transform_chat_response(response).unwrap();
        assert!(matches!(
            result.choices.first().unwrap().finish_reason,
            Some(crate::core::types::responses::FinishReason::Stop)
        ));

        // max_tokens -> Length
        let response = json!({
            "id": "msg_123",
            "model": "claude-3-opus-20240229",
            "content": [{"type": "text", "text": "Hi"}],
            "stop_reason": "max_tokens"
        });
        let result = client.transform_chat_response(response).unwrap();
        assert!(matches!(
            result.choices.first().unwrap().finish_reason,
            Some(crate::core::types::responses::FinishReason::Length)
        ));

        // tool_use -> ToolCalls
        let response = json!({
            "id": "msg_123",
            "model": "claude-3-opus-20240229",
            "content": [{"type": "text", "text": "Hi"}],
            "stop_reason": "tool_use"
        });
        let result = client.transform_chat_response(response).unwrap();
        assert!(matches!(
            result.choices.first().unwrap().finish_reason,
            Some(crate::core::types::responses::FinishReason::ToolCalls)
        ));

        // stop_sequence -> StopSequence
        let response = json!({
            "id": "msg_123",
            "model": "claude-3-opus-20240229",
            "content": [{"type": "text", "text": "Hi"}],
            "stop_reason": "stop_sequence"
        });
        let result = client.transform_chat_response(response).unwrap();
        assert!(matches!(
            result.choices.first().unwrap().finish_reason,
            Some(crate::core::types::responses::FinishReason::StopSequence)
        ));

        // refusal -> Refusal
        let response = json!({
            "id": "msg_123",
            "model": "claude-3-opus-20240229",
            "content": [{"type": "refusal", "refusal": "I cannot help with that."}],
            "stop_reason": "refusal"
        });
        let result = client.transform_chat_response(response).unwrap();
        assert!(matches!(
            result.choices.first().unwrap().finish_reason,
            Some(crate::core::types::responses::FinishReason::Refusal)
        ));

        // pause_turn -> PauseTurn
        let response = json!({
            "id": "msg_123",
            "model": "claude-3-opus-20240229",
            "content": [{"type": "text", "text": "Hi"}],
            "stop_reason": "pause_turn"
        });
        let result = client.transform_chat_response(response).unwrap();
        assert!(matches!(
            result.choices.first().unwrap().finish_reason,
            Some(crate::core::types::responses::FinishReason::PauseTurn)
        ));
    }
}
