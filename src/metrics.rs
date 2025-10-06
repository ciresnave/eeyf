//! Prometheus metrics collection and exposition
//!
//! This module provides comprehensive metrics collection for the EEYF library,
//! with Prometheus-compatible metrics that can be scraped by monitoring systems.

use crate::yahoo_error::YahooError;
use std::collections::HashMap;
use std::sync::Arc;
#[allow(unused_imports)] // Used in conditional compilation paths
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};

use tokio::sync::RwLock;

#[cfg(feature = "metrics")]
use metrics::{
    counter, describe_counter, describe_gauge, describe_histogram, 
    gauge, histogram, Unit
};

#[cfg(feature = "metrics")]
use metrics_exporter_prometheus::PrometheusBuilder;

/// Prometheus metrics collector for EEYF
#[derive(Debug)]
pub struct PrometheusMetrics {
    // Internal state tracking (kept for future metrics enhancements)
    #[allow(dead_code)]
    start_time: Instant,
    #[allow(dead_code)]
    request_durations: Arc<RwLock<Vec<Duration>>>,
    #[allow(dead_code)]
    error_counts: Arc<RwLock<HashMap<String, AtomicU64>>>,
    #[allow(dead_code)]
    symbol_counts: Arc<RwLock<HashMap<String, AtomicU64>>>,
}

impl Default for PrometheusMetrics {
    fn default() -> Self {
        Self::new()
    }
}

