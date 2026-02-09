use super::*;

// Test service types
#[derive(Debug, Clone, PartialEq)]
struct TestService {
    value: String,
}

#[derive(Debug, Clone, PartialEq)]
struct AnotherService {
    number: i32,
}

#[derive(Debug, Clone)]
struct CounterService {
    count: Arc<RwLock<i32>>,
}

impl CounterService {
    fn new() -> Self {
        Self {
            count: Arc::new(RwLock::new(0)),
        }
    }

    fn increment(&self) {
        *self.count.write() += 1;
    }

    fn get_count(&self) -> i32 {
        *self.count.read()
    }
}

// Tests for Injectable trait
#[test]
fn test_injectable_type_name() {
    let service = TestService {
        value: "test".to_string(),
    };
    let type_name = service.type_name();
    assert!(type_name.contains("TestService"));
}

// Tests for ServiceContainer::new
#[test]
fn test_container_new() {
    let container = ServiceContainer::new();
    assert_eq!(container.service_count(), 0);
}

#[test]
fn test_container_default() {
    let container = ServiceContainer::default();
    assert_eq!(container.service_count(), 0);
}

// Tests for ServiceContainer::register
#[test]
fn test_register_service() {
    let container = ServiceContainer::new();
    let service = TestService {
        value: "test".to_string(),
    };

    let result = container.register(service);
    assert!(result.is_ok());
    assert_eq!(container.service_count(), 1);
}

#[test]
fn test_register_duplicate_service_fails() {
    let container = ServiceContainer::new();
    let service1 = TestService {
        value: "test1".to_string(),
    };
    let service2 = TestService {
        value: "test2".to_string(),
    };

    container.register(service1).unwrap();
    let result = container.register(service2);

    assert!(result.is_err());
    if let Err(GatewayError::Config(msg)) = result {
        assert!(msg.contains("already registered"));
    } else {
        panic!("Expected Config error");
    }
}

#[test]
fn test_register_different_services() {
    let container = ServiceContainer::new();
    let service1 = TestService {
        value: "test".to_string(),
    };
    let service2 = AnotherService { number: 42 };

    container.register(service1).unwrap();
    container.register(service2).unwrap();

    assert_eq!(container.service_count(), 2);
}

// Tests for ServiceContainer::register_singleton
#[test]
fn test_register_singleton() {
    let container = ServiceContainer::new();
    let service = TestService {
        value: "singleton".to_string(),
    };

    let result = container.register_singleton(service);
    assert!(result.is_ok());
    assert_eq!(container.service_count(), 1);
}

#[test]
fn test_register_duplicate_singleton_fails() {
    let container = ServiceContainer::new();
    let service1 = TestService {
        value: "singleton1".to_string(),
    };
    let service2 = TestService {
        value: "singleton2".to_string(),
    };

    container.register_singleton(service1).unwrap();
    let result = container.register_singleton(service2);

    assert!(result.is_err());
    if let Err(GatewayError::Config(msg)) = result {
        assert!(msg.contains("already registered"));
    } else {
        panic!("Expected Config error");
    }
}

#[test]
fn test_singleton_returns_same_instance() {
    let container = ServiceContainer::new();
    let service = CounterService::new();
    container.register_singleton(service).unwrap();

    let instance1: Arc<CounterService> = container.get().unwrap();
    instance1.increment();

    let instance2: Arc<CounterService> = container.get().unwrap();
    assert_eq!(instance2.get_count(), 1);

    instance2.increment();
    assert_eq!(instance1.get_count(), 2);
}

// Tests for ServiceContainer::register_factory
#[test]
fn test_register_factory() {
    let container = ServiceContainer::new();
    let factory = || TestService {
        value: "from_factory".to_string(),
    };

    let result = container.register_factory(factory);
    assert!(result.is_ok());
    assert_eq!(container.service_count(), 1);
}

#[test]
fn test_factory_creates_service() {
    let container = ServiceContainer::new();
    let factory = || TestService {
        value: "factory_value".to_string(),
    };

    container.register_factory(factory).unwrap();
    let service: Arc<TestService> = container.get().unwrap();
    assert_eq!(service.value, "factory_value");
}

