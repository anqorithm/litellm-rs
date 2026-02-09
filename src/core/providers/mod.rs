//! AI Provider implementations using Rust-idiomatic enum-based design
//!
//! This module contains the unified Provider enum and all provider implementations.

// Base infrastructure
pub mod base;

// Provider modules - alphabetically ordered
pub mod ai21;
pub mod aiml_api;
pub mod aleph_alpha;
pub mod amazon_nova;
pub mod anthropic;
pub mod anyscale;
pub mod azure;
pub mod azure_ai;
pub mod baichuan;
pub mod baseten;
pub mod bedrock;
pub mod bytez;
pub mod cerebras;
pub mod clarifai;
pub mod cloudflare;
pub mod codestral;
pub mod cohere;
pub mod comet_api;
pub mod compactifai;
pub mod custom_api;
pub mod dashscope;
pub mod databricks;
pub mod datarobot;
pub mod deepgram;
pub mod deepinfra;
pub mod deepl;
pub mod deepseek;
pub mod docker_model_runner;
pub mod elevenlabs;
pub mod empower;
pub mod exa_ai;
pub mod fal_ai;
pub mod featherless;
pub mod firecrawl;
pub mod fireworks;
pub mod friendliai;
pub mod galadriel;
pub mod gemini;
pub mod gigachat;
pub mod github;
pub mod github_copilot;
pub mod google_pse;
pub mod gradient_ai;
pub mod groq;
pub mod heroku;
pub mod hosted_vllm;
pub mod huggingface;
pub mod hyperbolic;
pub mod infinity;
pub mod jina;
pub mod lambda_ai;
pub mod langgraph;
pub mod lemonade;
pub mod linkup;
pub mod llamafile;
pub mod lm_studio;
pub mod manus;
pub mod maritalk;
pub mod meta_llama;
pub mod milvus;
pub mod minimax;
pub mod mistral;
pub mod moonshot;
pub mod morph;
pub mod nanogpt;
pub mod nebius;
pub mod nlp_cloud;
pub mod novita;
pub mod nscale;
pub mod nvidia_nim;
pub mod oci;
pub mod ollama;
pub mod oobabooga;
pub mod openai;
pub mod openai_like;
pub mod openrouter;
pub mod ovhcloud;
pub mod perplexity;
pub mod petals;
pub mod pg_vector;
pub mod poe;
pub mod predibase;
pub mod qwen;
pub mod ragflow;
pub mod recraft;
pub mod replicate;
pub mod runwayml;
pub mod sagemaker;
pub mod sambanova;
pub mod sap_ai;
pub mod searxng;
pub mod siliconflow;
pub mod snowflake;
pub mod spark;
pub mod stability;
pub mod tavily;
pub mod together;
pub mod topaz;
pub mod triton;
pub mod v0;
pub mod vercel_ai;
pub mod vertex_ai;
pub mod vllm;
pub mod volcengine;
pub mod voyage;
pub mod wandb;
pub mod watsonx;
pub mod xai;
pub mod xiaomi_mimo;
pub mod xinference;
pub mod yi;
pub mod zhipu;

// Shared utilities and architecture
pub mod capabilities;
pub mod context;
pub mod macros; // Macros for reducing boilerplate
pub mod shared; // Shared utilities for all providers // Compile-time capability verification
pub mod thinking; // Thinking/reasoning provider trait (modular)
pub mod transform; // Request/Response transformation engine // Request/Response context and metadata

// Registry and unified provider
pub mod base_provider;
pub mod contextual_error;
pub mod provider_error_conversions;
pub mod provider_registry;
pub mod unified_provider;

// Test modules (only compiled during tests)
#[cfg(test)]
mod unified_provider_tests;

// Export main types
pub use crate::core::traits::provider::llm_provider::trait_definition::LLMProvider;
use crate::core::types::responses::{
    ChatChunk, ChatResponse, EmbeddingResponse, ImageGenerationResponse,
};
use crate::core::types::{chat::ChatRequest, embedding::EmbeddingRequest, image::ImageGenerationRequest};
use crate::core::types::{context::RequestContext, model::ProviderCapability};
use chrono::{DateTime, Utc};
pub use contextual_error::ContextualError;
pub use provider_registry::ProviderRegistry;
pub use unified_provider::{ProviderError, UnifiedProviderError}; // Both for compatibility

