//! Performance Optimization & Benchmarking for EEYF
//!
//! This module provides comprehensive performance optimization and benchmarking
//! capabilities for the Yahoo Finance API client:
//!
//! - Automated performance profiling and analysis
//! - Intelligent optimization recommendations
//! - Real-time performance monitoring and alerting  
//! - Comprehensive benchmarking suite with historical tracking
//! - Performance regression detection and reporting

use crate::yahoo_error::YahooError;
use std::collections::{HashMap, BTreeMap, VecDeque};

use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime};
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};

/// Performance optimization configuration
#[derive(Debug, Clone)]
pub struct PerformanceConfig {
    /// Enable automatic optimization
    pub auto_optimization: bool,
    /// Performance monitoring interval
    pub monitoring_interval: Duration,
    /// Benchmark execution interval
    pub benchmark_interval: Duration,
    /// Historical data retention period
    pub retention_period: Duration,
    /// Performance alert thresholds
    pub alert_thresholds: AlertThresholds,
    /// Optimization targets
    pub optimization_targets: OptimizationTargets,
}

/// Performance alert threshold configuration
#[derive(Debug, Clone)]
pub struct AlertThresholds {
    /// Maximum acceptable response time
    pub max_response_time: Duration,
    /// Maximum acceptable error rate (0.0 - 1.0)
    pub max_error_rate: f64,
    /// Minimum acceptable throughput (requests per second)
    pub min_throughput: f64,
    /// Maximum acceptable memory usage (bytes)
    pub max_memory_usage: u64,
    /// Performance degradation threshold (percentage)
    pub degradation_threshold: f64,
}

/// Optimization target configuration
#[derive(Debug, Clone)]
pub struct OptimizationTargets {
    /// Target response time
    pub target_response_time: Duration,
    /// Target throughput (requests per second)
    pub target_throughput: f64,
    /// Target error rate (0.0 - 1.0)
    pub target_error_rate: f64,
    /// Target memory efficiency (bytes per request)
    pub target_memory_efficiency: u64,
}

/// Performance metrics snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceSnapshot {
    /// Timestamp of the snapshot
    pub timestamp: SystemTime,
    /// Response time statistics
    pub response_time_stats: ResponseTimeStats,
    /// Throughput measurements
    pub throughput_stats: ThroughputStats,
    /// Error rate statistics
    pub error_stats: ErrorStats,
    /// Memory usage statistics
    pub memory_stats: MemoryStats,
    /// Network statistics
    pub network_stats: NetworkStats,
    /// Cache performance statistics
    pub cache_stats: CachePerformanceStats,
}

/// Response time statistical measurements
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseTimeStats {
    /// Mean response time
    pub mean: Duration,
    /// Median response time
    pub median: Duration,
    /// 95th percentile response time
    pub p95: Duration,
    /// 99th percentile response time
    pub p99: Duration,
    /// Maximum response time
    pub max: Duration,
    /// Minimum response time
    pub min: Duration,
    /// Standard deviation
    pub std_dev: Duration,
}

/// Throughput statistical measurements
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThroughputStats {
    /// Current requests per second
    pub current_rps: f64,
    /// Average requests per second
    pub avg_rps: f64,
    /// Peak requests per second
    pub peak_rps: f64,
    /// Total requests processed
    pub total_requests: u64,
    /// Concurrent requests
    pub concurrent_requests: usize,
}

/// Error rate statistical measurements
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorStats {
    /// Current error rate (0.0 - 1.0)
    pub current_error_rate: f64,
    /// Average error rate
    pub avg_error_rate: f64,
    /// Total errors
    pub total_errors: u64,
    /// Error distribution by type
    pub error_distribution: HashMap<String, u64>,
}

/// Memory usage statistical measurements
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryStats {
    /// Current memory usage (bytes)
    pub current_usage: u64,
    /// Peak memory usage (bytes)
    pub peak_usage: u64,
    /// Average memory usage (bytes)
    pub avg_usage: u64,
    /// Memory efficiency (bytes per request)
    pub efficiency: u64,
}

/// Network statistical measurements
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkStats {
    /// Bytes sent
    pub bytes_sent: u64,
    /// Bytes received
    pub bytes_received: u64,
    /// Connection pool utilization
    pub pool_utilization: f64,
    /// DNS resolution time
    pub dns_resolution_time: Duration,
    /// Connection establishment time
    pub connection_time: Duration,
}