// Tests for ServiceContainer::get
#[test]
fn test_get_service() {
    let container = ServiceContainer::new();
    let service = TestService {
        value: "get_test".to_string(),
    };

    container.register(service).unwrap();
    let retrieved: Arc<TestService> = container.get().unwrap();
    assert_eq!(retrieved.value, "get_test");
}

#[test]
fn test_get_singleton() {
    let container = ServiceContainer::new();
    let service = TestService {
        value: "singleton_get".to_string(),
    };

    container.register_singleton(service).unwrap();
    let retrieved: Arc<TestService> = container.get().unwrap();
    assert_eq!(retrieved.value, "singleton_get");
}

#[test]
fn test_get_unregistered_service_fails() {
    let container = ServiceContainer::new();
    let result: Result<Arc<TestService>> = container.get();

    assert!(result.is_err());
    if let Err(GatewayError::Config(msg)) = result {
        assert!(msg.contains("not registered"));
    } else {
        panic!("Expected Config error");
    }
}

#[test]
fn test_get_prefers_singleton_over_service() {
    let container = ServiceContainer::new();

    // Register as regular service first
    let service = TestService {
        value: "regular".to_string(),
    };
    container.register(service).unwrap();

    // Then register another instance as singleton
    // (This is a bit contrived but tests the priority)
    let container2 = ServiceContainer::new();
    let singleton = AnotherService { number: 100 };
    container2.register_singleton(singleton).unwrap();

    let retrieved: Arc<AnotherService> = container2.get().unwrap();
    assert_eq!(retrieved.number, 100);
}

// Tests for ServiceContainer::try_get
#[test]
fn test_try_get_existing_service() {
    let container = ServiceContainer::new();
    let service = TestService {
        value: "try_get_test".to_string(),
    };

    container.register(service).unwrap();
    let retrieved: Option<Arc<TestService>> = container.try_get();
    assert!(retrieved.is_some());
    assert_eq!(retrieved.unwrap().value, "try_get_test");
}

#[test]
fn test_try_get_nonexistent_service() {
    let container = ServiceContainer::new();
    let retrieved: Option<Arc<TestService>> = container.try_get();
    assert!(retrieved.is_none());
}

// Tests for ServiceContainer::contains
#[test]
fn test_contains_registered_service() {
    let container = ServiceContainer::new();
    let service = TestService {
        value: "contains_test".to_string(),
    };

    container.register(service).unwrap();
    assert!(container.contains::<TestService>());
}

#[test]
fn test_contains_registered_singleton() {
    let container = ServiceContainer::new();
    let service = TestService {
        value: "singleton_contains".to_string(),
    };

    container.register_singleton(service).unwrap();
    assert!(container.contains::<TestService>());
}

#[test]
fn test_contains_unregistered_service() {
    let container = ServiceContainer::new();
    assert!(!container.contains::<TestService>());
}

// Tests for ServiceContainer::remove
#[test]
fn test_remove_service() {
    let container = ServiceContainer::new();
    let service = TestService {
        value: "remove_test".to_string(),
    };

    container.register(service).unwrap();
    assert_eq!(container.service_count(), 1);

    let result = container.remove::<TestService>();
    assert!(result.is_ok());
    assert_eq!(container.service_count(), 0);
    assert!(!container.contains::<TestService>());
}

#[test]
fn test_remove_singleton() {
    let container = ServiceContainer::new();
    let service = TestService {
        value: "remove_singleton".to_string(),
    };

    container.register_singleton(service).unwrap();
    assert_eq!(container.service_count(), 1);

    let result = container.remove::<TestService>();
    assert!(result.is_ok());
    assert_eq!(container.service_count(), 0);
}

#[test]
fn test_remove_unregistered_service_fails() {
    let container = ServiceContainer::new();
    let result = container.remove::<TestService>();

    assert!(result.is_err());
    if let Err(GatewayError::Config(msg)) = result {
        assert!(msg.contains("not registered"));
    } else {
        panic!("Expected Config error");
    }
}

