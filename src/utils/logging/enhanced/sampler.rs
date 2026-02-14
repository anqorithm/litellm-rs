//! Log sampling manager for high-frequency events

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};

/// Log sampling manager for high-frequency events
#[allow(dead_code)]
pub struct LogSampler {
    sample_rates: HashMap<String, f64>,
    counters: HashMap<String, AtomicU64>,
}

#[allow(dead_code)]
impl Default for LogSampler {
    fn default() -> Self {
        Self::new()
    }
}

impl LogSampler {
    /// Create a new log sampler
    pub fn new() -> Self {
        Self {
            sample_rates: HashMap::new(),
            counters: HashMap::new(),
        }
    }

    /// Configure sampling rate for a log category
    #[allow(dead_code)]
    pub fn set_sample_rate(&mut self, category: &str, rate: f64) {
        self.sample_rates
            .insert(category.to_string(), rate.clamp(0.0, 1.0));
        self.counters
            .insert(category.to_string(), AtomicU64::new(0));
    }

    /// Check if a log should be sampled
    #[allow(dead_code)]
    pub fn should_log(&self, category: &str) -> bool {
        if let Some(&rate) = self.sample_rates.get(category) {
            if rate >= 1.0 {
                return true;
            }
            if rate <= 0.0 {
                return false;
            }

            if let Some(counter) = self.counters.get(category) {
                let count = counter.fetch_add(1, Ordering::Relaxed);
                let sample_threshold = (1.0 / rate) as u64;
                count % sample_threshold == 0
            } else {
                true
            }
        } else {
            true
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ==================== Construction Tests ====================

    #[test]
    fn test_log_sampler_new() {
        let sampler = LogSampler::new();
        assert!(sampler.sample_rates.is_empty());
        assert!(sampler.counters.is_empty());
    }

    #[test]
    fn test_log_sampler_default() {
        let sampler = LogSampler::default();
        assert!(sampler.sample_rates.is_empty());
        assert!(sampler.counters.is_empty());
    }

    #[test]
    fn test_new_equals_default() {
        let sampler1 = LogSampler::new();
        let sampler2 = LogSampler::default();
        assert_eq!(sampler1.sample_rates.len(), sampler2.sample_rates.len());
        assert_eq!(sampler1.counters.len(), sampler2.counters.len());
    }

    // ==================== Sample Rate Configuration Tests ====================

    #[test]
    fn test_set_sample_rate_normal() {
        let mut sampler = LogSampler::new();
        sampler.set_sample_rate("requests", 0.5);

        assert_eq!(sampler.sample_rates.get("requests"), Some(&0.5));
        assert!(sampler.counters.contains_key("requests"));
    }

    #[test]
    fn test_set_sample_rate_zero() {
        let mut sampler = LogSampler::new();
        sampler.set_sample_rate("debug", 0.0);

        assert_eq!(sampler.sample_rates.get("debug"), Some(&0.0));
    }

    #[test]
    fn test_set_sample_rate_one() {
        let mut sampler = LogSampler::new();
        sampler.set_sample_rate("errors", 1.0);

        assert_eq!(sampler.sample_rates.get("errors"), Some(&1.0));
    }

    #[test]
    fn test_set_sample_rate_clamped_above_one() {
        let mut sampler = LogSampler::new();
        sampler.set_sample_rate("test", 1.5);

        // Should be clamped to 1.0
        assert_eq!(sampler.sample_rates.get("test"), Some(&1.0));
    }

    #[test]
    fn test_set_sample_rate_clamped_below_zero() {
        let mut sampler = LogSampler::new();
        sampler.set_sample_rate("test", -0.5);

        // Should be clamped to 0.0
        assert_eq!(sampler.sample_rates.get("test"), Some(&0.0));
    }

    #[test]
    fn test_set_sample_rate_multiple_categories() {
        let mut sampler = LogSampler::new();
        sampler.set_sample_rate("requests", 0.1);
        sampler.set_sample_rate("errors", 1.0);
        sampler.set_sample_rate("debug", 0.01);

        assert_eq!(sampler.sample_rates.len(), 3);
        assert_eq!(sampler.counters.len(), 3);
    }

    #[test]
    fn test_set_sample_rate_overwrite() {
        let mut sampler = LogSampler::new();
        sampler.set_sample_rate("category", 0.1);
        sampler.set_sample_rate("category", 0.9);

        assert_eq!(sampler.sample_rates.get("category"), Some(&0.9));
    }

    // ==================== should_log Tests - Unconfigured Categories ====================

    #[test]
    fn test_should_log_unconfigured_category() {
        let sampler = LogSampler::new();

        // Unconfigured categories should always log
        assert!(sampler.should_log("unknown"));
        assert!(sampler.should_log("random"));
    }

    // ==================== should_log Tests - Rate = 1.0 (100%) ====================

    #[test]
    fn test_should_log_always_at_rate_one() {
        let mut sampler = LogSampler::new();
        sampler.set_sample_rate("always", 1.0);

        // Rate 1.0 should always return true
        for _ in 0..100 {
            assert!(sampler.should_log("always"));
        }
    }

    // ==================== should_log Tests - Rate = 0.0 (0%) ====================

    #[test]
    fn test_should_log_never_at_rate_zero() {
        let mut sampler = LogSampler::new();
        sampler.set_sample_rate("never", 0.0);

        // Rate 0.0 should never return true
        for _ in 0..100 {
            assert!(!sampler.should_log("never"));
        }
    }

    // ==================== should_log Tests - Rate = 0.5 (50%) ====================

    #[test]
    fn test_should_log_at_rate_half() {
        let mut sampler = LogSampler::new();
        sampler.set_sample_rate("half", 0.5);

        // Rate 0.5 means every 2nd call
        // First call (count=0): 0 % 2 == 0, should log
        assert!(sampler.should_log("half"));
        // Second call (count=1): 1 % 2 == 1, should not log
        assert!(!sampler.should_log("half"));
        // Third call (count=2): 2 % 2 == 0, should log
        assert!(sampler.should_log("half"));
        // Fourth call (count=3): 3 % 2 == 1, should not log
        assert!(!sampler.should_log("half"));
    }

    // ==================== should_log Tests - Rate = 0.25 (25%) ====================

    #[test]
    fn test_should_log_at_rate_quarter() {
        let mut sampler = LogSampler::new();
        sampler.set_sample_rate("quarter", 0.25);

        // Rate 0.25 means every 4th call (1/0.25 = 4)
        // Call 0: 0 % 4 == 0, should log
        assert!(sampler.should_log("quarter"));
        // Call 1: 1 % 4 == 1, should not log
        assert!(!sampler.should_log("quarter"));
        // Call 2: 2 % 4 == 2, should not log
        assert!(!sampler.should_log("quarter"));
        // Call 3: 3 % 4 == 3, should not log
        assert!(!sampler.should_log("quarter"));
        // Call 4: 4 % 4 == 0, should log
        assert!(sampler.should_log("quarter"));
    }

    // ==================== should_log Tests - Rate = 0.1 (10%) ====================

    #[test]
    fn test_should_log_at_rate_tenth() {
        let mut sampler = LogSampler::new();
        sampler.set_sample_rate("tenth", 0.1);

        // Rate 0.1 means every 10th call (1/0.1 = 10)
        let mut logged_count = 0;
        for _ in 0..100 {
            if sampler.should_log("tenth") {
                logged_count += 1;
            }
        }

        // Should log approximately 10 times (actually exactly 10 due to modulo)
        assert_eq!(logged_count, 10);
    }

    // ==================== Counter Behavior Tests ====================

    #[test]
    fn test_counter_increments_on_should_log() {
        let mut sampler = LogSampler::new();
        sampler.set_sample_rate("count", 0.5);

        // Each call should increment the counter
        let _ = sampler.should_log("count");
        let _ = sampler.should_log("count");
        let _ = sampler.should_log("count");

        // Counter should be at 3 after 3 calls
        let counter = sampler.counters.get("count").unwrap();
        assert_eq!(counter.load(Ordering::Relaxed), 3);
    }

    #[test]
    fn test_counters_are_independent() {
        let mut sampler = LogSampler::new();
        sampler.set_sample_rate("cat1", 0.5);
        sampler.set_sample_rate("cat2", 0.5);

        // Call cat1 5 times
        for _ in 0..5 {
            let _ = sampler.should_log("cat1");
        }

        // Call cat2 3 times
        for _ in 0..3 {
            let _ = sampler.should_log("cat2");
        }

        assert_eq!(
            sampler
                .counters
                .get("cat1")
                .unwrap()
                .load(Ordering::Relaxed),
            5
        );
        assert_eq!(
            sampler
                .counters
                .get("cat2")
                .unwrap()
                .load(Ordering::Relaxed),
            3
        );
    }

    // ==================== Edge Case Tests ====================

    #[test]
    fn test_very_small_rate() {
        let mut sampler = LogSampler::new();
        sampler.set_sample_rate("rare", 0.001);

        // Rate 0.001 means every 1000th call
        let mut logged_count = 0;
        for _ in 0..10000 {
            if sampler.should_log("rare") {
                logged_count += 1;
            }
        }

        // Should log approximately 10 times
        assert_eq!(logged_count, 10);
    }

    #[test]
    fn test_rate_very_close_to_one() {
        let mut sampler = LogSampler::new();
        sampler.set_sample_rate("almost_always", 0.999);

        // Rate 0.999 rounds to every 1 call (1/0.999 ≈ 1.001, truncates to 1)
        // So it should always log
        let mut logged_count = 0;
        for _ in 0..100 {
            if sampler.should_log("almost_always") {
                logged_count += 1;
            }
        }

        assert_eq!(logged_count, 100);
    }

    #[test]
    fn test_rate_very_close_to_zero() {
        let mut sampler = LogSampler::new();
        sampler.set_sample_rate("almost_never", 0.001);

        // Should log very rarely
        let mut logged = false;
        for _ in 0..999 {
            if sampler.should_log("almost_never") {
                logged = true;
                break;
            }
        }

        // First log happens at call 0
        assert!(logged);
    }

    #[test]
    fn test_empty_category_name() {
        let mut sampler = LogSampler::new();
        sampler.set_sample_rate("", 0.5);

        // Empty string is a valid category name
        assert!(sampler.sample_rates.contains_key(""));
        assert!(sampler.should_log("") || !sampler.should_log(""));
    }

    #[test]
    fn test_unicode_category_name() {
        let mut sampler = LogSampler::new();
        sampler.set_sample_rate("日志", 0.5);

        assert!(sampler.sample_rates.contains_key("日志"));
    }

    #[test]
    fn test_special_characters_in_category() {
        let mut sampler = LogSampler::new();
        sampler.set_sample_rate("http.requests.slow", 0.1);
        sampler.set_sample_rate("auth/login", 0.5);
        sampler.set_sample_rate("metrics::counter", 0.25);

        assert!(sampler.sample_rates.contains_key("http.requests.slow"));
        assert!(sampler.sample_rates.contains_key("auth/login"));
        assert!(sampler.sample_rates.contains_key("metrics::counter"));
    }

    // ==================== Sampling Pattern Tests ====================

    #[test]
    fn test_sampling_pattern_deterministic() {
        let mut sampler1 = LogSampler::new();
        let mut sampler2 = LogSampler::new();

        sampler1.set_sample_rate("test", 0.2);
        sampler2.set_sample_rate("test", 0.2);

        // Both samplers should produce the same pattern
        for _ in 0..20 {
            assert_eq!(sampler1.should_log("test"), sampler2.should_log("test"));
        }
    }

    #[test]
    fn test_sampling_distribution() {
        let mut sampler = LogSampler::new();
        sampler.set_sample_rate("dist", 0.1);

        let total_calls = 1000;
        let mut logged = 0;

        for _ in 0..total_calls {
            if sampler.should_log("dist") {
                logged += 1;
            }
        }

        // With rate 0.1, we expect exactly 100 logs (every 10th call)
        assert_eq!(logged, 100);
    }

    // ==================== Concurrent Access Simulation Tests ====================

    #[test]
    fn test_counter_atomic_operations() {
        let mut sampler = LogSampler::new();
        sampler.set_sample_rate("atomic", 0.5);

        // Simulate rapid calls
        let mut results = Vec::new();
        for _ in 0..1000 {
            results.push(sampler.should_log("atomic"));
        }

        // Count how many times we logged
        let logged = results.iter().filter(|&&x| x).count();
        assert_eq!(logged, 500);
    }

    // ==================== Rate Value Boundary Tests ====================

    #[test]
    fn test_rate_exactly_zero() {
        let mut sampler = LogSampler::new();
        sampler.set_sample_rate("zero", 0.0);

        assert!(!sampler.should_log("zero"));
    }

    #[test]
    fn test_rate_exactly_one() {
        let mut sampler = LogSampler::new();
        sampler.set_sample_rate("one", 1.0);

        assert!(sampler.should_log("one"));
    }

    #[test]
    fn test_rate_just_above_zero() {
        let mut sampler = LogSampler::new();
        sampler.set_sample_rate("tiny", 0.0001);

        // First call should log (count 0 % 10000 == 0)
        assert!(sampler.should_log("tiny"));

        // Next 9999 calls should not log
        for _ in 0..9999 {
            let _ = sampler.should_log("tiny");
        }

        // 10000th call should log again
        assert!(sampler.should_log("tiny"));
    }

    #[test]
    fn test_rate_just_below_one() {
        let mut sampler = LogSampler::new();
        sampler.set_sample_rate("almost", 0.9999);

        // 1/0.9999 ≈ 1.0001, truncates to 1
        // So every call should log
        for _ in 0..10 {
            assert!(sampler.should_log("almost"));
        }
    }

    // ==================== Stress Tests ====================

    #[test]
    fn test_many_categories() {
        let mut sampler = LogSampler::new();

        for i in 0..100 {
            sampler.set_sample_rate(&format!("category_{}", i), 0.5);
        }

        assert_eq!(sampler.sample_rates.len(), 100);
        assert_eq!(sampler.counters.len(), 100);
    }

    #[test]
    fn test_high_call_volume() {
        let mut sampler = LogSampler::new();
        sampler.set_sample_rate("high_volume", 0.01);

        let mut logged = 0;
        for _ in 0..100_000 {
            if sampler.should_log("high_volume") {
                logged += 1;
            }
        }

        // With rate 0.01, expect 1000 logs
        assert_eq!(logged, 1000);
    }
}
