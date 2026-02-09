#[derive(Debug, Clone)]
pub struct RuntimeConfig {
    pub gateway: litellm_contracts::GatewayConfig,
}

impl RuntimeConfig {
    pub fn new(gateway: litellm_contracts::GatewayConfig) -> Self {
        Self { gateway }
    }
}
