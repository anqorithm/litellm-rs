//! Canonical provider registry matrix.
//!
//! This module is the canonical provider identity matrix for enum variants:
//! aliases, display names, dispatchability, and factory support are derived
//! from these entries.

use crate::core::providers::provider_type::ProviderType;
use std::sync::LazyLock;

/// How a provider selector is currently dispatched.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProviderDispatchKind {
    /// Backed by a concrete `Provider` enum variant.
    Native,
    /// Backed by an explicit factory branch that creates `OpenAILikeProvider`.
    ExplicitOpenAiLike,
    /// Backed by the data-driven Tier-1 catalog.
    CatalogOpenAiLike,
    /// Represented in `ProviderType`, but not currently instantiable.
    UnsupportedEnum,
}

impl ProviderDispatchKind {
    pub fn is_dispatchable(self) -> bool {
        !matches!(self, Self::UnsupportedEnum)
    }
}

/// Canonical metadata for one non-custom `ProviderType` variant.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProviderRegistryEntry {
    pub provider_type: ProviderType,
    pub canonical_name: &'static str,
    pub aliases: &'static [&'static str],
    pub dispatch_kind: ProviderDispatchKind,
    /// True when the canonical selector is present in `PROVIDER_CATALOG`.
    pub catalog_backed: bool,
}

impl ProviderRegistryEntry {
    pub fn is_dispatchable(&self) -> bool {
        self.dispatch_kind.is_dispatchable()
    }

    pub fn names(&self) -> impl Iterator<Item = &'static str> + '_ {
        std::iter::once(self.canonical_name).chain(self.aliases.iter().copied())
    }

    pub fn matches_name(&self, name: &str) -> bool {
        let normalized = name.trim().to_ascii_lowercase();
        self.names().any(|candidate| candidate == normalized)
    }
}

