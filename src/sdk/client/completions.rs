//! Chat completion methods

use super::llm_client::LLMClient;
use crate::sdk::{errors::*, types::*};
use std::time::SystemTime;
use tracing::{debug, error, warn};

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
    ) -> Result<impl futures::Stream<Item = Result<ChatChunk>>> {
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
            _ => {
                warn!(
                    "Provider type {:?} not fully implemented, using mock response",
                    provider.provider_type
                );
                self.create_mock_response(provider, &request.messages).await
            }
        }
    }

    /// Execute stream request
    pub(crate) async fn execute_stream_request(
        &self,
        provider_id: &str,
        _messages: Vec<Message>,
    ) -> Result<impl futures::Stream<Item = Result<ChatChunk>>> {
        let _provider = self
            .config
            .providers
            .iter()
            .find(|p| p.id == provider_id)
            .ok_or_else(|| SDKError::ProviderNotFound(provider_id.to_string()))?;

        // Stream implementation
        // Currently returns a simple mock stream
        use futures::stream;

        let chunk = ChatChunk {
            id: "stream-test".to_string(),
            model: "test-model".to_string(),
            choices: vec![ChunkChoice {
                index: 0,
                delta: MessageDelta {
                    role: Some(Role::Assistant),
                    content: Some("Streaming response...".to_string()),
                    tool_calls: None,
                },
                finish_reason: Some("stop".to_string()),
            }],
        };

        Ok(stream::once(async move { Ok(chunk) }))
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

        let default_url = "https://api.openai.com".to_string();
        let base_url = provider.base_url.as_ref().unwrap_or(&default_url);
        let url = format!("{}/v1/chat/completions", base_url.trim_end_matches('/'));

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
        request: SdkChatRequest,
    ) -> Result<ChatResponse> {
        // Google API implementation placeholder
        warn!("Google API not fully implemented, using mock response");
        self.create_mock_response(provider, &request.messages).await
    }

    /// Create mock response for testing
    async fn create_mock_response(
        &self,
        provider: &crate::sdk::config::SdkProviderConfig,
        messages: &[Message],
    ) -> Result<ChatResponse> {
        let user_message = messages
            .iter()
            .rfind(|m| matches!(m.role, Role::User))
            .and_then(|m| match &m.content {
                Some(Content::Text(text)) => Some(text.as_str()),
                _ => None,
            })
            .unwrap_or("Hello");

        let mock_content = format!(
            "Mock response from {} provider. You said: \"{}\"",
            provider.name, user_message
        );

        Ok(ChatResponse {
            id: format!("mock-{}", uuid::Uuid::new_v4()),
            model: provider
                .models
                .first()
                .unwrap_or(&"mock-model".to_string())
                .clone(),
            choices: vec![ChatChoice {
                index: 0,
                message: Message {
                    role: Role::Assistant,
                    content: Some(Content::Text(mock_content)),
                    name: None,
                    tool_calls: None,
                },
                finish_reason: Some("stop".to_string()),
            }],
            usage: Usage {
                prompt_tokens: 10,
                completion_tokens: 15,
                total_tokens: 25,
            },
            created: SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        })
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
