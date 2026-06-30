// Dynamic provider creation methods for DefaultRouter.

use super::*;

mod routes;

pub(super) use routes::is_named_dynamic_provider_route;
use routes::{
    DynamicProviderRoute, resolve_dynamic_provider_api_key, resolve_dynamic_provider_route,
    uses_dynamic_openai_like_provider,
};

impl DefaultRouter {
    /// Dynamic provider creation (Python LiteLLM style)
    /// Creates providers on-demand based on model name and provided options
    pub(super) async fn try_dynamic_provider_creation(
        &self,
        chat_request: &ChatRequest,
        context: RequestContext,
        options: &CompletionOptions,
    ) -> Result<Option<CompletionResponse>> {
        let model = &chat_request.model;

        let Some(route) = resolve_dynamic_provider_route(model, options) else {
            return Ok(None);
        };

        let Some(api_key) = resolve_dynamic_provider_api_key(options, &route) else {
            return Ok(None);
        };

        debug!(
            provider_type = %route.provider_type,
            model = %route.actual_model,
            "Creating dynamic provider for model"
        );

        // Create dynamic provider based on type
        let response = match route.provider_type {
            "anthropic" => {
                self.create_dynamic_anthropic(
                    route.actual_model,
                    &api_key,
                    &route.api_base,
                    chat_request,
                    context,
                )
                .await?
            }
            "azure_ai" => {
                self.create_dynamic_azure_ai(
                    route.actual_model,
                    &api_key,
                    &route.api_base,
                    chat_request,
                    context,
                )
                .await?
            }
            _ if uses_dynamic_openai_like_provider(&route) => {
                self.create_dynamic_openai_like(&route, &api_key, chat_request, context, options)
                    .await?
            }
            _ => {
                self.create_dynamic_openai_compatible(
                    &route,
                    &api_key,
                    chat_request,
                    context,
                    options,
                )
                .await?
            }
        };

        Ok(Some(response))
    }

    pub(super) async fn try_dynamic_provider_stream_creation(
        &self,
        chat_request: &ChatRequest,
        context: RequestContext,
        options: &CompletionOptions,
    ) -> Result<Option<CompletionStream>> {
        let Some(route) = resolve_dynamic_provider_route(&chat_request.model, options) else {
            return Ok(None);
        };

        let Some(api_key) = resolve_dynamic_provider_api_key(options, &route) else {
            return Ok(None);
        };

        debug!(
            provider_type = %route.provider_type,
            model = %route.actual_model,
            "Creating dynamic streaming provider for model"
        );

        let stream = match route.provider_type {
            "anthropic" => {
                self.create_dynamic_anthropic_stream(
                    route.actual_model,
                    &api_key,
                    &route.api_base,
                    chat_request,
                    context,
                )
                .await?
            }
            "azure_ai" => {
                self.create_dynamic_azure_ai_stream(
                    route.actual_model,
                    &api_key,
                    &route.api_base,
                    chat_request,
                    context,
                )
                .await?
            }
            _ if uses_dynamic_openai_like_provider(&route) => {
                self.create_dynamic_openai_like_stream(
                    &route,
                    &api_key,
                    chat_request,
                    context,
                    options,
                )
                .await?
            }
            _ => {
                self.create_dynamic_openai_compatible_stream(
                    &route,
                    &api_key,
                    chat_request,
                    context,
                    options,
                )
                .await?
            }
        };

        Ok(Some(stream))
    }

    /// Create dynamic Anthropic provider
    async fn create_dynamic_anthropic(
        &self,
        model: &str,
        api_key: &str,
        api_base: &str,
        chat_request: &ChatRequest,
        context: RequestContext,
    ) -> Result<CompletionResponse> {
        use crate::core::providers::anthropic::{AnthropicConfig, AnthropicProvider};
        use crate::core::traits::provider::llm_provider::trait_definition::LLMProvider;

        let config = AnthropicConfig::new(api_key)
            .with_base_url(api_base)
            .with_experimental(false);

        let provider = AnthropicProvider::new(config)?;

        let mut updated_request = chat_request.clone();
        updated_request.model = model.to_string();

        let response = LLMProvider::chat_completion(&provider, updated_request, context)
            .await
            .map_err(|e| {
                GatewayError::internal(format!("Dynamic Anthropic provider error: {}", e))
            })?;

        convert_from_chat_completion_response(response)
    }