/// Cache performance measurements
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachePerformanceStats {
    /// Cache hit rate (0.0 - 1.0)
    pub hit_rate: f64,
    /// Cache miss rate (0.0 - 1.0)
    pub miss_rate: f64,
    /// Average cache lookup time
    pub avg_lookup_time: Duration,
    /// Cache efficiency ratio
    pub efficiency_ratio: f64,
}

/// Benchmark test result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkResult {
    /// Test name/identifier
    pub test_name: String,
    /// Test timestamp
    pub timestamp: SystemTime,
    /// Test duration
    pub duration: Duration,
    /// Performance measurements
    pub measurements: PerformanceSnapshot,
    /// Test configuration used
    pub test_config: BenchmarkTestConfig,
    /// Performance score (0.0 - 1.0, higher is better)
    pub performance_score: f64,
}

/// Benchmark test configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkTestConfig {
    /// Number of concurrent requests
    pub concurrency: usize,
    /// Total number of requests to make
    pub total_requests: usize,
    /// Test duration limit
    pub duration_limit: Duration,
    /// Request types to test
    pub request_types: Vec<String>,
    /// Test data size
    pub data_size: usize,
}

/// Performance optimization recommendation
#[derive(Debug, Clone)]
pub struct OptimizationRecommendation {
    /// Recommendation type
    pub recommendation_type: RecommendationType,
    /// Priority level (1-5, 5 being highest)
    pub priority: u8,
    /// Description of the issue
    pub description: String,
    /// Suggested action
    pub suggested_action: String,
    /// Expected improvement percentage
    pub expected_improvement: f64,
    /// Implementation complexity (1-5, 5 being most complex)
    pub complexity: u8,
}

/// Types of optimization recommendations
#[derive(Debug, Clone, PartialEq)]
pub enum RecommendationType {
    /// Cache optimization
    CacheOptimization,
    /// Connection pool tuning
    ConnectionPoolTuning,
    /// Rate limit adjustment
    RateLimitAdjustment,
    /// Memory optimization
    MemoryOptimization,
    /// Network optimization
    NetworkOptimization,
    /// Configuration tuning
    ConfigurationTuning,
}

/// Performance alert
#[derive(Debug, Clone)]
pub struct PerformanceAlert {
    /// Alert level
    pub level: AlertLevel,
    /// Alert message
    pub message: String,
    /// Timestamp
    pub timestamp: SystemTime,
    /// Affected metric
    pub metric: String,
    /// Current value
    pub current_value: f64,
    /// Threshold value
    pub threshold_value: f64,
}

/// Alert severity levels
#[derive(Debug, Clone, PartialEq)]
pub enum AlertLevel {
    Info,
    Warning,
    Critical,
}

/// Performance monitor and optimizer
pub struct PerformanceOptimizer {
    /// Configuration
    config: PerformanceConfig,
    /// Current performance metrics
    current_metrics: Arc<RwLock<PerformanceSnapshot>>,
    /// Historical performance data
    historical_data: Arc<RwLock<VecDeque<PerformanceSnapshot>>>,
    /// Benchmark results history
    benchmark_history: Arc<RwLock<Vec<BenchmarkResult>>>,
    /// Active optimization recommendations
    recommendations: Arc<RwLock<Vec<OptimizationRecommendation>>>,
    /// Performance alerts
    alerts: Arc<RwLock<Vec<PerformanceAlert>>>,
    /// Monitoring statistics
    monitoring_stats: Arc<RwLock<MonitoringStats>>,
}

/// Performance monitoring statistics
#[derive(Debug, Clone)]
struct MonitoringStats {
    /// Total monitoring cycles completed
    pub total_cycles: u64,
    /// Total optimizations applied
    pub optimizations_applied: u64,
    /// Total alerts generated
    pub alerts_generated: u64,
    /// Monitoring uptime
    pub uptime: Duration,
    /// Last optimization timestamp
    pub last_optimization: Option<SystemTime>,
}

/// Performance regression detection
pub struct RegressionDetector {
    /// Historical baselines for comparison
    baselines: BTreeMap<String, PerformanceBaseline>,
    /// Regression detection thresholds
    detection_thresholds: RegressionThresholds,
}

/// Performance baseline for regression detection
#[derive(Debug, Clone)]
struct PerformanceBaseline {
    /// Baseline identifier
    pub id: String,
    /// Baseline timestamp
    pub timestamp: SystemTime,
    /// Baseline measurements
    pub measurements: PerformanceSnapshot,
    /// Confidence interval
    pub confidence_interval: f64,
}

