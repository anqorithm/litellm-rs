//! Tests for the event publish-subscribe system

use super::broker::{EventBroker, EventBrokerConfig};
use super::types::{Event, EventType, Subscriber, SubscriptionHandle};
use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};
use std::time::Duration;
use tokio::sync::Barrier;
use tokio::time::timeout;

// ==================== Test Data Types ====================

#[derive(Debug, Clone, PartialEq)]
struct TestData {
    id: u64,
    value: String,
}

impl TestData {
    fn new(id: u64, value: &str) -> Self {
        Self {
            id,
            value: value.to_string(),
        }
    }
}

// ==================== EventType Tests ====================

#[test]
fn test_event_type_created() {
    let event_type = EventType::Created;
    assert!(event_type.is_created());
    assert!(!event_type.is_updated());
    assert!(!event_type.is_deleted());
    assert!(!event_type.is_custom());
    assert_eq!(event_type.custom_code(), None);
}

#[test]
fn test_event_type_updated() {
    let event_type = EventType::Updated;
    assert!(!event_type.is_created());
    assert!(event_type.is_updated());
    assert!(!event_type.is_deleted());
    assert!(!event_type.is_custom());
}

#[test]
fn test_event_type_deleted() {
    let event_type = EventType::Deleted;
    assert!(!event_type.is_created());
    assert!(!event_type.is_updated());
    assert!(event_type.is_deleted());
    assert!(!event_type.is_custom());
}

#[test]
fn test_event_type_custom() {
    let event_type = EventType::Custom(42);
    assert!(!event_type.is_created());
    assert!(!event_type.is_updated());
    assert!(!event_type.is_deleted());
    assert!(event_type.is_custom());
    assert_eq!(event_type.custom_code(), Some(42));
}

#[test]
fn test_event_type_display() {
    assert_eq!(format!("{}", EventType::Created), "Created");
    assert_eq!(format!("{}", EventType::Updated), "Updated");
    assert_eq!(format!("{}", EventType::Deleted), "Deleted");
    assert_eq!(format!("{}", EventType::Custom(100)), "Custom(100)");
}

#[test]
fn test_event_type_equality() {
    assert_eq!(EventType::Created, EventType::Created);
    assert_ne!(EventType::Created, EventType::Updated);
    assert_eq!(EventType::Custom(1), EventType::Custom(1));
    assert_ne!(EventType::Custom(1), EventType::Custom(2));
}

#[test]
fn test_event_type_clone() {
    let original = EventType::Custom(99);
    let cloned = original;
    assert_eq!(original, cloned);
}

// ==================== Event Tests ====================

#[test]
fn test_event_new() {
    let data = TestData::new(1, "test");
    let event = Event::new(EventType::Created, data.clone());

    assert!(!event.id.is_empty());
    assert_eq!(event.event_type, EventType::Created);
    assert_eq!(event.data, data);
    assert!(event.timestamp > 0);
    assert!(event.source.is_none());
    assert!(event.correlation_id.is_none());
}

#[test]
fn test_event_created() {
    let data = TestData::new(1, "created");
    let event = Event::created(data.clone());

    assert_eq!(event.event_type, EventType::Created);
    assert_eq!(event.data, data);
}

#[test]
fn test_event_updated() {
    let data = TestData::new(2, "updated");
    let event = Event::updated(data.clone());

    assert_eq!(event.event_type, EventType::Updated);
    assert_eq!(event.data, data);
}

#[test]
fn test_event_deleted() {
    let data = TestData::new(3, "deleted");
    let event = Event::deleted(data.clone());

    assert_eq!(event.event_type, EventType::Deleted);
    assert_eq!(event.data, data);
}

#[test]
fn test_event_custom() {
    let data = TestData::new(4, "custom");
    let event = Event::custom(42, data.clone());

    assert_eq!(event.event_type, EventType::Custom(42));
    assert_eq!(event.data, data);
}

#[test]
fn test_event_with_source() {
    let data = TestData::new(1, "test");
    let event = Event::created(data).with_source("test-component");

    assert_eq!(event.source, Some("test-component".to_string()));
}

