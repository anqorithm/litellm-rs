//! Provider selection methods for LoadBalancer

use super::core::LoadBalancer;
use super::deployment_info::DeploymentInfo;
use crate::core::providers::Provider;
use crate::core::types::context::RequestContext;
use crate::utils::error::error::{GatewayError, Result};
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{debug, info};

impl LoadBalancer {
    /// Select a provider for the given model and context
    pub async fn select_provider(&self, model: &str, context: &RequestContext) -> Result<Provider> {
        let supporting_providers = self.get_supporting_providers(model).await?;

        if supporting_providers.is_empty() {
            return Err(GatewayError::NoProvidersForModel(model.to_string()));
        }

        let healthy_providers = if let Some(health_checker) = &self.health_checker {
            let healthy_list = health_checker.get_healthy_providers().await?;
            supporting_providers
                .into_iter()
                .filter(|p| healthy_list.contains(p))
                .collect()
        } else {
            supporting_providers
        };

        if healthy_providers.is_empty() {
            return Err(GatewayError::NoHealthyProviders(
                "No healthy providers available".to_string(),
            ));
        }

        let selected_name = self
            .strategy
            .select_provider(&healthy_providers, model, context)
            .await?;

        if let Some(provider_ref) = self.providers.get(&selected_name) {
            Ok(provider_ref.value().clone())
        } else {
            Err(GatewayError::ProviderNotFound(format!(
                "Provider {} not found in load balancer",
                selected_name
            )))
        }
    }

    /// Get providers that support a specific model
    pub(crate) async fn get_supporting_providers(&self, model: &str) -> Result<Vec<String>> {
        if let Some(cached_providers) = self.model_support_cache.get(model) {
            debug!(
                "Found cached providers for model {}: {:?}",
                model,
                cached_providers.value()
            );
            return Ok(cached_providers.value().as_ref().clone());
        }

        let mut supporting_providers = Vec::with_capacity(self.providers.len());

        for entry in self.providers.iter() {
            let (name, provider) = entry.pair();
            if provider.supports_model(model) {
                supporting_providers.push(name.clone());
            }
        }

        self.model_support_cache
            .insert(model.to_string(), Arc::new(supporting_providers.clone()));

        debug!(
            "Providers supporting model {}: {:?}",
            model, supporting_providers
        );
        Ok(supporting_providers)
    }

    /// Add a provider to the load balancer
    pub async fn add_provider(&self, name: &str, provider: Provider) -> Result<()> {
        self.providers.insert(name.to_string(), provider);
        self.deployments.entry(name.to_string()).or_default();
        self.model_support_cache.clear();

        info!("Added provider {} to load balancer", name);
        Ok(())
    }

    /// Add a provider with deployment info
    pub async fn add_provider_with_deployment(
        &self,
        name: &str,
        provider: Provider,
        deployment_info: DeploymentInfo,
    ) -> Result<()> {
        self.providers.insert(name.to_string(), provider);
        self.deployments
            .insert(name.to_string(), deployment_info.clone());
        self.model_support_cache.clear();

        info!(
            "Added provider {} with tags {:?}, group {:?}",
            name, deployment_info.tags, deployment_info.model_group
        );
        Ok(())
    }

    /// Remove a provider from the load balancer
    pub async fn remove_provider(&self, name: &str) -> Result<()> {
        self.providers.remove(name);
        self.deployments.remove(name);
        self.model_support_cache
            .retain(|_, providers| !providers.contains(&name.to_string()));

        info!("Removed provider {} from load balancer", name);
        Ok(())
    }

    /// Get load balancer statistics
    pub async fn get_stats(&self) -> Result<super::core::LoadBalancerStats> {
        let provider_count = self.providers.len();

        let healthy_count = if let Some(health_checker) = &self.health_checker {
            health_checker.get_healthy_providers().await?.len()
        } else {
            provider_count
        };

        let cached_models = self.model_support_cache.len();

        Ok(super::core::LoadBalancerStats {
            total_providers: provider_count,
            healthy_providers: healthy_count,
            cached_models,
        })
    }

    /// Clear model support cache
    pub async fn clear_cache(&self) -> Result<()> {
        self.model_support_cache.clear();
        info!("Cleared model support cache");
        Ok(())
    }

    /// Get cached model support information
    pub async fn get_model_cache(&self) -> Result<HashMap<String, Vec<String>>> {
        let mut result = HashMap::with_capacity(self.model_support_cache.len());
        for entry in self.model_support_cache.iter() {
            let (key, value) = entry.pair();
            result.insert(key.clone(), value.as_ref().clone());
        }
        Ok(result)
    }

    /// Preload model support cache for common models
    pub async fn preload_cache(&self, models: &[String]) -> Result<()> {
        info!("Preloading model support cache for {} models", models.len());

        for model in models {
            self.get_supporting_providers(model).await?;
        }

        info!("Model support cache preloaded successfully");
        Ok(())
    }
}