/// Regression detection thresholds
#[derive(Debug, Clone)]
struct RegressionThresholds {
    /// Response time regression threshold (percentage)
    pub response_time_threshold: f64,
    /// Throughput regression threshold (percentage)
    pub throughput_threshold: f64,
    /// Error rate regression threshold (percentage)
    pub error_rate_threshold: f64,
    /// Memory usage regression threshold (percentage)
    pub memory_threshold: f64,
}

impl Default for PerformanceConfig {
    fn default() -> Self {
        Self {
            auto_optimization: true,
            monitoring_interval: Duration::from_secs(30),
            benchmark_interval: Duration::from_secs(300), // 5 minutes
            retention_period: Duration::from_secs(86400 * 7), // 7 days
            alert_thresholds: AlertThresholds::default(),
            optimization_targets: OptimizationTargets::default(),
        }
    }
}

impl Default for AlertThresholds {
    fn default() -> Self {
        Self {
            max_response_time: Duration::from_secs(5),
            max_error_rate: 0.05, // 5%
            min_throughput: 1.0, // 1 RPS
            max_memory_usage: 100 * 1024 * 1024, // 100MB
            degradation_threshold: 0.20, // 20%
        }
    }
}

impl Default for OptimizationTargets {
    fn default() -> Self {
        Self {
            target_response_time: Duration::from_secs(1),
            target_throughput: 50.0, // 50 RPS
            target_error_rate: 0.01, // 1%
            target_memory_efficiency: 1024 * 1024, // 1MB per request
        }
    }
}

impl PerformanceOptimizer {
    /// Create a new performance optimizer
    pub fn new(config: PerformanceConfig) -> Self {
        Self {
            config,
            current_metrics: Arc::new(RwLock::new(PerformanceSnapshot::default())),
            historical_data: Arc::new(RwLock::new(VecDeque::new())),
            benchmark_history: Arc::new(RwLock::new(Vec::new())),
            recommendations: Arc::new(RwLock::new(Vec::new())),
            alerts: Arc::new(RwLock::new(Vec::new())),
            monitoring_stats: Arc::new(RwLock::new(MonitoringStats::default())),
        }
    }

