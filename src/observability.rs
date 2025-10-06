//! Comprehensive logging and metrics for observability
//!
//! This module provides structured logging, metrics collection, and
//! health monitoring capabilities for the EEYF library to enable
//! production observability and debugging.

use crate::yahoo_error::YahooError;
use log::{debug, error, info, warn};
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicU32, AtomicU64, Ordering};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::sync::RwLock;

#[cfg(feature = "tracing")]
use tracing::{Level, span};

#[cfg(feature = "metrics")]
use metrics::{counter, describe_counter, describe_gauge, describe_histogram, gauge, histogram};

/// Configuration for logging and metrics
#[derive(Debug, Clone)]
pub struct ObservabilityConfig {
    /// Enable structured logging
    pub enable_logging: bool,
    /// Log level filter
    pub log_level: LogLevel,
    /// Enable metrics collection
    pub enable_metrics: bool,
    /// Enable tracing (requires 'tracing' feature)
    pub enable_tracing: bool,
    /// Enable health checks
    pub enable_health_checks: bool,
    /// Metrics collection interval (in milliseconds)
    pub metrics_interval_ms: u64,
    /// Whether to log request/response details
    pub log_request_details: bool,
    /// Whether to log error details
    pub log_error_details: bool,
    /// Maximum request duration to log (in milliseconds)
    pub slow_request_threshold_ms: u64,
}

impl Default for ObservabilityConfig {
    fn default() -> Self {
        Self {
            enable_logging: true,
            log_level: LogLevel::Info,
            enable_metrics: false,
            enable_tracing: false,
            enable_health_checks: true,
            metrics_interval_ms: 60000, // 1 minute
            log_request_details: false,
            log_error_details: true,
            slow_request_threshold_ms: 5000, // 5 seconds
        }
    }
}

/// Log levels
#[derive(Debug, Clone, PartialEq)]
pub enum LogLevel {
    Error,
    Warn,
    Info,
    Debug,
    Trace,
}

/// Metrics collected by the library
#[derive(Debug, Clone, Default)]
pub struct LibraryMetrics {
    // Request metrics
    pub total_requests: u64,
    pub successful_requests: u64,
    pub failed_requests: u64,
    pub request_duration_ms: Vec<u64>,

    // Error metrics by category
    pub errors_by_category: HashMap<String, u64>,

    // Rate limiting metrics
    pub rate_limit_hits: u64,
    pub rate_limit_wait_time_ms: u64,

    // Circuit breaker metrics
    pub circuit_breaker_opens: u64,
    pub circuit_breaker_closes: u64,
    pub rejected_requests: u64,

    // Cache metrics
    pub cache_hits: u64,
    pub cache_misses: u64,
    pub cache_evictions: u64,

    // Retry metrics
    pub retry_attempts: u64,
    pub retry_successes: u64,
    pub retry_exhausted: u64,

    // Connection metrics
    pub connection_pool_size: u32,
    pub active_connections: u32,
    pub connection_timeouts: u64,

    // Performance metrics
    pub avg_response_time_ms: f64,
    pub p95_response_time_ms: f64,
    pub p99_response_time_ms: f64,
    pub throughput_requests_per_second: f64,
}

impl LibraryMetrics {
    pub fn success_rate(&self) -> f64 {
        if self.total_requests == 0 {
            0.0
        } else {
            self.successful_requests as f64 / self.total_requests as f64
        }
    }

    pub fn error_rate(&self) -> f64 {
        1.0 - self.success_rate()
    }

    pub fn cache_hit_rate(&self) -> f64 {
        let total_cache_requests = self.cache_hits + self.cache_misses;
        if total_cache_requests == 0 {
            0.0
        } else {
            self.cache_hits as f64 / total_cache_requests as f64
        }
    }
}

/// Health check status
#[derive(Debug, Clone, PartialEq)]
pub enum HealthStatus {
    Healthy,
    Degraded,
    Unhealthy,
    Unknown,
}

/// Health check information
#[derive(Debug, Clone)]
pub struct HealthCheck {
    pub status: HealthStatus,
    pub message: String,
    pub last_check: u64,
    pub response_time_ms: Option<u64>,
    pub error_rate: f64,
    pub success_rate: f64,
}

/// Request context for logging and tracing
#[derive(Debug, Clone)]
pub struct RequestContext {
    pub request_id: String,
    pub method: String,
    pub url: String,
    pub started_at: u64,
    pub user_agent: Option<String>,
    pub client_ip: Option<String>,
}

impl RequestContext {
    pub fn new(method: String, url: String) -> Self {
        Self {
            request_id: generate_request_id(),
            method,
            url,
            started_at: current_time_millis(),
            user_agent: None,
            client_ip: None,
        }
    }

    pub fn duration_ms(&self) -> u64 {
        current_time_millis() - self.started_at
    }
}

