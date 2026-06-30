//! Image-specific pricing helpers for LiteLLM flat image cost fields.

use super::types::{LiteLLMModelInfo, PricingUsage};
use crate::utils::error::gateway_error::{GatewayError, Result};

pub(super) fn token_unit_prices(
    model: &str,
    model_info: &LiteLLMModelInfo,
    usage: &PricingUsage,
) -> Result<(f64, f64)> {
    let has_flat_image_price = usage.output_image_count.unwrap_or(0) > 0
        && model_info
            .extra
            .get("output_cost_per_image")
            .and_then(serde_json::Value::as_f64)
            .is_some();
    let input = price_for_units(
        model_info.input_cost_per_token,
        usage.prompt_tokens,
        model,
        "input_cost_per_token",
        has_flat_image_price,
    )?;
    let output = price_for_units(
        model_info.output_cost_per_token,
        usage.completion_tokens,
        model,
        "output_cost_per_token",
        has_flat_image_price,
    )?;
    Ok((input, output))
}

pub(super) fn output_image_cost(
    model: &str,
    model_info: &LiteLLMModelInfo,
    usage: &PricingUsage,
) -> Result<f64> {
    let has_image_token_price = model_info
        .extra
        .get("image_cost_per_token")
        .and_then(serde_json::Value::as_f64)
        .is_some();
    if usage.image_tokens.is_some() && has_image_token_price {
        return Ok(0.0);
    }

    let price = match model_info
        .extra
        .get("output_cost_per_image")
        .and_then(serde_json::Value::as_f64)
    {
        Some(price) => price,
        None if model_info.input_cost_per_token.is_some()
            || model_info.output_cost_per_token.is_some() =>
        {
            return Ok(0.0);
        }
        None => {
            return Err(GatewayError::Config(format!(
                "Missing image pricing for model {}: output_cost_per_image",
                model
            )));
        }
    };
    let count = usage.output_image_count.ok_or_else(|| {
        GatewayError::Config(format!(
            "Missing image count for model {}: output_image_count",
            model
        ))
    })?;
    if count == 0 {
        return Ok(0.0);
    }
    if price < 0.0 || price.is_nan() {
        return Err(GatewayError::Config(format!(
            "Invalid image pricing for model {}: output_cost_per_image ({})",
            model, price
        )));
    }
    if !flat_image_pricing_key_matches(model, usage) {
        return Err(GatewayError::Config(format!(
            "Missing image pricing variant for model {}: output_image_pricing_keys",
            model
        )));
    }
    Ok(count as f64 * price)
}

fn flat_image_pricing_key_matches(model: &str, usage: &PricingUsage) -> bool {
    if usage.output_image_pricing_keys.is_empty() {
        return !is_variant_image_pricing_key(model);
    }

    usage.output_image_pricing_keys.iter().any(|key| {
        key == model
            || (!is_variant_image_pricing_key(model)
                && key
                    .strip_suffix(model)
                    .is_some_and(|prefix| prefix.is_empty() || prefix.ends_with('/')))
    })
}

fn is_variant_image_pricing_key(model: &str) -> bool {
    let mut segments = model.rsplitn(2, '/');
    let _model_id = segments.next();
    let Some(prefix) = segments.next() else {
        return false;
    };

    prefix.split('/').any(is_image_variant_segment)
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

fn price_for_units(
    price: Option<f64>,
    units: u32,
    model: &str,
    field: &str,
    allow_missing_for_flat_image: bool,
) -> Result<f64> {
    if units == 0 || price.is_none() && allow_missing_for_flat_image {
        return Ok(price.unwrap_or(0.0));
    }
    super::service::require_pricing_field(price, model, "token pricing", field)
}
