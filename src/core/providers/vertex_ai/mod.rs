//! Google Vertex AI Provider Implementation
//!
//! Comprehensive support for Google Vertex AI including:
//! - Gemini models (Pro, Flash, Ultra)
//! - Partner models (Anthropic, AI21, Meta Llama)
//! - Model Garden
//! - Multimodal embeddings
//! - Image generation
//! - Text-to-speech
//! - Context caching
//! - Batch operations

pub mod auth;
pub mod batches;
pub mod client;
pub mod common_utils;
pub mod context_caching;
pub mod count_tokens;
pub mod embeddings;
pub mod error;
pub mod files;
pub mod fine_tuning;
pub mod gemini;
pub mod gemini_embeddings;
pub mod google_genai;
pub mod image_generation;
pub mod models;
pub mod multimodal_embeddings;
pub mod partner_models;
pub mod text_to_speech;
pub mod transformers;
pub mod vector_stores;
pub mod vertex_ai_partner_models;
pub mod vertex_embeddings;
pub mod vertex_model_garden;

pub use auth::{VertexAuth, VertexCredentials};
pub use client::VertexAIProvider;
pub use common_utils::VertexAIConfig;
pub use error::VertexAIError;

/// Main VertexAI Provider Configuration
#[derive(Debug, Clone)]
pub struct VertexAIProviderConfig {
    /// Google Cloud Project ID
    pub project_id: String,

    /// Vertex AI region (e.g., "us-central1")
    pub location: String,

    /// API version to use ("v1" or "v1beta1")
    pub api_version: String,

    /// Authentication credentials
    pub credentials: VertexCredentials,

    /// Custom API endpoint (optional)
    pub api_base: Option<String>,

    /// Request timeout in seconds
    pub timeout_seconds: u64,

    /// Maximum retry attempts
    pub max_retries: u32,

    /// Enable experimental features
    pub enable_experimental: bool,
}

impl Default for VertexAIProviderConfig {
    fn default() -> Self {
        Self {
            project_id: String::new(),
            location: "us-central1".to_string(),
            api_version: "v1".to_string(),
            credentials: VertexCredentials::ApplicationDefault,
            api_base: None,
            timeout_seconds: 60,
            max_retries: 3,
            enable_experimental: false,
        }
    }
}

impl crate::core::traits::provider::ProviderConfig for VertexAIProviderConfig {
    fn validate(&self) -> Result<(), String> {
        if self.project_id.is_empty() {
            return Err("Project ID is required".to_string());
        }
        if self.location.is_empty() {
            return Err("Location is required".to_string());
        }
        Ok(())
    }

    fn api_key(&self) -> Option<&str> {
        None // Vertex AI uses credentials, not API keys
    }

    fn api_base(&self) -> Option<&str> {
        self.api_base
            .as_deref()
            .or(Some("https://aiplatform.googleapis.com"))
    }

    fn timeout(&self) -> std::time::Duration {
        std::time::Duration::from_secs(self.timeout_seconds)
    }

    fn max_retries(&self) -> u32 {
        self.max_retries
    }
}

/// Supported Vertex AI models
#[derive(Debug, Clone)]
pub enum VertexAIModel {
    // Gemini 2.5 models (2025 - Latest)
    Gemini25Pro,       // gemini-2.5-pro
    Gemini25Flash,     // gemini-2.5-flash
    Gemini25FlashLite, // gemini-2.5-flash-lite-preview

    // Gemini 2.0 models (2024-2025)
    Gemini20Flash,         // gemini-2.0-flash
    Gemini20FlashExp,      // gemini-2.0-flash-exp
    Gemini20FlashThinking, // gemini-2.0-flash-thinking-exp
    Gemini20FlashLite,     // gemini-2.0-flash-lite

    // Gemini 1.5 models (2024)
    GeminiPro,       // gemini-1.5-pro
    GeminiProVision, // gemini-1.5-pro-vision (legacy)
    GeminiFlash,     // gemini-1.5-flash
    GeminiFlash8B,   // gemini-1.5-flash-8b

    // Gemini 1.0 models (legacy)
    GeminiUltra, // gemini-ultra (deprecated)

    // Partner models - Claude
    ClaudeOpus46,   // claude-opus-4-6@20260114
    ClaudeOpus45,   // claude-opus-4-5@20251110
    ClaudeSonnet45, // claude-sonnet-4-5@20250929
    ClaudeSonnet4,  // claude-sonnet-4@20250514
    Claude3Opus,
    Claude3Sonnet,
    Claude3Haiku,
    Claude35Sonnet, // claude-3-5-sonnet@20241022