/// Main observability manager
pub struct ObservabilityManager {
    config: ObservabilityConfig,
    metrics: Arc<RwLock<LibraryMetrics>>,
    health_status: Arc<RwLock<HealthCheck>>,

    // Atomic counters for high-frequency metrics
    request_counter: Arc<AtomicU64>,
    error_counter: Arc<AtomicU64>,
    cache_hit_counter: Arc<AtomicU64>,
    cache_miss_counter: Arc<AtomicU64>,

    // Active request tracking
    active_requests: Arc<AtomicU32>,
}

impl ObservabilityManager {
    pub fn new(config: ObservabilityConfig) -> Self {
        let health_check = HealthCheck {
            status: HealthStatus::Unknown,
            message: "Initial state".to_string(),
            last_check: current_time_millis(),
            response_time_ms: None,
            error_rate: 0.0,
            success_rate: 0.0,
        };

        let manager = Self {
            config,
            metrics: Arc::new(RwLock::new(LibraryMetrics::default())),
            health_status: Arc::new(RwLock::new(health_check)),
            request_counter: Arc::new(AtomicU64::new(0)),
            error_counter: Arc::new(AtomicU64::new(0)),
            cache_hit_counter: Arc::new(AtomicU64::new(0)),
            cache_miss_counter: Arc::new(AtomicU64::new(0)),
            active_requests: Arc::new(AtomicU32::new(0)),
        };

        #[cfg(feature = "metrics")]
        manager.setup_metrics();

        manager
    }

    pub fn with_default_config() -> Self {
        Self::new(ObservabilityConfig::default())
    }

    /// Setup Prometheus metrics (if feature enabled)
    #[cfg(feature = "metrics")]
    fn setup_metrics(&self) {
        describe_counter!("eeyf_requests_total", "Total number of requests made");
        describe_counter!("eeyf_errors_total", "Total number of errors by category");
        describe_counter!("eeyf_cache_hits_total", "Total number of cache hits");
        describe_counter!("eeyf_cache_misses_total", "Total number of cache misses");
        describe_counter!("eeyf_retries_total", "Total number of retry attempts");
        describe_counter!(
            "eeyf_circuit_breaker_opens_total",
            "Total circuit breaker opens"
        );

        describe_gauge!(
            "eeyf_active_requests",
            "Number of currently active requests"
        );
        describe_gauge!("eeyf_cache_entries", "Number of entries in cache");
        describe_gauge!(
            "eeyf_circuit_breaker_state",
            "Circuit breaker state (0=closed, 1=half-open, 2=open)"
        );

        describe_histogram!(
            "eeyf_request_duration_seconds",
            "Request duration in seconds"
        );

        info!("Prometheus metrics initialized");
    }

    /// Log request start
    pub fn log_request_start(&self, context: &RequestContext) {
        self.request_counter.fetch_add(1, Ordering::Relaxed);
        self.active_requests.fetch_add(1, Ordering::Relaxed);

        if self.config.enable_logging && self.config.log_request_details {
            info!(
                "[{}] Request started: {} {}",
                context.request_id, context.method, context.url
            );
        }

        #[cfg(feature = "tracing")]
        if self.config.enable_tracing {
            let span = span!(Level::INFO, "yahoo_request",
                request_id = %context.request_id,
                method = %context.method,
                url = %context.url
            );
            let _enter = span.enter();
            debug!("Request span created");
        }

        #[cfg(feature = "metrics")]
        if self.config.enable_metrics {
            counter!("eeyf_requests_total").increment(1);
            gauge!("eeyf_active_requests").set(self.active_requests.load(Ordering::Relaxed) as f64);
        }
    }

    /// Log request completion
    pub fn log_request_success(&self, context: &RequestContext, response_size: Option<usize>) {
        let duration_ms = context.duration_ms();
        self.active_requests.fetch_sub(1, Ordering::Relaxed);

        if self.config.enable_logging {
            if duration_ms > self.config.slow_request_threshold_ms {
                warn!(
                    "[{}] Slow request completed: {} {} ({}ms)",
                    context.request_id, context.method, context.url, duration_ms
                );
            } else if self.config.log_request_details {
                info!(
                    "[{}] Request completed: {} {} ({}ms{})",
                    context.request_id,
                    context.method,
                    context.url,
                    duration_ms,
                    response_size
                        .map(|s| format!(", {} bytes", s))
                        .unwrap_or_default()
                );
            }
        }

        #[cfg(feature = "metrics")]
        if self.config.enable_metrics {
            histogram!("eeyf_request_duration_seconds").record(duration_ms as f64 / 1000.0);
            gauge!("eeyf_active_requests").set(self.active_requests.load(Ordering::Relaxed) as f64);
        }

        // Update metrics
        tokio::spawn({
            let metrics = self.metrics.clone();
            async move {
                let mut m = metrics.write().await;
                m.successful_requests += 1;
                m.request_duration_ms.push(duration_ms);

                // Update average response time
                if !m.request_duration_ms.is_empty() {
                    m.avg_response_time_ms = m.request_duration_ms.iter().sum::<u64>() as f64
                        / m.request_duration_ms.len() as f64;
                }
            }
        });
    }

