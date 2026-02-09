use super::*;
use chrono::Utc;
use uuid::Uuid;

fn create_test_alert(severity: AlertSeverity) -> Alert {
    Alert {
        id: Uuid::new_v4().to_string(),
        title: "Test Alert".to_string(),
        description: "This is a test alert".to_string(),
        severity,
        source: "test".to_string(),
        timestamp: Utc::now(),
        metadata: serde_json::json!({}),
        resolved: false,
    }
}

// ==================== SmtpConfig Tests ====================

#[test]
fn test_smtp_config_creation() {
    let config = SmtpConfig {
        server: "smtp.example.com".to_string(),
        port: 587,
        username: "user@example.com".to_string(),
        password: "secret123".to_string(),
        from_address: "alerts@example.com".to_string(),
    };

    assert_eq!(config.server, "smtp.example.com");
    assert_eq!(config.port, 587);
    assert_eq!(config.username, "user@example.com");
    assert_eq!(config.password, "secret123");
    assert_eq!(config.from_address, "alerts@example.com");
}

#[test]
fn test_smtp_config_clone() {
    let original = SmtpConfig {
        server: "smtp.test.com".to_string(),
        port: 465,
        username: "test".to_string(),
        password: "pass".to_string(),
        from_address: "from@test.com".to_string(),
    };

    let cloned = original.clone();
    assert_eq!(original.server, cloned.server);
    assert_eq!(original.port, cloned.port);
}

#[test]
fn test_smtp_config_common_ports() {
    // Test common SMTP ports
    let ports = [25, 465, 587, 2525];
    for port in ports {
        let config = SmtpConfig {
            server: "smtp.test.com".to_string(),
            port,
            username: "user".to_string(),
            password: "pass".to_string(),
            from_address: "from@test.com".to_string(),
        };
        assert_eq!(config.port, port);
    }
}

// ==================== SlackChannel Tests ====================

#[test]
fn test_slack_channel_creation() {
    let channel = SlackChannel::new(
        "https://hooks.slack.com/services/xxx".to_string(),
        Some("#alerts".to_string()),
        Some("AlertBot".to_string()),
        AlertSeverity::Warning,
    );

    assert_eq!(channel.webhook_url, "https://hooks.slack.com/services/xxx");
    assert_eq!(channel.channel, Some("#alerts".to_string()));
    assert_eq!(channel.username, Some("AlertBot".to_string()));
}

#[test]
fn test_slack_channel_minimal() {
    let channel = SlackChannel::new(
        "https://hooks.slack.com/test".to_string(),
        None,
        None,
        AlertSeverity::Info,
    );

    assert!(channel.channel.is_none());
    assert!(channel.username.is_none());
}

#[test]
fn test_slack_channel_name() {
    let channel = SlackChannel::new(
        "https://test.com".to_string(),
        None,
        None,
        AlertSeverity::Info,
    );

    assert_eq!(channel.name(), "slack");
}

#[test]
fn test_slack_channel_supports_severity_info() {
    let channel = SlackChannel::new(
        "https://test.com".to_string(),
        None,
        None,
        AlertSeverity::Info,
    );

    assert!(channel.supports_severity(AlertSeverity::Info));
    assert!(channel.supports_severity(AlertSeverity::Warning));
    assert!(channel.supports_severity(AlertSeverity::Critical));
    assert!(channel.supports_severity(AlertSeverity::Emergency));
}

#[test]
fn test_slack_channel_supports_severity_warning() {
    let channel = SlackChannel::new(
        "https://test.com".to_string(),
        None,
        None,
        AlertSeverity::Warning,
    );

    assert!(!channel.supports_severity(AlertSeverity::Info));
    assert!(channel.supports_severity(AlertSeverity::Warning));
    assert!(channel.supports_severity(AlertSeverity::Critical));
    assert!(channel.supports_severity(AlertSeverity::Emergency));
}

#[test]
fn test_slack_channel_supports_severity_critical() {
    let channel = SlackChannel::new(
        "https://test.com".to_string(),
        None,
        None,
        AlertSeverity::Critical,
    );

    assert!(!channel.supports_severity(AlertSeverity::Info));
    assert!(!channel.supports_severity(AlertSeverity::Warning));
    assert!(channel.supports_severity(AlertSeverity::Critical));
    assert!(channel.supports_severity(AlertSeverity::Emergency));
}

#[test]
fn test_slack_channel_supports_severity_emergency() {
    let channel = SlackChannel::new(
        "https://test.com".to_string(),
        None,
        None,
        AlertSeverity::Emergency,
    );

    assert!(!channel.supports_severity(AlertSeverity::Info));
    assert!(!channel.supports_severity(AlertSeverity::Warning));
    assert!(!channel.supports_severity(AlertSeverity::Critical));
    assert!(channel.supports_severity(AlertSeverity::Emergency));
}

// ==================== EmailChannel Tests ====================

