//! Tests for the provider factory module.
//!
//! Extracted from `factory.rs` to keep the main file under 800 lines.

use super::*;

fn supported_factory_provider_types() -> Vec<ProviderType> {
    Provider::factory_supported_provider_types().to_vec()
}

#[tokio::test]
async fn test_from_config_async_supported_variants_do_not_fallthrough_to_not_implemented() {
    for provider_type in supported_factory_provider_types() {
        let err = Provider::from_config_async(provider_type.clone(), serde_json::json!({}))
            .await
            .expect_err("Expected empty config to fail");
        assert!(
            !matches!(err, ProviderError::NotImplemented { .. }),
            "{:?} unexpectedly fell through to NotImplemented: {}",
            provider_type,
            err
        );
    }
}

#[tokio::test]
async fn test_from_config_async_unsupported_variants_return_not_implemented() {
    let supported = supported_factory_provider_types();

    for provider_type in super::super::provider_type::all_non_custom_provider_types() {
        if supported.contains(&provider_type) {
            continue;
        }

        let err = Provider::from_config_async(provider_type.clone(), serde_json::json!({}))
            .await
            .expect_err("Expected unsupported provider to fail");
        assert!(
            matches!(err, ProviderError::NotImplemented { .. }),
            "Expected NotImplemented for {:?}, got {}",
            provider_type,
            err
        );
    }
}

#[test]
fn test_build_openai_config_from_factory_maps_optional_fields() {
    let config = serde_json::json!({
        "api_key": "sk-test123",
        "base_url": "https://example-openai.test/v1",
        "timeout": 42,
        "max_retries": 7,
        "organization": "org-test",
        "project": "proj-test",
        "headers": {
            "x-team-id": "team-1"
        },
        "custom_headers": {
            "x-request-source": "gateway"
        },
        "model_mappings": {
            "gpt-4": "gpt-4o",
            "ignored": 123
        }
    });

    let openai_config = build_openai_config_from_factory(&config)
        .unwrap_or_else(|err| panic!("openai config should parse: {err}"));
    assert_eq!(openai_config.base.api_key.as_deref(), Some("sk-test123"));
    assert_eq!(
        openai_config.base.api_base.as_deref(),
        Some("https://example-openai.test/v1")
    );
    assert_eq!(openai_config.base.timeout, 42);
    assert_eq!(openai_config.base.max_retries, 7);
    assert_eq!(openai_config.organization.as_deref(), Some("org-test"));
    assert_eq!(openai_config.project.as_deref(), Some("proj-test"));
    assert_eq!(
        openai_config
            .base
            .headers
            .get("x-team-id")
            .map(String::as_str),
        Some("team-1")
    );
    assert_eq!(
        openai_config
            .base
            .headers
            .get("x-request-source")
            .map(String::as_str),
        Some("gateway")
    );
    assert_eq!(
        openai_config
            .model_mappings
            .get("gpt-4")
            .map(String::as_str),
        Some("gpt-4o")
    );
    assert!(!openai_config.model_mappings.contains_key("ignored"));
}

#[test]
fn test_build_anthropic_config_from_factory_maps_optional_fields() {
    let config = serde_json::json!({
        "api_key": "sk-ant-test",
        "api_base": "https://example-anthropic.test",
        "api_version": "2024-01-01",
        "timeout": 99,
        "connect_timeout": 12,
        "max_retries": 6,
        "retry_delay_base": 250,
        "proxy": "http://localhost:8080",
        "headers": {
            "x-anthropic-a": "a"
        },
        "custom_headers": {
            "x-anthropic-b": "b"
        },
        "enable_multimodal": false,
        "enable_cache_control": false,
        "enable_computer_use": true,
        "enable_experimental": true
    });

    let anthropic_config = build_anthropic_config_from_factory(&config)
        .unwrap_or_else(|err| panic!("anthropic config should parse: {err}"));
    assert_eq!(anthropic_config.api_key.as_deref(), Some("sk-ant-test"));
    assert_eq!(anthropic_config.base_url, "https://example-anthropic.test");
    assert_eq!(anthropic_config.api_version, "2024-01-01");
    assert_eq!(anthropic_config.request_timeout, 99);
    assert_eq!(anthropic_config.connect_timeout, 12);
    assert_eq!(anthropic_config.max_retries, 6);
    assert_eq!(anthropic_config.retry_delay_base, 250);
    assert_eq!(
        anthropic_config.proxy_url.as_deref(),
        Some("http://localhost:8080")
    );
    assert_eq!(
        anthropic_config
            .custom_headers
            .get("x-anthropic-a")
            .map(String::as_str),
        Some("a")
    );
    assert_eq!(
        anthropic_config
            .custom_headers
            .get("x-anthropic-b")
            .map(String::as_str),
        Some("b")
    );
    assert!(!anthropic_config.enable_multimodal);
    assert!(!anthropic_config.enable_cache_control);
    assert!(anthropic_config.enable_computer_use);
    assert!(anthropic_config.enable_experimental);
}