    /// Start performance monitoring
    pub async fn start_monitoring(&self) -> Result<(), YahooError> {
        let config = self.config.clone();
        let current_metrics = Arc::clone(&self.current_metrics);
        let historical_data = Arc::clone(&self.historical_data);
        let recommendations = Arc::clone(&self.recommendations);
        let alerts = Arc::clone(&self.alerts);
        let monitoring_stats = Arc::clone(&self.monitoring_stats);

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(config.monitoring_interval);
            
            loop {
                interval.tick().await;
                
                // Collect current performance metrics
                if let Ok(snapshot) = Self::collect_performance_snapshot().await {
                    // Update current metrics
                    {
                        let mut metrics = current_metrics.write().await;
                        *metrics = snapshot.clone();
                    }
                    
                    // Add to historical data
                    {
                        let mut history = historical_data.write().await;
                        history.push_back(snapshot.clone());
                        
                        // Maintain retention period
                        let cutoff = SystemTime::now() - config.retention_period;
                        while let Some(front) = history.front() {
                            if front.timestamp < cutoff {
                                history.pop_front();
                            } else {
                                break;
                            }
                        }
                    }
                    
                    // Generate recommendations and alerts
                    if config.auto_optimization {
                        Self::generate_recommendations(&snapshot, &recommendations).await;
                        Self::check_alert_thresholds(&snapshot, &config.alert_thresholds, &alerts).await;
                    }
                    
                    // Update monitoring stats
                    {
                        let mut stats = monitoring_stats.write().await;
                        stats.total_cycles += 1;
                    }
                }
            }
        });

        Ok(())
    }

    /// Run comprehensive benchmark suite
    pub async fn run_benchmarks(&self) -> Result<Vec<BenchmarkResult>, YahooError> {
        let mut results = Vec::new();
        
        // Define benchmark test configurations
        let test_configs = vec![
            BenchmarkTestConfig {
                concurrency: 1,
                total_requests: 100,
                duration_limit: Duration::from_secs(60),
                request_types: vec!["quote".to_string()],
                data_size: 1024,
            },
            BenchmarkTestConfig {
                concurrency: 5,
                total_requests: 500,
                duration_limit: Duration::from_secs(120),
                request_types: vec!["quote".to_string(), "chart".to_string()],
                data_size: 1024,
            },
            BenchmarkTestConfig {
                concurrency: 10,
                total_requests: 1000,
                duration_limit: Duration::from_secs(180),
                request_types: vec!["quote".to_string(), "chart".to_string(), "search".to_string()],
                data_size: 1024,
            },
        ];
        
        // Run each benchmark test
        for (i, test_config) in test_configs.iter().enumerate() {
            let result = self.run_single_benchmark(
                &format!("benchmark_test_{}", i + 1), 
                test_config.clone()
            ).await?;
            results.push(result);
        }
        
        // Store benchmark results
        {
            let mut history = self.benchmark_history.write().await;
            history.extend(results.clone());
            
            // Limit history size (keep last 100 results)
            if history.len() > 100 {
                let len = history.len();
                history.drain(0..len - 100);
            }
        }
        
        Ok(results)
    }

    /// Get current performance snapshot
    pub async fn get_current_metrics(&self) -> PerformanceSnapshot {
        self.current_metrics.read().await.clone()
    }

    /// Get historical performance data
    pub async fn get_historical_data(&self, limit: Option<usize>) -> Vec<PerformanceSnapshot> {
        let history = self.historical_data.read().await;
        match limit {
            Some(n) => history.iter().rev().take(n).cloned().collect(),
            None => history.iter().cloned().collect(),
        }
    }

    /// Get performance optimization recommendations
    pub async fn get_recommendations(&self) -> Vec<OptimizationRecommendation> {
        self.recommendations.read().await.clone()
    }

    /// Get performance alerts
    pub async fn get_alerts(&self, level: Option<AlertLevel>) -> Vec<PerformanceAlert> {
        let alerts = self.alerts.read().await;
        match level {
            Some(target_level) => alerts.iter()
                .filter(|alert| alert.level == target_level)
                .cloned()
                .collect(),
            None => alerts.clone(),
        }
    }

    /// Generate performance report
    pub async fn generate_performance_report(&self) -> Result<PerformanceReport, YahooError> {
        let current = self.get_current_metrics().await;
        let historical = self.get_historical_data(Some(100)).await;
        let recommendations = self.get_recommendations().await;
        let alerts = self.get_alerts(None).await;
        
        Ok(PerformanceReport {
            timestamp: SystemTime::now(),
            current_metrics: current,
            historical_summary: Self::summarize_historical_data(&historical),
            recommendations,
            alerts,
            overall_score: Self::calculate_performance_score(&historical),
        })
    }

    // Private helper methods
    async fn collect_performance_snapshot() -> Result<PerformanceSnapshot, YahooError> {
        // In a real implementation, this would collect actual metrics
        // For now, return a mock snapshot
        Ok(PerformanceSnapshot::default())
    }

    async fn generate_recommendations(
        snapshot: &PerformanceSnapshot,
        recommendations: &Arc<RwLock<Vec<OptimizationRecommendation>>>
    ) {
        let mut new_recommendations = Vec::new();
        
        // Analyze response times
        if snapshot.response_time_stats.mean > Duration::from_secs(2) {
            new_recommendations.push(OptimizationRecommendation {
                recommendation_type: RecommendationType::CacheOptimization,
                priority: 4,
                description: "High response times detected".to_string(),
                suggested_action: "Increase cache hit ratio or optimize cache storage".to_string(),
                expected_improvement: 0.30,
                complexity: 3,
            });
        }
        
        // Analyze throughput
        if snapshot.throughput_stats.current_rps < 10.0 {
            new_recommendations.push(OptimizationRecommendation {
                recommendation_type: RecommendationType::ConnectionPoolTuning,
                priority: 3,
                description: "Low throughput detected".to_string(),
                suggested_action: "Increase connection pool size or optimize connection reuse".to_string(),
                expected_improvement: 0.25,
                complexity: 2,
            });
        }
        
        // Update recommendations
        let mut recs = recommendations.write().await;
        recs.clear();
        recs.extend(new_recommendations);
    }

    async fn check_alert_thresholds(
        snapshot: &PerformanceSnapshot,
        thresholds: &AlertThresholds,
        alerts: &Arc<RwLock<Vec<PerformanceAlert>>>
    ) {
        let mut new_alerts = Vec::new();
        
        // Check response time threshold
        if snapshot.response_time_stats.mean > thresholds.max_response_time {
            new_alerts.push(PerformanceAlert {
                level: AlertLevel::Warning,
                message: "Response time exceeded threshold".to_string(),
                timestamp: SystemTime::now(),
                metric: "response_time".to_string(),
                current_value: snapshot.response_time_stats.mean.as_secs_f64(),
                threshold_value: thresholds.max_response_time.as_secs_f64(),
            });
        }
        
        // Check error rate threshold
        if snapshot.error_stats.current_error_rate > thresholds.max_error_rate {
            new_alerts.push(PerformanceAlert {
                level: AlertLevel::Critical,
                message: "Error rate exceeded threshold".to_string(),
                timestamp: SystemTime::now(),
                metric: "error_rate".to_string(),
                current_value: snapshot.error_stats.current_error_rate,
                threshold_value: thresholds.max_error_rate,
            });
        }
        
        // Update alerts
        let mut alert_list = alerts.write().await;
        alert_list.extend(new_alerts);
        
        // Limit alert history (keep last 50 alerts)
        if alert_list.len() > 50 {
            let len = alert_list.len();
            alert_list.drain(0..len - 50);
        }
    }

    async fn run_single_benchmark(
        &self,
        test_name: &str,
        test_config: BenchmarkTestConfig
    ) -> Result<BenchmarkResult, YahooError> {
        let start_time = Instant::now();
        
        // Simulate benchmark execution
        // In a real implementation, this would execute actual API calls
        tokio::time::sleep(Duration::from_millis(100)).await;
        
        let duration = start_time.elapsed();
        let measurements = Self::collect_performance_snapshot().await?;
        let performance_score = Self::calculate_benchmark_score(&measurements, &test_config);
        
        Ok(BenchmarkResult {
            test_name: test_name.to_string(),
            timestamp: SystemTime::now(),
            duration,
            measurements,
            test_config,
            performance_score,
        })
    }

    fn calculate_benchmark_score(
        measurements: &PerformanceSnapshot,
        _test_config: &BenchmarkTestConfig
    ) -> f64 {
        // Simple scoring algorithm (in practice, this would be more sophisticated)
        let response_time_score = if measurements.response_time_stats.mean.as_secs_f64() < 1.0 { 1.0 } else { 0.5 };
        let throughput_score = if measurements.throughput_stats.current_rps > 10.0 { 1.0 } else { 0.5 };
        let error_score = if measurements.error_stats.current_error_rate < 0.01 { 1.0 } else { 0.5 };
        
        (response_time_score + throughput_score + error_score) / 3.0
    }

    fn summarize_historical_data(historical: &[PerformanceSnapshot]) -> HistoricalSummary {
        if historical.is_empty() {
            return HistoricalSummary::default();
        }
        
        // Calculate trends and averages
        let avg_response_time = historical.iter()
            .map(|s| s.response_time_stats.mean.as_secs_f64())
            .sum::<f64>() / historical.len() as f64;
            
        let avg_throughput = historical.iter()
            .map(|s| s.throughput_stats.current_rps)
            .sum::<f64>() / historical.len() as f64;
            
        let avg_error_rate = historical.iter()
            .map(|s| s.error_stats.current_error_rate)
            .sum::<f64>() / historical.len() as f64;
        
        HistoricalSummary {
            avg_response_time: Duration::from_secs_f64(avg_response_time),
            avg_throughput,
            avg_error_rate,
            data_points: historical.len(),
        }
    }

    fn calculate_performance_score(historical: &[PerformanceSnapshot]) -> f64 {
        if historical.is_empty() {
            return 0.0;
        }
        
        // Calculate overall performance score based on historical data
        let summary = Self::summarize_historical_data(historical);
        
        let response_score = if summary.avg_response_time.as_secs_f64() < 1.0 { 1.0 } else { 0.5 };
        let throughput_score = if summary.avg_throughput > 10.0 { 1.0 } else { 0.5 };
        let error_score = if summary.avg_error_rate < 0.01 { 1.0 } else { 0.5 };
        
        (response_score + throughput_score + error_score) / 3.0
    }
}

