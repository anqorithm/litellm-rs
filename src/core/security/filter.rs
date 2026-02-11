//! Content filtering and moderation
//!
//! Main content filtering implementation with PII detection and moderation.

use crate::core::models::openai::*;
use crate::utils::error::error::Result;
use tracing::warn;

use super::patterns::*;
use super::profanity::ProfanityFilter;
use super::types::*;

/// Content filter for detecting and handling sensitive content
pub struct ContentFilter {
    /// PII detection patterns
    pii_patterns: Vec<PIIPattern>,
    /// Content moderation rules
    moderation_rules: Vec<ModerationRule>,
    /// Profanity filter
    profanity_filter: ProfanityFilter,
}

impl ContentFilter {
    /// Create a new content filter
    pub fn new() -> Self {
        Self {
            pii_patterns: Self::default_pii_patterns(),
            moderation_rules: Self::default_moderation_rules(),
            profanity_filter: ProfanityFilter::new(),
        }
    }

    /// Filter chat completion request
    pub async fn filter_chat_request(
        &self,
        request: &mut ChatCompletionRequest,
    ) -> Result<FilterResult> {
        let mut issues = Vec::new();
        let mut blocked = false;
        let mut modified = false;

        // Check each message
        for message in &mut request.messages {
            if let Some(MessageContent::Text(text)) = &mut message.content {
                let result = self.filter_text(text).await?;

                if result.blocked {
                    blocked = true;
                }

                issues.extend(result.issues);

                if let Some(modified_text) = result.modified_content {
                    *text = modified_text;
                    modified = true;
                }
            }
        }

        let confidence = if issues.is_empty() {
            1.0
        } else {
            issues.iter().map(|i| i.confidence).sum::<f64>() / issues.len() as f64
        };

        Ok(FilterResult {
            blocked,
            issues,
            modified_content: if modified {
                Some("Messages modified".to_string())
            } else {
                None
            },
            confidence,
        })
    }

    /// Filter text content
    pub async fn filter_text(&self, text: &str) -> Result<FilterResult> {
        let mut issues = Vec::new();
        let mut modified_text = text.to_string();
        let mut blocked = false;

        // PII detection
        for pattern in &self.pii_patterns {
            if let Some(captures) = pattern.pattern.captures(text) {
                let issue = ContentIssue {
                    issue_type: format!("PII_{}", pattern.name),
                    description: format!("Detected {} in content", pattern.name),
                    severity: ModerationSeverity::High,
                    location: captures.get(0).map(|m| (m.start(), m.end())),
                    confidence: pattern.confidence,
                };
                issues.push(issue);

                // Apply replacement
                modified_text = self.apply_pii_replacement(&modified_text, pattern)?;
            }
        }

        // Content moderation
        for rule in &self.moderation_rules {
            if self.check_moderation_rule(&modified_text, rule).await? {
                let issue = ContentIssue {
                    issue_type: format!("MODERATION_{:?}", rule.rule_type),
                    description: format!("Content flagged for {:?}", rule.rule_type),
                    severity: rule.severity.clone(),
                    location: None,
                    confidence: 0.8, // Default confidence
                };
                issues.push(issue);

                match rule.action {
                    ModerationAction::Block => blocked = true,
                    ModerationAction::Warn => warn!("Content warning: {:?}", rule.rule_type),
                    _ => {}
                }
            }
        }

        // Profanity filtering
        if self.profanity_filter.contains_profanity(&modified_text) {
            modified_text = self.profanity_filter.filter(&modified_text);
            issues.push(ContentIssue {
                issue_type: "PROFANITY".to_string(),
                description: "Profanity detected and filtered".to_string(),
                severity: ModerationSeverity::Medium,
                location: None,
                confidence: 0.9,
            });
        }

        let confidence = if issues.is_empty() {
            1.0
        } else {
            issues.iter().map(|i| i.confidence).sum::<f64>() / issues.len() as f64
        };

        Ok(FilterResult {
            blocked,
            issues,
            modified_content: if modified_text != text {
                Some(modified_text)
            } else {
                None
            },
            confidence,
        })
    }