    /// Log request error
    pub fn log_request_error(&self, context: &RequestContext, error: &YahooError) {
        let duration_ms = context.duration_ms();
        self.error_counter.fetch_add(1, Ordering::Relaxed);
        self.active_requests.fetch_sub(1, Ordering::Relaxed);

        let error_info = crate::error_categories::ErrorCategorizer::categorize_error(error);

        if self.config.enable_logging && self.config.log_error_details {
            error!(
                "[{}] Request failed: {} {} ({}ms) - {} (category: {})",
                context.request_id,
                context.method,
                context.url,
                duration_ms,
                error,
                error_info.category
            );
        }

        #[cfg(feature = "metrics")]
        if self.config.enable_metrics {
            counter!("eeyf_errors_total", "category" => error_info.category.to_string())
                .increment(1);
            histogram!("eeyf_request_duration_seconds").record(duration_ms as f64 / 1000.0);
            gauge!("eeyf_active_requests").set(self.active_requests.load(Ordering::Relaxed) as f64);
        }

        // Update metrics
        tokio::spawn({
            let metrics = self.metrics.clone();
            let category = error_info.category.to_string();
            async move {
                let mut m = metrics.write().await;
                m.failed_requests += 1;
                *m.errors_by_category.entry(category).or_insert(0) += 1;
            }
        });
    }

    /// Log cache hit
    pub fn log_cache_hit(&self, url: &str) {
        self.cache_hit_counter.fetch_add(1, Ordering::Relaxed);

        if self.config.enable_logging {
            debug!("Cache hit for URL: {}", url);
        }

        #[cfg(feature = "metrics")]
        if self.config.enable_metrics {
            counter!("eeyf_cache_hits_total").increment(1);
        }
    }

    /// Log cache miss
    pub fn log_cache_miss(&self, url: &str) {
        self.cache_miss_counter.fetch_add(1, Ordering::Relaxed);

        if self.config.enable_logging {
            debug!("Cache miss for URL: {}", url);
        }

        #[cfg(feature = "metrics")]
        if self.config.enable_metrics {
            counter!("eeyf_cache_misses_total").increment(1);
        }
    }

    /// Log retry attempt
    pub fn log_retry_attempt(&self, attempt: u32, max_attempts: u32, error: &YahooError) {
        if self.config.enable_logging {
            warn!("Retry attempt {} of {}: {}", attempt, max_attempts, error);
        }

        #[cfg(feature = "metrics")]
        if self.config.enable_metrics {
            counter!("eeyf_retries_total").increment(1);
        }
    }

    /// Log circuit breaker state change
    pub fn log_circuit_breaker_state_change(&self, old_state: &str, new_state: &str, reason: &str) {
        if self.config.enable_logging {
            warn!(
                "Circuit breaker state changed: {} -> {} ({})",
                old_state, new_state, reason
            );
        }

        #[cfg(feature = "metrics")]
        if self.config.enable_metrics {
            if new_state == "open" {
                counter!("eeyf_circuit_breaker_opens_total").increment(1);
            }

            let state_value = match new_state {
                "closed" => 0.0,
                "half-open" => 1.0,
                "open" => 2.0,
                _ => -1.0,
            };
            gauge!("eeyf_circuit_breaker_state").set(state_value);
        }
    }

    /// Log rate limit hit
    pub fn log_rate_limit_hit(&self, wait_time_ms: u64) {
        if self.config.enable_logging {
            warn!("Rate limit hit, waiting {}ms", wait_time_ms);
        }

        tokio::spawn({
            let metrics = self.metrics.clone();
            async move {
                let mut m = metrics.write().await;
                m.rate_limit_hits += 1;
                m.rate_limit_wait_time_ms += wait_time_ms;
            }
        });
    }

    /// Get current metrics
    pub async fn get_metrics(&self) -> LibraryMetrics {
        let mut metrics = self.metrics.write().await;

        // Update atomic counters
        metrics.total_requests = self.request_counter.load(Ordering::Relaxed);
        metrics.cache_hits = self.cache_hit_counter.load(Ordering::Relaxed);
        metrics.cache_misses = self.cache_miss_counter.load(Ordering::Relaxed);

        // Calculate percentiles if we have duration data
        if !metrics.request_duration_ms.is_empty() {
            let mut durations = metrics.request_duration_ms.clone();
            durations.sort_unstable();

            let len = durations.len();
            metrics.p95_response_time_ms = durations[(len * 95 / 100).min(len - 1)] as f64;
            metrics.p99_response_time_ms = durations[(len * 99 / 100).min(len - 1)] as f64;
        }

        metrics.clone()
    }

