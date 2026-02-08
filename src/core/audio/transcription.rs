//! Audio transcription functionality

use crate::core::providers::{Provider, ProviderRegistry};
use crate::utils::error::error::{GatewayError, Result};
use std::sync::Arc;
use tracing::{debug, info};

use super::types::{SegmentInfo, TranscriptionRequest, TranscriptionResponse, WordInfo};

/// Audio service for handling audio transcription requests
pub struct TranscriptionService {
    provider_registry: Arc<ProviderRegistry>,
}

impl TranscriptionService {
    /// Create a new transcription service
    pub fn new(provider_registry: Arc<ProviderRegistry>) -> Self {
        Self { provider_registry }
    }

    /// Transcribe audio to text
    pub async fn transcribe(&self, request: TranscriptionRequest) -> Result<TranscriptionResponse> {
        info!(
            "Transcribing audio: model={}, file_size={}",
            request.model,
            request.file.len()
        );

        // Validate file size (max 25MB)
        if request.file.len() > 25 * 1024 * 1024 {
            return Err(GatewayError::validation("Audio file too large (max 25MB)"));
        }

        // Determine provider from model name
        let (provider_name, actual_model) = parse_model_string(&request.model);

        // Find provider
        let providers = self.provider_registry.all();
        let provider = providers
            .iter()
            .find(|p| p.name() == provider_name)
            .ok_or_else(|| {
                GatewayError::internal(format!(
                    "No provider found for audio transcription: {}",
                    provider_name
                ))
            })?;

        // Route to appropriate provider
        match provider {
            Provider::Groq(groq) => {
                debug!("Using Groq for transcription");
                let response = groq
                    .transcribe_audio(
                        request.file,
                        Some(actual_model.to_string()),
                        request.language,
                        request.response_format,
                    )
                    .await
                    .map_err(|e| {
                        GatewayError::internal(format!("Groq transcription error: {}", e))
                    })?;

                Ok(TranscriptionResponse {
                    text: response.text,
                    task: response.task,
                    language: response.language,
                    duration: response.duration.map(|d| d as f64),
                    words: response.words.map(|words| {
                        words
                            .into_iter()
                            .map(|w| WordInfo {
                                word: w.word,
                                start: w.start as f64,
                                end: w.end as f64,
                            })
                            .collect()
                    }),
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
            Provider::OpenAI(_openai) => {
                // OpenAI transcription - similar implementation
                Err(GatewayError::internal(
                    "OpenAI audio transcription not yet implemented",
                ))
            }
            _ => Err(GatewayError::internal(format!(
                "Provider {} does not support audio transcription",
                provider.name()
            ))),
        }
    }
}

/// Parse model string to extract provider and model name
/// Format: "provider/model" or just "model"
pub(crate) fn parse_model_string(model: &str) -> (&str, &str) {
    if let Some(idx) = model.find('/') {
        let provider = &model[..idx];
        let model_name = &model[idx + 1..];
        (provider, model_name)
    } else {
        // Default provider based on model name
        if model.starts_with("whisper") {
            ("groq", model) // Default Whisper to Groq (faster)
        } else {
            // Default to OpenAI for TTS and other models
            ("openai", model)
        }
    }
}
