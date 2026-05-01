//! Log probability and finish reason types

use serde::{Deserialize, Serialize};

/// Finish reason
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FinishReason {
    /// Natural stop
    Stop,
    /// Length limit reached
    Length,
    /// Tool call
    ToolCalls,
    /// Content filter
    ContentFilter,
    /// Function call (backward compatibility)
    FunctionCall,
    /// Provider stopped at an explicit stop sequence
    StopSequence,
    /// Provider refused the request
    Refusal,
    /// Provider paused the turn for a later continuation
    PauseTurn,
}

/// Log probabilities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogProbs {
    /// Token log probabilities
    pub content: Vec<TokenLogProb>,

    /// Refusal sampling information
    #[serde(skip_serializing_if = "Option::is_none")]
    pub refusal: Option<String>,
}

/// Single token log probability
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenLogProb {
    /// Token text
    pub token: String,

    /// Log probability
    pub logprob: f64,

    /// Token byte representation
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bytes: Option<Vec<u8>>,

    /// Top log probabilities
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_logprobs: Option<Vec<TopLogProb>>,
}

/// Top log probability
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopLogProb {
    /// Token text
    pub token: String,

    /// Log probability
    pub logprob: f64,

    /// Token byte representation
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bytes: Option<Vec<u8>>,
}

#[cfg(test)]
mod tests {
    use super::*;

    // ==================== FinishReason Tests ====================

    #[test]
    fn test_finish_reason_stop_serialization() {
        let reason = FinishReason::Stop;
        let json = serde_json::to_string(&reason).unwrap();
        assert_eq!(json, "\"stop\"");
    }

    #[test]
    fn test_finish_reason_length_serialization() {
        let reason = FinishReason::Length;
        let json = serde_json::to_string(&reason).unwrap();
        assert_eq!(json, "\"length\"");
    }

    #[test]
    fn test_finish_reason_tool_calls_serialization() {
        let reason = FinishReason::ToolCalls;
        let json = serde_json::to_string(&reason).unwrap();
        assert_eq!(json, "\"tool_calls\"");
    }

    #[test]
    fn test_finish_reason_content_filter_serialization() {
        let reason = FinishReason::ContentFilter;
        let json = serde_json::to_string(&reason).unwrap();
        assert_eq!(json, "\"content_filter\"");
    }

    #[test]
    fn test_finish_reason_function_call_serialization() {
        let reason = FinishReason::FunctionCall;
        let json = serde_json::to_string(&reason).unwrap();
        assert_eq!(json, "\"function_call\"");
    }

    #[test]
    fn test_finish_reason_provider_specific_serialization() {
        assert_eq!(
            serde_json::to_string(&FinishReason::StopSequence).unwrap(),
            "\"stop_sequence\""
        );
        assert_eq!(
            serde_json::to_string(&FinishReason::Refusal).unwrap(),
            "\"refusal\""
        );
        assert_eq!(
            serde_json::to_string(&FinishReason::PauseTurn).unwrap(),
            "\"pause_turn\""
        );
    }

    #[test]
    fn test_finish_reason_deserialization() {
        let stop: FinishReason = serde_json::from_str("\"stop\"").unwrap();
        let length: FinishReason = serde_json::from_str("\"length\"").unwrap();
        let tool_calls: FinishReason = serde_json::from_str("\"tool_calls\"").unwrap();
        let content_filter: FinishReason = serde_json::from_str("\"content_filter\"").unwrap();
        let function_call: FinishReason = serde_json::from_str("\"function_call\"").unwrap();
        let stop_sequence: FinishReason = serde_json::from_str("\"stop_sequence\"").unwrap();
        let refusal: FinishReason = serde_json::from_str("\"refusal\"").unwrap();
        let pause_turn: FinishReason = serde_json::from_str("\"pause_turn\"").unwrap();

        assert_eq!(stop, FinishReason::Stop);
        assert_eq!(length, FinishReason::Length);
        assert_eq!(tool_calls, FinishReason::ToolCalls);
        assert_eq!(content_filter, FinishReason::ContentFilter);
        assert_eq!(function_call, FinishReason::FunctionCall);
        assert_eq!(stop_sequence, FinishReason::StopSequence);
        assert_eq!(refusal, FinishReason::Refusal);
        assert_eq!(pause_turn, FinishReason::PauseTurn);
    }

