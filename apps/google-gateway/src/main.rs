#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config_path =
        std::env::var("GATEWAY_CONFIG").unwrap_or_else(|_| "config/gateway.yaml".to_string());
    let cfg = litellm_contracts::GatewayConfig::from_yaml_file(&config_path)
        .map_err(|e| anyhow::anyhow!(e.to_string()))?;
    litellm_gateway::run(cfg).await
}
