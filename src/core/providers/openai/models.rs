//! OpenAI Model Registry
//!
//! Dynamic model discovery and capability detection system

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::OnceLock;

use crate::core::providers::base::get_pricing_db;
use crate::core::types::{model::ModelInfo, model::ProviderCapability};

#[path = "static_models.rs"]
mod static_models;

/// OpenAI-specific model features
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum OpenAIModelFeature {
    /// Chat completion support
    ChatCompletion,
    /// Streaming response support
    StreamingSupport,
    /// Function/tool calling support
    FunctionCalling,
    /// Vision support (multimodal)
    VisionSupport,
    /// System message support
    SystemMessages,
    /// JSON mode support
    JsonMode,
    /// O-series reasoning mode
    ReasoningMode,
    /// Audio input support
    AudioInput,
    /// Audio output support (TTS)
    AudioOutput,
    /// Image generation (DALL-E)
    ImageGeneration,
    /// Image editing
    ImageEditing,
    /// Audio transcription
    AudioTranscription,
    /// Fine-tuning support
    FineTuning,
    /// Embeddings generation
    Embeddings,
    /// Code completion optimized
    CodeCompletion,
    /// High context window (>32K)
    LargeContext,
    /// Real-time audio processing
    RealtimeAudio,
}

impl OpenAIModelFeature {
    /// Convert OpenAI model feature to provider capability
    pub fn to_provider_capability(&self) -> Option<ProviderCapability> {
        match self {
            OpenAIModelFeature::ChatCompletion => Some(ProviderCapability::ChatCompletion),
            OpenAIModelFeature::StreamingSupport => Some(ProviderCapability::ChatCompletionStream),
            OpenAIModelFeature::FunctionCalling => Some(ProviderCapability::ToolCalling),
            OpenAIModelFeature::ImageGeneration => Some(ProviderCapability::ImageGeneration),
            OpenAIModelFeature::AudioTranscription => Some(ProviderCapability::AudioTranscription),
            OpenAIModelFeature::Embeddings => Some(ProviderCapability::Embeddings),
            OpenAIModelFeature::AudioOutput => Some(ProviderCapability::TextToSpeech),
            OpenAIModelFeature::ImageEditing => Some(ProviderCapability::ImageEdit),
            // Features that don't map directly to provider capabilities
            OpenAIModelFeature::SystemMessages
            | OpenAIModelFeature::JsonMode
            | OpenAIModelFeature::ReasoningMode
            | OpenAIModelFeature::VisionSupport
            | OpenAIModelFeature::AudioInput
            | OpenAIModelFeature::FineTuning
            | OpenAIModelFeature::CodeCompletion
            | OpenAIModelFeature::LargeContext
            | OpenAIModelFeature::RealtimeAudio => None,
        }
    }
}

/// OpenAI model specification
#[derive(Debug, Clone)]
pub struct OpenAIModelSpec {
    /// Basic model information
    pub model_info: ModelInfo,
    /// Supported features
    pub features: Vec<OpenAIModelFeature>,
    /// Model family (gpt-4, gpt-3.5, dalle, whisper, etc.)
    pub family: OpenAIModelFamily,
    /// Model configuration
    pub config: OpenAIModelConfig,
}

/// OpenAI model families
#[derive(Debug, Clone, PartialEq)]
pub enum OpenAIModelFamily {
    GPT4,
    GPT4Turbo,
    GPT4O,
    GPT4OMini,
    GPT41,
    GPT41Mini,
    GPT41Nano,
    GPT35,
    GPT5,          // GPT-5 models (2025)
    GPT5Mini,      // GPT-5 Mini models (2025)
    GPT5Nano,      // GPT-5 Nano models (2025)
    GPT51,         // GPT-5.1 models (Nov 2025)
    GPT51Thinking, // GPT-5.1 Thinking mode (Nov 2025)
    GPT52,         // GPT-5.2 models (2025)
    GPT52Pro,      // GPT-5.2 Pro models (2025)
    GPT52Codex,    // GPT-5.2 Codex models (2025)
    O1,            // O1 reasoning models
    O1Pro,         // O1 Pro reasoning models
    O3,            // O3 reasoning models (2025)
    O3Pro,         // O3 Pro reasoning models
    O3Mini,        // O3 Mini reasoning models
    O4Mini,        // O4 Mini reasoning models (2025)
    DALLE2,
    DALLE3,
    Whisper,
    TTS,
    Embedding,
    Moderation,
    GPT4OAudio, // GPT-4O with audio capabilities
    GPTAudio,   // GPT Audio models (2025)
    GPTImage,   // GPT image generation models
    Realtime,   // Realtime API models
}