#[test]
fn test_event_with_correlation_id() {
    let data = TestData::new(1, "test");
    let event = Event::created(data).with_correlation_id("corr-123");

    assert_eq!(event.correlation_id, Some("corr-123".to_string()));
}

#[test]
fn test_event_builder_chain() {
    let data = TestData::new(1, "test");
    let event = Event::created(data)
        .with_source("component-a")
        .with_correlation_id("trace-456");

    assert_eq!(event.source, Some("component-a".to_string()));
    assert_eq!(event.correlation_id, Some("trace-456".to_string()));
}

#[test]
fn test_event_is_type() {
    let event = Event::created(TestData::new(1, "test"));

    assert!(event.is_type(EventType::Created));
    assert!(!event.is_type(EventType::Updated));
    assert!(!event.is_type(EventType::Deleted));
}

#[test]
fn test_event_unique_ids() {
    let data = TestData::new(1, "test");
    let event1 = Event::created(data.clone());
    let event2 = Event::created(data);

    assert_ne!(event1.id, event2.id);
}

#[test]
fn test_event_clone() {
    let data = TestData::new(1, "test");
    let event = Event::created(data).with_source("src");
    let cloned = event.clone();

    assert_eq!(event.id, cloned.id);
    assert_eq!(event.event_type, cloned.event_type);
    assert_eq!(event.data, cloned.data);
    assert_eq!(event.source, cloned.source);
}

// ==================== SubscriptionHandle Tests ====================

#[test]
fn test_subscription_handle_new() {
    let handle = SubscriptionHandle::new();

    assert!(!handle.id.is_empty());
    assert!(handle.is_active());
}

#[test]
fn test_subscription_handle_cancel() {
    let handle = SubscriptionHandle::new();
    assert!(handle.is_active());

    handle.cancel();
    assert!(!handle.is_active());
}

#[test]
fn test_subscription_handle_default() {
    let handle = SubscriptionHandle::default();
    assert!(handle.is_active());
}

#[test]
fn test_subscription_handle_unique_ids() {
    let handle1 = SubscriptionHandle::new();
    let handle2 = SubscriptionHandle::new();

    assert_ne!(handle1.id, handle2.id);
}

// ==================== EventBroker Creation Tests ====================

#[test]
fn test_broker_new() {
    let broker = EventBroker::<TestData>::new();
    assert_eq!(broker.subscriber_count(), 0);
    assert_eq!(broker.events_published(), 0);
    assert_eq!(broker.events_dropped(), 0);
}

#[test]
fn test_broker_with_capacity() {
    let broker = EventBroker::<TestData>::with_capacity(100);
    assert_eq!(broker.subscriber_count(), 0);
}

#[test]
fn test_broker_with_config() {
    let config = EventBrokerConfig {
        channel_capacity: 512,
        log_dropped_events: false,
    };
    let broker = EventBroker::<TestData>::with_config(config);
    assert_eq!(broker.subscriber_count(), 0);
}

#[test]
fn test_broker_default() {
    let broker = EventBroker::<TestData>::default();
    assert_eq!(broker.subscriber_count(), 0);
}

#[test]
fn test_broker_debug() {
    let broker = EventBroker::<TestData>::new();
    let debug_str = format!("{:?}", broker);
    assert!(debug_str.contains("EventBroker"));
    assert!(debug_str.contains("subscriber_count"));
}

// ==================== EventBroker Subscribe Tests ====================

#[test]
fn test_broker_subscribe() {
    let broker = EventBroker::<TestData>::new();
    let (handle, _rx) = broker.subscribe();

    assert!(handle.is_active());
    assert_eq!(broker.subscriber_count(), 1);
}

#[test]
fn test_broker_subscribe_multiple() {
    let broker = EventBroker::<TestData>::new();

    let (_h1, _r1) = broker.subscribe();
    let (_h2, _r2) = broker.subscribe();
    let (_h3, _r3) = broker.subscribe();

    assert_eq!(broker.subscriber_count(), 3);
}

#[test]
fn test_broker_subscribe_with_capacity() {
    let broker = EventBroker::<TestData>::new();
    let (handle, _rx) = broker.subscribe_with_capacity(1024);

    assert!(handle.is_active());
    assert_eq!(broker.subscriber_count(), 1);
}