impl PrometheusMetrics {
    /// Create a new Prometheus metrics collector
    pub fn new() -> Self {
        Self {
            start_time: Instant::now(),
            request_durations: Arc::new(RwLock::new(Vec::new())),
            error_counts: Arc::new(RwLock::new(HashMap::new())),
            symbol_counts: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Initialize Prometheus exporter on the given port
    #[cfg(feature = "metrics")]
    pub async fn start_prometheus_server(port: u16) -> Result<(), YahooError> {
        let addr: std::net::SocketAddr = format!("0.0.0.0:{}", port).parse().map_err(|e| {
            YahooError::InvalidStatusCode(format!("Invalid port {}: {}", port, e))
        })?;
        let builder = PrometheusBuilder::new().with_http_listener(addr);

        builder.install().map_err(|e| {
            YahooError::InvalidStatusCode(format!("Failed to start Prometheus server: {}", e))
        })?;

        // Register all metric descriptions
        Self::register_metrics();

        println!("📊 Prometheus metrics server started on http://0.0.0.0:{}/metrics", port);
        Ok(())
    }

    /// Register all metrics with their descriptions
    #[cfg(feature = "metrics")]
    fn register_metrics() {
        // Request metrics
        describe_counter!("eeyf_requests_total", "Total number of requests made to Yahoo Finance API");
        describe_counter!("eeyf_requests_success_total", "Number of successful requests");
        describe_counter!("eeyf_requests_error_total", "Number of failed requests");
        describe_histogram!("eeyf_request_duration_seconds", Unit::Seconds, "Request duration in seconds");

        // Symbol-specific metrics
        describe_counter!("eeyf_symbol_requests_total", "Requests per symbol");

        // Rate limiter metrics
        describe_counter!("eeyf_rate_limit_hits_total", "Number of times rate limit was hit");
        describe_histogram!("eeyf_rate_limit_wait_seconds", Unit::Seconds, "Time spent waiting for rate limit");
        describe_gauge!("eeyf_rate_limit_tokens", "Current available rate limit tokens");

        // Circuit breaker metrics
        describe_counter!("eeyf_circuit_breaker_opens_total", "Circuit breaker open events");
        describe_counter!("eeyf_circuit_breaker_closes_total", "Circuit breaker close events");
        describe_counter!("eeyf_circuit_breaker_half_opens_total", "Circuit breaker half-open events");
        describe_gauge!("eeyf_circuit_breaker_state", "Circuit breaker state (0=closed, 1=open, 2=half-open)");
        describe_counter!("eeyf_requests_rejected_total", "Requests rejected by circuit breaker");

        // Cache metrics
        describe_counter!("eeyf_cache_hits_total", "Cache hit count");
        describe_counter!("eeyf_cache_misses_total", "Cache miss count");
        describe_counter!("eeyf_cache_evictions_total", "Cache eviction count");
        describe_gauge!("eeyf_cache_size", "Current cache size");
        describe_histogram!("eeyf_cache_lookup_duration_seconds", Unit::Seconds, "Cache lookup duration");

        // Retry metrics
        describe_counter!("eeyf_retry_attempts_total", "Total retry attempts");
        describe_counter!("eeyf_retry_success_total", "Successful retries");
        describe_counter!("eeyf_retry_exhausted_total", "Retries that exhausted all attempts");
        describe_histogram!("eeyf_retry_delay_seconds", Unit::Seconds, "Retry delay duration");

        // Connection pool metrics
        describe_gauge!("eeyf_connection_pool_size", "Connection pool size");
        describe_gauge!("eeyf_active_connections", "Active connections");
        describe_counter!("eeyf_connection_timeouts_total", "Connection timeout count");
        describe_histogram!("eeyf_connection_acquire_duration_seconds", Unit::Seconds, "Connection acquire duration");

        // Error metrics by type
        describe_counter!("eeyf_errors_by_type_total", "Errors categorized by type");

        // Performance metrics
        describe_gauge!("eeyf_uptime_seconds", "Library uptime in seconds");
        describe_gauge!("eeyf_success_rate", "Success rate (0-1)");
        describe_gauge!("eeyf_error_rate", "Error rate (0-1)");
        describe_gauge!("eeyf_throughput_rps", "Throughput in requests per second");
    }

    /// Record a successful request
    #[cfg(feature = "metrics")]
    pub async fn record_request_success(&self, symbol: &str, duration: Duration, endpoint: &str) {
        counter!("eeyf_requests_total", "symbol" => symbol.to_string(), "endpoint" => endpoint.to_string(), "status" => "success").increment(1);
        counter!("eeyf_requests_success_total", "symbol" => symbol.to_string(), "endpoint" => endpoint.to_string()).increment(1);
        histogram!("eeyf_request_duration_seconds", "symbol" => symbol.to_string(), "endpoint" => endpoint.to_string()).record(duration.as_secs_f64());

        // Update symbol-specific metrics
        counter!("eeyf_symbol_requests_total", "symbol" => symbol.to_string()).increment(1);

        // Store duration for performance calculations
        self.request_durations.write().await.push(duration);
        
        // Keep only recent durations (last 1000 requests for performance)
        let mut durations = self.request_durations.write().await;
        if durations.len() > 1000 {
            durations.remove(0);
        }
    }

    /// Record a failed request
    #[cfg(feature = "metrics")]
    pub async fn record_request_error(&self, symbol: &str, duration: Duration, endpoint: &str, error_type: &str) {
        counter!("eeyf_requests_total", "symbol" => symbol.to_string(), "endpoint" => endpoint.to_string(), "status" => "error", "error_type" => error_type.to_string()).increment(1);
        counter!("eeyf_requests_error_total", "symbol" => symbol.to_string(), "endpoint" => endpoint.to_string(), "error_type" => error_type.to_string()).increment(1);
        counter!("eeyf_errors_by_type_total", "error_type" => error_type.to_string()).increment(1);
        histogram!("eeyf_request_duration_seconds", "symbol" => symbol.to_string(), "endpoint" => endpoint.to_string()).record(duration.as_secs_f64());

        // Update error counts
        let mut error_counts = self.error_counts.write().await;
        error_counts.entry(error_type.to_string())
            .or_insert_with(|| AtomicU64::new(0))
            .fetch_add(1, Ordering::Relaxed);
    }

    /// Record rate limiter metrics
    #[cfg(feature = "metrics")]
    pub fn record_rate_limit_hit(&self, wait_duration: Duration) {
        counter!("eeyf_rate_limit_hits_total").increment(1);
        histogram!("eeyf_rate_limit_wait_seconds").record(wait_duration.as_secs_f64());
    }

    /// Update rate limiter token count
    #[cfg(feature = "metrics")]
    pub fn update_rate_limit_tokens(&self, tokens: f64) {
        gauge!("eeyf_rate_limit_tokens").set(tokens);
    }

    /// Record circuit breaker state change
    #[cfg(feature = "metrics")]
    pub fn record_circuit_breaker_open(&self) {
        counter!("eeyf_circuit_breaker_opens_total").increment(1);
        gauge!("eeyf_circuit_breaker_state").set(1.0); // 1 = open
    }

    /// Record circuit breaker close
    #[cfg(feature = "metrics")]
    pub fn record_circuit_breaker_close(&self) {
        counter!("eeyf_circuit_breaker_closes_total").increment(1);
        gauge!("eeyf_circuit_breaker_state").set(0.0); // 0 = closed
    }

    /// Record circuit breaker half-open
    #[cfg(feature = "metrics")]
    pub fn record_circuit_breaker_half_open(&self) {
        counter!("eeyf_circuit_breaker_half_opens_total").increment(1);
        gauge!("eeyf_circuit_breaker_state").set(2.0); // 2 = half-open
    }

    /// Record rejected request
    #[cfg(feature = "metrics")]
    pub fn record_request_rejected(&self) {
        counter!("eeyf_requests_rejected_total").increment(1);
    }

    /// Record cache hit
    #[cfg(feature = "metrics")]
    pub fn record_cache_hit(&self, lookup_duration: Duration) {
        counter!("eeyf_cache_hits_total").increment(1);
        histogram!("eeyf_cache_lookup_duration_seconds").record(lookup_duration.as_secs_f64());
    }

    /// Record cache miss
    #[cfg(feature = "metrics")]
    pub fn record_cache_miss(&self, lookup_duration: Duration) {
        counter!("eeyf_cache_misses_total").increment(1);
        histogram!("eeyf_cache_lookup_duration_seconds").record(lookup_duration.as_secs_f64());
    }

    /// Record cache eviction
    #[cfg(feature = "metrics")]
    pub fn record_cache_eviction(&self) {
        counter!("eeyf_cache_evictions_total").increment(1);
    }

    /// Update cache size
    #[cfg(feature = "metrics")]
    pub fn update_cache_size(&self, size: f64) {
        gauge!("eeyf_cache_size").set(size);
    }

    /// Record retry attempt
    #[cfg(feature = "metrics")]
    pub fn record_retry_attempt(&self, delay: Duration) {
        counter!("eeyf_retry_attempts_total").increment(1);
        histogram!("eeyf_retry_delay_seconds").record(delay.as_secs_f64());
    }

    /// Record successful retry
    #[cfg(feature = "metrics")]
    pub fn record_retry_success(&self) {
        counter!("eeyf_retry_success_total").increment(1);
    }

    /// Record exhausted retry
    #[cfg(feature = "metrics")]
    pub fn record_retry_exhausted(&self) {
        counter!("eeyf_retry_exhausted_total").increment(1);
    }

    /// Update connection pool metrics
    #[cfg(feature = "metrics")]
    pub fn update_connection_pool(&self, pool_size: f64, active_connections: f64) {
        gauge!("eeyf_connection_pool_size").set(pool_size);
        gauge!("eeyf_active_connections").set(active_connections);
    }

    /// Record connection timeout
    #[cfg(feature = "metrics")]
    pub fn record_connection_timeout(&self) {
        counter!("eeyf_connection_timeouts_total").increment(1);
    }

    /// Record connection acquire duration
    #[cfg(feature = "metrics")]
    pub fn record_connection_acquire(&self, duration: Duration) {
        histogram!("eeyf_connection_acquire_duration_seconds").record(duration.as_secs_f64());
    }

    /// Update performance metrics
    #[cfg(feature = "metrics")]
    pub async fn update_performance_metrics(&self) {
        // Update uptime
        let uptime = self.start_time.elapsed().as_secs_f64();
        gauge!("eeyf_uptime_seconds").set(uptime);

        // Calculate and update derived metrics from stored data
        let durations = self.request_durations.read().await;
        if !durations.is_empty() {
            let total_requests = durations.len() as f64;
            let total_errors = {
                let error_counts = self.error_counts.read().await;
                error_counts.values().map(|c| c.load(Ordering::Relaxed)).sum::<u64>() as f64
            };

            let success_rate = (total_requests - total_errors) / total_requests;
            let error_rate = total_errors / total_requests;
            let throughput = total_requests / uptime;

            gauge!("eeyf_success_rate").set(success_rate);
            gauge!("eeyf_error_rate").set(error_rate);
            gauge!("eeyf_throughput_rps").set(throughput);
        }
    }
}

/// Helper function to categorize errors for metrics
pub fn categorize_error(error: &YahooError) -> &'static str {
    match error {
        YahooError::InvalidStatusCode(_) => "http_error",
        YahooError::DeserializeFailed(_) => "parse_error",
        YahooError::ConnectionFailed(_) => "connection_error",
        YahooError::TooManyRequests(_) => "rate_limit",
        YahooError::Unauthorized => "auth_error",
        YahooError::FetchFailed(_) => "fetch_error",
        YahooError::ApiError(_) => "api_error",
        YahooError::NoResult | YahooError::NoQuotes => "no_data",
        YahooError::DataInconsistency => "data_error",
        YahooError::BuilderFailed => "config_error",
        _ => "unknown_error",
    }
}

// No-op implementations when metrics feature is disabled
#[cfg(not(feature = "metrics"))]
impl PrometheusMetrics {
    pub async fn start_prometheus_server(_port: u16) -> Result<(), YahooError> {
        Ok(())
    }
    
