use crate::core::providers::unified_provider::ProviderError;

use super::capabilities::ModelCapabilities;

pub struct ModelUtils;

impl ModelUtils {
    pub fn get_model_capabilities(model: &str) -> ModelCapabilities {
        let model_lower = model.to_lowercase();

        if model_lower.starts_with("gpt-5") {
            ModelCapabilities {
                supports_function_calling: true,
                supports_parallel_function_calling: true,
                supports_tool_choice: true,
                supports_response_schema: true,
                supports_system_messages: true,
                supports_web_search: false,
                supports_url_context: true,
                supports_vision: true,
                supports_streaming: true,
                max_tokens: Some(128000),
                context_window: Some(400000),
            }
        } else if model_lower.starts_with("gpt-image-") || model_lower.starts_with("chatgpt-image-")
        {
            ModelCapabilities {
                supports_function_calling: false,
                supports_parallel_function_calling: false,
                supports_tool_choice: false,
                supports_response_schema: false,
                supports_system_messages: false,
                supports_web_search: false,
                supports_url_context: false,
                supports_vision: true,
                supports_streaming: false,
                max_tokens: Some(16384),
                context_window: Some(128000),
            }
        } else if model_lower.starts_with("gpt-4.1") {
            ModelCapabilities {
                supports_function_calling: true,
                supports_parallel_function_calling: true,
                supports_tool_choice: true,
                supports_response_schema: true,
                supports_system_messages: true,
                supports_web_search: false,
                supports_url_context: true,
                supports_vision: true,
                supports_streaming: true,
                max_tokens: Some(32768),
                context_window: Some(128000),
            }
        } else if model_lower.starts_with("o3") || model_lower.starts_with("o4") {
            ModelCapabilities {
                supports_function_calling: true,
                supports_parallel_function_calling: false,
                supports_tool_choice: true,
                supports_response_schema: true,
                supports_system_messages: true,
                supports_web_search: false,
                supports_url_context: true,
                supports_vision: false,
                supports_streaming: true,
                max_tokens: Some(100000),
                context_window: Some(200000),
            }
        } else if model_lower.starts_with("gpt-4") {
            ModelCapabilities {
                supports_function_calling: true,
                supports_parallel_function_calling: true,
                supports_tool_choice: true,
                supports_response_schema: true,
                supports_system_messages: true,
                supports_web_search: false,
                supports_url_context: true,
                supports_vision: model_lower.contains("vision") || model_lower.contains("turbo"),
                supports_streaming: true,
                max_tokens: Some(if model_lower.contains("32k") {
                    32768
                } else {
                    8192
                }),
                context_window: Some(if model_lower.contains("32k") {
                    32768
                } else {
                    8192
                }),
            }
        } else if model_lower.starts_with("gpt-3.5") {
            ModelCapabilities {
                supports_function_calling: true,
                supports_parallel_function_calling: false,
                supports_tool_choice: true,
                supports_response_schema: false,
                supports_system_messages: true,
                supports_web_search: false,
                supports_url_context: false,
                supports_vision: false,
                supports_streaming: true,
                max_tokens: Some(if model_lower.contains("16k") {
                    16384
                } else {
                    4096
                }),
                context_window: Some(if model_lower.contains("16k") {
                    16384
                } else {
                    4096
                }),
            }
        } else if model_lower.starts_with("claude-opus-4-6") {
            ModelCapabilities {
                supports_function_calling: true,
                supports_parallel_function_calling: false,
                supports_tool_choice: true,
                supports_response_schema: false,
                supports_system_messages: true,
                supports_web_search: false,
                supports_url_context: true,
                supports_vision: true,
                supports_streaming: true,
                max_tokens: Some(1_000_000),
                context_window: Some(1_000_000),
            }
        } else if model_lower.starts_with("claude-opus-4")
            || model_lower.starts_with("claude-sonnet-4")
            || model_lower.starts_with("claude-3")
        {
            ModelCapabilities {
                supports_function_calling: true,
                supports_parallel_function_calling: false,
                supports_tool_choice: true,
                supports_response_schema: false,
                supports_system_messages: true,
                supports_web_search: false,
                supports_url_context: true,
                supports_vision: true,
                supports_streaming: true,
                max_tokens: Some(200000),
                context_window: Some(200000),
            }
        } else if model_lower.starts_with("claude-2") || model_lower.starts_with("claude-instant") {
            ModelCapabilities {
                supports_function_calling: false,
                supports_parallel_function_calling: false,
                supports_tool_choice: false,
                supports_response_schema: false,
                supports_system_messages: true,
                supports_web_search: false,
                supports_url_context: false,
                supports_vision: false,
                supports_streaming: true,
                max_tokens: Some(100000),
                context_window: Some(100000),
            }
        } else if model_lower.starts_with("gemini") {
            ModelCapabilities {
                supports_function_calling: true,
                supports_parallel_function_calling: false,
                supports_tool_choice: false,
                supports_response_schema: false,
                supports_system_messages: true,
                supports_web_search: true,
                supports_url_context: true,
                supports_vision: model_lower.contains("vision") || model_lower.contains("pro"),
                supports_streaming: true,
                max_tokens: Some(32768),
                context_window: Some(32768),
            }
        } else {
            ModelCapabilities::default()
        }
    }

