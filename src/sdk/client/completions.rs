//! Chat completion methods

use super::llm_client::LLMClient;
use crate::sdk::{errors::*, types::*};
use futures::StreamExt as _;
use std::pin::Pin;
use std::time::SystemTime;
use tracing::{debug, error};

impl LLMClient {
    /// Send chat message (using load balancing)
    pub async fn chat(&self, messages: Vec<Message>) -> Result<ChatResponse> {
        let request = SdkChatRequest {
            model: String::new(), // Will be set by load balancer
            messages,
            options: ChatOptions::default(),
        };

        self.chat_with_options(request).await
    }

    /// Send chat message (with options)
    pub async fn chat_with_options(&self, request: SdkChatRequest) -> Result<ChatResponse> {
        let start_time = SystemTime::now();

        // Select best provider
        let provider = self.select_provider(&request).await?;

        // Execute request
        let result = self.execute_chat_request(&provider.id, request).await;

        // Update statistics
        self.update_provider_stats(&provider.id, start_time, &result)
            .await;

        result
    }

    /// Streaming chat
    pub async fn chat_stream(
        &self,
        messages: Vec<Message>,
    ) -> Result<Pin<Box<dyn futures::Stream<Item = Result<ChatChunk>> + Send>>> {
        let provider = self.select_provider_for_stream(&messages).await?;
        self.execute_stream_request(&provider.id, messages).await
    }

    /// Execute chat request with a specific provider
    pub(crate) async fn execute_chat_request(
        &self,
        provider_id: &str,
        request: SdkChatRequest,
    ) -> Result<ChatResponse> {
        let provider = self
            .config
            .providers
            .iter()
            .find(|p| p.id == provider_id)
            .ok_or_else(|| SDKError::ProviderNotFound(provider_id.to_string()))?;

        debug!("Executing chat request with provider: {}", provider_id);

        match provider.provider_type {
            crate::sdk::config::ProviderType::Anthropic => {
                self.call_anthropic_api(provider, request).await
            }
            crate::sdk::config::ProviderType::OpenAI => {
                self.call_openai_api(provider, request).await
            }
            crate::sdk::config::ProviderType::Google => {
                self.call_google_api(provider, request).await
            }
            _ => Err(SDKError::ProviderError(format!(
                "Provider type {:?} is not implemented in SDK client",
                provider.provider_type
            ))),
        }
    }

    /// Execute stream request
    pub(crate) async fn execute_stream_request(
        &self,
        provider_id: &str,
        messages: Vec<Message>,
    ) -> Result<Pin<Box<dyn futures::Stream<Item = Result<ChatChunk>> + Send>>> {
        let provider = self
            .config
            .providers
            .iter()
            .find(|p| p.id == provider_id)
            .ok_or_else(|| SDKError::ProviderNotFound(provider_id.to_string()))?;

        match provider.provider_type {
            crate::sdk::config::ProviderType::OpenAI => {
                self.stream_openai_request(provider, messages).await
            }
            crate::sdk::config::ProviderType::Anthropic => {
                self.stream_anthropic_request(provider, messages).await
            }
            _ => Err(SDKError::NotSupported(format!(
                "Streaming is not implemented for provider type {:?}",
                provider.provider_type
            ))),
        }
    }

    /// Stream request to OpenAI-compatible endpoint
    async fn stream_openai_request(
        &self,
        provider: &crate::sdk::config::SdkProviderConfig,
        messages: Vec<Message>,
    ) -> Result<Pin<Box<dyn futures::Stream<Item = Result<ChatChunk>> + Send>>> {
        let model = provider
            .models
            .first()
            .cloned()
            .unwrap_or_else(|| "gpt-4".to_string());
        let body = serde_json::json!({
            "model": model,
            "messages": messages,
            "stream": true
        });

        let default_url = "https://api.openai.com/v1".to_string();
        let base_url = provider.base_url.as_ref().unwrap_or(&default_url);
        let url = if base_url.contains("/v1") {
            format!("{}/chat/completions", base_url.trim_end_matches('/'))
        } else {
            format!("{}/v1/chat/completions", base_url.trim_end_matches('/'))
        };

        debug!("Calling OpenAI streaming API: {}", url);

        let response = self
            .http_client
            .post(&url)
            .header("Authorization", format!("Bearer {}", provider.api_key))
            .header("content-type", "application/json")
            .json(&body)
            .send()
            .await
            .map_err(|e| SDKError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(SDKError::ApiError(format!(
                "HTTP {}: {}",
                status, error_text
            )));
        }

        Ok(Box::pin(parse_sse_stream_openai(response)))
    }

