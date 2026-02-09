//! Request models for the Gateway
//!
//! This module defines internal request structures used by the gateway.

use super::openai::*;
use super::{Metadata, RequestContext};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Internal gateway request wrapper
#[derive(Debug, Clone)]
pub struct GatewayRequest {
    /// Request metadata
    pub metadata: Metadata,
    /// Request context
    pub context: RequestContext,
    /// Request type
    pub request_type: RequestType,
    /// Original request data
    pub data: RequestData,
    /// Provider-specific parameters
    pub provider_params: HashMap<String, serde_json::Value>,
    /// Routing preferences
    pub routing: RoutingPreferences,
    /// Caching preferences
    pub caching: CachingPreferences,
}

/// Request type enumeration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RequestType {
    /// Chat completion requests
    ChatCompletion,
    /// Text completion requests
    Completion,
    /// Text embedding requests
    Embedding,
    /// Image generation requests
    ImageGeneration,
    /// Image editing requests
    ImageEdit,
    /// Image variation requests
    ImageVariation,
    /// Audio transcription requests
    AudioTranscription,
    /// Audio translation requests
    AudioTranslation,
    /// Text-to-speech requests
    AudioSpeech,
    /// Content moderation requests
    Moderation,
    /// Fine-tuning requests
    FineTuning,
    /// File management requests
    Files,
    /// Assistant API requests
    Assistants,
    /// Thread management requests
    Threads,
    /// Batch processing requests
    Batches,
    /// Vector store requests
    VectorStores,
    /// Document reranking requests
    Rerank,
    /// Real-time API requests
    Realtime,
}

/// Request data union
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum RequestData {
    /// Chat completion request data
    #[serde(rename = "chat_completion")]
    ChatCompletion(Box<ChatCompletionRequest>),
    /// Text completion request data
    #[serde(rename = "completion")]
    Completion(CompletionRequest),
    /// Embedding request data
    #[serde(rename = "embedding")]
    Embedding(EmbeddingRequest),
    /// Image generation request data
    #[serde(rename = "image_generation")]
    ImageGeneration(ImageGenerationRequest),
    /// Audio transcription request data
    #[serde(rename = "audio_transcription")]
    AudioTranscription(AudioTranscriptionRequest),
    /// Moderation request data
    #[serde(rename = "moderation")]
    Moderation(ModerationRequest),
    /// Rerank request data
    #[serde(rename = "rerank")]
    Rerank(RerankRequest),
}

/// Completion request (legacy)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionRequest {
    /// Model to use
    pub model: String,
    /// Prompt text
    pub prompt: Option<String>,
    /// Maximum tokens
    pub max_tokens: Option<u32>,
    /// Temperature
    pub temperature: Option<f32>,
    /// Top-p sampling
    pub top_p: Option<f32>,
    /// Number of completions
    pub n: Option<u32>,
    /// Stream response
    pub stream: Option<bool>,
    /// Stop sequences
    pub stop: Option<Vec<String>>,
    /// Presence penalty
    pub presence_penalty: Option<f32>,
    /// Frequency penalty
    pub frequency_penalty: Option<f32>,
    /// Logit bias
    pub logit_bias: Option<HashMap<String, f32>>,
    /// User identifier
    pub user: Option<String>,
    /// Suffix
    pub suffix: Option<String>,
    /// Echo prompt
    pub echo: Option<bool>,
    /// Best of
    pub best_of: Option<u32>,
    /// Logprobs
    pub logprobs: Option<u32>,
}

/// Embedding request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingRequest {
    /// Model to use
    pub model: String,
    /// Input text(s)
    pub input: EmbeddingInput,
    /// Encoding format
    pub encoding_format: Option<String>,
    /// Dimensions
    pub dimensions: Option<u32>,
    /// User identifier
    pub user: Option<String>,
}

/// Embedding input
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum EmbeddingInput {
    /// Single text string
    String(String),
    /// Array of text strings
    Array(Vec<String>),
    /// Array of token IDs
    Tokens(Vec<u32>),
    /// Array of token ID arrays
    TokenArrays(Vec<Vec<u32>>),
}

/// Image generation request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageGenerationRequest {
    /// Model to use
    pub model: Option<String>,
    /// Prompt
    pub prompt: String,
    /// Number of images
    pub n: Option<u32>,
    /// Image size
    pub size: Option<String>,
    /// Response format
    pub response_format: Option<String>,
    /// Quality
    pub quality: Option<String>,
    /// Style
    pub style: Option<String>,
    /// User identifier
    pub user: Option<String>,
}

/// Audio transcription request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioTranscriptionRequest {
    /// Model to use
    pub model: String,
    /// Audio file (base64 encoded)
    pub file: String,
    /// Language
    pub language: Option<String>,
    /// Prompt
    pub prompt: Option<String>,
    /// Response format
    pub response_format: Option<String>,
    /// Temperature
    pub temperature: Option<f32>,
    /// Timestamp granularities
    pub timestamp_granularities: Option<Vec<String>>,
}

