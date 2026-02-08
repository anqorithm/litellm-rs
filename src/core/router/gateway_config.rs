//! Gateway configuration integration
//!
//! This module contains the from_gateway_config method for creating
//! a Router from gateway configuration.

use super::config::RouterConfig;
use super::deployment::{Deployment, DeploymentConfig};
use super::error::RouterError;
use super::unified::Router;
use crate::config::models::provider::ProviderConfig;
use crate::core::providers::{Provider, ProviderType};

impl Router {
    /// Create a Router from gateway configuration
    ///
    /// This method initializes a Router with deployments created from provider configurations.
    /// Each provider in the config becomes a deployment in the router.
    pub async fn from_gateway_config(
        providers: &[ProviderConfig],
        router_config: Option<RouterConfig>,
    ) -> Result<Self, RouterError> {
        let config = router_config.unwrap_or_default();
        let router = Self::new(config);

        for provider_config in providers {
            if !provider_config.enabled {
                continue;
            }

            // Parse provider type
            let provider_type: ProviderType = provider_config.provider_type.as_str().into();

            // Build config JSON from provider settings
            let mut settings = provider_config.settings.clone();

            // Add api_key from the config if not in settings
            if !settings.contains_key("api_key") && !provider_config.api_key.is_empty() {
                settings.insert(
                    "api_key".to_string(),
                    serde_json::Value::String(provider_config.api_key.clone()),
                );
            }

            // Add base_url if present
            if let Some(ref base_url) = provider_config.base_url {
                settings.insert(
                    "base_url".to_string(),
                    serde_json::Value::String(base_url.clone()),
                );
            }

            // Create provider instance
            let provider = Provider::from_config_async(
                provider_type.clone(),
                serde_json::Value::Object(settings.into_iter().collect()),
            )
            .await
            .map_err(|e| {
                RouterError::DeploymentNotFound(format!(
                    "Failed to create provider {}: {}",
                    provider_config.name, e
                ))
            })?;

            // Determine which models this deployment serves
            let models: Vec<String> = if !provider_config.models.is_empty() {
                provider_config.models.clone()
            } else {
                provider
                    .list_models()
                    .iter()
                    .map(|m| m.id.clone())
                    .collect()
            };

            // Create deployments
            if models.is_empty() {
                // Create a single deployment with provider name
                let deployment = create_deployment_from_config(
                    &provider_config.name,
                    provider.clone(),
                    &provider_config.name,
                    provider_config,
                );
                router.add_deployment(deployment);
            } else {
                // Create one deployment per model
                for model in models {
                    let deployment_id = format!("{}-{}", provider_config.name, model);
                    let deployment = create_deployment_from_config(
                        &deployment_id,
                        provider.clone(),
                        &model,
                        provider_config,
                    );
                    router.add_deployment(deployment);
                }
            }
        }

        Ok(router)
    }
}

/// Helper function to create deployment from provider config
fn create_deployment_from_config(
    deployment_id: &str,
    provider: Provider,
    model: &str,
    config: &ProviderConfig,
) -> Deployment {
    let deployment_config = DeploymentConfig {
        tpm_limit: if config.tpm > 0 {
            Some(config.tpm as u64)
        } else {
            None
        },
        rpm_limit: if config.rpm > 0 {
            Some(config.rpm as u64)
        } else {
            None
        },
        max_parallel_requests: if config.max_concurrent_requests > 0 {
            Some(config.max_concurrent_requests)
        } else {
            None
        },
        weight: config.weight as u32,
        timeout_secs: config.timeout,
        priority: 0,
    };

    Deployment::new(
        deployment_id.to_string(),
        provider,
        model.to_string(),
        model.to_string(),
    )
    .with_config(deployment_config)
    .with_tags(config.tags.clone())
}