/// Model-specific configuration
#[derive(Debug, Clone)]
pub struct OpenAIModelConfig {
    /// Maximum requests per minute
    pub max_rpm: Option<u32>,
    /// Maximum tokens per minute  
    pub max_tpm: Option<u32>,
    /// Supports batch API
    pub supports_batch: bool,
    /// Default temperature
    pub default_temperature: Option<f32>,
    /// Supports streaming
    pub supports_streaming: bool,
    /// Custom parameters
    pub custom_params: HashMap<String, serde_json::Value>,
}

impl Default for OpenAIModelConfig {
    fn default() -> Self {
        Self {
            max_rpm: None,
            max_tpm: None,
            supports_batch: false,
            default_temperature: None,
            supports_streaming: true,
            custom_params: HashMap::new(),
        }
    }
}

/// OpenAI model registry
#[derive(Debug)]
pub struct OpenAIModelRegistry {
    models: HashMap<String, OpenAIModelSpec>,
}

impl Default for OpenAIModelRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl OpenAIModelRegistry {
    /// Create new registry instance
    pub fn new() -> Self {
        let mut registry = Self {
            models: HashMap::new(),
        };
        registry.load_models();
        registry
    }

    /// Load models from pricing database and add static definitions
    fn load_models(&mut self) {
        // Always load built-in static models first so we keep a comprehensive
        // fallback catalog even when pricing DB is partially populated.
        self.add_static_models();

        let pricing_db = get_pricing_db();
        let model_ids = pricing_db.get_provider_models("openai");

        // Load from pricing database
        for model_id in &model_ids {
            if let Some(mut model_info) = pricing_db.to_model_info(model_id, "openai") {
                let features = self.detect_features(&model_info);

                // Convert features to capabilities
                model_info.capabilities = features
                    .iter()
                    .filter_map(|f| f.to_provider_capability())
                    .collect();

                let family = self.determine_family(&model_info);
                let config = self.create_config(&model_info);

                self.models.insert(
                    model_id.clone(),
                    OpenAIModelSpec {
                        model_info,
                        features,
                        family,
                        config,
                    },
                );
            }
        }
    }

    /// Detect model features based on model info
    fn detect_features(&self, model_info: &ModelInfo) -> Vec<OpenAIModelFeature> {
        let mut features = vec![
            OpenAIModelFeature::SystemMessages,
            OpenAIModelFeature::StreamingSupport,
        ];

        let model_id = &model_info.id;

        // Chat models
        if model_id.starts_with("gpt-") {
            features.push(OpenAIModelFeature::ChatCompletion);
            features.push(OpenAIModelFeature::JsonMode);
        }

        // Function calling support
        if model_info.supports_tools {
            features.push(OpenAIModelFeature::FunctionCalling);
        }

        // Vision support
        if model_info.supports_multimodal || model_id.contains("vision") {
            features.push(OpenAIModelFeature::VisionSupport);
        }

        // O-series reasoning models
        if model_id.starts_with("o1") || model_id.starts_with("o3") || model_id.starts_with("o4") {
            features.push(OpenAIModelFeature::ReasoningMode);
        }

        // GPT-4O audio features
        if model_id.contains("gpt-4o-audio") {
            features.push(OpenAIModelFeature::AudioInput);
            features.push(OpenAIModelFeature::AudioOutput);
        }

        // DALL-E and GPT image models
        if model_id.starts_with("dall-e")
            || model_id.starts_with("gpt-image-")
            || model_id.starts_with("chatgpt-image-")
        {
            features.push(OpenAIModelFeature::ImageGeneration);
            if model_id.contains("dall-e-3") {
                features.push(OpenAIModelFeature::ImageEditing);
            }
        }

        // Whisper models
        if model_id.starts_with("whisper") {
            features.push(OpenAIModelFeature::AudioTranscription);
        }

        // TTS models
        if model_id.starts_with("tts") {
            features.push(OpenAIModelFeature::AudioOutput);
        }

        // Embedding models
        if model_id.contains("embedding") {
            features.push(OpenAIModelFeature::Embeddings);
        }

        // Code-optimized models
        if model_id.contains("code") || model_id.contains("codex") {
            features.push(OpenAIModelFeature::CodeCompletion);
        }

        // Large context models
        if model_info.max_context_length > 32000 {
            features.push(OpenAIModelFeature::LargeContext);
        }

        // Fine-tuning support (selected models)
        if matches!(
            model_id.as_str(),
            "gpt-3.5-turbo" | "gpt-4" | "gpt-4-turbo" | "babbage-002" | "davinci-002"
        ) {
            features.push(OpenAIModelFeature::FineTuning);
        }

        features
    }