    async fn create_dynamic_anthropic_stream(
        &self,
        model: &str,
        api_key: &str,
        api_base: &str,
        chat_request: &ChatRequest,
        context: RequestContext,
    ) -> Result<CompletionStream> {
        use crate::core::providers::anthropic::{AnthropicConfig, AnthropicProvider};
        use crate::core::traits::provider::llm_provider::trait_definition::LLMProvider;

        let config = AnthropicConfig::new(api_key)
            .with_base_url(api_base)
            .with_experimental(false);
        let provider = AnthropicProvider::new(config)?;

        let mut updated_request = chat_request.clone();
        updated_request.model = model.to_string();
        updated_request.stream = true;

        let stream = LLMProvider::chat_completion_stream(&provider, updated_request, context)
            .await
            .map_err(|e| {
                GatewayError::internal(format!("Dynamic Anthropic streaming error: {}", e))
            })?;

        Ok(convert_provider_stream(stream, "Anthropic"))
    }

    /// Create dynamic OpenAI-compatible provider
    async fn create_dynamic_openai_compatible(
        &self,
        route: &DynamicProviderRoute<'_>,
        api_key: &str,
        chat_request: &ChatRequest,
        context: RequestContext,
        options: &CompletionOptions,
    ) -> Result<CompletionResponse> {
        use crate::core::providers::openai::OpenAIProvider;
        use crate::core::traits::provider::llm_provider::trait_definition::LLMProvider;

        let config = dynamic_openai_compatible_config(api_key, &route.api_base, options);

        let provider = OpenAIProvider::new(config).await.map_err(|e| {
            GatewayError::internal(format!(
                "Failed to create dynamic {} provider: {}",
                route.provider_label, e
            ))
        })?;

        let mut updated_request = chat_request.clone();
        updated_request.model = route.actual_model.to_string();

        let response = provider
            .chat_completion(updated_request, context)
            .await
            .map_err(|e| {
                GatewayError::internal(format!(
                    "Dynamic {} provider error: {}",
                    route.provider_label, e
                ))
            })?;

        convert_from_chat_completion_response(response)
    }

    async fn create_dynamic_openai_like(
        &self,
        route: &DynamicProviderRoute<'_>,
        api_key: &str,
        chat_request: &ChatRequest,
        context: RequestContext,
        options: &CompletionOptions,
    ) -> Result<CompletionResponse> {
        use crate::core::providers::openai_like::OpenAILikeProvider;
        use crate::core::traits::provider::llm_provider::trait_definition::LLMProvider;

        let config = dynamic_openai_like_config(api_key, &route.api_base, route, options);
        let provider = OpenAILikeProvider::new(config).await.map_err(|e| {
            GatewayError::internal(format!(
                "Failed to create dynamic {} provider: {}",
                route.provider_label, e
            ))
        })?;

        let mut updated_request = chat_request.clone();
        updated_request.model = route.actual_model.to_string();

        let response = LLMProvider::chat_completion(&provider, updated_request, context)
            .await
            .map_err(|e| {
                GatewayError::internal(format!(
                    "Dynamic {} provider error: {}",
                    route.provider_label, e
                ))
            })?;

        convert_from_chat_completion_response(response)
    }

