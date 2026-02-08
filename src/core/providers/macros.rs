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
                $(#[serde(default = concat!("default_", stringify!($field)))])?
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
                #[allow(dead_code)]
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

/// Macro to define OpenAI-compatible providers with shared boilerplate.
///
/// This targets providers that:
/// - Use BaseConfig with `api_key` and `api_base`
/// - Accept OpenAI-style chat/completions
/// - Share the same request/response mapping logic
#[macro_export]
macro_rules! define_openai_compatible_provider {
    (
        provider: $provider_name:expr,
        struct_name: $struct_name:ident,
        config: $config_type:path,
        error_mapper: $error_mapper:path,
        model_info: $model_info:path,
        default_base_url: $default_base_url:expr,
        auth_header: $auth_header_name:literal,
        auth_prefix: $auth_prefix:literal,
        supported_params: [$($param:literal),* $(,)?]
        $(,)?
    ) => {
        #[derive(Debug, Clone)]
        pub struct $struct_name {
            config: $config_type,
            http_client: std::sync::Arc<reqwest::Client>,
            supported_models: Vec<$crate::core::types::ModelInfo>,
        }

        impl $struct_name {
            pub fn new(config: $config_type) -> Result<Self, $crate::core::providers::unified_provider::ProviderError> {
                <$config_type as $crate::core::traits::provider::ProviderConfig>::validate(&config)
                    .map_err(|e| $crate::core::providers::unified_provider::ProviderError::configuration($provider_name, e))?;

                let http_client = $crate::utils::net::http::get_client_with_timeout_fallible(
                    <$config_type as $crate::core::traits::provider::ProviderConfig>::timeout(&config),
                )
                .map_err(|e| {
                    $crate::core::providers::unified_provider::ProviderError::initialization(
                        $provider_name,
                        format!("Failed to create HTTP client: {}", e),
                    )
                })?;

                Ok(Self {
                    config,
                    http_client,
                    supported_models: ($model_info)(),
                })
            }

            pub async fn with_api_key(
                api_key: impl Into<String>,
            ) -> Result<Self, $crate::core::providers::unified_provider::ProviderError> {
                let config = <$config_type>::new(api_key);
                Self::new(config)
            }

            fn build_headers(&self) -> std::collections::HashMap<String, String> {
                let mut headers = std::collections::HashMap::new();

                if let Some(api_key) = &self.config.base.api_key {
                    headers.insert(
                        $auth_header_name.to_string(),
                        format!("{}{}", $auth_prefix, api_key),
                    );
                }

                headers.insert("Content-Type".to_string(), "application/json".to_string());
                headers
            }
        }

        #[async_trait::async_trait]
        impl $crate::core::traits::provider::llm_provider::trait_definition::LLMProvider for $struct_name {
            type Config = $config_type;
            type Error = $crate::core::providers::unified_provider::ProviderError;
            type ErrorMapper = $error_mapper;

            fn name(&self) -> &'static str {
                $provider_name
            }

            fn capabilities(&self) -> &'static [$crate::core::types::ProviderCapability] {
                &[
                    $crate::core::types::ProviderCapability::ChatCompletion,
                    $crate::core::types::ProviderCapability::ChatCompletionStream,
                ]
            }

            fn models(&self) -> &[$crate::core::types::ModelInfo] {
                &self.supported_models
            }

            fn get_supported_openai_params(&self, _model: &str) -> &'static [&'static str] {
                &[$($param),*]
            }

            async fn map_openai_params(
                &self,
                params: std::collections::HashMap<String, serde_json::Value>,
                _model: &str,
            ) -> Result<std::collections::HashMap<String, serde_json::Value>, Self::Error> {
                Ok(params)
            }

            async fn transform_request(
                &self,
                request: $crate::core::types::ChatRequest,
                _context: $crate::core::types::RequestContext,
            ) -> Result<serde_json::Value, Self::Error> {
                let mut req = serde_json::json!({
                    "model": request.model,
                    "messages": request.messages,
                });

                if let Some(max_tokens) = request.max_tokens {
                    req["max_tokens"] = serde_json::Value::Number(max_tokens.into());
                }

                if let Some(temperature) = request.temperature {
                    req["temperature"] = serde_json::to_value(temperature)
                        .map_err(|e| {
                            $crate::core::providers::unified_provider::ProviderError::serialization(
                                $provider_name,
                                e.to_string(),
                            )
                        })?;
                }

                if let Some(top_p) = request.top_p {
                    req["top_p"] = serde_json::to_value(top_p)
                        .map_err(|e| {
                            $crate::core::providers::unified_provider::ProviderError::serialization(
                                $provider_name,
                                e.to_string(),
                            )
                        })?;
                }

                if let Some(frequency_penalty) = request.frequency_penalty {
                    req["frequency_penalty"] = serde_json::to_value(frequency_penalty)
                        .map_err(|e| {
                            $crate::core::providers::unified_provider::ProviderError::serialization(
                                $provider_name,
                                e.to_string(),
                            )
                        })?;
                }

                if let Some(presence_penalty) = request.presence_penalty {
                    req["presence_penalty"] = serde_json::to_value(presence_penalty)
                        .map_err(|e| {
                            $crate::core::providers::unified_provider::ProviderError::serialization(
                                $provider_name,
                                e.to_string(),
                            )
                        })?;
                }

                if let Some(stop) = &request.stop {
                    req["stop"] = serde_json::to_value(stop)
                        .map_err(|e| {
                            $crate::core::providers::unified_provider::ProviderError::serialization(
                                $provider_name,
                                e.to_string(),
                            )
                        })?;
                }

                if request.stream {
                    req["stream"] = serde_json::Value::Bool(true);
                }

                Ok(req)
            }

            async fn transform_response(
                &self,
                raw_response: &[u8],
                _model: &str,
                _request_id: &str,
            ) -> Result<$crate::core::types::responses::ChatResponse, Self::Error> {
                let response_text = String::from_utf8_lossy(raw_response);
                let response: $crate::core::types::responses::ChatResponse =
                    serde_json::from_str(&response_text).map_err(|e| {
                        $crate::core::providers::unified_provider::ProviderError::serialization(
                            $provider_name,
                            e.to_string(),
                        )
                    })?;
                Ok(response)
            }

            fn get_error_mapper(&self) -> Self::ErrorMapper {
                $error_mapper
            }

            async fn chat_completion(
                &self,
                request: $crate::core::types::ChatRequest,
                context: $crate::core::types::RequestContext,
            ) -> Result<$crate::core::types::responses::ChatResponse, Self::Error> {
                let base_url = self
                    .config
                    .base
                    .api_base
                    .as_deref()
                    .unwrap_or($default_base_url);

                let url = format!("{}/chat/completions", base_url);

                let body = self.transform_request(request.clone(), context).await?;
                let headers = self.build_headers();

                let mut req_builder = self.http_client.post(&url);
                for (key, value) in headers {
                    req_builder = req_builder.header(key, value);
                }

                let response = req_builder
                    .json(&body)
                    .send()
                    .await
                    .map_err(|e| {
                        $crate::core::providers::unified_provider::ProviderError::network(
                            $provider_name,
                            e.to_string(),
                        )
                    })?;

                if !response.status().is_success() {
                    let status = response.status().as_u16();
                    let error_text = response.text().await.unwrap_or_default();

                    return Err(match status {
                        401 => $crate::core::providers::unified_provider::ProviderError::authentication(
                            $provider_name,
                            error_text,
                        ),
                        429 => $crate::core::providers::unified_provider::ProviderError::rate_limit(
                            $provider_name,
                            None,
                        ),
                        404 => $crate::core::providers::unified_provider::ProviderError::model_not_found(
                            $provider_name,
                            request.model,
                        ),
                        _ => $crate::core::providers::unified_provider::ProviderError::api_error(
                            $provider_name,
                            status,
                            error_text,
                        ),
                    });
                }

                let response_bytes = response
                    .bytes()
                    .await
                    .map_err(|e| {
                        $crate::core::providers::unified_provider::ProviderError::network(
                            $provider_name,
                            e.to_string(),
                        )
                    })?;

                self.transform_response(&response_bytes, &request.model, "")
                    .await
            }

            async fn chat_completion_stream(
                &self,
                _request: $crate::core::types::ChatRequest,
                _context: $crate::core::types::RequestContext,
            ) -> Result<
                std::pin::Pin<
                    Box<
                        dyn futures::Stream<
                                Item = Result<$crate::core::types::responses::ChatChunk, Self::Error>,
                            > + Send,
                    >,
                >,
                Self::Error,
            > {
                Err($crate::core::providers::unified_provider::ProviderError::not_implemented(
                    $provider_name,
                    "Streaming not yet implemented",
                ))
            }

            async fn health_check(&self) -> $crate::core::types::health::HealthStatus {
                $crate::core::types::health::HealthStatus::Healthy
            }

            async fn calculate_cost(
                &self,
                model: &str,
                input_tokens: u32,
                output_tokens: u32,
            ) -> Result<f64, Self::Error> {
                let model_info = self
                    .supported_models
                    .iter()
                    .find(|m| m.id == model)
                    .ok_or_else(|| {
                        $crate::core::providers::unified_provider::ProviderError::model_not_found(
                            $provider_name,
                            model.to_string(),
                        )
                    })?;

                let input_cost = model_info
                    .input_cost_per_1k_tokens
                    .unwrap_or(0.0)
                    * input_tokens as f64
                    / 1000.0;
                let output_cost = model_info
                    .output_cost_per_1k_tokens
                    .unwrap_or(0.0)
                    * output_tokens as f64
                    / 1000.0;

                Ok(input_cost + output_cost)
            }
        }
    };
}