    // Meta models - Llama 3.x and 4
    Llama3_70B,
    Llama3_8B,
    Llama31_405B,   // llama-3.1-405b
    Llama31_70B,    // llama-3.1-70b
    Llama32_90B,    // llama-3.2-90b
    Llama4Scout,    // llama-4-scout (2025)
    Llama4Maverick, // llama-4-maverick (2025)

    // AI21 models
    Jamba15Large,
    Jamba15Mini,
    Jamba2, // jamba-2 (2025)

    // Mistral models
    MistralLarge, // mistral-large
    MistralNemo,  // mistral-nemo
    Codestral,    // codestral

    // Custom model
    Custom(String),
}

impl VertexAIModel {
    /// Get the model ID string for API calls
    pub fn model_id(&self) -> String {
        match self {
            // Gemini 2.5 models
            Self::Gemini25Pro => "gemini-2.5-pro-preview-05-06".to_string(),
            Self::Gemini25Flash => "gemini-2.5-flash-preview-05-20".to_string(),
            Self::Gemini25FlashLite => "gemini-2.5-flash-lite-preview-06-17".to_string(),

            // Gemini 2.0 models
            Self::Gemini20Flash => "gemini-2.0-flash".to_string(),
            Self::Gemini20FlashExp => "gemini-2.0-flash-exp".to_string(),
            Self::Gemini20FlashThinking => "gemini-2.0-flash-thinking-exp-1219".to_string(),
            Self::Gemini20FlashLite => "gemini-2.0-flash-lite".to_string(),

            // Gemini 1.5 models
            Self::GeminiPro => "gemini-1.5-pro-002".to_string(),
            Self::GeminiProVision => "gemini-1.5-pro-vision".to_string(),
            Self::GeminiFlash => "gemini-1.5-flash-002".to_string(),
            Self::GeminiFlash8B => "gemini-1.5-flash-8b".to_string(),

            // Legacy
            Self::GeminiUltra => "gemini-ultra".to_string(),

            // Claude models
            Self::ClaudeOpus46 => "claude-opus-4-6@20260114".to_string(),
            Self::ClaudeOpus45 => "claude-opus-4-5@20251110".to_string(),
            Self::ClaudeSonnet45 => "claude-sonnet-4-5@20250929".to_string(),
            Self::ClaudeSonnet4 => "claude-sonnet-4@20250514".to_string(),
            Self::Claude3Opus => "claude-3-opus@20240229".to_string(),
            Self::Claude3Sonnet => "claude-3-sonnet@20240229".to_string(),
            Self::Claude3Haiku => "claude-3-haiku@20240307".to_string(),
            Self::Claude35Sonnet => "claude-3-5-sonnet@20241022".to_string(),

            // Meta Llama models
            Self::Llama3_70B => "meta/llama3-70b-instruct-maas".to_string(),
            Self::Llama3_8B => "meta/llama3-8b-instruct-maas".to_string(),
            Self::Llama31_405B => "meta/llama-3.1-405b-instruct-maas".to_string(),
            Self::Llama31_70B => "meta/llama-3.1-70b-instruct-maas".to_string(),
            Self::Llama32_90B => "meta/llama-3.2-90b-vision-instruct-maas".to_string(),
            Self::Llama4Scout => "meta/llama-4-scout-17b-16e-instruct".to_string(),
            Self::Llama4Maverick => "meta/llama-4-maverick-17b-128e-instruct".to_string(),

            // AI21 models
            Self::Jamba15Large => "ai21/jamba-1.5-large".to_string(),
            Self::Jamba15Mini => "ai21/jamba-1.5-mini".to_string(),
            Self::Jamba2 => "ai21/jamba-2-instruct".to_string(),

            // Mistral models
            Self::MistralLarge => "mistral/mistral-large-2411".to_string(),
            Self::MistralNemo => "mistral/mistral-nemo".to_string(),
            Self::Codestral => "mistral/codestral-2501".to_string(),

            Self::Custom(id) => id.clone(),
        }
    }

