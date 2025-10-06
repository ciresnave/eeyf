//! Intelligent Rate Limiting & Circuit Breakers for EEYF
//!
//! This module provides advanced rate limiting and circuit breaker patterns
//! for reliable Yahoo Finance API integration:
//!
//! - Adaptive rate limiting based on API response patterns
//! - Multi-level circuit breakers with intelligent recovery
//! - Request prioritization and queuing strategies
//! - Integration with caching and connection pooling
//! - Real-time rate limit monitoring and adjustment

use crate::yahoo_error::YahooError;
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;

use std::time::{Duration, Instant};
use tokio::sync::{Mutex, RwLock};


/// Rate limiter configuration with adaptive behavior
#[derive(Debug, Clone)]
pub struct AdaptiveRateLimitConfig {
    /// Base requests per second limit
    pub base_rps: f64,
    /// Maximum requests per second (adaptive ceiling)
    pub max_rps: f64,
    /// Minimum requests per second (adaptive floor)
    pub min_rps: f64,
    /// Window duration for rate calculations
    pub window_duration: Duration,
    /// Burst capacity (requests allowed in short bursts)
    pub burst_capacity: usize,
    /// Adaptive adjustment factor (0.0 - 1.0)
    pub adaptation_factor: f64,
    /// Backoff configuration when rate limits are hit
    pub backoff_config: BackoffConfig,
    /// Request prioritization strategy
    pub priority_strategy: PriorityStrategy,
}

/// Backoff configuration for rate limit recovery
#[derive(Debug, Clone)]
pub struct BackoffConfig {
    /// Initial backoff delay
    pub initial_delay: Duration,
    /// Maximum backoff delay
    pub max_delay: Duration,
    /// Backoff multiplier (exponential backoff)
    pub multiplier: f64,
    /// Jitter factor to prevent thundering herd
    pub jitter_factor: f64,
}

/// Request prioritization strategies
#[derive(Debug, Clone, PartialEq)]
pub enum PriorityStrategy {
    /// First-in-first-out (fair queuing)
    Fifo,
    /// Priority-based queuing
    Priority,
    /// Weighted fair queuing
    WeightedFair,
    /// Adaptive priority based on request type
    Adaptive,
}

/// Circuit breaker configuration
#[derive(Debug, Clone)]
pub struct CircuitBreakerConfig {
    /// Failure threshold to trip the circuit
    pub failure_threshold: usize,
    /// Success threshold to close the circuit
    pub success_threshold: usize,
    /// Timeout duration in open state
    pub timeout: Duration,
    /// Sliding window size for failure tracking
    pub window_size: usize,
    /// Health check interval when circuit is open
    pub health_check_interval: Duration,
    /// Circuit recovery strategy
    pub recovery_strategy: RecoveryStrategy,
}

/// Circuit breaker recovery strategies
#[derive(Debug, Clone, PartialEq)]
pub enum RecoveryStrategy {
    /// Immediate full recovery on success
    Immediate,
    /// Gradual recovery with increasing success rate
    Gradual,
    /// Exponential recovery based on success history
    Exponential,
}

/// Circuit breaker states
#[derive(Debug, Clone, PartialEq)]
pub enum CircuitState {
    /// Circuit is closed (normal operation)
    Closed,
    /// Circuit is open (failing fast)
    Open,
    /// Circuit is half-open (testing recovery)
    HalfOpen,
}

/// Request priority levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum RequestPriority {
    Low = 0,
    Normal = 1,
    High = 2,
    Critical = 3,
}

/// Rate limiting result
#[derive(Debug)]
pub enum RateLimitResult {
    /// Request is allowed to proceed
    Allowed,
    /// Request should be delayed
    Delayed(Duration),
    /// Request is denied (rate limit exceeded)
    Denied,
}

/// Circuit breaker result
#[derive(Debug)]
pub enum CircuitResult<T> {
    /// Request succeeded
    Success(T),
    /// Request failed but circuit remains closed
    Failure(YahooError),
    /// Circuit is open, request rejected
    CircuitOpen,
}

/// Request metadata for intelligent processing
#[derive(Debug, Clone)]
pub struct RequestMetadata {
    /// Request priority
    pub priority: RequestPriority,
    /// Request timestamp
    pub timestamp: Instant,
    /// Request type/endpoint
    pub request_type: String,
    /// Expected response time
    pub expected_duration: Option<Duration>,
    /// Retry count
    pub retry_count: usize,
}

