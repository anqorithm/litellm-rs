//! A2A Gateway
//!
//! Main gateway for managing agents and routing A2A requests.

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use super::config::{A2AGatewayConfig, AgentConfig, AgentProvider};
use super::error::{A2AError, A2AResult};
use super::message::{A2AMessage, A2AResponse, TaskResult};
use super::provider::{A2AProviderAdapter, get_provider_adapter};
use super::registry::{AgentRegistry, AgentState, RegistryStats};

/// A2A Gateway - main entry point for A2A protocol functionality
pub struct A2AGateway {
    /// Agent registry
    registry: Arc<AgentRegistry>,

    /// Provider adapters cache
    adapters: RwLock<HashMap<AgentProvider, Arc<dyn A2AProviderAdapter>>>,

    /// Enable request logging
    enable_logging: bool,

    /// Enable cost tracking
    enable_cost_tracking: bool,
}

impl Default for A2AGateway {
    fn default() -> Self {
        Self::new()
    }
}

impl A2AGateway {
    /// Create a new A2A Gateway
    pub fn new() -> Self {
        Self {
            registry: Arc::new(AgentRegistry::new()),
            adapters: RwLock::new(HashMap::new()),
            enable_logging: true,
            enable_cost_tracking: false,
        }
    }

    /// Create a gateway from configuration
    pub async fn from_config(config: A2AGatewayConfig) -> A2AResult<Self> {
        let gateway = Self {
            registry: Arc::new(AgentRegistry::new()),
            adapters: RwLock::new(HashMap::new()),
            enable_logging: config.enable_logging,
            enable_cost_tracking: config.enable_cost_tracking,
        };

        // Register all agents from config
        for (name, mut agent_config) in config.agents {
            if agent_config.name.is_empty() {
                agent_config.name = name;
            }
            gateway.register_agent(agent_config).await?;
        }

        Ok(gateway)
    }

    /// Register a new agent
    pub async fn register_agent(&self, config: AgentConfig) -> A2AResult<()> {
        self.registry.register(config).await
    }

    /// Unregister an agent
    pub async fn unregister_agent(&self, name: &str) -> bool {
        self.registry.unregister(name).await.is_some()
    }

    /// Get an agent configuration
    pub async fn get_agent(&self, name: &str) -> A2AResult<AgentConfig> {
        self.registry
            .get_config(name)
            .await
            .ok_or_else(|| A2AError::AgentNotFound {
                agent_name: name.to_string(),
            })
    }

    /// List all registered agent names
    pub async fn list_agents(&self) -> Vec<String> {
        self.registry.list_names().await
    }

    /// List available agents
    pub async fn list_available_agents(&self) -> Vec<AgentConfig> {
        self.registry.list_available().await
    }

    /// Send a message to an agent
    pub async fn send_message(
        &self,
        agent_name: &str,
        message: impl Into<String>,
    ) -> A2AResult<A2AResponse> {
        let config = self.get_agent(agent_name).await?;
        let adapter = self.get_adapter(config.provider).await;

        let a2a_message = A2AMessage::send(message);

        if self.enable_logging {
            tracing::info!(
                agent = agent_name,
                method = "message/send",
                "Sending A2A message"
            );
        }

        let response = adapter.send_message(&config, a2a_message).await?;

        // Update agent state based on response
        if response.is_success() {
            self.registry
                .update_state(agent_name, AgentState::Healthy)
                .await;
        } else if response.is_error() {
            self.registry
                .update_state(agent_name, AgentState::Degraded)
                .await;
        }

        // Record invocation
        if self.enable_cost_tracking {
            let cost = config.cost_per_request.unwrap_or(0.0);
            self.registry.record_invocation(agent_name, cost).await;
        }

        Ok(response)
    }

    /// Send a full A2A message to an agent
    pub async fn send(&self, agent_name: &str, message: A2AMessage) -> A2AResult<A2AResponse> {
        let config = self.get_agent(agent_name).await?;
        let adapter = self.get_adapter(config.provider).await;

        if self.enable_logging {
            tracing::info!(
                agent = agent_name,
                method = %message.method,
                "Sending A2A request"
            );
        }

        let response = adapter.send_message(&config, message).await?;

        // Update stats
        if self.enable_cost_tracking {
            let cost = config.cost_per_request.unwrap_or(0.0);
            self.registry.record_invocation(agent_name, cost).await;
        }

        Ok(response)
    }

    /// Get task status from an agent
    pub async fn get_task(&self, agent_name: &str, task_id: &str) -> A2AResult<TaskResult> {
        let config = self.get_agent(agent_name).await?;
        let adapter = self.get_adapter(config.provider).await;

        if self.enable_logging {
            tracing::info!(agent = agent_name, task_id = task_id, "Getting task status");
        }

        adapter.get_task(&config, task_id).await
    }

    /// Cancel a task
    pub async fn cancel_task(&self, agent_name: &str, task_id: &str) -> A2AResult<()> {
        let config = self.get_agent(agent_name).await?;
        let adapter = self.get_adapter(config.provider).await;

        if self.enable_logging {
            tracing::info!(agent = agent_name, task_id = task_id, "Cancelling task");
        }

        adapter.cancel_task(&config, task_id).await
    }

    /// Wait for a task to complete
    pub async fn wait_for_task(
        &self,
        agent_name: &str,
        task_id: &str,
        timeout_ms: u64,
    ) -> A2AResult<TaskResult> {
        let start = std::time::Instant::now();
        let timeout = std::time::Duration::from_millis(timeout_ms);

        loop {
            let result = self.get_task(agent_name, task_id).await?;

            if result.status.state.is_terminal() {
                return Ok(result);
            }

            if start.elapsed() > timeout {
                return Err(A2AError::Timeout {
                    agent_name: agent_name.to_string(),
                    timeout_ms,
                });
            }

            // Wait before polling again
            tokio::time::sleep(std::time::Duration::from_millis(1000)).await;
        }
    }