/// Macro to define HTTP-based providers with custom hooks for request/response handling.
#[macro_export]
macro_rules! define_http_provider_with_hooks {
    (
        provider: $provider_name:expr,
        struct_name: $struct_name:ident,
        config: $config_type:path,
        error_mapper: $error_mapper:path,
        model_info: $model_info:path,
        capabilities: $capabilities:expr,
        url_builder: $url_builder:expr,
        request_builder: $request_builder:expr,
        supported_params: [$($param:literal),* $(,)?],
        build_headers: $build_headers:expr,
        with_api_key: $with_api_key:expr,
        request_transform: $request_transform:expr,
        response_transform: $response_transform:expr,
        error_map: $error_map:expr,
        health_check: $health_check:expr,
        streaming_error: $streaming_error:expr,
        calculate_cost: $calculate_cost:expr $(,)?
    ) => {
        $crate::define_http_provider_with_hooks!(@impl
            provider: $provider_name,
            struct_name: $struct_name,
            config: $config_type,
            error_mapper: $error_mapper,
            model_info: $model_info,
            capabilities: $capabilities,
            url_builder: $url_builder,
            request_builder: $request_builder,
            supported_params: [$($param),*],
            build_headers: $build_headers,
            with_api_key: $with_api_key,
            request_transform: $request_transform,
            response_transform: $response_transform,
            error_map: $error_map,
            health_check: $health_check,
            streaming_error: $streaming_error,
            calculate_cost: $calculate_cost
        );
    };
    (
        provider: $provider_name:expr,
        struct_name: $struct_name:ident,
        config: $config_type:path,
        error_mapper: $error_mapper:path,
        model_info: $model_info:path,
        capabilities: $capabilities:expr,
        url_builder: $url_builder:expr,
        request_builder: $request_builder:expr,
        supported_params: [$($param:literal),* $(,)?],
        build_headers: $build_headers:expr,
        with_api_key: $with_api_key:expr,
        request_transform: $request_transform:expr,
        response_transform: $response_transform:expr,
        error_map: $error_map:expr,
        health_check: $health_check:expr,
        streaming_error: $streaming_error:expr $(,)?
    ) => {
        $crate::define_http_provider_with_hooks!(@impl
            provider: $provider_name,
            struct_name: $struct_name,
            config: $config_type,
            error_mapper: $error_mapper,
            model_info: $model_info,
            capabilities: $capabilities,
            url_builder: $url_builder,
            request_builder: $request_builder,
            supported_params: [$($param),*],
            build_headers: $build_headers,
            with_api_key: $with_api_key,
            request_transform: $request_transform,
            response_transform: $response_transform,
            error_map: $error_map,
            health_check: $health_check,
            streaming_error: $streaming_error,
            calculate_cost: |provider: &$struct_name,
                              model: &str,
                              input_tokens: u32,
                              output_tokens: u32|
             -> Result<f64, $crate::core::providers::unified_provider::ProviderError> {
                let model_info = provider
                    .supported_models
                    .iter()
                    .find(|m| m.id == model)
                    .ok_or_else(|| {
                        $crate::core::providers::unified_provider::ProviderError::model_not_found(
                            $provider_name,
                            model.to_string(),
                        )
                    })?;

                let input_cost = model_info
                    .input_cost_per_1k_tokens
                    .unwrap_or(0.0)
                    * input_tokens as f64
                    / 1000.0;
                let output_cost = model_info
                    .output_cost_per_1k_tokens
                    .unwrap_or(0.0)
                    * output_tokens as f64
                    / 1000.0;

                Ok(input_cost + output_cost)
            }
        );
    };
    (
        @impl
        provider: $provider_name:expr,
        struct_name: $struct_name:ident,
        config: $config_type:path,
        error_mapper: $error_mapper:path,
        model_info: $model_info:path,
        capabilities: $capabilities:expr,
        url_builder: $url_builder:expr,
        request_builder: $request_builder:expr,
        supported_params: [$($param:literal),* $(,)?],
        build_headers: $build_headers:expr,
        with_api_key: $with_api_key:expr,
        request_transform: $request_transform:expr,
        response_transform: $response_transform:expr,
        error_map: $error_map:expr,
        health_check: $health_check:expr,
        streaming_error: $streaming_error:expr,
        calculate_cost: $calculate_cost:expr $(,)?
    ) => {
        #[derive(Debug, Clone)]
        pub struct $struct_name {
            config: $config_type,
            http_client: std::sync::Arc<reqwest::Client>,
            supported_models: Vec<$crate::core::types::ModelInfo>,
        }

        impl $struct_name {
            pub fn new(config: $config_type) -> Result<Self, $crate::core::providers::unified_provider::ProviderError> {
                <$config_type as $crate::core::traits::provider::ProviderConfig>::validate(&config)
                    .map_err(|e| $crate::core::providers::unified_provider::ProviderError::configuration($provider_name, e))?;

                let http_client = $crate::utils::net::http::get_client_with_timeout_fallible(
                    <$config_type as $crate::core::traits::provider::ProviderConfig>::timeout(&config),
                )
                .map_err(|e| {
                    $crate::core::providers::unified_provider::ProviderError::initialization(
                        $provider_name,
                        format!("Failed to create HTTP client: {}", e),
                    )
                })?;

                Ok(Self {
                    config,
                    http_client,
                    supported_models: ($model_info)(),
                })
            }

            pub async fn with_api_key(
                api_key: impl Into<String>,
            ) -> Result<Self, $crate::core::providers::unified_provider::ProviderError> {
                ($with_api_key)(api_key.into())
            }

            fn build_headers(&self) -> std::collections::HashMap<String, String> {
                let mut headers = std::collections::HashMap::new();
                ($build_headers)(self, &mut headers);
                headers
            }
        }

        #[async_trait::async_trait]
        impl $crate::core::traits::provider::llm_provider::trait_definition::LLMProvider for $struct_name {
            type Config = $config_type;
            type Error = $crate::core::providers::unified_provider::ProviderError;
            type ErrorMapper = $error_mapper;

            fn name(&self) -> &'static str {
                $provider_name
            }

            fn capabilities(&self) -> &'static [$crate::core::types::ProviderCapability] {
                $capabilities
            }

            fn models(&self) -> &[$crate::core::types::ModelInfo] {
                &self.supported_models
            }

            fn get_supported_openai_params(&self, _model: &str) -> &'static [&'static str] {
                &[$($param),*]
            }

            async fn map_openai_params(
                &self,
                params: std::collections::HashMap<String, serde_json::Value>,
                _model: &str,
            ) -> Result<std::collections::HashMap<String, serde_json::Value>, Self::Error> {
                Ok(params)
            }

            async fn transform_request(
                &self,
                request: $crate::core::types::ChatRequest,
                _context: $crate::core::types::RequestContext,
            ) -> Result<serde_json::Value, Self::Error> {
                ($request_transform)(self, request)
            }

            async fn transform_response(
                &self,
                raw_response: &[u8],
                _model: &str,
                _request_id: &str,
            ) -> Result<$crate::core::types::responses::ChatResponse, Self::Error> {
                ($response_transform)(self, raw_response, _model, _request_id)
            }

            fn get_error_mapper(&self) -> Self::ErrorMapper {
                $error_mapper
            }

            async fn chat_completion(
                &self,
                request: $crate::core::types::ChatRequest,
                context: $crate::core::types::RequestContext,
            ) -> Result<$crate::core::types::responses::ChatResponse, Self::Error> {
                let url = ($url_builder)(self);

                let body = self.transform_request(request.clone(), context).await?;
                let headers = self.build_headers();

                let mut req_builder = ($request_builder)(self, &url);
                for (key, value) in headers {
                    req_builder = req_builder.header(key, value);
                }

                let response = req_builder
                    .json(&body)
                    .send()
                    .await
                    .map_err(|e| {
                        $crate::core::providers::unified_provider::ProviderError::network(
                            $provider_name,
                            e.to_string(),
                        )
                    })?;

                if !response.status().is_success() {
                    let status = response.status().as_u16();
                    let error_text = response.text().await.unwrap_or_default();
                    return Err(($error_map)(self, status, error_text, &request));
                }

                let response_bytes = response
                    .bytes()
                    .await
                    .map_err(|e| {
                        $crate::core::providers::unified_provider::ProviderError::network(
                            $provider_name,
                            e.to_string(),
                        )
                    })?;

                self.transform_response(&response_bytes, &request.model, "")
                    .await
            }

            async fn chat_completion_stream(
                &self,
                _request: $crate::core::types::ChatRequest,
                _context: $crate::core::types::RequestContext,
            ) -> Result<
                std::pin::Pin<
                    Box<
                        dyn futures::Stream<
                                Item = Result<$crate::core::types::responses::ChatChunk, Self::Error>,
                            > + Send,
                    >,
                >,
                Self::Error,
            > {
                Err($crate::core::providers::unified_provider::ProviderError::not_implemented(
                    $provider_name,
                    $streaming_error,
                ))
            }

            async fn health_check(&self) -> $crate::core::types::health::HealthStatus {
                ($health_check)(self).await
            }

            async fn calculate_cost(
                &self,
                model: &str,
                input_tokens: u32,
                output_tokens: u32,
            ) -> Result<f64, Self::Error> {
                ($calculate_cost)(self, model, input_tokens, output_tokens)
            }
        }
    };
}