    /// Stream request to Anthropic endpoint
    async fn stream_anthropic_request(
        &self,
        provider: &crate::sdk::config::SdkProviderConfig,
        messages: Vec<Message>,
    ) -> Result<Pin<Box<dyn futures::Stream<Item = Result<ChatChunk>> + Send>>> {
        let (system_message, anthropic_messages) = self.convert_messages_to_anthropic(&messages);

        let model = provider
            .models
            .first()
            .cloned()
            .unwrap_or_else(|| "claude-sonnet-4-5".to_string());
        let mut body = serde_json::json!({
            "model": model,
            "messages": anthropic_messages,
            "max_tokens": 1024,
            "stream": true
        });

        if let Some(system) = system_message {
            body["system"] = serde_json::json!(system);
        }

        let default_url = "https://api.anthropic.com".to_string();
        let base_url = provider.base_url.as_ref().unwrap_or(&default_url);
        let url = if base_url.contains("/v1") {
            format!("{}/messages", base_url.trim_end_matches('/'))
        } else {
            format!("{}/v1/messages", base_url.trim_end_matches('/'))
        };

        debug!("Calling Anthropic streaming API: {}", url);

        let response = self
            .http_client
            .post(&url)
            .header("x-api-key", &provider.api_key)
            .header("anthropic-version", "2023-06-01")
            .header("content-type", "application/json")
            .json(&body)
            .send()
            .await
            .map_err(|e| SDKError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(SDKError::ApiError(format!(
                "HTTP {}: {}",
                status, error_text
            )));
        }

        Ok(Box::pin(parse_sse_stream_anthropic(response)))
    }

    /// Call Anthropic API
    async fn call_anthropic_api(
        &self,
        provider: &crate::sdk::config::SdkProviderConfig,
        request: SdkChatRequest,
    ) -> Result<ChatResponse> {
        // Convert message format
        let (system_message, anthropic_messages) =
            self.convert_messages_to_anthropic(&request.messages);

        // Build request body
        let mut body = serde_json::json!({
            "model": provider.models.first().unwrap_or(&"claude-sonnet-4-5".to_string()),
            "messages": anthropic_messages,
            "max_tokens": request.options.max_tokens.unwrap_or(1000)
        });

        if let Some(system) = system_message {
            body["system"] = serde_json::json!(system);
        }

        if let Some(temp) = request.options.temperature {
            body["temperature"] = serde_json::json!(temp);
        }

        if let Some(top_p) = request.options.top_p {
            body["top_p"] = serde_json::json!(top_p);
        }

        // Send request
        let default_url = "https://api.anthropic.com".to_string();
        let base_url = provider.base_url.as_ref().unwrap_or(&default_url);
        let url = if base_url.contains("/v1") {
            format!("{}/messages", base_url.trim_end_matches('/'))
        } else {
            format!("{}/v1/messages", base_url.trim_end_matches('/'))
        };

        debug!("Calling Anthropic API: {}", url);

        let response = self
            .http_client
            .post(&url)
            .header("x-api-key", &provider.api_key)
            .header("anthropic-version", "2023-06-01")
            .header("content-type", "application/json")
            .json(&body)
            .send()
            .await
            .map_err(|e| SDKError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            error!("Anthropic API error: {} - {}", status, error_text);
            return Err(SDKError::ApiError(format!(
                "HTTP {}: {}",
                status, error_text
            )));
        }

        let anthropic_response: serde_json::Value = response
            .json()
            .await
            .map_err(|e| SDKError::ParseError(e.to_string()))?;

        // Convert response
        self.convert_anthropic_response(
            anthropic_response,
            provider
                .models
                .first()
                .unwrap_or(&"claude-sonnet-4-5".to_string()),
        )
    }

