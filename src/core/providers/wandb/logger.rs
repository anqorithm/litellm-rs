//! Weights & Biases (W&B) Logger
//!
//! Provides LLM call logging functionality to W&B for experiment tracking,
//! prompt monitoring, and cost analysis.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, error, warn};

use super::config::{PROVIDER_NAME, WandbConfig};
use crate::core::providers::base_provider::{BaseHttpClient, BaseProviderConfig, HeaderBuilder};
use crate::core::providers::unified_provider::ProviderError;
use crate::core::traits::provider::ProviderConfig;

/// LLM call log entry for W&B tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LLMCallLog {
    /// Unique identifier for this call
    pub call_id: String,

    /// Timestamp of the call
    pub timestamp: DateTime<Utc>,

    /// Provider name (e.g., "openai", "anthropic")
    pub provider: String,

    /// Model used (e.g., "gpt-4", "claude-3-opus")
    pub model: String,

    /// Request type (e.g., "chat_completion", "embedding")
    pub request_type: String,

    /// Input prompt/messages (optional, may be disabled for privacy)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub input: Option<Value>,

    /// Output response (optional, may be disabled for privacy)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output: Option<Value>,

    /// Input token count
    #[serde(skip_serializing_if = "Option::is_none")]
    pub input_tokens: Option<u32>,

    /// Output token count
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output_tokens: Option<u32>,

    /// Total token count
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_tokens: Option<u32>,

    /// Estimated cost in USD
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cost_usd: Option<f64>,

    /// Latency in milliseconds
    #[serde(skip_serializing_if = "Option::is_none")]
    pub latency_ms: Option<u64>,

    /// Whether the call was successful
    pub success: bool,

    /// Error message if the call failed
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,

    /// Additional metadata
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    pub metadata: HashMap<String, Value>,
}

impl LLMCallLog {
    /// Create a new LLM call log entry
    pub fn new(provider: impl Into<String>, model: impl Into<String>) -> Self {
        Self {
            call_id: uuid::Uuid::new_v4().to_string(),
            timestamp: Utc::now(),
            provider: provider.into(),
            model: model.into(),
            request_type: "chat_completion".to_string(),
            input: None,
            output: None,
            input_tokens: None,
            output_tokens: None,
            total_tokens: None,
            cost_usd: None,
            latency_ms: None,
            success: true,
            error: None,
            metadata: HashMap::new(),
        }
    }

    /// Set the request type
    pub fn with_request_type(mut self, request_type: impl Into<String>) -> Self {
        self.request_type = request_type.into();
        self
    }

    /// Set input prompt/messages
    pub fn with_input(mut self, input: Value) -> Self {
        self.input = Some(input);
        self
    }

    /// Set output response
    pub fn with_output(mut self, output: Value) -> Self {
        self.output = Some(output);
        self
    }

    /// Set token usage
    pub fn with_token_usage(
        mut self,
        input_tokens: u32,
        output_tokens: u32,
        total_tokens: u32,
    ) -> Self {
        self.input_tokens = Some(input_tokens);
        self.output_tokens = Some(output_tokens);
        self.total_tokens = Some(total_tokens);
        self
    }

    /// Set cost
    pub fn with_cost(mut self, cost_usd: f64) -> Self {
        self.cost_usd = Some(cost_usd);
        self
    }

    /// Set latency
    pub fn with_latency(mut self, latency_ms: u64) -> Self {
        self.latency_ms = Some(latency_ms);
        self
    }

    /// Mark as failed with error message
    pub fn with_error(mut self, error: impl Into<String>) -> Self {
        self.success = false;
        self.error = Some(error.into());
        self
    }

    /// Add metadata entry
    pub fn with_metadata(mut self, key: impl Into<String>, value: Value) -> Self {
        self.metadata.insert(key.into(), value);
        self
    }
}