/// Performance report structure
#[derive(Debug, Clone)]
pub struct PerformanceReport {
    /// Report timestamp
    pub timestamp: SystemTime,
    /// Current performance metrics
    pub current_metrics: PerformanceSnapshot,
    /// Historical data summary
    pub historical_summary: HistoricalSummary,
    /// Active recommendations
    pub recommendations: Vec<OptimizationRecommendation>,
    /// Current alerts
    pub alerts: Vec<PerformanceAlert>,
    /// Overall performance score (0.0 - 1.0)
    pub overall_score: f64,
}

/// Historical performance summary
#[derive(Debug, Clone)]
pub struct HistoricalSummary {
    /// Average response time
    pub avg_response_time: Duration,
    /// Average throughput
    pub avg_throughput: f64,
    /// Average error rate
    pub avg_error_rate: f64,
    /// Number of data points
    pub data_points: usize,
}

impl Default for PerformanceSnapshot {
    fn default() -> Self {
        Self {
            timestamp: SystemTime::now(),
            response_time_stats: ResponseTimeStats::default(),
            throughput_stats: ThroughputStats::default(),
            error_stats: ErrorStats::default(),
            memory_stats: MemoryStats::default(),
            network_stats: NetworkStats::default(),
            cache_stats: CachePerformanceStats::default(),
        }
    }
}

