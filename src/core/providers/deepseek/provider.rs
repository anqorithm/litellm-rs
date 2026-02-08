//! DeepSeek Provider Implementation
//!
//! Main provider implementation using the unified base infrastructure

use futures::Stream;
use serde_json::Value;
use std::collections::HashMap;
use std::pin::Pin;

use crate::core::providers::base::{
    HeaderPair, HttpMethod, get_pricing_db, header, header_owned, streaming_client,
};
use crate::core::providers::unified_provider::ProviderError;
use crate::core::types::{
    ChatRequest,
    context::RequestContext,
    health::HealthStatus,
    model::ProviderCapability,
    responses::{ChatChunk, ChatResponse},
};

use super::{DeepSeekClient, DeepSeekConfig};

const PROVIDER_NAME: &str = "deepseek";
const CAPABILITIES: &[ProviderCapability] = &[
    ProviderCapability::ChatCompletion,
    ProviderCapability::ChatCompletionStream,
    ProviderCapability::ToolCalling,
];

crate::define_pooled_http_provider_with_hooks!(
    provider: PROVIDER_NAME,
    struct_name: DeepSeekProvider,
    config: super::DeepSeekConfig,
    error_mapper: super::DeepSeekErrorMapper,
    model_info: DeepSeekClient::supported_models,
    capabilities: CAPABILITIES,
    url_builder: |provider: &DeepSeekProvider| -> String {
        format!(
            "{}/v1/chat/completions",
            provider.config.base.get_effective_api_base(PROVIDER_NAME)
        )
    },
    http_method: HttpMethod::POST,
    supported_params: [
        "temperature",
        "max_tokens",
        "top_p",
        "frequency_penalty",
        "presence_penalty",
        "stream",
        "tools",
        "tool_choice",
    ],
    build_headers: |provider: &DeepSeekProvider| -> Vec<HeaderPair> {
        provider.get_request_headers()
    },
    with_api_key: |api_key: String| -> Result<DeepSeekProvider, ProviderError> {
        let mut config = DeepSeekConfig::new(PROVIDER_NAME);
        config.base.api_key = Some(api_key);
        DeepSeekProvider::new(config)
    },
    map_openai_params: |_provider: &DeepSeekProvider,
                        params: HashMap<String, Value>,
                        _model: &str|
     -> Result<HashMap<String, Value>, ProviderError> { Ok(params) },
    request_transform: |_provider: &DeepSeekProvider, request: ChatRequest|
     -> Result<Value, ProviderError> { Ok(DeepSeekClient::transform_chat_request(request)) },
    response_transform: |_provider: &DeepSeekProvider,
                         raw_response: &[u8],
                         _model: &str,
                         _request_id: &str|
     -> Result<ChatResponse, ProviderError> {
        let response: Value = serde_json::from_slice(raw_response)
            .map_err(|e| ProviderError::response_parsing(PROVIDER_NAME, e.to_string()))?;
        DeepSeekClient::transform_chat_response(response)
    },
    error_map: |_provider: &DeepSeekProvider,
                status: u16,
                error_text: String,
                _request: &ChatRequest|
     -> ProviderError {
        if let Ok(value) = serde_json::from_str::<Value>(&error_text) {
            if let Err(err) = DeepSeekClient::transform_chat_response(value) {
                return err;
            }
        }

        ProviderError::api_error(PROVIDER_NAME, status, error_text)
    },
    health_check: |provider: &DeepSeekProvider| {
        let has_key = provider
            .config
            .base
            .get_effective_api_key(PROVIDER_NAME)
            .is_some();
        async move {
            if has_key {
                HealthStatus::Healthy
            } else {
                HealthStatus::Unhealthy
            }
        }
    },
    streaming: |provider: &DeepSeekProvider, request: ChatRequest, _context: RequestContext| {
        let url = format!(
            "{}/v1/chat/completions",
            provider.config.base.get_effective_api_base(PROVIDER_NAME)
        );
        let api_key = provider.config.base.get_effective_api_key(PROVIDER_NAME);

        let mut body = DeepSeekClient::transform_chat_request(request);
        body["stream"] = serde_json::Value::Bool(true);

        async move {
            let api_key = api_key.ok_or_else(|| {
                ProviderError::authentication(PROVIDER_NAME, "API key is required")
            })?;

            let client = streaming_client();
            let response = client
                .post(&url)
                .header("Authorization", format!("Bearer {}", api_key))
                .header("Content-Type", "application/json")
                .json(&body)
                .send()
                .await
                .map_err(|e| ProviderError::network(PROVIDER_NAME, e.to_string()))?;

            let status = response.status();
            if !status.is_success() {
                let error_text = response
                    .text()
                    .await
                    .unwrap_or_else(|_| "Unknown error".to_string());
                return Err(ProviderError::api_error(
                    PROVIDER_NAME,
                    status.as_u16(),
                    error_text,
                ));
            }

            let stream = super::streaming::create_deepseek_stream(response.bytes_stream());
            let stream: Pin<Box<dyn Stream<Item = Result<ChatChunk, ProviderError>> + Send>> =
                Box::pin(stream);
            Ok(stream)
        }
    },
    calculate_cost: |_provider: &DeepSeekProvider,
                     model: &str,
                     input_tokens: u32,
                     output_tokens: u32|
     -> Result<f64, ProviderError> {
        let usage = crate::core::providers::base::pricing::Usage {
            prompt_tokens: input_tokens,
            completion_tokens: output_tokens,
            total_tokens: input_tokens + output_tokens,
            reasoning_tokens: None,
        };

        Ok(get_pricing_db().calculate(model, &usage))
    },
);

impl DeepSeekProvider {
    pub fn from_env() -> Result<Self, ProviderError> {
        let config = DeepSeekConfig::from_env();
        Self::new(config)
    }

    fn get_request_headers(&self) -> Vec<HeaderPair> {
        let mut headers = Vec::with_capacity(2);

        if let Some(api_key) = &self.config.base.api_key {
            headers.push(header("Authorization", format!("Bearer {}", api_key)));
        }

        for (key, value) in &self.config.base.headers {
            headers.push(header_owned(key.clone(), value.clone()));
        }

        headers
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::traits::provider::llm_provider::trait_definition::LLMProvider;

    #[test]
    fn test_provider_creation() {
        let mut config = DeepSeekConfig::new("deepseek");
        config.base.api_key = Some("test-api-key".to_string());

        let provider = DeepSeekProvider::new(config);
        assert!(provider.is_ok());
    }

    #[test]
    fn test_provider_name() {
        let mut config = DeepSeekConfig::new("deepseek");
        config.base.api_key = Some("test-api-key".to_string());

        let provider = DeepSeekProvider::new(config).unwrap();
        assert_eq!(provider.name(), "deepseek");
    }

    #[test]
    fn test_provider_capabilities() {
        let mut config = DeepSeekConfig::new("deepseek");
        config.base.api_key = Some("test-api-key".to_string());

        let provider = DeepSeekProvider::new(config).unwrap();
        let caps = provider.capabilities();

        assert!(caps.contains(&ProviderCapability::ChatCompletion));
        assert!(caps.contains(&ProviderCapability::ChatCompletionStream));
        assert!(caps.contains(&ProviderCapability::ToolCalling));
    }
}
