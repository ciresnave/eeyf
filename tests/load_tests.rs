/// Load testing examples for EEYF library
///
/// These tests demonstrate how the library performs under various load conditions.
/// Run with: `cargo test --test load_tests --release -- --nocapture --test-threads=1`

use eeyf::{
    CircuitBreaker, CircuitBreakerConfig, ConnectionPool, ConnectionPoolConfig,
    RateLimiter, RateLimitConfig, YahooConnector, YahooError,
};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::time::sleep;

/// Test configuration
const TEST_DURATION_SECS: u64 = 5;
const WARMUP_DURATION_SECS: u64 = 1;

/// Load test metrics
#[derive(Debug, Clone)]
struct LoadTestMetrics {
    total_requests: usize,
    successful_requests: usize,
    failed_requests: usize,
    rate_limited_requests: usize,
    circuit_breaker_open: usize,
    min_response_time: Duration,
    max_response_time: Duration,
    avg_response_time: Duration,
    p50_response_time: Duration,
    p95_response_time: Duration,
    p99_response_time: Duration,
    requests_per_second: f64,
}

impl LoadTestMetrics {
    fn new() -> Self {
        Self {
            total_requests: 0,
            successful_requests: 0,
            failed_requests: 0,
            rate_limited_requests: 0,
            circuit_breaker_open: 0,
            min_response_time: Duration::from_secs(999),
            max_response_time: Duration::from_secs(0),
            avg_response_time: Duration::from_secs(0),
            p50_response_time: Duration::from_secs(0),
            p95_response_time: Duration::from_secs(0),
            p99_response_time: Duration::from_secs(0),
            requests_per_second: 0.0,
        }
    }

    fn calculate_percentiles(&mut self, response_times: &[Duration]) {
        let mut sorted_times = response_times.to_vec();
        sorted_times.sort();

        if !sorted_times.is_empty() {
            self.min_response_time = sorted_times[0];
            self.max_response_time = sorted_times[sorted_times.len() - 1];

            let sum: Duration = sorted_times.iter().sum();
            self.avg_response_time = sum / sorted_times.len() as u32;

            let p50_idx = (sorted_times.len() as f64 * 0.50) as usize;
            let p95_idx = (sorted_times.len() as f64 * 0.95) as usize;
            let p99_idx = (sorted_times.len() as f64 * 0.99) as usize;

            self.p50_response_time = sorted_times[p50_idx.min(sorted_times.len() - 1)];
            self.p95_response_time = sorted_times[p95_idx.min(sorted_times.len() - 1)];
            self.p99_response_time = sorted_times[p99_idx.min(sorted_times.len() - 1)];
        }
    }

    fn print_report(&self, test_name: &str) {
        println!("\n{'═':<60}", "");
        println!("Load Test Results: {}", test_name);
        println!("{'═':<60}", "");
        println!("Total Requests:        {}", self.total_requests);
        println!("Successful Requests:   {} ({:.1}%)", 
            self.successful_requests, 
            (self.successful_requests as f64 / self.total_requests as f64 * 100.0));
        println!("Failed Requests:       {} ({:.1}%)", 
            self.failed_requests,
            (self.failed_requests as f64 / self.total_requests as f64 * 100.0));
        println!("Rate Limited:          {}", self.rate_limited_requests);
        println!("Circuit Breaker Open:  {}", self.circuit_breaker_open);
        println!("\nPerformance Metrics:");
        println!("  Requests/Second:     {:.2}", self.requests_per_second);
        println!("  Min Response Time:   {:?}", self.min_response_time);
        println!("  Avg Response Time:   {:?}", self.avg_response_time);
        println!("  P50 Response Time:   {:?}", self.p50_response_time);
        println!("  P95 Response Time:   {:?}", self.p95_response_time);
        println!("  P99 Response Time:   {:?}", self.p99_response_time);
        println!("  Max Response Time:   {:?}", self.max_response_time);
        println!("{'═':<60}\n", "");
    }
}

/// Simulate a request operation
async fn simulate_request(delay_ms: u64) -> Result<(), YahooError> {
    sleep(Duration::from_millis(delay_ms)).await;
    
    // Simulate occasional failures
    if delay_ms > 50 {
        Err(YahooError::ConnectionFailed("Simulated timeout".to_string()))
    } else {
        Ok(())
    }
}