    #[test]
    fn test_finish_reason_equality() {
        assert_eq!(FinishReason::Stop, FinishReason::Stop);
        assert_ne!(FinishReason::Stop, FinishReason::Length);
    }

    #[test]
    fn test_finish_reason_clone() {
        let reason = FinishReason::ToolCalls;
        let cloned = reason.clone();
        assert_eq!(reason, cloned);
    }

    // ==================== LogProbs Tests ====================

    #[test]
    fn test_logprobs_structure() {
        let logprobs = LogProbs {
            content: vec![],
            refusal: None,
        };

        assert!(logprobs.content.is_empty());
        assert!(logprobs.refusal.is_none());
    }

    #[test]
    fn test_logprobs_with_content() {
        let token_prob = TokenLogProb {
            token: "hello".to_string(),
            logprob: -0.5,
            bytes: None,
            top_logprobs: None,
        };

        let logprobs = LogProbs {
            content: vec![token_prob],
            refusal: None,
        };

        assert_eq!(logprobs.content.len(), 1);
        assert_eq!(logprobs.content[0].token, "hello");
    }

    #[test]
    fn test_logprobs_with_refusal() {
        let logprobs = LogProbs {
            content: vec![],
            refusal: Some("Content policy violation".to_string()),
        };

        assert_eq!(
            logprobs.refusal,
            Some("Content policy violation".to_string())
        );
    }

    #[test]
    fn test_logprobs_serialization_skip_none_refusal() {
        let logprobs = LogProbs {
            content: vec![],
            refusal: None,
        };

        let json = serde_json::to_value(&logprobs).unwrap();
        assert!(!json.as_object().unwrap().contains_key("refusal"));
    }

    #[test]
    fn test_logprobs_serialization_include_refusal() {
        let logprobs = LogProbs {
            content: vec![],
            refusal: Some("refused".to_string()),
        };

        let json = serde_json::to_value(&logprobs).unwrap();
        assert_eq!(json["refusal"], "refused");
    }

    #[test]
    fn test_logprobs_clone() {
        let logprobs = LogProbs {
            content: vec![TokenLogProb {
                token: "test".to_string(),
                logprob: -1.0,
                bytes: None,
                top_logprobs: None,
            }],
            refusal: None,
        };

        let cloned = logprobs.clone();
        assert_eq!(logprobs.content.len(), cloned.content.len());
    }

    // ==================== TokenLogProb Tests ====================

    #[test]
    fn test_token_logprob_structure() {
        let token_prob = TokenLogProb {
            token: "world".to_string(),
            logprob: -2.5,
            bytes: None,
            top_logprobs: None,
        };

        assert_eq!(token_prob.token, "world");
        assert!((token_prob.logprob - (-2.5)).abs() < f64::EPSILON);
    }

    #[test]
    fn test_token_logprob_with_bytes() {
        let token_prob = TokenLogProb {
            token: "test".to_string(),
            logprob: -1.0,
            bytes: Some(vec![116, 101, 115, 116]), // "test" in bytes
            top_logprobs: None,
        };

        assert_eq!(token_prob.bytes, Some(vec![116, 101, 115, 116]));
    }

    #[test]
    fn test_token_logprob_with_top_logprobs() {
        let top_probs = vec![
            TopLogProb {
                token: "a".to_string(),
                logprob: -0.1,
                bytes: None,
            },
            TopLogProb {
                token: "b".to_string(),
                logprob: -0.5,
                bytes: None,
            },
        ];

        let token_prob = TokenLogProb {
            token: "a".to_string(),
            logprob: -0.1,
            bytes: None,
            top_logprobs: Some(top_probs),
        };

        assert_eq!(token_prob.top_logprobs.as_ref().unwrap().len(), 2);
    }