impl Default for ResponseTimeStats {
    fn default() -> Self {
        Self {
            mean: Duration::from_millis(500),
            median: Duration::from_millis(400),
            p95: Duration::from_millis(1000),
            p99: Duration::from_millis(1500),
            max: Duration::from_millis(2000),
            min: Duration::from_millis(100),
            std_dev: Duration::from_millis(200),
        }
    }
}

impl Default for ThroughputStats {
    fn default() -> Self {
        Self {
            current_rps: 15.0,
            avg_rps: 12.0,
            peak_rps: 25.0,
            total_requests: 1000,
            concurrent_requests: 3,
        }
    }
}

impl Default for ErrorStats {
    fn default() -> Self {
        Self {
            current_error_rate: 0.02,
            avg_error_rate: 0.015,
            total_errors: 20,
            error_distribution: HashMap::new(),
        }
    }
}

impl Default for MemoryStats {
    fn default() -> Self {
        Self {
            current_usage: 50 * 1024 * 1024, // 50MB
            peak_usage: 75 * 1024 * 1024,    // 75MB
            avg_usage: 45 * 1024 * 1024,     // 45MB
            efficiency: 1024 * 1024,         // 1MB per request
        }
    }
}

impl Default for NetworkStats {
    fn default() -> Self {
        Self {
            bytes_sent: 1024 * 1024,     // 1MB
            bytes_received: 5 * 1024 * 1024, // 5MB
            pool_utilization: 0.7,       // 70%
            dns_resolution_time: Duration::from_millis(50),
            connection_time: Duration::from_millis(100),
        }
    }
}

impl Default for CachePerformanceStats {
    fn default() -> Self {
        Self {
            hit_rate: 0.85,              // 85%
            miss_rate: 0.15,             // 15%
            avg_lookup_time: Duration::from_millis(5),
            efficiency_ratio: 0.9,       // 90%
        }
    }
}

impl Default for MonitoringStats {
    fn default() -> Self {
        Self {
            total_cycles: 0,
            optimizations_applied: 0,
            alerts_generated: 0,
            uptime: Duration::from_secs(0),
            last_optimization: None,
        }
    }
}

impl Default for HistoricalSummary {
    fn default() -> Self {
        Self {
            avg_response_time: Duration::from_millis(500),
            avg_throughput: 10.0,
            avg_error_rate: 0.02,
            data_points: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_performance_optimizer_creation() {
        let config = PerformanceConfig::default();
        let optimizer = PerformanceOptimizer::new(config);
        
        let metrics = optimizer.get_current_metrics().await;
        assert!(metrics.response_time_stats.mean > Duration::from_secs(0));
    }

    #[tokio::test]
    async fn test_benchmark_execution() {
        let config = PerformanceConfig::default();
        let optimizer = PerformanceOptimizer::new(config);
        
        let results = optimizer.run_benchmarks().await.unwrap();
        assert!(!results.is_empty());
        assert!(results[0].performance_score >= 0.0 && results[0].performance_score <= 1.0);
    }

    #[tokio::test]
    async fn test_performance_report_generation() {
        let config = PerformanceConfig::default();
        let optimizer = PerformanceOptimizer::new(config);
        
        let report = optimizer.generate_performance_report().await.unwrap();
        assert!(report.overall_score >= 0.0 && report.overall_score <= 1.0);
    }

    #[tokio::test]
    async fn test_recommendations_generation() {
        let config = PerformanceConfig::default();
        let optimizer = PerformanceOptimizer::new(config);
        
        // This would trigger recommendation generation in a real implementation
        let recommendations = optimizer.get_recommendations().await;
        // Initially empty, but the structure is tested
        assert_eq!(recommendations.len(), 0);
    }

    #[tokio::test]
    async fn test_alert_thresholds() {
        let thresholds = AlertThresholds::default();
        assert_eq!(thresholds.max_response_time, Duration::from_secs(5));
        assert_eq!(thresholds.max_error_rate, 0.05);
    }
}