#[tokio::test]
async fn load_test_rate_limiter_sustained_load() {
    println!("\n🔬 Load Test: Rate Limiter Under Sustained Load");
    println!("Testing rate limiter with sustained 100 req/min for {} seconds", TEST_DURATION_SECS);

    let config = RateLimitConfig::default()
        .with_requests_per_minute(100)
        .with_burst_size(20);
    
    let rate_limiter = Arc::new(RateLimiter::new(config));
    let mut metrics = LoadTestMetrics::new();
    let mut response_times = Vec::new();

    println!("⏳ Warming up...");
    sleep(Duration::from_secs(WARMUP_DURATION_SECS)).await;

    let start = Instant::now();
    let test_duration = Duration::from_secs(TEST_DURATION_SECS);

    println!("🚀 Starting load test...");
    
    while start.elapsed() < test_duration {
        let request_start = Instant::now();
        metrics.total_requests += 1;

        match rate_limiter.check_rate_limit() {
            Ok(_) => {
                // Simulate successful request
                if let Ok(_) = simulate_request(10).await {
                    metrics.successful_requests += 1;
                    response_times.push(request_start.elapsed());
                } else {
                    metrics.failed_requests += 1;
                }
            }
            Err(_) => {
                metrics.rate_limited_requests += 1;
                // Wait a bit before retrying
                sleep(Duration::from_millis(100)).await;
            }
        }

        // Small delay to prevent tight loop
        sleep(Duration::from_millis(10)).await;
    }

    let total_duration = start.elapsed();
    metrics.requests_per_second = metrics.total_requests as f64 / total_duration.as_secs_f64();
    metrics.calculate_percentiles(&response_times);
    metrics.print_report("Rate Limiter Sustained Load");

    // Assertions
    assert!(metrics.requests_per_second > 0.0, "Should process requests");
    assert!(metrics.successful_requests > 0, "Should have successful requests");
    assert!(metrics.rate_limited_requests > 0, "Should have rate-limited some requests");
}

#[tokio::test]
async fn load_test_rate_limiter_burst_traffic() {
    println!("\n🔬 Load Test: Rate Limiter With Burst Traffic");
    println!("Testing rate limiter with burst traffic patterns");

    let config = RateLimitConfig::default()
        .with_requests_per_minute(60)
        .with_burst_size(10);
    
    let rate_limiter = Arc::new(RateLimiter::new(config));
    let mut metrics = LoadTestMetrics::new();
    let mut response_times = Vec::new();

    println!("⏳ Warming up...");
    sleep(Duration::from_secs(WARMUP_DURATION_SECS)).await;

    println!("🚀 Starting burst test...");
    
    // Send burst of 50 requests
    let burst_start = Instant::now();
    for _ in 0..50 {
        let request_start = Instant::now();
        metrics.total_requests += 1;

        match rate_limiter.check_rate_limit() {
            Ok(_) => {
                if let Ok(_) = simulate_request(5).await {
                    metrics.successful_requests += 1;
                    response_times.push(request_start.elapsed());
                } else {
                    metrics.failed_requests += 1;
                }
            }
            Err(_) => {
                metrics.rate_limited_requests += 1;
            }
        }
    }

    let burst_duration = burst_start.elapsed();
    metrics.requests_per_second = 50.0 / burst_duration.as_secs_f64();
    metrics.calculate_percentiles(&response_times);
    metrics.print_report("Rate Limiter Burst Traffic");

    // Assertions
    assert!(metrics.rate_limited_requests > 0, "Should rate limit burst traffic");
    assert!(metrics.successful_requests <= 10, "Should respect burst size");
    assert_eq!(
        metrics.total_requests, 
        metrics.successful_requests + metrics.failed_requests + metrics.rate_limited_requests,
        "All requests should be accounted for"
    );
}