/// W&B Run information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WandbRun {
    /// Run ID
    pub id: String,

    /// Run name
    pub name: String,

    /// Project name
    pub project: String,

    /// Entity (team/username)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub entity: Option<String>,

    /// Run state
    pub state: RunState,

    /// Run URL
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,

    /// Creation timestamp
    pub created_at: DateTime<Utc>,
}

/// Run state
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum RunState {
    Running,
    Finished,
    Failed,
    Crashed,
}

/// Summary metrics tracked during a run
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RunSummary {
    /// Total number of LLM calls
    pub total_calls: u64,

    /// Successful calls
    pub successful_calls: u64,

    /// Failed calls
    pub failed_calls: u64,

    /// Total input tokens
    pub total_input_tokens: u64,

    /// Total output tokens
    pub total_output_tokens: u64,

    /// Total cost in USD
    pub total_cost_usd: f64,

    /// Average latency in milliseconds
    pub avg_latency_ms: f64,

    /// Calls by provider
    pub calls_by_provider: HashMap<String, u64>,

    /// Calls by model
    pub calls_by_model: HashMap<String, u64>,

    /// Cost by model
    pub cost_by_model: HashMap<String, f64>,
}

impl RunSummary {
    /// Update summary with a new call log
    pub fn update(&mut self, log: &LLMCallLog) {
        self.total_calls += 1;

        if log.success {
            self.successful_calls += 1;
        } else {
            self.failed_calls += 1;
        }

        if let Some(input_tokens) = log.input_tokens {
            self.total_input_tokens += input_tokens as u64;
        }

        if let Some(output_tokens) = log.output_tokens {
            self.total_output_tokens += output_tokens as u64;
        }

        if let Some(cost) = log.cost_usd {
            self.total_cost_usd += cost;
            *self.cost_by_model.entry(log.model.clone()).or_insert(0.0) += cost;
        }

        if let Some(latency) = log.latency_ms {
            // Update running average
            let n = self.total_calls as f64;
            self.avg_latency_ms = ((self.avg_latency_ms * (n - 1.0)) + latency as f64) / n;
        }

        *self
            .calls_by_provider
            .entry(log.provider.clone())
            .or_insert(0) += 1;
        *self.calls_by_model.entry(log.model.clone()).or_insert(0) += 1;
    }
}

/// W&B Logger for tracking LLM calls
///
/// This is the main struct for logging LLM calls to Weights & Biases.
/// It supports batched logging for performance optimization.
#[derive(Debug)]
pub struct WandbLogger {
    /// Configuration
    config: WandbConfig,

    /// HTTP client for API calls
    client: BaseHttpClient,

    /// Current run information
    run: Arc<RwLock<Option<WandbRun>>>,

    /// Buffered logs for batch sending
    log_buffer: Arc<RwLock<Vec<LLMCallLog>>>,

    /// Run summary metrics
    summary: Arc<RwLock<RunSummary>>,
}

impl WandbLogger {
    /// Create a new W&B logger
    pub fn new(config: WandbConfig) -> Result<Self, ProviderError> {
        config
            .validate()
            .map_err(|e| ProviderError::configuration(PROVIDER_NAME, e))?;

        let base_config = BaseProviderConfig {
            api_key: config.get_effective_api_key(),
            api_base: Some(config.api_base.clone()),
            timeout: Some(config.timeout_seconds),
            max_retries: Some(config.max_retries),
            headers: None,
            organization: None,
            api_version: None,
        };

        let client = BaseHttpClient::new(base_config)?;

        Ok(Self {
            config,
            client,
            run: Arc::new(RwLock::new(None)),
            log_buffer: Arc::new(RwLock::new(Vec::new())),
            summary: Arc::new(RwLock::new(RunSummary::default())),
        })
    }

    /// Create logger from environment variables
    pub fn from_env() -> Result<Self, ProviderError> {
        let config = WandbConfig::from_env()?;
        Self::new(config)
    }

