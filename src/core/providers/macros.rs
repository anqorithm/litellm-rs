//! Macros for provider implementation
//!
//! These macros help reduce boilerplate code when implementing providers,
//! following Rust's principle of zero-cost abstractions.

use crate::core::providers::unified_provider::ProviderError;

// ==================== Configuration Extraction Helpers ====================
// These helpers reduce boilerplate for extracting required/optional config values

/// Extract a required string value from configuration JSON
///
/// # Example
/// ```rust
/// # use litellm_rs::core::providers::macros::require_config_str;
/// # fn example() -> Result<(), litellm_rs::ProviderError> {
/// let config = serde_json::json!({"api_key": "sk-123"});
/// let api_key = require_config_str(&config, "api_key", "openai")?;
/// assert_eq!(api_key, "sk-123");
/// # Ok(())
/// # }
/// ```
#[inline]
pub fn require_config_str<'a>(
    config: &'a serde_json::Value,
    key: &str,
    provider: &'static str,
) -> Result<&'a str, ProviderError> {
    config
        .get(key)
        .and_then(|v| v.as_str())
        .ok_or_else(|| ProviderError::configuration(provider, format!("{} is required", key)))
}

/// Extract an optional string value from configuration JSON
#[inline]
pub fn get_config_str<'a>(config: &'a serde_json::Value, key: &str) -> Option<&'a str> {
    config.get(key).and_then(|v| v.as_str())
}

/// Extract a required u64 value from configuration JSON
#[inline]
pub fn require_config_u64(
    config: &serde_json::Value,
    key: &str,
    provider: &'static str,
) -> Result<u64, ProviderError> {
    config
        .get(key)
        .and_then(|v| v.as_u64())
        .ok_or_else(|| ProviderError::configuration(provider, format!("{} is required", key)))
}

/// Extract an optional u64 value from configuration JSON with a default
#[inline]
pub fn get_config_u64_or(config: &serde_json::Value, key: &str, default: u64) -> u64 {
    config.get(key).and_then(|v| v.as_u64()).unwrap_or(default)
}

/// Extract a required bool value from configuration JSON
#[inline]
pub fn require_config_bool(
    config: &serde_json::Value,
    key: &str,
    provider: &'static str,
) -> Result<bool, ProviderError> {
    config
        .get(key)
        .and_then(|v| v.as_bool())
        .ok_or_else(|| ProviderError::configuration(provider, format!("{} is required", key)))
}

/// Extract an optional bool value from configuration JSON with a default
#[inline]
pub fn get_config_bool_or(config: &serde_json::Value, key: &str, default: bool) -> bool {
    config.get(key).and_then(|v| v.as_bool()).unwrap_or(default)
}

/// Macro to extract required configuration value with provider context
///
/// # Example
/// ```rust
/// # use litellm_rs::require_config;
/// # fn example() -> Result<(), litellm_rs::ProviderError> {
/// let config = serde_json::json!({"api_key": "sk-123", "timeout": 30});
/// // Extract required string
/// let api_key = require_config!(&config, "api_key", str, "openai")?;
///
/// // Extract required u64
/// let timeout = require_config!(&config, "timeout", u64, "openai")?;
/// # Ok(())
/// # }
/// ```
#[macro_export]
macro_rules! require_config {
    ($config:expr, $key:literal, str, $provider:literal) => {
        $crate::core::providers::macros::require_config_str($config, $key, $provider)
    };
    ($config:expr, $key:literal, u64, $provider:literal) => {
        $crate::core::providers::macros::require_config_u64($config, $key, $provider)
    };
    ($config:expr, $key:literal, bool, $provider:literal) => {
        $crate::core::providers::macros::require_config_bool($config, $key, $provider)
    };
}

/// Macro to implement common provider methods
#[macro_export]
macro_rules! impl_provider_basics {
    ($provider_type:ty, $name:literal, $capabilities:expr) => {
        impl $provider_type {
            pub fn name(&self) -> &str {
                $name
            }

            pub fn capabilities(&self) -> &[ProviderCapability] {
                $capabilities
            }
        }
    };
}