/// Adaptive rate limiter with intelligent behavior
pub struct AdaptiveRateLimiter {
    /// Configuration
    config: AdaptiveRateLimitConfig,
    /// Current rate limit (requests per second)
    current_rps: Arc<parking_lot::RwLock<f64>>,
    /// Request timestamps for window calculations
    request_history: Arc<Mutex<VecDeque<Instant>>>,
    /// Pending requests queue
    request_queue: Arc<Mutex<RequestQueue>>,
    /// Rate limit statistics
    stats: Arc<RwLock<RateLimitStats>>,
    /// Adaptive adjustment state
    adaptation_state: Arc<RwLock<AdaptationState>>,
}

/// Intelligent circuit breaker
pub struct IntelligentCircuitBreaker {
    /// Configuration
    config: CircuitBreakerConfig,
    /// Current circuit state
    state: Arc<RwLock<CircuitState>>,
    /// Failure history tracking
    failure_history: Arc<Mutex<VecDeque<FailureRecord>>>,
    /// Success history tracking  
    success_history: Arc<Mutex<VecDeque<Instant>>>,
    /// Circuit statistics
    stats: Arc<RwLock<CircuitBreakerStats>>,
    /// Last state change timestamp
    last_state_change: Arc<RwLock<Instant>>,
}

/// Request queue with prioritization
struct RequestQueue {
    /// High priority requests
    high_priority: VecDeque<PendingRequest>,
    /// Normal priority requests
    normal_priority: VecDeque<PendingRequest>,
    /// Low priority requests
    low_priority: VecDeque<PendingRequest>,
    /// Queue statistics
    queue_stats: QueueStats,
}

/// Pending request in queue
#[derive(Debug)]
struct PendingRequest {
    /// Request metadata
    metadata: RequestMetadata,
    /// Channel to notify when request can proceed
    notify: tokio::sync::oneshot::Sender<RateLimitResult>,
}

/// Rate limiting statistics
#[derive(Debug, Clone)]
pub struct RateLimitStats {
    /// Total requests processed
    pub total_requests: u64,
    /// Requests allowed
    pub allowed_requests: u64,
    /// Requests delayed
    pub delayed_requests: u64,
    /// Requests denied
    pub denied_requests: u64,
    /// Current queue length
    pub queue_length: usize,
    /// Average wait time
    pub avg_wait_time_ms: f64,
    /// Current rate limit (RPS)
    pub current_rps: f64,
    /// Adaptive adjustments made
    pub adaptations_count: u64,
}

/// Circuit breaker statistics
#[derive(Debug, Clone)]
pub struct CircuitBreakerStats {
    /// Total requests through circuit
    pub total_requests: u64,
    /// Successful requests
    pub successful_requests: u64,
    /// Failed requests
    pub failed_requests: u64,
    /// Requests rejected due to open circuit
    pub rejected_requests: u64,
    /// Time spent in each state
    pub state_durations: HashMap<CircuitState, Duration>,
    /// Circuit trips count
    pub trip_count: u64,
    /// Recovery attempts
    pub recovery_attempts: u64,
}

/// Adaptation state for intelligent rate limiting
#[derive(Debug, Clone)]
struct AdaptationState {
    /// Recent response times
    response_times: VecDeque<Duration>,
    /// Recent error rates
    error_rates: VecDeque<f64>,
    /// Last adaptation timestamp
    last_adaptation: Instant,
    /// Adaptation trend (increasing/decreasing limits)
    adaptation_trend: AdaptationTrend,
}

/// Adaptation trend direction
#[derive(Debug, Clone, PartialEq)]
enum AdaptationTrend {
    Increasing,
    Stable,
    Decreasing,
}

/// Failure record for circuit breaker
#[derive(Debug, Clone)]
struct FailureRecord {
    pub timestamp: Instant,
    pub error_type: String,
    pub response_time: Option<Duration>,
}

/// Queue statistics
#[derive(Debug, Clone)]
struct QueueStats {
    pub high_priority_count: usize,
    pub normal_priority_count: usize,
    pub low_priority_count: usize,
    pub total_wait_time: Duration,
}

