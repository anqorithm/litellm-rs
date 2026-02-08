//! Empower Configuration

use crate::core::traits::provider::ProviderConfig;
use crate::define_provider_config;

define_provider_config!(EmpowerConfig {});

impl EmpowerConfig {
    pub fn from_env() -> Self {
        Self::new("empower")
    }

    pub fn get_api_base(&self) -> String {
        self.base
            .api_base
            .clone()
            .unwrap_or_else(|| "https://api.empower.dev/v1".to_string())
    }
}

impl ProviderConfig for EmpowerConfig {
    fn validate(&self) -> Result<(), String> {
        self.base.validate("empower")
    }

    fn api_key(&self) -> Option<&str> {
        self.base.api_key.as_deref()
    }

    fn api_base(&self) -> Option<&str> {
        self.base.api_base.as_deref()
    }

    fn timeout(&self) -> std::time::Duration {
        self.base.timeout_duration()
    }

    fn max_retries(&self) -> u32 {
        self.base.max_retries
    }
}
