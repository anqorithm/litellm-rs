use super::{
    DEFAULT_CATALOG_RUNTIME_PROVIDERS, PROVIDER_CATALOG, ProviderRouteSurface, canonical_selector,
    provider_type_registry, selector_has_matrix_entry, supports_provider_surface,
};

#[test]
fn support_matrix_covers_registry_and_catalog_selectors() {
    for entry in provider_type_registry() {
        assert!(
            selector_has_matrix_entry(entry.canonical_name),
            "missing support matrix row for {}",
            entry.canonical_name
        );
    }

    for selector in PROVIDER_CATALOG.keys() {
        assert!(
            selector_has_matrix_entry(selector),
            "missing support matrix fallback for catalog selector {selector}"
        );
    }
}

#[test]
fn default_completion_catalog_routes_are_marked_supported() {
    for selector in DEFAULT_CATALOG_RUNTIME_PROVIDERS {
        assert!(
            supports_provider_surface(selector, ProviderRouteSurface::CompletionChat),
            "{selector} should support completion() chat"
        );
        assert!(
            supports_provider_surface(selector, ProviderRouteSurface::CompletionChatStream),
            "{selector} should support completion() streaming"
        );
    }
}

#[test]
fn sdk_matrix_rejects_google_chat_until_adapter_exists() {
    assert!(supports_provider_surface(
        "openai",
        ProviderRouteSurface::SdkChat
    ));
    assert!(supports_provider_surface(
        "anthropic",
        ProviderRouteSurface::SdkChatStream
    ));
    assert!(!supports_provider_surface(
        "google",
        ProviderRouteSurface::SdkChat
    ));
    assert!(!supports_provider_surface(
        "gemini",
        ProviderRouteSurface::SdkChat
    ));
}

#[test]
fn completion_matrix_matches_default_router_support() {
    for surface in [
        ProviderRouteSurface::CompletionChat,
        ProviderRouteSurface::CompletionChatStream,
    ] {
        assert!(supports_provider_surface("azure", surface));
        assert!(!supports_provider_surface("bedrock", surface));
    }
    assert_eq!(
        supports_provider_surface("azure_ai", ProviderRouteSurface::CompletionChat),
        cfg!(feature = "providers-extra")
    );
}

#[test]
fn catalog_fallback_is_http_chat_only() {
    assert!(supports_provider_surface(
        "together",
        ProviderRouteSurface::HttpChat
    ));
    assert!(supports_provider_surface(
        "together",
        ProviderRouteSurface::HttpChatStream
    ));
    assert!(!supports_provider_surface(
        "together",
        ProviderRouteSurface::CompletionChat
    ));
    assert!(!supports_provider_surface(
        "together",
        ProviderRouteSurface::SdkChat
    ));
}

#[test]
fn selector_aliases_resolve_to_canonical_matrix_entries() {
    assert_eq!(canonical_selector("azure-openai"), "azure");
    assert_eq!(canonical_selector("google_vertex"), "vertex_ai");
    assert_eq!(canonical_selector("aws_bedrock"), "bedrock");
    assert_eq!(canonical_selector("openai-like"), "openai_compatible");
}
