//! Event broker implementation for publish-subscribe pattern

use super::types::{Event, EventReceiver, EventSender, SubscriptionHandle};
use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::mpsc;
use tracing::{debug, trace, warn};

/// Default channel capacity for subscribers
const DEFAULT_CHANNEL_CAPACITY: usize = 256;

/// Configuration for the EventBroker
#[derive(Debug, Clone)]
pub struct EventBrokerConfig {
    /// Channel buffer capacity for each subscriber
    pub channel_capacity: usize,
    /// Whether to log dropped events
    pub log_dropped_events: bool,
}

impl Default for EventBrokerConfig {
    fn default() -> Self {
        Self {
            channel_capacity: DEFAULT_CHANNEL_CAPACITY,
            log_dropped_events: true,
        }
    }
}

impl EventBrokerConfig {
    /// Create a new config with the specified channel capacity
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            channel_capacity: capacity.max(1),
            ..Default::default()
        }
    }
}

/// Internal subscriber entry
struct SubscriberEntry<T>
where
    T: Clone + Send + Sync,
{
    sender: EventSender<T>,
    handle: Arc<SubscriptionHandle>,
}

/// Generic event broker for publish-subscribe pattern
///
/// The EventBroker manages multiple subscribers and delivers events
/// to all active subscribers in a non-blocking manner.
///
/// # Type Parameters
///
/// * `T` - The event payload type. Must implement Clone, Send, and Sync.
///
/// # Thread Safety
///
/// The EventBroker is thread-safe and can be shared across multiple tasks
/// using `Arc<EventBroker<T>>`.
pub struct EventBroker<T>
where
    T: Clone + Send + Sync + 'static,
{
    /// Map of subscriber ID to subscriber entry
    subscribers: RwLock<HashMap<String, SubscriberEntry<T>>>,
    /// Broker configuration
    config: EventBrokerConfig,
    /// Total events published
    events_published: std::sync::atomic::AtomicU64,
    /// Total events dropped (due to slow consumers)
    events_dropped: std::sync::atomic::AtomicU64,
}

