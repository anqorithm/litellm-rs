use crate::{Deployment, Router, RouterError};

impl Router {
    pub async fn from_gateway_config(
        providers: &[litellm_contracts::ProviderConfig],
        factory: &litellm_provider_core::ProviderRegistry,
    ) -> Result<Self, RouterError> {
        let mut router = Router::new();
        for provider_cfg in providers.iter().filter(|p| p.enabled) {
            let provider = factory
                .create_provider(provider_cfg)
                .await
                .map_err(|e| RouterError::Provider(e.to_string()))?;

            if provider_cfg.models.is_empty() {
                let deployment = Deployment::new(
                    provider_cfg.name.clone(),
                    provider_cfg.name.clone(),
                    provider.clone(),
                );
                router.add_deployment(deployment);
            } else {
                for model in &provider_cfg.models {
                    let deployment_id = format!("{}-{model}", provider_cfg.name);
                    let deployment =
                        Deployment::new(deployment_id, model.clone(), provider.clone());
                    router.add_deployment(deployment);
                }
            }
        }

        Ok(router)
    }
}