    /// Default PII patterns
    fn default_pii_patterns() -> Vec<PIIPattern> {
        vec![
            PIIPattern {
                name: "SSN".to_string(),
                pattern: SSN_PATTERN.clone(),
                replacement: PIIReplacement::Placeholder("XXX-XX-XXXX".to_string()),
                confidence: 0.95,
            },
            PIIPattern {
                name: "Email".to_string(),
                pattern: EMAIL_PATTERN.clone(),
                replacement: PIIReplacement::PartialMask {
                    keep_start: 2,
                    keep_end: 0,
                },
                confidence: 0.9,
            },
            PIIPattern {
                name: "Phone".to_string(),
                pattern: PHONE_PATTERN.clone(),
                replacement: PIIReplacement::Placeholder("XXX-XXX-XXXX".to_string()),
                confidence: 0.85,
            },
            PIIPattern {
                name: "CreditCard".to_string(),
                pattern: CREDIT_CARD_PATTERN.clone(),
                replacement: PIIReplacement::Placeholder("XXXX-XXXX-XXXX-XXXX".to_string()),
                confidence: 0.9,
            },
        ]
    }

    /// Default moderation rules
    fn default_moderation_rules() -> Vec<ModerationRule> {
        vec![
            ModerationRule {
                name: "Hate Speech".to_string(),
                rule_type: ModerationType::HateSpeech,
                action: ModerationAction::Block,
                severity: ModerationSeverity::High,
            },
            ModerationRule {
                name: "Violence".to_string(),
                rule_type: ModerationType::Violence,
                action: ModerationAction::Warn,
                severity: ModerationSeverity::Medium,
            },
        ]
    }

    /// Apply PII replacement
    fn apply_pii_replacement(&self, text: &str, pattern: &PIIPattern) -> Result<String> {
        let result = match &pattern.replacement {
            PIIReplacement::Redact => pattern.pattern.replace_all(text, "***").to_string(),
            PIIReplacement::Placeholder(placeholder) => pattern
                .pattern
                .replace_all(text, placeholder.as_str())
                .to_string(),
            PIIReplacement::Hash => {
                // Simple hash replacement (in production, use proper hashing)
                pattern.pattern.replace_all(text, "[HASHED]").to_string()
            }
            PIIReplacement::Remove => pattern.pattern.replace_all(text, "").to_string(),
            PIIReplacement::PartialMask {
                keep_start,
                keep_end,
            } => {
                // Implement partial masking logic
                pattern
                    .pattern
                    .replace_all(text, |caps: &regex::Captures| {
                        // caps.get(0) should always succeed for a match, but handle gracefully
                        let matched = match caps.get(0) {
                            Some(m) => m.as_str(),
                            None => return String::new(),
                        };
                        let len = matched.len();
                        if len <= keep_start + keep_end {
                            "*".repeat(len)
                        } else {
                            let start = &matched[..*keep_start];
                            let end = if *keep_end > 0 {
                                &matched[len - keep_end..]
                            } else {
                                ""
                            };
                            let middle = "*".repeat(len - keep_start - keep_end);
                            format!("{}{}{}", start, middle, end)
                        }
                    })
                    .to_string()
            }
        };
        Ok(result)
    }

    /// Check moderation rule
    async fn check_moderation_rule(&self, text: &str, rule: &ModerationRule) -> Result<bool> {
        // Simplified moderation check - in production, integrate with external services
        match rule.rule_type {
            ModerationType::HateSpeech => {
                Ok(text.to_lowercase().contains("hate") || text.to_lowercase().contains("racist"))
            }
            ModerationType::Violence => Ok(
                text.to_lowercase().contains("violence") || text.to_lowercase().contains("kill")
            ),
            _ => Ok(false),
        }
    }
}