    /// Call OpenAI API
    async fn call_openai_api(
        &self,
        provider: &crate::sdk::config::SdkProviderConfig,
        request: SdkChatRequest,
    ) -> Result<ChatResponse> {
        let body = serde_json::json!({
            "model": provider.models.first().unwrap_or(&"gpt-5.2-chat".to_string()),
            "messages": request.messages,
            "max_tokens": request.options.max_tokens.unwrap_or(1000),
            "temperature": request.options.temperature.unwrap_or(0.7),
            "stream": false
        });

        let default_url = "https://api.openai.com/v1".to_string();
        let base_url = provider.base_url.as_ref().unwrap_or(&default_url);
        let url = if base_url.contains("/v1") {
            format!("{}/chat/completions", base_url.trim_end_matches('/'))
        } else {
            format!("{}/v1/chat/completions", base_url.trim_end_matches('/'))
        };

        debug!("Calling OpenAI API: {}", url);

        let response = self
            .http_client
            .post(&url)
            .header("Authorization", format!("Bearer {}", provider.api_key))
            .header("content-type", "application/json")
            .json(&body)
            .send()
            .await
            .map_err(|e| SDKError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(SDKError::ApiError(format!(
                "HTTP {}: {}",
                status, error_text
            )));
        }

        // Parse response
        let openai_response: ChatResponse = response
            .json()
            .await
            .map_err(|e| SDKError::ParseError(e.to_string()))?;

        Ok(openai_response)
    }

    /// Call Google API
    async fn call_google_api(
        &self,
        provider: &crate::sdk::config::SdkProviderConfig,
        _request: SdkChatRequest,
    ) -> Result<ChatResponse> {
        Err(SDKError::ProviderError(format!(
            "Provider '{}' (Google) is not implemented in SDK client",
            provider.id
        )))
    }

    /// Convert messages to Anthropic format
    fn convert_messages_to_anthropic(
        &self,
        messages: &[Message],
    ) -> (Option<String>, Vec<serde_json::Value>) {
        let mut system_message = None;
        let mut anthropic_messages = Vec::new();

        for message in messages {
            match message.role {
                Role::System => {
                    if let Some(Content::Text(text)) = &message.content {
                        system_message = Some(text.clone());
                    }
                }
                Role::User => {
                    anthropic_messages.push(serde_json::json!({
                        "role": "user",
                        "content": self.convert_content_to_anthropic(message.content.as_ref())
                    }));
                }
                Role::Assistant => {
                    anthropic_messages.push(serde_json::json!({
                        "role": "assistant",
                        "content": self.convert_content_to_anthropic(message.content.as_ref())
                    }));
                }
                _ => {} // Ignore other roles
            }
        }

        (system_message, anthropic_messages)
    }

    /// Convert content to Anthropic format
    fn convert_content_to_anthropic(&self, content: Option<&Content>) -> serde_json::Value {
        match content {
            Some(Content::Text(text)) => serde_json::json!(text),
            Some(Content::Multimodal(parts)) => {
                let mut anthropic_content = Vec::new();
                for part in parts {
                    match part {
                        ContentPart::Text { text } => {
                            anthropic_content.push(serde_json::json!({
                                "type": "text",
                                "text": text
                            }));
                        }
                        ContentPart::Image { image_url } => {
                            anthropic_content.push(serde_json::json!({
                                "type": "image",
                                "source": {
                                    "type": "base64",
                                    "media_type": "image/jpeg",
                                    "data": image_url.url.trim_start_matches("data:image/jpeg;base64,")
                                }
                            }));
                        }
                        _ => {} // Ignore other types
                    }
                }
                serde_json::json!(anthropic_content)
            }
            None => serde_json::json!(""),
        }
    }