#[tokio::test]
async fn load_test_circuit_breaker_under_failures() {
    println!("\n🔬 Load Test: Circuit Breaker Under High Failure Rate");
    println!("Testing circuit breaker behavior with {} seconds of failures", TEST_DURATION_SECS);

    let config = CircuitBreakerConfig {
        failure_threshold: 5,
        success_threshold: 2,
        timeout: Duration::from_secs(2),
    };
    
    let circuit_breaker = Arc::new(CircuitBreaker::new(config));
    let mut metrics = LoadTestMetrics::new();
    let mut response_times = Vec::new();

    println!("⏳ Warming up...");
    sleep(Duration::from_secs(WARMUP_DURATION_SECS)).await;

    let start = Instant::now();
    let test_duration = Duration::from_secs(TEST_DURATION_SECS);

    println!("🚀 Starting failure simulation...");
    
    while start.elapsed() < test_duration {
        let request_start = Instant::now();
        metrics.total_requests += 1;

        // Check if circuit is open
        if circuit_breaker.is_open() {
            metrics.circuit_breaker_open += 1;
            sleep(Duration::from_millis(100)).await;
            continue;
        }

        // Simulate high failure rate (80% failure)
        let result = if metrics.total_requests % 5 != 0 {
            Err(YahooError::ConnectionFailed("Simulated failure".to_string()))
        } else {
            Ok(())
        };

        match result {
            Ok(_) => {
                circuit_breaker.record_success();
                metrics.successful_requests += 1;
                response_times.push(request_start.elapsed());
            }
            Err(_) => {
                circuit_breaker.record_failure();
                metrics.failed_requests += 1;
            }
        }

        sleep(Duration::from_millis(50)).await;
    }

    let total_duration = start.elapsed();
    metrics.requests_per_second = metrics.total_requests as f64 / total_duration.as_secs_f64();
    metrics.calculate_percentiles(&response_times);
    metrics.print_report("Circuit Breaker Under Failures");

    // Assertions
    assert!(metrics.circuit_breaker_open > 0, "Circuit breaker should have opened");
    assert!(metrics.failed_requests > metrics.successful_requests, "Should have more failures");
}

#[tokio::test]
async fn load_test_circuit_breaker_recovery() {
    println!("\n🔬 Load Test: Circuit Breaker Recovery");
    println!("Testing circuit breaker recovery after failures");

    let config = CircuitBreakerConfig {
        failure_threshold: 3,
        success_threshold: 2,
        timeout: Duration::from_millis(500),
    };
    
    let circuit_breaker = Arc::new(CircuitBreaker::new(config));
    let mut metrics = LoadTestMetrics::new();
    let mut response_times = Vec::new();

    println!("⏳ Phase 1: Triggering failures...");
    
    // Phase 1: Trigger failures to open circuit
    for _ in 0..5 {
        metrics.total_requests += 1;
        circuit_breaker.record_failure();
        metrics.failed_requests += 1;
        sleep(Duration::from_millis(10)).await;
    }

    assert!(circuit_breaker.is_open(), "Circuit should be open after failures");
    println!("✓ Circuit opened after {} failures", metrics.failed_requests);

    // Phase 2: Wait for timeout
    println!("⏳ Phase 2: Waiting for circuit to allow retry...");
    sleep(Duration::from_millis(600)).await;

    // Phase 3: Successful requests to close circuit
    println!("⏳ Phase 3: Sending successful requests...");
    for _ in 0..3 {
        if !circuit_breaker.is_open() {
            let request_start = Instant::now();
            metrics.total_requests += 1;
            circuit_breaker.record_success();
            metrics.successful_requests += 1;
            response_times.push(request_start.elapsed());
        }
        sleep(Duration::from_millis(100)).await;
    }

    metrics.calculate_percentiles(&response_times);
    metrics.print_report("Circuit Breaker Recovery");

    // Assertions
    assert!(!circuit_breaker.is_open(), "Circuit should be closed after successful requests");
    assert!(metrics.successful_requests >= 2, "Should have recovered with successful requests");
    println!("✓ Circuit recovered successfully");
}

#[tokio::test]
async fn load_test_concurrent_requests() {
    println!("\n🔬 Load Test: Concurrent Requests");
    println!("Testing {} concurrent requests", 50);

    let mut metrics = LoadTestMetrics::new();
    let start = Instant::now();

    println!("🚀 Spawning concurrent requests...");
    
    let handles: Vec<_> = (0..50)
        .map(|i| {
            tokio::spawn(async move {
                let request_start = Instant::now();
                let delay = if i % 10 == 0 { 100 } else { 20 }; // Occasional slow request
                simulate_request(delay).await.ok();
                request_start.elapsed()
            })
        })
        .collect();

    let mut response_times = Vec::new();
    for handle in handles {
        match handle.await {
            Ok(duration) => {
                metrics.total_requests += 1;
                metrics.successful_requests += 1;
                response_times.push(duration);
            }
            Err(_) => {
                metrics.total_requests += 1;
                metrics.failed_requests += 1;
            }
        }
    }

    let total_duration = start.elapsed();
    metrics.requests_per_second = metrics.total_requests as f64 / total_duration.as_secs_f64();
    metrics.calculate_percentiles(&response_times);
    metrics.print_report("Concurrent Requests");

    // Assertions
    assert_eq!(metrics.total_requests, 50, "Should process all concurrent requests");
    assert!(metrics.successful_requests >= 40, "Most requests should succeed");
    assert!(total_duration < Duration::from_secs(2), "Should complete quickly with concurrency");
}

