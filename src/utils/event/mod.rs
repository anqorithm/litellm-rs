//! Generic Event Publish-Subscribe System
//!
//! This module provides a flexible, async-first event broker implementation
//! inspired by Crush's pubsub.Broker design. It supports multiple subscribers,
//! non-blocking event delivery, and configurable channel capacity.
//!
//! # Features
//!
//! - **Generic Events**: Type-safe events with any payload type
//! - **Multiple Subscribers**: Support for multiple concurrent subscribers
//! - **Non-blocking Delivery**: Slow consumers don't block publishers
//! - **Async Support**: Built on tokio for async operations
//! - **Unsubscribe Support**: Subscribers can be removed at any time
//! - **Configurable Capacity**: Channel buffer size can be configured
//!
//! # Example
//!
//! ```rust,ignore
//! use litellm_rs::utils::event::{EventBroker, Event, EventType};
//!
//! #[derive(Clone, Debug)]
//! struct UserData {
//!     id: u64,
//!     name: String,
//! }
//!
//! #[tokio::main]
//! async fn main() {
//!     let broker = EventBroker::<UserData>::new();
//!
//!     // Subscribe to events
//!     let mut rx = broker.subscribe();
//!
//!     // Publish an event
//!     let user = UserData { id: 1, name: "Alice".to_string() };
//!     broker.publish(Event::created(user)).await;
//!
//!     // Receive the event
//!     if let Ok(event) = rx.recv().await {
//!         println!("Received: {:?}", event);
//!     }
//! }
//! ```

mod broker;
mod types;

#[cfg(test)]
mod tests;

pub use broker::EventBroker;
pub use types::{Event, EventType, Subscriber, SubscriptionHandle};
