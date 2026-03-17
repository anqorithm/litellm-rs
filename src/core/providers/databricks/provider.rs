//! Databricks Provider Implementation
//!
//! Main provider implementation for Databricks Foundation Model APIs.

use serde_json::Value;
use std::sync::Arc;

use crate::core::providers::base::{GlobalPoolManager, HeaderPair, header, header_owned};
use crate::core::providers::unified_provider::ProviderError;
use crate::core::traits::{
    provider::ProviderConfig, provider::llm_provider::trait_definition::LLMProvider,
};
use crate::core::types::{
    chat::ChatMessage,
    chat::ChatRequest,
    message::MessageContent,
    model::ModelInfo,
    responses::{ChatChoice, ChatResponse, FinishReason, Usage},
};

use super::{DatabricksConfig, get_databricks_registry};

#[derive(Debug, Clone)]
pub struct DatabricksProvider {
    config: DatabricksConfig,
    pool_manager: Arc<GlobalPoolManager>,
    supported_models: Vec<ModelInfo>,
}

impl DatabricksProvider {
    /// Generate headers for Databricks API requests
    fn get_request_headers(&self) -> Vec<HeaderPair> {
        let mut headers = Vec::with_capacity(3);

        if let Some(api_key) = &self.config.base.api_key {
            headers.push(header("Authorization", format!("Bearer {}", api_key)));
        }
        headers.push(header("Content-Type", "application/json".to_string()));

        // Add custom user agent
        let user_agent = DatabricksConfig::build_user_agent(None);
        headers.push(header("User-Agent", user_agent));

        // Add custom headers
        for (key, value) in &self.config.base.headers {
            headers.push(header_owned(key.clone(), value.clone()));
        }

        headers
    }

    /// Create a new Databricks provider
    pub fn new(config: DatabricksConfig) -> Result<Self, ProviderError> {
        config
            .validate()
            .map_err(|e| ProviderError::configuration("databricks", e))?;

        let pool_manager = Arc::new(
            GlobalPoolManager::new()
                .map_err(|e| ProviderError::configuration("databricks", e.to_string()))?,
        );
        let supported_models = get_databricks_registry().models().to_vec();

        Ok(Self {
            config,
            pool_manager,
            supported_models,
        })
    }

    /// Create provider from environment variables
    pub fn from_env() -> Result<Self, ProviderError> {
        let config = DatabricksConfig::from_env();
        Self::new(config)
    }

    /// Create provider with credentials
    pub fn with_credentials(
        api_key: impl Into<String>,
        api_base: impl Into<String>,
    ) -> Result<Self, ProviderError> {
        let config = DatabricksConfig::with_credentials(api_key, api_base);
        Self::new(config)
    }

    /// Get the endpoint name from model
    fn get_endpoint_name(&self, model: &str) -> String {
        // Remove provider prefix if present
        let model_name = model.strip_prefix("databricks/").unwrap_or(model);

        model_name.to_string()
    }

    /// Build the full URL for a serving endpoint
    fn build_endpoint_url(
        &self,
        model: &str,
        _endpoint_type: &str,
    ) -> Result<String, ProviderError> {
        let base = self.config.get_serving_endpoint_base().ok_or_else(|| {
            ProviderError::configuration("databricks", "API base URL is required")
        })?;

        let endpoint_name = self.get_endpoint_name(model);

        Ok(format!("{}/{}/invocations", base, endpoint_name))
    }

    /// Transform chat request to Databricks format
    fn transform_chat_request_to_value(&self, request: &ChatRequest) -> Value {
        let registry = get_databricks_registry();
        let is_claude = registry.is_claude_model(&request.model);

        let mut body = serde_json::json!({
            "messages": self.transform_messages(&request.messages, is_claude),
        });

        // Add optional parameters
        if let Some(temperature) = request.temperature {
            body["temperature"] = serde_json::json!(temperature);
        }
        if let Some(top_p) = request.top_p {
            body["top_p"] = serde_json::json!(top_p);
        }
        if let Some(max_tokens) = request.max_tokens {
            body["max_tokens"] = serde_json::json!(max_tokens);
        }
        if let Some(n) = request.n {
            body["n"] = serde_json::json!(n);
        }
        if let Some(stop) = &request.stop {
            body["stop"] = serde_json::json!(stop);
        }
        if request.stream {
            body["stream"] = serde_json::json!(true);
        }

        // Tool calling (Claude on Databricks)
        if let Some(tools) = &request.tools {
            body["tools"] = serde_json::json!(tools);
        }
        if let Some(tool_choice) = &request.tool_choice {
            body["tool_choice"] = serde_json::json!(tool_choice);
        }

        body
    }