impl Default for ContentFilter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ==================== ContentFilter Creation Tests ====================

    #[test]
    fn test_content_filter_new() {
        let filter = ContentFilter::new();
        assert_eq!(filter.pii_patterns.len(), 4); // SSN, Email, Phone, CreditCard
        assert_eq!(filter.moderation_rules.len(), 2); // HateSpeech, Violence
    }

    #[test]
    fn test_content_filter_default() {
        let filter = ContentFilter::default();
        assert_eq!(filter.pii_patterns.len(), 4);
    }

    // ==================== PII Detection Tests ====================

    #[tokio::test]
    async fn test_pii_detection() {
        let filter = ContentFilter::new();
        let text = "My SSN is 123-45-6789 and email is test@example.com";

        let result = filter.filter_text(text).await.unwrap();
        assert!(!result.issues.is_empty());
        assert!(result.modified_content.is_some());
    }

    #[tokio::test]
    async fn test_ssn_detection() {
        let filter = ContentFilter::new();
        let text = "SSN: 123-45-6789";

        let result = filter.filter_text(text).await.unwrap();
        let ssn_issues: Vec<_> = result
            .issues
            .iter()
            .filter(|i| i.issue_type.contains("SSN"))
            .collect();
        assert!(!ssn_issues.is_empty());
        assert!(result.modified_content.unwrap().contains("XXX-XX-XXXX"));
    }

    #[tokio::test]
    async fn test_email_detection() {
        let filter = ContentFilter::new();
        let text = "Contact: user@example.com";

        let result = filter.filter_text(text).await.unwrap();
        let email_issues: Vec<_> = result
            .issues
            .iter()
            .filter(|i| i.issue_type.contains("Email"))
            .collect();
        assert!(!email_issues.is_empty());
        // Email uses partial mask (keep_start: 2)
        assert!(result.modified_content.is_some());
    }

    #[tokio::test]
    async fn test_phone_detection() {
        let filter = ContentFilter::new();
        let text = "Call me at 123-456-7890";

        let result = filter.filter_text(text).await.unwrap();
        let phone_issues: Vec<_> = result
            .issues
            .iter()
            .filter(|i| i.issue_type.contains("Phone"))
            .collect();
        assert!(!phone_issues.is_empty());
        assert!(result.modified_content.unwrap().contains("XXX-XXX-XXXX"));
    }

    #[tokio::test]
    async fn test_credit_card_detection() {
        let filter = ContentFilter::new();
        let text = "Card: 1234-5678-9012-3456";

        let result = filter.filter_text(text).await.unwrap();
        let cc_issues: Vec<_> = result
            .issues
            .iter()
            .filter(|i| i.issue_type.contains("CreditCard"))
            .collect();
        assert!(!cc_issues.is_empty());
        assert!(
            result
                .modified_content
                .unwrap()
                .contains("XXXX-XXXX-XXXX-XXXX")
        );
    }

    #[tokio::test]
    async fn test_multiple_pii_types() {
        let filter = ContentFilter::new();
        let text = "SSN: 123-45-6789, Phone: 123-456-7890, Card: 1234-5678-9012-3456";

        let result = filter.filter_text(text).await.unwrap();
        assert!(result.issues.len() >= 3);
        assert!(result.modified_content.is_some());
    }

    #[tokio::test]
    async fn test_no_pii_detected() {
        let filter = ContentFilter::new();
        let text = "This is clean text without any PII";

        let result = filter.filter_text(text).await.unwrap();
        let pii_issues: Vec<_> = result
            .issues
            .iter()
            .filter(|i| i.issue_type.starts_with("PII_"))
            .collect();
        assert!(pii_issues.is_empty());
        assert!(result.modified_content.is_none());
    }

    // ==================== Moderation Tests ====================

    #[tokio::test]
    async fn test_hate_speech_detection() {
        let filter = ContentFilter::new();
        let text = "This contains hate speech";

        let result = filter.filter_text(text).await.unwrap();
        let hate_issues: Vec<_> = result
            .issues
            .iter()
            .filter(|i| i.issue_type.contains("HateSpeech"))
            .collect();
        assert!(!hate_issues.is_empty());
        assert!(result.blocked);
    }

    #[tokio::test]
    async fn test_racist_content_detection() {
        let filter = ContentFilter::new();
        let text = "This is racist content";

        let result = filter.filter_text(text).await.unwrap();
        assert!(result.blocked);
    }

    #[tokio::test]
    async fn test_violence_detection() {
        let filter = ContentFilter::new();
        let text = "Content about violence";

        let result = filter.filter_text(text).await.unwrap();
        let violence_issues: Vec<_> = result
            .issues
            .iter()
            .filter(|i| i.issue_type.contains("Violence"))
            .collect();
        assert!(!violence_issues.is_empty());
        // Violence is a warning, not a block
        assert!(!result.blocked);
    }

    #[tokio::test]
    async fn test_kill_keyword_detection() {
        let filter = ContentFilter::new();
        let text = "Content with kill keyword";

        let result = filter.filter_text(text).await.unwrap();
        let violence_issues: Vec<_> = result
            .issues
            .iter()
            .filter(|i| i.issue_type.contains("Violence"))
            .collect();
        assert!(!violence_issues.is_empty());
    }

    #[tokio::test]
    async fn test_clean_content() {
        let filter = ContentFilter::new();
        let text = "This is perfectly clean content";

        let result = filter.filter_text(text).await.unwrap();
        assert!(!result.blocked);
        assert!(result.modified_content.is_none());
    }

    // ==================== Profanity Tests ====================

    #[tokio::test]
    async fn test_profanity_filtered() {
        let filter = ContentFilter::new();
        // Use known profanity words from the ProfanityFilter
        let text = "This is badword1 text";

        let result = filter.filter_text(text).await.unwrap();
        let profanity_issues: Vec<_> = result
            .issues
            .iter()
            .filter(|i| i.issue_type == "PROFANITY")
            .collect();
        assert!(!profanity_issues.is_empty());
    }

    // ==================== FilterResult Tests ====================

    #[tokio::test]
    async fn test_filter_result_confidence() {
        let filter = ContentFilter::new();
        let clean_text = "Normal text";

        let result = filter.filter_text(clean_text).await.unwrap();
        assert!((result.confidence - 1.0).abs() < 0.01); // Clean text = 1.0 confidence
    }

    #[tokio::test]
    async fn test_filter_result_confidence_with_issues() {
        let filter = ContentFilter::new();
        let text = "SSN: 123-45-6789";

        let result = filter.filter_text(text).await.unwrap();
        assert!(result.confidence > 0.0);
        assert!(result.confidence < 1.0);
    }

    // ==================== PIIReplacement Tests ====================

    #[test]
    fn test_pii_pattern_structures() {
        let patterns = ContentFilter::default_pii_patterns();

        assert_eq!(patterns.len(), 4);

        // SSN pattern
        let ssn = patterns.iter().find(|p| p.name == "SSN").unwrap();
        assert!((ssn.confidence - 0.95).abs() < 0.01);

        // Email pattern
        let email = patterns.iter().find(|p| p.name == "Email").unwrap();
        assert!((email.confidence - 0.9).abs() < 0.01);

        // Phone pattern
        let phone = patterns.iter().find(|p| p.name == "Phone").unwrap();
        assert!((phone.confidence - 0.85).abs() < 0.01);

        // CreditCard pattern
        let cc = patterns.iter().find(|p| p.name == "CreditCard").unwrap();
        assert!((cc.confidence - 0.9).abs() < 0.01);
    }

    // ==================== ModerationRule Tests ====================

    #[test]
    fn test_moderation_rule_structures() {
        let rules = ContentFilter::default_moderation_rules();

        assert_eq!(rules.len(), 2);

        let hate_rule = rules.iter().find(|r| r.name == "Hate Speech").unwrap();
        matches!(hate_rule.action, ModerationAction::Block);

        let violence_rule = rules.iter().find(|r| r.name == "Violence").unwrap();
        matches!(violence_rule.action, ModerationAction::Warn);
    }

    // ==================== Edge Cases ====================

    #[tokio::test]
    async fn test_empty_text() {
        let filter = ContentFilter::new();
        let text = "";

        let result = filter.filter_text(text).await.unwrap();
        assert!(!result.blocked);
        assert!(result.issues.is_empty());
    }

    #[tokio::test]
    async fn test_whitespace_only() {
        let filter = ContentFilter::new();
        let text = "   \t\n   ";

        let result = filter.filter_text(text).await.unwrap();
        assert!(!result.blocked);
    }

    #[tokio::test]
    async fn test_case_insensitive_moderation() {
        let filter = ContentFilter::new();

        // Uppercase
        let result1 = filter.filter_text("HATE").await.unwrap();
        assert!(result1.blocked);

        // Mixed case
        let result2 = filter.filter_text("HaTe").await.unwrap();
        assert!(result2.blocked);
    }

    #[tokio::test]
    async fn test_combined_pii_and_moderation() {
        let filter = ContentFilter::new();
        let text = "SSN: 123-45-6789 and this is hate speech";

        let result = filter.filter_text(text).await.unwrap();
        assert!(result.blocked);
        assert!(result.issues.len() >= 2);
        assert!(result.modified_content.is_some());
    }
}
