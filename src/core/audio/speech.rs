//! Text-to-speech functionality

use crate::core::providers::{Provider, ProviderRegistry};
use crate::utils::error::gateway_error::{GatewayError, Result};
use std::sync::Arc;
use tracing::info;

use super::transcription::parse_model_string;
use super::types::{SpeechRequest, SpeechResponse};

/// Audio service for handling text-to-speech requests
pub struct SpeechService {
    provider_registry: Arc<ProviderRegistry>,
}

impl SpeechService {
    /// Create a new speech service
    pub fn new(provider_registry: Arc<ProviderRegistry>) -> Self {
        Self { provider_registry }
    }

    /// Convert text to speech
    pub async fn speech(&self, request: SpeechRequest) -> Result<SpeechResponse> {
        info!(
            "Generating speech: model={}, voice={}, text_len={}",
            request.model,
            request.voice,
            request.input.len()
        );

        // Validate input length (max 4096 characters for most providers)
        if request.input.len() > 4096 {
            return Err(GatewayError::validation(
                "Input text too long (max 4096 characters)",
            ));
        }

        let (provider_name, _actual_model) = parse_model_string(&request.model);

        let providers = self.provider_registry.all();
        let provider = providers
            .iter()
            .find(|p| p.name() == provider_name)
            .ok_or_else(|| {
                GatewayError::internal(format!(
                    "No provider found for text-to-speech: {}",
                    provider_name
                ))
            })?;

        match provider {
            Provider::OpenAI(_openai) => {
                // OpenAI TTS implementation would go here
                Err(GatewayError::internal(
                    "OpenAI text-to-speech not yet implemented",
                ))
            }
            _ => Err(GatewayError::internal(format!(
                "Provider {} does not support text-to-speech",
                provider.name()
            ))),
        }
    }
}
