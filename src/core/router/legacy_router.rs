//! Legacy Router implementation
//!
//! This module contains the original Router struct and implementation.
//! It provides routing functionality with health checking, load balancing,
//! and metrics collection across multiple AI providers.

use crate::config::ProviderConfig;
use crate::core::providers::ProviderRegistry;
use crate::core::router::health::{HealthChecker, RouterHealthStatus};
use crate::core::router::load_balancer::LoadBalancer;
use crate::core::router::metrics::{RouterMetrics, RouterMetricsSnapshot};
use crate::core::router::strategy::types::RoutingStrategy;
use crate::core::types::{
    common::{ModelInfo, RequestContext},
    requests::{ChatRequest, CompletionRequest, EmbeddingRequest, ImageGenerationRequest},
    responses::{
        ChatChunk, ChatResponse, CompletionResponse, EmbeddingResponse, ImageGenerationResponse,
    },
};
use crate::storage::StorageLayer;
use crate::utils::error::Result;
use crate::utils::perf::r#async::{ConcurrentRunner, RetryPolicy, default_retry_policy};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::RwLock;
use tracing::{info, warn};

/// Core router for managing AI providers and routing requests
#[derive(Clone)]
pub struct Router {
    /// Available providers
    providers: Arc<RwLock<ProviderRegistry>>,
    /// Provider configurations
    configs: Arc<Vec<ProviderConfig>>,
    /// Storage layer for metrics and caching
    storage: Arc<StorageLayer>,
    /// Routing strategy
    strategy: RoutingStrategy,
    /// Health checker
    health_checker: Arc<HealthChecker>,
    /// Load balancer
    load_balancer: Arc<LoadBalancer>,
    /// Router metrics
    metrics: Arc<RouterMetrics>,
    /// Concurrent runner for parallel operations
    concurrent_runner: ConcurrentRunner,
    /// Retry policy for failed requests
    retry_policy: RetryPolicy,
}

impl Router {
    /// Create a new router
    pub async fn new(
        configs: Vec<ProviderConfig>,
        storage: Arc<StorageLayer>,
        strategy: RoutingStrategy,
    ) -> Result<Self> {
        info!("Initializing router with {} providers", configs.len());

        let providers = Arc::new(RwLock::new(ProviderRegistry::new()));
        let health_checker = Arc::new(HealthChecker::new(providers.clone()).await?);
        let load_balancer = Arc::new(LoadBalancer::new(strategy.clone()).await?);
        let metrics = Arc::new(RouterMetrics::new().await?);

        // Create concurrent runner for parallel operations
        let concurrent_runner =
            ConcurrentRunner::new(10).with_timeout(std::time::Duration::from_secs(30));

        // Create retry policy for failed requests using default settings
        let retry_policy = default_retry_policy();

        Ok(Self {
            providers,
            configs: Arc::new(configs),
            storage,
            strategy,
            health_checker,
            load_balancer,
            metrics,
            concurrent_runner,
            retry_policy,
        })
    }

    /// Route a chat completion request with retry logic
    pub async fn route_chat_completion(
        &self,
        request: ChatRequest,
        context: RequestContext,
    ) -> Result<ChatResponse> {
        let start_time = Instant::now();

        // Wrap request and context in Arc to avoid cloning on each retry
        // This is especially important for large message arrays
        let request = Arc::new(request);
        let context = Arc::new(context);

        // Execute request with retry policy
        let result = self
            .retry_policy
            .execute(|| {
                let request = Arc::clone(&request);
                let context = Arc::clone(&context);
                let load_balancer = self.load_balancer.clone();

                async move {
                    // Select provider using load balancer
                    let provider = load_balancer
                        .select_provider(&request.model, &context)
                        .await?;

                    // Clone only when needed for the actual API call
                    // The provider API requires owned values
                    provider
                        .chat_completion((*request).clone(), (*context).clone())
                        .await
                        .map_err(|e| {
                            crate::utils::error::GatewayError::Provider(
                                crate::core::providers::unified_provider::ProviderError::Other {
                                    provider: "unknown",
                                    message: e.to_string(),
                                },
                            )
                        })
                }
            })
            .await;

        // Record metrics (run concurrently with result processing)
        let duration = start_time.elapsed();
        let provider_name = if let Ok(provider) = self
            .load_balancer
            .select_provider(&request.model, &context)
            .await
        {
            provider.name().to_string()
        } else {
            "unknown".to_string()
        };

        // Record metrics asynchronously
        let metrics = self.metrics.clone();
        let model = request.model.clone();
        let success = result.is_ok();
        tokio::spawn(async move {
            let _ = metrics
                .record_request(&provider_name, &model, duration, success)
                .await;
        });

        result
    }