/// Model pricing information
#[derive(Debug, Clone)]
pub struct ModelPricing {
    pub model: String,
    pub input_cost_per_1k: f64,
    pub output_cost_per_1k: f64,
    pub currency: String,
    pub updated_at: DateTime<Utc>,
}

/// Provider type enumeration
#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum ProviderType {
    OpenAI,
    Anthropic,
    Bedrock,
    OpenRouter,
    VertexAI,
    Azure,
    AzureAI,
    DeepSeek,
    DeepInfra,
    V0,
    MetaLlama,
    Mistral,
    Moonshot,
    Minimax,
    Dashscope,
    Groq,
    XAI,
    Cloudflare,
    Perplexity,
    Replicate,
    FalAI,
    AmazonNova,
    GitHub,
    GitHubCopilot,
    Hyperbolic,
    Infinity,
    Novita,
    Volcengine,
    Nebius,
    Nscale,
    PydanticAI,
    OpenAICompatible,
    Custom(String),
}

impl From<&str> for ProviderType {
    fn from(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "openai" => ProviderType::OpenAI,
            "anthropic" => ProviderType::Anthropic,
            "bedrock" | "aws-bedrock" => ProviderType::Bedrock,
            "openrouter" => ProviderType::OpenRouter,
            "vertex_ai" | "vertexai" | "vertex-ai" => ProviderType::VertexAI,
            "azure" | "azure-openai" => ProviderType::Azure,
            "azure_ai" | "azureai" | "azure-ai" => ProviderType::AzureAI,
            "deepseek" | "deep-seek" => ProviderType::DeepSeek,
            "deepinfra" | "deep-infra" => ProviderType::DeepInfra,
            "v0" => ProviderType::V0,
            "meta_llama" | "llama" | "meta-llama" => ProviderType::MetaLlama,
            "mistral" | "mistralai" => ProviderType::Mistral,
            "moonshot" | "moonshot-ai" => ProviderType::Moonshot,
            "minimax" | "minimax-ai" => ProviderType::Minimax,
            "dashscope" | "alibaba" | "qwen" | "tongyi" => ProviderType::Dashscope,
            "groq" => ProviderType::Groq,
            "xai" => ProviderType::XAI,
            "cloudflare" | "cf" | "workers-ai" => ProviderType::Cloudflare,
            "perplexity" | "perplexity-ai" | "pplx" => ProviderType::Perplexity,
            "replicate" | "replicate-ai" => ProviderType::Replicate,
            "fal_ai" | "fal-ai" | "fal" => ProviderType::FalAI,
            "amazon_nova" | "amazon-nova" | "nova" => ProviderType::AmazonNova,
            "github" | "github-models" => ProviderType::GitHub,
            "github_copilot" | "github-copilot" | "copilot" => ProviderType::GitHubCopilot,
            "hyperbolic" | "hyperbolic-ai" => ProviderType::Hyperbolic,
            "infinity" | "infinity-embedding" => ProviderType::Infinity,
            "novita" | "novita-ai" => ProviderType::Novita,
            "volcengine" | "volc" | "doubao" | "bytedance" => ProviderType::Volcengine,
            "nebius" | "nebius-ai" => ProviderType::Nebius,
            "nscale" | "nscale-ai" => ProviderType::Nscale,
            "pydantic_ai" | "pydantic-ai" | "pydantic" => ProviderType::PydanticAI,
            "openai_compatible" | "openai-compatible" => ProviderType::OpenAICompatible,
            _ => ProviderType::Custom(s.to_string()),
        }
    }
}