    /// Initialize a new W&B run
    pub async fn init_run(&self) -> Result<WandbRun, ProviderError> {
        if !self.config.enabled {
            return Err(ProviderError::configuration(
                PROVIDER_NAME,
                "W&B logging is disabled",
            ));
        }

        let project = self.config.get_effective_project().unwrap_or_else(|| {
            warn!("W&B project not set, using default 'litellm-logs'");
            "litellm-logs".to_string()
        });

        let run_name = self
            .config
            .run_name
            .clone()
            .unwrap_or_else(|| format!("run-{}", &uuid::Uuid::new_v4().to_string()[..8]));

        let run = WandbRun {
            id: uuid::Uuid::new_v4().to_string(),
            name: run_name,
            project,
            entity: self.config.get_effective_entity(),
            state: RunState::Running,
            url: None,
            created_at: Utc::now(),
        };

        // In a real implementation, this would make an API call to W&B
        // to create the run. For now, we just store it locally.
        debug!(
            "Initialized W&B run: {} in project {}",
            run.name, run.project
        );

        let mut run_lock = self.run.write().await;
        *run_lock = Some(run.clone());

        Ok(run)
    }

    /// Log an LLM call
    pub async fn log(&self, mut log_entry: LLMCallLog) -> Result<(), ProviderError> {
        if !self.config.enabled {
            return Ok(());
        }

        // Apply privacy filters
        if !self.config.log_prompts {
            log_entry.input = None;
        }

        if !self.config.log_responses {
            log_entry.output = None;
        }

        if !self.config.log_token_usage {
            log_entry.input_tokens = None;
            log_entry.output_tokens = None;
            log_entry.total_tokens = None;
        }

        if !self.config.log_costs {
            log_entry.cost_usd = None;
        }

        if !self.config.log_latency {
            log_entry.latency_ms = None;
        }

        // Update summary
        {
            let mut summary = self.summary.write().await;
            summary.update(&log_entry);
        }

        // Add to buffer
        let should_flush = {
            let mut buffer = self.log_buffer.write().await;
            buffer.push(log_entry);
            buffer.len() >= self.config.batch_size
        };

        // Flush if buffer is full
        if should_flush {
            self.flush().await?;
        }

        Ok(())
    }

    /// Log a successful LLM call with all details
    #[allow(clippy::too_many_arguments)]
    pub async fn log_success(
        &self,
        provider: &str,
        model: &str,
        input: Option<Value>,
        output: Option<Value>,
        input_tokens: u32,
        output_tokens: u32,
        cost_usd: Option<f64>,
        latency_ms: u64,
    ) -> Result<(), ProviderError> {
        let mut log = LLMCallLog::new(provider, model)
            .with_token_usage(input_tokens, output_tokens, input_tokens + output_tokens)
            .with_latency(latency_ms);

        if let Some(input_val) = input {
            log = log.with_input(input_val);
        }

        if let Some(output_val) = output {
            log = log.with_output(output_val);
        }

        if let Some(cost) = cost_usd {
            log = log.with_cost(cost);
        }

        self.log(log).await
    }

    /// Log a failed LLM call
    pub async fn log_failure(
        &self,
        provider: &str,
        model: &str,
        error: &str,
        latency_ms: Option<u64>,
    ) -> Result<(), ProviderError> {
        let mut log = LLMCallLog::new(provider, model).with_error(error);

        if let Some(latency) = latency_ms {
            log = log.with_latency(latency);
        }

        self.log(log).await
    }

    /// Flush buffered logs to W&B
    pub async fn flush(&self) -> Result<(), ProviderError> {
        if !self.config.enabled {
            return Ok(());
        }

        let logs_to_send: Vec<LLMCallLog> = {
            let mut buffer = self.log_buffer.write().await;
            std::mem::take(&mut *buffer)
        };

        if logs_to_send.is_empty() {
            return Ok(());
        }

        debug!("Flushing {} logs to W&B", logs_to_send.len());

        // In a real implementation, this would send logs to W&B API
        // For now, we just log them
        for log in &logs_to_send {
            debug!(
                "W&B Log: {} {} {} tokens={:?} cost={:?} latency={:?}ms success={}",
                log.provider,
                log.model,
                log.request_type,
                log.total_tokens,
                log.cost_usd,
                log.latency_ms,
                log.success
            );
        }

        // Attempt to send to W&B API
        if let Err(e) = self.send_logs_to_wandb(&logs_to_send).await {
            error!("Failed to send logs to W&B: {}", e);
            // Re-add logs to buffer for retry
            let mut buffer = self.log_buffer.write().await;
            buffer.extend(logs_to_send);
            return Err(e);
        }

        Ok(())
    }

