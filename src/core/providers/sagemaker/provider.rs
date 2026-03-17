//! AWS Sagemaker Provider Implementation
//!
//! Implements the LLMProvider trait for AWS Sagemaker endpoints.

use std::collections::HashMap;
use std::sync::Arc;

use super::config::SagemakerConfig;
use super::error::SagemakerError;
use super::sigv4::SagemakerSigV4Signer;
use crate::core::providers::base::GlobalPoolManager;
use crate::core::providers::unified_provider::ProviderError;
use crate::core::traits::provider::ProviderConfig as _;
use crate::core::traits::provider::llm_provider::trait_definition::LLMProvider;
use crate::core::types::chat::ChatRequest;
use crate::core::types::responses::ChatResponse;
use crate::core::types::{model::ModelInfo, model::ProviderCapability};

/// Static capabilities for Sagemaker provider
const SAGEMAKER_CAPABILITIES: &[ProviderCapability] = &[
    ProviderCapability::ChatCompletion,
    ProviderCapability::ChatCompletionStream,
];

/// AWS Sagemaker provider implementation
#[derive(Debug, Clone)]
pub struct SagemakerProvider {
    config: SagemakerConfig,
    #[allow(dead_code)]
    pool_manager: Arc<GlobalPoolManager>,
    signer: SagemakerSigV4Signer,
    models: Vec<ModelInfo>,
}

impl SagemakerProvider {
    /// Create a new Sagemaker provider instance
    pub async fn new(config: SagemakerConfig) -> Result<Self, SagemakerError> {
        config
            .validate()
            .map_err(|e| ProviderError::configuration("sagemaker", e))?;

        let pool_manager = Arc::new(GlobalPoolManager::new().map_err(|e| {
            ProviderError::configuration(
                "sagemaker",
                format!("Failed to create pool manager: {}", e),
            )
        })?);

        let signer = SagemakerSigV4Signer::new(
            config.get_access_key_id().unwrap_or_default(),
            config.get_secret_access_key().unwrap_or_default(),
            config.get_session_token(),
            config.get_region(),
        );

        // Sagemaker doesn't have a fixed model list - models are custom endpoints
        let models = vec![ModelInfo {
            id: "sagemaker-endpoint".to_string(),
            name: "Sagemaker Endpoint".to_string(),
            provider: "sagemaker".to_string(),
            max_context_length: 4096,
            max_output_length: Some(4096),
            supports_streaming: true,
            supports_tools: false,
            supports_multimodal: false,
            input_cost_per_1k_tokens: None,
            output_cost_per_1k_tokens: None,
            currency: "USD".to_string(),
            capabilities: vec![
                ProviderCapability::ChatCompletion,
                ProviderCapability::ChatCompletionStream,
            ],
            created_at: None,
            updated_at: None,
            metadata: HashMap::new(),
        }];

        Ok(Self {
            config,
            pool_manager,
            signer,
            models,
        })
    }

    /// Create provider with AWS credentials
    pub async fn with_credentials(
        access_key_id: impl Into<String>,
        secret_access_key: impl Into<String>,
        region: impl Into<String>,
    ) -> Result<Self, SagemakerError> {
        let config = SagemakerConfig {
            aws_access_key_id: Some(access_key_id.into()),
            aws_secret_access_key: Some(secret_access_key.into()),
            aws_region: Some(region.into()),
            ..Default::default()
        };
        Self::new(config).await
    }
}

/// Format messages for HuggingFace TGI format
fn format_messages_for_tgi(request: &ChatRequest) -> String {
    let mut prompt = String::new();

    for message in &request.messages {
        let role = match message.role {
            crate::core::types::message::MessageRole::System => "System",
            crate::core::types::message::MessageRole::User => "User",
            crate::core::types::message::MessageRole::Assistant => "Assistant",
            _ => "User",
        };

        if let Some(content) = &message.content {
            let text = match content {
                crate::core::types::message::MessageContent::Text(t) => t.clone(),
                crate::core::types::message::MessageContent::Parts(parts) => parts
                    .iter()
                    .filter_map(|p| {
                        if let crate::core::types::content::ContentPart::Text { text } = p {
                            Some(text.clone())
                        } else {
                            None
                        }
                    })
                    .collect::<Vec<_>>()
                    .join(" "),
            };
            prompt.push_str(&format!("{}: {}\n", role, text));
        }
    }

    prompt.push_str("Assistant:");
    prompt
}

/// Parse HuggingFace TGI response
fn parse_tgi_response(response_bytes: &[u8], model: &str) -> Result<ChatResponse, SagemakerError> {
    let json: serde_json::Value = serde_json::from_slice(response_bytes).map_err(|e| {
        ProviderError::response_parsing("sagemaker", format!("Failed to parse response: {}", e))
    })?;

    // TGI returns either a single object or an array
    let generated_text = if let Some(arr) = json.as_array() {
        arr.first()
            .and_then(|v| v.get("generated_text"))
            .and_then(|v| v.as_str())
            .unwrap_or("")
    } else {
        json.get("generated_text")
            .and_then(|v| v.as_str())
            .unwrap_or("")
    };

    Ok(ChatResponse {
        id: format!("sagemaker-{}", uuid::Uuid::new_v4().simple()),
        object: "chat.completion".to_string(),
        created: chrono::Utc::now().timestamp(),
        model: format!("sagemaker/{}", model),
        choices: vec![crate::core::types::responses::ChatChoice {
            index: 0,
            message: crate::core::types::chat::ChatMessage {
                role: crate::core::types::message::MessageRole::Assistant,
                content: Some(crate::core::types::message::MessageContent::Text(
                    generated_text.to_string(),
                )),
                thinking: None,
                name: None,
                tool_calls: None,
                tool_call_id: None,
                function_call: None,
            },
            finish_reason: Some(crate::core::types::responses::FinishReason::Stop),
            logprobs: None,
        }],
        usage: None,
        system_fingerprint: None,
    })
}