impl std::fmt::Display for ProviderType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProviderType::OpenAI => write!(f, "openai"),
            ProviderType::Anthropic => write!(f, "anthropic"),
            ProviderType::Bedrock => write!(f, "bedrock"),
            ProviderType::OpenRouter => write!(f, "openrouter"),
            ProviderType::VertexAI => write!(f, "vertex_ai"),
            ProviderType::Azure => write!(f, "azure"),
            ProviderType::AzureAI => write!(f, "azure_ai"),
            ProviderType::DeepSeek => write!(f, "deepseek"),
            ProviderType::DeepInfra => write!(f, "deepinfra"),
            ProviderType::V0 => write!(f, "v0"),
            ProviderType::MetaLlama => write!(f, "meta_llama"),
            ProviderType::Mistral => write!(f, "mistral"),
            ProviderType::Moonshot => write!(f, "moonshot"),
            ProviderType::Minimax => write!(f, "minimax"),
            ProviderType::Dashscope => write!(f, "dashscope"),
            ProviderType::Groq => write!(f, "groq"),
            ProviderType::XAI => write!(f, "xai"),
            ProviderType::Cloudflare => write!(f, "cloudflare"),
            ProviderType::Perplexity => write!(f, "perplexity"),
            ProviderType::Replicate => write!(f, "replicate"),
            ProviderType::FalAI => write!(f, "fal_ai"),
            ProviderType::AmazonNova => write!(f, "amazon_nova"),
            ProviderType::GitHub => write!(f, "github"),
            ProviderType::GitHubCopilot => write!(f, "github_copilot"),
            ProviderType::Hyperbolic => write!(f, "hyperbolic"),
            ProviderType::Infinity => write!(f, "infinity"),
            ProviderType::Novita => write!(f, "novita"),
            ProviderType::Volcengine => write!(f, "volcengine"),
            ProviderType::Nebius => write!(f, "nebius"),
            ProviderType::Nscale => write!(f, "nscale"),
            ProviderType::PydanticAI => write!(f, "pydantic_ai"),
            ProviderType::OpenAICompatible => write!(f, "openai_compatible"),
            ProviderType::Custom(name) => write!(f, "{}", name),
        }
    }
}

// ==================== Provider Dispatch Macros ====================
// These macros eliminate repetitive match patterns across all provider methods

/// Macro for dispatching synchronous methods to all providers
macro_rules! dispatch_provider {
    ($self:expr, $method:ident) => {
        match $self {
            Provider::OpenAI(p) => p.$method(),
            Provider::Anthropic(p) => p.$method(),
            Provider::Azure(p) => p.$method(),
            Provider::Bedrock(p) => p.$method(),
            Provider::Mistral(p) => p.$method(),
            Provider::DeepSeek(p) => p.$method(),
            Provider::Moonshot(p) => p.$method(),
            Provider::MetaLlama(p) => p.$method(),
            Provider::OpenRouter(p) => p.$method(),
            Provider::VertexAI(p) => p.$method(),
            Provider::V0(p) => p.$method(),
            Provider::DeepInfra(p) => p.$method(),
            Provider::AzureAI(p) => p.$method(),
            Provider::Groq(p) => p.$method(),
            Provider::XAI(p) => p.$method(),
            Provider::Cloudflare(p) => p.$method(),
        }
    };

    ($self:expr, $method:ident, $($arg:expr),+) => {
        match $self {
            Provider::OpenAI(p) => p.$method($($arg),+),
            Provider::Anthropic(p) => p.$method($($arg),+),
            Provider::Azure(p) => p.$method($($arg),+),
            Provider::Bedrock(p) => p.$method($($arg),+),
            Provider::Mistral(p) => p.$method($($arg),+),
            Provider::DeepSeek(p) => p.$method($($arg),+),
            Provider::Moonshot(p) => p.$method($($arg),+),
            Provider::MetaLlama(p) => p.$method($($arg),+),
            Provider::OpenRouter(p) => p.$method($($arg),+),
            Provider::VertexAI(p) => p.$method($($arg),+),
            Provider::V0(p) => p.$method($($arg),+),
            Provider::DeepInfra(p) => p.$method($($arg),+),
            Provider::AzureAI(p) => p.$method($($arg),+),
            Provider::Groq(p) => p.$method($($arg),+),
            Provider::XAI(p) => p.$method($($arg),+),
            Provider::Cloudflare(p) => p.$method($($arg),+),
        }
    };
}

