//! HuggingFace Models and Types
//!
//! Model definitions, task types, and inference provider mappings.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::core::types::model::ModelInfo;

/// HuggingFace task types for inference
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum HuggingFaceTask {
    /// Text generation using TGI (Text Generation Inference)
    #[serde(rename = "text-generation-inference")]
    TextGenerationInference,

    /// Conversational models
    #[serde(rename = "conversational")]
    Conversational,

    /// Text classification
    #[serde(rename = "text-classification")]
    TextClassification,

    /// Text generation (non-TGI)
    #[serde(rename = "text-generation")]
    TextGeneration,

    /// Sentence similarity / embeddings
    #[serde(rename = "sentence-similarity")]
    SentenceSimilarity,

    /// Feature extraction (embeddings)
    #[serde(rename = "feature-extraction")]
    FeatureExtraction,

    /// Reranking
    #[serde(rename = "rerank")]
    Rerank,
}

impl HuggingFaceTask {
    /// Get task string for API calls
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::TextGenerationInference => "text-generation-inference",
            Self::Conversational => "conversational",
            Self::TextClassification => "text-classification",
            Self::TextGeneration => "text-generation",
            Self::SentenceSimilarity => "sentence-similarity",
            Self::FeatureExtraction => "feature-extraction",
            Self::Rerank => "rerank",
        }
    }

    /// Determine if this is a chat-compatible task
    pub fn is_chat_task(&self) -> bool {
        matches!(
            self,
            Self::TextGenerationInference | Self::Conversational | Self::TextGeneration
        )
    }

    /// Determine if this is an embedding task
    pub fn is_embedding_task(&self) -> bool {
        matches!(
            self,
            Self::SentenceSimilarity | Self::FeatureExtraction | Self::Rerank
        )
    }
}

impl std::fmt::Display for HuggingFaceTask {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Supported inference providers through HuggingFace Hub
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum InferenceProvider {
    /// HuggingFace Inference API (default)
    #[serde(rename = "hf-inference")]
    HFInference,

    /// Together AI
    #[serde(rename = "together")]
    Together,

    /// Sambanova
    #[serde(rename = "sambanova")]
    Sambanova,

    /// Fireworks AI
    #[serde(rename = "fireworks-ai")]
    FireworksAI,

    /// Novita AI
    #[serde(rename = "novita")]
    Novita,

    /// TGI (Text Generation Inference) Dedicated Endpoint
    #[serde(rename = "tgi")]
    TGI,

    /// Custom/Other provider
    #[serde(untagged)]
    Custom(String),
}

impl InferenceProvider {
    /// Get provider string for API routing
    pub fn as_str(&self) -> &str {
        match self {
            Self::HFInference => "hf-inference",
            Self::Together => "together",
            Self::Sambanova => "sambanova",
            Self::FireworksAI => "fireworks-ai",
            Self::Novita => "novita",
            Self::TGI => "tgi",
            Self::Custom(s) => s.as_str(),
        }
    }

    /// Parse provider from string
    pub fn parse(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "hf-inference" | "hf_inference" => Self::HFInference,
            "together" | "together-ai" => Self::Together,
            "sambanova" => Self::Sambanova,
            "fireworks-ai" | "fireworks" => Self::FireworksAI,
            "novita" => Self::Novita,
            "tgi" | "text-generation-inference" => Self::TGI,
            _ => Self::Custom(s.to_string()),
        }
    }
}

impl std::fmt::Display for InferenceProvider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Provider mapping information from HuggingFace Hub API
#[derive(Debug, Clone, Deserialize)]
pub struct ProviderMapping {
    /// Provider-specific model ID
    #[serde(rename = "providerId")]
    pub provider_id: String,

    /// Status (live, staging, etc.)
    pub status: String,
}