    /// Send logs to W&B API
    async fn send_logs_to_wandb(&self, logs: &[LLMCallLog]) -> Result<(), ProviderError> {
        let api_key = self
            .config
            .get_effective_api_key()
            .ok_or_else(|| ProviderError::authentication(PROVIDER_NAME, "API key not found"))?;

        let run = self.run.read().await;
        let run_info = run
            .as_ref()
            .ok_or_else(|| ProviderError::configuration(PROVIDER_NAME, "Run not initialized"))?;

        // Build the request payload
        // W&B uses a specific format for logging metrics
        let payload = serde_json::json!({
            "run_id": run_info.id,
            "project": run_info.project,
            "entity": run_info.entity,
            "logs": logs,
        });

        let url = format!("{}/api/v1/runs/{}/logs", self.config.api_base, run_info.id);

        let headers = HeaderBuilder::new()
            .with_bearer_token(&api_key)
            .with_content_type("application/json")
            .build_reqwest()
            .map_err(|e| ProviderError::invalid_request(PROVIDER_NAME, e.to_string()))?;

        let response = self
            .client
            .inner()
            .post(&url)
            .headers(headers)
            .json(&payload)
            .send()
            .await
            .map_err(|e| ProviderError::network(PROVIDER_NAME, e.to_string()))?;

        if !response.status().is_success() {
            let status = response.status().as_u16();
            let body = response.text().await.unwrap_or_default();
            return Err(ProviderError::api_error(PROVIDER_NAME, status, body));
        }

        Ok(())
    }

    /// Get current run summary
    pub async fn get_summary(&self) -> RunSummary {
        self.summary.read().await.clone()
    }

    /// Get current run info
    pub async fn get_run(&self) -> Option<WandbRun> {
        self.run.read().await.clone()
    }

    /// Finish the current run
    pub async fn finish(&self) -> Result<(), ProviderError> {
        // Flush remaining logs
        self.flush().await?;

        // Update run state
        let mut run_lock = self.run.write().await;
        if let Some(ref mut run) = *run_lock {
            run.state = RunState::Finished;
            debug!("Finished W&B run: {}", run.name);
        }

        Ok(())
    }

    /// Check if logging is enabled
    pub fn is_enabled(&self) -> bool {
        self.config.enabled
    }

    /// Get the configuration
    pub fn config(&self) -> &WandbConfig {
        &self.config
    }
}

/// Create a log entry from a chat request/response
pub fn create_chat_log(
    provider: &str,
    model: &str,
    request: &crate::core::types::chat::ChatRequest,
    response: Option<&crate::core::types::responses::ChatResponse>,
    latency_ms: u64,
    error: Option<&str>,
) -> LLMCallLog {
    let mut log = LLMCallLog::new(provider, model)
        .with_request_type("chat_completion")
        .with_latency(latency_ms);

    // Add input (messages)
    if let Ok(input_json) = serde_json::to_value(&request.messages) {
        log = log.with_input(input_json);
    }

    // Add output if successful
    if let Some(resp) = response {
        if let Ok(output_json) = serde_json::to_value(&resp.choices) {
            log = log.with_output(output_json);
        }

        // Add token usage if available
        if let Some(usage) = &resp.usage {
            log = log.with_token_usage(
                usage.prompt_tokens,
                usage.completion_tokens,
                usage.total_tokens,
            );
        }
    }

    // Add error if present
    if let Some(err) = error {
        log = log.with_error(err);
    }

    log
}


#[cfg(test)]
#[path = "logger_tests.rs"]
mod tests;
