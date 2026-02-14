//! Empower Configuration

use crate::define_provider_config;

define_provider_config!(EmpowerConfig, provider: "empower");

impl EmpowerConfig {
    pub fn get_api_base(&self) -> String {
        self.base
            .api_base
            .clone()
            .unwrap_or_else(|| "https://api.empower.dev/v1".to_string())
    }
}