/// Parse model string to extract provider and model ID
///
/// Model format: `huggingface/<provider>/<org>/<model>` or `huggingface/<org>/<model>`
/// Returns: (provider, model_id)
pub fn parse_model_string(model: &str) -> (Option<String>, String) {
    let model = model.strip_prefix("huggingface/").unwrap_or(model);

    let parts: Vec<&str> = model.splitn(2, '/').collect();
    if parts.len() < 2 {
        return (None, model.to_string());
    }

    // Check if first part is a known provider
    let first_part = parts[0].to_lowercase();
    let known_providers = [
        "together",
        "sambanova",
        "fireworks-ai",
        "fireworks",
        "novita",
        "hf-inference",
        "tgi",
    ];

    if known_providers.contains(&first_part.as_str()) {
        (Some(parts[0].to_string()), parts[1].to_string())
    } else {
        // First part is org name, not provider
        (None, model.to_string())
    }
}

/// Get default models available through HuggingFace
pub fn get_default_models() -> Vec<ModelInfo> {
    vec![
        // Popular chat/completion models
        ModelInfo {
            id: "meta-llama/Llama-3.3-70B-Instruct".to_string(),
            name: "Llama 3.3 70B Instruct".to_string(),
            provider: "huggingface".to_string(),
            max_context_length: 128000,
            max_output_length: Some(8192),
            supports_streaming: true,
            supports_tools: true,
            supports_multimodal: false,
            input_cost_per_1k_tokens: None, // Varies by provider
            output_cost_per_1k_tokens: None,
            currency: "USD".to_string(),
            capabilities: vec![],
            created_at: None,
            updated_at: None,
            metadata: HashMap::new(),
        },
        ModelInfo {
            id: "deepseek-ai/DeepSeek-R1".to_string(),
            name: "DeepSeek R1".to_string(),
            provider: "huggingface".to_string(),
            max_context_length: 128000,
            max_output_length: Some(8192),
            supports_streaming: true,
            supports_tools: true,
            supports_multimodal: false,
            input_cost_per_1k_tokens: None,
            output_cost_per_1k_tokens: None,
            currency: "USD".to_string(),
            capabilities: vec![],
            created_at: None,
            updated_at: None,
            metadata: HashMap::new(),
        },
        ModelInfo {
            id: "Qwen/Qwen2.5-72B-Instruct".to_string(),
            name: "Qwen 2.5 72B Instruct".to_string(),
            provider: "huggingface".to_string(),
            max_context_length: 32768,
            max_output_length: Some(8192),
            supports_streaming: true,
            supports_tools: true,
            supports_multimodal: false,
            input_cost_per_1k_tokens: None,
            output_cost_per_1k_tokens: None,
            currency: "USD".to_string(),
            capabilities: vec![],
            created_at: None,
            updated_at: None,
            metadata: HashMap::new(),
        },
        ModelInfo {
            id: "mistralai/Mistral-7B-Instruct-v0.3".to_string(),
            name: "Mistral 7B Instruct v0.3".to_string(),
            provider: "huggingface".to_string(),
            max_context_length: 32768,
            max_output_length: Some(4096),
            supports_streaming: true,
            supports_tools: true,
            supports_multimodal: false,
            input_cost_per_1k_tokens: None,
            output_cost_per_1k_tokens: None,
            currency: "USD".to_string(),
            capabilities: vec![],
            created_at: None,
            updated_at: None,
            metadata: HashMap::new(),
        },
        // Vision model
        ModelInfo {
            id: "meta-llama/Llama-3.2-11B-Vision-Instruct".to_string(),
            name: "Llama 3.2 11B Vision Instruct".to_string(),
            provider: "huggingface".to_string(),
            max_context_length: 128000,
            max_output_length: Some(4096),
            supports_streaming: true,
            supports_tools: false,
            supports_multimodal: true,
            input_cost_per_1k_tokens: None,
            output_cost_per_1k_tokens: None,
            currency: "USD".to_string(),
            capabilities: vec![],
            created_at: None,
            updated_at: None,
            metadata: HashMap::new(),
        },
        // Embedding model
        ModelInfo {
            id: "microsoft/codebert-base".to_string(),
            name: "CodeBERT Base".to_string(),
            provider: "huggingface".to_string(),
            max_context_length: 512,
            max_output_length: None,
            supports_streaming: false,
            supports_tools: false,
            supports_multimodal: false,
            input_cost_per_1k_tokens: None,
            output_cost_per_1k_tokens: None,
            currency: "USD".to_string(),
            capabilities: vec![],
            created_at: None,
            updated_at: None,
            metadata: HashMap::new(),
        },
        ModelInfo {
            id: "sentence-transformers/all-MiniLM-L6-v2".to_string(),
            name: "All MiniLM L6 v2".to_string(),
            provider: "huggingface".to_string(),
            max_context_length: 512,
            max_output_length: None,
            supports_streaming: false,
            supports_tools: false,
            supports_multimodal: false,
            input_cost_per_1k_tokens: None,
            output_cost_per_1k_tokens: None,
            currency: "USD".to_string(),
            capabilities: vec![],
            created_at: None,
            updated_at: None,
            metadata: HashMap::new(),
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_task_as_str() {
        assert_eq!(
            HuggingFaceTask::TextGenerationInference.as_str(),
            "text-generation-inference"
        );
        assert_eq!(
            HuggingFaceTask::FeatureExtraction.as_str(),
            "feature-extraction"
        );
        assert_eq!(
            HuggingFaceTask::SentenceSimilarity.as_str(),
            "sentence-similarity"
        );
    }

    #[test]
    fn test_task_is_chat() {
        assert!(HuggingFaceTask::TextGenerationInference.is_chat_task());
        assert!(HuggingFaceTask::Conversational.is_chat_task());
        assert!(!HuggingFaceTask::FeatureExtraction.is_chat_task());
    }

    #[test]
    fn test_task_is_embedding() {
        assert!(HuggingFaceTask::FeatureExtraction.is_embedding_task());
        assert!(HuggingFaceTask::SentenceSimilarity.is_embedding_task());
        assert!(!HuggingFaceTask::TextGenerationInference.is_embedding_task());
    }

    #[test]
    fn test_provider_from_str() {
        assert_eq!(
            InferenceProvider::parse("together"),
            InferenceProvider::Together
        );
        assert_eq!(
            InferenceProvider::parse("fireworks-ai"),
            InferenceProvider::FireworksAI
        );
        assert_eq!(
            InferenceProvider::parse("hf-inference"),
            InferenceProvider::HFInference
        );
        assert!(matches!(
            InferenceProvider::parse("custom-provider"),
            InferenceProvider::Custom(_)
        ));
    }

    #[test]
    fn test_provider_as_str() {
        assert_eq!(InferenceProvider::Together.as_str(), "together");
        assert_eq!(InferenceProvider::FireworksAI.as_str(), "fireworks-ai");
        assert_eq!(
            InferenceProvider::Custom("my-provider".to_string()).as_str(),
            "my-provider"
        );
    }

    #[test]
    fn test_parse_model_string_with_provider() {
        let (provider, model) = parse_model_string("huggingface/together/deepseek-ai/DeepSeek-R1");
        assert_eq!(provider, Some("together".to_string()));
        assert_eq!(model, "deepseek-ai/DeepSeek-R1");
    }

    #[test]
    fn test_parse_model_string_without_provider() {
        let (provider, model) = parse_model_string("huggingface/meta-llama/Llama-3.3-70B-Instruct");
        assert!(provider.is_none());
        assert_eq!(model, "meta-llama/Llama-3.3-70B-Instruct");
    }

    #[test]
    fn test_parse_model_string_no_prefix() {
        let (provider, model) = parse_model_string("meta-llama/Llama-3.3-70B-Instruct");
        assert!(provider.is_none());
        assert_eq!(model, "meta-llama/Llama-3.3-70B-Instruct");
    }

    #[test]
    fn test_parse_model_string_sambanova() {
        let (provider, model) = parse_model_string("sambanova/Qwen/Qwen2.5-72B-Instruct");
        assert_eq!(provider, Some("sambanova".to_string()));
        assert_eq!(model, "Qwen/Qwen2.5-72B-Instruct");
    }

    #[test]
    fn test_default_models() {
        let models = get_default_models();
        assert!(!models.is_empty());
        assert!(models.iter().any(|m| m.id.contains("Llama")));
        assert!(models.iter().any(|m| m.id.contains("DeepSeek")));
        assert!(models.iter().all(|m| m.provider == "huggingface"));
    }
}
