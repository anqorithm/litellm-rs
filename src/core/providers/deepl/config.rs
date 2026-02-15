//! DeepL Configuration

use crate::define_provider_config;

define_provider_config!(DeepLConfig {
    use_pro: bool = false,
}, env_key: "DEEPL_API_KEY", provider: "deepl");

impl DeepLConfig {
    pub fn with_pro(mut self, use_pro: bool) -> Self {
        self.use_pro = use_pro;
        if use_pro {
            self.base.api_base = Some(super::PRO_BASE_URL.to_string());
        } else {
            self.base.api_base = Some(super::DEFAULT_BASE_URL.to_string());
        }
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::traits::provider::ProviderConfig;

    #[test]
    fn test_default_config() {
        let config = DeepLConfig::default();
        assert_eq!(config.base.timeout, 60);
        assert!(!config.use_pro);
    }

    #[test]
    fn test_config_validation() {
        let mut config = DeepLConfig::default();
        assert!(config.validate().is_err());

        config.base.api_key = Some("test-key".to_string());
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_pro_api_config() {
        let config = DeepLConfig::new("deepl")
            .with_api_key("test-key")
            .with_pro(true);
        assert_eq!(
            config.base.api_base.as_deref(),
            Some(super::super::PRO_BASE_URL)
        );
        assert!(config.use_pro);

        let config = config.with_pro(false);
        assert_eq!(
            config.base.api_base.as_deref(),
            Some(super::super::DEFAULT_BASE_URL)
        );
        assert!(!config.use_pro);
    }
}