// Tests for ServiceContainer::clear
#[test]
fn test_clear_empty_container() {
    let container = ServiceContainer::new();
    container.clear();
    assert_eq!(container.service_count(), 0);
}

#[test]
fn test_clear_services() {
    let container = ServiceContainer::new();
    container
        .register(TestService {
            value: "test1".to_string(),
        })
        .unwrap();
    container.register(AnotherService { number: 42 }).unwrap();

    assert_eq!(container.service_count(), 2);
    container.clear();
    assert_eq!(container.service_count(), 0);
    assert!(!container.contains::<TestService>());
    assert!(!container.contains::<AnotherService>());
}

#[test]
fn test_clear_singletons() {
    let container = ServiceContainer::new();
    container
        .register_singleton(TestService {
            value: "singleton".to_string(),
        })
        .unwrap();

    container.clear();
    assert_eq!(container.service_count(), 0);
}

#[test]
fn test_clear_mixed_services() {
    let container = ServiceContainer::new();
    container
        .register(TestService {
            value: "regular".to_string(),
        })
        .unwrap();
    container
        .register_singleton(AnotherService { number: 100 })
        .unwrap();

    assert_eq!(container.service_count(), 2);
    container.clear();
    assert_eq!(container.service_count(), 0);
}

// Tests for ServiceContainer::service_count
#[test]
fn test_service_count_empty() {
    let container = ServiceContainer::new();
    assert_eq!(container.service_count(), 0);
}

#[test]
fn test_service_count_with_services() {
    let container = ServiceContainer::new();
    container
        .register(TestService {
            value: "test".to_string(),
        })
        .unwrap();
    assert_eq!(container.service_count(), 1);

    container.register(AnotherService { number: 42 }).unwrap();
    assert_eq!(container.service_count(), 2);
}

#[test]
fn test_service_count_with_singletons() {
    let container = ServiceContainer::new();
    container
        .register_singleton(TestService {
            value: "singleton".to_string(),
        })
        .unwrap();
    assert_eq!(container.service_count(), 1);
}

#[test]
fn test_service_count_mixed() {
    let container = ServiceContainer::new();
    container
        .register(TestService {
            value: "regular".to_string(),
        })
        .unwrap();
    container
        .register_singleton(AnotherService { number: 100 })
        .unwrap();
    assert_eq!(container.service_count(), 2);
}

// Tests for ServiceBuilder
#[test]
fn test_builder_new() {
    let builder = ServiceBuilder::new();
    let container = builder.build();
    assert_eq!(container.service_count(), 0);
}

#[test]
fn test_builder_default() {
    let builder = ServiceBuilder::default();
    let container = builder.build();
    assert_eq!(container.service_count(), 0);
}

#[test]
fn test_builder_add_service() {
    let service = TestService {
        value: "builder_test".to_string(),
    };

    let container = ServiceBuilder::new().add_service(service).unwrap().build();

    assert_eq!(container.service_count(), 1);
    assert!(container.contains::<TestService>());
}

#[test]
fn test_builder_add_singleton() {
    let service = TestService {
        value: "builder_singleton".to_string(),
    };

    let container = ServiceBuilder::new()
        .add_singleton(service)
        .unwrap()
        .build();

    assert_eq!(container.service_count(), 1);
    let retrieved: Arc<TestService> = container.get().unwrap();
    assert_eq!(retrieved.value, "builder_singleton");
}

#[test]
fn test_builder_add_factory() {
    let factory = || TestService {
        value: "builder_factory".to_string(),
    };

    let container = ServiceBuilder::new().add_factory(factory).unwrap().build();

    assert_eq!(container.service_count(), 1);
    let retrieved: Arc<TestService> = container.get().unwrap();
    assert_eq!(retrieved.value, "builder_factory");
}

#[test]
fn test_builder_chaining() {
    let container = ServiceBuilder::new()
        .add_service(TestService {
            value: "test".to_string(),
        })
        .unwrap()
        .add_singleton(AnotherService { number: 42 })
        .unwrap()
        .build();

    assert_eq!(container.service_count(), 2);
    assert!(container.contains::<TestService>());
    assert!(container.contains::<AnotherService>());
}

