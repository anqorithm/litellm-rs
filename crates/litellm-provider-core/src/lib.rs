use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;

pub type DynProvider = Arc<dyn Provider + Send + Sync>;

#[derive(Debug, thiserror::Error)]
pub enum ProviderError {
    #[error("provider not found: {0}")]
    NotFound(String),
    #[error("provider init error: {0}")]
    Init(String),
    #[error("provider request error: {0}")]
    Request(String),
}

#[async_trait]
pub trait Provider: std::fmt::Debug + Send + Sync {
    fn id(&self) -> &str;
    async fn complete(&self, model: &str, input: &str) -> Result<String, ProviderError>;
}

#[async_trait]
pub trait ProviderFactory: Send + Sync {
    fn provider_type(&self) -> &str;
    async fn create(
        &self,
        config: &litellm_contracts::ProviderConfig,
    ) -> Result<DynProvider, ProviderError>;
}

#[derive(Default)]
pub struct ProviderRegistry {
    factories: HashMap<String, Arc<dyn ProviderFactory>>,
}

impl ProviderRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register_factory(&mut self, factory: Arc<dyn ProviderFactory>) {
        self.factories
            .insert(factory.provider_type().to_string(), factory);
    }

    pub async fn create_provider(
        &self,
        config: &litellm_contracts::ProviderConfig,
    ) -> Result<DynProvider, ProviderError> {
        let Some(factory) = self.factories.get(&config.provider_type) else {
            return Err(ProviderError::NotFound(config.provider_type.clone()));
        };
        factory.create(config).await
    }
}