#[test]
fn test_build_mistral_config_from_factory_maps_optional_fields() {
    let config = serde_json::json!({
        "api_key": "mistral-key",
        "api_base": "https://example-mistral.test/v1",
        "timeout": 88,
        "max_retries": 4
    });

    let mistral_config = build_mistral_config_from_factory(&config)
        .unwrap_or_else(|err| panic!("mistral config should parse: {err}"));
    assert_eq!(mistral_config.api_key, "mistral-key");
    assert_eq!(mistral_config.api_base, "https://example-mistral.test/v1");
    assert_eq!(mistral_config.timeout_seconds, 88);
    assert_eq!(mistral_config.max_retries, 4);
}

#[test]
fn test_build_cloudflare_config_from_factory_maps_alias_and_optional_fields() {
    let config = serde_json::json!({
        "organization": "acct-xyz",
        "api_key": "token-xyz",
        "base_url": "https://cf.example.test",
        "timeout": 77,
        "max_retries": 5,
        "debug": true
    });

    let cf_config = build_cloudflare_config_from_factory(&config)
        .unwrap_or_else(|err| panic!("cloudflare config should parse: {err}"));
    assert_eq!(cf_config.account_id.as_deref(), Some("acct-xyz"));
    assert_eq!(cf_config.api_token.as_deref(), Some("token-xyz"));
    assert_eq!(
        cf_config.api_base.as_deref(),
        Some("https://cf.example.test")
    );
    assert_eq!(cf_config.timeout, 77);
    assert_eq!(cf_config.max_retries, 5);
    assert!(cf_config.debug);
}

#[tokio::test]
async fn test_from_config_async_cloudflare_accepts_alias_fields() {
    let config = serde_json::json!({
        "organization": "acct-alias",
        "api_key": "token-alias"
    });

    let provider = Provider::from_config_async(ProviderType::Cloudflare, config)
        .await
        .unwrap_or_else(|err| {
            panic!("cloudflare should be creatable from alias fields: {err}")
        });
    assert!(matches!(provider, Provider::Cloudflare(_)));
}

#[test]
fn test_build_openai_like_config_from_factory_maps_optional_fields() {
    let config = serde_json::json!({
        "base_url": "https://openai-like.example.test/v1",
        "api_key": "sk-openai-like",
        "provider_name": "custom-like",
        "timeout": 55,
        "max_retries": 4,
        "model_prefix": "prefix/",
        "default_model": "gpt-4o-mini",
        "pass_through_params": false,
        "skip_api_key": true,
        "organization": "org-like",
        "api_version": "2024-12-01",
        "headers": {
            "x-base-header": "base"
        },
        "custom_headers": {
            "x-custom-header": "custom"
        }
    });

    let oai_like = build_openai_like_config_from_factory(&config)
        .unwrap_or_else(|err| panic!("openai_like config should parse: {err}"));

    assert_eq!(
        oai_like.base.api_base.as_deref(),
        Some("https://openai-like.example.test/v1")
    );
    assert_eq!(oai_like.base.api_key.as_deref(), Some("sk-openai-like"));
    assert_eq!(oai_like.provider_name, "custom-like");
    assert_eq!(oai_like.base.timeout, 55);
    assert_eq!(oai_like.base.max_retries, 4);
    assert_eq!(oai_like.model_prefix.as_deref(), Some("prefix/"));
    assert_eq!(oai_like.default_model.as_deref(), Some("gpt-4o-mini"));
    assert!(!oai_like.pass_through_params);
    assert!(oai_like.skip_api_key);
    assert_eq!(oai_like.base.organization.as_deref(), Some("org-like"));
    assert_eq!(oai_like.base.api_version.as_deref(), Some("2024-12-01"));
    assert_eq!(
        oai_like
            .base
            .headers
            .get("x-base-header")
            .map(String::as_str),
        Some("base")
    );
    assert_eq!(
        oai_like
            .custom_headers
            .get("x-custom-header")
            .map(String::as_str),
        Some("custom")
    );
}