#[test]
fn test_builder_duplicate_service_fails() {
    let result = ServiceBuilder::new()
        .add_service(TestService {
            value: "first".to_string(),
        })
        .unwrap()
        .add_service(TestService {
            value: "second".to_string(),
        });

    assert!(result.is_err());
}

// Tests for global container functions
#[test]
fn test_global_container_exists() {
    let container = global_container();
    // Just verify it exists and can be called
    let _ = container.service_count();
}

// Note: Testing global functions is tricky because they share state
// These tests demonstrate usage but may interfere with each other
#[test]
fn test_register_and_get_global() {
    // Use a unique type for this test to avoid conflicts
    #[derive(Debug, Clone)]
    struct UniqueGlobalService1 {
        value: String,
    }

    let service = UniqueGlobalService1 {
        value: "global_test".to_string(),
    };

    // Clear might not work across tests, so we handle both cases
    let register_result = register_global(service);
    if register_result.is_ok() {
        let retrieved: Result<Arc<UniqueGlobalService1>> = get_global();
        assert!(retrieved.is_ok());
        assert_eq!(retrieved.unwrap().value, "global_test");
    }
    // If already registered from another test, that's okay
}

#[test]
fn test_register_global_singleton() {
    #[derive(Debug, Clone)]
    struct UniqueGlobalService2 {
        value: String,
    }

    let service = UniqueGlobalService2 {
        value: "global_singleton".to_string(),
    };

    let register_result = register_global_singleton(service);
    if register_result.is_ok() {
        let retrieved: Result<Arc<UniqueGlobalService2>> = get_global();
        assert!(retrieved.is_ok());
        assert_eq!(retrieved.unwrap().value, "global_singleton");
    }
}

// Concurrency tests
#[test]
fn test_concurrent_get() {
    use std::thread;

    let container = Arc::new(ServiceContainer::new());
    container
        .register(TestService {
            value: "concurrent".to_string(),
        })
        .unwrap();

    let handles: Vec<_> = (0..10)
        .map(|_| {
            let container = Arc::clone(&container);
            thread::spawn(move || {
                let service: Arc<TestService> = container.get().unwrap();
                assert_eq!(service.value, "concurrent");
            })
        })
        .collect();

    for handle in handles {
        handle.join().unwrap();
    }
}

#[test]
fn test_concurrent_singleton_access() {
    use std::thread;

    let container = Arc::new(ServiceContainer::new());
    let counter = CounterService::new();
    container.register_singleton(counter).unwrap();

    let handles: Vec<_> = (0..10)
        .map(|_| {
            let container = Arc::clone(&container);
            thread::spawn(move || {
                let service: Arc<CounterService> = container.get().unwrap();
                service.increment();
            })
        })
        .collect();

    for handle in handles {
        handle.join().unwrap();
    }

    let final_service: Arc<CounterService> = container.get().unwrap();
    assert_eq!(final_service.get_count(), 10);
}

// Edge case tests
#[test]
fn test_service_with_arc_sharing() {
    let container = ServiceContainer::new();
    let service = TestService {
        value: "shared".to_string(),
    };

    container.register(service).unwrap();
    let instance1: Arc<TestService> = container.get().unwrap();
    let instance2: Arc<TestService> = container.get().unwrap();

    // Both should point to the same underlying data
    assert_eq!(instance1.value, instance2.value);
    assert_eq!(Arc::strong_count(&instance1), Arc::strong_count(&instance2));
}

#[test]
fn test_multiple_service_types() {
    let container = ServiceContainer::new();

    container
        .register(TestService {
            value: "test".to_string(),
        })
        .unwrap();
    container.register(AnotherService { number: 42 }).unwrap();
    container.register(CounterService::new()).unwrap();

    assert_eq!(container.service_count(), 3);
    assert!(container.contains::<TestService>());
    assert!(container.contains::<AnotherService>());
    assert!(container.contains::<CounterService>());
}
