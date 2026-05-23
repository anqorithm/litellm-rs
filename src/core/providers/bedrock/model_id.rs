//! Bedrock model ID parsing.
//!
//! Bedrock accepts foundation model IDs, inference profile IDs, and ARNs as
//! execution `modelId` values. Only the user-facing `bedrock/` selector is
//! removed from the execution path.

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BedrockModelIdKind {
    Arn,
    InferenceProfile,
    FoundationModel,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedBedrockModelId {
    pub user_selector: String,
    pub execution_model_id: String,
    pub metadata_lookup_ids: Vec<String>,
    pub kind: BedrockModelIdKind,
    pub family_hint: Option<String>,
}

impl ParsedBedrockModelId {
    pub fn new(model_id: &str) -> Self {
        let user_selector = model_id.trim().to_string();
        let execution_model_id = user_selector
            .strip_prefix("bedrock/")
            .unwrap_or(&user_selector)
            .to_string();
        let mut metadata_lookup_ids = Vec::new();

        push_unique(&mut metadata_lookup_ids, execution_model_id.clone());

        if let Some(resource_id) = arn_resource_model_id(&execution_model_id) {
            push_unique(&mut metadata_lookup_ids, resource_id.to_string());
            if let Some(canonical) = canonical_metadata_id(resource_id) {
                push_unique(&mut metadata_lookup_ids, canonical);
            }
        } else if let Some(canonical) = canonical_metadata_id(&execution_model_id) {
            push_unique(&mut metadata_lookup_ids, canonical);
        }

        let family_hint = metadata_lookup_ids
            .iter()
            .find_map(|id| id.split_once('.').map(|(family, _)| family.to_string()));

        let kind = if execution_model_id.starts_with("arn:") {
            BedrockModelIdKind::Arn
        } else if canonical_metadata_id(&execution_model_id).is_some() {
            BedrockModelIdKind::InferenceProfile
        } else {
            BedrockModelIdKind::FoundationModel
        };

        Self {
            user_selector,
            execution_model_id,
            metadata_lookup_ids,
            kind,
            family_hint,
        }
    }

    pub fn canonical_metadata_id(&self) -> &str {
        self.metadata_lookup_ids
            .last()
            .map(String::as_str)
            .unwrap_or(&self.execution_model_id)
    }
}

pub fn parse_bedrock_model_id(model_id: &str) -> ParsedBedrockModelId {
    ParsedBedrockModelId::new(model_id)
}

pub fn get_model_config_for_model_id(
    model_id: &str,
) -> Result<&'static super::model_config::ModelConfig, crate::core::providers::ProviderError> {
    let parsed = parse_bedrock_model_id(model_id);
    for lookup_id in &parsed.metadata_lookup_ids {
        if let Ok(config) = super::model_config::get_model_config(lookup_id) {
            return Ok(config);
        }
    }

    Err(crate::core::providers::ProviderError::model_not_found(
        "bedrock",
        format!("Model {} not supported", parsed.execution_model_id),
    ))
}

fn canonical_metadata_id(model_id: &str) -> Option<String> {
    let (prefix, rest) = model_id.split_once('.')?;
    if is_geo_prefix(prefix) || is_region_prefix(prefix) {
        Some(rest.to_string())
    } else {
        None
    }
}

fn arn_resource_model_id(model_id: &str) -> Option<&str> {
    if !model_id.starts_with("arn:") {
        return None;
    }

    model_id
        .rsplit_once('/')
        .map(|(_, resource_id)| resource_id)
        .filter(|resource_id| !resource_id.is_empty())
}

fn is_geo_prefix(prefix: &str) -> bool {
    matches!(
        prefix,
        "global" | "us" | "eu" | "ap" | "apac" | "sa" | "ca" | "me" | "af"
    )
}

fn is_region_prefix(prefix: &str) -> bool {
    prefix.len() >= 4
        && prefix.contains('-')
        && prefix
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
}

fn push_unique(values: &mut Vec<String>, value: String) {
    if !values.iter().any(|existing| existing == &value) {
        values.push(value);
    }
}

#[cfg(test)]
mod tests {
    use super::{BedrockModelIdKind, parse_bedrock_model_id};

    #[test]
    fn bedrock_selector_is_removed_only_for_execution() {
        let parsed = parse_bedrock_model_id("bedrock/us.anthropic.claude-3-5-sonnet-20241022-v2:0");

        assert_eq!(
            parsed.execution_model_id,
            "us.anthropic.claude-3-5-sonnet-20241022-v2:0"
        );
        assert_eq!(
            parsed.metadata_lookup_ids,
            vec![
                "us.anthropic.claude-3-5-sonnet-20241022-v2:0",
                "anthropic.claude-3-5-sonnet-20241022-v2:0"
            ]
        );
        assert_eq!(parsed.kind, BedrockModelIdKind::InferenceProfile);
    }

    #[test]
    fn global_profile_is_preserved_for_execution() {
        let parsed = parse_bedrock_model_id("global.anthropic.claude-sonnet-4-v1:0");

        assert_eq!(
            parsed.execution_model_id,
            "global.anthropic.claude-sonnet-4-v1:0"
        );
        assert_eq!(
            parsed.metadata_lookup_ids,
            vec![
                "global.anthropic.claude-sonnet-4-v1:0",
                "anthropic.claude-sonnet-4-v1:0"
            ]
        );
    }

    #[test]
    fn region_like_model_id_is_preserved_for_execution() {
        let parsed = parse_bedrock_model_id("us-east-1.anthropic.claude-3-haiku-20240307");

        assert_eq!(
            parsed.execution_model_id,
            "us-east-1.anthropic.claude-3-haiku-20240307"
        );
        assert_eq!(
            parsed.metadata_lookup_ids,
            vec![
                "us-east-1.anthropic.claude-3-haiku-20240307",
                "anthropic.claude-3-haiku-20240307"
            ]
        );
    }

    #[test]
    fn arn_is_preserved_for_execution_with_resource_metadata_fallback() {
        let parsed = parse_bedrock_model_id(
            "arn:aws:bedrock:us-east-1:123456789012:inference-profile/us.anthropic.claude-3-5-sonnet-20241022-v2:0",
        );

        assert!(parsed.execution_model_id.starts_with("arn:aws:bedrock:"));
        assert_eq!(
            parsed.metadata_lookup_ids,
            vec![
                "arn:aws:bedrock:us-east-1:123456789012:inference-profile/us.anthropic.claude-3-5-sonnet-20241022-v2:0",
                "us.anthropic.claude-3-5-sonnet-20241022-v2:0",
                "anthropic.claude-3-5-sonnet-20241022-v2:0"
            ]
        );
        assert_eq!(parsed.kind, BedrockModelIdKind::Arn);
    }
}