#[test]
fn test_build_openai_like_config_from_factory_requires_api_base() {
    let config = serde_json::json!({
        "api_key": "sk-openai-like"
    });

    let err = build_openai_like_config_from_factory(&config)
        .err()
        .unwrap_or_else(|| panic!("missing base_url should return an error"));
    assert!(err.to_string().contains("base_url"));
}

#[tokio::test]
async fn test_from_config_async_openai_compatible_accepts_api_base_alias() {
    let config = serde_json::json!({
        "api_base": "http://localhost:11434/v1",
        "skip_api_key": true,
        "provider_name": "local-openai-like"
    });

    let provider = Provider::from_config_async(ProviderType::OpenAICompatible, config)
        .await
        .unwrap_or_else(|err| panic!("openai_compatible should be creatable: {err}"));
    assert!(matches!(provider, Provider::OpenAILike(_)));
}

#[test]
fn test_provider_selector_support_detection() {
    assert!(is_provider_selector_supported("openai"));
    assert!(is_provider_selector_supported("openai_compatible"));
    assert!(is_provider_selector_supported("groq")); // Tier-1 catalog
    assert!(!is_provider_selector_supported("totally_unknown_provider"));
}

#[test]
fn test_catalog_entries_are_supported_selectors() {
    for name in registry::PROVIDER_CATALOG.keys() {
        assert!(
            is_provider_selector_supported(name),
            "Catalog provider '{}' must be a supported selector",
            name
        );
    }
}

#[tokio::test]
async fn test_catalog_entries_are_creatable_via_factory() {
    for (name, def) in registry::PROVIDER_CATALOG.iter() {
        let config = crate::config::models::provider::ProviderConfig {
            name: (*name).to_string(),
            provider_type: (*name).to_string(),
            api_key: if def.skip_api_key {
                String::new()
            } else {
                "test-key".to_string()
            },
            ..Default::default()
        };

        let provider = create_provider(config).await.unwrap_or_else(|e| {
            panic!("Catalog provider '{}' should be creatable: {}", name, e)
        });

        assert!(
            matches!(provider, Provider::OpenAILike(_)),
            "Catalog provider '{}' must create OpenAILike variant",
            name
        );
    }
}

#[tokio::test]
async fn test_create_provider_prefers_provider_type_over_name() {
    let config = crate::config::models::provider::ProviderConfig {
        name: "openai".to_string(),
        provider_type: "pydantic_ai".to_string(),
        api_key: "test-key".to_string(),
        ..Default::default()
    };

    let err = create_provider(config)
        .await
        .expect_err("Expected unsupported provider type to fail");
    assert!(
        matches!(err, ProviderError::NotImplemented { .. }),
        "Expected NotImplemented error, got {}",
        err
    );
}

#[tokio::test]
async fn test_create_provider_falls_back_to_name_when_provider_type_empty() {
    let config = crate::config::models::provider::ProviderConfig {
        name: "pydantic_ai".to_string(),
        provider_type: "".to_string(),
        api_key: "test-key".to_string(),
        ..Default::default()
    };

    let err = create_provider(config)
        .await
        .expect_err("Expected unsupported provider name to fail");
    assert!(
        matches!(err, ProviderError::NotImplemented { .. }),
        "Expected NotImplemented error, got {}",
        err
    );
}

#[tokio::test]
async fn test_create_provider_tier1_catalog_creates_openai_like() {
    let config = crate::config::models::provider::ProviderConfig {
        name: "perplexity".to_string(),
        provider_type: "".to_string(),
        api_key: "test-key".to_string(),
        ..Default::default()
    };

    let provider = create_provider(config)
        .await
        .expect("Tier 1 provider should succeed");
    assert!(matches!(provider, Provider::OpenAILike(_)));
}

