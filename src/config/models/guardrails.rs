//! Gateway-specific defaults and deserialization for content guardrails.

use crate::core::guardrails::config::CustomRuleConfig;
use crate::core::guardrails::{
    GuardrailAction, GuardrailConfig, OpenAIModerationConfig, PIIConfig, PromptInjectionConfig,
};
use serde::Deserialize;

pub(super) fn default_gateway_guardrails() -> GuardrailConfig {
    GuardrailConfig::default()
        .enable()
        .with_prompt_injection(PromptInjectionConfig::new())
}

#[derive(Deserialize, Default)]
struct GatewayGuardrailsWire {
    enabled: Option<bool>,
    openai_moderation: Option<OpenAIModerationConfig>,
    pii: Option<PIIConfig>,
    prompt_injection: Option<PromptInjectionConfig>,
    custom_rules: Option<Vec<CustomRuleConfig>>,
    default_action: Option<GuardrailAction>,
    check_input: Option<bool>,
    check_output: Option<bool>,
    exclude_paths: Option<Vec<String>>,
    fail_open: Option<bool>,
}

pub(super) fn deserialize_gateway_guardrails<'de, D>(
    deserializer: D,
) -> Result<GuardrailConfig, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let wire = GatewayGuardrailsWire::deserialize(deserializer)?;
    let mut config = default_gateway_guardrails();
    if let Some(value) = wire.enabled {
        config.enabled = value;
    }
    if let Some(value) = wire.openai_moderation {
        config.openai_moderation = Some(value);
    }
    if let Some(value) = wire.pii {
        config.pii = Some(value);
    }
    if let Some(value) = wire.prompt_injection {
        config.prompt_injection = Some(value);
    }
    if let Some(value) = wire.custom_rules {
        config.custom_rules = value;
    }
    if let Some(value) = wire.default_action {
        config.default_action = value;
    }
    if let Some(value) = wire.check_input {
        config.check_input = value;
    }
    if let Some(value) = wire.check_output {
        config.check_output = value;
    }
    if let Some(value) = wire.exclude_paths {
        config.exclude_paths = value;
    }
    if let Some(value) = wire.fail_open {
        config.fail_open = value;
    }
    Ok(config)
}

#[cfg(test)]
mod tests {
    use super::super::gateway::GatewayConfig;

    #[test]
    fn partial_gateway_guardrails_keep_secure_defaults() {
        let mut value = serde_json::to_value(GatewayConfig::default()).unwrap();
        value["guardrails"] = serde_json::json!({"exclude_paths": ["/health"]});

        let config: GatewayConfig = serde_json::from_value(value).unwrap();

        assert!(config.guardrails.enabled);
        assert!(
            config
                .guardrails
                .prompt_injection
                .as_ref()
                .is_some_and(|policy| policy.enabled)
        );
        assert_eq!(config.guardrails.exclude_paths, vec!["/health"]);
    }
}
