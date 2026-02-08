//! Audio translation functionality

use crate::core::providers::{Provider, ProviderRegistry};
use crate::utils::error::error::{GatewayError, Result};
use std::sync::Arc;
use tracing::info;

use super::transcription::parse_model_string;
use super::types::{SegmentInfo, TranslationRequest, TranslationResponse};

/// Audio service for handling audio translation requests
pub struct TranslationService {
    provider_registry: Arc<ProviderRegistry>,
}

impl TranslationService {
    /// Create a new translation service
    pub fn new(provider_registry: Arc<ProviderRegistry>) -> Self {
        Self { provider_registry }
    }

    /// Translate audio to English text
    pub async fn translate(&self, request: TranslationRequest) -> Result<TranslationResponse> {
        info!(
            "Translating audio: model={}, file_size={}",
            request.model,
            request.file.len()
        );

        // Validate file size (max 25MB)
        if request.file.len() > 25 * 1024 * 1024 {
            return Err(GatewayError::validation("Audio file too large (max 25MB)"));
        }

        // For translation, we use transcription with target language = English
        // Most providers use the same endpoint with different parameters
        let (provider_name, actual_model) = parse_model_string(&request.model);

        let providers = self.provider_registry.all();
        let provider = providers
            .iter()
            .find(|p| p.name() == provider_name)
            .ok_or_else(|| {
                GatewayError::internal(format!(
                    "No provider found for audio translation: {}",
                    provider_name
                ))
            })?;

        match provider {
            Provider::Groq(groq) => {
                // Groq uses the same endpoint, we set language to "en" for translation
                let response = groq
                    .transcribe_audio(
                        request.file,
                        Some(actual_model.to_string()),
                        Some("en".to_string()), // Force English output
                        request.response_format,
                    )
                    .await
                    .map_err(|e| {
                        GatewayError::internal(format!("Groq translation error: {}", e))
                    })?;

                Ok(TranslationResponse {
                    text: response.text,
                    task: Some("translate".to_string()),
                    language: response.language,
                    duration: response.duration.map(|d| d as f64),
                    segments: response.segments.map(|segs| {
                        segs.into_iter()
                            .map(|s| SegmentInfo {
                                id: s.id,
                                start: s.start as f64,
                                end: s.end as f64,
                                text: s.text,
                            })
                            .collect()
                    }),
                })
            }
            _ => Err(GatewayError::internal(format!(
                "Provider {} does not support audio translation",
                provider.name()
            ))),
        }
    }
}
