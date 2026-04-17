//! Chat completion methods

use super::llm_client::LLMClient;
use crate::sdk::{errors::*, types::*};
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
        _messages: Vec<Message>,
    ) -> Result<impl futures::Stream<Item = Result<ChatChunk>>> {
        let provider = self
            .config
            .providers
            .iter()
            .find(|p| p.id == provider_id)
            .ok_or_else(|| SDKError::ProviderNotFound(provider_id.to_string()))?;

        Err::<futures::stream::Empty<Result<ChatChunk>>, _>(SDKError::ProviderError(format!(
            "Streaming is not implemented for provider type {:?}",
            provider.provider_type
        )))
    }

    /// Call Anthropic API
    async fn call_anthropic_api(
        &self,
        provider: &crate::sdk::config::SdkProviderConfig,
        request: SdkChatRequest,
    ) -> Result<ChatResponse> {
        // Convert message format
        let (system_message, anthropic_messages) =
            self.convert_messages_to_anthropic(&request.messages)?;

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
    ) -> Result<(Option<String>, Vec<serde_json::Value>)> {
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
                        "content": self.convert_content_to_anthropic(message.content.as_ref())?
                    }));
                }
                Role::Assistant => {
                    anthropic_messages.push(serde_json::json!({
                        "role": "assistant",
                        "content": self.convert_content_to_anthropic(message.content.as_ref())?
                    }));
                }
                _ => {}
            }
        }

        Ok((system_message, anthropic_messages))
    }

    /// Parse `data:<media_type>;base64,<data>` into `(media_type, base64_data)`.
    /// Returns `None` for plain URLs, non-base64 data URIs, or malformed data URIs.
    /// Requires the explicit `;base64,` marker — `data:image/png;charset=utf-8,…` returns `None`.
    fn parse_data_uri(url: &str) -> Option<(&str, &str)> {
        let rest = url.strip_prefix("data:")?;
        // Split on the explicit ";base64," marker so non-base64 params (charset, name, …) are rejected.
        let (header, data) = rest.split_once(";base64,")?;
        // Strip any trailing media-type parameters (e.g. `image/png;charset=utf-8` → `image/png`).
        let media_type = header.split(';').next().filter(|s| !s.is_empty())?;
        Some((media_type, data))
    }

    /// Convert content to Anthropic format.
    /// Returns `Err(SDKError::InvalidRequest)` for `data:` URIs that lack the `;base64,` marker.
    fn convert_content_to_anthropic(
        &self,
        content: Option<&Content>,
    ) -> Result<serde_json::Value> {
        match content {
            Some(Content::Text(text)) => Ok(serde_json::json!(text)),
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
                            let url = &image_url.url;
                            if url.starts_with("data:") {
                                match Self::parse_data_uri(url) {
                                    Some((media_type, data)) => {
                                        anthropic_content.push(serde_json::json!({
                                            "type": "image",
                                            "source": {
                                                "type": "base64",
                                                "media_type": media_type,
                                                "data": data
                                            }
                                        }));
                                    }
                                    None => {
                                        // Truncate to 100 chars to avoid large binary payloads in logs/responses.
                                        let preview: String = url.chars().take(100).collect();
                                        return Err(SDKError::InvalidRequest(format!(
                                            "data URI must use ';base64,' encoding: {}",
                                            preview
                                        )));
                                    }
                                }
                            } else {
                                anthropic_content.push(serde_json::json!({
                                    "type": "image",
                                    "source": {
                                        "type": "url",
                                        "url": url
                                    }
                                }));
                            }
                        }
                        _ => {}
                    }
                }
                Ok(serde_json::json!(anthropic_content))
            }
            None => Ok(serde_json::json!("")),
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sdk::{
        config::{ConfigBuilder, ProviderType, SdkProviderConfig},
        types::{Content, ContentPart, ImageUrl},
    };
    use std::collections::HashMap;

    fn make_client() -> LLMClient {
        let config = ConfigBuilder::new()
            .add_provider(SdkProviderConfig {
                id: "anthropic".to_string(),
                provider_type: ProviderType::Anthropic,
                name: "Anthropic".to_string(),
                api_key: "test-key".to_string(),
                base_url: None,
                models: vec!["claude-3-5-sonnet-20241022".to_string()],
                enabled: true,
                weight: 1.0,
                rate_limit_rpm: None,
                rate_limit_tpm: None,
                settings: HashMap::new(),
            })
            .build();
        LLMClient::new(config).unwrap()
    }

    // parse_data_uri tests

    #[test]
    fn test_parse_data_uri_jpeg() {
        let (mt, data) = LLMClient::parse_data_uri("data:image/jpeg;base64,/9j/abc").unwrap();
        assert_eq!(mt, "image/jpeg");
        assert_eq!(data, "/9j/abc");
    }

    #[test]
    fn test_parse_data_uri_png() {
        let (mt, data) = LLMClient::parse_data_uri("data:image/png;base64,iVBOR").unwrap();
        assert_eq!(mt, "image/png");
        assert_eq!(data, "iVBOR");
    }

    #[test]
    fn test_parse_data_uri_webp() {
        let (mt, data) = LLMClient::parse_data_uri("data:image/webp;base64,UklGR").unwrap();
        assert_eq!(mt, "image/webp");
        assert_eq!(data, "UklGR");
    }

    #[test]
    fn test_parse_data_uri_gif() {
        let (mt, data) = LLMClient::parse_data_uri("data:image/gif;base64,R0lGO").unwrap();
        assert_eq!(mt, "image/gif");
        assert_eq!(data, "R0lGO");
    }

    #[test]
    fn test_parse_data_uri_plain_url_returns_none() {
        assert!(LLMClient::parse_data_uri("https://example.com/image.png").is_none());
    }

    // convert_content_to_anthropic tests

    #[test]
    fn test_png_image_uses_correct_media_type() {
        let client = make_client();
        let content = Content::Multimodal(vec![ContentPart::Image {
            image_url: ImageUrl {
                url: "data:image/png;base64,iVBORw0KGgo=".to_string(),
                detail: None,
            },
        }]);
        let result = client.convert_content_to_anthropic(Some(&content)).unwrap();
        let source = &result[0]["source"];
        assert_eq!(source["media_type"], "image/png");
        assert_eq!(source["data"], "iVBORw0KGgo=");
        assert_eq!(source["type"], "base64");
    }

    #[test]
    fn test_webp_image_uses_correct_media_type() {
        let client = make_client();
        let content = Content::Multimodal(vec![ContentPart::Image {
            image_url: ImageUrl {
                url: "data:image/webp;base64,UklGRgAA".to_string(),
                detail: None,
            },
        }]);
        let result = client.convert_content_to_anthropic(Some(&content)).unwrap();
        let source = &result[0]["source"];
        assert_eq!(source["media_type"], "image/webp");
        assert_eq!(source["data"], "UklGRgAA");
    }

    #[test]
    fn test_jpeg_image_still_works() {
        let client = make_client();
        let content = Content::Multimodal(vec![ContentPart::Image {
            image_url: ImageUrl {
                url: "data:image/jpeg;base64,/9j/4AAQ".to_string(),
                detail: None,
            },
        }]);
        let result = client.convert_content_to_anthropic(Some(&content)).unwrap();
        let source = &result[0]["source"];
        assert_eq!(source["media_type"], "image/jpeg");
        assert_eq!(source["data"], "/9j/4AAQ");
    }

    #[test]
    fn test_plain_url_image_uses_url_source_type() {
        let client = make_client();
        let content = Content::Multimodal(vec![ContentPart::Image {
            image_url: ImageUrl {
                url: "https://example.com/photo.png".to_string(),
                detail: None,
            },
        }]);
        let result = client.convert_content_to_anthropic(Some(&content)).unwrap();
        let source = &result[0]["source"];
        assert_eq!(source["type"], "url");
        assert_eq!(source["url"], "https://example.com/photo.png");
    }

    #[test]
    fn test_non_base64_data_uri_returns_error() {
        let client = make_client();
        let content = Content::Multimodal(vec![ContentPart::Image {
            image_url: ImageUrl {
                url: "data:image/png;charset=utf-8,abc".to_string(),
                detail: None,
            },
        }]);
        let err = client.convert_content_to_anthropic(Some(&content)).unwrap_err();
        assert!(matches!(err, SDKError::InvalidRequest(_)));
    }

    #[test]
    fn test_non_base64_data_uri_name_param_returns_error() {
        let client = make_client();
        let content = Content::Multimodal(vec![ContentPart::Image {
            image_url: ImageUrl {
                url: "data:image/png;name=foo,abc".to_string(),
                detail: None,
            },
        }]);
        let err = client.convert_content_to_anthropic(Some(&content)).unwrap_err();
        assert!(matches!(err, SDKError::InvalidRequest(_)));
    }

    #[test]
    fn test_non_base64_error_message_truncated() {
        let client = make_client();
        let long_payload = "data:image/png;name=foo,".to_string() + &"A".repeat(500);
        let content = Content::Multimodal(vec![ContentPart::Image {
            image_url: ImageUrl {
                url: long_payload,
                detail: None,
            },
        }]);
        let err = client.convert_content_to_anthropic(Some(&content)).unwrap_err();
        if let SDKError::InvalidRequest(msg) = err {
            // Error message must not embed large payloads (truncated to 100 chars of the URI).
            assert!(msg.len() < 200, "error message too long: {} chars", msg.len());
        } else {
            panic!("expected InvalidRequest");
        }
    }

    #[test]
    fn test_parse_data_uri_with_media_type_params() {
        // Valid base64 URI with a media-type parameter before ;base64,
        let (mt, data) =
            LLMClient::parse_data_uri("data:image/png;charset=utf-8;base64,iVBOR").unwrap();
        assert_eq!(mt, "image/png");
        assert_eq!(data, "iVBOR");
    }

    #[test]
    fn test_parse_data_uri_non_base64_returns_none() {
        assert!(LLMClient::parse_data_uri("data:image/png;charset=utf-8,abc").is_none());
        assert!(LLMClient::parse_data_uri("data:image/png;name=foo,abc").is_none());
        assert!(LLMClient::parse_data_uri("data:image/png,abc").is_none());
    }
}
