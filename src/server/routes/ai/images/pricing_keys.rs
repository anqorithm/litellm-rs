use crate::core::pricing_service::{LiteLLMModelInfo, PricingService};

pub(super) fn image_pricing_keys(
    pricing_provider: &str,
    pricing_model: &str,
    size: Option<&str>,
    quality: Option<&str>,
) -> Vec<String> {
    let model = pricing_model.trim();
    if model.is_empty() {
        return Vec::new();
    }

    let size = size.map(normalize_image_pricing_size);
    let quality = quality
        .map(str::trim)
        .filter(|quality| !quality.is_empty())
        .map(str::to_ascii_lowercase);
    let provider = pricing_provider.trim();
    let mut keys = Vec::new();
    push_unique_key(&mut keys, model.to_string());
    if let Some(size) = size.as_deref() {
        push_unique_key(&mut keys, format!("{size}/{model}"));
        if !provider.is_empty() {
            push_unique_key(&mut keys, format!("{provider}/{size}/{model}"));
        }
        if let Some(quality) = quality.as_deref() {
            push_unique_key(&mut keys, format!("{quality}/{size}/{model}"));
            push_unique_key(&mut keys, format!("{size}/{quality}/{model}"));
            if !provider.is_empty() {
                push_unique_key(&mut keys, format!("{provider}/{quality}/{size}/{model}"));
                push_unique_key(&mut keys, format!("{provider}/{size}/{quality}/{model}"));
            }
        }
    } else if let Some(quality) = quality.as_deref() {
        push_unique_key(&mut keys, format!("{quality}/{model}"));
        if !provider.is_empty() {
            push_unique_key(&mut keys, format!("{provider}/{quality}/{model}"));
        }
    }
    keys
}

pub(super) fn resolve_image_pricing_model(
    pricing_service: &PricingService,
    pricing_provider: &str,
    model: &str,
    size: Option<&str>,
    quality: Option<&str>,
) -> Option<String> {
    image_pricing_model_candidates(model, size, quality)
        .into_iter()
        .find(|candidate| {
            pricing_service
                .get_model_info_for_provider(pricing_provider, candidate)
                .is_some_and(|(resolved, info)| {
                    resolved == candidate.as_str() && supports_image_output_pricing(&info)
                })
        })
}

pub(super) fn is_variant_image_pricing_key(model: &str) -> bool {
    model
        .rsplit_once('/')
        .is_some_and(|(prefix, _)| prefix.split('/').any(is_image_variant_segment))
}

fn image_pricing_model_candidates(
    model: &str,
    size: Option<&str>,
    quality: Option<&str>,
) -> Vec<String> {
    let model = image_pricing_base_model(model.trim());
    if model.is_empty() {
        return Vec::new();
    }

    let size = size.map(normalize_image_pricing_size);
    let quality = quality
        .map(str::trim)
        .filter(|quality| !quality.is_empty())
        .map(str::to_ascii_lowercase);
    let mut candidates = Vec::new();
    if let Some(size) = size.as_deref() {
        if let Some(quality) = quality.as_deref() {
            push_unique_key(&mut candidates, format!("{quality}/{size}/{model}"));
            push_unique_key(&mut candidates, format!("{size}/{quality}/{model}"));
        }
        push_unique_key(&mut candidates, format!("{size}/{model}"));
    } else if let Some(quality) = quality.as_deref() {
        push_unique_key(&mut candidates, format!("{quality}/{model}"));
    }
    candidates
}

fn image_pricing_base_model(model: &str) -> &str {
    model
        .rsplit_once('/')
        .filter(|(prefix, _)| prefix.split('/').any(is_image_variant_segment))
        .map(|(_, model_id)| model_id)
        .unwrap_or(model)
}

fn normalize_image_pricing_size(size: &str) -> String {
    size.trim().replace('x', "-x-")
}

fn supports_image_output_pricing(info: &LiteLLMModelInfo) -> bool {
    info.input_cost_per_token.is_some()
        || info.output_cost_per_token.is_some()
        || [
            "output_cost_per_image",
            "image_cost_per_token",
            "output_cost_per_image_token",
        ]
        .into_iter()
        .any(|key| {
            info.extra
                .get(key)
                .and_then(serde_json::Value::as_f64)
                .is_some()
        })
}

fn is_image_variant_segment(segment: &str) -> bool {
    let segment = segment.to_ascii_lowercase();
    matches!(
        segment.as_str(),
        "hd" | "standard" | "low" | "medium" | "high" | "max-steps"
    ) || segment.ends_with("-steps")
        || segment.contains("-x-")
        || segment.split_once('x').is_some_and(|(width, height)| {
            width.chars().all(|ch| ch.is_ascii_digit())
                && height.chars().all(|ch| ch.is_ascii_digit())
        })
}

fn push_unique_key(keys: &mut Vec<String>, key: String) {
    if !keys.contains(&key) {
        keys.push(key);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn model_info(extra: HashMap<String, serde_json::Value>) -> LiteLLMModelInfo {
        LiteLLMModelInfo {
            max_tokens: None,
            max_input_tokens: None,
            max_output_tokens: None,
            input_cost_per_token: None,
            output_cost_per_token: None,
            input_cost_per_character: None,
            output_cost_per_character: None,
            cost_per_second: None,
            litellm_provider: "openai".to_string(),
            mode: "image_generation".to_string(),
            supports_function_calling: None,
            supports_vision: None,
            supports_streaming: None,
            supports_parallel_function_calling: None,
            supports_system_message: None,
            extra,
        }
    }

    #[test]
    fn resolve_image_pricing_model_skips_unsupported_input_only_variant() {
        let service = PricingService::new(None);
        service.add_custom_model(
            "medium/1024-x-1024/gpt-image-1.5".to_string(),
            model_info(HashMap::from([(
                "input_cost_per_image".to_string(),
                serde_json::Value::from(0.034),
            )])),
        );

        let resolved = resolve_image_pricing_model(
            &service,
            "openai",
            "gpt-image-1.5",
            Some("1024x1024"),
            Some("medium"),
        );

        assert_eq!(resolved, None);
    }

    #[test]
    fn resolve_image_pricing_model_accepts_output_priced_variant() {
        let service = PricingService::new(None);
        service.add_custom_model(
            "hd/1024-x-1024/flat-variant-model".to_string(),
            model_info(HashMap::from([(
                "output_cost_per_image".to_string(),
                serde_json::Value::from(0.10),
            )])),
        );

        let resolved = resolve_image_pricing_model(
            &service,
            "openai",
            "flat-variant-model",
            Some("1024x1024"),
            Some("hd"),
        );

        assert_eq!(
            resolved,
            Some("hd/1024-x-1024/flat-variant-model".to_string())
        );
    }
}