/// Macro for dispatching async methods with unified error conversion
macro_rules! dispatch_provider_async {
    ($self:expr, $method:ident, $($arg:expr),*) => {
        match $self {
            Provider::OpenAI(p) => LLMProvider::$method(p, $($arg),*).await.map_err(ProviderError::from),
            Provider::Anthropic(p) => LLMProvider::$method(p, $($arg),*).await.map_err(ProviderError::from),
            Provider::Azure(p) => LLMProvider::$method(p, $($arg),*).await.map_err(ProviderError::from),
            Provider::Bedrock(p) => LLMProvider::$method(p, $($arg),*).await.map_err(ProviderError::from),
            Provider::Mistral(p) => LLMProvider::$method(p, $($arg),*).await.map_err(ProviderError::from),
            Provider::DeepSeek(p) => LLMProvider::$method(p, $($arg),*).await.map_err(ProviderError::from),
            Provider::Moonshot(p) => LLMProvider::$method(p, $($arg),*).await.map_err(ProviderError::from),
            Provider::MetaLlama(p) => LLMProvider::$method(p, $($arg),*).await.map_err(ProviderError::from),
            Provider::OpenRouter(p) => LLMProvider::$method(p, $($arg),*).await.map_err(ProviderError::from),
            Provider::VertexAI(p) => LLMProvider::$method(p, $($arg),*).await.map_err(ProviderError::from),
            Provider::V0(p) => LLMProvider::$method(p, $($arg),*).await.map_err(ProviderError::from),
            Provider::DeepInfra(p) => LLMProvider::$method(p, $($arg),*).await.map_err(ProviderError::from),
            Provider::AzureAI(p) => LLMProvider::$method(p, $($arg),*).await.map_err(ProviderError::from),
            Provider::Groq(p) => LLMProvider::$method(p, $($arg),*).await.map_err(ProviderError::from),
            Provider::XAI(p) => LLMProvider::$method(p, $($arg),*).await.map_err(ProviderError::from),
            Provider::Cloudflare(p) => LLMProvider::$method(p, $($arg),*).await.map_err(ProviderError::from),
        }
    };
}

/// Macro for dispatching methods that return values directly (no Result)
macro_rules! dispatch_provider_value {
    ($self:expr, $method:ident) => {
        match $self {
            Provider::OpenAI(p) => LLMProvider::$method(p),
            Provider::Anthropic(p) => LLMProvider::$method(p),
            Provider::Azure(p) => LLMProvider::$method(p),
            Provider::Bedrock(p) => LLMProvider::$method(p),
            Provider::Mistral(p) => LLMProvider::$method(p),
            Provider::DeepSeek(p) => LLMProvider::$method(p),
            Provider::Moonshot(p) => LLMProvider::$method(p),
            Provider::MetaLlama(p) => LLMProvider::$method(p),
            Provider::OpenRouter(p) => LLMProvider::$method(p),
            Provider::VertexAI(p) => LLMProvider::$method(p),
            Provider::V0(p) => LLMProvider::$method(p),
            Provider::DeepInfra(p) => LLMProvider::$method(p),
            Provider::AzureAI(p) => LLMProvider::$method(p),
            Provider::Groq(p) => LLMProvider::$method(p),
            Provider::XAI(p) => LLMProvider::$method(p),
            Provider::Cloudflare(p) => LLMProvider::$method(p),
        }
    };

    ($self:expr, $method:ident, $($arg:expr),+) => {
        match $self {
            Provider::OpenAI(p) => LLMProvider::$method(p, $($arg),+),
            Provider::Anthropic(p) => LLMProvider::$method(p, $($arg),+),
            Provider::Azure(p) => LLMProvider::$method(p, $($arg),+),
            Provider::Bedrock(p) => LLMProvider::$method(p, $($arg),+),
            Provider::Mistral(p) => LLMProvider::$method(p, $($arg),+),
            Provider::DeepSeek(p) => LLMProvider::$method(p, $($arg),+),
            Provider::Moonshot(p) => LLMProvider::$method(p, $($arg),+),
            Provider::MetaLlama(p) => LLMProvider::$method(p, $($arg),+),
            Provider::OpenRouter(p) => LLMProvider::$method(p, $($arg),+),
            Provider::VertexAI(p) => LLMProvider::$method(p, $($arg),+),
            Provider::V0(p) => LLMProvider::$method(p, $($arg),+),
            Provider::DeepInfra(p) => LLMProvider::$method(p, $($arg),+),
            Provider::AzureAI(p) => LLMProvider::$method(p, $($arg),+),
            Provider::Groq(p) => LLMProvider::$method(p, $($arg),+),
            Provider::XAI(p) => LLMProvider::$method(p, $($arg),+),
            Provider::Cloudflare(p) => LLMProvider::$method(p, $($arg),+),
        }
    };
}