    /// Check if this is a Gemini model
    pub fn is_gemini(&self) -> bool {
        matches!(
            self,
            Self::Gemini25Pro
                | Self::Gemini25Flash
                | Self::Gemini25FlashLite
                | Self::Gemini20Flash
                | Self::Gemini20FlashExp
                | Self::Gemini20FlashThinking
                | Self::Gemini20FlashLite
                | Self::GeminiPro
                | Self::GeminiProVision
                | Self::GeminiFlash
                | Self::GeminiFlash8B
                | Self::GeminiUltra
        )
    }

    /// Check if this is a partner model
    pub fn is_partner_model(&self) -> bool {
        matches!(
            self,
            Self::ClaudeOpus46
                | Self::ClaudeOpus45
                | Self::ClaudeSonnet45
                | Self::ClaudeSonnet4
                | Self::Claude3Opus
                | Self::Claude3Sonnet
                | Self::Claude3Haiku
                | Self::Claude35Sonnet
                | Self::Llama3_70B
                | Self::Llama3_8B
                | Self::Llama31_405B
                | Self::Llama31_70B
                | Self::Llama32_90B
                | Self::Llama4Scout
                | Self::Llama4Maverick
                | Self::Jamba15Large
                | Self::Jamba15Mini
                | Self::Jamba2
                | Self::MistralLarge
                | Self::MistralNemo
                | Self::Codestral
        )
    }

    /// Check if model supports vision/multimodal
    pub fn supports_vision(&self) -> bool {
        matches!(
            self,
            Self::Gemini25Pro
                | Self::Gemini25Flash
                | Self::Gemini25FlashLite
                | Self::Gemini20Flash
                | Self::Gemini20FlashExp
                | Self::Gemini20FlashThinking
                | Self::Gemini20FlashLite
                | Self::GeminiPro
                | Self::GeminiProVision
                | Self::GeminiFlash
                | Self::GeminiFlash8B
                | Self::ClaudeOpus46
                | Self::ClaudeOpus45
                | Self::ClaudeSonnet45
                | Self::ClaudeSonnet4
                | Self::Claude3Opus
                | Self::Claude3Sonnet
                | Self::Claude3Haiku
                | Self::Claude35Sonnet
                | Self::Llama32_90B
                | Self::Llama4Scout
                | Self::Llama4Maverick
        )
    }

    /// Check if model supports system messages
    pub fn supports_system_messages(&self) -> bool {
        !matches!(self, Self::Custom(_))
    }

    /// Check if model supports response schema/JSON mode
    pub fn supports_response_schema(&self) -> bool {
        self.is_gemini()
    }

    /// Check if model supports function calling
    pub fn supports_function_calling(&self) -> bool {
        self.is_gemini()
            || matches!(
                self,
                Self::ClaudeOpus46
                    | Self::ClaudeOpus45
                    | Self::ClaudeSonnet45
                    | Self::ClaudeSonnet4
                    | Self::Claude3Opus
                    | Self::Claude3Sonnet
                    | Self::Claude3Haiku
                    | Self::Claude35Sonnet
                    | Self::MistralLarge
            )
    }

    /// Check if model supports thinking/reasoning mode
    pub fn supports_thinking_mode(&self) -> bool {
        matches!(
            self,
            Self::Gemini25Pro | Self::Gemini25Flash | Self::Gemini20FlashThinking
        )
    }

    /// Get maximum context window
    pub fn max_context_tokens(&self) -> usize {
        match self {
            // Gemini 2.5 models - 1M+ context
            Self::Gemini25Pro => 1_048_576,       // 1M tokens
            Self::Gemini25Flash => 1_048_576,     // 1M tokens
            Self::Gemini25FlashLite => 1_048_576, // 1M tokens

            // Gemini 2.0 models
            Self::Gemini20Flash => 1_048_576,
            Self::Gemini20FlashExp => 1_048_576,
            Self::Gemini20FlashThinking => 1_048_576,
            Self::Gemini20FlashLite => 1_048_576,

            // Gemini 1.5 models
            Self::GeminiPro => 2_097_152, // 2M tokens
            Self::GeminiProVision => 2_097_152,
            Self::GeminiFlash => 1_048_576, // 1M tokens
            Self::GeminiFlash8B => 1_048_576,

            // Legacy
            Self::GeminiUltra => 1_048_576,

            // Claude models
            Self::ClaudeOpus46 => 1_000_000,
            Self::ClaudeOpus45 => 200_000,
            Self::ClaudeSonnet45 => 200_000,
            Self::ClaudeSonnet4 => 200_000,
            Self::Claude3Opus => 200_000,
            Self::Claude3Sonnet => 200_000,
            Self::Claude3Haiku => 200_000,
            Self::Claude35Sonnet => 200_000,

            // Llama models
            Self::Llama3_70B => 32_768,
            Self::Llama3_8B => 8_192,
            Self::Llama31_405B => 128_000,
            Self::Llama31_70B => 128_000,
            Self::Llama32_90B => 128_000,
            Self::Llama4Scout => 10_000_000, // 10M context (claimed)
            Self::Llama4Maverick => 1_000_000, // 1M context

            // AI21 models
            Self::Jamba15Large => 256_000,
            Self::Jamba15Mini => 256_000,
            Self::Jamba2 => 256_000,

            // Mistral models
            Self::MistralLarge => 128_000,
            Self::MistralNemo => 128_000,
            Self::Codestral => 256_000,

            Self::Custom(_) => 32_768, // Default
        }
    }
}

