//! Standard provider macro
//!
//! `standard_provider!` creates a basic provider implementation with struct,
//! client, and LLMProvider trait impl.

/// Macro to create standard provider implementation
#[macro_export]
macro_rules! standard_provider {
    (
        name: $name:literal,
        struct: $struct_name:ident,
        config: $config_type:ty,
        capabilities: [$($cap:expr),* $(,)?],
        models: [$($model:literal),* $(,)?],
        owner: $owner:literal
    ) => {
        pub struct $struct_name {
            config: $config_type,
            client: reqwest::Client,
            capabilities: Vec<ProviderCapability>,
        }

        impl $struct_name {
            pub fn new(config: $config_type) -> Result<Self, ProviderError> {
                let client = $crate::utils::net::http::create_custom_client(
                    std::time::Duration::from_secs(config.timeout),
                )
                .map_err(|e| ProviderError::Configuration {
                    provider: $name,
                    message: format!("Failed to create HTTP client: {}", e),
                })?;

                let capabilities = vec![$($cap),*];

                Ok(Self {
                    config,
                    client,
                    capabilities,
                })
            }
        }

                impl LLMProvider for $struct_name {
            fn name(&self) -> &str {
                $name
            }

            fn capabilities(&self) -> &[ProviderCapability] {
                &self.capabilities
            }

            fn list_models(&self) -> Vec<ModelInfo> {
                model_list!($owner, $($model),*)
            }

            async fn get_model(&self, model_id: &str) -> Result<Option<ModelInfo>, ProviderError> {
                Ok(self.list_models().into_iter().find(|m| m.id == model_id))
            }

            impl_health_check!($struct_name, "models");
        }
    };
}
