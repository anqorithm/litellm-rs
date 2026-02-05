//! Event types and traits for the publish-subscribe system

use std::fmt::Debug;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::mpsc;
use uuid::Uuid;

/// Event type enumeration representing common CRUD operations
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EventType {
    /// Entity was created
    Created,
    /// Entity was updated
    Updated,
    /// Entity was deleted
    Deleted,
    /// Custom event type with a numeric identifier
    Custom(u32),
}

impl EventType {
    /// Returns true if this is a Created event
    pub fn is_created(&self) -> bool {
        matches!(self, EventType::Created)
    }

    /// Returns true if this is an Updated event
    pub fn is_updated(&self) -> bool {
        matches!(self, EventType::Updated)
    }

    /// Returns true if this is a Deleted event
    pub fn is_deleted(&self) -> bool {
        matches!(self, EventType::Deleted)
    }

    /// Returns true if this is a Custom event
    pub fn is_custom(&self) -> bool {
        matches!(self, EventType::Custom(_))
    }

    /// Returns the custom event code if this is a Custom event
    pub fn custom_code(&self) -> Option<u32> {
        match self {
            EventType::Custom(code) => Some(*code),
            _ => None,
        }
    }
}

impl std::fmt::Display for EventType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EventType::Created => write!(f, "Created"),
            EventType::Updated => write!(f, "Updated"),
            EventType::Deleted => write!(f, "Deleted"),
            EventType::Custom(code) => write!(f, "Custom({})", code),
        }
    }
}

/// Generic event structure containing metadata and payload
#[derive(Debug, Clone)]
pub struct Event<T>
where
    T: Clone + Send + Sync,
{
    /// Unique event identifier
    pub id: String,
    /// Type of the event
    pub event_type: EventType,
    /// Event payload data
    pub data: T,
    /// Unix timestamp in milliseconds when the event was created
    pub timestamp: u64,
    /// Optional source identifier (e.g., component name)
    pub source: Option<String>,
    /// Optional correlation ID for tracing related events
    pub correlation_id: Option<String>,
}

impl<T> Event<T>
where
    T: Clone + Send + Sync,
{
    /// Create a new event with the specified type and data
    pub fn new(event_type: EventType, data: T) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            event_type,
            data,
            timestamp: current_timestamp_millis(),
            source: None,
            correlation_id: None,
        }
    }

    /// Create a Created event
    pub fn created(data: T) -> Self {
        Self::new(EventType::Created, data)
    }

    /// Create an Updated event
    pub fn updated(data: T) -> Self {
        Self::new(EventType::Updated, data)
    }

    /// Create a Deleted event
    pub fn deleted(data: T) -> Self {
        Self::new(EventType::Deleted, data)
    }

    /// Create a Custom event with the specified code
    pub fn custom(code: u32, data: T) -> Self {
        Self::new(EventType::Custom(code), data)
    }

    /// Set the source of the event
    pub fn with_source(mut self, source: impl Into<String>) -> Self {
        self.source = Some(source.into());
        self
    }

    /// Set the correlation ID for the event
    pub fn with_correlation_id(mut self, correlation_id: impl Into<String>) -> Self {
        self.correlation_id = Some(correlation_id.into());
        self
    }

    /// Check if this event matches the specified type
    pub fn is_type(&self, event_type: EventType) -> bool {
        self.event_type == event_type
    }
}

/// Subscriber trait for handling events
///
/// Implement this trait to create custom event handlers that can be
/// registered with the EventBroker.
#[async_trait::async_trait]
pub trait Subscriber<T>: Send + Sync
where
    T: Clone + Send + Sync + 'static,
{
    /// Handle an incoming event
    ///
    /// This method is called for each event published to the broker.
    /// Implementations should handle errors gracefully and not panic.
    async fn on_event(&self, event: Event<T>);

    /// Optional filter to determine if this subscriber should receive an event
    ///
    /// Return true to receive the event, false to skip it.
    /// Default implementation returns true for all events.
    fn should_receive(&self, event: &Event<T>) -> bool {
        let _ = event;
        true
    }
}

/// Handle for managing a subscription
///
/// When dropped, the subscription is automatically cancelled.
#[derive(Debug)]
pub struct SubscriptionHandle {
    /// Unique identifier for this subscription
    pub id: String,
    /// Flag indicating if the subscription is still active
    active: std::sync::atomic::AtomicBool,
}

impl SubscriptionHandle {
    /// Create a new subscription handle
    pub fn new() -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            active: std::sync::atomic::AtomicBool::new(true),
        }
    }

    /// Check if the subscription is still active
    pub fn is_active(&self) -> bool {
        self.active.load(std::sync::atomic::Ordering::Relaxed)
    }

    /// Cancel the subscription
    pub fn cancel(&self) {
        self.active
            .store(false, std::sync::atomic::Ordering::Relaxed);
    }
}

impl Default for SubscriptionHandle {
    fn default() -> Self {
        Self::new()
    }
}

/// Type alias for the event receiver channel
pub type EventReceiver<T> = mpsc::Receiver<Event<T>>;

/// Type alias for the event sender channel
pub type EventSender<T> = mpsc::Sender<Event<T>>;

/// Get current timestamp in milliseconds
fn current_timestamp_millis() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}