/// Macro for selective provider dispatch with default fallback
/// Use this when only some providers support a method
#[allow(unused_macros)]
macro_rules! dispatch_provider_selective {
    // Dispatch to specific providers only, with a default for others
    ($self:expr, $method:ident, { $($provider:ident),+ }, $default:expr) => {
        match $self {
            $(Provider::$provider(p) => p.$method()),+,
            _ => $default,
        }
    };

    ($self:expr, $method:ident($($arg:expr),+), { $($provider:ident),+ }, $default:expr) => {
        match $self {
            $(Provider::$provider(p) => p.$method($($arg),+)),+,
            _ => $default,
        }
    };
}

/// Macro for dispatching async methods without error transformation
macro_rules! dispatch_provider_async_direct {
    ($self:expr, $method:ident) => {
        match $self {
            Provider::OpenAI(p) => LLMProvider::$method(p).await,
            Provider::Anthropic(p) => LLMProvider::$method(p).await,
            Provider::Azure(p) => LLMProvider::$method(p).await,
            Provider::Bedrock(p) => LLMProvider::$method(p).await,
            Provider::Mistral(p) => LLMProvider::$method(p).await,
            Provider::DeepSeek(p) => LLMProvider::$method(p).await,
            Provider::Moonshot(p) => LLMProvider::$method(p).await,
            Provider::MetaLlama(p) => LLMProvider::$method(p).await,
            Provider::OpenRouter(p) => LLMProvider::$method(p).await,
            Provider::VertexAI(p) => LLMProvider::$method(p).await,
            Provider::V0(p) => LLMProvider::$method(p).await,
            Provider::DeepInfra(p) => LLMProvider::$method(p).await,
            Provider::AzureAI(p) => LLMProvider::$method(p).await,
            Provider::Groq(p) => LLMProvider::$method(p).await,
            Provider::XAI(p) => LLMProvider::$method(p).await,
            Provider::Cloudflare(p) => LLMProvider::$method(p).await,
        }
    };
}

/// Unified Provider Enum (Rust-idiomatic design)
///
/// This enum provides zero-cost abstractions and type safety for all providers.
/// Each variant contains a concrete provider implementation.
#[derive(Debug, Clone)]
pub enum Provider {
    OpenAI(openai::OpenAIProvider),
    Anthropic(anthropic::AnthropicProvider),
    Azure(azure::AzureOpenAIProvider),
    Bedrock(bedrock::BedrockProvider),
    Mistral(mistral::MistralProvider),
    DeepSeek(deepseek::DeepSeekProvider),
    Moonshot(moonshot::MoonshotProvider),
    MetaLlama(meta_llama::LlamaProvider),
    OpenRouter(openrouter::OpenRouterProvider),
    VertexAI(vertex_ai::VertexAIProvider),
    V0(v0::V0Provider),
    DeepInfra(deepinfra::DeepInfraProvider),
    AzureAI(azure_ai::AzureAIProvider),
    Groq(groq::GroqProvider),
    XAI(xai::XAIProvider),
    Cloudflare(cloudflare::CloudflareProvider),
}

