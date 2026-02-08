//! Fallback selection for LoadBalancer

use super::core::LoadBalancer;
use crate::core::providers::Provider;
use crate::core::providers::unified_provider::ProviderError;
use crate::core::types::context::RequestContext;
use tracing::{debug, info, warn};

impl LoadBalancer {
    /// Select fallback model based on error type
    ///
    /// Returns an ordered list of fallback models to try based on the error type.
    pub fn select_fallback_models(
        &self,
        error: &ProviderError,
        original_model: &str,
    ) -> Option<Vec<String>> {
        let specific_fallbacks = match error {
            ProviderError::ContextLengthExceeded { .. } => {
                debug!(
                    "Looking for context window fallbacks for model: {}",
                    original_model
                );
                self.fallback_config
                    .context_window_fallbacks
                    .get(original_model)
            }
            ProviderError::ContentFiltered { .. } => {
                debug!(
                    "Looking for content policy fallbacks for model: {}",
                    original_model
                );
                self.fallback_config
                    .content_policy_fallbacks
                    .get(original_model)
            }
            ProviderError::RateLimit { .. } => {
                debug!(
                    "Looking for rate limit fallbacks for model: {}",
                    original_model
                );
                self.fallback_config
                    .rate_limit_fallbacks
                    .get(original_model)
            }
            _ => None,
        };

        if let Some(fallbacks) = specific_fallbacks {
            if !fallbacks.is_empty() {
                info!(
                    "Found error-specific fallbacks for {}: {:?}",
                    original_model, fallbacks
                );
                return Some(fallbacks.clone());
            }
        }

        if let Some(general) = self.fallback_config.general_fallbacks.get(original_model) {
            if !general.is_empty() {
                info!(
                    "Using general fallbacks for {}: {:?}",
                    original_model, general
                );
                return Some(general.clone());
            }
        }

        debug!("No fallbacks configured for model: {}", original_model);
        None
    }

    /// Select fallback provider for error with context
    pub async fn select_fallback_provider(
        &self,
        error: &ProviderError,
        original_model: &str,
        context: &RequestContext,
    ) -> Option<(String, Provider)> {
        let fallback_models = self.select_fallback_models(error, original_model)?;

        for fallback_model in fallback_models {
            match self.select_provider(&fallback_model, context).await {
                Ok(provider) => {
                    info!(
                        "Selected fallback: model={}, provider for original={}",
                        fallback_model, original_model
                    );
                    return Some((fallback_model, provider));
                }
                Err(e) => {
                    warn!("Fallback model {} not available: {}", fallback_model, e);
                    continue;
                }
            }
        }

        warn!(
            "No fallback providers available for model: {}",
            original_model
        );
        None
    }
}
