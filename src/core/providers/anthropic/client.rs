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
    chat::ChatMessage, chat::ChatRequest, content::ContentPart, message::MessageRole,
    responses::ChatResponse,
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

    pub(crate) fn allows_unknown_model(&self, model: &str) -> bool {
        self.config.allows_unknown_model(model)
    }

    pub(crate) fn uses_compatible_model_allow_list(&self) -> bool {
        self.config.uses_compatible_model_allow_list()
    }

    pub(crate) fn allows_unknown_model_image_input(&self, model: &str) -> bool {
        self.config.allows_unknown_model_image_input(model)
    }

    /// Request
    pub async fn chat(&self, request: ChatRequest) -> Result<ChatResponse, ProviderError> {
        let tool_name_map = self.anthropic_tool_name_map_for_request(&request)?;
        let anthropic_request = self.transform_chat_request(&request)?;
        let mut headers = self.get_request_headers();
        headers.extend(self.compute_beta_headers(&request));
        let response = self
            .send_request("/v1/messages", anthropic_request, headers)
            .await?;
        self.transform_chat_response_with_tool_name_map(response, &tool_name_map)
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
        if self.config.uses_compatible_model_allow_list()
            && !self.config.allows_unknown_model(&request.model)
        {
            return Err(anthropic_api_error(
                400,
                format!("Unsupported model: {}", request.model),
            ));
        }

        let registry = get_anthropic_registry();

        // Check
        let model_spec = if self.config.uses_compatible_model_allow_list() {
            None
        } else {
            registry.get_model_spec(&request.model)
        };
        if model_spec.is_none() && !self.config.allows_unknown_model(&request.model) {
            return Err(anthropic_api_error(
                400,
                format!("Unsupported model: {}", request.model),
            ));
        }
        if model_spec.is_none()
            && (request
                .tools
                .as_ref()
                .is_some_and(|tools| !tools.is_empty())
                || Self::has_anthropic_tools_extra_param(request)
                || request.functions.as_ref().is_some_and(|f| !f.is_empty())
                || request.function_call.is_some())
        {
            return Err(ProviderError::not_supported(
                "anthropic",
                format!(
                    "Unknown model {} cannot declare tool calling support",
                    request.model
                ),
            ));
        }
        if model_spec.is_none() && Self::has_unsupported_unknown_model_content(request) {
            return Err(ProviderError::not_supported(
                "anthropic",
                format!(
                    "Unknown model {} only supports text and image content",
                    request.model
                ),
            ));
        }
        if model_spec.is_none()
            && Self::has_image_content(request)
            && !self.config.allows_unknown_model_image_input(&request.model)
        {
            return Err(ProviderError::not_supported(
                "anthropic",
                format!(
                    "Unknown model {} does not support image input",
                    request.model
                ),
            ));
        }

        // The Messages API only returns a single candidate; any n other than 1
        // (including 0) cannot be honored, so reject it instead of silently
        // returning the wrong number of choices.
        if let Some(n) = request.n
            && n != 1
        {
            return Err(ProviderError::invalid_request(
                "anthropic",
                format!("anthropic only supports n=1 (got n={})", n),
            ));
        }

        // Warn once about OpenAI-style parameters Anthropic has no equivalent for.
        let mut ignored_params = Vec::new();
        if request.frequency_penalty.is_some() {
            ignored_params.push("frequency_penalty");
        }
        if request.presence_penalty.is_some() {
            ignored_params.push("presence_penalty");
        }
        if request.seed.is_some() {
            ignored_params.push("seed");
        }
        if request.logit_bias.is_some() {
            ignored_params.push("logit_bias");
        }
        if !ignored_params.is_empty() {
            tracing::warn!(
                "Anthropic request ignores unsupported parameters: {}",
                ignored_params.join(", ")
            );
        }

        // Separate system messages from user messages
        let (system_message, messages) = self.separate_system_messages(&request.messages)?;
        let tool_name_map = self.anthropic_tool_name_map_for_request(request)?;

        let anthropic_messages = self.transform_messages(messages, model_spec, &tool_name_map)?;

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
            && !tools.is_empty()
        {
            let Some(model_spec) = model_spec else {
                return Err(ProviderError::not_supported(
                    "anthropic",
                    format!(
                        "Unknown model {} cannot declare tool calling support",
                        request.model
                    ),
                ));
            };
            if !model_spec.features.contains(&ModelFeature::ToolCalling) {
                return Err(ProviderError::not_supported(
                    "anthropic",
                    format!("Model {} does not support tool calling", request.model),
                ));
            }
            let anthropic_tools = self.transform_tools(tools)?;
            anthropic_request["tools"] = json!(anthropic_tools);

            if let Some(tool_choice) = &request.tool_choice {
                anthropic_request["tool_choice"] =
                    self.transform_tool_choice(tool_choice, &tool_name_map)?;
            }
        }

        // Add thinking configuration
        if let Some(thinking) = &request.thinking
            && thinking.enabled
        {
            let Some(model_spec) = model_spec else {
                return Err(ProviderError::not_supported(
                    "anthropic",
                    format!(
                        "Unknown model {} cannot declare thinking support",
                        request.model
                    ),
                ));
            };
            if !model_spec.features.contains(&ModelFeature::ThinkingMode) {
                return Err(ProviderError::not_supported(
                    "anthropic",
                    format!("Model {} does not support thinking", request.model),
                ));
            }
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
        model_spec: Option<&super::models::ModelSpec>,
        tool_name_map: &request_utils::ToolNameMap,
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
                                    if model_spec.is_none_or(|spec| {
                                        spec.features.contains(&ModelFeature::MultimodalSupport)
                                    }) =>
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
                                ContentPart::Image { source, .. }
                                    if model_spec.is_none_or(|spec| {
                                        spec.features.contains(&ModelFeature::MultimodalSupport)
                                    }) =>
                                {
                                    anthropic_parts.push(json!({
                                        "type": "image",
                                        "source": {
                                            "type": "base64",
                                            "media_type": source.media_type,
                                            "data": source.data
                                        }
                                    }));
                                }
                                ContentPart::Document { source, .. }
                                    if model_spec.is_some_and(|spec| {
                                        spec.features.contains(&ModelFeature::MultimodalSupport)
                                    }) =>
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
                        "name": request_utils::declared_tool_name(
                            &tool_call.function.name, tool_name_map, "Tool call"
                        )?,
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

    pub(crate) fn has_multimodal_content(request: &ChatRequest) -> bool {
        request.messages.iter().any(|msg| {
            if let Some(crate::core::types::message::MessageContent::Parts(parts)) = &msg.content {
                parts
                    .iter()
                    .any(|part| !matches!(part, ContentPart::Text { .. }))
            } else {
                false
            }
        })
    }

    pub(crate) fn has_anthropic_tools_extra_param(request: &ChatRequest) -> bool {
        request
            .extra_params
            .get("anthropic_tools")
            .and_then(|value| value.as_array())
            .is_some_and(|tools| !tools.is_empty())
    }

    pub(crate) fn has_unsupported_unknown_model_content(request: &ChatRequest) -> bool {
        request.messages.iter().any(|message| {
            matches!(message.role, MessageRole::Tool | MessageRole::Function)
                || message.thinking.is_some()
                || message
                    .tool_calls
                    .as_ref()
                    .is_some_and(|calls| !calls.is_empty())
                || message.function_call.is_some()
                || matches!(
                    &message.content,
                    Some(crate::core::types::message::MessageContent::Parts(parts))
                        if parts.iter().any(|part| !matches!(
                            part,
                            ContentPart::Text { .. }
                                | ContentPart::ImageUrl { .. }
                                | ContentPart::Image { .. }
                        ))
                )
        })
    }

    pub(crate) fn has_image_content(request: &ChatRequest) -> bool {
        request.messages.iter().any(|message| {
            matches!(
                &message.content,
                Some(crate::core::types::message::MessageContent::Parts(parts))
                    if parts.iter().any(|part| matches!(
                        part,
                        ContentPart::ImageUrl { .. } | ContentPart::Image { .. }
                    ))
            )
        })
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
        request_utils::anthropic_tools(tools)
    }

    /// Transform tool choice
    fn transform_tool_choice(
        &self,
        tool_choice: &crate::core::types::tools::ToolChoice,
        tool_name_map: &request_utils::ToolNameMap,
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
                        "name": request_utils::declared_tool_name(
                            &func.name, tool_name_map, "Tool choice"
                        )?
                    }))
                } else {
                    Ok(json!({"type": "auto"}))
                }
            }
        }
    }
}

mod request_utils;
mod response;
mod usage;

#[cfg(test)]
mod compatible_tests;
#[cfg(test)]
mod request_tests;
#[cfg(test)]
mod tests;
