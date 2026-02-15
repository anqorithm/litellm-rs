//! Comet API Configuration

use crate::define_provider_config;

define_provider_config!(CometApiConfig, env_key: "COMET_API_KEY", provider: "comet_api");
