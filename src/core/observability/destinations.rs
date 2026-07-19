//! Log destinations and trace exporters

use std::collections::HashMap;
use std::time::Duration;

/// Log destinations
#[derive(Clone)]
pub enum LogDestination {
    /// Elasticsearch
    Elasticsearch {
        url: String,
        index: String,
        auth: Option<String>,
    },
    /// Splunk
    Splunk {
        url: String,
        token: String,
        index: String,
    },
    /// AWS CloudWatch
    CloudWatch {
        region: String,
        log_group: String,
        log_stream: String,
    },
    /// Google Cloud Logging
    GCPLogging {
        project_id: String,
        log_name: String,
    },
    /// Datadog Logs
    DatadogLogs { api_key: String, site: String },
    /// Custom webhook
    Webhook {
        url: String,
        headers: HashMap<String, String>,
    },
}

impl std::fmt::Debug for LogDestination {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("LogDestination { redacted }")
    }
}

/// Alert channels
#[derive(Debug, Clone)]
pub enum AlertChannel {
    /// Slack webhook
    Slack {
        webhook_url: String,
        channel: String,
        username: String,
    },
    /// Email SMTP
    Email {
        smtp_host: String,
        smtp_port: u16,
        username: String,
        password: String,
        from: String,
        to: Vec<String>,
    },
    /// PagerDuty
    PagerDuty {
        integration_key: String,
        severity: String,
    },
    /// Discord webhook
    Discord { webhook_url: String },
    /// Microsoft Teams
    Teams { webhook_url: String },
    /// Custom webhook
    Webhook {
        url: String,
        headers: HashMap<String, String>,
    },
}

/// Alert rules
#[derive(Debug, Clone)]
pub struct AlertRule {
    /// Rule ID
    pub id: String,
    /// Rule name
    pub name: String,
    /// Metric to monitor
    pub metric: String,
    /// Condition
    pub condition: super::types::AlertCondition,
    /// Threshold value
    pub threshold: f64,
    /// Evaluation window
    pub window: Duration,
    /// Alert severity
    pub severity: super::types::AlertSeverity,
    /// Channels to notify
    pub channels: Vec<String>,
    /// Whether rule is enabled
    pub enabled: bool,
}

/// Trace exporters
#[derive(Debug, Clone)]
pub enum TraceExporter {
    /// Jaeger
    Jaeger {
        endpoint: String,
        service_name: String,
    },
    /// Zipkin
    Zipkin {
        endpoint: String,
        service_name: String,
    },
    /// OpenTelemetry
    OpenTelemetry {
        endpoint: String,
        headers: HashMap<String, String>,
    },
    /// DataDog APM
    DataDogAPM {
        api_key: String,
        service_name: String,
    },
}
