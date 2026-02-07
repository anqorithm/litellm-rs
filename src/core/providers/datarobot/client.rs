//! DataRobot Client

use crate::core::types::ModelInfo;
use crate::core::types::ChatRequest;
use serde_json::Value;

pub struct DataRobotClient;

impl DataRobotClient {
    pub fn supported_models() -> Vec<ModelInfo> {
        super::models::DataRobotModelRegistry::get_models()
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