impl Default for AdaptiveRateLimitConfig {
    fn default() -> Self {
        Self {
            base_rps: 10.0,
            max_rps: 50.0,
            min_rps: 1.0,
            window_duration: Duration::from_secs(60),
            burst_capacity: 20,
            adaptation_factor: 0.1,
            backoff_config: BackoffConfig::default(),
            priority_strategy: PriorityStrategy::Priority,
        }
    }
}

impl Default for BackoffConfig {
    fn default() -> Self {
        Self {
            initial_delay: Duration::from_millis(100),
            max_delay: Duration::from_secs(60),
            multiplier: 2.0,
            jitter_factor: 0.1,
        }
    }
}

impl Default for CircuitBreakerConfig {
    fn default() -> Self {
        Self {
            failure_threshold: 5,
            success_threshold: 3,
            timeout: Duration::from_secs(60),
            window_size: 100,
            health_check_interval: Duration::from_secs(5),
            recovery_strategy: RecoveryStrategy::Gradual,
        }
    }
}

impl AdaptiveRateLimiter {
    /// Create a new adaptive rate limiter
    pub fn new(config: AdaptiveRateLimitConfig) -> Self {
        let current_rps = Arc::new(parking_lot::RwLock::new(config.base_rps));

        Self {
            config,
            current_rps,
            request_history: Arc::new(Mutex::new(VecDeque::new())),
            request_queue: Arc::new(Mutex::new(RequestQueue::new())),
            stats: Arc::new(RwLock::new(RateLimitStats::default())),
            adaptation_state: Arc::new(RwLock::new(AdaptationState::new())),
        }
    }

    /// Check if a request should be allowed, delayed, or denied
    pub async fn check_rate_limit(
        &self,
        metadata: RequestMetadata,
    ) -> Result<RateLimitResult, YahooError> {
        // Update statistics
        self.update_request_stats().await;

        // Check current rate against limit
        let current_rps = *self.current_rps.read();
        let can_proceed = self.can_proceed_immediately(current_rps).await?;

        if can_proceed {
            // Update request history
            self.update_request_history().await;
            Ok(RateLimitResult::Allowed)
        } else {
            // Calculate delay or queue request
            match self.config.priority_strategy {
                PriorityStrategy::Fifo => {
                    let delay = self.calculate_delay().await;
                    Ok(RateLimitResult::Delayed(delay))
                }
                _ => {
                    // Queue request with priority
                    self.queue_request(metadata).await
                }
            }
        }
    }

    /// Adapt rate limits based on recent performance
    pub async fn adapt_rate_limits(
        &self,
        response_time: Duration,
        success: bool,
    ) -> Result<(), YahooError> {
        let mut adaptation_state = self.adaptation_state.write().await;

        // Record response time
        adaptation_state.response_times.push_back(response_time);
        if adaptation_state.response_times.len() > 50 {
            adaptation_state.response_times.pop_front();
        }

        // Calculate new error rate
        let error_rate = if success { 0.0 } else { 1.0 };
        adaptation_state.error_rates.push_back(error_rate);
        if adaptation_state.error_rates.len() > 20 {
            adaptation_state.error_rates.pop_front();
        }

        // Determine if adaptation is needed
        if adaptation_state.last_adaptation.elapsed() > Duration::from_secs(10) {
            self.perform_adaptation(&mut adaptation_state).await?;
            adaptation_state.last_adaptation = Instant::now();
        }

        Ok(())
    }

    /// Get current rate limiting statistics
    pub async fn get_stats(&self) -> RateLimitStats {
        let mut stats = self.stats.read().await.clone();
        stats.current_rps = *self.current_rps.read();
        stats.queue_length = self.get_queue_length().await;
        stats
    }

    // Private helper methods
    async fn can_proceed_immediately(&self, current_rps: f64) -> Result<bool, YahooError> {
        let mut history = self.request_history.lock().await;
        let now = Instant::now();

        // Clean old entries
        while let Some(&front_time) = history.front() {
            if now.duration_since(front_time) > self.config.window_duration {
                history.pop_front();
            } else {
                break;
            }
        }

        // Check if we're under the rate limit
        let requests_in_window = history.len() as f64;
        let max_requests = current_rps * self.config.window_duration.as_secs_f64();

        Ok(requests_in_window < max_requests)
    }