impl<T> EventBroker<T>
where
    T: Clone + Send + Sync + 'static,
{
    /// Create a new EventBroker with default configuration
    pub fn new() -> Self {
        Self::with_config(EventBrokerConfig::default())
    }

    /// Create a new EventBroker with the specified channel capacity
    pub fn with_capacity(capacity: usize) -> Self {
        Self::with_config(EventBrokerConfig::with_capacity(capacity))
    }

    /// Create a new EventBroker with the specified configuration
    pub fn with_config(config: EventBrokerConfig) -> Self {
        debug!(
            "Creating EventBroker with capacity: {}",
            config.channel_capacity
        );
        Self {
            subscribers: RwLock::new(HashMap::new()),
            config,
            events_published: std::sync::atomic::AtomicU64::new(0),
            events_dropped: std::sync::atomic::AtomicU64::new(0),
        }
    }

    /// Subscribe to events and receive them via a channel
    ///
    /// Returns a tuple of (SubscriptionHandle, EventReceiver).
    /// The handle can be used to unsubscribe, and the receiver
    /// is used to receive events.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let broker = EventBroker::<String>::new();
    /// let (handle, mut rx) = broker.subscribe();
    ///
    /// // Receive events
    /// while let Some(event) = rx.recv().await {
    ///     println!("Received: {:?}", event);
    /// }
    /// ```
    pub fn subscribe(&self) -> (Arc<SubscriptionHandle>, EventReceiver<T>) {
        let capacity = self.config.channel_capacity.max(1);
        let (tx, rx) = mpsc::channel(capacity);
        let handle = Arc::new(SubscriptionHandle::new());

        let entry = SubscriberEntry {
            sender: tx,
            handle: handle.clone(),
        };

        {
            let mut subscribers = self.subscribers.write();
            subscribers.insert(handle.id.clone(), entry);
        }

        debug!("New subscriber registered: {}", handle.id);
        (handle, rx)
    }

    /// Subscribe with a custom channel capacity
    ///
    /// This allows individual subscribers to have different buffer sizes.
    pub fn subscribe_with_capacity(
        &self,
        capacity: usize,
    ) -> (Arc<SubscriptionHandle>, EventReceiver<T>) {
        let capacity = capacity.max(1);
        let (tx, rx) = mpsc::channel(capacity);
        let handle = Arc::new(SubscriptionHandle::new());

        let entry = SubscriberEntry {
            sender: tx,
            handle: handle.clone(),
        };

        {
            let mut subscribers = self.subscribers.write();
            subscribers.insert(handle.id.clone(), entry);
        }

        debug!(
            "New subscriber registered with custom capacity {}: {}",
            capacity, handle.id
        );
        (handle, rx)
    }

    /// Unsubscribe using the subscription handle
    ///
    /// Returns true if the subscriber was found and removed.
    pub fn unsubscribe(&self, handle: &SubscriptionHandle) -> bool {
        handle.cancel();
        let mut subscribers = self.subscribers.write();
        let removed = subscribers.remove(&handle.id).is_some();

        if removed {
            debug!("Subscriber unsubscribed: {}", handle.id);
        } else {
            trace!("Attempted to unsubscribe unknown subscriber: {}", handle.id);
        }

        removed
    }

    /// Unsubscribe by subscription ID
    ///
    /// Returns true if the subscriber was found and removed.
    pub fn unsubscribe_by_id(&self, id: &str) -> bool {
        let mut subscribers = self.subscribers.write();
        if let Some(entry) = subscribers.remove(id) {
            entry.handle.cancel();
            debug!("Subscriber unsubscribed by ID: {}", id);
            true
        } else {
            trace!("Attempted to unsubscribe unknown subscriber ID: {}", id);
            false
        }
    }

    /// Publish an event to all subscribers
    ///
    /// This method is non-blocking. If a subscriber's channel is full,
    /// the event is dropped for that subscriber (logged if configured).
    ///
    /// # Returns
    ///
    /// The number of subscribers that successfully received the event.
    pub async fn publish(&self, event: Event<T>) -> usize {
        self.events_published
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);

        let subscribers_snapshot: Vec<(String, EventSender<T>, Arc<SubscriptionHandle>)> = {
            let subscribers = self.subscribers.read();
            subscribers
                .iter()
                .filter(|(_, entry)| entry.handle.is_active())
                .map(|(id, entry)| (id.clone(), entry.sender.clone(), entry.handle.clone()))
                .collect()
        };

        let mut delivered = 0;
        let mut to_remove = Vec::new();

        for (id, sender, handle) in subscribers_snapshot {
            if !handle.is_active() {
                to_remove.push(id);
                continue;
            }

            // Use try_send for non-blocking delivery
            match sender.try_send(event.clone()) {
                Ok(()) => {
                    delivered += 1;
                    trace!("Event {} delivered to subscriber {}", event.id, id);
                }
                Err(mpsc::error::TrySendError::Full(_)) => {
                    self.events_dropped
                        .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                    if self.config.log_dropped_events {
                        warn!(
                            "Event {} dropped for slow subscriber {}: channel full",
                            event.id, id
                        );
                    }
                }
                Err(mpsc::error::TrySendError::Closed(_)) => {
                    to_remove.push(id.clone());
                    debug!("Subscriber {} channel closed, marking for removal", id);
                }
            }
        }

        // Clean up closed subscribers
        if !to_remove.is_empty() {
            let mut subscribers = self.subscribers.write();
            for id in to_remove {
                if let Some(entry) = subscribers.remove(&id) {
                    entry.handle.cancel();
                }
            }
        }

        delivered
    }

    /// Publish an event and wait for all subscribers to receive it
    ///
    /// Unlike `publish`, this method will wait for slow subscribers
    /// (up to the channel capacity). Use with caution as it can block.
    ///
    /// # Returns
    ///
    /// The number of subscribers that successfully received the event.
    pub async fn publish_blocking(&self, event: Event<T>) -> usize {
        self.events_published
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);

        let subscribers_snapshot: Vec<(String, EventSender<T>, Arc<SubscriptionHandle>)> = {
            let subscribers = self.subscribers.read();
            subscribers
                .iter()
                .filter(|(_, entry)| entry.handle.is_active())
                .map(|(id, entry)| (id.clone(), entry.sender.clone(), entry.handle.clone()))
                .collect()
        };

        let mut delivered = 0;
        let mut to_remove = Vec::new();

        for (id, sender, handle) in subscribers_snapshot {
            if !handle.is_active() {
                to_remove.push(id);
                continue;
            }

            match sender.send(event.clone()).await {
                Ok(()) => {
                    delivered += 1;
                    trace!("Event {} delivered to subscriber {}", event.id, id);
                }
                Err(_) => {
                    to_remove.push(id.clone());
                    debug!("Subscriber {} channel closed, marking for removal", id);
                }
            }
        }

        // Clean up closed subscribers
        if !to_remove.is_empty() {
            let mut subscribers = self.subscribers.write();
            for id in to_remove {
                if let Some(entry) = subscribers.remove(&id) {
                    entry.handle.cancel();
                }
            }
        }

        delivered
    }

    /// Get the number of active subscribers
    pub fn subscriber_count(&self) -> usize {
        let subscribers = self.subscribers.read();
        subscribers
            .values()
            .filter(|entry| entry.handle.is_active())
            .count()
    }

    /// Get the total number of events published
    pub fn events_published(&self) -> u64 {
        self.events_published
            .load(std::sync::atomic::Ordering::Relaxed)
    }

    /// Get the total number of events dropped
    pub fn events_dropped(&self) -> u64 {
        self.events_dropped
            .load(std::sync::atomic::Ordering::Relaxed)
    }

    /// Check if there are any active subscribers
    pub fn has_subscribers(&self) -> bool {
        self.subscriber_count() > 0
    }

    /// Clear all subscribers
    pub fn clear(&self) {
        let mut subscribers = self.subscribers.write();
        for (_, entry) in subscribers.drain() {
            entry.handle.cancel();
        }
        debug!("All subscribers cleared");
    }

    /// Get broker statistics
    pub fn stats(&self) -> EventBrokerStats {
        EventBrokerStats {
            subscriber_count: self.subscriber_count(),
            events_published: self.events_published(),
            events_dropped: self.events_dropped(),
            channel_capacity: self.config.channel_capacity,
        }
    }
}

impl<T> Default for EventBroker<T>
where
    T: Clone + Send + Sync + 'static,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<T> std::fmt::Debug for EventBroker<T>
where
    T: Clone + Send + Sync + 'static,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EventBroker")
            .field("subscriber_count", &self.subscriber_count())
            .field("events_published", &self.events_published())
            .field("events_dropped", &self.events_dropped())
            .field("channel_capacity", &self.config.channel_capacity)
            .finish()
    }
}

/// Statistics for the EventBroker
#[derive(Debug, Clone)]
pub struct EventBrokerStats {
    /// Number of active subscribers
    pub subscriber_count: usize,
    /// Total events published
    pub events_published: u64,
    /// Total events dropped
    pub events_dropped: u64,
    /// Channel capacity per subscriber
    pub channel_capacity: usize,
}
