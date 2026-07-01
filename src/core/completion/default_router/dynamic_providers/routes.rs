use crate::core::completion::CompletionOptions;

pub(super) struct DynamicProviderRoute<'a> {
    pub(super) provider_type: &'static str,
    pub(super) provider_label: &'static str,
    pub(super) actual_model: &'a str,
    pub(super) api_base: String,
}

struct DynamicProviderPrefix {
    prefix: &'static str,
    provider_type: &'static str,
    provider_label: &'static str,
    default_api_base: &'static str,
}

const DYNAMIC_PROVIDER_PREFIXES: &[DynamicProviderPrefix] = &[
    prefix(
        "openrouter/",
        "openrouter",
        "OpenRouter",
        "https://openrouter.ai/api/v1",
    ),
    prefix(
        "anthropic/",
        "anthropic",
        "Anthropic",
        "https://api.anthropic.com",
    ),
    prefix(
        "deepseek/",
        "deepseek",
        "DeepSeek",
        "https://api.deepseek.com",
    ),
    prefix(
        "moonshot/",
        "moonshot",
        "Moonshot",
        "https://api.moonshot.cn/v1",
    ),
    prefix(
        "minimax/",
        "minimax",
        "MiniMax",
        "https://api.minimax.chat/v1",
    ),
    prefix(
        "zhipu/",
        "zhipu",
        "Zhipu",
        "https://open.bigmodel.cn/api/paas/v4",
    ),
    prefix(
        "glm/",
        "zhipu",
        "Zhipu",
        "https://open.bigmodel.cn/api/paas/v4",
    ),
    prefix("zai/", "zai", "ZAI", "https://api.z.ai/api/paas/v4"),
    prefix("xai/", "xai", "xAI", "https://api.x.ai/v1"),
    prefix(
        "together_ai/",
        "together_ai",
        "Together AI",
        "https://api.together.xyz/v1",
    ),
    prefix(
        "together/",
        "together",
        "Together AI",
        "https://api.together.xyz/v1",
    ),
    prefix(
        "fireworks_ai/",
        "fireworks_ai",
        "Fireworks AI",
        "https://api.fireworks.ai/inference/v1",
    ),
    prefix(
        "fireworks/",
        "fireworks",
        "Fireworks AI",
        "https://api.fireworks.ai/inference/v1",
    ),
    prefix("aiml/", "aiml", "AIML API", "https://api.aimlapi.com/v1"),
    prefix(
        "aiml_api/",
        "aiml_api",
        "AIML API",
        "https://api.aimlapi.com/v1",
    ),
    prefix("groq/", "groq", "Groq", "https://api.groq.com/openai/v1"),
    prefix(
        "xiaomi_mimo/",
        "xiaomi_mimo",
        "Xiaomi MiMo",
        "https://api.xiaomimimo.com/v1",
    ),
    prefix(
        "mimo/",
        "xiaomi_mimo",
        "Xiaomi MiMo",
        "https://api.xiaomimimo.com/v1",
    ),
    prefix("openai/", "openai", "OpenAI", "https://api.openai.com/v1"),
];

const fn prefix(
    prefix: &'static str,
    provider_type: &'static str,
    provider_label: &'static str,
    default_api_base: &'static str,
) -> DynamicProviderPrefix {
    DynamicProviderPrefix {
        prefix,
        provider_type,
        provider_label,
        default_api_base,
    }
}

pub(super) fn resolve_dynamic_provider_route<'a>(
    model: &'a str,
    options: &CompletionOptions,
) -> Option<DynamicProviderRoute<'a>> {
    for config in DYNAMIC_PROVIDER_PREFIXES {
        if let Some(actual_model) = model.strip_prefix(config.prefix) {
            let api_base = options
                .api_base
                .clone()
                .unwrap_or_else(|| config.default_api_base.to_string());
            return Some(DynamicProviderRoute {
                provider_type: config.provider_type,
                provider_label: config.provider_label,
                actual_model,
                api_base,
            });
        }
    }

    if model.starts_with("azure_ai/") || model.starts_with("azure-ai/") {
        let actual_model = model
            .strip_prefix("azure_ai/")
            .or_else(|| model.strip_prefix("azure-ai/"))
            .unwrap_or(model);
        let api_base = options
            .api_base
            .clone()
            .or_else(|| std::env::var("AZURE_AI_API_BASE").ok())
            .unwrap_or_else(|| "https://api.azure.com".to_string());
        return Some(DynamicProviderRoute {
            provider_type: "azure_ai",
            provider_label: "Azure AI",
            actual_model,
            api_base,
        });
    }

    options
        .api_base
        .clone()
        .map(|api_base| DynamicProviderRoute {
            provider_type: "openai-compatible",
            provider_label: "OpenAI-Compatible",
            actual_model: model,
            api_base,
        })
}

