#![cfg(feature = "providers-extended")]

// Normal `cargo test` must compile and execute these deprecated-use lanes. Clippy
// skips them because CI promotes the expected deprecation warnings to errors while
// the repository guard correctly forbids suppressing those warnings.
#[cfg(not(clippy))]
use litellm_rs::core::providers::amazon_nova::{
    AmazonNovaConfig, AmazonNovaErrorMapper, AmazonNovaModel, AmazonNovaModelRegistry,
    AmazonNovaProvider,
};
#[cfg(not(clippy))]
use litellm_rs::core::providers::custom_api::{
    CustomApiErrorMapper, CustomHttpxConfig, CustomHttpxProvider, PROVIDER_NAME,
};
#[cfg(not(clippy))]
use litellm_rs::core::providers::github::{GitHubConfig, GitHubProvider, get_model_info};

#[cfg(not(clippy))]
#[test]
fn amazon_nova_deprecated_in_0_6() {
    let registry = AmazonNovaModelRegistry::new();
    assert!(registry.is_supported("amazon.nova-pro-v1:0"));
    let _model = AmazonNovaModel::new("compat-only", "Compat", "compat probe", 1_024, 128);
    let _mapper = AmazonNovaErrorMapper;
    let _provider = AmazonNovaProvider::new(AmazonNovaConfig::with_api_key("compat-only-key"))
        .expect("0.6 public construction must remain available without issuing a request");
}
#[cfg(not(clippy))]
#[test]
fn custom_api_deprecated_in_0_6() {
    let mut config = CustomHttpxConfig::new("https://8.8.8.8/v1/chat")
        .with_api_key("compat-only-key")
        .with_http_method("PATCH")
        .with_request_template(r#"{"model":"{model}","messages":{messages}}"#)
        .with_header("X-Compat-Test", "0.6");
    config.response_parser = Some("$.choices[0]".to_string());

    assert_eq!(config.endpoint_url, "https://8.8.8.8/v1/chat");
    assert_eq!(config.http_method, "PATCH");
    assert!(config.request_template.is_some());
    assert_eq!(config.response_parser.as_deref(), Some("$.choices[0]"));
    assert_eq!(PROVIDER_NAME, "custom_httpx");

    let _mapper = CustomApiErrorMapper;
    let _provider = CustomHttpxProvider::new(config)
        .expect("0.6 public construction must remain available without issuing a request");
    let _from_endpoint = CustomHttpxProvider::with_endpoint("https://8.8.8.8/v1/chat")
        .expect("0.6 with_endpoint signature must remain available without issuing a request");
}

#[cfg(not(clippy))]
#[tokio::test]
async fn github_deprecated_in_0_6() {
    let model = get_model_info("gpt-4o")
        .expect("0.6 native github model metadata must remain available without issuing a request");
    assert_eq!(model.display_name, "GPT-4o");
    let config = GitHubConfig {
        api_key: Some("compat-only-key".to_string()),
        ..Default::default()
    };
    let _provider = GitHubProvider::new(config)
        .await
        .expect("0.6 public construction must remain available without issuing a request");
}