    pub fn supports_function_calling(model: &str) -> bool {
        Self::get_model_capabilities(model).supports_function_calling
    }

    pub fn supports_parallel_function_calling(model: &str) -> bool {
        Self::get_model_capabilities(model).supports_parallel_function_calling
    }

    pub fn supports_tool_choice(model: &str) -> bool {
        Self::get_model_capabilities(model).supports_tool_choice
    }

    pub fn supports_response_schema(model: &str) -> bool {
        Self::get_model_capabilities(model).supports_response_schema
    }

    pub fn supports_system_messages(model: &str) -> bool {
        Self::get_model_capabilities(model).supports_system_messages
    }

    pub fn supports_web_search(model: &str) -> bool {
        Self::get_model_capabilities(model).supports_web_search
    }

    pub fn supports_url_context(model: &str) -> bool {
        Self::get_model_capabilities(model).supports_url_context
    }

    pub fn supports_vision(model: &str) -> bool {
        Self::get_model_capabilities(model).supports_vision
    }

    pub fn supports_streaming(model: &str) -> bool {
        Self::get_model_capabilities(model).supports_streaming
    }

    pub fn get_provider_from_model(model: &str) -> Option<String> {
        let model_lower = model.to_lowercase();

        if model_lower.starts_with("gpt-")
            || model_lower.starts_with("chatgpt-image-")
            || model_lower.contains("openai")
        {
            Some("openai".to_string())
        } else if model_lower.starts_with("claude-") || model_lower.contains("anthropic") {
            Some("anthropic".to_string())
        } else if model_lower.starts_with("gemini-") || model_lower.contains("google") {
            Some("google".to_string())
        } else if model_lower.starts_with("command") || model_lower.contains("cohere") {
            Some("cohere".to_string())
        } else if model_lower.contains("mistral") {
            Some("mistral".to_string())
        } else if model_lower.contains("llama") {
            Some("meta".to_string())
        } else {
            None
        }
    }

    pub fn get_base_model(model: &str) -> String {
        let model_lower = model.to_lowercase();

        if model_lower.starts_with("gpt-5") {
            if model_lower.contains("nano") {
                "gpt-5-nano".to_string()
            } else if model_lower.contains("mini") {
                "gpt-5-mini".to_string()
            } else if model_lower.contains("codex") {
                if model_lower.contains("5.2") {
                    "gpt-5.2-codex".to_string()
                } else {
                    "gpt-5-codex".to_string()
                }
            } else if model_lower.contains("chat") {
                "gpt-5.2-chat".to_string()
            } else {
                "gpt-5.2".to_string()
            }
        } else if model_lower.starts_with("gpt-image-") || model_lower.starts_with("chatgpt-image-")
        {
            if model_lower.contains("1-mini") {
                "gpt-image-1-mini".to_string()
            } else if model_lower.contains("1.5") || model_lower.starts_with("chatgpt-image-") {
                "gpt-image-1.5".to_string()
            } else {
                "gpt-image-1".to_string()
            }
        } else if model_lower.starts_with("gpt-4.1") {
            if model_lower.contains("nano") {
                "gpt-4.1-nano".to_string()
            } else if model_lower.contains("mini") {
                "gpt-4.1-mini".to_string()
            } else {
                "gpt-4.1".to_string()
            }
        } else if model_lower.starts_with("o3-pro") {
            "o3-pro".to_string()
        } else if model_lower.starts_with("gpt-4") {
            if model_lower.contains("32k") {
                "gpt-4-32k".to_string()
            } else if model_lower.contains("turbo") {
                "gpt-4-turbo".to_string()
            } else {
                "gpt-4".to_string()
            }
        } else if model_lower.starts_with("gpt-3.5") {
            if model_lower.contains("16k") {
                "gpt-3.5-turbo-16k".to_string()
            } else {
                "gpt-3.5-turbo".to_string()
            }
        } else if model_lower.starts_with("claude-opus-4-6") {
            "claude-opus-4-6".to_string()
        } else if model_lower.starts_with("claude-opus-4-5") {
            "claude-opus-4-5".to_string()
        } else if model_lower.starts_with("claude-sonnet-4-5") {
            "claude-sonnet-4-5".to_string()
        } else if model_lower.starts_with("claude-sonnet-4") {
            "claude-sonnet-4".to_string()
        } else if model_lower.starts_with("claude-3") {
            if model_lower.contains("opus") {
                "claude-3-opus".to_string()
            } else if model_lower.contains("sonnet") {
                "claude-3-sonnet".to_string()
            } else if model_lower.contains("haiku") {
                "claude-3-haiku".to_string()
            } else {
                "claude-3".to_string()
            }
        } else {
            model.to_string()
        }
    }

