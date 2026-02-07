//! Team settings and configuration

use super::entity::TeamVisibility;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Team settings
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TeamSettings {
    /// Default user role for new members
    pub default_member_role: Option<String>,
    /// Require approval for new members
    pub require_approval: bool,
    /// Allow members to invite others
    pub allow_member_invites: bool,
    /// Team visibility
    pub visibility: TeamVisibility,
    /// API access settings
    pub api_access: ApiAccessSettings,
    /// Notification settings
    pub notifications: TeamNotificationSettings,
    /// Security settings
    pub security: TeamSecuritySettings,
}

/// API access settings
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ApiAccessSettings {
    /// Enable API access
    pub enabled: bool,
    /// Allowed IP addresses
    pub allowed_ips: Vec<String>,
    /// Allowed domains
    pub allowed_domains: Vec<String>,
    /// Require API key authentication
    pub require_api_key: bool,
    /// Default API settings
    pub default_settings: HashMap<String, serde_json::Value>,
}

/// Team notification settings
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TeamNotificationSettings {
    /// Slack webhook URL
    pub slack_webhook: Option<String>,
    /// Email notifications
    pub email_notifications: bool,
    /// Webhook notifications
    pub webhook_notifications: bool,
    /// Notification channels
    pub channels: Vec<NotificationChannel>,
}

/// Notification channel
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationChannel {
    /// Channel name
    pub name: String,
    /// Channel type
    pub channel_type: ChannelType,
    /// Channel configuration
    pub config: HashMap<String, serde_json::Value>,
    /// Enabled
    pub enabled: bool,
}

/// Channel type
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ChannelType {
    /// Email channel
    Email,
    /// Slack channel
    Slack,
    /// Webhook channel
    Webhook,
    /// Microsoft Teams channel
    Teams,
    /// Discord channel
    Discord,
}

/// Team security settings
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TeamSecuritySettings {
    /// Require two-factor authentication
    pub require_2fa: bool,
    /// Password policy
    pub password_policy: PasswordPolicy,
    /// Session timeout in minutes
    pub session_timeout: Option<u32>,
    /// IP whitelist
    pub ip_whitelist: Vec<String>,
    /// Audit logging enabled
    pub audit_logging: bool,
}

/// Password policy
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PasswordPolicy {
    /// Minimum length
    pub min_length: u32,
    /// Require uppercase
    pub require_uppercase: bool,
    /// Require lowercase
    pub require_lowercase: bool,
    /// Require numbers
    pub require_numbers: bool,
    /// Require special characters
    pub require_special: bool,
    /// Password expiry in days
    pub expiry_days: Option<u32>,
}

#[cfg(test)]
mod tests {
    use super::*;

    // ==================== TeamSettings Tests ====================

    #[test]
    fn test_team_settings_default() {
        let settings = TeamSettings::default();
        assert!(settings.default_member_role.is_none());
        assert!(!settings.require_approval);
        assert!(!settings.allow_member_invites);
    }

    #[test]
    fn test_team_settings_with_values() {
        let settings = TeamSettings {
            default_member_role: Some("member".to_string()),
            require_approval: true,
            allow_member_invites: true,
            visibility: TeamVisibility::Public,
            api_access: ApiAccessSettings::default(),
            notifications: TeamNotificationSettings::default(),
            security: TeamSecuritySettings::default(),
        };

        assert_eq!(settings.default_member_role, Some("member".to_string()));
        assert!(settings.require_approval);
        assert!(settings.allow_member_invites);
    }

    #[test]
    fn test_team_settings_serialize() {
        let settings = TeamSettings::default();
        let json = serde_json::to_string(&settings).unwrap();
        assert!(json.contains("require_approval"));
        assert!(json.contains("visibility"));
    }

    #[test]
    fn test_team_settings_clone() {
        let original = TeamSettings {
            default_member_role: Some("admin".to_string()),
            ..Default::default()
        };
        let cloned = original.clone();
        assert_eq!(original.default_member_role, cloned.default_member_role);
    }

    // ==================== ApiAccessSettings Tests ====================

    #[test]
    fn test_api_access_settings_default() {
        let settings = ApiAccessSettings::default();
        assert!(!settings.enabled);
        assert!(settings.allowed_ips.is_empty());
        assert!(settings.allowed_domains.is_empty());
        assert!(!settings.require_api_key);
    }

    #[test]
    fn test_api_access_settings_enabled() {
        let settings = ApiAccessSettings {
            enabled: true,
            allowed_ips: vec!["192.168.1.0/24".to_string()],
            allowed_domains: vec!["example.com".to_string()],
            require_api_key: true,
            default_settings: std::collections::HashMap::new(),
        };

        assert!(settings.enabled);
        assert_eq!(settings.allowed_ips.len(), 1);
        assert!(settings.require_api_key);
    }

    #[test]
    fn test_api_access_settings_serialize() {
        let settings = ApiAccessSettings {
            enabled: true,
            allowed_ips: vec!["10.0.0.0/8".to_string()],
            allowed_domains: vec![],
            require_api_key: false,
            default_settings: std::collections::HashMap::new(),
        };

        let json = serde_json::to_string(&settings).unwrap();
        assert!(json.contains("10.0.0.0/8"));
    }

    // ==================== TeamNotificationSettings Tests ====================

    #[test]
    fn test_team_notification_settings_default() {
        let settings = TeamNotificationSettings::default();
        assert!(settings.slack_webhook.is_none());
        assert!(!settings.email_notifications);
        assert!(!settings.webhook_notifications);
        assert!(settings.channels.is_empty());
    }

