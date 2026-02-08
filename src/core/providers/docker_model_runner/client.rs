//! Docker Model Runner Client

use crate::core::types::chat::ChatRequest;
use crate::core::types::model::ModelInfo;
use serde_json::Value;

pub struct DockerModelRunnerClient;

impl DockerModelRunnerClient {
    pub fn supported_models() -> Vec<ModelInfo> {
        super::models::DockerModelRunnerModelRegistry::get_models()
    }

    pub fn supported_openai_params() -> &'static [&'static str] {
        &["temperature", "max_tokens", "top_p", "stream", "stop"]
    }

    pub fn transform_chat_request(request: ChatRequest) -> Value {
        serde_json::json!({
            "model": request.model,
            "messages": request.messages,
            "temperature": request.temperature,
            "max_tokens": request.max_tokens,
            "top_p": request.top_p,
            "stream": request.stream,
            "stop": request.stop,
        })
    }
}
