//! Log aggregation and destination management

use super::destinations::LogDestination;
use super::types::LogEntry;
use crate::core::net::ProviderEndpointPolicy;
use crate::utils::error::gateway_error::{GatewayError, Result};
use crate::utils::net::http::ProviderHttpClient;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tracing::{debug, error};

pub(super) const LOG_WEBHOOK_TIMEOUT: Duration = Duration::from_secs(120);
const INVALID_LOG_WEBHOOK_URL: &str = "Log webhook URL is invalid or disallowed by outbound policy";

/// Log aggregator for centralized logging
pub struct LogAggregator {
    /// Configured log destinations
    destinations: Vec<LogDestination>,
    /// Log buffer
    pub(crate) buffer: Arc<RwLock<Vec<LogEntry>>>,
    webhook_client: Option<ProviderHttpClient>,
    /// Buffer flush interval
    flush_interval: Duration,
}

impl Default for LogAggregator {
    fn default() -> Self {
        Self::new()
    }
}

impl LogAggregator {
    /// Create a new log aggregator
    pub fn new() -> Self {
        Self {
            destinations: vec![],
            buffer: Arc::new(RwLock::new(Vec::new())),
            webhook_client: None,
            flush_interval: Duration::from_secs(10),
        }
    }

    /// Add log destination
    pub fn add_destination(mut self, destination: LogDestination) -> Result<Self> {
        let destination = if let LogDestination::Webhook { url, headers } = destination {
            let url = reqwest::Url::parse(&url)
                .map_err(|_| GatewayError::Validation(INVALID_LOG_WEBHOOK_URL.to_string()))?;
            let policy = ProviderEndpointPolicy::public_only();
            if !matches!(url.scheme(), "http" | "https")
                || policy.validate_url_without_resolution(&url).is_err()
            {
                return Err(GatewayError::Validation(
                    INVALID_LOG_WEBHOOK_URL.to_string(),
                ));
            }
            if self.webhook_client.is_none() {
                self.webhook_client = Some(
                    ProviderHttpClient::no_redirect(policy, LOG_WEBHOOK_TIMEOUT).map_err(|_| {
                        GatewayError::Network(
                            "Failed to create policy-bound log webhook client".to_string(),
                        )
                    })?,
                );
            }
            LogDestination::Webhook {
                url: url.to_string(),
                headers,
            }
        } else {
            destination
        };
        self.destinations.push(destination);
        Ok(self)
    }

    #[cfg(test)]
    pub(super) fn with_webhook_client_for_test(mut self, client: ProviderHttpClient) -> Self {
        self.webhook_client = Some(client);
        self
    }

    /// Log an entry
    pub async fn log(&self, entry: LogEntry) {
        let mut buffer = self.buffer.write().await;
        buffer.push(entry);

        // Flush if buffer is full
        if buffer.len() >= 100 {
            drop(buffer);
            self.flush_buffer().await;
        }
    }

    /// Flush log buffer
    pub async fn flush_buffer(&self) {
        let mut buffer = self.buffer.write().await;
        if buffer.is_empty() {
            return;
        }

        let entries = buffer.drain(..).collect::<Vec<_>>();
        drop(buffer);

        // Send to all destinations
        for destination in &self.destinations {
            if let Err(e) = self.send_to_destination(destination, &entries).await {
                error!("Failed to send logs to destination: {}", e);
            }
        }
    }

    /// Send logs to a specific destination
    pub(super) async fn send_to_destination(
        &self,
        destination: &LogDestination,
        entries: &[LogEntry],
    ) -> Result<()> {
        match destination {
            LogDestination::Elasticsearch {
                url: _,
                index: _,
                auth: _,
            } => {
                // Send to Elasticsearch
                debug!("Sending {} logs to Elasticsearch", entries.len());
            }
            LogDestination::Splunk {
                url: _,
                token: _,
                index: _,
            } => {
                // Send to Splunk
                debug!("Sending {} logs to Splunk", entries.len());
            }
            LogDestination::DatadogLogs {
                api_key: _,
                site: _,
            } => {
                // Send to Datadog Logs
                debug!("Sending {} logs to Datadog", entries.len());
            }
            LogDestination::Webhook { url, headers } => {
                let client = self.webhook_client.as_ref().ok_or_else(|| {
                    GatewayError::Internal("Log webhook client is not configured".to_string())
                })?;
                let mut request = client
                    .post(url)
                    .map_err(|_| {
                        GatewayError::Network(
                            "Log webhook rejected by outbound endpoint policy".to_string(),
                        )
                    })?
                    .json(entries);

                for (key, value) in headers {
                    request = request.header(key, value);
                }

                let response = request.send().await.map_err(|error| {
                    let message = if ProviderHttpClient::request_error_is_endpoint_policy(&error) {
                        "Log webhook rejected by outbound endpoint policy"
                    } else {
                        "Failed to send logs to webhook"
                    };
                    GatewayError::Network(message.to_string())
                })?;
                if !response.status().is_success() {
                    return Err(GatewayError::Network(format!(
                        "Log webhook returned status: {}",
                        response.status()
                    )));
                }
            }
            _ => {
                // Other destinations would be implemented similarly
                debug!("Sending {} logs to destination", entries.len());
            }
        }
        Ok(())
    }

    /// Start background flushing
    pub async fn start_background_flush(&self) {
        let aggregator = self.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(aggregator.flush_interval);
            loop {
                interval.tick().await;
                aggregator.flush_buffer().await;
            }
        });
    }
}

impl Clone for LogAggregator {
    fn clone(&self) -> Self {
        Self {
            destinations: self.destinations.clone(),
            buffer: self.buffer.clone(),
            webhook_client: self.webhook_client.clone(),
            flush_interval: self.flush_interval,
        }
    }
}
