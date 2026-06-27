use super::*;
use crate::core::router::config::RouterConfig;

#[test]
fn budget_retry_fallbacks_skip_retry_delay() {
    let config = RouterConfig {
        retry_after_secs: 5,
        ..Default::default()
    };

    let provider_budget =
        ProviderError::quota_exceeded("budget", "provider 'openai' budget exceeded");
    let model_budget = ProviderError::quota_exceeded("budget", "model 'gpt-4o' budget exceeded");
    let rate_limit = ProviderError::rate_limit("openai", Some(60));

    assert_eq!(retry_delay_for_error(&config, 1, &provider_budget), None);
    assert_eq!(retry_delay_for_error(&config, 1, &model_budget), None);
    assert_eq!(
        retry_delay_for_error(&config, 1, &rate_limit),
        Some(std::time::Duration::from_secs(5))
    );
}
