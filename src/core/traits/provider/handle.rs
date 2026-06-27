//! Legacy provider handle metadata wrapper
//!
//! Router deployments dispatch through the closed `Provider` enum. This wrapper
//! keeps source compatibility for older type-erased experiments, but it is not a
//! router dispatch path.

use crate::core::types::health::HealthStatus;
use crate::core::types::{chat::ChatRequest, context::RequestContext, responses::ChatResponse};
use crate::utils::error::gateway_error::GatewayError;

use super::llm_provider::trait_definition::LLMProvider;

/// Legacy provider handle metadata wrapper.
///
/// This struct stores a provider name, weight, and enabled flag alongside an
/// erased provider value for compatibility with older APIs. It does not
/// downcast the provider and cannot make an arbitrary `LLMProvider`
/// implementation routeable. Router deployments currently use the built-in
/// `Provider` enum and its dispatch arms.
///
/// # Design Principles
/// - Preserve source compatibility for legacy metadata usage
/// - Avoid optimistic capability, health, cost, or success-rate data
/// - Keep router dispatch explicit in the `Provider` enum
///
/// # Example
///
/// `ProviderHandle::new` accepts any type implementing `LLMProvider`, but this
/// only captures metadata. The provider is not automatically available to the
/// router unless it is also wired into the `Provider` enum.
pub struct ProviderHandle {
    name: String,
    _provider: std::sync::Arc<dyn std::any::Any + Send + Sync>,
    weight: f64,
    enabled: bool,
}

impl ProviderHandle {
    /// Create a new legacy provider handle.
    ///
    /// # Parameters
    /// * `provider` - The provider instance to wrap
    /// * `weight` - Routing weight (higher values = more traffic)
    ///
    /// # Returns
    /// A new `ProviderHandle` with the provider enabled by default. This does
    /// not register the provider with the router.
    ///
    /// # Type Parameters
    /// * `P` - Any type implementing LLMProvider
    pub fn new<P>(provider: P, weight: f64) -> Self
    where
        P: LLMProvider + Send + Sync + 'static,
    {
        Self {
            name: provider.name().to_string(),
            _provider: std::sync::Arc::new(provider)
                as std::sync::Arc<dyn std::any::Any + Send + Sync>,
            weight,
            enabled: true,
        }
    }

    /// Get provider name
    ///
    /// # Returns
    /// The provider's identifier string
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get routing weight
    ///
    /// # Returns
    /// The weight used for weighted routing strategies
    pub fn weight(&self) -> f64 {
        self.weight
    }

    /// Check if provider is enabled
    ///
    /// # Returns
    /// `true` if the provider can receive traffic, `false` otherwise
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Set enabled state
    ///
    /// # Parameters
    /// * `enabled` - Whether to enable or disable this provider
    ///
    /// # Use Cases
    /// - Disable unhealthy providers automatically
    /// - Manual traffic control
    /// - Gradual rollout/rollback
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    /// Execute chat completion request.
    ///
    /// # Parameters
    /// * `request` - Chat completion request
    /// * `context` - Request context with metadata
    ///
    /// # Returns
    /// Chat completion response
    ///
    /// # Note
    /// `ProviderHandle` is not a router dispatch path. Use a `Deployment` with
    /// a built-in `Provider` enum variant for routed chat completion.
    pub async fn chat_completion(
        &self,
        _request: ChatRequest,
        _context: RequestContext,
    ) -> Result<ChatResponse, GatewayError> {
        Err(Self::unsupported_contract_error("chat_completion"))
    }

    /// Check if model is supported
    ///
    /// # Parameters
    /// * `model` - Model name to check
    ///
    /// # Returns
    /// `false` because this wrapper has no verified model metadata.
    pub fn supports_model(&self, _model: &str) -> bool {
        false
    }

    /// Check if tools are supported
    ///
    /// # Returns
    /// `false` because this wrapper has no verified capability metadata.
    pub fn supports_tools(&self) -> bool {
        false
    }

