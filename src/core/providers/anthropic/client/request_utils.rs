use std::collections::HashSet;

use serde_json::{Value, json};

use crate::core::providers::unified_provider::ProviderError;
use crate::core::types::tools::Tool;

const ANTHROPIC_TOOL_NAME_MAX_LEN: usize = 64;

pub(super) fn anthropic_tool_name(name: &str) -> String {
    let mut sanitized = name
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '_' || c == '-' {
                c
            } else {
                '_'
            }
        })
        .collect::<String>();
    if sanitized.is_empty() {
        sanitized.push_str("tool");
    }
    sanitized.truncate(ANTHROPIC_TOOL_NAME_MAX_LEN);
    sanitized
}

pub(super) fn anthropic_tools(tools: &[Tool]) -> Result<Vec<Value>, ProviderError> {
    let names =
        sanitized_anthropic_tool_names(tools.iter().map(|tool| tool.function.name.as_str()))?;
    Ok(tools
        .iter()
        .zip(names)
        .map(|(tool, name)| {
            json!({
                "name": name,
                "description": tool.function.description.as_deref().unwrap_or(""),
                "input_schema": tool.function.parameters.as_ref().unwrap_or(&json!({}))
            })
        })
        .collect())
}

fn sanitized_anthropic_tool_names<'a>(
    names: impl IntoIterator<Item = &'a str>,
) -> Result<Vec<String>, ProviderError> {
    let mut seen = HashSet::new();
    let mut sanitized_names = Vec::new();

    for name in names {
        let sanitized = anthropic_tool_name(name);
        if !seen.insert(sanitized.clone()) {
            return Err(ProviderError::invalid_request(
                "anthropic",
                format!(
                    "Tool name '{}' collides after Anthropic name sanitization",
                    name
                ),
            ));
        }
        sanitized_names.push(sanitized);
    }

    Ok(sanitized_names)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn issue_761_sanitizes_tool_names_to_anthropic_shape() {
        assert_eq!(
            anthropic_tool_name("get.weather forecast"),
            "get_weather_forecast"
        );
        assert_eq!(anthropic_tool_name(""), "tool");
        assert_eq!(anthropic_tool_name(&"a".repeat(80)).len(), 64);
    }

    #[test]
    fn issue_761_rejects_sanitized_tool_name_collisions() {
        let error = sanitized_anthropic_tool_names(["get.weather", "get_weather"])
            .expect_err("sanitized tool names must be unique");

        assert!(error.to_string().contains("collides"));
    }
}