    /// Route a completion request
    pub async fn route_completion(
        &self,
        request: CompletionRequest,
        context: RequestContext,
    ) -> Result<CompletionResponse> {
        let start_time = Instant::now();

        // Select provider using load balancer
        let provider = self
            .load_balancer
            .select_provider(&request.model, &context)
            .await?;

        // Convert CompletionRequest to ChatRequest (the provider expects ChatRequest)
        let chat_request = ChatRequest {
            model: request.model.clone(),
            messages: vec![crate::core::types::ChatMessage {
                role: crate::core::types::MessageRole::User,
                content: Some(crate::core::types::MessageContent::Text(
                    request.prompt,
                )),
                ..Default::default()
            }],
            temperature: request.temperature,
            max_tokens: request.max_tokens,
            max_completion_tokens: None,
            top_p: request.top_p,
            frequency_penalty: request.frequency_penalty,
            presence_penalty: request.presence_penalty,
            stop: request.stop,
            stream: false,
            tools: None,
            tool_choice: None,
            parallel_tool_calls: None,
            response_format: None,
            user: request.user,
            seed: None,
            n: None,
            logit_bias: None,
            functions: None,
            function_call: None,
            logprobs: None,
            top_logprobs: None,
            thinking: None,
            extra_params: HashMap::new(),
        };

        // Execute request
        let chat_result = provider
            .chat_completion(chat_request, context.clone())
            .await?;

        // Convert ChatResponse to CompletionResponse
        let result = CompletionResponse {
            id: chat_result.id,
            object: "text_completion".to_string(),
            created: chat_result.created,
            model: chat_result.model,
            choices: chat_result
                .choices
                .into_iter()
                .map(|choice| {
                    let text = match choice.message.content {
                        Some(crate::core::types::MessageContent::Text(content)) => {
                            content
                        }
                        Some(crate::core::types::MessageContent::Parts(_)) => {
                            "".to_string()
                        }
                        None => "".to_string(),
                    };
                    crate::core::types::responses::CompletionChoice {
                        text,
                        index: choice.index,
                        finish_reason: choice.finish_reason,
                        logprobs: None,
                    }
                })
                .collect(),
            usage: chat_result.usage,
            system_fingerprint: chat_result.system_fingerprint,
        };

        // Record metrics
        let duration = start_time.elapsed();
        self.metrics
            .record_request(provider.name(), &request.model, duration, true)
            .await;

        Ok(result)
    }

    /// Route an embedding request
    pub async fn route_embedding(
        &self,
        request: EmbeddingRequest,
        context: RequestContext,
    ) -> Result<EmbeddingResponse> {
        let start_time = Instant::now();

        // Select provider using load balancer
        let provider = self
            .load_balancer
            .select_provider(&request.model, &context)
            .await?;

        // Save model name for metrics before moving request
        let model_name = request.model.clone();
        let provider_name = provider.name();

        // Execute request - pass owned values, no cloning needed
        let result = provider.create_embeddings(request, context).await?;

        // Record metrics
        let duration = start_time.elapsed();
        self.metrics
            .record_request(provider_name, &model_name, duration, true)
            .await;

        Ok(result)
    }

    /// Get router health status
    pub async fn health_status(&self) -> Result<RouterHealthStatus> {
        self.health_checker.get_status().await
    }

    /// Get router metrics
    pub async fn get_metrics(&self) -> Result<RouterMetricsSnapshot> {
        self.metrics.get_snapshot().await
    }

    /// Route chat completion request with streaming
    pub async fn route_chat_completion_stream(
        &self,
        request: ChatRequest,
        context: RequestContext,
    ) -> Result<impl futures::Stream<Item = Result<ChatChunk>>> {
        use futures::StreamExt;

        // Find the best provider for this request
        let model = &request.model;
        let provider = self.load_balancer.select_provider(model, &context).await?;

        // Route to the selected provider for streaming
        let stream = provider.chat_completion_stream(request, context).await?;

        // Map the error types from UnifiedProviderError to GatewayError
        let mapped_stream =
            stream.map(|result| result.map_err(crate::utils::error::GatewayError::from));

        Ok(mapped_stream)
    }

    /// Route image generation request
    pub async fn route_image_generation(
        &self,
        request: ImageGenerationRequest,
        context: RequestContext,
    ) -> Result<ImageGenerationResponse> {
        // Find the best provider for this request
        let default_model = "dall-e-3".to_string();
        let model = request.model.as_ref().unwrap_or(&default_model);
        let provider = self.load_balancer.select_provider(model, &context).await?;

        // Route to the selected provider
        provider
            .create_images(request, context)
            .await
            .map_err(crate::utils::error::GatewayError::from)
    }

