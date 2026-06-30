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
    image_token_cost: f64,
) -> Result<f64> {
    let count = usage.output_image_count.unwrap_or(0);
    if count == 0 || usage.image_tokens.is_some() && image_token_cost > 0.0 {
        return Ok(0.0);
    }
    let price = model_info
        .extra
        .get("output_cost_per_image")
        .and_then(serde_json::Value::as_f64)
        .ok_or_else(|| {
            GatewayError::Config(format!(
                "Missing image pricing for model {}: output_cost_per_image",
                model
            ))
        })?;
    if price < 0.0 || price.is_nan() {
        return Err(GatewayError::Config(format!(
            "Invalid image pricing for model {}: output_cost_per_image ({})",
            model, price
        )));
    }
    Ok(count as f64 * price)
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