#[test]
fn test_broker_has_subscribers() {
    let broker = EventBroker::<TestData>::new();
    assert!(!broker.has_subscribers());

    let (_handle, _rx) = broker.subscribe();
    assert!(broker.has_subscribers());
}

// ==================== EventBroker Unsubscribe Tests ====================

#[test]
fn test_broker_unsubscribe() {
    let broker = EventBroker::<TestData>::new();
    let (handle, _rx) = broker.subscribe();

    assert_eq!(broker.subscriber_count(), 1);

    let removed = broker.unsubscribe(&handle);
    assert!(removed);
    assert!(!handle.is_active());
    assert_eq!(broker.subscriber_count(), 0);
}

#[test]
fn test_broker_unsubscribe_by_id() {
    let broker = EventBroker::<TestData>::new();
    let (handle, _rx) = broker.subscribe();
    let id = handle.id.clone();

    assert_eq!(broker.subscriber_count(), 1);

    let removed = broker.unsubscribe_by_id(&id);
    assert!(removed);
    assert_eq!(broker.subscriber_count(), 0);
}

#[test]
fn test_broker_unsubscribe_unknown() {
    let broker = EventBroker::<TestData>::new();
    let handle = SubscriptionHandle::new();

    let removed = broker.unsubscribe(&handle);
    assert!(!removed);
}

#[test]
fn test_broker_unsubscribe_by_unknown_id() {
    let broker = EventBroker::<TestData>::new();

    let removed = broker.unsubscribe_by_id("unknown-id");
    assert!(!removed);
}

#[test]
fn test_broker_clear() {
    let broker = EventBroker::<TestData>::new();

    let (h1, _r1) = broker.subscribe();
    let (h2, _r2) = broker.subscribe();

    assert_eq!(broker.subscriber_count(), 2);

    broker.clear();

    assert_eq!(broker.subscriber_count(), 0);
    assert!(!h1.is_active());
    assert!(!h2.is_active());
}

// ==================== EventBroker Publish Tests ====================

#[tokio::test]
async fn test_broker_publish_no_subscribers() {
    let broker = EventBroker::<TestData>::new();
    let event = Event::created(TestData::new(1, "test"));

    let delivered = broker.publish(event).await;

    assert_eq!(delivered, 0);
    assert_eq!(broker.events_published(), 1);
}

#[tokio::test]
async fn test_broker_publish_single_subscriber() {
    let broker = EventBroker::<TestData>::new();
    let (_handle, mut rx) = broker.subscribe();

    let data = TestData::new(1, "hello");
    let event = Event::created(data.clone());
    let event_id = event.id.clone();

    let delivered = broker.publish(event).await;
    assert_eq!(delivered, 1);

    let received = rx.recv().await.unwrap();
    assert_eq!(received.id, event_id);
    assert_eq!(received.data, data);
}

#[tokio::test]
async fn test_broker_publish_multiple_subscribers() {
    let broker = EventBroker::<TestData>::new();

    let (_h1, mut r1) = broker.subscribe();
    let (_h2, mut r2) = broker.subscribe();
    let (_h3, mut r3) = broker.subscribe();

    let data = TestData::new(1, "broadcast");
    let event = Event::created(data.clone());

    let delivered = broker.publish(event).await;
    assert_eq!(delivered, 3);

    let e1 = r1.recv().await.unwrap();
    let e2 = r2.recv().await.unwrap();
    let e3 = r3.recv().await.unwrap();

    assert_eq!(e1.data, data);
    assert_eq!(e2.data, data);
    assert_eq!(e3.data, data);
}

#[tokio::test]
async fn test_broker_publish_multiple_events() {
    let broker = EventBroker::<TestData>::new();
    let (_handle, mut rx) = broker.subscribe();

    for i in 0..5 {
        let event = Event::created(TestData::new(i, &format!("event-{}", i)));
        broker.publish(event).await;
    }

    assert_eq!(broker.events_published(), 5);

    for i in 0..5 {
        let received = rx.recv().await.unwrap();
        assert_eq!(received.data.id, i);
    }
}