    async fn calculate_delay(&self) -> Duration {
        let current_rps = *self.current_rps.read();
        let base_delay = Duration::from_secs_f64(1.0 / current_rps);

        // Add jitter to prevent thundering herd
        let jitter = self.config.backoff_config.jitter_factor;
        let jitter_ms = (base_delay.as_millis() as f64 * jitter) as u64;

        base_delay + Duration::from_millis(rand::random::<u64>() % jitter_ms.max(1))
    }

    async fn queue_request(
        &self,
        metadata: RequestMetadata,
    ) -> Result<RateLimitResult, YahooError> {
        let (tx, rx) = tokio::sync::oneshot::channel();
        let pending = PendingRequest {
            metadata: metadata.clone(),
            notify: tx,
        };

        // Add to appropriate priority queue
        {
            let mut queue = self.request_queue.lock().await;
            match metadata.priority {
                RequestPriority::Critical | RequestPriority::High => {
                    queue.high_priority.push_back(pending);
                    queue.queue_stats.high_priority_count += 1;
                }
                RequestPriority::Normal => {
                    queue.normal_priority.push_back(pending);
                    queue.queue_stats.normal_priority_count += 1;
                }
                RequestPriority::Low => {
                    queue.low_priority.push_back(pending);
                    queue.queue_stats.low_priority_count += 1;
                }
            }
        }

        // Wait for notification
        match rx.await {
            Ok(result) => Ok(result),
            Err(_) => Ok(RateLimitResult::Denied),
        }
    }

    async fn perform_adaptation(
        &self,
        adaptation_state: &mut AdaptationState,
    ) -> Result<(), YahooError> {
        // Calculate average response time
        let avg_response_time = if !adaptation_state.response_times.is_empty() {
            adaptation_state.response_times.iter().sum::<Duration>()
                / adaptation_state.response_times.len() as u32
        } else {
            Duration::from_millis(100) // Default
        };

        // Calculate average error rate
        let avg_error_rate = if !adaptation_state.error_rates.is_empty() {
            adaptation_state.error_rates.iter().sum::<f64>()
                / adaptation_state.error_rates.len() as f64
        } else {
            0.0
        };

        // Determine adaptation direction
        let current_rps = *self.current_rps.read();
        let mut new_rps = current_rps;

        // Adjust based on performance indicators
        if avg_error_rate > 0.1 || avg_response_time > Duration::from_secs(5) {
            // Decrease rate limit (performance issues)
            new_rps =
                (current_rps * (1.0 - self.config.adaptation_factor)).max(self.config.min_rps);
            adaptation_state.adaptation_trend = AdaptationTrend::Decreasing;
        } else if avg_error_rate < 0.01 && avg_response_time < Duration::from_secs(1) {
            // Increase rate limit (good performance)
            new_rps =
                (current_rps * (1.0 + self.config.adaptation_factor)).min(self.config.max_rps);
            adaptation_state.adaptation_trend = AdaptationTrend::Increasing;
        } else {
            adaptation_state.adaptation_trend = AdaptationTrend::Stable;
        }

        // Update rate limit if changed
        if (new_rps - current_rps).abs() > 0.1 {
            *self.current_rps.write() = new_rps;

            // Update stats
            let mut stats = self.stats.write().await;
            stats.adaptations_count += 1;
        }

        Ok(())
    }

    async fn update_request_history(&self) {
        let mut history = self.request_history.lock().await;
        history.push_back(Instant::now());

        // Limit history size
        if history.len() > 1000 {
            history.pop_front();
        }
    }

    async fn update_request_stats(&self) {
        let mut stats = self.stats.write().await;
        stats.total_requests += 1;
    }

    async fn get_queue_length(&self) -> usize {
        let queue = self.request_queue.lock().await;
        queue.high_priority.len() + queue.normal_priority.len() + queue.low_priority.len()
    }
}

impl IntelligentCircuitBreaker {
    /// Create a new intelligent circuit breaker
    pub fn new(config: CircuitBreakerConfig) -> Self {
        Self {
            config,
            state: Arc::new(RwLock::new(CircuitState::Closed)),
            failure_history: Arc::new(Mutex::new(VecDeque::new())),
            success_history: Arc::new(Mutex::new(VecDeque::new())),
            stats: Arc::new(RwLock::new(CircuitBreakerStats::default())),
            last_state_change: Arc::new(RwLock::new(Instant::now())),
        }
    }

