//! Compact builder for [`BedrockCatalogEntry`] used by the per-vendor seed
//! modules. Reduces repetition while keeping every field explicit at the
//! call site.

use super::super::super::model_config::{BedrockApiType, BedrockModelFamily};
use super::super::{
    BedrockCatalogEntry, BedrockPricing, BedrockVendor, EndpointSupport, InferenceProfileScope,
    ModelCapabilities, ModelLifecycle, ModelLimits, NoPricingReason, SourceMetadata,
};

pub(super) const NO_PROFILES: &[InferenceProfileScope] = &[];

pub(super) const US_GLOBAL: &[InferenceProfileScope] = &[
    InferenceProfileScope::Global,
    InferenceProfileScope::UnitedStates,
];

pub(super) const COMMON_GEO: &[InferenceProfileScope] = &[
    InferenceProfileScope::Global,
    InferenceProfileScope::UnitedStates,
    InferenceProfileScope::Europe,
    InferenceProfileScope::AsiaPacific,
];

#[allow(clippy::too_many_arguments)]
pub(super) fn entry(
    model_id: &'static str,
    display_name: &'static str,
    vendor: BedrockVendor,
    family: BedrockModelFamily,
    api_type: BedrockApiType,
    lifecycle: ModelLifecycle,
    endpoints: EndpointSupport,
    inference_profiles: &'static [InferenceProfileScope],
    limits: ModelLimits,
    capabilities: ModelCapabilities,
    pricing: Option<BedrockPricing>,
    no_pricing_reason: Option<NoPricingReason>,
    source: SourceMetadata,
) -> BedrockCatalogEntry {
    BedrockCatalogEntry {
        model_id,
        canonical_id: model_id,
        display_name,
        vendor,
        family,
        api_type,
        lifecycle,
        endpoints,
        inference_profiles,
        limits,
        capabilities,
        pricing,
        no_pricing_reason,
        source,
    }
}

pub(super) fn alias_entry(
    model_id: &'static str,
    canonical_id: &'static str,
    base: BedrockCatalogEntry,
) -> BedrockCatalogEntry {
    BedrockCatalogEntry {
        model_id,
        canonical_id,
        ..base
    }
}
