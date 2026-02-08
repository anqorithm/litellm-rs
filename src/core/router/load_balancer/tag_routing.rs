//! Tag and group-based routing for LoadBalancer

use super::core::LoadBalancer;
use crate::core::providers::Provider;
use crate::core::types::context::RequestContext;
use crate::utils::error::{GatewayError, Result};
use tracing::debug;

impl LoadBalancer {
    /// Select a provider with tag filtering
    pub async fn select_provider_with_tags(
        &self,
        model: &str,
        tags: &[String],
        require_all_tags: bool,
        context: &RequestContext,
    ) -> Result<Provider> {
        let supporting_providers = self.get_supporting_providers(model).await?;

        if supporting_providers.is_empty() {
            return Err(GatewayError::NoProvidersForModel(model.to_string()));
        }

        let tagged_providers: Vec<String> = supporting_providers
            .into_iter()
            .filter(|name| {
                self.deployments
                    .get(name)
                    .map(|info| {
                        if require_all_tags {
                            info.has_all_tags(tags)
                        } else {
                            info.has_any_tag(tags)
                        }
                    })
                    .unwrap_or(false)
            })
            .collect();

        if tagged_providers.is_empty() {
            return Err(GatewayError::NoProvidersForModel(format!(
                "{} with tags {:?}",
                model, tags
            )));
        }

        let healthy_providers = if let Some(health_checker) = &self.health_checker {
            let healthy_list = health_checker.get_healthy_providers().await?;
            tagged_providers
                .into_iter()
                .filter(|p| healthy_list.contains(p))
                .collect()
        } else {
            tagged_providers
        };

        if healthy_providers.is_empty() {
            return Err(GatewayError::NoHealthyProviders(
                "No healthy providers with matching tags available".to_string(),
            ));
        }

        let selected_name = self
            .strategy
            .select_provider(&healthy_providers, model, context)
            .await?;

        if let Some(provider_ref) = self.providers.get(&selected_name) {
            debug!(
                "Selected provider {} for model {} with tags {:?}",
                selected_name, model, tags
            );
            Ok(provider_ref.value().clone())
        } else {
            Err(GatewayError::ProviderNotFound(format!(
                "Provider {} not found in load balancer",
                selected_name
            )))
        }
    }

    /// Select a provider by model group
    pub async fn select_provider_by_group(
        &self,
        model: &str,
        group: &str,
        context: &RequestContext,
    ) -> Result<Provider> {
        let supporting_providers = self.get_supporting_providers(model).await?;

        if supporting_providers.is_empty() {
            return Err(GatewayError::NoProvidersForModel(model.to_string()));
        }

        let mut grouped_providers: Vec<(String, u32)> = supporting_providers
            .into_iter()
            .filter_map(|name| {
                self.deployments.get(&name).and_then(|info| {
                    if info.model_group.as_deref() == Some(group) {
                        Some((name, info.priority))
                    } else {
                        None
                    }
                })
            })
            .collect();

        if grouped_providers.is_empty() {
            return Err(GatewayError::NoProvidersForModel(format!(
                "{} in group {}",
                model, group
            )));
        }

        grouped_providers.sort_by_key(|(_, priority)| *priority);

        let provider_names: Vec<String> = grouped_providers
            .into_iter()
            .map(|(name, _)| name)
            .collect();

        let healthy_providers = if let Some(health_checker) = &self.health_checker {
            let healthy_list = health_checker.get_healthy_providers().await?;
            provider_names
                .into_iter()
                .filter(|p| healthy_list.contains(p))
                .collect()
        } else {
            provider_names
        };

        if healthy_providers.is_empty() {
            return Err(GatewayError::NoHealthyProviders(
                "No healthy providers in group available".to_string(),
            ));
        }

        let selected_name = self
            .strategy
            .select_provider(&healthy_providers, model, context)
            .await?;

        if let Some(provider_ref) = self.providers.get(&selected_name) {
            debug!(
                "Selected provider {} for model {} in group {}",
                selected_name, model, group
            );
            Ok(provider_ref.value().clone())
        } else {
            Err(GatewayError::ProviderNotFound(format!(
                "Provider {} not found in load balancer",
                selected_name
            )))
        }
    }

    /// Get all providers with a specific tag
    pub fn get_providers_by_tag(&self, tag: &str) -> Vec<String> {
        self.deployments
            .iter()
            .filter_map(|entry| {
                if entry.value().tags.contains(&tag.to_string()) {
                    Some(entry.key().clone())
                } else {
                    None
                }
            })
            .collect()
    }

    /// Get all providers in a specific model group
    pub fn get_providers_by_group(&self, group: &str) -> Vec<String> {
        self.deployments
            .iter()
            .filter_map(|entry| {
                if entry.value().model_group.as_deref() == Some(group) {
                    Some(entry.key().clone())
                } else {
                    None
                }
            })
            .collect()
    }

    /// Get all unique tags across all deployments
    pub fn get_all_tags(&self) -> Vec<String> {
        let mut tags: Vec<String> = self
            .deployments
            .iter()
            .flat_map(|entry| entry.value().tags.clone())
            .collect();
        tags.sort();
        tags.dedup();
        tags
    }

    /// Get all unique model groups
    pub fn get_all_groups(&self) -> Vec<String> {
        let mut groups: Vec<String> = self
            .deployments
            .iter()
            .filter_map(|entry| entry.value().model_group.clone())
            .collect();
        groups.sort();
        groups.dedup();
        groups
    }
}