    /// Convert Anthropic response to standard format
    fn convert_anthropic_response(
        &self,
        anthropic_response: serde_json::Value,
        model: &str,
    ) -> Result<ChatResponse> {
        let id = anthropic_response
            .get("id")
            .and_then(|v| v.as_str())
            .unwrap_or("chatcmpl-anthropic")
            .to_string();

        let content = anthropic_response
            .get("content")
            .and_then(|v| v.as_array())
            .and_then(|arr| arr.first())
            .and_then(|item| item.get("text"))
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let usage = if let Some(u) = anthropic_response.get("usage") {
            Usage {
                prompt_tokens: u.get("input_tokens").and_then(|v| v.as_u64()).unwrap_or(0) as u32,
                completion_tokens: u.get("output_tokens").and_then(|v| v.as_u64()).unwrap_or(0)
                    as u32,
                total_tokens: 0, // Will be calculated below
            }
        } else {
            Usage::default()
        };

        let mut usage = usage;
        usage.total_tokens = usage.prompt_tokens + usage.completion_tokens;

        Ok(ChatResponse {
            id,
            model: model.to_string(),
            choices: vec![ChatChoice {
                index: 0,
                message: Message {
                    role: Role::Assistant,
                    content: Some(Content::Text(content)),
                    name: None,
                    tool_calls: None,
                },
                finish_reason: Some("stop".to_string()),
            }],
            usage,
            created: SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        })
    }
}

/// Trim trailing `\r` and `\n` bytes from a byte slice.
fn trim_line_end(b: &[u8]) -> &[u8] {
    let mut end = b.len();
    while end > 0 && (b[end - 1] == b'\n' || b[end - 1] == b'\r') {
        end -= 1;
    }
    &b[..end]
}

