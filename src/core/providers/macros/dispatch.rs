//! Unified provider dispatch macro
//!
//! Eliminates repetitive match statements across provider enum variants.

/// Macro for unified provider dispatch that eliminates repetitive match statements
#[macro_export]
macro_rules! dispatch_all_providers {
    // For async methods returning Result with error conversion
    ($self:expr, async $method:ident($($arg:expr),*)) => {
        match $self {
            Provider::OpenAI(p) => LLMProvider::$method(p, $($arg),*).await.map_err(ProviderError::from),
            Provider::Anthropic(p) => LLMProvider::$method(p, $($arg),*).await.map_err(ProviderError::from),
            Provider::Azure(p) => LLMProvider::$method(p, $($arg),*).await.map_err(ProviderError::from),
            Provider::Mistral(p) => LLMProvider::$method(p, $($arg),*).await.map_err(ProviderError::from),
            Provider::DeepSeek(p) => LLMProvider::$method(p, $($arg),*).await.map_err(ProviderError::from),
            Provider::Moonshot(p) => LLMProvider::$method(p, $($arg),*).await.map_err(ProviderError::from),
            Provider::MetaLlama(p) => LLMProvider::$method(p, $($arg),*).await.map_err(ProviderError::from),
            Provider::OpenRouter(p) => LLMProvider::$method(p, $($arg),*).await.map_err(ProviderError::from),
            Provider::VertexAI(p) => LLMProvider::$method(p, $($arg),*).await.map_err(ProviderError::from),
            Provider::V0(p) => LLMProvider::$method(p, $($arg),*).await.map_err(ProviderError::from),
            Provider::DeepInfra(p) => LLMProvider::$method(p, $($arg),*).await.map_err(ProviderError::from),
            Provider::AzureAI(p) => LLMProvider::$method(p, $($arg),*).await.map_err(ProviderError::from),
        }
    };

    // For sync methods returning values directly
    ($self:expr, sync $method:ident($($arg:expr),*)) => {
        match $self {
            Provider::OpenAI(p) => LLMProvider::$method(p, $($arg),*),
            Provider::Anthropic(p) => LLMProvider::$method(p, $($arg),*),
            Provider::Azure(p) => LLMProvider::$method(p, $($arg),*),
            Provider::Mistral(p) => LLMProvider::$method(p, $($arg),*),
            Provider::DeepSeek(p) => LLMProvider::$method(p, $($arg),*),
            Provider::Moonshot(p) => LLMProvider::$method(p, $($arg),*),
            Provider::MetaLlama(p) => LLMProvider::$method(p, $($arg),*),
            Provider::OpenRouter(p) => LLMProvider::$method(p, $($arg),*),
            Provider::VertexAI(p) => LLMProvider::$method(p, $($arg),*),
            Provider::V0(p) => LLMProvider::$method(p, $($arg),*),
            Provider::DeepInfra(p) => LLMProvider::$method(p, $($arg),*),
            Provider::AzureAI(p) => LLMProvider::$method(p, $($arg),*),
        }
    };

    // For async methods without result conversion
    ($self:expr, async_direct $method:ident($($arg:expr),*)) => {
        match $self {
            Provider::OpenAI(p) => LLMProvider::$method(p, $($arg),*).await,
            Provider::Anthropic(p) => LLMProvider::$method(p, $($arg),*).await,
            Provider::Azure(p) => LLMProvider::$method(p, $($arg),*).await,
            Provider::Mistral(p) => LLMProvider::$method(p, $($arg),*).await,
            Provider::DeepSeek(p) => LLMProvider::$method(p, $($arg),*).await,
            Provider::Moonshot(p) => LLMProvider::$method(p, $($arg),*).await,
            Provider::MetaLlama(p) => LLMProvider::$method(p, $($arg),*).await,
            Provider::OpenRouter(p) => LLMProvider::$method(p, $($arg),*).await,
            Provider::VertexAI(p) => LLMProvider::$method(p, $($arg),*).await,
            Provider::V0(p) => LLMProvider::$method(p, $($arg),*).await,
            Provider::DeepInfra(p) => LLMProvider::$method(p, $($arg),*).await,
            Provider::AzureAI(p) => LLMProvider::$method(p, $($arg),*).await,
        }
    };
}
