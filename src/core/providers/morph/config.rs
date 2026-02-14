//! Morph AI Provider Configuration

use crate::define_standalone_provider_config;

define_standalone_provider_config!(MorphConfig,
    provider: "Morph",
    env_prefix: "MORPH",
    default_base_url: "https://api.morph.so/v1",
    default_timeout: 60,
    extra_fields: { debug: bool = false },
);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_morph_config_default() {
        let config = MorphConfig::default();
        assert!(config.api_key.is_none());
        assert_eq!(config.timeout, 60);
        assert_eq!(config.max_retries, 3);
    }

    #[test]
    fn test_morph_config_get_api_base() {
        let config = MorphConfig::default();
        assert_eq!(config.get_api_base(), "https://api.morph.so/v1");
    }

    #[test]
    fn test_morph_config_with_api_key() {
        let config = MorphConfig::new("test-key");
        assert_eq!(config.get_api_key(), Some("test-key".to_string()));
    }
}
