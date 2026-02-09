use super::*;

#[test]
fn test_voice_display() {
    assert_eq!(Voice::Alloy.to_string(), "alloy");
    assert_eq!(Voice::Echo.to_string(), "echo");
    assert_eq!(Voice::Nova.to_string(), "nova");
}

#[test]
fn test_session_config_default() {
    let config = SessionConfig::default();
    assert!(config.modalities.is_some());
    assert_eq!(config.voice, Some(Voice::Alloy));
}

#[test]
fn test_client_event_serialization() {
    let event = ClientEvent::SessionUpdate {
        event_id: Some("evt-1".to_string()),
        session: SessionConfig::default(),
    };

    let json = serde_json::to_string(&event).unwrap();
    assert!(json.contains("session_update"));
}

#[test]
fn test_error_constructors() {
    let err = RealtimeError::connection("Failed to connect");
    assert!(matches!(err, RealtimeError::Connection(_)));

    let err = RealtimeError::auth("Invalid token");
    assert!(matches!(err, RealtimeError::Authentication(_)));

    let err = RealtimeError::server("500", "Internal error");
    assert!(matches!(err, RealtimeError::Server { .. }));
}