/// Parse model string to VertexAIModel enum
pub fn parse_vertex_model(model: &str) -> VertexAIModel {
    let model_lower = model.to_lowercase();

    // Gemini 2.5 models (newest first)
    if model_lower.contains("gemini-2.5-pro") {
        return VertexAIModel::Gemini25Pro;
    }
    if model_lower.contains("gemini-2.5-flash-lite") {
        return VertexAIModel::Gemini25FlashLite;
    }
    if model_lower.contains("gemini-2.5-flash") {
        return VertexAIModel::Gemini25Flash;
    }

    // Gemini 2.0 models
    if model_lower.contains("gemini-2.0-flash-thinking") {
        return VertexAIModel::Gemini20FlashThinking;
    }
    if model_lower.contains("gemini-2.0-flash-lite") {
        return VertexAIModel::Gemini20FlashLite;
    }
    if model_lower.contains("gemini-2.0-flash-exp") {
        return VertexAIModel::Gemini20FlashExp;
    }
    if model_lower.contains("gemini-2.0-flash") {
        return VertexAIModel::Gemini20Flash;
    }

    // Gemini 1.5 models
    if model_lower.contains("gemini-1.5-flash-8b") {
        return VertexAIModel::GeminiFlash8B;
    }
    if model_lower.contains("gemini-1.5-pro-vision") || model_lower.contains("gemini-pro-vision") {
        return VertexAIModel::GeminiProVision;
    }
    if model_lower.contains("gemini-1.5-pro") || model == "gemini-pro" {
        return VertexAIModel::GeminiPro;
    }
    if model_lower.contains("gemini-1.5-flash") || model == "gemini-flash" {
        return VertexAIModel::GeminiFlash;
    }

    // Legacy Gemini
    if model_lower.contains("gemini-ultra") {
        return VertexAIModel::GeminiUltra;
    }

    // Claude models (check more specific first)
    if model_lower.contains("claude-opus-4-6") || model_lower.contains("claude-opus-4.6") {
        return VertexAIModel::ClaudeOpus46;
    }
    if model_lower.contains("claude-opus-4-5") || model_lower.contains("claude-opus-4.5") {
        return VertexAIModel::ClaudeOpus45;
    }
    if model_lower.contains("claude-sonnet-4-5") || model_lower.contains("claude-sonnet-4.5") {
        return VertexAIModel::ClaudeSonnet45;
    }
    if model_lower.contains("claude-sonnet-4") && !model_lower.contains("claude-sonnet-4-5") {
        return VertexAIModel::ClaudeSonnet4;
    }
    if model_lower.contains("claude-3-5-sonnet") || model_lower.contains("claude-3.5-sonnet") {
        return VertexAIModel::Claude35Sonnet;
    }
    if model_lower.contains("claude-3-opus") {
        return VertexAIModel::Claude3Opus;
    }
    if model_lower.contains("claude-3-sonnet") {
        return VertexAIModel::Claude3Sonnet;
    }
    if model_lower.contains("claude-3-haiku") {
        return VertexAIModel::Claude3Haiku;
    }

    // Llama 4 models (newest first)
    if model_lower.contains("llama-4-scout") || model_lower.contains("llama4-scout") {
        return VertexAIModel::Llama4Scout;
    }
    if model_lower.contains("llama-4-maverick") || model_lower.contains("llama4-maverick") {
        return VertexAIModel::Llama4Maverick;
    }

    // Llama 3.x models
    if model_lower.contains("llama-3.2-90b") || model_lower.contains("llama3.2-90b") {
        return VertexAIModel::Llama32_90B;
    }
    if model_lower.contains("llama-3.1-405b") || model_lower.contains("llama3.1-405b") {
        return VertexAIModel::Llama31_405B;
    }
    if model_lower.contains("llama-3.1-70b") || model_lower.contains("llama3.1-70b") {
        return VertexAIModel::Llama31_70B;
    }
    if model_lower.contains("llama3-70b") || model_lower.contains("llama-3-70b") {
        return VertexAIModel::Llama3_70B;
    }
    if model_lower.contains("llama3-8b") || model_lower.contains("llama-3-8b") {
        return VertexAIModel::Llama3_8B;
    }

    // AI21 Jamba models
    if model_lower.contains("jamba-2") {
        return VertexAIModel::Jamba2;
    }
    if model_lower.contains("jamba-1.5-large") {
        return VertexAIModel::Jamba15Large;
    }
    if model_lower.contains("jamba-1.5-mini") {
        return VertexAIModel::Jamba15Mini;
    }

    // Mistral models
    if model_lower.contains("codestral") {
        return VertexAIModel::Codestral;
    }
    if model_lower.contains("mistral-large") {
        return VertexAIModel::MistralLarge;
    }
    if model_lower.contains("mistral-nemo") {
        return VertexAIModel::MistralNemo;
    }

    VertexAIModel::Custom(model.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::traits::provider::ProviderConfig;

    // ==================== VertexAIProviderConfig Tests ====================

    #[test]
    fn test_vertex_ai_provider_config_default() {
        let config = VertexAIProviderConfig::default();
        assert!(config.project_id.is_empty());
        assert_eq!(config.location, "us-central1");
        assert_eq!(config.api_version, "v1");
        assert_eq!(config.timeout_seconds, 60);
        assert_eq!(config.max_retries, 3);
        assert!(!config.enable_experimental);
    }

    #[test]
    fn test_vertex_ai_provider_config_validate_empty_project() {
        let config = VertexAIProviderConfig::default();
        let result = config.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Project ID"));
    }

    #[test]
    fn test_vertex_ai_provider_config_validate_empty_location() {
        let config = VertexAIProviderConfig {
            project_id: "my-project".to_string(),
            location: "".to_string(),
            ..Default::default()
        };
        let result = config.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Location"));
    }

    #[test]
    fn test_vertex_ai_provider_config_validate_success() {
        let config = VertexAIProviderConfig {
            project_id: "my-project".to_string(),
            location: "us-central1".to_string(),
            ..Default::default()
        };
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_vertex_ai_provider_config_api_key() {
        let config = VertexAIProviderConfig::default();
        assert!(config.api_key().is_none());
    }

    #[test]
    fn test_vertex_ai_provider_config_api_base_default() {
        let config = VertexAIProviderConfig::default();
        assert_eq!(config.api_base(), Some("https://aiplatform.googleapis.com"));
    }

    #[test]
    fn test_vertex_ai_provider_config_api_base_custom() {
        let config = VertexAIProviderConfig {
            api_base: Some("https://custom.endpoint.com".to_string()),
            ..Default::default()
        };
        assert_eq!(config.api_base(), Some("https://custom.endpoint.com"));
    }

    #[test]
    fn test_vertex_ai_provider_config_timeout() {
        let config = VertexAIProviderConfig {
            timeout_seconds: 120,
            ..Default::default()
        };
        assert_eq!(config.timeout(), std::time::Duration::from_secs(120));
    }

    #[test]
    fn test_vertex_ai_provider_config_max_retries() {
        let config = VertexAIProviderConfig {
            max_retries: 5,
            ..Default::default()
        };
        assert_eq!(config.max_retries(), 5);
    }

    // ==================== VertexAIModel Tests ====================

    #[test]
    fn test_vertex_ai_model_gemini_ids() {
        assert_eq!(
            VertexAIModel::Gemini25Pro.model_id(),
            "gemini-2.5-pro-preview-05-06"
        );
        assert_eq!(
            VertexAIModel::Gemini25Flash.model_id(),
            "gemini-2.5-flash-preview-05-20"
        );
        assert_eq!(VertexAIModel::Gemini20Flash.model_id(), "gemini-2.0-flash");
        assert_eq!(VertexAIModel::GeminiPro.model_id(), "gemini-1.5-pro-002");
        assert_eq!(
            VertexAIModel::GeminiFlash.model_id(),
            "gemini-1.5-flash-002"
        );
    }

    #[test]
    fn test_vertex_ai_model_claude_ids() {
        assert_eq!(
            VertexAIModel::ClaudeOpus46.model_id(),
            "claude-opus-4-6@20260114"
        );
        assert_eq!(
            VertexAIModel::ClaudeOpus45.model_id(),
            "claude-opus-4-5@20251110"
        );
        assert_eq!(
            VertexAIModel::ClaudeSonnet45.model_id(),
            "claude-sonnet-4-5@20250929"
        );
        assert_eq!(
            VertexAIModel::ClaudeSonnet4.model_id(),
            "claude-sonnet-4@20250514"
        );
        assert_eq!(
            VertexAIModel::Claude3Opus.model_id(),
            "claude-3-opus@20240229"
        );
        assert_eq!(
            VertexAIModel::Claude3Sonnet.model_id(),
            "claude-3-sonnet@20240229"
        );
        assert_eq!(
            VertexAIModel::Claude3Haiku.model_id(),
            "claude-3-haiku@20240307"
        );
        assert_eq!(
            VertexAIModel::Claude35Sonnet.model_id(),
            "claude-3-5-sonnet@20241022"
        );
    }

    #[test]
    fn test_vertex_ai_model_llama_ids() {
        assert_eq!(
            VertexAIModel::Llama3_70B.model_id(),
            "meta/llama3-70b-instruct-maas"
        );
        assert_eq!(
            VertexAIModel::Llama31_405B.model_id(),
            "meta/llama-3.1-405b-instruct-maas"
        );
        assert_eq!(
            VertexAIModel::Llama4Scout.model_id(),
            "meta/llama-4-scout-17b-16e-instruct"
        );
    }

    #[test]
    fn test_vertex_ai_model_custom() {
        let model = VertexAIModel::Custom("my-custom-model".to_string());
        assert_eq!(model.model_id(), "my-custom-model");
    }

    #[test]
    fn test_vertex_ai_model_is_gemini() {
        assert!(VertexAIModel::Gemini25Pro.is_gemini());
        assert!(VertexAIModel::Gemini20Flash.is_gemini());
        assert!(VertexAIModel::GeminiPro.is_gemini());
        assert!(VertexAIModel::GeminiFlash.is_gemini());
        assert!(VertexAIModel::GeminiUltra.is_gemini());

        assert!(!VertexAIModel::Claude3Opus.is_gemini());
        assert!(!VertexAIModel::Llama3_70B.is_gemini());
    }

    #[test]
    fn test_vertex_ai_model_is_partner_model() {
        assert!(VertexAIModel::ClaudeOpus46.is_partner_model());
        assert!(VertexAIModel::ClaudeSonnet45.is_partner_model());
        assert!(VertexAIModel::Claude3Opus.is_partner_model());
        assert!(VertexAIModel::Claude35Sonnet.is_partner_model());
        assert!(VertexAIModel::Llama3_70B.is_partner_model());
        assert!(VertexAIModel::Llama4Scout.is_partner_model());
        assert!(VertexAIModel::Jamba15Large.is_partner_model());
        assert!(VertexAIModel::MistralLarge.is_partner_model());

        assert!(!VertexAIModel::GeminiPro.is_partner_model());
        assert!(!VertexAIModel::Gemini20Flash.is_partner_model());
    }

    #[test]
    fn test_vertex_ai_model_supports_vision() {
        assert!(VertexAIModel::Gemini25Pro.supports_vision());
        assert!(VertexAIModel::Gemini20Flash.supports_vision());
        assert!(VertexAIModel::GeminiPro.supports_vision());
        assert!(VertexAIModel::GeminiProVision.supports_vision());
        assert!(VertexAIModel::Llama32_90B.supports_vision());
        assert!(VertexAIModel::Llama4Scout.supports_vision());

        assert!(VertexAIModel::ClaudeOpus46.supports_vision());
        assert!(VertexAIModel::Claude3Opus.supports_vision());
        assert!(!VertexAIModel::Llama3_70B.supports_vision());
    }

    #[test]
    fn test_vertex_ai_model_supports_system_messages() {
        assert!(VertexAIModel::GeminiPro.supports_system_messages());
        assert!(VertexAIModel::Claude3Opus.supports_system_messages());
        assert!(VertexAIModel::Llama3_70B.supports_system_messages());

        assert!(!VertexAIModel::Custom("custom".to_string()).supports_system_messages());
    }

    #[test]
    fn test_vertex_ai_model_supports_response_schema() {
        assert!(VertexAIModel::GeminiPro.supports_response_schema());
        assert!(VertexAIModel::Gemini20Flash.supports_response_schema());

        assert!(!VertexAIModel::Claude3Opus.supports_response_schema());
        assert!(!VertexAIModel::Llama3_70B.supports_response_schema());
    }

    #[test]
    fn test_vertex_ai_model_supports_function_calling() {
        assert!(VertexAIModel::GeminiPro.supports_function_calling());
        assert!(VertexAIModel::Claude3Opus.supports_function_calling());
        assert!(VertexAIModel::Claude35Sonnet.supports_function_calling());
        assert!(VertexAIModel::MistralLarge.supports_function_calling());

        assert!(!VertexAIModel::Llama3_70B.supports_function_calling());
        assert!(!VertexAIModel::Jamba15Large.supports_function_calling());
    }

    #[test]
    fn test_vertex_ai_model_supports_thinking_mode() {
        assert!(VertexAIModel::Gemini25Pro.supports_thinking_mode());
        assert!(VertexAIModel::Gemini25Flash.supports_thinking_mode());
        assert!(VertexAIModel::Gemini20FlashThinking.supports_thinking_mode());

        assert!(!VertexAIModel::GeminiPro.supports_thinking_mode());
        assert!(!VertexAIModel::Claude3Opus.supports_thinking_mode());
    }

    #[test]
    fn test_vertex_ai_model_max_context_tokens() {
        // Gemini 2.5
        assert_eq!(VertexAIModel::Gemini25Pro.max_context_tokens(), 1_048_576);

        // Gemini 1.5 Pro has largest
        assert_eq!(VertexAIModel::GeminiPro.max_context_tokens(), 2_097_152);

        assert_eq!(VertexAIModel::ClaudeOpus46.max_context_tokens(), 1_000_000);

        // Claude
        assert_eq!(VertexAIModel::Claude3Opus.max_context_tokens(), 200_000);

        // Llama 4 Scout
        assert_eq!(VertexAIModel::Llama4Scout.max_context_tokens(), 10_000_000);

        // Jamba
        assert_eq!(VertexAIModel::Jamba15Large.max_context_tokens(), 256_000);

        // Custom
        assert_eq!(
            VertexAIModel::Custom("custom".to_string()).max_context_tokens(),
            32_768
        );
    }

    // ==================== parse_vertex_model Tests ====================

    #[test]
    fn test_parse_vertex_model_gemini_25() {
        assert!(matches!(
            parse_vertex_model("gemini-2.5-pro"),
            VertexAIModel::Gemini25Pro
        ));
        assert!(matches!(
            parse_vertex_model("gemini-2.5-flash"),
            VertexAIModel::Gemini25Flash
        ));
        assert!(matches!(
            parse_vertex_model("gemini-2.5-flash-lite"),
            VertexAIModel::Gemini25FlashLite
        ));
    }

    #[test]
    fn test_parse_vertex_model_gemini_20() {
        assert!(matches!(
            parse_vertex_model("gemini-2.0-flash"),
            VertexAIModel::Gemini20Flash
        ));
        assert!(matches!(
            parse_vertex_model("gemini-2.0-flash-exp"),
            VertexAIModel::Gemini20FlashExp
        ));
        assert!(matches!(
            parse_vertex_model("gemini-2.0-flash-thinking"),
            VertexAIModel::Gemini20FlashThinking
        ));
        assert!(matches!(
            parse_vertex_model("gemini-2.0-flash-lite"),
            VertexAIModel::Gemini20FlashLite
        ));
    }

    #[test]
    fn test_parse_vertex_model_gemini_15() {
        assert!(matches!(
            parse_vertex_model("gemini-1.5-pro"),
            VertexAIModel::GeminiPro
        ));
        assert!(matches!(
            parse_vertex_model("gemini-pro"),
            VertexAIModel::GeminiPro
        ));
        assert!(matches!(
            parse_vertex_model("gemini-1.5-flash"),
            VertexAIModel::GeminiFlash
        ));
        assert!(matches!(
            parse_vertex_model("gemini-flash"),
            VertexAIModel::GeminiFlash
        ));
        assert!(matches!(
            parse_vertex_model("gemini-1.5-flash-8b"),
            VertexAIModel::GeminiFlash8B
        ));
    }

    #[test]
    fn test_parse_vertex_model_claude() {
        assert!(matches!(
            parse_vertex_model("claude-opus-4-6"),
            VertexAIModel::ClaudeOpus46
        ));
        assert!(matches!(
            parse_vertex_model("claude-opus-4.6"),
            VertexAIModel::ClaudeOpus46
        ));
        assert!(matches!(
            parse_vertex_model("claude-sonnet-4-5"),
            VertexAIModel::ClaudeSonnet45
        ));
        assert!(matches!(
            parse_vertex_model("claude-sonnet-4"),
            VertexAIModel::ClaudeSonnet4
        ));
        assert!(matches!(
            parse_vertex_model("claude-3-opus"),
            VertexAIModel::Claude3Opus
        ));
        assert!(matches!(
            parse_vertex_model("claude-3-sonnet"),
            VertexAIModel::Claude3Sonnet
        ));
        assert!(matches!(
            parse_vertex_model("claude-3-haiku"),
            VertexAIModel::Claude3Haiku
        ));
        assert!(matches!(
            parse_vertex_model("claude-3-5-sonnet"),
            VertexAIModel::Claude35Sonnet
        ));
        assert!(matches!(
            parse_vertex_model("claude-3.5-sonnet"),
            VertexAIModel::Claude35Sonnet
        ));
    }

    #[test]
    fn test_parse_vertex_model_llama() {
        assert!(matches!(
            parse_vertex_model("llama3-70b"),
            VertexAIModel::Llama3_70B
        ));
        assert!(matches!(
            parse_vertex_model("llama-3-70b"),
            VertexAIModel::Llama3_70B
        ));
        assert!(matches!(
            parse_vertex_model("llama-3.1-405b"),
            VertexAIModel::Llama31_405B
        ));
        assert!(matches!(
            parse_vertex_model("llama-3.2-90b"),
            VertexAIModel::Llama32_90B
        ));
        assert!(matches!(
            parse_vertex_model("llama-4-scout"),
            VertexAIModel::Llama4Scout
        ));
        assert!(matches!(
            parse_vertex_model("llama-4-maverick"),
            VertexAIModel::Llama4Maverick
        ));
    }

    #[test]
    fn test_parse_vertex_model_jamba() {
        assert!(matches!(
            parse_vertex_model("jamba-1.5-large"),
            VertexAIModel::Jamba15Large
        ));
        assert!(matches!(
            parse_vertex_model("jamba-1.5-mini"),
            VertexAIModel::Jamba15Mini
        ));
        assert!(matches!(
            parse_vertex_model("jamba-2"),
            VertexAIModel::Jamba2
        ));
    }

    #[test]
    fn test_parse_vertex_model_mistral() {
        assert!(matches!(
            parse_vertex_model("mistral-large"),
            VertexAIModel::MistralLarge
        ));
        assert!(matches!(
            parse_vertex_model("mistral-nemo"),
            VertexAIModel::MistralNemo
        ));
        assert!(matches!(
            parse_vertex_model("codestral"),
            VertexAIModel::Codestral
        ));
    }

    #[test]
    fn test_parse_vertex_model_custom() {
        let model = parse_vertex_model("unknown-model");
        assert!(matches!(model, VertexAIModel::Custom(_)));
        if let VertexAIModel::Custom(id) = model {
            assert_eq!(id, "unknown-model");
        }
    }

    #[test]
    fn test_parse_vertex_model_case_insensitive() {
        assert!(matches!(
            parse_vertex_model("GEMINI-2.5-PRO"),
            VertexAIModel::Gemini25Pro
        ));
        assert!(matches!(
            parse_vertex_model("Claude-3-Opus"),
            VertexAIModel::Claude3Opus
        ));
        assert!(matches!(
            parse_vertex_model("LLAMA-3.1-405B"),
            VertexAIModel::Llama31_405B
        ));
    }

    #[test]
    fn test_vertex_ai_model_clone() {
        let model = VertexAIModel::GeminiPro;
        let cloned = model.clone();
        assert_eq!(model.model_id(), cloned.model_id());
    }

    #[test]
    fn test_vertex_ai_model_debug() {
        let model = VertexAIModel::GeminiPro;
        let debug = format!("{:?}", model);
        assert!(debug.contains("GeminiPro"));
    }
}