/// Macro to implement error conversion for all provider errors
/// This eliminates the 15 repetitive From implementations
#[macro_export]
macro_rules! impl_error_conversion {
    // Generate all error conversions for a provider
    ($($provider_error:ty => $provider_name:expr),+ $(,)?) => {
        $(
            impl From<$provider_error> for ProviderError {
                fn from(error: $provider_error) -> Self {
                    use $crate::core::types::errors::ProviderErrorTrait;

                    match error.error_type() {
                        $crate::core::types::errors::ErrorType::Authentication => {
                            ProviderError::Authentication {
                                provider: $provider_name,
                                message: error.to_string(),
                            }
                        }
                        $crate::core::types::errors::ErrorType::RateLimit => {
                            ProviderError::RateLimit {
                                provider: $provider_name,
                                message: error.to_string(),
                                retry_after: error.retry_after(),
                            }
                        }
                        $crate::core::types::errors::ErrorType::InvalidRequest => {
                            ProviderError::InvalidRequest {
                                provider: $provider_name,
                                message: error.to_string(),
                            }
                        }
                        $crate::core::types::errors::ErrorType::ModelNotFound => {
                            ProviderError::ModelNotFound {
                                provider: $provider_name,
                                model: error.model().unwrap_or("").to_string(),
                                message: error.to_string(),
                            }
                        }
                        $crate::core::types::errors::ErrorType::ServiceUnavailable => {
                            ProviderError::ServiceUnavailable {
                                provider: $provider_name,
                                message: error.to_string(),
                            }
                        }
                        $crate::core::types::errors::ErrorType::Timeout => {
                            ProviderError::Timeout {
                                provider: $provider_name,
                                message: error.to_string(),
                            }
                        }
                        $crate::core::types::errors::ErrorType::Network => {
                            ProviderError::Network {
                                provider: $provider_name,
                                message: error.to_string(),
                            }
                        }
                        $crate::core::types::errors::ErrorType::Serialization => {
                            ProviderError::Serialization {
                                provider: $provider_name,
                                message: error.to_string(),
                            }
                        }
                        $crate::core::types::errors::ErrorType::NotSupported => {
                            ProviderError::NotSupported {
                                provider: $provider_name,
                                feature: error.to_string(),
                            }
                        }
                        _ => ProviderError::Other {
                            provider: $provider_name,
                            message: error.to_string(),
                        }
                    }
                }
            }
        )+
    };

    // Simplified version for common library errors
    (standard: $provider_name:literal) => {
        impl From<reqwest::Error> for ProviderError {
            fn from(err: reqwest::Error) -> Self {
                ProviderError::Network {
                    provider: $provider_name,
                    message: err.to_string(),
                }
            }
        }

        impl From<serde_json::Error> for ProviderError {
            fn from(err: serde_json::Error) -> Self {
                ProviderError::Serialization {
                    provider: $provider_name,
                    message: err.to_string(),
                }
            }
        }
    };
}

