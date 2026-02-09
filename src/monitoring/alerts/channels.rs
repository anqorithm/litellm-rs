//! Notification channel implementations

use crate::monitoring::types::{Alert, AlertSeverity};
use crate::utils::error::error::{GatewayError, Result};
use base64::Engine as _;
use std::io::Write;
use std::process::{Command, Stdio};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpStream;

/// Notification channel trait
#[async_trait::async_trait]
pub trait NotificationChannel: Send + Sync + std::fmt::Debug {
    /// Send a notification
    async fn send(&self, alert: &Alert) -> Result<()>;

    /// Get channel name
    fn name(&self) -> &str;

    /// Check if channel supports severity level
    fn supports_severity(&self, severity: AlertSeverity) -> bool;
}

/// Slack notification channel
#[derive(Debug)]
pub struct SlackChannel {
    webhook_url: String,
    channel: Option<String>,
    username: Option<String>,
    min_severity: AlertSeverity,
}

/// Email notification channel
#[derive(Debug)]
pub struct EmailChannel {
    smtp_config: SmtpConfig,
    recipients: Vec<String>,
    min_severity: AlertSeverity,
}

/// SMTP configuration
#[derive(Debug, Clone)]
pub struct SmtpConfig {
    pub server: String,
    pub port: u16,
    pub username: String,
    pub password: String,
    pub from_address: String,
}

impl SlackChannel {
    /// Create a new Slack notification channel
    pub fn new(
        webhook_url: String,
        channel: Option<String>,
        username: Option<String>,
        min_severity: AlertSeverity,
    ) -> Self {
        Self {
            webhook_url,
            channel,
            username,
            min_severity,
        }
    }
}

#[async_trait::async_trait]
impl NotificationChannel for SlackChannel {
    async fn send(&self, alert: &Alert) -> Result<()> {
        let color = match alert.severity {
            AlertSeverity::Info => "#36a64f",      // Green
            AlertSeverity::Warning => "#ff9500",   // Orange
            AlertSeverity::Critical => "#ff0000",  // Red
            AlertSeverity::Emergency => "#8b0000", // Dark Red
        };

        let payload = serde_json::json!({
            "username": self.username.as_deref().unwrap_or("Gateway Alert"),
            "channel": self.channel,
            "attachments": [{
                "color": color,
                "title": alert.title,
                "text": alert.description,
                "fields": [
                    {
                        "title": "Severity",
                        "value": format!("{:?}", alert.severity),
                        "short": true
                    },
                    {
                        "title": "Source",
                        "value": alert.source,
                        "short": true
                    },
                    {
                        "title": "Time",
                        "value": alert.timestamp.format("%Y-%m-%d %H:%M:%S UTC").to_string(),
                        "short": true
                    }
                ],
                "footer": "Gateway Monitoring",
                "ts": alert.timestamp.timestamp()
            }]
        });

        let client = reqwest::Client::new();
        let response = client
            .post(&self.webhook_url)
            .json(&payload)
            .send()
            .await
            .map_err(|e| {
                GatewayError::Alert(format!("Failed to send Slack notification: {}", e))
            })?;

        if !response.status().is_success() {
            return Err(GatewayError::Alert(format!(
                "Slack webhook returned status: {}",
                response.status()
            )));
        }

        Ok(())
    }

    fn name(&self) -> &str {
        "slack"
    }

    fn supports_severity(&self, severity: AlertSeverity) -> bool {
        severity as u8 >= self.min_severity as u8
    }
}

impl EmailChannel {
    /// Create a new email notification channel
    pub fn new(
        smtp_config: SmtpConfig,
        recipients: Vec<String>,
        min_severity: AlertSeverity,
    ) -> Self {
        Self {
            smtp_config,
            recipients,
            min_severity,
        }
    }
}

#[async_trait::async_trait]
impl NotificationChannel for EmailChannel {
    async fn send(&self, _alert: &Alert) -> Result<()> {
        let alert = _alert;

        if self.recipients.is_empty() {
            return Err(GatewayError::Alert(
                "No email recipients configured".to_string(),
            ));
        }

        let email = self.build_message(alert)?;
        if let Err(err) = self.send_via_smtp(&email).await {
            // Fallback to sendmail if SMTP fails
            self.send_via_sendmail(&email)
                .map_err(|e| GatewayError::Alert(format!("Email send failed: {}; {}", err, e)))?;
        }

        Ok(())
    }

    fn name(&self) -> &str {
        "email"
    }

    fn supports_severity(&self, severity: AlertSeverity) -> bool {
        severity as u8 >= self.min_severity as u8
    }
}

impl EmailChannel {
    fn build_message(&self, alert: &Alert) -> Result<String> {
        if self.smtp_config.from_address.trim().is_empty() {
            return Err(GatewayError::Alert(
                "SMTP from_address is required".to_string(),
            ));
        }

        let subject = format!("[{:?}] {}", alert.severity, alert.title);
        let metadata = serde_json::to_string_pretty(&alert.metadata).unwrap_or_default();
        let body = format!(
            "Alert: {title}\nSeverity: {severity:?}\nSource: {source}\nTime: {time}\n\n{description}\n\nMetadata:\n{metadata}\n",
            title = alert.title,
            severity = alert.severity,
            source = alert.source,
            time = alert.timestamp.format("%Y-%m-%d %H:%M:%S UTC"),
            description = alert.description,
            metadata = metadata
        );

        let to = self.recipients.join(", ");
        let date = alert.timestamp.to_rfc2822();

        Ok(format!(
            "From: {from}\r\nTo: {to}\r\nSubject: {subject}\r\nDate: {date}\r\nMIME-Version: 1.0\r\nContent-Type: text/plain; charset=\"utf-8\"\r\n\r\n{body}",
            from = self.smtp_config.from_address,
            to = to,
            subject = subject,
            date = date,
            body = body
        ))
    }