#[tokio::test]
async fn test_create_provider_tier1_catalog_applies_openai_like_overrides() {
    let mut config = crate::config::models::provider::ProviderConfig {
        name: "perplexity".to_string(),
        provider_type: "".to_string(),
        api_key: "test-key".to_string(),
        timeout: 42,
        max_retries: 6,
        api_version: Some("2024-01-01".to_string()),
        organization: Some("org-top-level".to_string()),
        ..Default::default()
    };
    config
        .settings
        .insert("model_prefix".to_string(), serde_json::json!("pplx/"));
    config.settings.insert(
        "default_model".to_string(),
        serde_json::json!("llama-3.1-sonar-small"),
    );
    config
        .settings
        .insert("pass_through_params".to_string(), serde_json::json!(false));
    config.settings.insert(
        "headers".to_string(),
        serde_json::json!({"x-test-header": "ok"}),
    );
    config.settings.insert(
        "custom_headers".to_string(),
        serde_json::json!({"x-custom-header": "ok"}),
    );

    let provider = create_provider(config)
        .await
        .expect("Tier 1 provider should accept openai-like overrides");

    match provider {
        Provider::OpenAILike(provider) => {
            let cfg = provider.config();
            assert_eq!(cfg.provider_name, "perplexity");
            assert_eq!(cfg.base.timeout, 42);
            assert_eq!(cfg.base.max_retries, 6);
            assert_eq!(cfg.base.api_version.as_deref(), Some("2024-01-01"));
            assert_eq!(cfg.base.organization.as_deref(), Some("org-top-level"));
            assert_eq!(cfg.model_prefix.as_deref(), Some("pplx/"));
            assert_eq!(cfg.default_model.as_deref(), Some("llama-3.1-sonar-small"));
            assert!(!cfg.pass_through_params);
            assert_eq!(
                cfg.base.headers.get("x-test-header").map(String::as_str),
                Some("ok")
            );
            assert_eq!(
                cfg.custom_headers
                    .get("x-custom-header")
                    .map(String::as_str),
                Some("ok")
            );
        }
        _ => panic!("Expected OpenAILike provider"),
    }
}

#[test]
fn test_b1_first_batch_selectors_are_supported() {
    for selector in ["aiml_api", "anyscale", "bytez", "comet_api"] {
        assert!(
            is_provider_selector_supported(selector),
            "Expected selector '{}' to be supported",
            selector
        );
    }
}

#[tokio::test]
async fn test_b1_first_batch_create_provider_from_name() {
    for provider_name in ["aiml_api", "anyscale", "bytez", "comet_api"] {
        let config = crate::config::models::provider::ProviderConfig {
            name: provider_name.to_string(),
            provider_type: "".to_string(),
            api_key: "test-key".to_string(),
            ..Default::default()
        };

        let provider = create_provider(config)
            .await
            .unwrap_or_else(|e| panic!("Expected '{}' to be creatable: {}", provider_name, e));
        assert!(
            matches!(provider, Provider::OpenAILike(_)),
            "Expected '{}' to create OpenAILike provider",
            provider_name
        );
    }
}

#[tokio::test]
async fn test_b1_first_batch_create_provider_from_provider_type() {
    for provider_type in ["aiml_api", "anyscale", "bytez", "comet_api"] {
        let config = crate::config::models::provider::ProviderConfig {
            name: "openai".to_string(),
            provider_type: provider_type.to_string(),
            api_key: "test-key".to_string(),
            ..Default::default()
        };

        let provider = create_provider(config).await.unwrap_or_else(|e| {
            panic!(
                "Expected '{}' provider_type to be creatable: {}",
                provider_type, e
            )
        });
        assert!(
            matches!(provider, Provider::OpenAILike(_)),
            "Expected provider_type '{}' to create OpenAILike provider",
            provider_type
        );
    }
}

#[test]
fn test_b2_second_batch_selectors_are_supported() {
    for selector in ["compactifai", "aleph_alpha", "yi", "lambda_ai"] {
        assert!(
            is_provider_selector_supported(selector),
            "Expected selector '{}' to be supported",
            selector
        );
    }
}