#[tokio::test]
async fn test_broker_publish_blocking() {
    let broker = EventBroker::<TestData>::new();
    let (_handle, mut rx) = broker.subscribe();

    let data = TestData::new(1, "blocking");
    let event = Event::created(data.clone());

    let delivered = broker.publish_blocking(event).await;
    assert_eq!(delivered, 1);

    let received = rx.recv().await.unwrap();
    assert_eq!(received.data, data);
}

#[tokio::test]
async fn test_broker_publish_after_unsubscribe() {
    let broker = EventBroker::<TestData>::new();
    let (handle, _rx) = broker.subscribe();

    broker.unsubscribe(&handle);

    let event = Event::created(TestData::new(1, "test"));
    let delivered = broker.publish(event).await;

    assert_eq!(delivered, 0);
}

// ==================== Non-blocking Delivery Tests ====================

#[tokio::test]
async fn test_broker_non_blocking_slow_consumer() {
    // Create broker with very small capacity
    let broker = EventBroker::<TestData>::with_capacity(2);
    let (_handle, _rx) = broker.subscribe(); // Don't read from rx

    // Publish more events than capacity
    for i in 0..10 {
        let event = Event::created(TestData::new(i, "overflow"));
        broker.publish(event).await;
    }

    // Should have dropped some events
    assert!(broker.events_dropped() > 0);
    assert_eq!(broker.events_published(), 10);
}

#[tokio::test]
async fn test_broker_fast_consumer_no_drops() {
    let broker = EventBroker::<TestData>::with_capacity(10);
    let (_handle, mut rx) = broker.subscribe();

    // Spawn consumer
    let consumer = tokio::spawn(async move {
        let mut count = 0;
        while let Ok(Some(_)) = timeout(Duration::from_millis(100), rx.recv()).await {
            count += 1;
        }
        count
    });

    // Publish events
    for i in 0..5 {
        let event = Event::created(TestData::new(i, "fast"));
        broker.publish(event).await;
        tokio::time::sleep(Duration::from_millis(10)).await;
    }

    // Wait for consumer
    tokio::time::sleep(Duration::from_millis(200)).await;
    let received = consumer.await.unwrap();

    assert_eq!(received, 5);
    assert_eq!(broker.events_dropped(), 0);
}

// ==================== Closed Channel Tests ====================

#[tokio::test]
async fn test_broker_removes_closed_channels() {
    let broker = EventBroker::<TestData>::new();
    let (_handle, rx) = broker.subscribe();

    assert_eq!(broker.subscriber_count(), 1);

    // Drop the receiver to close the channel
    drop(rx);

    // Publish should detect closed channel and remove subscriber
    let event = Event::created(TestData::new(1, "test"));
    let delivered = broker.publish(event).await;

    assert_eq!(delivered, 0);
    assert_eq!(broker.subscriber_count(), 0);
}

// ==================== Stats Tests ====================

#[tokio::test]
async fn test_broker_stats() {
    let broker = EventBroker::<TestData>::with_capacity(100);

    let (_h1, _r1) = broker.subscribe();
    let (_h2, _r2) = broker.subscribe();

    for i in 0..5 {
        let event = Event::created(TestData::new(i, "stats"));
        broker.publish(event).await;
    }

    let stats = broker.stats();

    assert_eq!(stats.subscriber_count, 2);
    assert_eq!(stats.events_published, 5);
    assert_eq!(stats.events_dropped, 0);
    assert_eq!(stats.channel_capacity, 100);
}

// ==================== Concurrent Tests ====================