    /// Determine model family
    fn determine_family(&self, model_info: &ModelInfo) -> OpenAIModelFamily {
        let model_id = &model_info.id;

        // Check most specific patterns first
        if model_id.starts_with("gpt-4o-mini") {
            OpenAIModelFamily::GPT4OMini
        } else if model_id.starts_with("gpt-4.1-nano") {
            OpenAIModelFamily::GPT41Nano
        } else if model_id.starts_with("gpt-4.1-mini") {
            OpenAIModelFamily::GPT41Mini
        } else if model_id.starts_with("gpt-4.1") {
            OpenAIModelFamily::GPT41
        } else if model_id.starts_with("gpt-4o-audio") || model_id.contains("audio-preview") {
            OpenAIModelFamily::GPT4OAudio
        } else if model_id.starts_with("gpt-4o-realtime") {
            OpenAIModelFamily::Realtime
        } else if model_id.starts_with("gpt-4o") {
            OpenAIModelFamily::GPT4O
        } else if model_id.starts_with("gpt-4-turbo")
            || model_id.starts_with("gpt-4-1106")
            || model_id.starts_with("gpt-4-0125")
        {
            OpenAIModelFamily::GPT4Turbo
        } else if model_id.starts_with("gpt-4") {
            OpenAIModelFamily::GPT4
        } else if model_id.starts_with("gpt-3.5") {
            OpenAIModelFamily::GPT35
        }
        // GPT-5 series (check specific variants first, most specific first)
        else if model_id.starts_with("gpt-5.2-pro") {
            OpenAIModelFamily::GPT52Pro
        } else if model_id.starts_with("gpt-5.2-codex") || model_id.starts_with("gpt-5-codex") {
            OpenAIModelFamily::GPT52Codex
        } else if model_id.starts_with("gpt-5.2") || model_id.contains("gpt-5.2") {
            OpenAIModelFamily::GPT52
        } else if model_id.starts_with("gpt-5.1-thinking") || model_id.contains("5.1-thinking") {
            OpenAIModelFamily::GPT51Thinking
        } else if model_id.starts_with("gpt-5.1") || model_id.contains("gpt-5.1") {
            OpenAIModelFamily::GPT51
        } else if model_id.starts_with("gpt-5-nano") {
            OpenAIModelFamily::GPT5Nano
        } else if model_id.starts_with("gpt-5-mini") {
            OpenAIModelFamily::GPT5Mini
        } else if model_id.starts_with("gpt-5") {
            OpenAIModelFamily::GPT5
        }
        // GPT Audio models
        else if model_id.starts_with("gpt-audio") {
            OpenAIModelFamily::GPTAudio
        }
        // O-series reasoning models
        else if model_id.starts_with("o4-mini") {
            OpenAIModelFamily::O4Mini
        } else if model_id.starts_with("o3-pro") {
            OpenAIModelFamily::O3Pro
        } else if model_id.starts_with("o3-mini") {
            OpenAIModelFamily::O3Mini
        } else if model_id.starts_with("o3") {
            OpenAIModelFamily::O3
        } else if model_id.starts_with("o1-pro") {
            OpenAIModelFamily::O1Pro
        } else if model_id.starts_with("o1") {
            OpenAIModelFamily::O1
        } else if model_id.starts_with("gpt-image-") || model_id.starts_with("chatgpt-image-") {
            OpenAIModelFamily::GPTImage
        } else if model_id.starts_with("dall-e-2") {
            OpenAIModelFamily::DALLE2
        } else if model_id.starts_with("dall-e-3") {
            OpenAIModelFamily::DALLE3
        } else if model_id.starts_with("whisper") {
            OpenAIModelFamily::Whisper
        } else if model_id.starts_with("tts") {
            OpenAIModelFamily::TTS
        } else if model_id.contains("embedding") {
            OpenAIModelFamily::Embedding
        } else {
            OpenAIModelFamily::GPT4 // Default fallback
        }
    }

