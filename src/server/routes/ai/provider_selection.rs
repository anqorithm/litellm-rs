//! Provider selection helpers for AI routes

use crate::core::providers::{Provider, ProviderRegistry};
use crate::core::types::model::ProviderCapability;
use crate::utils::error::GatewayError;

pub struct ProviderSelection<'a> {
    pub provider: &'a Provider,
    pub model: String,
}

pub fn select_provider_for_model<'a>(
    pool: &'a ProviderRegistry,
    model: &str,
    capability: ProviderCapability,
) -> Result<ProviderSelection<'a>, GatewayError> {
    if model.trim().is_empty() {
        return Err(GatewayError::validation("Model is required"));
    }

    if let Some((prefix, actual)) = model.split_once('/') {
        if pool.contains(prefix) {
            let provider = pool
                .get(prefix)
                .ok_or_else(|| GatewayError::internal("Provider not available"))?;
            if !provider_supports_capability(provider, &capability) {
                return Err(GatewayError::validation(format!(
                    "Provider '{}' does not support {:?}",
                    prefix, capability
                )));
            }
            return Ok(ProviderSelection {
                provider,
                model: actual.to_string(),
            });
        }
    }

    let mut candidates: Vec<&Provider> = pool
        .find_supporting_model(model)
        .into_iter()
        .filter(|p| provider_supports_capability(p, &capability))
        .collect();

    if candidates.len() == 1 {
        return Ok(ProviderSelection {
            provider: candidates.remove(0),
            model: model.to_string(),
        });
    }

    let mut capable: Vec<&Provider> = pool
        .values()
        .filter(|p| provider_supports_capability(p, &capability))
        .collect();

    if capable.len() == 1 {
        return Ok(ProviderSelection {
            provider: capable.remove(0),
            model: model.to_string(),
        });
    }

    if capable.is_empty() {
        return Err(GatewayError::internal(format!(
            "No providers available for {:?}",
            capability
        )));
    }

    Err(GatewayError::validation(
        "Multiple providers available; use provider/model prefix to disambiguate",
    ))
}

pub fn select_provider_for_optional_model<'a>(
    pool: &'a ProviderRegistry,
    model: Option<&str>,
    capability: ProviderCapability,
) -> Result<(&'a Provider, Option<String>), GatewayError> {
    if let Some(model) = model {
        let selection = select_provider_for_model(pool, model, capability)?;
        return Ok((selection.provider, Some(selection.model)));
    }

    let mut capable: Vec<&Provider> = pool
        .values()
        .filter(|p| provider_supports_capability(p, &capability))
        .collect();

    if capable.len() == 1 {
        return Ok((capable.remove(0), None));
    }

    if capable.is_empty() {
        return Err(GatewayError::internal(format!(
            "No providers available for {:?}",
            capability
        )));
    }

    Err(GatewayError::validation(
        "Multiple providers available; specify model with provider prefix",
    ))
}

fn provider_supports_capability(provider: &Provider, capability: &ProviderCapability) -> bool {
    provider.capabilities().iter().any(|cap| cap == capability)
}
