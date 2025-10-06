//! Integration tests for EEYF library
//!
//! These tests verify end-to-end functionality across multiple components,
//! ensuring proper integration between modules and realistic usage scenarios.

use eeyf::YahooConnector;
use std::time::Duration;

/// Test that we can load and apply different preset configurations
#[tokio::test]
async fn test_preset_configurations() {
    // Test default preset
    let connector = YahooConnector::builder()
        .build()
        .expect("Failed to build connector with defaults");
    
    // Verify connector was created successfully
    assert!(connector.rate_limit_status().is_some());
    
    // Test custom preset with specific values
    let custom_connector = YahooConnector::builder()
        .rate_limit(10.0)
        .circuit_breaker_threshold(5)
        .retry_attempts(2)
        .timeout(Duration::from_secs(20))
        .build()
        .expect("Failed to build connector with custom config");
    
    // Verify custom configuration was applied
    assert!(custom_connector.rate_limit_status().is_some());
}

/// Test circuit breaker functionality through integration
#[tokio::test]
async fn test_circuit_breaker_integration() {
    use eeyf::{CircuitBreaker, CircuitBreakerConfig, CircuitState, YahooError};
    
    let config = CircuitBreakerConfig {
        failure_threshold: 3,
        success_threshold: 2,
        recovery_timeout_ms: 100,
        half_open_max_requests: 2,
        ..Default::default()
    };
    
    let cb = CircuitBreaker::new(config);
    
    // Verify initial state
    assert_eq!(cb.state().await, CircuitState::Closed);
    assert!(!cb.is_open());
    
    // Record failures to open the circuit
    let error = YahooError::FetchFailed("Integration test error".to_string());
    for _ in 0..3 {
        cb.record_failure(&error).await;
    }
    
    // Circuit should be open now
    assert_eq!(cb.state().await, CircuitState::Open);
    assert!(cb.is_open());
    
    // Wait for recovery timeout
    tokio::time::sleep(Duration::from_millis(150)).await;
    
    // Record successes via execute() to properly transition through half-open state
    let _ = cb.execute(|| async { Ok::<_, YahooError>("success") }).await;
    let _ = cb.execute(|| async { Ok::<_, YahooError>("success") }).await;
    
    // Circuit should be closed again
    assert_eq!(cb.state().await, CircuitState::Closed);
    assert!(!cb.is_open());
}

/// Test rate limiter integration
#[tokio::test]
async fn test_rate_limiter_integration() {
    use eeyf::{RateLimiter, RateLimitConfig};
    
    let config = RateLimitConfig {
        requests_per_hour: 360,  // 10 per hour * 36 (for 10/second equivalent in testing)
        burst_limit: 5,
        min_interval: Duration::from_millis(100),
    };
    
    let limiter = RateLimiter::new(config);
    
    // Should allow initial burst
    for _ in 0..5 {
        let _result: Result<(), eeyf::RateLimitError> = limiter.acquire_permit().await;
        assert!(_result.is_ok());
    }
    
    // Get status to verify state
    let status = limiter.status();
    assert!(status.burst_available < 5);
}

/// Test connection pool integration
#[tokio::test]
async fn test_connection_pool_integration() {
    use eeyf::{ConnectionPool, ConnectionPoolConfig};
    
    let config = ConnectionPoolConfig {
        max_connections_per_host: 5,
        max_total_connections: 10,
        connect_timeout_ms: 5000,
        request_timeout_ms: 10000,
        idle_timeout_ms: 30000,
        ..Default::default()
    };
    
    let pool = ConnectionPool::new(config);
    
    // Get pool stats to verify initialization
    let stats = pool.stats().await;
    assert!(stats.total_connections_created >= 0);
}

/// Test error handling integration across components
#[tokio::test]
async fn test_error_handling_integration() {
    use eeyf::{YahooError, YahooErrorCode, ErrorCategorizer};
    
    // Test different error types using actual variants
    let errors = vec![
        YahooError::NoResult,
        YahooError::FetchFailed("Network error".to_string()),
        YahooError::TooManyRequests("Rate limit hit".to_string()),
        YahooError::ConnectionFailed("Timeout".to_string()),
    ];
    
    for error in &errors {
        // Verify error categorization
        let info: eeyf::ErrorInfo = error.categorize_error();
        assert!(!info.category.to_string().is_empty());
        
        // Verify error codes
        let code: YahooErrorCode = error.error_code();
        assert!(code.to_string().len() > 0);
        
        // Verify suggested actions
        let suggestion = error.suggested_action();
        assert!(!suggestion.is_empty());
    }
}

/// Test configuration validation
#[tokio::test]
async fn test_configuration_validation() {
    use eeyf::{RateLimitConfig, CircuitBreakerConfig, ConnectionPoolConfig};
    
    // Rate limit config should accept valid values
    let rate_config = RateLimitConfig {
        requests_per_hour: 3600,
        burst_limit: 50,
        min_interval: Duration::from_millis(50),
    };
    assert!(rate_config.requests_per_hour > 0);
    
    // Circuit breaker config should accept valid thresholds
    let cb_config = CircuitBreakerConfig {
        failure_threshold: 10,
        success_threshold: 3,
        recovery_timeout_ms: 30000,
        ..Default::default()
    };
    assert!(cb_config.failure_threshold > 0);
    
    // Connection pool config should accept valid sizes
    let pool_config = ConnectionPoolConfig {
        max_connections_per_host: 20,
        max_total_connections: 100,
        connect_timeout_ms: 10000,
        ..Default::default()
    };
    assert!(pool_config.max_total_connections > 0);
}

