use super::{
    PROVIDER_CATALOG, PROVIDER_TYPE_REGISTRY, ProviderDispatchKind, ProviderRegistryEntry,
    entry_for_name,
};
use crate::core::providers::provider_type::ProviderType;
use std::collections::BTreeSet;

#[derive(Debug)]
struct ReadmeTier2Row {
    selector: String,
    feature_cell: String,
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

fn markdown_table_cells(line: &str) -> Vec<String> {
    line.trim()
        .trim_matches('|')
        .split('|')
        .map(|cell| cell.trim().to_string())
        .collect()
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
            let cells = markdown_table_cells(line);
            let provider_cell = cells.first().map(String::as_str).unwrap_or("");
            let feature_cell = cells
                .get(1)
                .unwrap_or_else(|| panic!("README Tier 2 row is missing feature cell: {line}"))
                .clone();
            let selector = code_spans(provider_cell)
                .into_iter()
                .next()
                .unwrap_or_else(|| {
                    panic!("README Tier 2 row is missing selector in first column: {line}")
                });
            ReadmeTier2Row {
                selector,
                feature_cell,
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

fn expected_readme_feature_cell(entry: &ProviderRegistryEntry) -> Option<&'static str> {
    match entry.provider_type {
        ProviderType::OpenAI
        | ProviderType::Anthropic
        | ProviderType::Mistral
        | ProviderType::Cloudflare
        | ProviderType::Bedrock
        | ProviderType::OpenAICompatible => Some("always"),
        ProviderType::Azure | ProviderType::AzureAI => {
            Some("native factory (`providers-extra`); OpenAILike fallback")
        }
        ProviderType::VertexAI => Some("native factory (`providers-extra`)"),
        ProviderType::Cohere
        | ProviderType::Gemini
        | ProviderType::FalAI
        | ProviderType::Replicate
        | ProviderType::GitHubCopilot => Some("native factory (`providers-extended`)"),
        ProviderType::MetaLlama
        | ProviderType::V0
        | ProviderType::AmazonNova
        | ProviderType::GitHub => Some("catalog-only (`OpenAILike`)"),
        _ => None,
    }
}

fn expected_readme_tier2_selectors() -> BTreeSet<String> {
    PROVIDER_TYPE_REGISTRY
        .iter()
        .filter(|entry| expected_readme_feature_cell(entry).is_some())
        .map(|entry| entry.canonical_name.to_string())
        .collect()
}

fn assert_readme_row_matches_dispatch_kind(row: &ReadmeTier2Row, entry: &ProviderRegistryEntry) {
    let expected_feature_cell = expected_readme_feature_cell(entry).unwrap_or_else(|| {
        panic!(
            "README Tier 2 matrix should not document unsupported provider {}",
            entry.canonical_name
        )
    });
    assert_eq!(
        row.feature_cell, expected_feature_cell,
        "README Tier 2 row for {} should document the exact registry feature gate: {}",
        row.selector, row.row
    );

    match entry.dispatch_kind {
        ProviderDispatchKind::Native => assert!(
            row.feature_cell.contains("native factory") || row.feature_cell == "always",
            "README Tier 2 row for {} should document native dispatch: {}",
            row.selector,
            row.row
        ),
        ProviderDispatchKind::ExplicitOpenAiLike => assert!(
            row.feature_cell == "always" || row.feature_cell.contains("OpenAILike fallback"),
            "README Tier 2 row for {} should document OpenAILike dispatch: {}",
            row.selector,
            row.row
        ),
        ProviderDispatchKind::CatalogOpenAiLike => assert!(
            row.feature_cell.contains("catalog-only"),
            "README Tier 2 row for {} should document catalog-only dispatch: {}",
            row.selector,
            row.row
        ),
        ProviderDispatchKind::UnsupportedEnum => assert!(
            row.feature_cell.contains("providers-extra")
                || row.feature_cell.contains("providers-extended"),
            "README Tier 2 row for {} should document the feature gate that makes it constructible: {}",
            row.selector,
            row.row
        ),
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
    let expected_tier2_selectors = expected_readme_tier2_selectors();
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

    let tier2_selectors = tier2_rows
        .iter()
        .map(|row| row.selector.clone())
        .collect::<BTreeSet<_>>();
    assert_eq!(
        tier2_selectors, expected_tier2_selectors,
        "README Tier 2 rows must document every expected provider selector independent of active cargo features"
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
            PROVIDER_CATALOG.contains_key(selector.as_str()),
            "README Tier 1 selector {selector} must exist in the Tier 1 catalog"
        );
    }

    for selector in PROVIDER_CATALOG.keys() {
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
            !PROVIDER_CATALOG.contains_key(selector.as_str()),
            "experimental selector {selector} must not be a Tier 1 catalog entry"
        );
        assert!(
            !expected_tier2_selectors.contains(&selector),
            "experimental selector {selector} must not be a Tier 2 provider support row"
        );
        assert!(
            entry_for_name(&selector).is_none_or(|entry| !entry.is_dispatchable()),
            "experimental selector {selector} must not be dispatchable under active features"
        );
    }
}