impl Provider {
    /// Get provider name
    pub fn name(&self) -> &'static str {
        match self {
            Provider::OpenAI(_) => "openai",
            Provider::Anthropic(_) => "anthropic",
            Provider::Azure(_) => "azure",
            Provider::Bedrock(_) => "bedrock",
            Provider::Mistral(_) => "mistral",
            Provider::DeepSeek(_) => "deepseek",
            Provider::Moonshot(_) => "moonshot",
            Provider::MetaLlama(_) => "meta_llama",
            Provider::OpenRouter(_) => "openrouter",
            Provider::VertexAI(_) => "vertex_ai",
            Provider::V0(_) => "v0",
            Provider::DeepInfra(_) => "deepinfra",
            Provider::AzureAI(_) => "azure_ai",
            Provider::Groq(_) => "groq",
            Provider::XAI(_) => "xai",
            Provider::Cloudflare(_) => "cloudflare",
        }
    }

    /// Get provider type
    pub fn provider_type(&self) -> ProviderType {
        match self {
            Provider::OpenAI(_) => ProviderType::OpenAI,
            Provider::Anthropic(_) => ProviderType::Anthropic,
            Provider::Azure(_) => ProviderType::Azure,
            Provider::Bedrock(_) => ProviderType::Bedrock,
            Provider::Mistral(_) => ProviderType::Mistral,
            Provider::DeepSeek(_) => ProviderType::DeepSeek,
            Provider::Moonshot(_) => ProviderType::Moonshot,
            Provider::MetaLlama(_) => ProviderType::MetaLlama,
            Provider::OpenRouter(_) => ProviderType::OpenRouter,
            Provider::VertexAI(_) => ProviderType::VertexAI,
            Provider::V0(_) => ProviderType::V0,
            Provider::DeepInfra(_) => ProviderType::DeepInfra,
            Provider::AzureAI(_) => ProviderType::AzureAI,
            Provider::Groq(_) => ProviderType::Groq,
            Provider::XAI(_) => ProviderType::XAI,
            Provider::Cloudflare(_) => ProviderType::Cloudflare,
        }
    }

    /// Check if provider supports a specific model
    pub fn supports_model(&self, model: &str) -> bool {
        use crate::core::traits::provider::llm_provider::trait_definition::LLMProvider;
        dispatch_provider_value!(self, supports_model, model)
    }

    /// Get provider capabilities
    pub fn capabilities(&self) -> &'static [ProviderCapability] {
        // All providers implement capabilities, using generic macro
        dispatch_provider!(self, capabilities)

        // But if future providers don't implement it, can change to:
        // dispatch_provider_selective!(
        //     self,
        //     capabilities,
        //     { OpenAI, Anthropic, Azure, Mistral, Moonshot, V0 },
        //     &[ProviderCapability::ChatCompletion]  // Default capability
        // )
    }

    /// Execute chat completion
    pub async fn chat_completion(
        &self,
        request: ChatRequest,
        context: RequestContext,
    ) -> Result<ChatResponse, UnifiedProviderError> {
        use crate::core::traits::provider::llm_provider::trait_definition::LLMProvider;
        dispatch_provider_async!(self, chat_completion, request, context)
    }

    /// Execute health check
    pub async fn health_check(&self) -> crate::core::types::health::HealthStatus {
        use crate::core::traits::provider::llm_provider::trait_definition::LLMProvider;
        dispatch_provider_async_direct!(self, health_check)
    }

    /// List available models
    pub fn list_models(&self) -> &[crate::core::types::model::ModelInfo] {
        use crate::core::traits::provider::llm_provider::trait_definition::LLMProvider;
        dispatch_provider_value!(self, models)
    }

    /// Calculate cost using unified pricing database
    pub async fn calculate_cost(
        &self,
        model: &str,
        input_tokens: u32,
        output_tokens: u32,
    ) -> Result<f64, UnifiedProviderError> {
        // Use unified pricing database instead of each provider implementing its own
        let usage = crate::core::providers::base::pricing::Usage {
            prompt_tokens: input_tokens,
            completion_tokens: output_tokens,
            total_tokens: input_tokens + output_tokens,
            reasoning_tokens: None,
        };

        Ok(crate::core::providers::base::get_pricing_db().calculate(model, &usage))
    }

    /// Execute streaming chat completion
    pub async fn chat_completion_stream(
        &self,
        request: ChatRequest,
        context: RequestContext,
    ) -> Result<
        std::pin::Pin<
            Box<
                dyn futures::Stream<Item = Result<ChatChunk, UnifiedProviderError>>
                    + Send
                    + 'static,
            >,
        >,
        UnifiedProviderError,
    > {
        use crate::core::traits::provider::llm_provider::trait_definition::LLMProvider;
        use futures::StreamExt;

        match self {
            Provider::OpenAI(p) => {
                let stream = LLMProvider::chat_completion_stream(p, request, context).await?;
                let mapped = stream.map(|result| result);
                Ok(Box::pin(mapped))
            }
            Provider::Anthropic(p) => {
                let stream = LLMProvider::chat_completion_stream(p, request, context).await?;
                let mapped = stream.map(|result| result);
                Ok(Box::pin(mapped))
            }
            Provider::DeepInfra(p) => {
                let stream = LLMProvider::chat_completion_stream(p, request, context).await?;
                let mapped = stream.map(|result| result);
                Ok(Box::pin(mapped))
            }
            Provider::AzureAI(p) => {
                let stream = LLMProvider::chat_completion_stream(p, request, context).await?;
                let mapped = stream.map(|result| result);
                Ok(Box::pin(mapped))
            }
            Provider::Groq(p) => {
                let stream = LLMProvider::chat_completion_stream(p, request, context).await?;
                let mapped = stream.map(|result| result);
                Ok(Box::pin(mapped))
            }
            _ => Err(UnifiedProviderError::not_implemented(
                "unknown",
                format!("Streaming not implemented for {}", self.name()),
            )),
        }
    }

    /// Create embeddings
    pub async fn create_embeddings(
        &self,
        request: EmbeddingRequest,
        context: RequestContext,
    ) -> Result<EmbeddingResponse, UnifiedProviderError> {
        use crate::core::traits::provider::llm_provider::trait_definition::LLMProvider;

        match self {
            Provider::OpenAI(p) => LLMProvider::embeddings(p, request, context).await,
            Provider::Azure(p) => LLMProvider::embeddings(p, request, context).await,
            _ => Err(UnifiedProviderError::not_implemented(
                "unknown",
                format!("Embeddings not supported by {}", self.name()),
            )),
        }
    }

    /// Create images
    pub async fn create_images(
        &self,
        request: ImageGenerationRequest,
        context: RequestContext,
    ) -> Result<ImageGenerationResponse, UnifiedProviderError> {
        use crate::core::traits::provider::llm_provider::trait_definition::LLMProvider;

        match self {
            Provider::OpenAI(p) => LLMProvider::image_generation(p, request, context).await,
            _ => Err(UnifiedProviderError::not_implemented(
                "unknown",
                format!("Image generation not supported by {}", self.name()),
            )),
        }
    }

    /// Get model information by ID
    pub async fn get_model(
        &self,
        model_id: &str,
    ) -> Result<Option<crate::core::types::model::ModelInfo>, UnifiedProviderError> {
        // Look through available models for this provider
        let models = self.list_models();
        for model in models {
            if model.id == model_id || model.name == model_id {
                return Ok(Some(model.clone()));
            }
        }

        // Model not found in this provider
        Ok(None)
    }
}