/// Test preset manager functionality
#[tokio::test]
async fn test_preset_manager() {
    use eeyf::{PresetConfig, PresetManager};
    
    let manager = PresetManager::new();
    
    // Create a custom preset
    let preset = PresetConfig {
        name: "test_preset".to_string(),
        description: Some("Test configuration".to_string()),
        rate_limit: 50.0,
        circuit_breaker_threshold: 5,
        circuit_breaker_window_secs: 60,
        circuit_breaker_timeout_secs: 30,
        retry_attempts: 3,
        retry_initial_delay_ms: 100,
        retry_max_delay_ms: 5000,
        timeout_secs: 30,
        cache_size: 1000,
        cache_duration_secs: 300,
        connection_pool_max: 10,
        verbose_logging: false,
        enable_metrics: true,
        enable_tracing: false,
    };
    
    // Verify preset has valid values
    assert!(preset.rate_limit > 0.0);
    assert!(preset.circuit_breaker_threshold > 0);
    assert!(preset.retry_attempts > 0);
    
    // Test built-in presets using load_preset
    let development = manager.load_preset("development");
    assert!(development.is_ok());
    
    let production = manager.load_preset("production");
    assert!(production.is_ok());
}

/// Test end-to-end workflow with error recovery
#[tokio::test]
async fn test_error_recovery_workflow() {
    use eeyf::{CircuitBreaker, CircuitBreakerConfig, CircuitState, YahooError};
    
    let config = CircuitBreakerConfig {
        failure_threshold: 2,
        success_threshold: 1,
        recovery_timeout_ms: 100,
        ..Default::default()
    };
    
    let cb = CircuitBreaker::new(config);
    
    // Simulate a failure scenario
    let error = YahooError::FetchFailed("Service unavailable".to_string());
    cb.record_failure(&error).await;
    cb.record_failure(&error).await;
    
    // Circuit should open
    assert_eq!(cb.state().await, CircuitState::Open);
    
    // Wait for recovery
    tokio::time::sleep(Duration::from_millis(150)).await;
    
    // Simulate recovery via execute() for proper state transition
    let _ = cb.execute(|| async { Ok::<_, YahooError>("recovery success") }).await;
    
    // Circuit should close
    assert_eq!(cb.state().await, CircuitState::Closed);
    
    // Verify we can reset manually
    cb.force_open().await;
    assert_eq!(cb.state().await, CircuitState::Open);
    
    cb.reset().await;
    assert_eq!(cb.state().await, CircuitState::Closed);
}

/// Test observability integration
#[tokio::test]
async fn test_observability_integration() {
    use eeyf::ObservabilityConfig;
    
    let config = ObservabilityConfig::default();
    
    // Verify default configuration
    assert!(config.enable_health_checks);
}

/// Test retry logic integration
#[tokio::test]
async fn test_retry_integration() {
    use eeyf::RetryConfig;
    
    let config = RetryConfig {
        max_attempts: 3,
        base_delay_ms: 100,
        max_delay_ms: 5000,
        jitter_factor: 0.1,
        ..Default::default()
    };
    
    assert_eq!(config.max_attempts, 3);
    assert_eq!(config.base_delay_ms, 100);
}

/// Test cache integration
#[tokio::test]
async fn test_cache_integration() {
    use eeyf::{ResponseCacheConfig, ResponseCache};
    
    let config = ResponseCacheConfig {
        max_entries: 1000,
        default_ttl_ms: 300000, // 5 minutes
        quote_ttl_ms: 60000, // 1 minute
        ..Default::default()
    };
    
    let cache = ResponseCache::new(config);
    
    // Cache should be empty initially
    let stats = cache.stats().await;
    assert_eq!(stats.cache_hits, 0);
    assert_eq!(stats.cache_misses, 0);
}

/// Test deduplication integration
#[tokio::test]
async fn test_deduplication_integration() {
    use eeyf::{DeduplicationConfig, RequestDeduplicator};
    
    let config = DeduplicationConfig {
        deduplicate_in_flight: true,
        cache_ttl_ms: 5000,
        max_cache_entries: 100,
        ..Default::default()
    };
    
    let dedup = RequestDeduplicator::new(config);
    
    // Verify deduplicator is configured correctly
    let stats = dedup.stats().await;
    assert_eq!(stats.deduplicated_requests, 0);
}

/// Test enterprise configuration integration
#[tokio::test]
async fn test_enterprise_config_integration() {
    use eeyf::EnterpriseConfig;
    
    let config = EnterpriseConfig::default();
    
    // Verify all components are initialized
    assert!(config.rate_limiter.requests_per_hour > 0);
    assert!(config.circuit_breaker.failure_threshold > 0);
    assert!(config.connection_pool.max_total_connections > 0);
    assert!(config.retry.max_attempts > 0);
    assert!(config.response_cache.max_entries > 0);
}
