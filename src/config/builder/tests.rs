//! Tests for configuration builders

#[cfg(test)]
mod unit_tests {
    use super::super::presets;
    use super::super::types::{ConfigBuilder, ProviderConfigBuilder};

    #[test]
    fn test_config_builder() {
        let config = ConfigBuilder::new()
            .with_server(presets::dev_server().build())
            .add_provider(
                presets::openai_provider("openai", "test-key")
                    .unwrap()
                    .build()
                    .unwrap(),
            )
            .enable_feature("metrics")
            .build();

        assert!(config.is_ok());
        let config = config.unwrap();
        assert_eq!(config.gateway.server.port, 8080);
        assert_eq!(config.gateway.providers.len(), 1);
    }

    #[test]
    fn test_provider_builder() {
        let provider = ProviderConfigBuilder::new()
            .name("test-provider")
            .unwrap()
            .provider_type("openai")
            .unwrap()
            .api_key("test-key")
            .add_model("gpt-4")
            .weight(2.0)
            .unwrap()
            .build();

        assert!(provider.is_ok());
        let provider = provider.unwrap();
        assert_eq!(provider.name, "test-provider");
        assert_eq!(provider.weight, 2.0);
    }
}