#[tokio::test]
async fn test_broker_concurrent_subscribe() {
    let broker = Arc::new(EventBroker::<TestData>::new());
    let mut handles = vec![];

    for _ in 0..10 {
        let broker_clone = broker.clone();
        let handle = tokio::spawn(async move {
            let (_h, _r) = broker_clone.subscribe();
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.await.unwrap();
    }

    assert_eq!(broker.subscriber_count(), 10);
}

#[tokio::test]
async fn test_broker_concurrent_publish() {
    let broker = Arc::new(EventBroker::<TestData>::new());
    let (_handle, mut rx) = broker.subscribe();

    let received_count = Arc::new(AtomicU32::new(0));
    let received_count_clone = received_count.clone();

    // Spawn consumer
    let consumer = tokio::spawn(async move {
        while let Ok(Some(_)) = timeout(Duration::from_millis(500), rx.recv()).await {
            received_count_clone.fetch_add(1, Ordering::Relaxed);
        }
    });

    // Spawn multiple publishers
    let mut publisher_handles = vec![];
    for i in 0..5 {
        let broker_clone = broker.clone();
        let handle = tokio::spawn(async move {
            for j in 0..10 {
                let event = Event::created(TestData::new(i * 10 + j, "concurrent"));
                broker_clone.publish(event).await;
            }
        });
        publisher_handles.push(handle);
    }

    // Wait for publishers
    for handle in publisher_handles {
        handle.await.unwrap();
    }

    // Wait for consumer
    tokio::time::sleep(Duration::from_millis(600)).await;
    consumer.abort();

    assert_eq!(broker.events_published(), 50);
    assert!(received_count.load(Ordering::Relaxed) > 0);
}

#[tokio::test]
async fn test_broker_concurrent_subscribe_unsubscribe() {
    let broker = Arc::new(EventBroker::<TestData>::new());
    let barrier = Arc::new(Barrier::new(20));

    let mut handles = vec![];

    // 10 subscribers
    for _ in 0..10 {
        let broker_clone = broker.clone();
        let barrier_clone = barrier.clone();
        let handle = tokio::spawn(async move {
            barrier_clone.wait().await;
            let (h, _r) = broker_clone.subscribe();
            tokio::time::sleep(Duration::from_millis(50)).await;
            broker_clone.unsubscribe(&h);
        });
        handles.push(handle);
    }

    // 10 publishers
    for i in 0..10 {
        let broker_clone = broker.clone();
        let barrier_clone = barrier.clone();
        let handle = tokio::spawn(async move {
            barrier_clone.wait().await;
            for j in 0..5 {
                let event = Event::created(TestData::new(i * 5 + j, "chaos"));
                broker_clone.publish(event).await;
            }
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.await.unwrap();
    }

    // All subscribers should have unsubscribed
    assert_eq!(broker.subscriber_count(), 0);
    assert_eq!(broker.events_published(), 50);
}

#[tokio::test]
async fn test_broker_concurrent_clear() {
    let broker = Arc::new(EventBroker::<TestData>::new());

    // Add subscribers
    for _ in 0..5 {
        broker.subscribe();
    }

    let broker_clone = broker.clone();
    let clear_handle = tokio::spawn(async move {
        tokio::time::sleep(Duration::from_millis(10)).await;
        broker_clone.clear();
    });

    let broker_clone2 = broker.clone();
    let publish_handle = tokio::spawn(async move {
        for i in 0..20 {
            let event = Event::created(TestData::new(i, "during-clear"));
            broker_clone2.publish(event).await;
            tokio::time::sleep(Duration::from_millis(5)).await;
        }
    });

    clear_handle.await.unwrap();
    publish_handle.await.unwrap();

    // After clear, no subscribers
    assert_eq!(broker.subscriber_count(), 0);
}

// ==================== Subscriber Trait Tests ====================

struct CountingSubscriber {
    count: AtomicU32,
}

impl CountingSubscriber {
    fn new() -> Self {
        Self {
            count: AtomicU32::new(0),
        }
    }

    fn count(&self) -> u32 {
        self.count.load(Ordering::Relaxed)
    }
}

#[async_trait::async_trait]
impl Subscriber<TestData> for CountingSubscriber {
    async fn on_event(&self, _event: Event<TestData>) {
        self.count.fetch_add(1, Ordering::Relaxed);
    }
}

struct FilteringSubscriber {
    filter_id: u64,
    count: AtomicU32,
}

impl FilteringSubscriber {
    fn new(filter_id: u64) -> Self {
        Self {
            filter_id,
            count: AtomicU32::new(0),
        }
    }

    fn count(&self) -> u32 {
        self.count.load(Ordering::Relaxed)
    }
}

#[async_trait::async_trait]
impl Subscriber<TestData> for FilteringSubscriber {
    async fn on_event(&self, _event: Event<TestData>) {
        self.count.fetch_add(1, Ordering::Relaxed);
    }

    fn should_receive(&self, event: &Event<TestData>) -> bool {
        event.data.id == self.filter_id
    }
}

#[tokio::test]
async fn test_subscriber_trait_counting() {
    let subscriber = Arc::new(CountingSubscriber::new());

    let event1 = Event::created(TestData::new(1, "a"));
    let event2 = Event::created(TestData::new(2, "b"));

    subscriber.on_event(event1).await;
    subscriber.on_event(event2).await;

    assert_eq!(subscriber.count(), 2);
}

#[tokio::test]
async fn test_subscriber_trait_filtering() {
    let subscriber = FilteringSubscriber::new(42);

    let event1 = Event::created(TestData::new(1, "no"));
    let event2 = Event::created(TestData::new(42, "yes"));
    let event3 = Event::created(TestData::new(100, "no"));

    assert!(!subscriber.should_receive(&event1));
    assert!(subscriber.should_receive(&event2));
    assert!(!subscriber.should_receive(&event3));
    assert_eq!(subscriber.count(), 0);
}

// ==================== Edge Cases ====================

#[tokio::test]
async fn test_broker_empty_string_data() {
    let broker = EventBroker::<String>::new();
    let (_handle, mut rx) = broker.subscribe();

    let event = Event::created(String::new());
    broker.publish(event).await;

    let received = rx.recv().await.unwrap();
    assert_eq!(received.data, "");
}

#[tokio::test]
async fn test_broker_large_data() {
    let broker = EventBroker::<Vec<u8>>::new();
    let (_handle, mut rx) = broker.subscribe();

    let large_data: Vec<u8> = vec![0u8; 1024 * 1024]; // 1MB
    let event = Event::created(large_data.clone());

    broker.publish(event).await;

    let received = rx.recv().await.unwrap();
    assert_eq!(received.data.len(), 1024 * 1024);
}

#[tokio::test]
async fn test_broker_zero_capacity() {
    // Zero capacity should still work (becomes 1 internally in tokio)
    let broker = EventBroker::<TestData>::with_capacity(0);
    let (_handle, mut rx) = broker.subscribe();

    let event = Event::created(TestData::new(1, "zero"));
    broker.publish(event).await;

    // Should still be able to receive
    let result = timeout(Duration::from_millis(100), rx.recv()).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_broker_rapid_subscribe_unsubscribe() {
    let broker = EventBroker::<TestData>::new();

    for _ in 0..100 {
        let (handle, _rx) = broker.subscribe();
        broker.unsubscribe(&handle);
    }

    assert_eq!(broker.subscriber_count(), 0);
}

#[tokio::test]
async fn test_broker_publish_different_event_types() {
    let broker = EventBroker::<TestData>::new();
    let (_handle, mut rx) = broker.subscribe();

    broker
        .publish(Event::created(TestData::new(1, "created")))
        .await;
    broker
        .publish(Event::updated(TestData::new(2, "updated")))
        .await;
    broker
        .publish(Event::deleted(TestData::new(3, "deleted")))
        .await;
    broker
        .publish(Event::custom(99, TestData::new(4, "custom")))
        .await;

    let e1 = rx.recv().await.unwrap();
    let e2 = rx.recv().await.unwrap();
    let e3 = rx.recv().await.unwrap();
    let e4 = rx.recv().await.unwrap();

    assert_eq!(e1.event_type, EventType::Created);
    assert_eq!(e2.event_type, EventType::Updated);
    assert_eq!(e3.event_type, EventType::Deleted);
    assert_eq!(e4.event_type, EventType::Custom(99));
}

// ==================== Config Tests ====================

#[test]
fn test_broker_config_default() {
    let config = EventBrokerConfig::default();
    assert_eq!(config.channel_capacity, 256);
    assert!(config.log_dropped_events);
}

#[test]
fn test_broker_config_with_capacity() {
    let config = EventBrokerConfig::with_capacity(1024);
    assert_eq!(config.channel_capacity, 1024);
    assert!(config.log_dropped_events);
}

#[test]
fn test_broker_config_clone() {
    let config = EventBrokerConfig {
        channel_capacity: 512,
        log_dropped_events: false,
    };
    let cloned = config.clone();

    assert_eq!(cloned.channel_capacity, 512);
    assert!(!cloned.log_dropped_events);
}