#[test]
fn test_email_channel_creation() {
    let smtp_config = SmtpConfig {
        server: "smtp.example.com".to_string(),
        port: 587,
        username: "user".to_string(),
        password: "pass".to_string(),
        from_address: "alerts@example.com".to_string(),
    };

    let channel = EmailChannel::new(
        smtp_config,
        vec![
            "admin@example.com".to_string(),
            "ops@example.com".to_string(),
        ],
        AlertSeverity::Critical,
    );

    assert_eq!(channel.recipients.len(), 2);
}

#[test]
fn test_email_channel_name() {
    let smtp_config = SmtpConfig {
        server: "smtp.test.com".to_string(),
        port: 587,
        username: "user".to_string(),
        password: "pass".to_string(),
        from_address: "from@test.com".to_string(),
    };

    let channel = EmailChannel::new(smtp_config, vec![], AlertSeverity::Info);

    assert_eq!(channel.name(), "email");
}

#[test]
fn test_email_channel_supports_severity() {
    let smtp_config = SmtpConfig {
        server: "smtp.test.com".to_string(),
        port: 587,
        username: "user".to_string(),
        password: "pass".to_string(),
        from_address: "from@test.com".to_string(),
    };

    let channel = EmailChannel::new(smtp_config, vec![], AlertSeverity::Warning);

    assert!(!channel.supports_severity(AlertSeverity::Info));
    assert!(channel.supports_severity(AlertSeverity::Warning));
    assert!(channel.supports_severity(AlertSeverity::Critical));
}

#[test]
fn test_email_channel_empty_recipients() {
    let smtp_config = SmtpConfig {
        server: "smtp.test.com".to_string(),
        port: 587,
        username: "user".to_string(),
        password: "pass".to_string(),
        from_address: "from@test.com".to_string(),
    };

    let channel = EmailChannel::new(smtp_config, vec![], AlertSeverity::Info);

    assert!(channel.recipients.is_empty());
}

#[test]
fn test_email_channel_many_recipients() {
    let smtp_config = SmtpConfig {
        server: "smtp.test.com".to_string(),
        port: 587,
        username: "user".to_string(),
        password: "pass".to_string(),
        from_address: "from@test.com".to_string(),
    };

    let recipients: Vec<String> = (0..10).map(|i| format!("user{}@test.com", i)).collect();
    let channel = EmailChannel::new(smtp_config, recipients, AlertSeverity::Info);

    assert_eq!(channel.recipients.len(), 10);
}

#[test]
fn test_email_channel_build_message() {
    let smtp_config = SmtpConfig {
        server: "smtp.test.com".to_string(),
        port: 587,
        username: "user".to_string(),
        password: "pass".to_string(),
        from_address: "from@test.com".to_string(),
    };

    let channel = EmailChannel::new(
        smtp_config,
        vec!["test@example.com".to_string()],
        AlertSeverity::Info,
    );

    let alert = create_test_alert(AlertSeverity::Warning);

    let message = channel.build_message(&alert);
    assert!(message.is_ok());
}

// ==================== Alert Creation Tests ====================

#[test]
fn test_alert_creation_for_channels() {
    let alert = create_test_alert(AlertSeverity::Critical);

    assert_eq!(alert.title, "Test Alert");
    assert_eq!(alert.source, "test");
    assert!(matches!(alert.severity, AlertSeverity::Critical));
}

#[test]
fn test_alert_with_different_severities() {
    let severities = [
        AlertSeverity::Info,
        AlertSeverity::Warning,
        AlertSeverity::Critical,
        AlertSeverity::Emergency,
    ];

    for severity in severities {
        let alert = create_test_alert(severity);
        assert!(
            matches!(alert.severity, s if std::mem::discriminant(&s) == std::mem::discriminant(&severity))
        );
    }
}

// ==================== Channel Debug Tests ====================

#[test]
fn test_slack_channel_debug() {
    let channel = SlackChannel::new(
        "https://test.com".to_string(),
        Some("#test".to_string()),
        None,
        AlertSeverity::Info,
    );

    let debug_str = format!("{:?}", channel);
    assert!(debug_str.contains("SlackChannel"));
}

#[test]
fn test_email_channel_debug() {
    let smtp_config = SmtpConfig {
        server: "smtp.test.com".to_string(),
        port: 587,
        username: "user".to_string(),
        password: "pass".to_string(),
        from_address: "from@test.com".to_string(),
    };

    let channel = EmailChannel::new(smtp_config, vec![], AlertSeverity::Info);

    let debug_str = format!("{:?}", channel);
    assert!(debug_str.contains("EmailChannel"));
}

#[test]
fn test_smtp_config_debug() {
    let config = SmtpConfig {
        server: "smtp.test.com".to_string(),
        port: 587,
        username: "user".to_string(),
        password: "pass".to_string(),
        from_address: "from@test.com".to_string(),
    };

    let debug_str = format!("{:?}", config);
    assert!(debug_str.contains("SmtpConfig"));
    assert!(debug_str.contains("smtp.test.com"));
}