/// Moderation request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModerationRequest {
    /// Model to use
    pub model: Option<String>,
    /// Input text(s)
    pub input: ModerationInput,
}

/// Moderation input
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ModerationInput {
    /// Single text string
    String(String),
    /// Array of text strings
    Array(Vec<String>),
}

/// Rerank request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RerankRequest {
    /// Model to use
    pub model: String,
    /// Query
    pub query: String,
    /// Documents to rerank
    pub documents: Vec<RerankDocument>,
    /// Top K results
    pub top_k: Option<u32>,
    /// Return documents
    pub return_documents: Option<bool>,
    /// Maximum chunks per document
    pub max_chunks_per_doc: Option<u32>,
}

/// Rerank document
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum RerankDocument {
    /// Document as plain text
    String(String),
    /// Document as object with text field
    Object {
        /// Document text content
        text: String,
    },
}

/// Routing preferences
#[derive(Debug, Clone, Default)]
pub struct RoutingPreferences {
    /// Preferred providers (in order)
    pub preferred_providers: Vec<String>,
    /// Excluded providers
    pub excluded_providers: Vec<String>,
    /// Routing strategy override
    pub strategy_override: Option<String>,
    /// Tags for tag-based routing
    pub tags: Vec<String>,
    /// Region preference
    pub region: Option<String>,
    /// Cost optimization preference
    pub optimize_cost: bool,
    /// Latency optimization preference
    pub optimize_latency: bool,
}

/// Caching preferences
#[derive(Debug, Clone, Default)]
pub struct CachingPreferences {
    /// Enable caching for this request
    pub enabled: bool,
    /// Cache TTL override
    pub ttl_seconds: Option<u64>,
    /// Cache key prefix
    pub key_prefix: Option<String>,
    /// Enable semantic caching
    pub semantic_cache: bool,
    /// Semantic similarity threshold
    pub similarity_threshold: Option<f32>,
    /// Cache tags
    pub tags: Vec<String>,
}

impl GatewayRequest {
    /// Create a new gateway request
    pub fn new(request_type: RequestType, data: RequestData, context: RequestContext) -> Self {
        Self {
            metadata: Metadata::new(),
            context,
            request_type,
            data,
            provider_params: HashMap::new(),
            routing: RoutingPreferences::default(),
            caching: CachingPreferences::default(),
        }
    }

    /// Get the model name from the request
    pub fn model(&self) -> Option<&str> {
        match &self.data {
            RequestData::ChatCompletion(req) => Some(&req.model),
            RequestData::Completion(req) => Some(&req.model),
            RequestData::Embedding(req) => Some(&req.model),
            RequestData::ImageGeneration(req) => req.model.as_deref(),
            RequestData::AudioTranscription(req) => Some(&req.model),
            RequestData::Moderation(req) => req.model.as_deref(),
            RequestData::Rerank(req) => Some(&req.model),
        }
    }

    /// Check if the request is streaming
    pub fn is_streaming(&self) -> bool {
        match &self.data {
            RequestData::ChatCompletion(req) => req.stream.unwrap_or(false),
            RequestData::Completion(req) => req.stream.unwrap_or(false),
            _ => false,
        }
    }

    /// Get estimated token count for the request
    pub fn estimated_tokens(&self) -> Option<u32> {
        // This would be implemented with actual token counting logic
        // For now, return None
        None
    }

    /// Set provider parameter
    pub fn set_provider_param<K: Into<String>, V: Into<serde_json::Value>>(
        &mut self,
        key: K,
        value: V,
    ) {
        self.provider_params.insert(key.into(), value.into());
    }

    /// Get provider parameter
    pub fn get_provider_param(&self, key: &str) -> Option<&serde_json::Value> {
        self.provider_params.get(key)
    }

    /// Set routing preferences
    pub fn with_routing(mut self, routing: RoutingPreferences) -> Self {
        self.routing = routing;
        self
    }

    /// Set caching preferences
    pub fn with_caching(mut self, caching: CachingPreferences) -> Self {
        self.caching = caching;
        self
    }

    /// Add preferred provider
    pub fn add_preferred_provider<S: Into<String>>(mut self, provider: S) -> Self {
        self.routing.preferred_providers.push(provider.into());
        self
    }

    /// Exclude provider
    pub fn exclude_provider<S: Into<String>>(mut self, provider: S) -> Self {
        self.routing.excluded_providers.push(provider.into());
        self
    }

    /// Enable caching
    pub fn enable_caching(mut self, ttl_seconds: Option<u64>) -> Self {
        self.caching.enabled = true;
        self.caching.ttl_seconds = ttl_seconds;
        self
    }
}

#[cfg(test)]
#[path = "request_tests.rs"]
mod tests;