    /// Create model configuration
    fn create_config(&self, model_info: &ModelInfo) -> OpenAIModelConfig {
        let mut config = OpenAIModelConfig::default();
        let model_id = &model_info.id;

        // Set rate limits based on model
        match model_id.as_str() {
            m if m.starts_with("gpt-5") => {
                config.max_rpm = Some(6000);
                config.max_tpm = Some(400000);
            }
            m if m.starts_with("gpt-4") => {
                config.max_rpm = Some(10000);
                config.max_tpm = Some(300000);
            }
            m if m.starts_with("gpt-3.5") => {
                config.max_rpm = Some(10000);
                config.max_tpm = Some(1000000);
            }
            m if m.starts_with("o1") || m.starts_with("o3") || m.starts_with("o4") => {
                config.max_rpm = Some(5000);
                config.max_tpm = Some(100000);
                config.default_temperature = Some(1.0);
            }
            _ => {
                config.max_rpm = Some(5000);
                config.max_tpm = Some(200000);
            }
        }

        // Batch API support
        config.supports_batch = matches!(
            model_id.as_str(),
            "gpt-4"
                | "gpt-4-turbo"
                | "gpt-3.5-turbo"
                | "text-embedding-ada-002"
                | "text-embedding-3-small"
                | "text-embedding-3-large"
        );

        // Streaming support
        config.supports_streaming =
            !model_id.contains("embedding") && !model_id.contains("whisper");

        config
    }

    /// Add static model definitions as fallback
    fn add_static_models(&mut self) {
        for (id, name, family, max_context, max_output, input_cost, output_cost) in
            static_models::definitions()
        {
            let mut model_info = ModelInfo {
                id: id.to_string(),
                name: name.to_string(),
                provider: "openai".to_string(),
                max_context_length: max_context,
                max_output_length: max_output,
                supports_streaming: family != OpenAIModelFamily::Embedding
                    && family != OpenAIModelFamily::Whisper,
                supports_tools: matches!(
                    family,
                    OpenAIModelFamily::GPT4
                        | OpenAIModelFamily::GPT4Turbo
                        | OpenAIModelFamily::GPT4O
                        | OpenAIModelFamily::GPT4OMini
                        | OpenAIModelFamily::GPT35
                        | OpenAIModelFamily::GPT5
                        | OpenAIModelFamily::GPT5Mini
                        | OpenAIModelFamily::GPT5Nano
                        | OpenAIModelFamily::GPT51
                        | OpenAIModelFamily::GPT51Thinking
                        | OpenAIModelFamily::GPT52
                        | OpenAIModelFamily::GPT52Pro
                        | OpenAIModelFamily::GPT52Codex
                        | OpenAIModelFamily::O1
                        | OpenAIModelFamily::O1Pro
                        | OpenAIModelFamily::O3
                        | OpenAIModelFamily::O3Mini
                        | OpenAIModelFamily::O4Mini
                        | OpenAIModelFamily::GPT4OAudio
                        | OpenAIModelFamily::GPTAudio
                ),
                supports_multimodal: matches!(
                    family,
                    OpenAIModelFamily::GPT4O
                        | OpenAIModelFamily::GPT4OMini
                        | OpenAIModelFamily::GPT4OAudio
                        | OpenAIModelFamily::GPT5
                        | OpenAIModelFamily::GPT5Mini
                        | OpenAIModelFamily::GPT51
                        | OpenAIModelFamily::GPT51Thinking
                        | OpenAIModelFamily::GPT52
                        | OpenAIModelFamily::GPT52Pro
                        | OpenAIModelFamily::GPT52Codex
                        | OpenAIModelFamily::GPTAudio
                        | OpenAIModelFamily::O1
                        | OpenAIModelFamily::O1Pro
                        | OpenAIModelFamily::O3
                        | OpenAIModelFamily::O3Mini
                        | OpenAIModelFamily::O4Mini
                ) || id.contains("vision"),
                input_cost_per_1k_tokens: Some(input_cost),
                output_cost_per_1k_tokens: Some(output_cost),
                currency: "USD".to_string(),
                capabilities: vec![], // Will be set below from features
                created_at: None,
                updated_at: None,
                metadata: HashMap::new(),
            };

            let features = self.detect_features(&model_info);

            // Convert features to capabilities
            model_info.capabilities = features
                .iter()
                .filter_map(|f| f.to_provider_capability())
                .collect();
            let config = self.create_config(&model_info);

            self.models.insert(
                id.to_string(),
                OpenAIModelSpec {
                    model_info,
                    features,
                    family,
                    config,
                },
            );
        }
    }

