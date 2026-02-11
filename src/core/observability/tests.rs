//! Tests for observability module

#[cfg(test)]
mod tests {
    use super::super::histogram::{BoundedHistogram, HISTOGRAM_MAX_SAMPLES};
    use super::super::logging::LogAggregator;
    use super::super::metrics::MetricsCollector;
    use super::super::types::{LogEntry, TokenUsage};
    use chrono::Utc;
    use std::collections::HashMap;
    use std::time::Duration;

    #[test]
    fn test_bounded_histogram_basic() {
        let mut histogram = BoundedHistogram::new(5);

        // Record some values
        histogram.record(1.0);
        histogram.record(2.0);
        histogram.record(3.0);

        assert_eq!(histogram.count(), 3);
        assert_eq!(histogram.window_size(), 3);
        assert!((histogram.mean() - 2.0).abs() < 0.001);
        assert!((histogram.min() - 1.0).abs() < 0.001);
        assert!((histogram.max() - 3.0).abs() < 0.001);
    }

    #[test]
    fn test_bounded_histogram_rolling_window() {
        let mut histogram = BoundedHistogram::new(3);

        // Fill the histogram
        histogram.record(1.0);
        histogram.record(2.0);
        histogram.record(3.0);

        assert_eq!(histogram.window_size(), 3);
        assert!((histogram.mean() - 2.0).abs() < 0.001);

        // Add more values - oldest should be evicted
        histogram.record(4.0);
        histogram.record(5.0);

        // Window should still be 3, but now contains [3.0, 4.0, 5.0]
        assert_eq!(histogram.window_size(), 3);
        assert_eq!(histogram.count(), 5); // Total count should be 5
        assert!((histogram.mean() - 4.0).abs() < 0.001); // (3+4+5)/3 = 4
        assert!((histogram.min() - 3.0).abs() < 0.001);
        assert!((histogram.max() - 5.0).abs() < 0.001);
    }

    #[test]
    fn test_bounded_histogram_percentile() {
        let mut histogram = BoundedHistogram::new(100);

        // Record values 1-100
        for i in 1..=100 {
            histogram.record(i as f64);
        }

        // Test percentiles
        assert!((histogram.percentile(50.0) - 50.0).abs() < 1.0);
        assert!((histogram.percentile(90.0) - 90.0).abs() < 1.0);
        assert!((histogram.percentile(99.0) - 99.0).abs() < 1.0);
    }

    #[test]
    fn test_bounded_histogram_prevents_memory_leak() {
        let mut histogram = BoundedHistogram::new(100);

        // Record many more values than capacity
        for i in 0..10000 {
            histogram.record(i as f64);
        }

        // Window size should be capped at 100
        assert_eq!(histogram.window_size(), 100);
        // But total count should reflect all recordings
        assert_eq!(histogram.count(), 10000);
    }

    #[tokio::test]
    async fn test_metrics_collection() {
        let collector = MetricsCollector::new();

        collector
            .record_request(
                "openai",
                "gpt-4",
                Duration::from_millis(500),
                Some(TokenUsage {
                    prompt_tokens: 100,
                    completion_tokens: 50,
                    total_tokens: 150,
                }),
                Some(0.01),
                true,
            )
            .await;

        let prometheus_output = collector.export_prometheus().await;
        assert!(prometheus_output.contains("litellm_requests_total"));
        assert!(prometheus_output.contains("provider=\"openai\""));
        assert!(prometheus_output.contains("model=\"gpt-4\""));
    }

    #[tokio::test]
    async fn test_metrics_histogram_bounded() {
        let collector = MetricsCollector::new();

        // Record many requests to test histogram bounding
        for i in 0..2000 {
            collector
                .record_request(
                    "openai",
                    "gpt-4",
                    Duration::from_millis(i),
                    None,
                    None,
                    true,
                )
                .await;
        }

        // Verify histogram is bounded
        let metrics = collector.prometheus_metrics.read().await;
        let histogram = metrics.request_duration.get("openai:gpt-4").unwrap();

        // Window should be capped at HISTOGRAM_MAX_SAMPLES
        assert!(histogram.window_size() <= HISTOGRAM_MAX_SAMPLES);
        // But count should reflect all recordings
        assert_eq!(histogram.count(), 2000);
    }

    #[tokio::test]
    async fn test_log_aggregation() {
        let aggregator = LogAggregator::new();

        let entry = LogEntry {
            timestamp: Utc::now(),
            level: "INFO".to_string(),
            message: "Test log entry".to_string(),
            module: Some("observability".to_string()),
            request_id: Some("req-123".to_string()),
            metadata: HashMap::new(),
        };

        aggregator.log(entry).await;

        let buffer = aggregator.buffer.read().await;
        assert_eq!(buffer.len(), 1);
        assert_eq!(buffer[0].message, "Test log entry");
    }
}