    /// List all available models
    pub async fn list_models(&self) -> Result<Vec<ModelInfo>> {
        // Collect models from all providers
        let providers = self.providers.read().await;

        // Pre-estimate capacity: assume ~10 models per provider on average
        let estimated_capacity = providers.len() * 10;
        let mut all_models = Vec::with_capacity(estimated_capacity);

        for provider in providers.values() {
            let models = provider.list_models();
            all_models.extend(models.iter().cloned());
        }

        Ok(all_models)
    }

    /// Get specific model information
    pub async fn get_model(&self, model_id: &str) -> Result<Option<ModelInfo>> {
        // Try to find the model in any provider
        let providers = self.providers.read().await;
        for provider in providers.values() {
            match provider.get_model(model_id).await {
                Ok(Some(model)) => return Ok(Some(model)),
                Ok(None) => continue,
                Err(e) => warn!(
                    "Failed to get model {} from provider {}: {}",
                    model_id,
                    provider.name(),
                    e
                ),
            }
        }

        Ok(None)
    }

    /// Add a new provider
    pub async fn add_provider(&self, config: ProviderConfig) -> Result<()> {
        info!("Adding provider: {}", config.name);

        // Create provider instance
        let provider = crate::core::providers::create_provider(config.clone()).await?;

        // Add to providers registry
        let mut providers = self.providers.write().await;
        providers.register(provider);

        // Update health checker
        self.health_checker.add_provider(&config.name).await?;

        // TODO: Update load balancer - need to fix provider ownership issue

        info!("Provider {} added successfully", config.name);
        Ok(())
    }

    /// Remove a provider
    pub async fn remove_provider(&self, name: &str) -> Result<()> {
        info!("Removing provider: {}", name);

        // Remove from providers map
        let mut providers = self.providers.write().await;
        providers.remove(name);

        // Update health checker
        self.health_checker.remove_provider(name).await?;

        // Update load balancer
        self.load_balancer.remove_provider(name).await?;

        info!("Provider {} removed successfully", name);
        Ok(())
    }

    /// List all providers
    pub async fn list_providers(&self) -> Result<Vec<String>> {
        let providers = self.providers.read().await;
        Ok(providers.list())
    }
}

#[cfg(test)]
mod tests {
    //! Unit tests for the legacy Router implementation
    //!
    //! These tests verify the core router functionality including:
    //! - Router initialization with different configurations
    //! - Routing strategies
    //! - Provider config structure

    use crate::config::ProviderConfig;
    use crate::core::router::strategy::types::RoutingStrategy;

    /// Helper to create a test provider config
    fn create_test_provider_config(name: &str, provider_type: &str) -> ProviderConfig {
        ProviderConfig {
            name: name.to_string(),
            provider_type: provider_type.to_string(),
            ..Default::default()
        }
    }

    #[test]
    fn test_provider_config_creation() {
        let config = create_test_provider_config("openai", "openai");
        assert_eq!(config.name, "openai");
        assert_eq!(config.provider_type, "openai");
    }

    #[test]
    fn test_routing_strategy_variants() {
        // Test that all routing strategy variants are available
        let _ = RoutingStrategy::RoundRobin;
        let _ = RoutingStrategy::Random;
        let _ = RoutingStrategy::LeastLatency;
        let _ = RoutingStrategy::LeastCost;
        let _ = RoutingStrategy::Weighted;
        let _ = RoutingStrategy::Priority;
    }

    #[test]
    fn test_routing_strategy_default() {
        let strategy = RoutingStrategy::default();
        assert!(matches!(strategy, RoutingStrategy::RoundRobin));
    }

    #[test]
    fn test_routing_strategy_clone() {
        let strategy = RoutingStrategy::LeastLatency;
        let cloned = strategy.clone();
        assert!(matches!(cloned, RoutingStrategy::LeastLatency));
    }

    #[test]
    fn test_provider_config_with_defaults() {
        let config = ProviderConfig::default();
        assert!(config.enabled);
        assert_eq!(config.weight, 1.0);
    }

    #[test]
    fn test_multiple_provider_configs() {
        let configs = [
            create_test_provider_config("openai", "openai"),
            create_test_provider_config("anthropic", "anthropic"),
            create_test_provider_config("azure", "azure"),
        ];

        assert_eq!(configs.len(), 3);
        assert_eq!(configs[0].name, "openai");
        assert_eq!(configs[1].name, "anthropic");
        assert_eq!(configs[2].name, "azure");
    }
}
