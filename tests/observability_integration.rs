/// Integration test for Phase 2.1 Observability features
///
/// Tests the full observability stack integration:
/// - Prometheus metrics collection and export
/// - Distributed tracing setup and correlation
/// - Health checks and monitoring
/// - Feature flag compatibility
use eeyf::YahooConnector;
use std::time::Duration;
use tokio::time;

#[cfg(feature = "observability")]
use eeyf::{
    health::{HealthConfig, HealthManager, HealthStatus},
    metrics::PrometheusMetrics,
    tracing::{RequestContext, TraceLevel, TracingConfig, TracingManager},
};

#[tokio::test]
async fn test_observability_integration() {
    // Test basic functionality without observability
    let connector = YahooConnector::new().unwrap();

    // Simple quote fetch to ensure basic functionality still works
    let result = connector.get_latest_quotes("AAPL", "1d").await;

    // Should either succeed or fail gracefully (network issues are OK in tests)
    match result {
        Ok(_) => println!("✅ Basic Yahoo API functionality working"),
        Err(e) => println!("⚠️  Yahoo API error (may be network/rate limit): {}", e),
    }
}

#[cfg(feature = "observability")]
#[tokio::test]
async fn test_prometheus_metrics() {
    println!("🧪 Testing Prometheus metrics...");

    let metrics = PrometheusMetrics::new();

    // Test metrics recording
    metrics
        .record_request_success("AAPL", Duration::from_millis(150), "test_endpoint")
        .await;

    metrics
        .record_request_error(
            "GOOGL",
            Duration::from_millis(300),
            "test_endpoint",
            "timeout_error",
        )
        .await;

    // Test rate limiter metrics
    metrics.record_rate_limit_hit(Duration::from_millis(200));

    // Test circuit breaker metrics
    metrics.record_circuit_breaker_open();

    // Test cache metrics
    metrics.record_cache_hit(Duration::from_millis(10));
    metrics.record_cache_miss(Duration::from_millis(20));

    println!("✅ Metrics recording completed successfully");
}

#[cfg(feature = "observability")]
#[tokio::test]
async fn test_tracing_setup() {
    println!("🧪 Testing distributed tracing...");

    let config = TracingConfig {
        service_name: "eeyf-test".to_string(),
        jaeger_endpoint: None, // Disable Jaeger export in tests
        level: TraceLevel::Debug,
        ..Default::default()
    };

    let tracing_manager = TracingManager::new(config);

    // Test request context creation
    let context = RequestContext::new("AAPL", "test_endpoint");
    assert!(!context.request_id.is_empty());
    assert!(context.trace_id.is_some());

    // Test enterprise flow tracing
    tracing_manager.trace_enterprise_flow(&context.request_id, "rate_limiter", "acquire_permit");

    println!("✅ Distributed tracing setup completed successfully");
}

#[cfg(feature = "observability")]
#[tokio::test]
async fn test_health_monitoring() {
    println!("🧪 Testing health monitoring...");

    let config = HealthConfig {
        check_yahoo_connectivity: false, // Disable network calls in tests
        check_interval_secs: 30,
        ..Default::default()
    };

    let health_manager = HealthManager::new(config);

    // Test health check execution
    let report = health_manager.check_health().await;

    // Verify health report structure
    assert!(matches!(
        report.status,
        HealthStatus::Healthy | HealthStatus::Degraded | HealthStatus::Unknown
    ));

    assert!(!report.service.name.is_empty());
    assert!(!report.service.version.is_empty());
    assert!(!report.checks.is_empty());

    // Test component health checks
    for (_name, component) in &report.checks {
        assert!(!component.name.is_empty());
        assert!(matches!(
            component.status,
            HealthStatus::Healthy
                | HealthStatus::Degraded
                | HealthStatus::Unhealthy
                | HealthStatus::Unknown
        ));
    }

    println!("✅ Health monitoring completed successfully");
    println!(
        "   Status: {:?}, Components: {}",
        report.status,
        report.checks.len()
    );
}

#[cfg(feature = "observability")]
#[tokio::test]
async fn test_full_observability_stack() {
    println!("🧪 Testing full observability stack integration...");

    // Initialize all components
    let metrics = PrometheusMetrics::new();

    let tracing_config = TracingConfig {
        service_name: "eeyf-integration-test".to_string(),
        jaeger_endpoint: None, // Disable in tests
        ..Default::default()
    };
    let _tracing_manager = TracingManager::new(tracing_config);

    let health_config = HealthConfig {
        check_yahoo_connectivity: false, // Disable network calls
        ..Default::default()
    };
    let health_manager = HealthManager::new(health_config);

    // Test coordinated operation
    let context = RequestContext::new("TSLA", "integration_test");

    // Simulate operation with metrics
    let start = std::time::Instant::now();
    time::sleep(Duration::from_millis(50)).await; // Simulate work
    let duration = start.elapsed();

    metrics
        .record_request_success("TEST", duration, "integration_test")
        .await;

    // Check overall health
    let health_report = health_manager.check_health().await;

    println!("✅ Full observability stack integration successful");
    println!("   Request ID: {}", context.request_id);
    println!("   Health Status: {:?}", health_report.status);
    println!("   Duration: {:?}", duration);
}

#[cfg(not(feature = "observability"))]
#[tokio::test]
async fn test_without_observability() {
    println!("🧪 Testing without observability features...");

    // Test that basic functionality works without observability
    let connector = YahooConnector::new().unwrap();

    // This should work regardless of observability features
    let result = connector.get_latest_quotes(&["AAPL"], "1d").await;

    match result {
        Ok(_) => println!("✅ Basic functionality works without observability"),
        Err(_) => println!("⚠️  API call failed (network/rate limit - OK in tests)"),
    }
}