pub(in crate::core::completion::default_router) fn is_named_dynamic_provider_route(
    model: &str,
    options: &CompletionOptions,
) -> bool {
    resolve_dynamic_provider_route(model, options)
        .map(|route| route.provider_type != "openai-compatible")
        .unwrap_or(false)
}

pub(super) fn resolve_dynamic_provider_api_key(
    options: &CompletionOptions,
    route: &DynamicProviderRoute<'_>,
) -> Option<String> {
    resolve_dynamic_provider_api_key_from_sources(
        options,
        route,
        dynamic_provider_api_key(route),
        std::env::var("OPENAI_API_KEY").ok(),
    )
}

pub(super) fn resolve_dynamic_provider_api_key_from_sources(
    options: &CompletionOptions,
    route: &DynamicProviderRoute<'_>,
    provider_api_key: Option<String>,
    openai_api_key: Option<String>,
) -> Option<String> {
    if let Some(api_key) = options.api_key.clone() {
        return Some(api_key);
    }

    if route.provider_type != "openai-compatible"
        && let Some(api_key) = provider_api_key
    {
        return Some(api_key);
    }

    options.api_base.as_ref()?;
    custom_api_base_api_key_fallback(options, route, openai_api_key, None)
}

fn dynamic_provider_api_key(route: &DynamicProviderRoute<'_>) -> Option<String> {
    if let Some(definition) = crate::core::providers::registry::get_definition(route.provider_type)
    {
        return definition.resolve_api_key(None);
    }

    dynamic_provider_api_key_env_var(route).and_then(|env_var| std::env::var(env_var).ok())
}

pub(super) fn dynamic_provider_api_key_env_var(
    route: &DynamicProviderRoute<'_>,
) -> Option<&'static str> {
    match route.provider_type {
        "openai" => Some("OPENAI_API_KEY"),
        "openrouter" => Some("OPENROUTER_API_KEY"),
        "anthropic" => Some("ANTHROPIC_API_KEY"),
        "deepseek" => Some("DEEPSEEK_API_KEY"),
        "moonshot" => Some("MOONSHOT_API_KEY"),
        "minimax" => Some("MINIMAX_API_KEY"),
        "zhipu" => Some("ZHIPU_API_KEY"),
        "zai" => Some("ZAI_API_KEY"),
        "together" | "together_ai" => Some("TOGETHER_API_KEY"),
        "fireworks" | "fireworks_ai" => Some("FIREWORKS_API_KEY"),
        "aiml" | "aiml_api" => Some("AIML_API_KEY"),
        "xai" => Some("XAI_API_KEY"),
        "groq" => Some("GROQ_API_KEY"),
        "xiaomi_mimo" => Some("MIMO_API_KEY"),
        "azure_ai" => Some("AZURE_AI_API_KEY"),
        _ => None,
    }
}

fn is_openai_compatible_api_key_fallback(route: &DynamicProviderRoute<'_>) -> bool {
    matches!(
        route.provider_type,
        "openai-compatible"
            | "openai"
            | "openrouter"
            | "deepseek"
            | "moonshot"
            | "minimax"
            | "zhipu"
            | "zai"
            | "together"
            | "together_ai"
            | "fireworks"
            | "fireworks_ai"
            | "aiml"
            | "aiml_api"
            | "xai"
            | "groq"
            | "xiaomi_mimo"
    )
}

pub(super) fn uses_dynamic_openai_like_provider(route: &DynamicProviderRoute<'_>) -> bool {
    matches!(
        route.provider_type,
        "openrouter"
            | "xai"
            | "groq"
            | "xiaomi_mimo"
            | "zai"
            | "together"
            | "together_ai"
            | "fireworks"
            | "fireworks_ai"
            | "aiml"
            | "aiml_api"
    )
}

pub(super) fn custom_api_base_api_key_fallback(
    options: &CompletionOptions,
    route: &DynamicProviderRoute<'_>,
    openai_api_key: Option<String>,
    provider_api_key: Option<String>,
) -> Option<String> {
    options.api_base.as_ref()?;

    provider_api_key.or_else(|| {
        is_openai_compatible_api_key_fallback(route)
            .then(|| openai_api_key.unwrap_or_else(|| "dummy-key-for-local".to_string()))
    })
}
