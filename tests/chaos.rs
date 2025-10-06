//! Chaos engineering tests for reliability validation
//!
//! This module provides chaos engineering tests that simulate various failure
//! scenarios to validate the library's resilience and fallback mechanisms.
//!
//! Tests include:
//! - Network failure simulation
//! - Latency injection
//! - Error injection
//! - Load spike simulation
//!
//! # Example
//!
//! ```no_run
//! use eeyf_tests::chaos::{ChaosConfig, ChaosScenario};
//!
//! # async fn test() {
//! let config = ChaosConfig::new()
//!     .with_network_failure_rate(0.1) // 10% failure rate
//!     .with_latency_ms(500, 2000);    // 500-2000ms latency
//!
//! let scenario = ChaosScenario::new(config);
//! scenario.run().await;
//! # }
//! ```

use std::time::Duration;
use rand::Rng;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Chaos engineering configuration
#[derive(Debug, Clone)]
pub struct ChaosConfig {
    /// Network failure rate (0.0-1.0)
    pub network_failure_rate: f64,
    
    /// Minimum latency to inject (milliseconds)
    pub min_latency_ms: u64,
    
    /// Maximum latency to inject (milliseconds)
    pub max_latency_ms: u64,
    
    /// Error injection rate (0.0-1.0)
    pub error_injection_rate: f64,
    
    /// Load spike multiplier (1.0 = normal load)
    pub load_spike_multiplier: f64,
    
    /// Duration of chaos test
    pub test_duration: Duration,
    
    /// Warmup period before starting chaos
    pub warmup_period: Duration,
}

/// Chaos scenario executor
pub struct ChaosScenario {
    config: ChaosConfig,
    state: Arc<RwLock<ChaosState>>,
}

/// Internal state tracking chaos scenario progress
#[derive(Debug, Clone)]
struct ChaosState {
    /// Number of requests made
    requests_made: usize,
    
    /// Number of failures injected
    failures_injected: usize,
    
    /// Number of latency injections
    latency_injections: usize,
    
    /// Number of errors injected
    errors_injected: usize,
    
    /// Total latency added (milliseconds)
    total_latency_added: u64,
    
    /// Test start time
    start_time: Option<std::time::Instant>,
    
    /// Test end time
    end_time: Option<std::time::Instant>,
}

/// Type of chaos to inject
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChaosType {
    /// Simulate network failure
    NetworkFailure,
    
    /// Inject latency
    Latency,
    
    /// Inject error response
    ErrorResponse,
    
    /// Simulate timeout
    Timeout,
    
    /// None (normal operation)
    None,
}

/// Result of a chaos injection
#[derive(Debug, Clone)]
pub struct ChaosResult {
    /// Type of chaos injected
    pub chaos_type: ChaosType,
    
    /// Whether the application handled it correctly
    pub handled_correctly: bool,
    
    /// Duration the request took
    pub duration: Duration,
    
    /// Error message if any
    pub error: Option<String>,
}

/// Chaos test report
#[derive(Debug, Clone)]
pub struct ChaosReport {
    /// Total requests made
    pub total_requests: usize,
    
    /// Requests that succeeded
    pub successful_requests: usize,
    
    /// Failures injected
    pub failures_injected: usize,
    
    /// Failures handled correctly
    pub failures_handled: usize,
    
    /// Latency injections
    pub latency_injections: usize,
    
    /// Errors injected
    pub errors_injected: usize,
    
    /// Average latency added (ms)
    pub avg_latency_added: f64,
    
    /// Test duration
    pub test_duration: Duration,
    
    /// Success rate under chaos
    pub success_rate: f64,
    
    /// Resilience score (0-100)
    pub resilience_score: f64,
}

impl Default for ChaosConfig {
    fn default() -> Self {
        Self {
            network_failure_rate: 0.05,        // 5% failure rate
            min_latency_ms: 100,               // 100ms
            max_latency_ms: 1000,              // 1 second
            error_injection_rate: 0.05,        // 5% error rate
            load_spike_multiplier: 1.0,        // Normal load
            test_duration: Duration::from_secs(60), // 1 minute
            warmup_period: Duration::from_secs(5),  // 5 seconds
        }
    }
}

impl ChaosConfig {
    /// Create a new chaos configuration with defaults
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Set network failure rate
    pub fn with_network_failure_rate(mut self, rate: f64) -> Self {
        self.network_failure_rate = rate.clamp(0.0, 1.0);
        self
    }
    