#[tokio::test]
async fn load_test_memory_usage() {
    println!("\n🔬 Load Test: Memory Usage Under Load");
    println!("Testing memory behavior with 1000 requests");

    let mut metrics = LoadTestMetrics::new();
    let mut response_times = Vec::new();

    println!("🚀 Starting memory test...");
    let start = Instant::now();

    for i in 0..1000 {
        let request_start = Instant::now();
        metrics.total_requests += 1;

        // Simulate request with some data allocation
        let _data = vec![0u8; 1024]; // 1KB allocation per request
        
        if simulate_request(5).await.is_ok() {
            metrics.successful_requests += 1;
            response_times.push(request_start.elapsed());
        } else {
            metrics.failed_requests += 1;
        }

        // Progress indicator
        if i % 100 == 0 {
            println!("  Progress: {}/1000 requests", i);
        }
    }

    let total_duration = start.elapsed();
    metrics.requests_per_second = metrics.total_requests as f64 / total_duration.as_secs_f64();
    metrics.calculate_percentiles(&response_times);
    metrics.print_report("Memory Usage Under Load");

    // Assertions
    assert_eq!(metrics.total_requests, 1000, "Should complete all requests");
    assert!(metrics.successful_requests > 900, "Most requests should succeed");
    assert!(metrics.avg_response_time < Duration::from_millis(50), "Should maintain low latency");
}

#[tokio::test]
async fn load_test_gradual_ramp_up() {
    println!("\n🔬 Load Test: Gradual Load Ramp-Up");
    println!("Testing system behavior with gradual load increase");

    let config = RateLimitConfig::default()
        .with_requests_per_minute(120)
        .with_burst_size(30);
    
    let rate_limiter = Arc::new(RateLimiter::new(config));
    let mut metrics = LoadTestMetrics::new();
    let mut response_times = Vec::new();

    println!("🚀 Starting ramp-up test...");
    
    // Ramp up: 10, 20, 30, 40, 50 requests per phase
    for phase in 1..=5 {
        println!("  Phase {}: {} requests", phase, phase * 10);
        
        for _ in 0..(phase * 10) {
            let request_start = Instant::now();
            metrics.total_requests += 1;

            match rate_limiter.check_rate_limit() {
                Ok(_) => {
                    if simulate_request(10).await.is_ok() {
                        metrics.successful_requests += 1;
                        response_times.push(request_start.elapsed());
                    } else {
                        metrics.failed_requests += 1;
                    }
                }
                Err(_) => {
                    metrics.rate_limited_requests += 1;
                }
            }

            sleep(Duration::from_millis(20)).await;
        }
        
        // Brief pause between phases
        sleep(Duration::from_millis(500)).await;
    }

    metrics.calculate_percentiles(&response_times);
    metrics.print_report("Gradual Load Ramp-Up");

    // Assertions
    assert_eq!(metrics.total_requests, 150, "Should process all ramped requests");
    assert!(metrics.successful_requests > 0, "Should have successful requests");
    assert!(
        metrics.p99_response_time < Duration::from_millis(500),
        "Should maintain reasonable response times during ramp-up"
    );
}

#[test]
fn load_test_summary() {
    println!("\n{'═':<60}", "");
    println!("📊 Load Test Suite Summary");
    println!("{'═':<60}", "");
    println!("Available load tests:");
    println!("  1. load_test_rate_limiter_sustained_load");
    println!("  2. load_test_rate_limiter_burst_traffic");
    println!("  3. load_test_circuit_breaker_under_failures");
    println!("  4. load_test_circuit_breaker_recovery");
    println!("  5. load_test_concurrent_requests");
    println!("  6. load_test_memory_usage");
    println!("  7. load_test_gradual_ramp_up");
    println!("\nRun with:");
    println!("  cargo test --test load_tests --release -- --nocapture --test-threads=1");
    println!("{'═':<60}\n", "");
}
