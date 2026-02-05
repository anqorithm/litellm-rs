//! Realtime session management
//!
//! Handles WebSocket connections and session state.

use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use tokio::sync::mpsc;
use tracing::{debug, error, info};

use super::config::RealtimeConfig;
use super::events::{
    ClientEvent, ContentPart, Item, ItemRole, ItemType, RealtimeError, RealtimeResult,
    ResponseConfig, ServerEvent, SessionConfig,
};

/// Session state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SessionState {
    /// Not connected
    #[default]
    Disconnected,
    /// Connecting to server
    Connecting,
    /// Connected and ready
    Connected,
    /// Session is active with ongoing conversation
    Active,
    /// Closing connection
    Closing,
    /// Connection closed
    Closed,
    /// Error state
    Error,
}

/// Realtime session
///
/// Manages a WebSocket connection to the Realtime API.
pub struct RealtimeSession {
    /// Configuration
    config: RealtimeConfig,
    /// Current state
    state: RwLock<SessionState>,
    /// Session ID (assigned by server)
    session_id: RwLock<Option<String>>,
    /// Conversation ID
    conversation_id: RwLock<Option<String>>,
    /// Event counter for generating event IDs
    event_counter: AtomicU64,
    /// Pending items in the conversation
    items: RwLock<HashMap<String, Item>>,
    /// Channel for sending events to the WebSocket
    tx: RwLock<Option<mpsc::Sender<ClientEvent>>>,
}

impl RealtimeSession {
    /// Create a new session
    pub fn new(config: RealtimeConfig) -> Self {
        Self {
            config,
            state: RwLock::new(SessionState::Disconnected),
            session_id: RwLock::new(None),
            conversation_id: RwLock::new(None),
            event_counter: AtomicU64::new(0),
            items: RwLock::new(HashMap::new()),
            tx: RwLock::new(None),
        }
    }

    /// Get the current state
    pub fn state(&self) -> SessionState {
        *self.state.read()
    }

    /// Get the session ID
    pub fn session_id(&self) -> Option<String> {
        self.session_id.read().clone()
    }

    /// Get the conversation ID
    pub fn conversation_id(&self) -> Option<String> {
        self.conversation_id.read().clone()
    }

    /// Check if connected
    pub fn is_connected(&self) -> bool {
        matches!(self.state(), SessionState::Connected | SessionState::Active)
    }

    /// Generate a unique event ID
    fn next_event_id(&self) -> String {
        let id = self.event_counter.fetch_add(1, Ordering::SeqCst);
        format!("evt_{}", id)
    }

    /// Set the sender channel
    pub fn set_sender(&self, tx: mpsc::Sender<ClientEvent>) {
        *self.tx.write() = Some(tx);
    }

    /// Send a client event
    pub async fn send(&self, event: ClientEvent) -> RealtimeResult<()> {
        let tx = self.tx.read().clone();
        match tx {
            Some(tx) => tx
                .send(event)
                .await
                .map_err(|_| RealtimeError::connection("Channel closed")),
            None => Err(RealtimeError::connection("Not connected")),
        }
    }

    /// Update session configuration
    pub async fn update_session(&self, config: SessionConfig) -> RealtimeResult<()> {
        let event = ClientEvent::SessionUpdate {
            event_id: Some(self.next_event_id()),
            session: config,
        };
        self.send(event).await
    }

    /// Send a text message
    pub async fn send_text(&self, text: impl Into<String>) -> RealtimeResult<()> {
        let item = Item {
            id: format!("item_{}", self.event_counter.fetch_add(1, Ordering::SeqCst)),
            item_type: ItemType::Message,
            role: Some(ItemRole::User),
            content: Some(vec![ContentPart::InputText { text: text.into() }]),
            call_id: None,
            name: None,
            arguments: None,
            output: None,
        };

        let event = ClientEvent::ConversationItemCreate {
            event_id: Some(self.next_event_id()),
            previous_item_id: None,
            item,
        };

        self.send(event).await
    }

    /// Append audio to the input buffer
    pub async fn append_audio(&self, audio: impl Into<String>) -> RealtimeResult<()> {
        let event = ClientEvent::InputAudioBufferAppend {
            event_id: Some(self.next_event_id()),
            audio: audio.into(),
        };
        self.send(event).await
    }

    /// Commit the audio buffer
    pub async fn commit_audio(&self) -> RealtimeResult<()> {
        let event = ClientEvent::InputAudioBufferCommit {
            event_id: Some(self.next_event_id()),
        };
        self.send(event).await
    }

    /// Clear the audio buffer
    pub async fn clear_audio(&self) -> RealtimeResult<()> {
        let event = ClientEvent::InputAudioBufferClear {
            event_id: Some(self.next_event_id()),
        };
        self.send(event).await
    }

    /// Create a response
    pub async fn create_response(&self, config: Option<ResponseConfig>) -> RealtimeResult<()> {
        let event = ClientEvent::ResponseCreate {
            event_id: Some(self.next_event_id()),
            response: config,
        };
        self.send(event).await
    }

    /// Cancel the current response
    pub async fn cancel_response(&self) -> RealtimeResult<()> {
        let event = ClientEvent::ResponseCancel {
            event_id: Some(self.next_event_id()),
        };
        self.send(event).await
    }

    /// Submit a function call result
    pub async fn submit_function_result(
        &self,
        call_id: impl Into<String>,
        output: impl Into<String>,
    ) -> RealtimeResult<()> {
        let item = Item {
            id: format!("item_{}", self.event_counter.fetch_add(1, Ordering::SeqCst)),
            item_type: ItemType::FunctionCallOutput,
            role: None,
            content: None,
            call_id: Some(call_id.into()),
            name: None,
            arguments: None,
            output: Some(output.into()),
        };

        let event = ClientEvent::ConversationItemCreate {
            event_id: Some(self.next_event_id()),
            previous_item_id: None,
            item,
        };

        self.send(event).await
    }

