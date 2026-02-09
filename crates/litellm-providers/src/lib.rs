pub mod sdk;

use async_trait::async_trait;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct EchoProvider {
    id: String,
}

impl EchoProvider {
    pub fn new(id: impl Into<String>) -> Self {
        Self { id: id.into() }
    }
}

#[async_trait]
impl litellm_provider_core::Provider for EchoProvider {
    fn id(&self) -> &str {
        &self.id
    }

    async fn complete(
        &self,
        model: &str,
        input: &str,
    ) -> Result<String, litellm_provider_core::ProviderError> {
        Ok(format!("[{model}] {input}"))
    }
}

#[derive(Debug, Default)]
pub struct EchoProviderFactory;

#[async_trait]
impl litellm_provider_core::ProviderFactory for EchoProviderFactory {
    fn provider_type(&self) -> &str {
        "echo"
    }

    async fn create(
        &self,
        config: &litellm_contracts::ProviderConfig,
    ) -> Result<litellm_provider_core::DynProvider, litellm_provider_core::ProviderError> {
        Ok(Arc::new(EchoProvider::new(config.name.clone())))
    }
}

pub fn default_registry() -> litellm_provider_core::ProviderRegistry {
    let mut registry = litellm_provider_core::ProviderRegistry::new();
    registry.register_factory(Arc::new(EchoProviderFactory));
    registry
}
