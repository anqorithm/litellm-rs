//! Log aggregation and destination management

use super::destinations::LogDestination;
use super::types::LogEntry;
use crate::utils::error::error::{GatewayError, Result};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tracing::{debug, error};

/// Log aggregator for centralized logging
pub struct LogAggregator {
    /// Configured log destinations
    destinations: Vec<LogDestination>,
    /// Log buffer
    pub(crate) buffer: Arc<RwLock<Vec<LogEntry>>>,
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
            flush_interval: Duration::from_secs(10),
        }
    }

    /// Add log destination
    pub fn add_destination(mut self, destination: LogDestination) -> Self {
        self.destinations.push(destination);
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
    async fn send_to_destination(
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
                // Send to webhook
                let client = reqwest::Client::new();
                let mut request = client.post(url).json(entries);

                for (key, value) in headers {
                    request = request.header(key, value);
                }

                request
                    .send()
                    .await
                    .map_err(|e| GatewayError::Network(e.to_string()))?;
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
            flush_interval: self.flush_interval,
        }
    }
}
