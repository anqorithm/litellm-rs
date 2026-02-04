//! AI API endpoints (OpenAI compatible)
//!
//! This module provides OpenAI-compatible API endpoints for AI services.

#![allow(dead_code)]

// Module declarations
mod audio;
mod chat;
mod completions;
mod context;
mod embeddings;
mod images;
mod models;
mod provider_selection;

// Public re-exports for backward compatibility
pub use audio::{audio_speech, audio_transcriptions, audio_translations};
pub use chat::chat_completions;
pub use completions::completions;
pub use context::{
    check_permission, get_authenticated_api_key, get_authenticated_user, get_request_context,
    log_api_usage,
};
pub use embeddings::embeddings;
pub use images::image_generations;
pub use models::{get_model, list_models};

use actix_web::web;

/// Configure AI API routes
pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/v1")
            // Chat completions
            .route("/chat/completions", web::post().to(chat_completions))
            // Text completions (legacy)
            .route("/completions", web::post().to(completions))
            // Embeddings
            .route("/embeddings", web::post().to(embeddings))
            // Image generation
            .route("/images/generations", web::post().to(image_generations))
            // Models
            .route("/models", web::get().to(list_models))
            .route("/models/{model_id}", web::get().to(get_model))
            // Audio (future implementation)
            .route(
                "/audio/transcriptions",
                web::post().to(audio_transcriptions),
            )
            .route("/audio/translations", web::post().to(audio_translations))
            .route("/audio/speech", web::post().to(audio_speech)),
    );
}

#[cfg(test)]
mod tests {
    use crate::core::models::RequestContext;

    #[test]
    fn test_get_request_context() {
        // This test would need a mock HttpRequest in a real implementation
        // For now, we'll test the basic functionality
        let context = RequestContext::new();
        assert!(!context.request_id.is_empty());
        assert!(context.user_agent.is_none());
    }
}