/// Macro to generate provider configuration struct
#[macro_export]
macro_rules! provider_config {
    (
        $(#[$meta:meta])*
        pub struct $name:ident {
            $(
                $(#[$field_meta:meta])*
                $field:ident: $field_type:ty $(= $default:expr)?
            ),* $(,)?
        }
    ) => {
        $(#[$meta])*
        #[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
        pub struct $name {
            $(
                $(#[$field_meta])*
                $(#[serde(default = stringify!($field))])?
                pub $field: $field_type,
            )*
        }

        impl Default for $name {
            fn default() -> Self {
                Self {
                    $(
                        $field: provider_config!(@default $field_type $(, $default)?),
                    )*
                }
            }
        }

        $(
            $(
                fn $field() -> $field_type {
                    provider_config!(@default $field_type $(, $default)?)
                }
            )?
        )*
    };

    (@default $field_type:ty) => {
        <$field_type>::default()
    };

    (@default $field_type:ty, $default:expr) => {
        $default
    };
}

/// Macro to implement health check using a simple API call
#[macro_export]
macro_rules! impl_health_check {
    ($provider_type:ty, $endpoint:expr) => {
        async fn check_health(&self) -> Result<HealthStatus, ProviderError> {
            let url = format!("{}/{}", self.config.base_url, $endpoint);
            let response = self
                .client
                .get(&url)
                .header("Authorization", format!("Bearer {}", self.config.api_key))
                .send()
                .await;

            match response {
                Ok(res) if res.status().is_success() => Ok(HealthStatus::Healthy),
                Ok(res) if res.status() == 401 => {
                    Ok(HealthStatus::Unhealthy("Authentication failed".to_string()))
                }
                Ok(res) => Ok(HealthStatus::Unhealthy(format!("HTTP {}", res.status()))),
                Err(e) => Ok(HealthStatus::Unhealthy(format!("Connection failed: {}", e))),
            }
        }
    };
}

/// Macro to implement standard HTTP request builder
#[macro_export]
macro_rules! build_request {
    ($self:expr, $method:ident, $url:expr) => {{
        $self.client
            .$method($url)
            .header("Authorization", format!("Bearer {}", $self.config.api_key))
            .header("Content-Type", "application/json")
    }};

    ($self:expr, $method:ident, $url:expr, headers: {$($key:expr => $value:expr),* $(,)?}) => {{
        let mut request = $self.client
            .$method($url)
            .header("Authorization", format!("Bearer {}", $self.config.api_key))
            .header("Content-Type", "application/json");

        $(
            request = request.header($key, $value);
        )*

        request
    }};
}

/// Macro to implement not-implemented methods
#[macro_export]
macro_rules! not_implemented {
    ($provider:literal, $feature:literal) => {
        Err(ProviderError::NotImplemented {
            provider: $provider,
            feature: $feature.to_string(),
        })
    };
}

/// Macro to generate model list
#[macro_export]
macro_rules! model_list {
    ($provider:literal, $($model_id:literal),* $(,)?) => {
        vec![
            $(
                ModelInfo {
                    id: $model_id.to_string(),
                    name: $model_id.to_string(),
                    provider: $provider.to_string(),
                    max_context_length: 4096, // Default, should be overridden
                    max_output_length: None,
                    supports_streaming: true,
                    supports_tools: false, // Default
                    supports_multimodal: false, // Default
                    input_cost_per_1k_tokens: None,
                    output_cost_per_1k_tokens: None,
                    currency: "USD".to_string(),
                    capabilities: Vec::new(),
                    created_at: None,
                    updated_at: None,
                    metadata: std::collections::HashMap::new(),
                },
            )*
        ]
    };
}

/// Macro to implement streaming response handler
/// This eliminates the repetitive 20-line streaming handler pattern
#[macro_export]
macro_rules! impl_streaming {
    ($provider:literal, $response:expr) => {{ impl_streaming!($provider, $response, ChatChunk) }};

    ($provider:literal, $response:expr, $chunk_type:ty) => {{
        use futures::StreamExt;
        use std::pin::Pin;

        let stream = $response
            .bytes_stream()
            .map(move |chunk| {
                match chunk {
                    Ok(bytes) => {
                        let data = String::from_utf8_lossy(&bytes);

                        // Handle SSE format
                        if let Some(json_str) = data.strip_prefix("data: ") {
                            // Check for stream end
                            if json_str.trim() == "[DONE]" {
                                return Ok(None);
                            }

                            // Parse chunk
                            match serde_json::from_str::<$chunk_type>(json_str) {
                                Ok(chunk) => Ok(Some(chunk)),
                                Err(e) => Err(ProviderError::ResponseParsing {
                                    provider: $provider,
                                    message: format!("Failed to parse chunk: {}", e),
                                }),
                            }
                        } else if data.trim().is_empty() {
                            // Skip empty lines
                            Ok(None)
                        } else {
                            // Skip non-data lines (like comments)
                            Ok(None)
                        }
                    }
                    Err(e) => Err(ProviderError::Network {
                        provider: $provider,
                        message: format!("Stream error: {}", e),
                    }),
                }
            })
            .filter_map(|result| async move {
                match result {
                    Ok(Some(chunk)) => Some(Ok(chunk)),
                    Ok(None) => None,
                    Err(e) => Some(Err(e)),
                }
            });

        Ok(Box::pin(stream)
            as Pin<
                Box<dyn futures::Stream<Item = Result<$chunk_type, ProviderError>> + Send>,
            >)
    }};
}

/// Macro to validate required fields in a response
#[macro_export]
macro_rules! validate_response {
    ($response:expr, $provider:literal, required: [$($field:literal),* $(,)?]) => {{
        if !$response.is_object() {
            return Err(ProviderError::ResponseParsing {
                provider: $provider,
                message: "Response is not a JSON object".to_string(),
            });
        }

        $(
            if $response.get($field).is_none() {
                return Err(ProviderError::ResponseParsing {
                    provider: $provider,
                    message: format!("Missing required field: {}", $field),
                });
            }
        )*

        Ok(())
    }};
}

/// Macro to implement retry logic
#[macro_export]
macro_rules! with_retry {
    ($provider:literal, $max_retries:expr, $operation:expr) => {{
        let mut retries = 0;
        let mut delay = std::time::Duration::from_secs(1);

        loop {
            match $operation {
                Ok(result) => break Ok(result),
                Err(e) if retries < $max_retries => {
                    retries += 1;
                    tracing::warn!(
                        "Provider {} operation failed: {}, retrying ({}/{})",
                        $provider,
                        e,
                        retries,
                        $max_retries
                    );
                    tokio::time::sleep(delay).await;
                    delay *= 2; // Exponential backoff
                }
                Err(e) => break Err(e),
            }
        }
    }};
}

/// Macro to implement usage extraction from response
#[macro_export]
macro_rules! extract_usage {
    ($response:expr) => {{
        $response.get("usage").and_then(|u| {
            Some(Usage {
                prompt_tokens: u.get("prompt_tokens")?.as_u64()? as u32,
                completion_tokens: u.get("completion_tokens")?.as_u64()? as u32,
                total_tokens: u.get("total_tokens")?.as_u64()? as u32,
            })
        })
    }};
}

/// Macro to create standard provider implementation
#[macro_export]
macro_rules! standard_provider {
    (
        name: $name:literal,
        struct: $struct_name:ident,
        config: $config_type:ty,
        capabilities: [$($cap:expr),* $(,)?],
        models: [$($model:literal),* $(,)?],
        owner: $owner:literal
    ) => {
        pub struct $struct_name {
            config: $config_type,
            client: reqwest::Client,
            capabilities: Vec<ProviderCapability>,
        }

        impl $struct_name {
            pub fn new(config: $config_type) -> Result<Self, ProviderError> {
                let client = $crate::utils::net::http::create_custom_client(
                    std::time::Duration::from_secs(config.timeout),
                )
                .map_err(|e| ProviderError::Configuration {
                    provider: $name,
                    message: format!("Failed to create HTTP client: {}", e),
                })?;

                let capabilities = vec![$($cap),*];

                Ok(Self {
                    config,
                    client,
                    capabilities,
                })
            }
        }

        #[async_trait::async_trait]
        impl LLMProvider for $struct_name {
            fn name(&self) -> &str {
                $name
            }

            fn capabilities(&self) -> &[ProviderCapability] {
                &self.capabilities
            }

            fn list_models(&self) -> Vec<ModelInfo> {
                model_list!($owner, $($model),*)
            }

            async fn get_model(&self, model_id: &str) -> Result<Option<ModelInfo>, ProviderError> {
                Ok(self.list_models().into_iter().find(|m| m.id == model_id))
            }

            impl_health_check!($struct_name, "models");
        }
    };
}

#[path = "macros_provider_impl.rs"]
mod provider_impl_macros;

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

#[cfg(test)]
#[path = "macros_tests.rs"]
mod tests;