    pub async fn record_request_success(&self, _symbol: &str, _duration: Duration, _endpoint: &str) {}
    pub async fn record_request_error(&self, _symbol: &str, _duration: Duration, _endpoint: &str, _error_type: &str) {}
    pub fn record_rate_limit_hit(&self, _wait_duration: Duration) {}
    pub fn update_rate_limit_tokens(&self, _tokens: f64) {}
    pub fn record_circuit_breaker_open(&self) {}
    pub fn record_circuit_breaker_close(&self) {}
    pub fn record_circuit_breaker_half_open(&self) {}
    pub fn record_request_rejected(&self) {}
    pub fn record_cache_hit(&self, _lookup_duration: Duration) {}
    pub fn record_cache_miss(&self, _lookup_duration: Duration) {}
    pub fn record_cache_eviction(&self) {}
    pub fn update_cache_size(&self, _size: f64) {}
    pub fn record_retry_attempt(&self, _delay: Duration) {}
    pub fn record_retry_success(&self) {}
    pub fn record_retry_exhausted(&self) {}
    pub fn update_connection_pool(&self, _pool_size: f64, _active_connections: f64) {}
    pub fn record_connection_timeout(&self) {}
    pub fn record_connection_acquire(&self, _duration: Duration) {}
    pub async fn update_performance_metrics(&self) {}
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_metrics_creation() {
        let metrics = PrometheusMetrics::new();
        assert!(metrics.request_durations.read().await.is_empty());
    }

    #[test]
    fn test_error_categorization() {
        let fetch_error = YahooError::FetchFailed("test fetch".to_string());
        assert_eq!(categorize_error(&fetch_error), "fetch_error");

        let connection_error = YahooError::ConnectionFailed("test connection".to_string());
        assert_eq!(categorize_error(&connection_error), "connection_error");
    }

    #[cfg(feature = "metrics")]
    #[tokio::test]
    async fn test_request_tracking() {
        let metrics = PrometheusMetrics::new();
        
        // Test successful request recording
        metrics.record_request_success("AAPL", Duration::from_millis(100), "quotes").await;
        
        let durations = metrics.request_durations.read().await;
        assert_eq!(durations.len(), 1);
        assert_eq!(durations[0], Duration::from_millis(100));
    }
}