pub static PROVIDER_TYPE_REGISTRY: &[ProviderRegistryEntry] = &[
    entry(
        ProviderType::OpenAI,
        "openai",
        &[],
        ProviderDispatchKind::Native,
        false,
    ),
    entry(
        ProviderType::Anthropic,
        "anthropic",
        &[],
        ProviderDispatchKind::Native,
        false,
    ),
    entry(
        ProviderType::Bedrock,
        "bedrock",
        &["aws-bedrock"],
        ProviderDispatchKind::Native,
        false,
    ),
    entry(
        ProviderType::OpenRouter,
        "openrouter",
        &[],
        ProviderDispatchKind::CatalogOpenAiLike,
        true,
    ),
    entry(
        ProviderType::VertexAI,
        "vertex_ai",
        &["vertexai", "vertex-ai"],
        provider_extra_only_native_dispatch_kind(),
        false,
    ),
    entry(
        ProviderType::Gemini,
        "gemini",
        &["google-gemini", "google_ai", "google-ai"],
        providers_extended_native_dispatch_kind(),
        false,
    ),
    entry(
        ProviderType::Azure,
        "azure",
        &["azure-openai"],
        provider_extra_native_dispatch_kind(),
        false,
    ),
    entry(
        ProviderType::AzureAI,
        "azure_ai",
        &["azureai", "azure-ai"],
        provider_extra_native_dispatch_kind(),
        false,
    ),
    entry(
        ProviderType::Cohere,
        "cohere",
        &["cohere-ai"],
        providers_extended_native_dispatch_kind(),
        false,
    ),
    entry(
        ProviderType::DeepSeek,
        "deepseek",
        &["deep-seek"],
        ProviderDispatchKind::CatalogOpenAiLike,
        true,
    ),
    entry(
        ProviderType::DeepInfra,
        "deepinfra",
        &["deep-infra"],
        ProviderDispatchKind::CatalogOpenAiLike,
        true,
    ),
    entry(
        ProviderType::V0,
        "v0",
        &[],
        ProviderDispatchKind::CatalogOpenAiLike,
        true,
    ),
    entry(
        ProviderType::MetaLlama,
        "meta_llama",
        &["llama", "meta-llama"],
        ProviderDispatchKind::CatalogOpenAiLike,
        true,
    ),
    entry(
        ProviderType::Mistral,
        "mistral",
        &["mistralai"],
        ProviderDispatchKind::Native,
        false,
    ),
    entry(
        ProviderType::Moonshot,
        "moonshot",
        &["moonshot-ai"],
        ProviderDispatchKind::CatalogOpenAiLike,
        true,
    ),
    entry(
        ProviderType::Minimax,
        "minimax",
        &["minimax-ai"],
        ProviderDispatchKind::CatalogOpenAiLike,
        true,
    ),
    entry(
        ProviderType::Dashscope,
        "dashscope",
        &["alibaba", "qwen", "tongyi"],
        ProviderDispatchKind::CatalogOpenAiLike,
        true,
    ),
    entry(
        ProviderType::Groq,
        "groq",
        &[],
        ProviderDispatchKind::CatalogOpenAiLike,
        true,
    ),
    entry(
        ProviderType::XAI,
        "xai",
        &[],
        ProviderDispatchKind::CatalogOpenAiLike,
        true,
    ),
    entry(
        ProviderType::Cloudflare,
        "cloudflare",
        &["cf", "workers-ai"],
        ProviderDispatchKind::Native,
        false,
    ),
    entry(
        ProviderType::Perplexity,
        "perplexity",
        &["perplexity-ai", "pplx"],
        ProviderDispatchKind::CatalogOpenAiLike,
        true,
    ),
    entry(
        ProviderType::Replicate,
        "replicate",
        &["replicate-ai"],
        providers_extended_native_dispatch_kind(),
        false,
    ),
    entry(
        ProviderType::FalAI,
        "fal_ai",
        &["fal-ai", "fal"],
        providers_extended_native_dispatch_kind(),
        false,
    ),
    entry(
        ProviderType::AmazonNova,
        "amazon_nova",
        &["amazon-nova", "nova"],
        ProviderDispatchKind::CatalogOpenAiLike,
        true,
    ),
    entry(
        ProviderType::GitHub,
        "github",
        &["github-models"],
        ProviderDispatchKind::CatalogOpenAiLike,
        true,
    ),
    entry(
        ProviderType::GitHubCopilot,
        "github_copilot",
        &["github-copilot", "copilot"],
        providers_extended_native_dispatch_kind(),
        false,
    ),
    entry(
        ProviderType::Hyperbolic,
        "hyperbolic",
        &["hyperbolic-ai"],
        ProviderDispatchKind::CatalogOpenAiLike,
        true,
    ),
    entry(
        ProviderType::Infinity,
        "infinity",
        &["infinity-embedding"],
        ProviderDispatchKind::CatalogOpenAiLike,
        true,
    ),
    entry(
        ProviderType::Novita,
        "novita",
        &["novita-ai"],
        ProviderDispatchKind::CatalogOpenAiLike,
        true,
    ),
    entry(
        ProviderType::Volcengine,
        "volcengine",
        &["volc", "doubao", "bytedance"],
        ProviderDispatchKind::CatalogOpenAiLike,
        true,
    ),
    entry(
        ProviderType::Nebius,
        "nebius",
        &["nebius-ai"],
        ProviderDispatchKind::CatalogOpenAiLike,
        true,
    ),
    entry(
        ProviderType::Nscale,
        "nscale",
        &["nscale-ai"],
        ProviderDispatchKind::CatalogOpenAiLike,
        true,
    ),
    entry(
        ProviderType::PydanticAI,
        "pydantic_ai",
        &["pydantic-ai", "pydantic"],
        ProviderDispatchKind::UnsupportedEnum,
        false,
    ),
    entry(
        ProviderType::OpenAICompatible,
        "openai_compatible",
        &["openai-compatible", "openai_like", "openai-like"],
        ProviderDispatchKind::ExplicitOpenAiLike,
        false,
    ),
];

static DISPATCHABLE_PROVIDER_TYPES: LazyLock<Vec<ProviderType>> = LazyLock::new(|| {
    PROVIDER_TYPE_REGISTRY
        .iter()
        .filter(|entry| entry.is_dispatchable())
        .map(|entry| entry.provider_type.clone())
        .collect()
});

pub fn provider_type_registry() -> &'static [ProviderRegistryEntry] {
    PROVIDER_TYPE_REGISTRY
}

pub fn entry_for_type(provider_type: &ProviderType) -> Option<&'static ProviderRegistryEntry> {
    PROVIDER_TYPE_REGISTRY
        .iter()
        .find(|entry| &entry.provider_type == provider_type)
}

pub fn entry_for_name(name: &str) -> Option<&'static ProviderRegistryEntry> {
    PROVIDER_TYPE_REGISTRY
        .iter()
        .find(|entry| entry.matches_name(name))
}

