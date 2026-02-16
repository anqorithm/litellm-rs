//! OpenAI-compatible provider macro
//!
//! `define_openai_compatible_provider!` targets providers that:
//! - Use BaseConfig with `api_key` and `api_base`
//! - Accept OpenAI-style chat/completions
//! - Share the same request/response mapping logic

/// Macro to define OpenAI-compatible providers with shared boilerplate.
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
            supported_models: Vec<$crate::core::types::model::ModelInfo>,
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
                let config = <$config_type>::new($provider_name)
                    .with_api_key(api_key.into());
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

            fn capabilities(&self) -> &'static [$crate::core::types::model::ProviderCapability] {
                &[
                    $crate::core::types::model::ProviderCapability::ChatCompletion,
                    $crate::core::types::model::ProviderCapability::ChatCompletionStream,
                ]
            }

            fn models(&self) -> &[$crate::core::types::model::ModelInfo] {
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
                request: $crate::core::types::chat::ChatRequest,
                _context: $crate::core::types::context::RequestContext,
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
                request: $crate::core::types::chat::ChatRequest,
                context: $crate::core::types::context::RequestContext,
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
                _request: $crate::core::types::chat::ChatRequest,
                _context: $crate::core::types::context::RequestContext,
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
