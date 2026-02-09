use super::*;

#[test]
fn test_datadog_config_builder() {
    let config = DataDogConfig::new("test-api-key")
        .site("datadoghq.eu")
        .service("my-service")
        .env("production")
        .version("1.0.0")
        .tag("team", "platform");

    assert_eq!(config.api_key, "test-api-key");
    assert_eq!(config.site, "datadoghq.eu");
    assert_eq!(config.service, "my-service");
    assert_eq!(config.env, Some("production".to_string()));
    assert_eq!(config.version, Some("1.0.0".to_string()));
    assert_eq!(config.tags.get("team"), Some(&"platform".to_string()));
}

#[test]
fn test_datadog_config_urls() {
    let config = DataDogConfig::new("test-key").site("datadoghq.eu");

    assert!(config.metrics_url().contains("datadoghq.eu"));
    assert!(config.logs_url().contains("datadoghq.eu"));
    assert!(config.traces_url().contains("datadoghq.eu"));
}

#[test]
fn test_datadog_config_default() {
    let config = DataDogConfig::default();

    assert_eq!(config.site, "datadoghq.com");
    assert_eq!(config.service, "litellm-gateway");
    assert!(config.enable_metrics);
    assert!(config.enable_traces);
    assert!(config.enable_logs);
}

#[test]
fn test_datadog_integration_requires_api_key() {
    let config = DataDogConfig::default();
    let result = DataDogIntegration::new(config);
    assert!(result.is_err());
}

#[test]
fn test_datadog_integration_creation() {
    let config = DataDogConfig::new("test-api-key");
    let result = DataDogIntegration::new(config);
    assert!(result.is_ok());

    let integration = result.unwrap();
    assert_eq!(integration.name(), "datadog");
    assert!(integration.is_enabled());
}

#[test]
fn test_build_tags() {
    let config = DataDogConfig::new("test-key")
        .service("test-service")
        .env("test")
        .tag("custom", "value");
    let integration = DataDogIntegration::new(config).unwrap();

    let tags = integration.build_tags(&[("extra", "tag")]);

    assert!(tags.contains(&"service:test-service".to_string()));
    assert!(tags.contains(&"env:test".to_string()));
    assert!(tags.contains(&"custom:value".to_string()));
    assert!(tags.contains(&"extra:tag".to_string()));
}
