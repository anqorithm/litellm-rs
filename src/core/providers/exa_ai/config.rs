//! ExaAi Configuration

use crate::define_provider_config;

define_provider_config!(ExaAiConfig, provider: "exa_ai");

impl ExaAiConfig {
    pub fn get_api_base(&self) -> String {
        self.base
            .api_base
            .clone()
            .unwrap_or_else(|| "https://api.exa.ai/v1".to_string())
    }
}