    /// Get all model information
    pub fn get_all_models(&self) -> Vec<ModelInfo> {
        self.models
            .values()
            .map(|spec| spec.model_info.clone())
            .collect()
    }

    /// Get specific model specification
    pub fn get_model_spec(&self, model_id: &str) -> Option<&OpenAIModelSpec> {
        self.models.get(model_id)
    }

    /// Check if model supports a feature
    pub fn supports_feature(&self, model_id: &str, feature: &OpenAIModelFeature) -> bool {
        self.models
            .get(model_id)
            .map(|spec| spec.features.contains(feature))
            .unwrap_or(false)
    }

    /// Get models by family
    pub fn get_models_by_family(&self, family: &OpenAIModelFamily) -> Vec<String> {
        self.models
            .iter()
            .filter_map(|(id, spec)| {
                if &spec.family == family {
                    Some(id.clone())
                } else {
                    None
                }
            })
            .collect()
    }

    /// Get models supporting specific feature
    pub fn get_models_with_feature(&self, feature: &OpenAIModelFeature) -> Vec<String> {
        self.models
            .iter()
            .filter_map(|(id, spec)| {
                if spec.features.contains(feature) {
                    Some(id.clone())
                } else {
                    None
                }
            })
            .collect()
    }

    /// Get the best model for a specific use case
    pub fn get_recommended_model(&self, use_case: OpenAIUseCase) -> Option<String> {
        match use_case {
            OpenAIUseCase::GeneralChat => Some("gpt-5.2-chat".to_string()),
            OpenAIUseCase::CodeGeneration => Some("gpt-5.2-codex".to_string()),
            OpenAIUseCase::Reasoning => Some("o3-pro".to_string()),
            OpenAIUseCase::Vision => Some("gpt-5.2".to_string()),
            OpenAIUseCase::ImageGeneration => Some("gpt-image-1.5".to_string()),
            OpenAIUseCase::AudioTranscription => Some("whisper-1".to_string()),
            OpenAIUseCase::TextToSpeech => Some("tts-1-hd".to_string()),
            OpenAIUseCase::Embeddings => Some("text-embedding-3-large".to_string()),
            OpenAIUseCase::CostOptimized => Some("gpt-5-nano".to_string()),
        }
    }
}

/// OpenAI use cases for model recommendation
#[derive(Debug, Clone)]
pub enum OpenAIUseCase {
    GeneralChat,
    CodeGeneration,
    Reasoning,
    Vision,
    ImageGeneration,
    AudioTranscription,
    TextToSpeech,
    Embeddings,
    CostOptimized,
}

/// Global model registry instance
static OPENAI_REGISTRY: OnceLock<OpenAIModelRegistry> = OnceLock::new();

/// Get global OpenAI model registry
pub fn get_openai_registry() -> &'static OpenAIModelRegistry {
    OPENAI_REGISTRY.get_or_init(OpenAIModelRegistry::new)
}

#[cfg(test)]
#[path = "models_tests.rs"]
mod tests;

// ============================================================================
// OpenAI API Request/Response Types
// ============================================================================

/// OpenAI Chat Request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAIChatRequest {
    pub model: String,
    pub messages: Vec<OpenAIMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_completion_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_p: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub frequency_penalty: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub presence_penalty: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<OpenAITool>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_choice: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parallel_tool_calls: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response_format: Option<OpenAIResponseFormat>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub seed: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub n: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logit_bias: Option<HashMap<String, f32>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logprobs: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_logprobs: Option<u32>,
}