    async fn create_dynamic_openai_compatible_stream(
        &self,
        route: &DynamicProviderRoute<'_>,
        api_key: &str,
        chat_request: &ChatRequest,
        context: RequestContext,
        options: &CompletionOptions,
    ) -> Result<CompletionStream> {
        use crate::core::providers::openai::OpenAIProvider;
        use crate::core::traits::provider::llm_provider::trait_definition::LLMProvider;

        let config = dynamic_openai_compatible_config(api_key, &route.api_base, options);

        let provider = OpenAIProvider::new(config).await.map_err(|e| {
            GatewayError::internal(format!(
                "Failed to create dynamic {} streaming provider: {}",
                route.provider_label, e
            ))
        })?;

        let mut updated_request = chat_request.clone();
        updated_request.model = route.actual_model.to_string();
        updated_request.stream = true;

        let stream = provider
            .chat_completion_stream(updated_request, context)
            .await
            .map_err(|e| {
                GatewayError::internal(format!(
                    "Dynamic {} streaming error: {}",
                    route.provider_label, e
                ))
            })?;

        Ok(convert_provider_stream(stream, route.provider_label))
    }

    async fn create_dynamic_openai_like_stream(
        &self,
        route: &DynamicProviderRoute<'_>,
        api_key: &str,
        chat_request: &ChatRequest,
        context: RequestContext,
        options: &CompletionOptions,
    ) -> Result<CompletionStream> {
        use crate::core::providers::openai_like::OpenAILikeProvider;
        use crate::core::traits::provider::llm_provider::trait_definition::LLMProvider;

        let config = dynamic_openai_like_config(api_key, &route.api_base, route, options);
        let provider = OpenAILikeProvider::new(config).await.map_err(|e| {
            GatewayError::internal(format!(
                "Failed to create dynamic {} streaming provider: {}",
                route.provider_label, e
            ))
        })?;

        let mut updated_request = chat_request.clone();
        updated_request.model = route.actual_model.to_string();
        updated_request.stream = true;

        let stream = LLMProvider::chat_completion_stream(&provider, updated_request, context)
            .await
            .map_err(|e| {
                GatewayError::internal(format!(
                    "Dynamic {} streaming error: {}",
                    route.provider_label, e
                ))
            })?;

        Ok(convert_provider_stream(stream, route.provider_label))
    }

    /// Create dynamic Azure AI provider
    #[cfg(feature = "providers-extra")]
    async fn create_dynamic_azure_ai(
        &self,
        model: &str,
        api_key: &str,
        api_base: &str,
        chat_request: &ChatRequest,
        context: RequestContext,
    ) -> Result<CompletionResponse> {
        use crate::core::providers::azure_ai::{AzureAIConfig, AzureAIProvider};
        use crate::core::traits::provider::llm_provider::trait_definition::LLMProvider;

        let mut config = AzureAIConfig::new("azure_ai");
        config.base.api_key = Some(api_key.to_string());
        config.base.api_base = Some(api_base.to_string());

        // Also check environment variables
        if config.base.api_key.is_none()
            && let Ok(key) = std::env::var("AZURE_AI_API_KEY")
        {
            config.base.api_key = Some(key);
        }
        if config.base.api_base.is_none()
            && let Ok(base) = std::env::var("AZURE_AI_API_BASE")
        {
            config.base.api_base = Some(base);
        }

        let provider = AzureAIProvider::new(config).map_err(|e| {
            GatewayError::internal(format!("Failed to create dynamic Azure AI provider: {}", e))
        })?;

        let mut updated_request = chat_request.clone();
        updated_request.model = model.to_string();

        let response = provider
            .chat_completion(updated_request, context)
            .await
            .map_err(|e| {
                GatewayError::internal(format!("Dynamic Azure AI provider error: {}", e))
            })?;

        convert_from_chat_completion_response(response)
    }