    #[test]
    fn test_token_logprob_serialization_skip_none() {
        let token_prob = TokenLogProb {
            token: "skip".to_string(),
            logprob: -1.0,
            bytes: None,
            top_logprobs: None,
        };

        let json = serde_json::to_value(&token_prob).unwrap();
        assert!(!json.as_object().unwrap().contains_key("bytes"));
        assert!(!json.as_object().unwrap().contains_key("top_logprobs"));
    }

    #[test]
    fn test_token_logprob_clone() {
        let token_prob = TokenLogProb {
            token: "clone".to_string(),
            logprob: -0.5,
            bytes: Some(vec![1, 2, 3]),
            top_logprobs: None,
        };

        let cloned = token_prob.clone();
        assert_eq!(token_prob.token, cloned.token);
        assert_eq!(token_prob.bytes, cloned.bytes);
    }

    // ==================== TopLogProb Tests ====================

    #[test]
    fn test_top_logprob_structure() {
        let top_prob = TopLogProb {
            token: "candidate".to_string(),
            logprob: -0.3,
            bytes: None,
        };

        assert_eq!(top_prob.token, "candidate");
        assert!((top_prob.logprob - (-0.3)).abs() < f64::EPSILON);
    }

    #[test]
    fn test_top_logprob_with_bytes() {
        let top_prob = TopLogProb {
            token: "hi".to_string(),
            logprob: -0.1,
            bytes: Some(vec![104, 105]), // "hi" in bytes
        };

        assert_eq!(top_prob.bytes, Some(vec![104, 105]));
    }

    #[test]
    fn test_top_logprob_serialization() {
        let top_prob = TopLogProb {
            token: "ser".to_string(),
            logprob: -0.5,
            bytes: Some(vec![1]),
        };

        let json = serde_json::to_value(&top_prob).unwrap();
        assert_eq!(json["token"], "ser");
        assert_eq!(json["logprob"], -0.5);
    }

    #[test]
    fn test_top_logprob_clone() {
        let top_prob = TopLogProb {
            token: "clone".to_string(),
            logprob: -0.2,
            bytes: None,
        };

        let cloned = top_prob.clone();
        assert_eq!(top_prob.token, cloned.token);
    }

    #[test]
    fn test_top_logprob_deserialization() {
        let json = r#"{"token": "test", "logprob": -1.5}"#;
        let top_prob: TopLogProb = serde_json::from_str(json).unwrap();

        assert_eq!(top_prob.token, "test");
        assert!((top_prob.logprob - (-1.5)).abs() < f64::EPSILON);
        assert!(top_prob.bytes.is_none());
    }

    // ==================== Integration Tests ====================

    #[test]
    fn test_full_logprobs_structure() {
        let logprobs = LogProbs {
            content: vec![
                TokenLogProb {
                    token: "Hello".to_string(),
                    logprob: -0.1,
                    bytes: Some(vec![72, 101, 108, 108, 111]),
                    top_logprobs: Some(vec![
                        TopLogProb {
                            token: "Hello".to_string(),
                            logprob: -0.1,
                            bytes: Some(vec![72, 101, 108, 108, 111]),
                        },
                        TopLogProb {
                            token: "Hi".to_string(),
                            logprob: -0.5,
                            bytes: Some(vec![72, 105]),
                        },
                    ]),
                },
                TokenLogProb {
                    token: " world".to_string(),
                    logprob: -0.2,
                    bytes: None,
                    top_logprobs: None,
                },
            ],
            refusal: None,
        };

        assert_eq!(logprobs.content.len(), 2);
        assert_eq!(logprobs.content[0].top_logprobs.as_ref().unwrap().len(), 2);

        // Verify serialization roundtrip
        let json = serde_json::to_string(&logprobs).unwrap();
        let deserialized: LogProbs = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.content.len(), 2);
    }
}