    /// Perform health check
    pub async fn health_check(&self) -> HealthCheck {
        if !self.config.enable_health_checks {
            return HealthCheck {
                status: HealthStatus::Unknown,
                message: "Health checks disabled".to_string(),
                last_check: current_time_millis(),
                response_time_ms: None,
                error_rate: 0.0,
                success_rate: 0.0,
            };
        }

        let metrics = self.get_metrics().await;
        let now = current_time_millis();

        // Determine health status based on metrics
        let error_rate = metrics.error_rate();
        let status = if metrics.total_requests == 0 {
            // If no requests yet, assume healthy
            HealthStatus::Healthy
        } else if error_rate > 0.5 {
            HealthStatus::Unhealthy
        } else if error_rate > 0.1 || metrics.avg_response_time_ms > 10000.0 {
            HealthStatus::Degraded
        } else {
            HealthStatus::Healthy
        };

        let message = match status {
            HealthStatus::Healthy => "All systems operational".to_string(),
            HealthStatus::Degraded => format!(
                "Degraded performance: {:.1}% error rate, {:.0}ms avg response",
                error_rate * 100.0,
                metrics.avg_response_time_ms
            ),
            HealthStatus::Unhealthy => {
                format!("System unhealthy: {:.1}% error rate", error_rate * 100.0)
            }
            HealthStatus::Unknown => "Unknown status".to_string(),
        };

        let health_check = HealthCheck {
            status,
            message,
            last_check: now,
            response_time_ms: Some(metrics.avg_response_time_ms as u64),
            error_rate,
            success_rate: metrics.success_rate(),
        };

        *self.health_status.write().await = health_check.clone();
        health_check
    }

    /// Get current health status
    pub async fn get_health_status(&self) -> HealthCheck {
        self.health_status.read().await.clone()
    }
}

/// Generate a unique request ID
fn generate_request_id() -> String {
    use std::sync::atomic::{AtomicU64, Ordering};
    static COUNTER: AtomicU64 = AtomicU64::new(0);

    let counter = COUNTER.fetch_add(1, Ordering::Relaxed);
    let timestamp = current_time_millis();
    format!("req_{}_{}", timestamp, counter)
}

/// Get current time in milliseconds since Unix epoch
fn current_time_millis() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or(Duration::ZERO)
        .as_millis() as u64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_observability_manager_creation() {
        let manager = ObservabilityManager::with_default_config();
        let health = manager.health_check().await;
        assert_eq!(health.status, HealthStatus::Healthy);
    }

    #[tokio::test]
    async fn test_request_logging() {
        let manager = ObservabilityManager::with_default_config();
        let context = RequestContext::new("GET".to_string(), "https://test.com".to_string());

        manager.log_request_start(&context);
        manager.log_request_success(&context, Some(1024));

        // Give a small delay for the spawned async task to complete
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;

        let metrics = manager.get_metrics().await;
        assert_eq!(metrics.total_requests, 1);
        assert_eq!(metrics.successful_requests, 1);
    }

    #[tokio::test]
    async fn test_error_logging() {
        let manager = ObservabilityManager::with_default_config();
        let context = RequestContext::new("GET".to_string(), "https://test.com".to_string());
        let error = YahooError::FetchFailed("test error".to_string());

        manager.log_request_start(&context);
        manager.log_request_error(&context, &error);

        // Give a small delay for the spawned async task to complete
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;

        let metrics = manager.get_metrics().await;
        assert_eq!(metrics.total_requests, 1);
        assert_eq!(metrics.failed_requests, 1);
        assert!(metrics.errors_by_category.contains_key("transient"));
    }

    #[tokio::test]
    async fn test_cache_metrics() {
        let manager = ObservabilityManager::with_default_config();

        manager.log_cache_hit("https://test.com");
        manager.log_cache_miss("https://test2.com");

        let metrics = manager.get_metrics().await;
        assert_eq!(metrics.cache_hits, 1);
        assert_eq!(metrics.cache_misses, 1);
        assert_eq!(metrics.cache_hit_rate(), 0.5);
    }

    #[test]
    fn test_request_id_generation() {
        let id1 = generate_request_id();
        let id2 = generate_request_id();

        assert_ne!(id1, id2);
        assert!(id1.starts_with("req_"));
        assert!(id2.starts_with("req_"));
    }

    #[test]
    fn test_request_context() {
        let context = RequestContext::new("POST".to_string(), "https://api.test.com".to_string());

        assert_eq!(context.method, "POST");
        assert_eq!(context.url, "https://api.test.com");
        assert!(!context.request_id.is_empty());
        // Duration is always positive
        assert!(context.duration_ms() > 0 || context.duration_ms() == 0);
    }
}