    #[cfg(feature = "providers-extra")]
    async fn create_dynamic_azure_ai_stream(
        &self,
        model: &str,
        api_key: &str,
        api_base: &str,
        chat_request: &ChatRequest,
        context: RequestContext,
    ) -> Result<CompletionStream> {
        use crate::core::providers::azure_ai::{AzureAIConfig, AzureAIProvider};
        use crate::core::traits::provider::llm_provider::trait_definition::LLMProvider;

        let mut config = AzureAIConfig::new("azure_ai");
        config.base.api_key = Some(api_key.to_string());
        config.base.api_base = Some(api_base.to_string());

        if config.base.api_key.is_none()
            && let Ok(key) = std::env::var("AZURE_AI_API_KEY")
        {
            config.base.api_key = Some(key);
        }
        if config.base.api_base.is_none()
            && let Ok(base) = std::env::var("AZURE_AI_API_BASE")
        {
            config.base.api_base = Some(base);
        }

        let provider = AzureAIProvider::new(config).map_err(|e| {
            GatewayError::internal(format!(
                "Failed to create dynamic Azure AI streaming provider: {}",
                e
            ))
        })?;

        let mut updated_request = chat_request.clone();
        updated_request.model = model.to_string();
        updated_request.stream = true;

        let stream = provider
            .chat_completion_stream(updated_request, context)
            .await
            .map_err(|e| {
                GatewayError::internal(format!("Dynamic Azure AI streaming error: {}", e))
            })?;

        Ok(convert_provider_stream(stream, "Azure AI"))
    }

    /// Create dynamic Azure AI provider (stub when providers-extra is disabled)
    #[cfg(not(feature = "providers-extra"))]
    async fn create_dynamic_azure_ai(
        &self,
        model: &str,
        api_key: &str,
        api_base: &str,
        chat_request: &ChatRequest,
        context: RequestContext,
    ) -> Result<CompletionResponse> {
        let _ = (model, api_key, api_base, chat_request, context);
        Err(GatewayError::not_implemented(
            "dynamic azure_ai requires the `providers-extra` feature",
        ))
    }

    #[cfg(not(feature = "providers-extra"))]
    async fn create_dynamic_azure_ai_stream(
        &self,
        _model: &str,
        _api_key: &str,
        _api_base: &str,
        _chat_request: &ChatRequest,
        _context: RequestContext,
    ) -> Result<CompletionStream> {
        Err(GatewayError::not_implemented(
            "dynamic azure_ai streaming requires the `providers-extra` feature",
        ))
    }
}

fn convert_provider_stream(
    stream: std::pin::Pin<
        Box<
            dyn futures::Stream<
                    Item = std::result::Result<
                        crate::core::types::responses::ChatChunk,
                        crate::core::providers::ProviderError,
                    >,
                > + Send,
        >,
    >,
    provider_name: &str,
) -> CompletionStream {
    let provider_name = provider_name.to_string();
    Box::pin(stream.map(move |result| {
        result
            .map(convert_chat_chunk_to_completion_chunk)
            .map_err(|e| {
                GatewayError::internal(format!("Dynamic {provider_name} stream chunk error: {e}"))
            })
    }))
}

fn dynamic_openai_compatible_config(
    api_key: &str,
    api_base: &str,
    options: &CompletionOptions,
) -> crate::core::providers::openai::config::OpenAIConfig {
    use crate::core::providers::base::BaseConfig;
    use crate::core::providers::openai::config::OpenAIConfig;

    OpenAIConfig {
        base: BaseConfig {
            api_key: Some(api_key.to_string()),
            api_base: Some(api_base.to_string()),
            timeout: options.timeout.unwrap_or(60),
            max_retries: 3,
            headers: options.headers.clone().unwrap_or_default(),
            organization: options.organization.clone(),
            api_version: None,
        },
        provider_name: "openai".to_string(),
        organization: options.organization.clone(),
        project: None,
        model_mappings: Default::default(),
        features: Default::default(),
    }
}

fn dynamic_openai_like_config(
    api_key: &str,
    api_base: &str,
    route: &DynamicProviderRoute<'_>,
    options: &CompletionOptions,
) -> crate::core::providers::openai_like::OpenAILikeConfig {
    let mut config =
        crate::core::providers::openai_like::OpenAILikeConfig::with_api_key(api_base, api_key)
            .with_provider_name(route.provider_type);
    config.base.timeout = options.timeout.unwrap_or(60);
    config.base.headers = options.headers.clone().unwrap_or_default();
    config.base.organization = options.organization.clone();
    config
}

#[cfg(test)]
mod tests;