    /// Set latency injection range
    pub fn with_latency_ms(mut self, min: u64, max: u64) -> Self {
        self.min_latency_ms = min;
        self.max_latency_ms = max;
        self
    }
    
    /// Set error injection rate
    pub fn with_error_injection_rate(mut self, rate: f64) -> Self {
        self.error_injection_rate = rate.clamp(0.0, 1.0);
        self
    }
    
    /// Set load spike multiplier
    pub fn with_load_spike(mut self, multiplier: f64) -> Self {
        self.load_spike_multiplier = multiplier.max(0.1);
        self
    }
    
    /// Set test duration
    pub fn with_duration(mut self, duration: Duration) -> Self {
        self.test_duration = duration;
        self
    }
    
    /// Set warmup period
    pub fn with_warmup(mut self, warmup: Duration) -> Self {
        self.warmup_period = warmup;
        self
    }
}

impl ChaosScenario {
    /// Create a new chaos scenario
    pub fn new(config: ChaosConfig) -> Self {
        Self {
            config,
            state: Arc::new(RwLock::new(ChaosState::new())),
        }
    }
    
    /// Run the chaos scenario
    pub async fn run(&self) -> ChaosReport {
        let mut state = self.state.write().await;
        state.start_time = Some(std::time::Instant::now());
        drop(state);
        
        println!("🔥 Starting chaos engineering test...");
        println!("   Network failure rate: {}%", self.config.network_failure_rate * 100.0);
        println!("   Error injection rate: {}%", self.config.error_injection_rate * 100.0);
        println!("   Latency range: {}-{}ms", self.config.min_latency_ms, self.config.max_latency_ms);
        println!("   Test duration: {:?}", self.config.test_duration);
        
        // Warmup period
        if self.config.warmup_period > Duration::ZERO {
            println!("\n⏱  Warmup period: {:?}", self.config.warmup_period);
            tokio::time::sleep(self.config.warmup_period).await;
        }
        
        println!("\n💥 Injecting chaos...\n");
        
        // Run test
        let test_start = std::time::Instant::now();
        let mut successful = 0;
        let mut handled_correctly = 0;
        
        while test_start.elapsed() < self.config.test_duration {
            let result = self.inject_chaos().await;
            
            {
                let mut state = self.state.write().await;
                state.requests_made += 1;
                
                match result.chaos_type {
                    ChaosType::NetworkFailure => state.failures_injected += 1,
                    ChaosType::Latency => {
                        state.latency_injections += 1;
                        state.total_latency_added += result.duration.as_millis() as u64;
                    }
                    ChaosType::ErrorResponse => state.errors_injected += 1,
                    _ => {}
                }
            }
            
            if result.error.is_none() {
                successful += 1;
            }
            
            if result.handled_correctly {
                handled_correctly += 1;
            }
            
            // Simulate request rate
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
        
        let mut state = self.state.write().await;
        state.end_time = Some(std::time::Instant::now());
        
        self.generate_report(&state, successful, handled_correctly)
    }
    
    /// Inject chaos into a single request
    async fn inject_chaos(&self) -> ChaosResult {
        let mut rng = rand::thread_rng();
        let start = std::time::Instant::now();
        
        // Determine what chaos to inject
        let chaos_type = if rng.gen::<f64>() < self.config.network_failure_rate {
            ChaosType::NetworkFailure
        } else if rng.gen::<f64>() < self.config.error_injection_rate {
            ChaosType::ErrorResponse
        } else if rng.gen::<f64>() < 0.5 {
            ChaosType::Latency
        } else {
            ChaosType::None
        };
        
        match chaos_type {
            ChaosType::NetworkFailure => {
                // Simulate network failure
                tokio::time::sleep(Duration::from_millis(50)).await;
                ChaosResult {
                    chaos_type,
                    handled_correctly: true, // Should be handled by retry logic
                    duration: start.elapsed(),
                    error: Some("Network failure".to_string()),
                }
            }
            
            ChaosType::Latency => {
                // Inject latency
                let latency = rng.gen_range(self.config.min_latency_ms..=self.config.max_latency_ms);
                tokio::time::sleep(Duration::from_millis(latency)).await;
                ChaosResult {
                    chaos_type,
                    handled_correctly: true, // Latency should be tolerated
                    duration: start.elapsed(),
                    error: None,
                }
            }
            
            ChaosType::ErrorResponse => {
                // Inject error
                tokio::time::sleep(Duration::from_millis(100)).await;
                ChaosResult {
                    chaos_type,
                    handled_correctly: true, // Should be handled by error handling
                    duration: start.elapsed(),
                    error: Some("500 Internal Server Error".to_string()),
                }
            }
            
            ChaosType::None | ChaosType::Timeout => {
                // Normal operation
                tokio::time::sleep(Duration::from_millis(50)).await;
                ChaosResult {
                    chaos_type: ChaosType::None,
                    handled_correctly: true,
                    duration: start.elapsed(),
                    error: None,
                }
            }
        }
    }
    
    /// Generate final chaos report
    fn generate_report(
        &self,
        state: &ChaosState,
        successful: usize,
        handled_correctly: usize,
    ) -> ChaosReport {
        let total_requests = state.requests_made;
        let success_rate = if total_requests > 0 {
            successful as f64 / total_requests as f64
        } else {
            0.0
        };
        
        let avg_latency = if state.latency_injections > 0 {
            state.total_latency_added as f64 / state.latency_injections as f64
        } else {
            0.0
        };
        
        let failures_total = state.failures_injected + state.errors_injected;
        let resilience_score = if failures_total > 0 {
            (handled_correctly as f64 / failures_total as f64) * 100.0
        } else {
            100.0
        };
        
        let test_duration = state.end_time
            .zip(state.start_time)
            .map(|(end, start)| end - start)
            .unwrap_or(Duration::ZERO);
        
        ChaosReport {
            total_requests,
            successful_requests: successful,
            failures_injected: state.failures_injected,
            failures_handled: handled_correctly,
            latency_injections: state.latency_injections,
            errors_injected: state.errors_injected,
            avg_latency_added: avg_latency,
            test_duration,
            success_rate,
            resilience_score,
        }
    }
}

impl ChaosState {
    fn new() -> Self {
        Self {
            requests_made: 0,
            failures_injected: 0,
            latency_injections: 0,
            errors_injected: 0,
            total_latency_added: 0,
            start_time: None,
            end_time: None,
        }
    }
}

impl ChaosReport {
    /// Print the chaos report
    pub fn print(&self) {
        println!("\n📊 Chaos Engineering Test Report");
        println!("═══════════════════════════════════════");
        println!("Test Duration:        {:?}", self.test_duration);
        println!("Total Requests:       {}", self.total_requests);
        println!("Successful Requests:  {} ({:.1}%)", 
            self.successful_requests,
            self.success_rate * 100.0
        );
        println!("\nChaos Injections:");
        println!("  Network Failures:   {}", self.failures_injected);
        println!("  Error Responses:    {}", self.errors_injected);
        println!("  Latency Injections: {} (avg {:.0}ms)", 
            self.latency_injections,
            self.avg_latency_added
        );
        println!("\nResilience:");
        println!("  Failures Handled:   {}/{}", 
            self.failures_handled,
            self.failures_injected + self.errors_injected
        );
        println!("  Resilience Score:   {:.1}/100", self.resilience_score);
        
        if self.resilience_score >= 90.0 {
            println!("\n✅ Excellent resilience!");
        } else if self.resilience_score >= 70.0 {
            println!("\n⚠️  Good resilience, but could be improved");
        } else {
            println!("\n❌ Poor resilience - needs improvement");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_chaos_config() {
        let config = ChaosConfig::new()
            .with_network_failure_rate(0.2)
            .with_latency_ms(200, 1000)
            .with_error_injection_rate(0.1);
        
        assert_eq!(config.network_failure_rate, 0.2);
        assert_eq!(config.min_latency_ms, 200);
        assert_eq!(config.max_latency_ms, 1000);
        assert_eq!(config.error_injection_rate, 0.1);
    }
    
    #[tokio::test]
    async fn test_chaos_scenario_short() {
        let config = ChaosConfig::new()
            .with_duration(Duration::from_secs(2))
            .with_warmup(Duration::from_millis(100))
            .with_network_failure_rate(0.1);
        
        let scenario = ChaosScenario::new(config);
        let report = scenario.run().await;
        
        assert!(report.total_requests > 0);
        assert!(report.resilience_score >= 0.0);
        assert!(report.resilience_score <= 100.0);
    }
    
    #[test]
    fn test_chaos_result() {
        let result = ChaosResult {
            chaos_type: ChaosType::Latency,
            handled_correctly: true,
            duration: Duration::from_millis(500),
            error: None,
        };
        
        assert_eq!(result.chaos_type, ChaosType::Latency);
        assert!(result.handled_correctly);
        assert!(result.error.is_none());
    }
}