    /// Delete a conversation item
    pub async fn delete_item(&self, item_id: impl Into<String>) -> RealtimeResult<()> {
        let event = ClientEvent::ConversationItemDelete {
            event_id: Some(self.next_event_id()),
            item_id: item_id.into(),
        };
        self.send(event).await
    }

    /// Handle a server event
    pub fn handle_event(&self, event: &ServerEvent) {
        match event {
            ServerEvent::SessionCreated { session, .. } => {
                info!("Session created: {}", session.id);
                *self.session_id.write() = Some(session.id.clone());
                *self.state.write() = SessionState::Connected;
            }
            ServerEvent::SessionUpdated { session, .. } => {
                debug!("Session updated: {}", session.id);
            }
            ServerEvent::ConversationCreated { conversation, .. } => {
                info!("Conversation created: {}", conversation.id);
                *self.conversation_id.write() = Some(conversation.id.clone());
                *self.state.write() = SessionState::Active;
            }
            ServerEvent::ConversationItemCreated { item, .. } => {
                debug!("Item created: {}", item.id);
                self.items.write().insert(item.id.clone(), item.clone());
            }
            ServerEvent::ConversationItemDeleted { item_id, .. } => {
                debug!("Item deleted: {}", item_id);
                self.items.write().remove(item_id);
            }
            ServerEvent::Error { error, .. } => {
                error!("Server error: {} - {}", error.error_type, error.message);
            }
            _ => {}
        }
    }

    /// Update state
    pub fn set_state(&self, state: SessionState) {
        *self.state.write() = state;
    }

    /// Get configuration
    pub fn config(&self) -> &RealtimeConfig {
        &self.config
    }

    /// Get items
    pub fn items(&self) -> HashMap<String, Item> {
        self.items.read().clone()
    }

    /// Close the session
    pub fn close(&self) {
        *self.state.write() = SessionState::Closed;
        *self.tx.write() = None;
    }
}

impl Default for RealtimeSession {
    fn default() -> Self {
        Self::new(RealtimeConfig::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_creation() {
        let config = RealtimeConfig::new("gpt-4o-realtime-preview");
        let session = RealtimeSession::new(config);

        assert_eq!(session.state(), SessionState::Disconnected);
        assert!(session.session_id().is_none());
        assert!(!session.is_connected());
    }

    #[test]
    fn test_event_id_generation() {
        let session = RealtimeSession::default();

        let id1 = session.next_event_id();
        let id2 = session.next_event_id();

        assert_ne!(id1, id2);
        assert!(id1.starts_with("evt_"));
        assert!(id2.starts_with("evt_"));
    }

    #[test]
    fn test_state_transitions() {
        let session = RealtimeSession::default();

        assert_eq!(session.state(), SessionState::Disconnected);

        session.set_state(SessionState::Connecting);
        assert_eq!(session.state(), SessionState::Connecting);

        session.set_state(SessionState::Connected);
        assert_eq!(session.state(), SessionState::Connected);
        assert!(session.is_connected());

        session.set_state(SessionState::Active);
        assert!(session.is_connected());

        session.close();
        assert_eq!(session.state(), SessionState::Closed);
        assert!(!session.is_connected());
    }

    #[test]
    fn test_handle_session_created() {
        use super::super::events::{SessionConfig, SessionInfo};

        let session = RealtimeSession::default();

        let event = ServerEvent::SessionCreated {
            event_id: "evt_1".to_string(),
            session: SessionInfo {
                id: "sess_123".to_string(),
                object: "realtime.session".to_string(),
                model: "gpt-4o-realtime-preview".to_string(),
                config: SessionConfig::default(),
            },
        };

        session.handle_event(&event);

        assert_eq!(session.session_id(), Some("sess_123".to_string()));
        assert_eq!(session.state(), SessionState::Connected);
    }

    #[test]
    fn test_handle_conversation_created() {
        use super::super::events::ConversationInfo;

        let session = RealtimeSession::default();
        session.set_state(SessionState::Connected);

        let event = ServerEvent::ConversationCreated {
            event_id: "evt_1".to_string(),
            conversation: ConversationInfo {
                id: "conv_456".to_string(),
                object: "realtime.conversation".to_string(),
            },
        };

        session.handle_event(&event);

        assert_eq!(session.conversation_id(), Some("conv_456".to_string()));
        assert_eq!(session.state(), SessionState::Active);
    }

    #[test]
    fn test_handle_item_created() {
        let session = RealtimeSession::default();

        let item = Item {
            id: "item_1".to_string(),
            item_type: ItemType::Message,
            role: Some(ItemRole::User),
            content: Some(vec![ContentPart::InputText {
                text: "Hello".to_string(),
            }]),
            call_id: None,
            name: None,
            arguments: None,
            output: None,
        };

        let event = ServerEvent::ConversationItemCreated {
            event_id: "evt_1".to_string(),
            previous_item_id: None,
            item: item.clone(),
        };

        session.handle_event(&event);

        let items = session.items();
        assert!(items.contains_key("item_1"));
    }

    #[tokio::test]
    async fn test_send_without_connection() {
        let session = RealtimeSession::default();

        let result = session.send_text("Hello").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_send_with_channel() {
        let session = RealtimeSession::default();
        let (tx, mut rx) = mpsc::channel(10);

        session.set_sender(tx);

        session.send_text("Hello").await.unwrap();

        let event = rx.recv().await.unwrap();
        assert!(matches!(event, ClientEvent::ConversationItemCreate { .. }));
    }
}