    /// Check provider health status
    ///
    /// # Returns
    /// `Unknown` because this wrapper cannot call the provider health endpoint.
    pub async fn health_check(&self) -> HealthStatus {
        HealthStatus::Unknown
    }

    /// Calculate request cost
    ///
    /// # Parameters
    /// * `model` - Model name used
    /// * `input` - Number of input tokens
    /// * `output` - Number of output tokens
    ///
    /// # Returns
    /// Explicit error because this wrapper cannot calculate verified cost.
    pub async fn calculate_cost(
        &self,
        _model: &str,
        _input: u32,
        _output: u32,
    ) -> Result<f64, GatewayError> {
        Err(Self::unsupported_contract_error("calculate_cost"))
    }

    /// Get average response latency
    ///
    /// # Returns
    /// Explicit error because this wrapper has no verified latency telemetry.
    pub async fn get_average_latency(&self) -> Result<std::time::Duration, GatewayError> {
        Err(Self::unsupported_contract_error("get_average_latency"))
    }

    /// Get success rate
    ///
    /// # Returns
    /// Explicit error because this wrapper has no verified success telemetry.
    pub async fn get_success_rate(&self) -> Result<f32, GatewayError> {
        Err(Self::unsupported_contract_error("get_success_rate"))
    }

    fn unsupported_contract_error(method: &str) -> GatewayError {
        GatewayError::Internal(format!(
            "ProviderHandle::{method} is not a router dispatch path; router deployments use the built-in Provider enum"
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_handle() -> ProviderHandle {
        ProviderHandle {
            name: "legacy".to_string(),
            _provider: std::sync::Arc::new(()),
            weight: 1.0,
            enabled: true,
        }
    }

    #[test]
    fn test_provider_handle_metadata_accessors() {
        let mut handle = test_handle();

        assert_eq!(handle.name(), "legacy");
        assert_eq!(handle.weight(), 1.0);
        assert!(handle.is_enabled());

        handle.set_enabled(false);
        assert!(!handle.is_enabled());
    }

    #[tokio::test]
    async fn provider_handle_does_not_report_optimistic_routing_data() {
        let handle = test_handle();

        assert!(!handle.supports_model("gpt-4"));
        assert!(!handle.supports_tools());
        assert_eq!(handle.health_check().await, HealthStatus::Unknown);

        let cost = handle.calculate_cost("gpt-4", 1, 1).await;
        assert!(
            matches!(cost, Err(GatewayError::Internal(message)) if message.contains("ProviderHandle::calculate_cost"))
        );

        let latency = handle.get_average_latency().await;
        assert!(
            matches!(latency, Err(GatewayError::Internal(message)) if message.contains("ProviderHandle::get_average_latency"))
        );

        let success_rate = handle.get_success_rate().await;
        assert!(
            matches!(success_rate, Err(GatewayError::Internal(message)) if message.contains("ProviderHandle::get_success_rate"))
        );
    }

    #[test]
    fn test_request_context_default() {
        let context = RequestContext::default();
        // Verify default context is created with a request_id
        assert!(!context.request_id.is_empty());
    }

    #[test]
    fn test_chat_request_default() {
        let request = ChatRequest {
            model: "test-model".to_string(),
            messages: vec![],
            ..Default::default()
        };

        assert_eq!(request.model, "test-model");
        assert!(request.messages.is_empty());
    }

    #[tokio::test]
    async fn provider_handle_chat_completion_returns_explicit_contract_error() {
        let handle = test_handle();
        let request = ChatRequest {
            model: "test-model".to_string(),
            messages: vec![],
            ..Default::default()
        };

        let result = handle
            .chat_completion(request, RequestContext::default())
            .await;

        assert!(
            matches!(result, Err(GatewayError::Internal(message)) if message.contains("ProviderHandle::chat_completion"))
        );
    }
}
