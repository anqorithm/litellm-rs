//! Audio API module for speech-to-text and text-to-speech
//!
//! Provides unified audio processing capabilities across providers.

mod speech;
mod tests;
mod transcription;
mod translation;

// Make types module publicly accessible
pub mod types;

use crate::core::providers::ProviderRegistry;
use crate::utils::error::error::Result;
use std::sync::Arc;

// Internal service imports
use speech::SpeechService;
use transcription::TranscriptionService;
use translation::TranslationService;

// Import types for AudioService method signatures
use types::{
    SpeechRequest, SpeechResponse, TranscriptionRequest, TranscriptionResponse, TranslationRequest,
    TranslationResponse,
};

/// Audio service for handling audio API requests
pub struct AudioService {
    transcription_service: TranscriptionService,
    translation_service: TranslationService,
    speech_service: SpeechService,
}

impl AudioService {
    /// Create a new audio service
    pub fn new(provider_registry: Arc<ProviderRegistry>) -> Self {
        Self {
            transcription_service: TranscriptionService::new(Arc::clone(&provider_registry)),
            translation_service: TranslationService::new(Arc::clone(&provider_registry)),
            speech_service: SpeechService::new(provider_registry),
        }
    }

    /// Transcribe audio to text
    pub async fn transcribe(&self, request: TranscriptionRequest) -> Result<TranscriptionResponse> {
        self.transcription_service.transcribe(request).await
    }

    /// Translate audio to English text
    pub async fn translate(&self, request: TranslationRequest) -> Result<TranslationResponse> {
        self.translation_service.translate(request).await
    }

    /// Convert text to speech
    pub async fn speech(&self, request: SpeechRequest) -> Result<SpeechResponse> {
        self.speech_service.speech(request).await
    }
}
