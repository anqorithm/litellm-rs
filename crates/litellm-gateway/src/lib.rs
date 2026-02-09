pub mod auth;
pub mod deployment;
pub mod execute_impl;
pub mod gateway_config;
pub mod server;
pub mod state;
pub mod storage;

pub async fn build_router(
    config: &litellm_contracts::GatewayConfig,
) -> anyhow::Result<litellm_router::Router> {
    let registry = litellm_providers::default_registry();
    let router = litellm_router::Router::from_gateway_config(&config.providers, &registry)
        .await
        .map_err(|e| anyhow::anyhow!(e.to_string()))?;
    Ok(router)
}

pub async fn run(config: litellm_contracts::GatewayConfig) -> anyhow::Result<()> {
    let router = build_router(&config).await?;
    let state = state::GatewayState::new(router);
    tracing::info!("litellm-gateway runtime initialized");
    server::run_server(&config, state).await
}