/// Create a provider from configuration
///
/// This is the main factory function for creating providers
pub async fn create_provider(
    config: crate::config::models::provider::ProviderConfig,
) -> Result<Provider, ProviderError> {
    // Determine provider type from config
    let provider_type = match config.name.as_str() {
        "openai" => ProviderType::OpenAI,
        "anthropic" => ProviderType::Anthropic,
        "azure" => ProviderType::Azure,
        "mistral" => ProviderType::Mistral,
        "deepseek" => ProviderType::DeepSeek,
        "moonshot" => ProviderType::Moonshot,
        "meta_llama" => ProviderType::MetaLlama,
        "openrouter" => ProviderType::OpenRouter,
        "vertex_ai" => ProviderType::VertexAI,
        "v0" => ProviderType::V0,
        name => {
            return Err(ProviderError::not_implemented(
                "unknown",
                format!("Unknown provider: {}", name),
            ));
        }
    };

    // For now, return a placeholder error until all providers are properly configured
    Err(ProviderError::not_implemented(
        "unknown",
        format!(
            "Provider factory for {:?} not yet fully implemented",
            provider_type
        ),
    ))
}

// Provider factory functions
impl Provider {
    /// Create provider from configuration asynchronously
    ///
    /// This is the preferred method for creating providers from configuration.
    /// It supports all provider types and handles async initialization properly.
    pub async fn from_config_async(
        provider_type: ProviderType,
        config: serde_json::Value,
    ) -> Result<Self, ProviderError> {
        match provider_type {
            ProviderType::OpenAI => {
                let api_key = macros::require_config_str(&config, "api_key", "openai")?;
                let provider = openai::OpenAIProvider::with_api_key(api_key)
                    .await
                    .map_err(|e| ProviderError::initialization("openai", e.to_string()))?;
                Ok(Provider::OpenAI(provider))
            }
            ProviderType::Anthropic => {
                let api_key = macros::require_config_str(&config, "api_key", "anthropic")?;
                let provider = anthropic::AnthropicProvider::new(
                    anthropic::AnthropicConfig::default().with_api_key(api_key),
                )?;
                Ok(Provider::Anthropic(provider))
            }
            ProviderType::Groq => {
                let api_key = macros::require_config_str(&config, "api_key", "groq")?;
                let provider = groq::GroqProvider::with_api_key(api_key)
                    .await
                    .map_err(|e| ProviderError::initialization("groq", e.to_string()))?;
                Ok(Provider::Groq(provider))
            }
            ProviderType::XAI => {
                let api_key = macros::require_config_str(&config, "api_key", "xai")?;
                let provider = xai::XAIProvider::with_api_key(api_key)
                    .await
                    .map_err(|e| ProviderError::initialization("xai", e.to_string()))?;
                Ok(Provider::XAI(provider))
            }
            ProviderType::OpenRouter => {
                let api_key = macros::require_config_str(&config, "api_key", "openrouter")?;
                let or_config = openrouter::OpenRouterConfig::new(api_key);
                let provider = openrouter::OpenRouterProvider::new(or_config)?;
                Ok(Provider::OpenRouter(provider))
            }
            ProviderType::Mistral => {
                let api_key = macros::require_config_str(&config, "api_key", "mistral")?;
                let mistral_config = mistral::MistralConfig {
                    api_key: api_key.to_string(),
                    ..Default::default()
                };
                let provider = mistral::MistralProvider::new(mistral_config)
                    .await
                    .map_err(|e| ProviderError::initialization("mistral", e.to_string()))?;
                Ok(Provider::Mistral(provider))
            }
            ProviderType::DeepSeek => {
                let api_key = macros::require_config_str(&config, "api_key", "deepseek")?;
                let mut ds_config = deepseek::DeepSeekConfig::new("deepseek");
                ds_config.base.api_key = Some(api_key.to_string());
                let provider = deepseek::DeepSeekProvider::new(ds_config)?;
                Ok(Provider::DeepSeek(provider))
            }
            ProviderType::Moonshot => {
                let api_key = macros::require_config_str(&config, "api_key", "moonshot")?;
                let moonshot_config = moonshot::MoonshotConfig {
                    api_key: api_key.to_string(),
                    ..Default::default()
                };
                let provider = moonshot::MoonshotProvider::new(moonshot_config)
                    .await
                    .map_err(|e| ProviderError::initialization("moonshot", e.to_string()))?;
                Ok(Provider::Moonshot(provider))
            }
            ProviderType::Cloudflare => {
                let account_id = macros::require_config_str(&config, "account_id", "cloudflare")?;
                let api_token = macros::require_config_str(&config, "api_token", "cloudflare")?;
                let cf_config = cloudflare::CloudflareConfig {
                    account_id: Some(account_id.to_string()),
                    api_token: Some(api_token.to_string()),
                    ..Default::default()
                };
                let provider = cloudflare::CloudflareProvider::new(cf_config)
                    .await
                    .map_err(|e| ProviderError::initialization("cloudflare", e.to_string()))?;
                Ok(Provider::Cloudflare(provider))
            }
            _ => Err(ProviderError::not_implemented(
                "unknown",
                format!("Factory for {:?} not yet implemented", provider_type),
            )),
        }
    }
}

// ==================== Unit Tests ====================


#[cfg(test)]
#[path = "mod_tests.rs"]
mod tests;