    pub fn is_valid_model(model: &str) -> bool {
        let known_providers = [
            "openai",
            "anthropic",
            "google",
            "cohere",
            "mistral",
            "meta",
            "azure",
            "replicate",
        ];

        let known_models = [
            "gpt-5.2",
            "gpt-5-codex",
            "gpt-5-mini",
            "gpt-5-nano",
            "gpt-image-1",
            "gpt-4.1",
            "gpt-4",
            "gpt-3.5-turbo",
            "o3-pro",
            "o3-mini",
            "o4-mini",
            "claude-opus-4",
            "claude-sonnet-4",
            "claude-3",
            "claude-2",
            "gemini",
            "command",
            "mistral",
        ];

        let model_lower = model.to_lowercase();

        for provider in &known_providers {
            if model_lower.contains(provider) {
                return true;
            }
        }

        for base_model in &known_models {
            if model_lower.starts_with(base_model) {
                return true;
            }
        }

        false
    }

    pub fn get_model_family(model: &str) -> String {
        let model_lower = model.to_lowercase();

        if model_lower.starts_with("gpt-") {
            "gpt".to_string()
        } else if model_lower.starts_with("claude-") {
            "claude".to_string()
        } else if model_lower.starts_with("gemini-") {
            "gemini".to_string()
        } else if model_lower.starts_with("command") {
            "command".to_string()
        } else if model_lower.contains("llama") {
            "llama".to_string()
        } else if model_lower.contains("mistral") {
            "mistral".to_string()
        } else {
            "unknown".to_string()
        }
    }

    pub fn validate_model_with_provider(model: &str, provider: &str) -> Result<(), ProviderError> {
        let compatible_models = Self::get_compatible_models_for_provider(provider);

        if compatible_models.is_empty() {
            return Ok(());
        }

        let model_matches = compatible_models.iter().any(|compatible_model| {
            model
                .to_lowercase()
                .starts_with(&compatible_model.to_lowercase())
        });

        if !model_matches {
            return Err(ProviderError::ModelNotFound {
                provider: "unknown",
                model: format!(
                    "Model '{}' is not compatible with provider '{}'",
                    model, provider
                ),
            });
        }

        Ok(())
    }

    pub fn get_compatible_models_for_provider(provider: &str) -> Vec<String> {
        match provider.to_lowercase().as_str() {
            "openai" => vec![
                "gpt-5.2".to_string(),
                "gpt-5.2-chat".to_string(),
                "gpt-5.2-codex".to_string(),
                "gpt-5-codex".to_string(),
                "gpt-5.1".to_string(),
                "gpt-5.1-thinking".to_string(),
                "gpt-5-mini".to_string(),
                "gpt-5-nano".to_string(),
                "gpt-image-1".to_string(),
                "gpt-image-1-mini".to_string(),
                "gpt-image-1.5".to_string(),
                "chatgpt-image-latest".to_string(),
                "o3-pro".to_string(),
                "o3-mini".to_string(),
                "o4-mini".to_string(),
                "gpt-4.1".to_string(),
                "gpt-4.1-mini".to_string(),
                "gpt-4.1-nano".to_string(),
                "gpt-4".to_string(),
                "gpt-4-turbo".to_string(),
                "gpt-4-32k".to_string(),
                "gpt-3.5-turbo".to_string(),
                "gpt-3.5-turbo-16k".to_string(),
            ],
            "anthropic" => vec![
                "claude-opus-4-6".to_string(),
                "claude-opus-4-5".to_string(),
                "claude-sonnet-4-5".to_string(),
                "claude-sonnet-4".to_string(),
                "claude-3-opus".to_string(),
                "claude-3-sonnet".to_string(),
                "claude-3-haiku".to_string(),
                "claude-2".to_string(),
                "claude-instant".to_string(),
            ],
            "google" => vec![
                "gemini-pro".to_string(),
                "gemini-pro-vision".to_string(),
                "gemini-1.5-pro".to_string(),
            ],
            "cohere" => vec![
                "command".to_string(),
                "command-r".to_string(),
                "command-r-plus".to_string(),
            ],
            "mistral" => vec![
                "mistral-tiny".to_string(),
                "mistral-small".to_string(),
                "mistral-medium".to_string(),
                "mistral-large".to_string(),
            ],
            _ => vec![],
        }
    }
}

#[cfg(test)]
#[path = "utils_tests.rs"]
mod tests;