    async fn send_via_smtp(&self, message: &str) -> Result<()> {
        let address = format!("{}:{}", self.smtp_config.server, self.smtp_config.port);
        let stream = TcpStream::connect(address)
            .await
            .map_err(|e| GatewayError::Alert(format!("SMTP connect failed: {}", e)))?;

        let (read_half, mut write_half) = stream.into_split();
        let mut reader = BufReader::new(read_half);

        read_smtp_response(&mut reader, &[220]).await?;
        write_smtp_command(&mut write_half, "EHLO gateway").await?;
        read_smtp_response(&mut reader, &[250]).await?;

        if !self.smtp_config.username.is_empty() || !self.smtp_config.password.is_empty() {
            write_smtp_command(&mut write_half, "AUTH LOGIN").await?;
            read_smtp_response(&mut reader, &[334]).await?;

            let username = base64::engine::general_purpose::STANDARD
                .encode(self.smtp_config.username.as_bytes());
            write_smtp_command(&mut write_half, &username).await?;
            read_smtp_response(&mut reader, &[334]).await?;

            let password = base64::engine::general_purpose::STANDARD
                .encode(self.smtp_config.password.as_bytes());
            write_smtp_command(&mut write_half, &password).await?;
            read_smtp_response(&mut reader, &[235]).await?;
        }

        write_smtp_command(
            &mut write_half,
            &format!("MAIL FROM:<{}>", self.smtp_config.from_address),
        )
        .await?;
        read_smtp_response(&mut reader, &[250]).await?;

        for recipient in &self.recipients {
            write_smtp_command(&mut write_half, &format!("RCPT TO:<{}>", recipient)).await?;
            read_smtp_response(&mut reader, &[250, 251]).await?;
        }

        write_smtp_command(&mut write_half, "DATA").await?;
        read_smtp_response(&mut reader, &[354]).await?;

        write_half
            .write_all(message.as_bytes())
            .await
            .map_err(|e| GatewayError::Alert(format!("SMTP write failed: {}", e)))?;
        write_half
            .write_all(b"\r\n.\r\n")
            .await
            .map_err(|e| GatewayError::Alert(format!("SMTP write failed: {}", e)))?;
        write_half
            .flush()
            .await
            .map_err(|e| GatewayError::Alert(format!("SMTP flush failed: {}", e)))?;

        read_smtp_response(&mut reader, &[250]).await?;
        write_smtp_command(&mut write_half, "QUIT").await?;

        Ok(())
    }

    fn send_via_sendmail(&self, message: &str) -> Result<()> {
        let mut child = Command::new("sendmail")
            .arg("-t")
            .arg("-i")
            .stdin(Stdio::piped())
            .spawn()
            .map_err(|e| GatewayError::Alert(format!("sendmail spawn failed: {}", e)))?;

        if let Some(stdin) = child.stdin.as_mut() {
            stdin
                .write_all(message.as_bytes())
                .map_err(|e| GatewayError::Alert(format!("sendmail write failed: {}", e)))?;
        }

        let status = child
            .wait()
            .map_err(|e| GatewayError::Alert(format!("sendmail wait failed: {}", e)))?;

        if !status.success() {
            return Err(GatewayError::Alert(format!(
                "sendmail exited with status {}",
                status
            )));
        }

        Ok(())
    }
}

async fn read_smtp_response(
    reader: &mut BufReader<tokio::net::tcp::OwnedReadHalf>,
    expected: &[u16],
) -> Result<()> {
    let mut line = String::new();
    let mut code: u16 = 0;
    loop {
        line.clear();
        let bytes = reader
            .read_line(&mut line)
            .await
            .map_err(|e| GatewayError::Alert(format!("SMTP read failed: {}", e)))?;
        if bytes == 0 {
            return Err(GatewayError::Alert("SMTP connection closed".to_string()));
        }
        if line.len() >= 3 {
            code = line[0..3].parse().unwrap_or(0);
        }
        if line.as_bytes().get(3) == Some(&b' ') {
            break;
        }
    }

    if !expected.contains(&code) {
        return Err(GatewayError::Alert(format!(
            "Unexpected SMTP response: {}",
            line.trim_end()
        )));
    }

    Ok(())
}

async fn write_smtp_command(
    writer: &mut tokio::net::tcp::OwnedWriteHalf,
    command: &str,
) -> Result<()> {
    writer
        .write_all(command.as_bytes())
        .await
        .map_err(|e| GatewayError::Alert(format!("SMTP write failed: {}", e)))?;
    writer
        .write_all(b"\r\n")
        .await
        .map_err(|e| GatewayError::Alert(format!("SMTP write failed: {}", e)))?;
    writer
        .flush()
        .await
        .map_err(|e| GatewayError::Alert(format!("SMTP flush failed: {}", e)))?;
    Ok(())
}

#[cfg(test)]
mod channels_tests;