    /// Transform messages for Databricks
    fn transform_messages(&self, messages: &[ChatMessage], is_claude: bool) -> Vec<Value> {
        messages
            .iter()
            .map(|msg| {
                let mut message = serde_json::json!({
                    "role": msg.role.to_string(),
                });

                // Handle content based on type
                match &msg.content {
                    Some(MessageContent::Text(text)) => {
                        message["content"] = serde_json::json!(text);
                    }
                    Some(MessageContent::Parts(parts)) => {
                        if is_claude {
                            // Claude can handle multimodal content
                            let content_parts: Vec<Value> = parts
                                .iter()
                                .map(|part| serde_json::to_value(part).unwrap_or(Value::Null))
                                .collect();
                            message["content"] = serde_json::json!(content_parts);
                        } else {
                            // For non-Claude models, extract text only
                            let text: String = parts
                                .iter()
                                .filter_map(|part| {
                                    if let crate::core::types::content::ContentPart::Text {
                                        text,
                                        ..
                                    } = part
                                    {
                                        Some(text.as_str())
                                    } else {
                                        None
                                    }
                                })
                                .collect::<Vec<_>>()
                                .join("\n");
                            message["content"] = serde_json::json!(text);
                        }
                    }
                    None => {
                        message["content"] = serde_json::json!("");
                    }
                }

                // Add tool calls if present
                if let Some(tool_calls) = &msg.tool_calls {
                    message["tool_calls"] = serde_json::json!(tool_calls);
                }
                if let Some(tool_call_id) = &msg.tool_call_id {
                    message["tool_call_id"] = serde_json::json!(tool_call_id);
                }
                if let Some(name) = &msg.name {
                    message["name"] = serde_json::json!(name);
                }

                message
            })
            .collect()
    }

    /// Parse Databricks chat response
    fn parse_chat_response(
        &self,
        response: &Value,
        model: &str,
    ) -> Result<ChatResponse, ProviderError> {
        let id = response
            .get("id")
            .and_then(|v| v.as_str())
            .unwrap_or("chatcmpl-databricks")
            .to_string();

        let created = response
            .get("created")
            .and_then(|v| v.as_i64())
            .unwrap_or_else(|| chrono::Utc::now().timestamp());

        let mut choices = Vec::new();

        if let Some(choices_array) = response.get("choices").and_then(|v| v.as_array()) {
            for choice in choices_array {
                let index = choice.get("index").and_then(|v| v.as_u64()).unwrap_or(0) as u32;

                let message = if let Some(msg) = choice.get("message") {
                    let role = msg
                        .get("role")
                        .and_then(|v| v.as_str())
                        .map(|r| match r {
                            "assistant" => crate::core::types::message::MessageRole::Assistant,
                            "user" => crate::core::types::message::MessageRole::User,
                            "system" => crate::core::types::message::MessageRole::System,
                            "tool" => crate::core::types::message::MessageRole::Tool,
                            _ => crate::core::types::message::MessageRole::Assistant,
                        })
                        .unwrap_or(crate::core::types::message::MessageRole::Assistant);

                    let content = msg
                        .get("content")
                        .and_then(|v| v.as_str())
                        .map(|s| MessageContent::Text(s.to_string()));

                    ChatMessage {
                        role,
                        content,
                        thinking: None,
                        name: None,
                        tool_calls: msg
                            .get("tool_calls")
                            .and_then(|tc| serde_json::from_value(tc.clone()).ok()),
                        tool_call_id: None,
                        function_call: None,
                    }
                } else {
                    ChatMessage {
                        role: crate::core::types::message::MessageRole::Assistant,
                        content: None,
                        thinking: None,
                        name: None,
                        tool_calls: None,
                        tool_call_id: None,
                        function_call: None,
                    }
                };

                let finish_reason = choice
                    .get("finish_reason")
                    .and_then(|v| v.as_str())
                    .and_then(|r| match r {
                        "stop" => Some(FinishReason::Stop),
                        "length" => Some(FinishReason::Length),
                        "tool_calls" => Some(FinishReason::ToolCalls),
                        "content_filter" => Some(FinishReason::ContentFilter),
                        _ => None,
                    });

                choices.push(ChatChoice {
                    index,
                    message,
                    finish_reason,
                    logprobs: None,
                });
            }
        }

        let usage = response.get("usage").map(|u| Usage {
            prompt_tokens: u.get("prompt_tokens").and_then(|v| v.as_u64()).unwrap_or(0) as u32,
            completion_tokens: u
                .get("completion_tokens")
                .and_then(|v| v.as_u64())
                .unwrap_or(0) as u32,
            total_tokens: u.get("total_tokens").and_then(|v| v.as_u64()).unwrap_or(0) as u32,
            prompt_tokens_details: None,
            completion_tokens_details: None,
            thinking_usage: None,
        });

        Ok(ChatResponse {
            id,
            object: "chat.completion".to_string(),
            created,
            model: model.to_string(),
            choices,
            usage,
            system_fingerprint: None,
        })
    }
}