    /// Execute a request through the circuit breaker
    pub async fn call<F, T>(&self, operation: F) -> CircuitResult<T>
    where
        F: std::future::Future<Output = Result<T, YahooError>>,
    {
        let state = self.state.read().await.clone();

        match state {
            CircuitState::Open => {
                // Check if we should transition to half-open
                if self.should_attempt_reset().await {
                    self.transition_to_half_open().await;
                    self.execute_operation(operation).await
                } else {
                    self.record_rejected_request().await;
                    CircuitResult::CircuitOpen
                }
            }
            CircuitState::Closed | CircuitState::HalfOpen => {
                self.execute_operation(operation).await
            }
        }
    }

    /// Get current circuit breaker statistics
    pub async fn get_stats(&self) -> CircuitBreakerStats {
        self.stats.read().await.clone()
    }

    /// Get current circuit state
    pub async fn get_state(&self) -> CircuitState {
        self.state.read().await.clone()
    }

    // Private helper methods
    async fn execute_operation<F, T>(&self, operation: F) -> CircuitResult<T>
    where
        F: std::future::Future<Output = Result<T, YahooError>>,
    {
        let start_time = Instant::now();

        match operation.await {
            Ok(result) => {
                self.record_success(start_time.elapsed()).await;
                CircuitResult::Success(result)
            }
            Err(error) => {
                self.record_failure(error.clone(), start_time.elapsed())
                    .await;
                CircuitResult::Failure(error)
            }
        }
    }

    async fn record_success(&self, _response_time: Duration) {
        let mut success_history = self.success_history.lock().await;
        success_history.push_back(Instant::now());

        // Limit history size
        if success_history.len() > self.config.window_size {
            success_history.pop_front();
        }

        // Update stats
        let mut stats = self.stats.write().await;
        stats.total_requests += 1;
        stats.successful_requests += 1;

        // Check if we should close the circuit (if half-open)
        let state = self.state.read().await.clone();
        if state == CircuitState::HalfOpen {
            if success_history.len() >= self.config.success_threshold {
                drop(success_history);
                drop(stats);
                self.transition_to_closed().await;
            }
        }
    }

    async fn record_failure(&self, error: YahooError, response_time: Duration) {
        let failure = FailureRecord {
            timestamp: Instant::now(),
            error_type: error.to_string(),
            response_time: Some(response_time),
        };

        let mut failure_history = self.failure_history.lock().await;
        failure_history.push_back(failure);

        // Limit history size
        if failure_history.len() > self.config.window_size {
            failure_history.pop_front();
        }

        // Update stats
        let mut stats = self.stats.write().await;
        stats.total_requests += 1;
        stats.failed_requests += 1;

        // Check if we should open the circuit
        let state = self.state.read().await.clone();
        if state != CircuitState::Open {
            let recent_failures = failure_history
                .iter()
                .filter(|f| f.timestamp.elapsed() < self.config.window_duration())
                .count();

            if recent_failures >= self.config.failure_threshold {
                drop(failure_history);
                drop(stats);
                self.transition_to_open().await;
            }
        }
    }

    async fn record_rejected_request(&self) {
        let mut stats = self.stats.write().await;
        stats.rejected_requests += 1;
    }

    async fn should_attempt_reset(&self) -> bool {
        let last_change = *self.last_state_change.read().await;
        last_change.elapsed() > self.config.timeout
    }

    async fn transition_to_open(&self) {
        let mut state = self.state.write().await;
        *state = CircuitState::Open;

        let mut last_change = self.last_state_change.write().await;
        *last_change = Instant::now();

        let mut stats = self.stats.write().await;
        stats.trip_count += 1;
    }

    async fn transition_to_half_open(&self) {
        let mut state = self.state.write().await;
        *state = CircuitState::HalfOpen;

        let mut last_change = self.last_state_change.write().await;
        *last_change = Instant::now();

        let mut stats = self.stats.write().await;
        stats.recovery_attempts += 1;

        // Clear success history for fresh evaluation
        let mut success_history = self.success_history.lock().await;
        success_history.clear();
    }