pub fn dispatchable_provider_types() -> Vec<ProviderType> {
    dispatchable_provider_types_slice().to_vec()
}

pub fn dispatchable_provider_types_slice() -> &'static [ProviderType] {
    DISPATCHABLE_PROVIDER_TYPES.as_slice()
}

const fn entry(
    provider_type: ProviderType,
    canonical_name: &'static str,
    aliases: &'static [&'static str],
    dispatch_kind: ProviderDispatchKind,
    catalog_backed: bool,
) -> ProviderRegistryEntry {
    ProviderRegistryEntry {
        provider_type,
        canonical_name,
        aliases,
        dispatch_kind,
        catalog_backed,
    }
}

const fn provider_extra_native_dispatch_kind() -> ProviderDispatchKind {
    if cfg!(feature = "providers-extra") {
        ProviderDispatchKind::Native
    } else {
        ProviderDispatchKind::ExplicitOpenAiLike
    }
}

const fn providers_extended_native_dispatch_kind() -> ProviderDispatchKind {
    if cfg!(feature = "providers-extended") {
        ProviderDispatchKind::Native
    } else {
        ProviderDispatchKind::UnsupportedEnum
    }
}

const fn provider_extra_only_native_dispatch_kind() -> ProviderDispatchKind {
    if cfg!(feature = "providers-extra") {
        ProviderDispatchKind::Native
    } else {
        ProviderDispatchKind::UnsupportedEnum
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::providers::{Provider, provider_type::all_non_custom_provider_types};
    use std::collections::{BTreeSet, HashSet};
    use std::str::FromStr;

    fn sorted_type_names(values: impl IntoIterator<Item = ProviderType>) -> Vec<String> {
        let mut values = values
            .into_iter()
            .map(|provider_type| provider_type.to_string())
            .collect::<Vec<_>>();
        values.sort();
        values
    }

    fn dispatch_kind_for(provider_type: &ProviderType) -> ProviderDispatchKind {
        match entry_for_type(provider_type) {
            Some(entry) => entry.dispatch_kind,
            None => panic!("missing provider registry entry for {:?}", provider_type),
        }
    }

    #[derive(Debug)]
    struct ReadmeTier2Row {
        selector: String,
        row: String,
    }

    fn readme_provider_support_section() -> &'static str {
        section_between(
            include_str!("../../../../README.md"),
            "## Provider Support",
            "## Environment Variables",
        )
    }

    fn section_between<'a>(text: &'a str, start: &str, end: &str) -> &'a str {
        let start_index = text
            .find(start)
            .unwrap_or_else(|| panic!("missing README section start: {start}"));
        let after_start = &text[start_index + start.len()..];
        let end_index = after_start
            .find(end)
            .unwrap_or_else(|| panic!("missing README section end: {end}"));
        &after_start[..end_index]
    }

    fn code_spans(line: &str) -> Vec<String> {
        let mut spans = Vec::new();
        let mut rest = line;

        while let Some(start) = rest.find('`') {
            let after_start = &rest[start + 1..];
            let Some(end) = after_start.find('`') else {
                break;
            };
            spans.push(after_start[..end].to_string());
            rest = &after_start[end + 1..];
        }

        spans
    }

    fn readme_tier2_rows() -> Vec<ReadmeTier2Row> {
        let tier2 = section_between(
            readme_provider_support_section(),
            "### Tier 2",
            "### Tier 1",
        );

        tier2
            .lines()
            .filter(|line| {
                let trimmed = line.trim_start();
                trimmed.starts_with('|')
                    && !trimmed.starts_with("| Provider")
                    && !trimmed.starts_with("|---")
                    && !trimmed.starts_with("|----------")
            })
            .map(|line| {
                let provider_cell = line.split('|').nth(1).unwrap_or("");
                let selector = code_spans(provider_cell)
                    .into_iter()
                    .next()
                    .unwrap_or_else(|| {
                        panic!("README Tier 2 row is missing selector in first column: {line}")
                    });
                ReadmeTier2Row {
                    selector,
                    row: line.to_string(),
                }
            })
            .collect()
    }

    fn readme_code_list_selectors(section_start: &str, section_end: &str) -> BTreeSet<String> {
        let section = section_between(
            readme_provider_support_section(),
            section_start,
            section_end,
        );

        section
            .lines()
            .filter(|line| line.trim_start().starts_with('`'))
            .flat_map(code_spans)
            .collect()
    }

    fn assert_readme_row_matches_dispatch_kind(
        row: &ReadmeTier2Row,
        entry: &ProviderRegistryEntry,
    ) {
        match entry.dispatch_kind {
            ProviderDispatchKind::Native => assert!(
                row.row.contains("native factory") || row.row.contains("always"),
                "README Tier 2 row for {} should document native dispatch: {}",
                row.selector,
                row.row
            ),
            ProviderDispatchKind::ExplicitOpenAiLike => assert!(
                row.row.contains("OpenAILike") || row.row.contains("OpenAI-compatible"),
                "README Tier 2 row for {} should document OpenAILike dispatch: {}",
                row.selector,
                row.row
            ),
            ProviderDispatchKind::CatalogOpenAiLike => assert!(
                row.row.contains("catalog-only"),
                "README Tier 2 row for {} should document catalog-only dispatch: {}",
                row.selector,
                row.row
            ),
            ProviderDispatchKind::UnsupportedEnum => assert!(
                row.row.contains("providers-extra") || row.row.contains("providers-extended"),
                "README Tier 2 row for {} should document the feature gate that makes it constructible: {}",
                row.selector,
                row.row
            ),
        }
    }

    #[test]
    fn provider_registry_contains_all_non_custom_provider_types() {
        assert_eq!(
            sorted_type_names(all_non_custom_provider_types()),
            sorted_type_names(
                PROVIDER_TYPE_REGISTRY
                    .iter()
                    .map(|entry| entry.provider_type.clone())
            )
        );
    }

    #[test]
    fn provider_registry_aliases_parse_to_declared_type() {
        for entry in PROVIDER_TYPE_REGISTRY {
            for name in entry.names() {
                assert_eq!(
                    ProviderType::from_str(name).unwrap(),
                    entry.provider_type,
                    "alias {name} should parse to {:?}",
                    entry.provider_type
                );
            }
        }
    }

    #[test]
    fn provider_registry_display_names_match_canonical_names() {
        for entry in PROVIDER_TYPE_REGISTRY {
            assert_eq!(entry.provider_type.to_string(), entry.canonical_name);
        }
    }

    #[test]
    fn provider_registry_native_entries_match_provider_enum_variants() {
        let native_types = PROVIDER_TYPE_REGISTRY
            .iter()
            .filter(|entry| entry.dispatch_kind == ProviderDispatchKind::Native)
            .map(|entry| entry.provider_type.clone())
            .collect::<HashSet<_>>();
        let expected = [
            ProviderType::OpenAI,
            ProviderType::Anthropic,
            ProviderType::Bedrock,
            ProviderType::Mistral,
            ProviderType::Cloudflare,
        ]
        .into_iter()
        .chain([
            #[cfg(feature = "providers-extra")]
            ProviderType::Azure,
            #[cfg(feature = "providers-extra")]
            ProviderType::AzureAI,
            #[cfg(feature = "providers-extra")]
            ProviderType::VertexAI,
            #[cfg(feature = "providers-extended")]
            ProviderType::Cohere,
            #[cfg(feature = "providers-extended")]
            ProviderType::FalAI,
            #[cfg(feature = "providers-extended")]
            ProviderType::Replicate,
            #[cfg(feature = "providers-extended")]
            ProviderType::Gemini,
            #[cfg(feature = "providers-extended")]
            ProviderType::GitHubCopilot,
        ])
        .collect::<HashSet<_>>();

        assert_eq!(native_types, expected);
    }

    #[test]
    fn provider_registry_phase0_key_provider_classifications() {
        assert_eq!(
            dispatch_kind_for(&ProviderType::Bedrock),
            ProviderDispatchKind::Native
        );
        assert_eq!(
            dispatch_kind_for(&ProviderType::VertexAI),
            provider_extra_only_native_dispatch_kind()
        );
        assert_eq!(
            dispatch_kind_for(&ProviderType::Gemini),
            providers_extended_native_dispatch_kind()
        );
        assert_eq!(
            dispatch_kind_for(&ProviderType::Azure),
            provider_extra_native_dispatch_kind()
        );
        assert_eq!(
            dispatch_kind_for(&ProviderType::AzureAI),
            provider_extra_native_dispatch_kind()
        );
        assert_eq!(
            dispatch_kind_for(&ProviderType::GitHubCopilot),
            providers_extended_native_dispatch_kind()
        );
        assert_eq!(
            dispatch_kind_for(&ProviderType::Cohere),
            providers_extended_native_dispatch_kind()
        );
        assert_eq!(
            dispatch_kind_for(&ProviderType::FalAI),
            providers_extended_native_dispatch_kind()
        );
        assert_eq!(
            dispatch_kind_for(&ProviderType::Replicate),
            providers_extended_native_dispatch_kind()
        );
        assert_eq!(
            dispatch_kind_for(&ProviderType::OpenAICompatible),
            ProviderDispatchKind::ExplicitOpenAiLike
        );
        for provider_type in [
            ProviderType::MetaLlama,
            ProviderType::V0,
            ProviderType::AmazonNova,
            ProviderType::GitHub,
        ] {
            assert_eq!(
                dispatch_kind_for(&provider_type),
                ProviderDispatchKind::CatalogOpenAiLike,
                "{provider_type:?} should be catalog-only metadata"
            );
        }
        assert_eq!(
            dispatch_kind_for(&ProviderType::PydanticAI),
            ProviderDispatchKind::UnsupportedEnum
        );
    }

    #[test]
    fn provider_registry_dispatchability_matches_factory_supported_types() {
        assert_eq!(
            sorted_type_names(dispatchable_provider_types()),
            sorted_type_names(Provider::factory_supported_provider_types().iter().cloned())
        );
    }

    #[test]
    fn provider_registry_catalog_flags_match_catalog() {
        for entry in PROVIDER_TYPE_REGISTRY {
            assert_eq!(
                super::super::catalog::get_definition(entry.canonical_name).is_some(),
                entry.catalog_backed,
                "{} catalog flag drifted",
                entry.canonical_name
            );
        }
    }

    #[test]
    fn provider_registry_names_are_unique() {
        let mut seen = HashSet::new();
        for entry in PROVIDER_TYPE_REGISTRY {
            for name in entry.names() {
                assert!(
                    seen.insert(name),
                    "duplicate provider registry name: {name}"
                );
            }
        }
    }

    #[test]
    fn provider_registry_readme_provider_support_matrix_matches_registry_and_catalog() {
        let tier2_rows = readme_tier2_rows();
        let tier1_selectors =
            readme_code_list_selectors("### Tier 1", "### Experimental / module-only");
        let experimental_selectors = readme_code_list_selectors(
            "### Experimental / module-only",
            "For self-hosted or unlisted OpenAI-compatible endpoints",
        );
        let mut documented_selectors = tier1_selectors.clone();

        assert!(
            !tier2_rows.is_empty(),
            "README Tier 2 matrix must not be empty"
        );
        assert!(
            !tier1_selectors.is_empty(),
            "README Tier 1 catalog list must not be empty"
        );
        assert!(
            !experimental_selectors.is_empty(),
            "README experimental provider list must not be empty"
        );

        for row in &tier2_rows {
            let entry = entry_for_name(&row.selector).unwrap_or_else(|| {
                panic!(
                    "README Tier 2 selector {} must exist in the provider registry",
                    row.selector
                )
            });
            assert_readme_row_matches_dispatch_kind(row, entry);
            documented_selectors.insert(row.selector.clone());
        }

        for selector in &tier1_selectors {
            assert!(
                super::super::catalog::PROVIDER_CATALOG.contains_key(selector.as_str()),
                "README Tier 1 selector {selector} must exist in the Tier 1 catalog"
            );
        }

        for selector in super::super::catalog::PROVIDER_CATALOG.keys() {
            assert!(
                documented_selectors.contains(*selector),
                "catalog selector {selector} must be documented in README provider support"
            );
        }

        for entry in PROVIDER_TYPE_REGISTRY {
            if entry.is_dispatchable() {
                assert!(
                    documented_selectors.contains(entry.canonical_name),
                    "dispatchable registry selector {} must be documented in README provider support",
                    entry.canonical_name
                );
            }
        }

        for selector in experimental_selectors {
            assert!(
                !super::super::catalog::PROVIDER_CATALOG.contains_key(selector.as_str()),
                "experimental selector {selector} must not be a Tier 1 catalog entry"
            );
            assert!(
                entry_for_name(&selector).is_none_or(|entry| !entry.is_dispatchable()),
                "experimental selector {selector} must not be dispatchable"
            );
        }
    }
}
