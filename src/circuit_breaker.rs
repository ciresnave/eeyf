//! Circuit breaker pattern implementation for fault tolerance
//!
//! This module provides a circuit breaker to prevent cascading failures
//! by temporarily stopping requests to a failing service and allowing
//! recovery detection through periodic health checks.

use crate::error_categories::{ErrorCategorizer, ErrorCategory};
use crate::yahoo_error::YahooError;
use log::{debug, warn, info, error};
use std::sync::atomic::{AtomicU32, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::sync::RwLock;

/// Circuit breaker states
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CircuitState {
    /// Normal operation - requests are allowed through
    Closed,
    /// Failure threshold exceeded - requests are blocked
    Open,
    /// Testing recovery - limited requests allowed to test service health
    HalfOpen,
}

/// Configuration for circuit breaker behavior
#[derive(Debug, Clone)]
pub struct CircuitBreakerConfig {
    /// Number of failures before opening the circuit
    pub failure_threshold: u32,
    /// Number of successful requests to close the circuit when half-open
    pub success_threshold: u32,
    /// Time to wait before attempting recovery (in milliseconds)
    pub recovery_timeout_ms: u64,
    /// Maximum number of requests allowed in half-open state
    pub half_open_max_requests: u32,
    /// Window size for failure rate calculation (in milliseconds)
    pub failure_rate_window_ms: u64,
    /// Minimum number of requests before failure rate is calculated
    pub minimum_request_volume: u32,
    /// Whether to consider different error categories differently
    pub categorize_failures: bool,
}

impl Default for CircuitBreakerConfig {
    fn default() -> Self {
        Self {
            failure_threshold: 5,
            success_threshold: 3,
            recovery_timeout_ms: 30000, // 30 seconds
            half_open_max_requests: 3,
            failure_rate_window_ms: 60000, // 1 minute
            minimum_request_volume: 10,
            categorize_failures: true,
        }
    }
}

impl CircuitBreakerConfig {
    /// Configuration optimized for high-availability services
    pub fn high_availability() -> Self {
        Self {
            failure_threshold: 3,
            success_threshold: 5,
            recovery_timeout_ms: 10000, // 10 seconds
            half_open_max_requests: 2,
            failure_rate_window_ms: 30000, // 30 seconds
            minimum_request_volume: 5,
            categorize_failures: true,
        }
    }

    /// Configuration for fault-tolerant but less aggressive breaking
    pub fn fault_tolerant() -> Self {
        Self {
            failure_threshold: 10,
            success_threshold: 2,
            recovery_timeout_ms: 60000, // 1 minute
            half_open_max_requests: 5,
            failure_rate_window_ms: 120000, // 2 minutes
            minimum_request_volume: 20,
            categorize_failures: true,
        }
    }

    /// Configuration for development/testing (more permissive)
    pub fn development() -> Self {
        Self {
            failure_threshold: 15,
            success_threshold: 1,
            recovery_timeout_ms: 5000, // 5 seconds
            half_open_max_requests: 10,
            failure_rate_window_ms: 30000,
            minimum_request_volume: 3,
            categorize_failures: true,
        }
    }
}

/// Statistics for circuit breaker monitoring
#[derive(Debug, Clone, Default)]
pub struct CircuitBreakerStats {
    pub total_requests: u64,
    pub successful_requests: u64,
    pub failed_requests: u64,
    pub rejected_requests: u64,
    pub state_transitions: u64,
    pub last_failure_time: Option<u64>,
    pub last_success_time: Option<u64>,
    pub current_consecutive_failures: u32,
    pub current_consecutive_successes: u32,
}

impl CircuitBreakerStats {
    pub fn failure_rate(&self) -> f64 {
        if self.total_requests == 0 {
            0.0
        } else {
            self.failed_requests as f64 / self.total_requests as f64
        }
    }

    pub fn success_rate(&self) -> f64 {
        1.0 - self.failure_rate()
    }
}

/// Circuit breaker implementation
pub struct CircuitBreaker {
    config: CircuitBreakerConfig,
    state: Arc<RwLock<CircuitState>>,
    stats: Arc<RwLock<CircuitBreakerStats>>,
    failure_count: Arc<AtomicU32>,
    success_count: Arc<AtomicU32>,
    last_failure_time: Arc<AtomicU64>,
    half_open_requests: Arc<AtomicU32>,
}

impl CircuitBreaker {
    pub fn new(config: CircuitBreakerConfig) -> Self {
        Self {
            config,
            state: Arc::new(RwLock::new(CircuitState::Closed)),
            stats: Arc::new(RwLock::new(CircuitBreakerStats::default())),
            failure_count: Arc::new(AtomicU32::new(0)),
            success_count: Arc::new(AtomicU32::new(0)),
            last_failure_time: Arc::new(AtomicU64::new(0)),
            half_open_requests: Arc::new(AtomicU32::new(0)),
        }
    }

    pub fn with_default_config() -> Self {
        Self::new(CircuitBreakerConfig::default())
    }

    /// Get current circuit breaker state
    pub async fn state(&self) -> CircuitState {
        *self.state.read().await
    }

    /// Get circuit breaker statistics
    pub async fn stats(&self) -> CircuitBreakerStats {
        self.stats.read().await.clone()
    }

    /// Execute an operation through the circuit breaker
    pub async fn execute<F, Fut, T>(&self, operation: F) -> Result<T, YahooError>
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = Result<T, YahooError>>,
    {
        // Check if request should be allowed
        if !self.should_allow_request().await {
            let mut stats = self.stats.write().await;
            stats.rejected_requests += 1;
            
            error!("Circuit breaker is OPEN - rejecting request");
            return Err(YahooError::FetchFailed(
                "Circuit breaker is open - service appears to be down".to_string()
            ));
        }

        // Track half-open requests
        let current_state = self.state().await;
        if current_state == CircuitState::HalfOpen {
            self.half_open_requests.fetch_add(1, Ordering::SeqCst);
        }

        // Execute the operation
        let result = operation().await;

        // Handle the result
        match &result {
            Ok(_) => {
                self.on_success().await;
            }
            Err(error) => {
                self.on_failure(error).await;
            }
        }

        // Decrement half-open request counter
        if current_state == CircuitState::HalfOpen {
            self.half_open_requests.fetch_sub(1, Ordering::SeqCst);
        }

        result
    }

    /// Check if a request should be allowed through
    async fn should_allow_request(&self) -> bool {
        let current_state = *self.state.read().await;
        
        match current_state {
            CircuitState::Closed => true,
            CircuitState::Open => {
                // Check if recovery timeout has elapsed
                let last_failure = self.last_failure_time.load(Ordering::SeqCst);
                let now = current_time_millis();
                
                if now.saturating_sub(last_failure) >= self.config.recovery_timeout_ms {
                    debug!("Recovery timeout elapsed, transitioning to half-open");
                    self.transition_to_half_open().await;
                    true
                } else {
                    false
                }
            }
            CircuitState::HalfOpen => {
                // Allow limited requests for testing
                self.half_open_requests.load(Ordering::SeqCst) < self.config.half_open_max_requests
            }
        }
    }

    /// Handle successful operation
    async fn on_success(&self) {
        let mut stats = self.stats.write().await;
        stats.total_requests += 1;
        stats.successful_requests += 1;
        stats.last_success_time = Some(current_time_millis());
        stats.current_consecutive_successes += 1;
        stats.current_consecutive_failures = 0;
        drop(stats);

        let success_count = self.success_count.fetch_add(1, Ordering::SeqCst) + 1;
        self.failure_count.store(0, Ordering::SeqCst);

        let current_state = self.state().await;
        
        if current_state == CircuitState::HalfOpen && success_count >= self.config.success_threshold {
            info!("Success threshold met in half-open state, closing circuit");
            self.transition_to_closed().await;
        }

        debug!("Operation succeeded (consecutive successes: {})", success_count);
    }

    /// Handle failed operation
    async fn on_failure(&self, error: &YahooError) {
        let mut stats = self.stats.write().await;
        stats.total_requests += 1;
        stats.failed_requests += 1;
        stats.last_failure_time = Some(current_time_millis());
        stats.current_consecutive_failures += 1;
        stats.current_consecutive_successes = 0;
        drop(stats);

        // Check if failure should count towards circuit breaking
        let should_count = if self.config.categorize_failures {
            let error_info = error.categorize_error();
            // Only count failures that indicate service issues, not client errors
            matches!(
                error_info.category,
                ErrorCategory::Transient | ErrorCategory::ServerError | ErrorCategory::RateLimit
            )
        } else {
            true
        };

        if !should_count {
            debug!("Failure not counted towards circuit breaking: {}", error);
            return;
        }

        let failure_count = self.failure_count.fetch_add(1, Ordering::SeqCst) + 1;
        self.success_count.store(0, Ordering::SeqCst);
        self.last_failure_time.store(current_time_millis(), Ordering::SeqCst);

        warn!("Operation failed (consecutive failures: {}): {}", failure_count, error);

        let current_state = self.state().await;
        
        match current_state {
            CircuitState::Closed => {
                if failure_count >= self.config.failure_threshold {
                    warn!("Failure threshold exceeded, opening circuit");
                    self.transition_to_open().await;
                }
            }
            CircuitState::HalfOpen => {
                warn!("Failure in half-open state, reopening circuit");
                self.transition_to_open().await;
            }
            CircuitState::Open => {
                // Already open, just update counters
                debug!("Additional failure while circuit is open");
            }
        }
    }

    /// Transition to closed state
    async fn transition_to_closed(&self) {
        let mut state = self.state.write().await;
        *state = CircuitState::Closed;
        drop(state);

        let mut stats = self.stats.write().await;
        stats.state_transitions += 1;
        drop(stats);

        self.failure_count.store(0, Ordering::SeqCst);
        self.success_count.store(0, Ordering::SeqCst);
        
        info!("Circuit breaker transitioned to CLOSED");
    }

    /// Transition to open state
    async fn transition_to_open(&self) {
        let mut state = self.state.write().await;
        *state = CircuitState::Open;
        drop(state);

        let mut stats = self.stats.write().await;
        stats.state_transitions += 1;
        drop(stats);
        
        error!("Circuit breaker transitioned to OPEN");
    }

    /// Transition to half-open state
    async fn transition_to_half_open(&self) {
        let mut state = self.state.write().await;
        *state = CircuitState::HalfOpen;
        drop(state);

        let mut stats = self.stats.write().await;
        stats.state_transitions += 1;
        drop(stats);

        self.success_count.store(0, Ordering::SeqCst);
        self.half_open_requests.store(0, Ordering::SeqCst);
        
        info!("Circuit breaker transitioned to HALF-OPEN");
    }

    /// Manually reset the circuit breaker to closed state
    pub async fn reset(&self) {
        info!("Manually resetting circuit breaker");
        self.transition_to_closed().await;
    }

    /// Force the circuit breaker to open (for testing/maintenance)
    pub async fn force_open(&self) {
        warn!("Manually forcing circuit breaker to OPEN state");
        self.transition_to_open().await;
    }

    /// Check if the circuit breaker is currently open (synchronous check)
    /// 
    /// Note: This performs a try_read which may return false positives if the lock is held.
    /// For definitive state checking, use the async `state()` method.
    pub fn is_open(&self) -> bool {
        self.state.try_read()
            .map(|state| *state == CircuitState::Open)
            .unwrap_or(false)
    }

    /// Manually record a successful operation for testing purposes
    /// 
    /// This bypasses the normal `execute()` flow and directly updates internal counters.
    /// Useful for integration tests that need to control circuit breaker state.
    pub async fn record_success(&self) {
        self.on_success().await;
    }

    /// Manually record a failed operation for testing purposes
    /// 
    /// This bypasses the normal `execute()` flow and directly updates internal counters.
    /// Useful for integration tests that need to control circuit breaker state.
    pub async fn record_failure(&self, error: &YahooError) {
        self.on_failure(error).await;
    }
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
    use tokio::time::{sleep, Duration};

    #[tokio::test]
    async fn test_circuit_breaker_closed_state() {
        let config = CircuitBreakerConfig {
            failure_threshold: 2,
            ..Default::default()
        };
        let cb = CircuitBreaker::new(config);
        
        // Should start in closed state
        assert_eq!(cb.state().await, CircuitState::Closed);
        
        // Successful request should work
        let result = cb.execute(|| async { Ok::<_, YahooError>("success") }).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_circuit_breaker_opens_after_failures() {
        let config = CircuitBreakerConfig {
            failure_threshold: 2,
            recovery_timeout_ms: 100,
            ..Default::default()
        };
        let cb = CircuitBreaker::new(config);
        
        // First failure
        let _ = cb.execute(|| async { 
            Err::<(), _>(YahooError::FetchFailed("test failure".to_string())) 
        }).await;
        assert_eq!(cb.state().await, CircuitState::Closed);
        
        // Second failure should open the circuit
        let _ = cb.execute(|| async { 
            Err::<(), _>(YahooError::FetchFailed("test failure".to_string())) 
        }).await;
        assert_eq!(cb.state().await, CircuitState::Open);
        
        // Third request should be rejected
        let result = cb.execute(|| async { Ok::<_, YahooError>("should be rejected") }).await;
        assert!(result.is_err());
        assert_eq!(cb.state().await, CircuitState::Open);
    }

    #[tokio::test]
    async fn test_circuit_breaker_recovery() {
        let config = CircuitBreakerConfig {
            failure_threshold: 1,
            recovery_timeout_ms: 50, // Short timeout for testing
            success_threshold: 1,
            ..Default::default()
        };
        let cb = CircuitBreaker::new(config);
        
        // Cause failure to open circuit
        let _ = cb.execute(|| async { 
            Err::<(), _>(YahooError::FetchFailed("test failure".to_string())) 
        }).await;
        assert_eq!(cb.state().await, CircuitState::Open);
        
        // Wait for recovery timeout
        sleep(Duration::from_millis(60)).await;
        
        // Next request should transition to half-open and succeed
        let result = cb.execute(|| async { Ok::<_, YahooError>("recovery success") }).await;
        assert!(result.is_ok());
        assert_eq!(cb.state().await, CircuitState::Closed);
    }

    #[tokio::test]
    async fn test_circuit_breaker_stats() {
        let cb = CircuitBreaker::with_default_config();
        
        // Execute some operations
        let _ = cb.execute(|| async { Ok::<_, YahooError>("success1") }).await;
        let _ = cb.execute(|| async { Ok::<_, YahooError>("success2") }).await;
        let _ = cb.execute(|| async { 
            Err::<(), _>(YahooError::FetchFailed("failure".to_string())) 
        }).await;
        
        let stats = cb.stats().await;
        assert_eq!(stats.total_requests, 3);
        assert_eq!(stats.successful_requests, 2);
        assert_eq!(stats.failed_requests, 1);
        assert!((stats.failure_rate() - 0.333).abs() < 0.01);
    }
}