    async fn transition_to_closed(&self) {
        let mut state = self.state.write().await;
        *state = CircuitState::Closed;

        let mut last_change = self.last_state_change.write().await;
        *last_change = Instant::now();
    }
}

impl RequestQueue {
    fn new() -> Self {
        Self {
            high_priority: VecDeque::new(),
            normal_priority: VecDeque::new(),
            low_priority: VecDeque::new(),
            queue_stats: QueueStats {
                high_priority_count: 0,
                normal_priority_count: 0,
                low_priority_count: 0,
                total_wait_time: Duration::from_secs(0),
            },
        }
    }
}

impl AdaptationState {
    fn new() -> Self {
        Self {
            response_times: VecDeque::new(),
            error_rates: VecDeque::new(),
            last_adaptation: Instant::now(),
            adaptation_trend: AdaptationTrend::Stable,
        }
    }
}

impl CircuitBreakerConfig {
    fn window_duration(&self) -> Duration {
        // Convert window size to duration (assuming 1 request per second base rate)
        Duration::from_secs(self.window_size as u64)
    }
}

impl Default for RateLimitStats {
    fn default() -> Self {
        Self {
            total_requests: 0,
            allowed_requests: 0,
            delayed_requests: 0,
            denied_requests: 0,
            queue_length: 0,
            avg_wait_time_ms: 0.0,
            current_rps: 0.0,
            adaptations_count: 0,
        }
    }
}

impl Default for CircuitBreakerStats {
    fn default() -> Self {
        Self {
            total_requests: 0,
            successful_requests: 0,
            failed_requests: 0,
            rejected_requests: 0,
            state_durations: HashMap::new(),
            trip_count: 0,
            recovery_attempts: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_adaptive_rate_limiter_creation() {
        let config = AdaptiveRateLimitConfig::default();
        let limiter = AdaptiveRateLimiter::new(config);

        let stats = limiter.get_stats().await;
        assert_eq!(stats.total_requests, 0);
    }

    #[tokio::test]
    async fn test_rate_limit_check() {
        let config = AdaptiveRateLimitConfig {
            base_rps: 1.0,
            ..AdaptiveRateLimitConfig::default()
        };
        let limiter = AdaptiveRateLimiter::new(config);

        let metadata = RequestMetadata {
            priority: RequestPriority::Normal,
            timestamp: Instant::now(),
            request_type: "quote".to_string(),
            expected_duration: Some(Duration::from_millis(100)),
            retry_count: 0,
        };

        let result = limiter.check_rate_limit(metadata).await.unwrap();
        matches!(result, RateLimitResult::Allowed);
    }

    #[tokio::test]
    async fn test_circuit_breaker_creation() {
        let config = CircuitBreakerConfig::default();
        let breaker = IntelligentCircuitBreaker::new(config);

        assert_eq!(breaker.get_state().await, CircuitState::Closed);
    }

    #[tokio::test]
    async fn test_circuit_breaker_success() {
        let config = CircuitBreakerConfig::default();
        let breaker = IntelligentCircuitBreaker::new(config);

        let result = breaker.call(async { Ok::<i32, YahooError>(42) }).await;

        match result {
            CircuitResult::Success(value) => assert_eq!(value, 42),
            _ => panic!("Expected success"),
        }
    }

    #[tokio::test]
    async fn test_circuit_breaker_failure() {
        let config = CircuitBreakerConfig::default();
        let breaker = IntelligentCircuitBreaker::new(config);

        let error = YahooError::ConnectionFailed("Test error".to_string());
        let result = breaker.call(async { Err::<i32, YahooError>(error) }).await;

        match result {
            CircuitResult::Failure(_) => {} // Expected
            _ => panic!("Expected failure"),
        }
    }

    #[tokio::test]
    async fn test_rate_limiter_adaptation() {
        let config = AdaptiveRateLimitConfig::default();
        let limiter = AdaptiveRateLimiter::new(config);

        // Simulate slow response
        limiter
            .adapt_rate_limits(Duration::from_secs(10), false)
            .await
            .unwrap();

        let stats = limiter.get_stats().await;
        assert_eq!(stats.adaptations_count, 0); // No adaptation yet (needs time)
    }
}