/// Parse an OpenAI-compatible SSE response into a stream of `ChatChunk`s.
///
/// OpenAI sends newline-delimited `data: <json>` lines ending with `data: [DONE]`.
/// The JSON structure matches `ChatChunk` directly, so each line is deserialized
/// straight into that type.
///
/// Bytes are accumulated in a raw buffer and split on `\n` boundaries so that
/// multi-byte UTF-8 characters split across HTTP chunks are handled correctly.
fn parse_sse_stream_openai(
    response: reqwest::Response,
) -> impl futures::Stream<Item = Result<ChatChunk>> + Send {
    async_stream::stream! {
        let mut byte_stream = response.bytes_stream();
        let mut raw_buffer: Vec<u8> = Vec::new();

        while let Some(chunk_result) = byte_stream.next().await {
            match chunk_result {
                Err(e) => {
                    yield Err(SDKError::NetworkError(e.to_string()));
                    return;
                }
                Ok(chunk) => {
                    raw_buffer.extend_from_slice(&chunk);

                    while let Some(pos) = raw_buffer.iter().position(|&b| b == b'\n') {
                        let line_bytes: Vec<u8> = raw_buffer.drain(..=pos).collect();
                        let trimmed = trim_line_end(&line_bytes);
                        let line = match std::str::from_utf8(trimmed) {
                            Ok(s) => s,
                            Err(e) => {
                                yield Err(SDKError::ParseError(e.to_string()));
                                return;
                            }
                        };

                        if let Some(data) = line.strip_prefix("data: ") {
                            if data == "[DONE]" {
                                return;
                            }
                            if !data.is_empty() {
                                match serde_json::from_str::<ChatChunk>(data) {
                                    Ok(chunk) => yield Ok(chunk),
                                    Err(e) => yield Err(SDKError::ParseError(format!(
                                        "Failed to parse OpenAI chunk: {}",
                                        e
                                    ))),
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Parse an Anthropic SSE response into a stream of `ChatChunk`s.
///
/// Anthropic sends typed events (`message_start`, `content_block_delta`, etc.).
/// Only `content_block_delta` events with a `text_delta` produce output chunks.
///
/// Bytes are accumulated in a raw buffer and split on `\n` boundaries so that
/// multi-byte UTF-8 characters split across HTTP chunks are handled correctly.
/// `event: error` lines are propagated as stream errors rather than silently dropped.
fn parse_sse_stream_anthropic(
    response: reqwest::Response,
) -> impl futures::Stream<Item = Result<ChatChunk>> + Send {
    async_stream::stream! {
        let mut byte_stream = response.bytes_stream();
        let mut raw_buffer: Vec<u8> = Vec::new();
        let mut message_id = String::from("msg_stream");
        let mut model = String::from("claude");
        let mut current_event = String::new();

        while let Some(chunk_result) = byte_stream.next().await {
            match chunk_result {
                Err(e) => {
                    yield Err(SDKError::NetworkError(e.to_string()));
                    return;
                }
                Ok(chunk) => {
                    raw_buffer.extend_from_slice(&chunk);

                    while let Some(pos) = raw_buffer.iter().position(|&b| b == b'\n') {
                        let line_bytes: Vec<u8> = raw_buffer.drain(..=pos).collect();
                        let trimmed = trim_line_end(&line_bytes);
                        let line = match std::str::from_utf8(trimmed) {
                            Ok(s) => s,
                            Err(e) => {
                                yield Err(SDKError::ParseError(e.to_string()));
                                return;
                            }
                        };

                        if line.is_empty() {
                            current_event.clear();
                            continue;
                        }

                        if let Some(event_name) = line.strip_prefix("event: ") {
                            current_event = event_name.trim().to_string();
                            continue;
                        }

                        if let Some(data) = line.strip_prefix("data: ") {
                            // Propagate in-band Anthropic stream errors (event: error)
                            if current_event == "error" {
                                let error_msg = serde_json::from_str::<serde_json::Value>(data)
                                    .ok()
                                    .and_then(|v| {
                                        v.get("error")
                                            .and_then(|e| e.get("message"))
                                            .and_then(|m| m.as_str())
                                            .map(|s| s.to_string())
                                    })
                                    .unwrap_or_else(|| data.to_string());
                                yield Err(SDKError::ApiError(format!(
                                    "Anthropic stream error: {}",
                                    error_msg
                                )));
                                return;
                            }

                            let value: serde_json::Value = match serde_json::from_str(data) {
                                Ok(v) => v,
                                Err(_) => continue,
                            };

                            match value.get("type").and_then(|v| v.as_str()).unwrap_or("") {
                                "message_start" => {
                                    if let Some(msg) = value.get("message") {
                                        if let Some(id) = msg.get("id").and_then(|v| v.as_str()) {
                                            message_id = id.to_string();
                                        }
                                        if let Some(m) = msg.get("model").and_then(|v| v.as_str()) {
                                            model = m.to_string();
                                        }
                                    }
                                }
                                "content_block_delta" => {
                                    if let Some(text) = value
                                        .get("delta")
                                        .and_then(|d| d.get("text"))
                                        .and_then(|t| t.as_str())
                                    {
                                        yield Ok(ChatChunk {
                                            id: message_id.clone(),
                                            model: model.clone(),
                                            choices: vec![ChunkChoice {
                                                index: 0,
                                                delta: MessageDelta {
                                                    role: None,
                                                    content: Some(text.to_string()),
                                                    tool_calls: None,
                                                },
                                                finish_reason: None,
                                            }],
                                        });
                                    }
                                }
                                "error" => {
                                    let error_msg = value
                                        .get("error")
                                        .and_then(|e| e.get("message"))
                                        .and_then(|m| m.as_str())
                                        .unwrap_or("unknown stream error")
                                        .to_string();
                                    yield Err(SDKError::ApiError(format!(
                                        "Anthropic stream error: {}",
                                        error_msg
                                    )));
                                    return;
                                }
                                "message_stop" => return,
                                _ => {}
                            }
                        }
                    }
                }
            }
        }
    }
}