#[tokio::test]
async fn test_b2_second_batch_create_provider_from_name() {
    for provider_name in ["compactifai", "aleph_alpha", "yi", "lambda_ai"] {
        let config = crate::config::models::provider::ProviderConfig {
            name: provider_name.to_string(),
            provider_type: "".to_string(),
            api_key: "test-key".to_string(),
            ..Default::default()
        };

        let provider = create_provider(config)
            .await
            .unwrap_or_else(|e| panic!("Expected '{}' to be creatable: {}", provider_name, e));
        assert!(
            matches!(provider, Provider::OpenAILike(_)),
            "Expected '{}' to create OpenAILike provider",
            provider_name
        );
    }
}

#[tokio::test]
async fn test_b2_second_batch_create_provider_from_provider_type() {
    for provider_type in ["compactifai", "aleph_alpha", "yi", "lambda_ai"] {
        let config = crate::config::models::provider::ProviderConfig {
            name: "openai".to_string(),
            provider_type: provider_type.to_string(),
            api_key: "test-key".to_string(),
            ..Default::default()
        };

        let provider = create_provider(config).await.unwrap_or_else(|e| {
            panic!(
                "Expected '{}' provider_type to be creatable: {}",
                provider_type, e
            )
        });
        assert!(
            matches!(provider, Provider::OpenAILike(_)),
            "Expected provider_type '{}' to create OpenAILike provider",
            provider_type
        );
    }
}

#[test]
fn test_b3_third_batch_selectors_are_supported() {
    for selector in ["ovhcloud", "maritalk", "siliconflow", "lemonade"] {
        assert!(
            is_provider_selector_supported(selector),
            "Expected selector '{}' to be supported",
            selector
        );
    }
}

#[tokio::test]
async fn test_b3_third_batch_create_provider_from_name() {
    for provider_name in ["ovhcloud", "maritalk", "siliconflow", "lemonade"] {
        let config = crate::config::models::provider::ProviderConfig {
            name: provider_name.to_string(),
            provider_type: "".to_string(),
            api_key: "test-key".to_string(),
            ..Default::default()
        };

        let provider = create_provider(config)
            .await
            .unwrap_or_else(|e| panic!("Expected '{}' to be creatable: {}", provider_name, e));
        assert!(
            matches!(provider, Provider::OpenAILike(_)),
            "Expected '{}' to create OpenAILike provider",
            provider_name
        );
    }
}

#[tokio::test]
async fn test_b3_third_batch_create_provider_from_provider_type() {
    for provider_type in ["ovhcloud", "maritalk", "siliconflow", "lemonade"] {
        let config = crate::config::models::provider::ProviderConfig {
            name: "openai".to_string(),
            provider_type: provider_type.to_string(),
            api_key: "test-key".to_string(),
            ..Default::default()
        };

        let provider = create_provider(config).await.unwrap_or_else(|e| {
            panic!(
                "Expected '{}' provider_type to be creatable: {}",
                provider_type, e
            )
        });
        assert!(
            matches!(provider, Provider::OpenAILike(_)),
            "Expected provider_type '{}' to create OpenAILike provider",
            provider_type
        );
    }
}

#[tokio::test]
async fn test_create_provider_reports_unknown_custom_provider() {
    let config = crate::config::models::provider::ProviderConfig {
        name: "my-custom-provider".to_string(),
        provider_type: "".to_string(),
        api_key: "test-key".to_string(),
        ..Default::default()
    };

    let err = create_provider(config)
        .await
        .expect_err("Expected unknown custom provider to fail");
    assert!(
        matches!(err, ProviderError::NotImplemented { .. }),
        "Expected NotImplemented error, got {}",
        err
    );
    assert!(
        err.to_string().contains("my-custom-provider"),
        "Expected custom provider name in error, got {}",
        err
    );
}

#[tokio::test]
async fn test_create_provider_openai_compatible_factory() {
    let mut config = crate::config::models::provider::ProviderConfig {
        name: "local-openai-like".to_string(),
        provider_type: "openai_compatible".to_string(),
        api_key: "".to_string(),
        base_url: Some("http://localhost:11434/v1".to_string()),
        ..Default::default()
    };
    config
        .settings
        .insert("skip_api_key".to_string(), serde_json::Value::Bool(true));

    let provider = create_provider(config)
        .await
        .expect("openai_compatible provider should be creatable");
    assert!(matches!(provider, Provider::OpenAILike(_)));
}