    /// Get adapter for a provider type
    async fn get_adapter(&self, provider: AgentProvider) -> Arc<dyn A2AProviderAdapter> {
        // Check cache first
        {
            let adapters = self.adapters.read().await;
            if let Some(adapter) = adapters.get(&provider) {
                return adapter.clone();
            }
        }

        // Create and cache adapter
        let adapter = get_provider_adapter(provider);
        self.adapters
            .write()
            .await
            .insert(provider, adapter.clone());
        adapter
    }

    /// Update agent health state
    pub async fn set_agent_state(&self, name: &str, state: AgentState) {
        self.registry.update_state(name, state).await;
    }

    /// Get gateway statistics
    pub async fn stats(&self) -> GatewayStats {
        let registry_stats = self.registry.stats().await;
        GatewayStats {
            registry: registry_stats,
            logging_enabled: self.enable_logging,
            cost_tracking_enabled: self.enable_cost_tracking,
        }
    }

    /// Health check for the gateway
    pub async fn health_check(&self) -> GatewayHealth {
        let stats = self.registry.stats().await;
        let healthy = stats.healthy_agents > 0 || stats.total_agents == 0;

        GatewayHealth {
            healthy,
            total_agents: stats.total_agents,
            available_agents: stats.healthy_agents + stats.degraded_agents + stats.unknown_agents,
        }
    }
}

/// Gateway statistics
#[derive(Debug, Clone, serde::Serialize)]
pub struct GatewayStats {
    /// Registry statistics
    pub registry: RegistryStats,
    /// Whether logging is enabled
    pub logging_enabled: bool,
    /// Whether cost tracking is enabled
    pub cost_tracking_enabled: bool,
}

/// Gateway health status
#[derive(Debug, Clone, serde::Serialize)]
pub struct GatewayHealth {
    /// Whether the gateway is healthy
    pub healthy: bool,
    /// Total registered agents
    pub total_agents: usize,
    /// Available agents
    pub available_agents: usize,
}

/// Thread-safe gateway handle
pub type A2AGatewayHandle = Arc<A2AGateway>;

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_gateway_creation() {
        let gateway = A2AGateway::new();
        assert_eq!(gateway.list_agents().await.len(), 0);
    }

    #[tokio::test]
    async fn test_register_agent() {
        let gateway = A2AGateway::new();

        gateway
            .register_agent(AgentConfig::new("test", "https://example.com/agent"))
            .await
            .unwrap();

        assert_eq!(gateway.list_agents().await.len(), 1);
    }

    #[tokio::test]
    async fn test_unregister_agent() {
        let gateway = A2AGateway::new();

        gateway
            .register_agent(AgentConfig::new("test", "https://example.com/agent"))
            .await
            .unwrap();

        assert!(gateway.unregister_agent("test").await);
        assert_eq!(gateway.list_agents().await.len(), 0);
    }

    #[tokio::test]
    async fn test_get_agent() {
        let gateway = A2AGateway::new();

        gateway
            .register_agent(AgentConfig::new("test", "https://example.com/agent"))
            .await
            .unwrap();

        let config = gateway.get_agent("test").await;
        assert!(config.is_ok());
        assert_eq!(config.unwrap().name, "test");

        let not_found = gateway.get_agent("nonexistent").await;
        assert!(matches!(not_found, Err(A2AError::AgentNotFound { .. })));
    }

    #[tokio::test]
    async fn test_from_config() {
        let mut config = A2AGatewayConfig::default();
        config.add_agent(AgentConfig::new("agent1", "https://example.com/agent1"));
        config.add_agent(AgentConfig::new("agent2", "https://example.com/agent2"));

        let gateway = A2AGateway::from_config(config).await.unwrap();
        assert_eq!(gateway.list_agents().await.len(), 2);
    }

    #[tokio::test]
    async fn test_gateway_stats() {
        let gateway = A2AGateway::new();

        gateway
            .register_agent(AgentConfig::new("test", "https://example.com/agent"))
            .await
            .unwrap();

        let stats = gateway.stats().await;
        assert_eq!(stats.registry.total_agents, 1);
        assert!(stats.logging_enabled);
    }

    #[tokio::test]
    async fn test_gateway_health_empty() {
        let gateway = A2AGateway::new();
        let health = gateway.health_check().await;

        // Empty gateway should be healthy
        assert!(health.healthy);
        assert_eq!(health.total_agents, 0);
    }

    #[tokio::test]
    async fn test_set_agent_state() {
        let gateway = A2AGateway::new();

        gateway
            .register_agent(AgentConfig::new("test", "https://example.com/agent"))
            .await
            .unwrap();

        gateway.set_agent_state("test", AgentState::Healthy).await;

        let health = gateway.health_check().await;
        assert!(health.healthy);
        assert_eq!(health.available_agents, 1);
    }

    #[tokio::test]
    async fn test_get_adapter_caching() {
        let gateway = A2AGateway::new();

        // First call creates adapter
        let adapter1 = gateway.get_adapter(AgentProvider::LangGraph).await;
        // Second call should return cached adapter
        let adapter2 = gateway.get_adapter(AgentProvider::LangGraph).await;

        assert_eq!(adapter1.provider_type(), adapter2.provider_type());
    }
}