/// OpenAI Message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAIMessage {
    pub role: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<Vec<OpenAIToolCall>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_call_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub function_call: Option<OpenAIFunctionCall>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reasoning: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reasoning_details: Option<serde_json::Value>,
    /// DeepSeek reasoning content field
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reasoning_content: Option<String>,
}

/// OpenAI Tool
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAITool {
    #[serde(rename = "type")]
    pub tool_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub function: Option<OpenAIFunction>,
}

/// OpenAI Function
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAIFunction {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parameters: Option<serde_json::Value>,
}

/// OpenAI Tool Call
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAIToolCall {
    pub id: String,
    #[serde(rename = "type")]
    pub tool_type: String,
    pub function: OpenAIFunctionCall,
}

/// OpenAI Function Call
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAIFunctionCall {
    pub name: String,
    pub arguments: String,
}

/// OpenAI Response Format
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAIResponseFormat {
    #[serde(rename = "type")]
    pub format_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub json_schema: Option<serde_json::Value>,
}

/// OpenAI Chat Response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAIChatResponse {
    pub id: String,
    pub object: String,
    pub created: i64,
    pub model: String,
    pub choices: Vec<OpenAIChoice>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub usage: Option<OpenAIUsage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system_fingerprint: Option<String>,
}

/// OpenAI Choice
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAIChoice {
    pub index: u32,
    pub message: OpenAIMessage,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub finish_reason: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logprobs: Option<serde_json::Value>,
}

/// OpenAI Usage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAIUsage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompt_tokens_details: Option<OpenAITokenDetails>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub completion_tokens_details: Option<OpenAITokenDetails>,
}

/// OpenAI Token Details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAITokenDetails {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cached_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub audio_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reasoning_tokens: Option<u32>,
}

/// OpenAI Stream Chunk
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAIStreamChunk {
    pub id: String,
    pub object: String,
    pub created: i64,
    pub model: String,
    pub choices: Vec<OpenAIStreamChoice>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub usage: Option<OpenAIUsage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system_fingerprint: Option<String>,
}

/// OpenAI Stream Choice
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAIStreamChoice {
    pub index: u32,
    pub delta: OpenAIDelta,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub finish_reason: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logprobs: Option<serde_json::Value>,
}

/// OpenAI Delta
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAIDelta {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub role: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<Vec<OpenAIToolCallDelta>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub function_call: Option<OpenAIFunctionCallDelta>,
}

/// OpenAI Tool Call Delta
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAIToolCallDelta {
    pub index: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "type")]
    pub tool_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub function: Option<OpenAIFunctionCallDelta>,
}

/// OpenAI Function Call Delta
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAIFunctionCallDelta {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub arguments: Option<String>,
}

/// OpenAI Content Part
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum OpenAIContentPart {
    #[serde(rename = "text")]
    Text { text: String },
    #[serde(rename = "image_url")]
    ImageUrl { image_url: OpenAIImageUrl },
    #[serde(rename = "input_audio")]
    InputAudio { input_audio: OpenAIInputAudio },
}

/// OpenAI Image URL
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAIImageUrl {
    pub url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
}

/// OpenAI Input Audio
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAIInputAudio {
    pub data: String,
    pub format: String,
}

/// OpenAI Tool Choice
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum OpenAIToolChoice {
    String(String), // "none", "auto", "required"
    Function {
        #[serde(rename = "type")]
        r#type: String,
        function: OpenAIFunctionChoice,
    },
}

impl OpenAIToolChoice {
    pub fn none() -> Self {
        Self::String("none".to_string())
    }

    pub fn auto() -> Self {
        Self::String("auto".to_string())
    }

    pub fn required() -> Self {
        Self::String("required".to_string())
    }
}

/// OpenAI Function Choice
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAIFunctionChoice {
    pub name: String,
}

/// OpenAI Logprobs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAILogprobs {
    pub content: Option<Vec<OpenAITokenLogprob>>,
    pub refusal: Option<serde_json::Value>,
}

/// OpenAI Token Logprob
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAITokenLogprob {
    pub token: String,
    pub logprob: f64,
    pub bytes: Option<Vec<u8>>,
    pub top_logprobs: Vec<OpenAITopLogprob>,
}

/// OpenAI Top Logprob
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAITopLogprob {
    pub token: String,
    pub logprob: f64,
    pub bytes: Option<Vec<u8>>,
}
