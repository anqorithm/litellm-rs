//! Firecrawl Configuration

use crate::define_provider_config;

define_provider_config!(FirecrawlConfig, provider: "firecrawl");

impl FirecrawlConfig {
    pub fn get_api_base(&self) -> String {
        self.base
            .api_base
            .clone()
            .unwrap_or_else(|| "https://api.firecrawl.dev/v1".to_string())
    }
}
