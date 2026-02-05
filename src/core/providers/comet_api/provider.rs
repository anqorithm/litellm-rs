//! Comet API Provider Implementation

crate::define_openai_compatible_provider!(
    provider: super::PROVIDER_NAME,
    struct_name: CometApiProvider,
    config: super::config::CometApiConfig,
    error_mapper: super::error_mapper::CometApiErrorMapper,
    model_info: super::model_info::get_supported_models,
    default_base_url: super::DEFAULT_BASE_URL,
    auth_header: "Authorization",
    auth_prefix: "Bearer ",
    supported_params: ["temperature", "max_tokens", "top_p", "stream", "stop"],
);
