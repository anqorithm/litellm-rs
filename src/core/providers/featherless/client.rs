//! Featherless Client

use crate::core::types::chat::ChatRequest;
use crate::core::types::model::ModelInfo;
use serde_json::Value;

pub struct FeatherlessClient;

impl FeatherlessClient {
    pub fn supported_models() -> Vec<ModelInfo> {
        super::models::FeatherlessModelRegistry::get_models()
    }

    pub fn supported_openai_params() -> &'static [&'static str] {
        &[
            "temperature",
            "max_tokens",
            "top_p",
            "stream",
            "stop",
            "tools",
            "tool_choice",
        ]
    }

    pub fn transform_chat_request(request: ChatRequest) -> Value {
        let mut body = serde_json::json!({
            "model": request.model,
            "messages": request.messages,
            "stream": request.stream,
        });

        if let Some(temp) = request.temperature {
            body["temperature"] = serde_json::json!(temp);
        }
        if let Some(max_tokens) = request.max_tokens {
            body["max_tokens"] = serde_json::json!(max_tokens);
        }
        if let Some(top_p) = request.top_p {
            body["top_p"] = serde_json::json!(top_p);
        }
        if let Some(stop) = request.stop {
            body["stop"] = serde_json::json!(stop);
        }
        if let Some(tools) = request.tools {
            body["tools"] = serde_json::json!(tools);
        }
        if let Some(tool_choice) = request.tool_choice {
            body["tool_choice"] = serde_json::json!(tool_choice);
        }

        body
    }
}