    #[test]
    fn test_team_notification_settings_with_slack() {
        let settings = TeamNotificationSettings {
            slack_webhook: Some("https://hooks.slack.com/test".to_string()),
            email_notifications: true,
            webhook_notifications: false,
            channels: vec![],
        };

        assert!(settings.slack_webhook.is_some());
        assert!(settings.email_notifications);
    }

    #[test]
    fn test_team_notification_settings_serialize() {
        let settings = TeamNotificationSettings {
            slack_webhook: Some("https://test.slack.com".to_string()),
            email_notifications: true,
            webhook_notifications: true,
            channels: vec![],
        };

        let json = serde_json::to_string(&settings).unwrap();
        assert!(json.contains("slack_webhook"));
    }

    // ==================== NotificationChannel Tests ====================

    #[test]
    fn test_notification_channel_creation() {
        let channel = NotificationChannel {
            name: "alerts".to_string(),
            channel_type: ChannelType::Slack,
            config: std::collections::HashMap::new(),
            enabled: true,
        };

        assert_eq!(channel.name, "alerts");
        assert!(matches!(channel.channel_type, ChannelType::Slack));
        assert!(channel.enabled);
    }

    #[test]
    fn test_notification_channel_serialize() {
        let channel = NotificationChannel {
            name: "email-alerts".to_string(),
            channel_type: ChannelType::Email,
            config: std::collections::HashMap::new(),
            enabled: false,
        };

        let json = serde_json::to_string(&channel).unwrap();
        assert!(json.contains("email-alerts"));
        assert!(json.contains("\"enabled\":false"));
    }

    // ==================== ChannelType Tests ====================

    #[test]
    fn test_channel_type_email() {
        let t = ChannelType::Email;
        let json = serde_json::to_string(&t).unwrap();
        assert_eq!(json, "\"email\"");
    }

    #[test]
    fn test_channel_type_slack() {
        let t = ChannelType::Slack;
        let json = serde_json::to_string(&t).unwrap();
        assert_eq!(json, "\"slack\"");
    }

    #[test]
    fn test_channel_type_webhook() {
        let t = ChannelType::Webhook;
        let json = serde_json::to_string(&t).unwrap();
        assert_eq!(json, "\"webhook\"");
    }

    #[test]
    fn test_channel_type_teams() {
        let t = ChannelType::Teams;
        let json = serde_json::to_string(&t).unwrap();
        assert_eq!(json, "\"teams\"");
    }

    #[test]
    fn test_channel_type_discord() {
        let t = ChannelType::Discord;
        let json = serde_json::to_string(&t).unwrap();
        assert_eq!(json, "\"discord\"");
    }

    #[test]
    fn test_channel_type_deserialize() {
        let t: ChannelType = serde_json::from_str("\"slack\"").unwrap();
        assert!(matches!(t, ChannelType::Slack));
    }

    // ==================== TeamSecuritySettings Tests ====================

    #[test]
    fn test_team_security_settings_default() {
        let settings = TeamSecuritySettings::default();
        assert!(!settings.require_2fa);
        assert!(settings.session_timeout.is_none());
        assert!(settings.ip_whitelist.is_empty());
        assert!(!settings.audit_logging);
    }

    #[test]
    fn test_team_security_settings_strict() {
        let settings = TeamSecuritySettings {
            require_2fa: true,
            password_policy: PasswordPolicy {
                min_length: 12,
                require_uppercase: true,
                require_lowercase: true,
                require_numbers: true,
                require_special: true,
                expiry_days: Some(90),
            },
            session_timeout: Some(30),
            ip_whitelist: vec!["10.0.0.0/8".to_string()],
            audit_logging: true,
        };

        assert!(settings.require_2fa);
        assert_eq!(settings.session_timeout, Some(30));
        assert!(settings.audit_logging);
    }

    #[test]
    fn test_team_security_settings_serialize() {
        let settings = TeamSecuritySettings {
            require_2fa: true,
            password_policy: PasswordPolicy::default(),
            session_timeout: Some(60),
            ip_whitelist: vec![],
            audit_logging: true,
        };

        let json = serde_json::to_string(&settings).unwrap();
        assert!(json.contains("require_2fa"));
        assert!(json.contains("audit_logging"));
    }

    // ==================== PasswordPolicy Tests ====================

    #[test]
    fn test_password_policy_default() {
        let policy = PasswordPolicy::default();
        assert_eq!(policy.min_length, 0);
        assert!(!policy.require_uppercase);
        assert!(!policy.require_lowercase);
        assert!(!policy.require_numbers);
        assert!(!policy.require_special);
        assert!(policy.expiry_days.is_none());
    }

    #[test]
    fn test_password_policy_strict() {
        let policy = PasswordPolicy {
            min_length: 16,
            require_uppercase: true,
            require_lowercase: true,
            require_numbers: true,
            require_special: true,
            expiry_days: Some(30),
        };

        assert_eq!(policy.min_length, 16);
        assert!(policy.require_uppercase);
        assert!(policy.require_special);
        assert_eq!(policy.expiry_days, Some(30));
    }

    #[test]
    fn test_password_policy_serialize() {
        let policy = PasswordPolicy {
            min_length: 8,
            require_uppercase: true,
            require_lowercase: true,
            require_numbers: false,
            require_special: false,
            expiry_days: None,
        };

        let json = serde_json::to_string(&policy).unwrap();
        assert!(json.contains("min_length"));
        assert!(json.contains("require_uppercase"));
    }

    #[test]
    fn test_password_policy_clone() {
        let original = PasswordPolicy {
            min_length: 10,
            require_uppercase: true,
            require_lowercase: true,
            require_numbers: true,
            require_special: false,
            expiry_days: Some(60),
        };

        let cloned = original.clone();
        assert_eq!(original.min_length, cloned.min_length);
        assert_eq!(original.expiry_days, cloned.expiry_days);
    }
}
