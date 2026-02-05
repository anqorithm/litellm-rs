//! Bytez AI Provider Implementation

crate::define_openai_compatible_provider!(
    provider: super::PROVIDER_NAME,
    struct_name: BytezProvider,
    config: super::config::BytezConfig,
    error_mapper: super::error_mapper::BytezErrorMapper,
    model_info: super::model_info::get_supported_models,
    default_base_url: super::DEFAULT_BASE_URL,
    auth_header: "Authorization",
    auth_prefix: "Bearer ",
    supported_params: ["temperature", "max_tokens", "top_p", "stream", "stop"],
);