/// Macro to define pooled HTTP providers that use GlobalPoolManager with custom hooks.
///
/// This targets providers that:
/// - Rely on GlobalPoolManager for non-streaming requests
/// - Have custom request/response transforms
/// - Optionally implement streaming via a custom async hook
#[macro_export]
macro_rules! define_pooled_http_provider_with_hooks {
    (
        provider: $provider_name:expr,
        struct_name: $struct_name:ident,
        config: $config_type:path,
        error_mapper: $error_mapper:path,
        model_info: $model_info:path,
        capabilities: $capabilities:expr,
        url_builder: $url_builder:expr,
        http_method: $http_method:expr,
        supported_params: [$($param:literal),* $(,)?],
        build_headers: $build_headers:expr,
        with_api_key: $with_api_key:expr,
        map_openai_params: $map_openai_params:expr,
        request_transform: $request_transform:expr,
        response_transform: $response_transform:expr,
        error_map: $error_map:expr,
        health_check: $health_check:expr,
        streaming: $streaming:expr,
        calculate_cost: $calculate_cost:expr $(,)?
    ) => {
        #[derive(Debug, Clone)]
        pub struct $struct_name {
            config: $config_type,
            pool_manager: std::sync::Arc<$crate::core::providers::base::GlobalPoolManager>,
            supported_models: Vec<$crate::core::types::ModelInfo>,
        }

        impl $struct_name {
            pub fn new(
                config: $config_type,
            ) -> Result<Self, $crate::core::providers::unified_provider::ProviderError> {
                <$config_type as $crate::core::traits::provider::ProviderConfig>::validate(&config)
                    .map_err(|e| {
                        $crate::core::providers::unified_provider::ProviderError::configuration(
                            $provider_name,
                            e,
                        )
                    })?;

                let pool_manager = std::sync::Arc::new(
                    $crate::core::providers::base::GlobalPoolManager::new().map_err(|e| {
                        $crate::core::providers::unified_provider::ProviderError::configuration(
                            $provider_name,
                            e.to_string(),
                        )
                    })?,
                );

                Ok(Self {
                    config,
                    pool_manager,
                    supported_models: ($model_info)(),
                })
            }

            pub async fn with_api_key(
                api_key: impl Into<String>,
            ) -> Result<Self, $crate::core::providers::unified_provider::ProviderError> {
                ($with_api_key)(api_key.into())
            }

            fn build_headers(&self) -> Vec<$crate::core::providers::base::HeaderPair> {
                ($build_headers)(self)
            }
        }

        #[async_trait::async_trait]
        impl $crate::core::traits::provider::llm_provider::trait_definition::LLMProvider for $struct_name {
            type Config = $config_type;
            type Error = $crate::core::providers::unified_provider::ProviderError;
            type ErrorMapper = $error_mapper;

            fn name(&self) -> &'static str {
                $provider_name
            }

            fn capabilities(&self) -> &'static [$crate::core::types::ProviderCapability] {
                $capabilities
            }

            fn models(&self) -> &[$crate::core::types::ModelInfo] {
                &self.supported_models
            }

            fn get_supported_openai_params(&self, _model: &str) -> &'static [&'static str] {
                &[$($param),*]
            }

            async fn map_openai_params(
                &self,
                params: std::collections::HashMap<String, serde_json::Value>,
                model: &str,
            ) -> Result<std::collections::HashMap<String, serde_json::Value>, Self::Error> {
                ($map_openai_params)(self, params, model)
            }

            async fn transform_request(
                &self,
                request: $crate::core::types::ChatRequest,
                _context: $crate::core::types::RequestContext,
            ) -> Result<serde_json::Value, Self::Error> {
                ($request_transform)(self, request)
            }

            async fn transform_response(
                &self,
                raw_response: &[u8],
                _model: &str,
                _request_id: &str,
            ) -> Result<$crate::core::types::responses::ChatResponse, Self::Error> {
                ($response_transform)(self, raw_response, _model, _request_id)
            }

            fn get_error_mapper(&self) -> Self::ErrorMapper {
                $error_mapper
            }

            async fn chat_completion(
                &self,
                request: $crate::core::types::ChatRequest,
                context: $crate::core::types::RequestContext,
            ) -> Result<$crate::core::types::responses::ChatResponse, Self::Error> {
                let url = ($url_builder)(self);
                let body = self.transform_request(request.clone(), context.clone()).await?;
                let headers = self.build_headers();

                let response = self
                    .pool_manager
                    .execute_request(&url, $http_method, headers, Some(body))
                    .await?;

                let status = response.status();
                let response_bytes = response
                    .bytes()
                    .await
                    .map_err(|e| {
                        $crate::core::providers::unified_provider::ProviderError::network(
                            $provider_name,
                            e.to_string(),
                        )
                    })?;

                if !status.is_success() {
                    let error_text = String::from_utf8_lossy(&response_bytes).to_string();
                    return Err(($error_map)(self, status.as_u16(), error_text, &request));
                }

                self.transform_response(&response_bytes, &request.model, &context.request_id)
                    .await
            }

            async fn chat_completion_stream(
                &self,
                request: $crate::core::types::ChatRequest,
                context: $crate::core::types::RequestContext,
            ) -> Result<
                std::pin::Pin<
                    Box<
                        dyn futures::Stream<
                                Item = Result<$crate::core::types::responses::ChatChunk, Self::Error>,
                            > + Send,
                    >,
                >,
                Self::Error,
            > {
                ($streaming)(self, request, context).await
            }

            async fn health_check(&self) -> $crate::core::types::health::HealthStatus {
                ($health_check)(self).await
            }

            async fn calculate_cost(
                &self,
                model: &str,
                input_tokens: u32,
                output_tokens: u32,
            ) -> Result<f64, Self::Error> {
                ($calculate_cost)(self, model, input_tokens, output_tokens)
            }
        }
    };
}

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
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_require_config_str_success() {
        let config = json!({
            "api_key": "sk-test-key",
            "base_url": "https://api.example.com"
        });

        let result = require_config_str(&config, "api_key", "openai");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "sk-test-key");
    }

    #[test]
    fn test_require_config_str_missing() {
        let config = json!({
            "base_url": "https://api.example.com"
        });

        let result = require_config_str(&config, "api_key", "openai");
        assert!(result.is_err());

        let err = result.unwrap_err();
        assert!(matches!(err, ProviderError::Configuration { .. }));
    }

    #[test]
    fn test_require_config_str_wrong_type() {
        let config = json!({
            "api_key": 12345
        });

        let result = require_config_str(&config, "api_key", "openai");
        assert!(result.is_err());
    }

    #[test]
    fn test_get_config_str_some() {
        let config = json!({
            "api_key": "sk-test-key"
        });

        let result = get_config_str(&config, "api_key");
        assert_eq!(result, Some("sk-test-key"));
    }

    #[test]
    fn test_get_config_str_none() {
        let config = json!({});

        let result = get_config_str(&config, "api_key");
        assert_eq!(result, None);
    }

    #[test]
    fn test_require_config_u64_success() {
        let config = json!({
            "timeout": 30,
            "max_retries": 5
        });

        let result = require_config_u64(&config, "timeout", "openai");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 30);
    }

    #[test]
    fn test_require_config_u64_missing() {
        let config = json!({});

        let result = require_config_u64(&config, "timeout", "openai");
        assert!(result.is_err());
    }

    #[test]
    fn test_get_config_u64_or_present() {
        let config = json!({
            "timeout": 60
        });

        let result = get_config_u64_or(&config, "timeout", 30);
        assert_eq!(result, 60);
    }

    #[test]
    fn test_get_config_u64_or_default() {
        let config = json!({});

        let result = get_config_u64_or(&config, "timeout", 30);
        assert_eq!(result, 30);
    }

    #[test]
    fn test_require_config_bool_success() {
        let config = json!({
            "enable_streaming": true
        });

        let result = require_config_bool(&config, "enable_streaming", "openai");
        assert!(result.is_ok());
        assert!(result.unwrap());
    }

    #[test]
    fn test_require_config_bool_missing() {
        let config = json!({});

        let result = require_config_bool(&config, "enable_streaming", "openai");
        assert!(result.is_err());
    }

    #[test]
    fn test_get_config_bool_or_present() {
        let config = json!({
            "debug": true
        });

        let result = get_config_bool_or(&config, "debug", false);
        assert!(result);
    }

    #[test]
    fn test_get_config_bool_or_default() {
        let config = json!({});

        let result = get_config_bool_or(&config, "debug", false);
        assert!(!result);
    }

    #[test]
    fn test_nested_config_extraction() {
        let config = json!({
            "provider": {
                "api_key": "nested-key"
            }
        });

        // Direct extraction from nested won't work - need to get nested first
        let nested = config.get("provider").unwrap();
        let result = require_config_str(nested, "api_key", "openai");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "nested-key");
    }

    #[test]
    fn test_null_value_treated_as_missing() {
        let config = json!({
            "api_key": null
        });

        let result = require_config_str(&config, "api_key", "openai");
        assert!(result.is_err());
    }

    #[test]
    fn test_empty_string_is_valid() {
        let config = json!({
            "api_key": ""
        });

        let result = require_config_str(&config, "api_key", "openai");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "");
    }

    #[test]
    fn test_error_message_contains_key() {
        let config = json!({});

        let result = require_config_str(&config, "my_special_key", "test_provider");
        let err = result.unwrap_err();

        match err {
            ProviderError::Configuration { message, provider } => {
                assert!(message.contains("my_special_key"));
                assert_eq!(provider, "test_provider");
            }
            _ => panic!("Expected Configuration error"),
        }
    }
}
