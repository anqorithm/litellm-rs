use super::{
    GeminiModelFamily, GeminiModelRegistry, ModelFeature, ModelLimits, ModelPricing, ModelSpec,
};
use crate::core::types::model::ModelInfo;

pub(super) fn initialize_models(registry: &mut GeminiModelRegistry) {
    // ==================== Gemini 3.0 Series (2025 - Latest) ====================

    // Gemini 3 Pro
    registry.register_model(
        "gemini-3-pro",
        ModelSpec {
            model_info: ModelInfo {
                id: "gemini-3-pro".to_string(),
                name: "Gemini 3 Pro".to_string(),
                provider: "gemini".to_string(),
                max_context_length: 1_000_000,
                max_output_length: Some(65536),
                supports_streaming: true,
                supports_tools: true,
                supports_multimodal: true,
                input_cost_per_1k_tokens: Some(0.002),
                output_cost_per_1k_tokens: Some(0.012),
                currency: "USD".to_string(),
                capabilities: vec![
                    crate::core::types::model::ProviderCapability::ChatCompletion,
                    crate::core::types::model::ProviderCapability::ChatCompletionStream,
                    crate::core::types::model::ProviderCapability::ToolCalling,
                ],
                created_at: None,
                updated_at: None,
                metadata: std::collections::HashMap::new(),
            },
            family: GeminiModelFamily::Gemini3Pro,
            features: vec![
                ModelFeature::MultimodalSupport,
                ModelFeature::ToolCalling,
                ModelFeature::FunctionCalling,
                ModelFeature::StreamingSupport,
                ModelFeature::ContextCaching,
                ModelFeature::SystemInstructions,
                ModelFeature::BatchProcessing,
                ModelFeature::JsonMode,
                ModelFeature::CodeExecution,
                ModelFeature::SearchGrounding,
                ModelFeature::VideoUnderstanding,
                ModelFeature::AudioUnderstanding,
            ],
            pricing: ModelPricing {
                input_price: 2.0,   // $2 per 1M tokens (<=200K)
                output_price: 12.0, // $12 per 1M tokens (<=200K)
                cached_input_price: Some(0.5),
                image_price: Some(0.005),
                video_price_per_second: Some(0.005),
                audio_price_per_second: Some(0.0005),
            },
            limits: ModelLimits {
                max_context_length: 1_000_000,
                max_output_tokens: 65536,
                max_images: Some(3000),
                max_video_seconds: Some(3600),
                max_audio_seconds: Some(9600),
                rpm_limit: Some(1000),
                tpm_limit: Some(4_000_000),
            },
        },
    );

    // Gemini 3 Pro Deep Think
    registry.register_model(
        "gemini-3-pro-deep-think",
        ModelSpec {
            model_info: ModelInfo {
                id: "gemini-3-pro-deep-think".to_string(),
                name: "Gemini 3 Pro Deep Think".to_string(),
                provider: "gemini".to_string(),
                max_context_length: 1_000_000,
                max_output_length: Some(65536),
                supports_streaming: true,
                supports_tools: true,
                supports_multimodal: true,
                input_cost_per_1k_tokens: Some(0.004),
                output_cost_per_1k_tokens: Some(0.024),
                currency: "USD".to_string(),
                capabilities: vec![
                    crate::core::types::model::ProviderCapability::ChatCompletion,
                    crate::core::types::model::ProviderCapability::ChatCompletionStream,
                    crate::core::types::model::ProviderCapability::ToolCalling,
                ],
                created_at: None,
                updated_at: None,
                metadata: std::collections::HashMap::new(),
            },
            family: GeminiModelFamily::Gemini3ProDeepThink,
            features: vec![
                ModelFeature::MultimodalSupport,
                ModelFeature::ToolCalling,
                ModelFeature::FunctionCalling,
                ModelFeature::StreamingSupport,
                ModelFeature::ContextCaching,
                ModelFeature::SystemInstructions,
                ModelFeature::JsonMode,
                ModelFeature::CodeExecution,
                ModelFeature::SearchGrounding,
                ModelFeature::VideoUnderstanding,
                ModelFeature::AudioUnderstanding,
            ],
            pricing: ModelPricing {
                input_price: 4.0,   // $4 per 1M tokens (deep think mode)
                output_price: 24.0, // $24 per 1M tokens (deep think mode)
                cached_input_price: Some(1.0),
                image_price: Some(0.01),
                video_price_per_second: Some(0.01),
                audio_price_per_second: Some(0.001),
            },
            limits: ModelLimits {
                max_context_length: 1_000_000,
                max_output_tokens: 65536,
                max_images: Some(3000),
                max_video_seconds: Some(3600),
                max_audio_seconds: Some(9600),
                rpm_limit: Some(500),
                tpm_limit: Some(2_000_000),
            },
        },
    );

    // Gemini 3 Flash Preview
    registry.register_model(
        "gemini-3-flash-preview",
        ModelSpec {
            model_info: ModelInfo {
                id: "gemini-3-flash-preview".to_string(),
                name: "Gemini 3 Flash Preview".to_string(),
                provider: "gemini".to_string(),
                max_context_length: 1_048_576,
                max_output_length: Some(65536),
                supports_streaming: true,
                supports_tools: true,
                supports_multimodal: true,
                input_cost_per_1k_tokens: Some(0.0005),
                output_cost_per_1k_tokens: Some(0.003),
                currency: "USD".to_string(),
                capabilities: vec![
                    crate::core::types::model::ProviderCapability::ChatCompletion,
                    crate::core::types::model::ProviderCapability::ChatCompletionStream,
                    crate::core::types::model::ProviderCapability::ToolCalling,
                ],
                created_at: None,
                updated_at: None,
                metadata: std::collections::HashMap::new(),
            },
            family: GeminiModelFamily::Gemini3Flash,
            features: vec![
                ModelFeature::MultimodalSupport,
                ModelFeature::ToolCalling,
                ModelFeature::FunctionCalling,
                ModelFeature::StreamingSupport,
                ModelFeature::ContextCaching,
                ModelFeature::SystemInstructions,
                ModelFeature::BatchProcessing,
                ModelFeature::JsonMode,
                ModelFeature::CodeExecution,
                ModelFeature::SearchGrounding,
                ModelFeature::VideoUnderstanding,
                ModelFeature::AudioUnderstanding,
            ],
            pricing: ModelPricing {
                input_price: 0.5,  // $0.50 per 1M tokens
                output_price: 3.0, // $3 per 1M tokens
                cached_input_price: Some(0.125),
                image_price: Some(0.002),
                video_price_per_second: Some(0.002),
                audio_price_per_second: Some(0.0002),
            },
            limits: ModelLimits {
                max_context_length: 1_048_576,
                max_output_tokens: 65536,
                max_images: Some(3000),
                max_video_seconds: Some(3600),
                max_audio_seconds: Some(9600),
                rpm_limit: Some(2000),
                tpm_limit: Some(8_000_000),
            },
        },
    );

    // Gemini 3 Pro Image Preview
    registry.register_model(
        "gemini-3-pro-image-preview",
        ModelSpec {
            model_info: ModelInfo {
                id: "gemini-3-pro-image-preview".to_string(),
                name: "Gemini 3 Pro Image Preview".to_string(),
                provider: "gemini".to_string(),
                max_context_length: 65536,
                max_output_length: Some(8192),
                supports_streaming: true,
                supports_tools: true,
                supports_multimodal: true,
                input_cost_per_1k_tokens: Some(0.002),
                output_cost_per_1k_tokens: Some(0.012),
                currency: "USD".to_string(),
                capabilities: vec![
                    crate::core::types::model::ProviderCapability::ChatCompletion,
                    crate::core::types::model::ProviderCapability::ChatCompletionStream,
                    crate::core::types::model::ProviderCapability::ImageGeneration,
                ],
                created_at: None,
                updated_at: None,
                metadata: std::collections::HashMap::new(),
            },
            family: GeminiModelFamily::Gemini3ProImage,
            features: vec![
                ModelFeature::MultimodalSupport,
                ModelFeature::StreamingSupport,
                ModelFeature::SystemInstructions,
                ModelFeature::JsonMode,
            ],
            pricing: ModelPricing {
                input_price: 2.0,   // $2 per 1M tokens
                output_price: 12.0, // $12 per 1M tokens
                cached_input_price: Some(0.5),
                image_price: Some(0.04), // Image generation pricing
                video_price_per_second: None,
                audio_price_per_second: None,
            },
            limits: ModelLimits {
                max_context_length: 65536,
                max_output_tokens: 8192,
                max_images: Some(16),
                max_video_seconds: None,
                max_audio_seconds: None,
                rpm_limit: Some(500),
                tpm_limit: Some(1_000_000),
            },
        },
    );

    // ==================== Gemini 2.5 Series (2025) ====================

    // Gemini 2.5 Pro
    registry.register_model(
        "gemini-2.5-pro",
        ModelSpec {
            model_info: ModelInfo {
                id: "gemini-2.5-pro".to_string(),
                name: "Gemini 2.5 Pro".to_string(),
                provider: "gemini".to_string(),
                max_context_length: 1_000_000,
                max_output_length: Some(65536),
                supports_streaming: true,
                supports_tools: true,
                supports_multimodal: true,
                input_cost_per_1k_tokens: Some(0.00125),
                output_cost_per_1k_tokens: Some(0.010),
                currency: "USD".to_string(),
                capabilities: vec![
                    crate::core::types::model::ProviderCapability::ChatCompletion,
                    crate::core::types::model::ProviderCapability::ChatCompletionStream,
                    crate::core::types::model::ProviderCapability::ToolCalling,
                ],
                created_at: None,
                updated_at: None,
                metadata: std::collections::HashMap::new(),
            },
            family: GeminiModelFamily::Gemini25Pro,
            features: vec![
                ModelFeature::MultimodalSupport,
                ModelFeature::ToolCalling,
                ModelFeature::FunctionCalling,
                ModelFeature::StreamingSupport,
                ModelFeature::ContextCaching,
                ModelFeature::SystemInstructions,
                ModelFeature::BatchProcessing,
                ModelFeature::JsonMode,
                ModelFeature::CodeExecution,
                ModelFeature::SearchGrounding,
                ModelFeature::VideoUnderstanding,
                ModelFeature::AudioUnderstanding,
            ],
            pricing: ModelPricing {
                input_price: 1.25,  // $1.25 per 1M tokens (<=200K)
                output_price: 10.0, // $10 per 1M tokens (<=200K)
                cached_input_price: Some(0.3125),
                image_price: Some(0.005),
                video_price_per_second: Some(0.005),
                audio_price_per_second: Some(0.0005),
            },
            limits: ModelLimits {
                max_context_length: 1_000_000,
                max_output_tokens: 65536,
                max_images: Some(3000),
                max_video_seconds: Some(3600),
                max_audio_seconds: Some(9600),
                rpm_limit: Some(1000),
                tpm_limit: Some(4_000_000),
            },
        },
    );

    // Gemini 2.5 Flash
    registry.register_model(
        "gemini-2.5-flash",
        ModelSpec {
            model_info: ModelInfo {
                id: "gemini-2.5-flash".to_string(),
                name: "Gemini 2.5 Flash".to_string(),
                provider: "gemini".to_string(),
                max_context_length: 1_000_000,
                max_output_length: Some(65536),
                supports_streaming: true,
                supports_tools: true,
                supports_multimodal: true,
                input_cost_per_1k_tokens: Some(0.0003),
                output_cost_per_1k_tokens: Some(0.0025),
                currency: "USD".to_string(),
                capabilities: vec![
                    crate::core::types::model::ProviderCapability::ChatCompletion,
                    crate::core::types::model::ProviderCapability::ChatCompletionStream,
                    crate::core::types::model::ProviderCapability::ToolCalling,
                ],
                created_at: None,
                updated_at: None,
                metadata: std::collections::HashMap::new(),
            },
            family: GeminiModelFamily::Gemini25Flash,
            features: vec![
                ModelFeature::MultimodalSupport,
                ModelFeature::ToolCalling,
                ModelFeature::FunctionCalling,
                ModelFeature::StreamingSupport,
                ModelFeature::ContextCaching,
                ModelFeature::SystemInstructions,
                ModelFeature::BatchProcessing,
                ModelFeature::JsonMode,
                ModelFeature::CodeExecution,
                ModelFeature::SearchGrounding,
                ModelFeature::VideoUnderstanding,
                ModelFeature::AudioUnderstanding,
            ],
            pricing: ModelPricing {
                input_price: 0.30,  // $0.30 per 1M tokens
                output_price: 2.50, // $2.50 per 1M tokens
                cached_input_price: Some(0.075),
                image_price: Some(0.0002),
                video_price_per_second: Some(0.0002),
                audio_price_per_second: Some(0.0001),
            },
            limits: ModelLimits {
                max_context_length: 1_000_000,
                max_output_tokens: 65536,
                max_images: Some(3000),
                max_video_seconds: Some(3600),
                max_audio_seconds: Some(9600),
                rpm_limit: Some(2000),
                tpm_limit: Some(4_000_000),
            },
        },
    );

    // Gemini 2.5 Flash-Lite
    registry.register_model(
        "gemini-2.5-flash-lite",
        ModelSpec {
            model_info: ModelInfo {
                id: "gemini-2.5-flash-lite".to_string(),
                name: "Gemini 2.5 Flash-Lite".to_string(),
                provider: "gemini".to_string(),
                max_context_length: 1_000_000,
                max_output_length: Some(65536),
                supports_streaming: true,
                supports_tools: true,
                supports_multimodal: true,
                input_cost_per_1k_tokens: Some(0.0001),
                output_cost_per_1k_tokens: Some(0.0004),
                currency: "USD".to_string(),
                capabilities: vec![
                    crate::core::types::model::ProviderCapability::ChatCompletion,
                    crate::core::types::model::ProviderCapability::ChatCompletionStream,
                    crate::core::types::model::ProviderCapability::ToolCalling,
                ],
                created_at: None,
                updated_at: None,
                metadata: std::collections::HashMap::new(),
            },
            family: GeminiModelFamily::Gemini25FlashLite,
            features: vec![
                ModelFeature::MultimodalSupport,
                ModelFeature::ToolCalling,
                ModelFeature::FunctionCalling,
                ModelFeature::StreamingSupport,
                ModelFeature::ContextCaching,
                ModelFeature::SystemInstructions,
                ModelFeature::BatchProcessing,
                ModelFeature::JsonMode,
                ModelFeature::CodeExecution,
                ModelFeature::SearchGrounding,
            ],
            pricing: ModelPricing {
                input_price: 0.10,  // $0.10 per 1M tokens
                output_price: 0.40, // $0.40 per 1M tokens
                cached_input_price: Some(0.025),
                image_price: Some(0.0001),
                video_price_per_second: None,
                audio_price_per_second: None,
            },
            limits: ModelLimits {
                max_context_length: 1_000_000,
                max_output_tokens: 65536,
                max_images: Some(3000),
                max_video_seconds: None,
                max_audio_seconds: None,
                rpm_limit: Some(4000),
                tpm_limit: Some(4_000_000),
            },
        },
    );

    // ==================== Gemini 2.0 Series ====================

    // Gemini 2.0 Flash
    registry.register_model(
        "gemini-2.0-flash-exp",
        ModelSpec {
            model_info: ModelInfo {
                id: "gemini-2.0-flash-exp".to_string(),
                name: "Gemini 2.0 Flash".to_string(),
                provider: "gemini".to_string(),
                max_context_length: 1_000_000,
                max_output_length: Some(8192),
                supports_streaming: true,
                supports_tools: true,
                supports_multimodal: true,
                input_cost_per_1k_tokens: Some(0.00001),
                output_cost_per_1k_tokens: Some(0.00004),
                currency: "USD".to_string(),
                capabilities: vec![
                    crate::core::types::model::ProviderCapability::ChatCompletion,
                    crate::core::types::model::ProviderCapability::ChatCompletionStream,
                    crate::core::types::model::ProviderCapability::ToolCalling,
                ],
                created_at: None,
                updated_at: None,
                metadata: std::collections::HashMap::new(),
            },
            family: GeminiModelFamily::Gemini20Flash,
            features: vec![
                ModelFeature::MultimodalSupport,
                ModelFeature::ToolCalling,
                ModelFeature::FunctionCalling,
                ModelFeature::StreamingSupport,
                ModelFeature::ContextCaching,
                ModelFeature::SystemInstructions,
                ModelFeature::BatchProcessing,
                ModelFeature::JsonMode,
                ModelFeature::CodeExecution,
                ModelFeature::SearchGrounding,
                ModelFeature::VideoUnderstanding,
                ModelFeature::AudioUnderstanding,
            ],
            pricing: ModelPricing {
                input_price: 0.01,  // $0.01 per 1M tokens
                output_price: 0.04, // $0.04 per 1M tokens
                cached_input_price: Some(0.0025),
                image_price: Some(0.0001),
                video_price_per_second: Some(0.001),
                audio_price_per_second: Some(0.0001),
            },
            limits: ModelLimits {
                max_context_length: 1_000_000,
                max_output_tokens: 8192,
                max_images: Some(3000),
                max_video_seconds: Some(3600),
                max_audio_seconds: Some(9600),
                rpm_limit: Some(2000),
                tpm_limit: Some(4_000_000),
            },
        },
    );

    // Gemini 2.0 Flash Thinking (experimental)
    registry.register_model(
        "gemini-2.0-flash-thinking-exp",
        ModelSpec {
            model_info: ModelInfo {
                id: "gemini-2.0-flash-thinking-exp".to_string(),
                name: "Gemini 2.0 Flash Thinking".to_string(),
                provider: "gemini".to_string(),
                max_context_length: 32_000,
                max_output_length: Some(8192),
                supports_streaming: true,
                supports_tools: true,
                supports_multimodal: true,
                input_cost_per_1k_tokens: Some(0.00001),
                output_cost_per_1k_tokens: Some(0.00004),
                currency: "USD".to_string(),
                capabilities: vec![
                    crate::core::types::model::ProviderCapability::ChatCompletion,
                    crate::core::types::model::ProviderCapability::ChatCompletionStream,
                ],
                created_at: None,
                updated_at: None,
                metadata: std::collections::HashMap::new(),
            },
            family: GeminiModelFamily::Gemini20FlashThinking,
            features: vec![
                ModelFeature::MultimodalSupport,
                ModelFeature::StreamingSupport,
                ModelFeature::SystemInstructions,
            ],
            pricing: ModelPricing {
                input_price: 0.01,
                output_price: 0.04,
                cached_input_price: None,
                image_price: Some(0.0001),
                video_price_per_second: None,
                audio_price_per_second: None,
            },
            limits: ModelLimits {
                max_context_length: 32_000,
                max_output_tokens: 8192,
                max_images: Some(50),
                max_video_seconds: None,
                max_audio_seconds: None,
                rpm_limit: Some(100),
                tpm_limit: Some(100_000),
            },
        },
    );

    // Gemini 1.5 Pro
    registry.register_model(
        "gemini-1.5-pro",
        ModelSpec {
            model_info: ModelInfo {
                id: "gemini-1.5-pro".to_string(),
                name: "Gemini 1.5 Pro".to_string(),
                provider: "gemini".to_string(),
                max_context_length: 2_000_000,
                max_output_length: Some(8192),
                supports_streaming: true,
                supports_tools: true,
                supports_multimodal: true,
                input_cost_per_1k_tokens: Some(0.00125),
                output_cost_per_1k_tokens: Some(0.005),
                currency: "USD".to_string(),
                capabilities: vec![
                    crate::core::types::model::ProviderCapability::ChatCompletion,
                    crate::core::types::model::ProviderCapability::ChatCompletionStream,
                    crate::core::types::model::ProviderCapability::ToolCalling,
                ],
                created_at: None,
                updated_at: None,
                metadata: std::collections::HashMap::new(),
            },
            family: GeminiModelFamily::Gemini15Pro,
            features: vec![
                ModelFeature::MultimodalSupport,
                ModelFeature::ToolCalling,
                ModelFeature::FunctionCalling,
                ModelFeature::StreamingSupport,
                ModelFeature::ContextCaching,
                ModelFeature::SystemInstructions,
                ModelFeature::BatchProcessing,
                ModelFeature::JsonMode,
                ModelFeature::CodeExecution,
                ModelFeature::SearchGrounding,
                ModelFeature::VideoUnderstanding,
                ModelFeature::AudioUnderstanding,
            ],
            pricing: ModelPricing {
                input_price: 1.25, // $1.25 per 1M tokens (<=128K)
                output_price: 5.0, // $5.00 per 1M tokens (<=128K)
                cached_input_price: Some(0.3125),
                image_price: Some(0.002625),
                video_price_per_second: Some(0.002625),
                audio_price_per_second: Some(0.000125),
            },
            limits: ModelLimits {
                max_context_length: 2_000_000,
                max_output_tokens: 8192,
                max_images: Some(3000),
                max_video_seconds: Some(3600),
                max_audio_seconds: Some(9600),
                rpm_limit: Some(360),
                tpm_limit: Some(4_000_000),
            },
        },
    );

    // Gemini 1.5 Flash
    registry.register_model(
        "gemini-1.5-flash",
        ModelSpec {
            model_info: ModelInfo {
                id: "gemini-1.5-flash".to_string(),
                name: "Gemini 1.5 Flash".to_string(),
                provider: "gemini".to_string(),
                max_context_length: 1_000_000,
                max_output_length: Some(8192),
                supports_streaming: true,
                supports_tools: true,
                supports_multimodal: true,
                input_cost_per_1k_tokens: Some(0.000075),
                output_cost_per_1k_tokens: Some(0.0003),
                currency: "USD".to_string(),
                capabilities: vec![
                    crate::core::types::model::ProviderCapability::ChatCompletion,
                    crate::core::types::model::ProviderCapability::ChatCompletionStream,
                    crate::core::types::model::ProviderCapability::ToolCalling,
                ],
                created_at: None,
                updated_at: None,
                metadata: std::collections::HashMap::new(),
            },
            family: GeminiModelFamily::Gemini15Flash,
            features: vec![
                ModelFeature::MultimodalSupport,
                ModelFeature::ToolCalling,
                ModelFeature::FunctionCalling,
                ModelFeature::StreamingSupport,
                ModelFeature::ContextCaching,
                ModelFeature::SystemInstructions,
                ModelFeature::BatchProcessing,
                ModelFeature::JsonMode,
                ModelFeature::CodeExecution,
                ModelFeature::SearchGrounding,
                ModelFeature::VideoUnderstanding,
                ModelFeature::AudioUnderstanding,
            ],
            pricing: ModelPricing {
                input_price: 0.075, // $0.075 per 1M tokens (<=128K)
                output_price: 0.30, // $0.30 per 1M tokens (<=128K)
                cached_input_price: Some(0.01875),
                image_price: Some(0.0002),
                video_price_per_second: Some(0.0002),
                audio_price_per_second: Some(0.0001),
            },
            limits: ModelLimits {
                max_context_length: 1_000_000,
                max_output_tokens: 8192,
                max_images: Some(3000),
                max_video_seconds: Some(3600),
                max_audio_seconds: Some(9600),
                rpm_limit: Some(1500),
                tpm_limit: Some(4_000_000),
            },
        },
    );

    // Gemini 1.5 Flash-8B
    registry.register_model(
        "gemini-1.5-flash-8b",
        ModelSpec {
            model_info: ModelInfo {
                id: "gemini-1.5-flash-8b".to_string(),
                name: "Gemini 1.5 Flash 8B".to_string(),
                provider: "gemini".to_string(),
                max_context_length: 1_000_000,
                max_output_length: Some(8192),
                supports_streaming: true,
                supports_tools: true,
                supports_multimodal: true,
                input_cost_per_1k_tokens: Some(0.0000375),
                output_cost_per_1k_tokens: Some(0.00015),
                currency: "USD".to_string(),
                capabilities: vec![
                    crate::core::types::model::ProviderCapability::ChatCompletion,
                    crate::core::types::model::ProviderCapability::ChatCompletionStream,
                    crate::core::types::model::ProviderCapability::ToolCalling,
                ],
                created_at: None,
                updated_at: None,
                metadata: std::collections::HashMap::new(),
            },
            family: GeminiModelFamily::Gemini15Flash8B,
            features: vec![
                ModelFeature::MultimodalSupport,
                ModelFeature::ToolCalling,
                ModelFeature::FunctionCalling,
                ModelFeature::StreamingSupport,
                ModelFeature::ContextCaching,
                ModelFeature::SystemInstructions,
                ModelFeature::BatchProcessing,
                ModelFeature::JsonMode,
                ModelFeature::VideoUnderstanding,
                ModelFeature::AudioUnderstanding,
            ],
            pricing: ModelPricing {
                input_price: 0.0375, // $0.0375 per 1M tokens
                output_price: 0.15,  // $0.15 per 1M tokens
                cached_input_price: Some(0.01),
                image_price: Some(0.0001),
                video_price_per_second: Some(0.0001),
                audio_price_per_second: Some(0.00005),
            },
            limits: ModelLimits {
                max_context_length: 1_000_000,
                max_output_tokens: 8192,
                max_images: Some(3000),
                max_video_seconds: Some(3600),
                max_audio_seconds: Some(9600),
                rpm_limit: Some(4000),
                tpm_limit: Some(4_000_000),
            },
        },
    );

    // Gemini 1.0 Pro
    registry.register_model(
        "gemini-1.0-pro",
        ModelSpec {
            model_info: ModelInfo {
                id: "gemini-1.0-pro".to_string(),
                name: "Gemini 1.0 Pro".to_string(),
                provider: "gemini".to_string(),
                max_context_length: 32_000,
                max_output_length: Some(8192),
                supports_streaming: true,
                supports_tools: true,
                supports_multimodal: false,
                input_cost_per_1k_tokens: Some(0.0005),
                output_cost_per_1k_tokens: Some(0.0015),
                currency: "USD".to_string(),
                capabilities: vec![
                    crate::core::types::model::ProviderCapability::ChatCompletion,
                    crate::core::types::model::ProviderCapability::ChatCompletionStream,
                    crate::core::types::model::ProviderCapability::ToolCalling,
                ],
                created_at: None,
                updated_at: None,
                metadata: std::collections::HashMap::new(),
            },
            family: GeminiModelFamily::Gemini10Pro,
            features: vec![
                ModelFeature::ToolCalling,
                ModelFeature::FunctionCalling,
                ModelFeature::StreamingSupport,
                ModelFeature::SystemInstructions,
                ModelFeature::BatchProcessing,
            ],
            pricing: ModelPricing {
                input_price: 0.50,  // $0.50 per 1M tokens
                output_price: 1.50, // $1.50 per 1M tokens
                cached_input_price: None,
                image_price: None,
                video_price_per_second: None,
                audio_price_per_second: None,
            },
            limits: ModelLimits {
                max_context_length: 32_000,
                max_output_tokens: 8192,
                max_images: None,
                max_video_seconds: None,
                max_audio_seconds: None,
                rpm_limit: Some(300),
                tpm_limit: Some(300_000),
            },
        },
    );
}
