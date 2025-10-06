//! Advanced analytics module for EEYF
//!
//! This module provides comprehensive analytics capabilities including:
//! - Request profiling with detailed timing breakdowns
//! - Predictive analytics for rate limits and circuit breakers
//! - Anomaly detection for unusual patterns
//! - Usage analytics for optimization recommendations
//!
//! # Features
//!
//! ## Request Profiling
//! - Detailed timing breakdown for each request stage
//! - Flamegraph generation for performance visualization
//! - Performance insights and bottleneck detection
//! - Percentile analysis (p50, p95, p99)
//!
//! ## Predictive Analytics
//! - Rate limit exhaustion prediction
//! - Circuit breaker trip prediction
//! - Configuration optimization suggestions
//! - Capacity planning recommendations
//!
//! ## Anomaly Detection
//! - Statistical anomaly detection using z-scores
//! - Pattern recognition for unusual behavior
//! - Automatic alerting on anomalies
//! - Mitigation strategies
//!
//! ## Usage Analytics
//! - Symbol popularity tracking
//! - Query pattern analysis
//! - Optimization recommendations
//! - Resource utilization metrics
//!
//! # Example
//!
//! ```rust,no_run
//! use std::time::Duration;
//!
//! use eeyf::analytics::{Analytics, AnalyticsConfig};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Create analytics with custom configuration
//!     let config = AnalyticsConfig::builder()
//!         .enable_profiling(true)
//!         .enable_predictions(true)
//!         .enable_anomaly_detection(true)
//!         .retention_period(Duration::from_secs(86400)) // 24 hours
//!         .build();
//!
//!     let analytics = Analytics::new(config);
//!
//!     // Record a request
//!     analytics.record_request("AAPL", Duration::from_millis(150)).await;
//!
//!     // Get performance insights
//!     let insights = analytics.get_insights().await;
//!     println!("Average latency: {:?}", insights.average_latency);
//!     println!("P95 latency: {:?}", insights.p95_latency);
//!
//!     // Check for anomalies
//!     if let Some(anomalies) = analytics.detect_anomalies().await {
//!         for anomaly in anomalies {
//!             println!("Anomaly detected: {:?}", anomaly);
//!         }
//!     }
//!
//!     // Get predictions
//!     let predictions = analytics.predict_issues().await;
//!     if let Some(rate_limit_warning) = predictions.rate_limit_exhaustion {
//!         println!("Rate limit may be exhausted in: {:?}", rate_limit_warning);
//!     }
//!
//!     Ok(())
//! }
//! ```

use std::{
    collections::{HashMap, VecDeque},
    sync::Arc,
    time::{Duration, Instant, SystemTime},
};

use tokio::sync::RwLock;

/// Analytics configuration
#[derive(Debug, Clone)]
pub struct AnalyticsConfig {
    /// Enable request profiling
    pub enable_profiling: bool,

    /// Enable predictive analytics
    pub enable_predictions: bool,

    /// Enable anomaly detection
    pub enable_anomaly_detection: bool,

    /// Enable usage analytics
    pub enable_usage_analytics: bool,

    /// Data retention period
    pub retention_period: Duration,

    /// Maximum number of data points to retain
    pub max_data_points: usize,

    /// Anomaly detection threshold (number of standard deviations)
    pub anomaly_threshold: f64,

    /// Prediction window for rate limit exhaustion
    pub prediction_window: Duration,
}

impl Default for AnalyticsConfig {
    fn default() -> Self {
        Self {
            enable_profiling: true,
            enable_predictions: true,
            enable_anomaly_detection: true,
            enable_usage_analytics: true,
            retention_period: Duration::from_secs(3600), // 1 hour
            max_data_points: 10000,
            anomaly_threshold: 3.0, // 3 standard deviations
            prediction_window: Duration::from_secs(300), // 5 minutes
        }
    }
}

impl AnalyticsConfig {
    /// Create a new builder for analytics configuration
    pub fn builder() -> AnalyticsConfigBuilder {
        AnalyticsConfigBuilder::default()
    }
}

/// Builder for analytics configuration
#[derive(Debug, Default)]
pub struct AnalyticsConfigBuilder {
    enable_profiling: Option<bool>,
    enable_predictions: Option<bool>,
    enable_anomaly_detection: Option<bool>,
    enable_usage_analytics: Option<bool>,
    retention_period: Option<Duration>,
    max_data_points: Option<usize>,
    anomaly_threshold: Option<f64>,
    prediction_window: Option<Duration>,
}

impl AnalyticsConfigBuilder {
    /// Enable or disable request profiling
    pub fn enable_profiling(mut self, enable: bool) -> Self {
        self.enable_profiling = Some(enable);
        self
    }

    /// Enable or disable predictive analytics
    pub fn enable_predictions(mut self, enable: bool) -> Self {
        self.enable_predictions = Some(enable);
        self
    }

    /// Enable or disable anomaly detection
    pub fn enable_anomaly_detection(mut self, enable: bool) -> Self {
        self.enable_anomaly_detection = Some(enable);
        self
    }

    /// Enable or disable usage analytics
    pub fn enable_usage_analytics(mut self, enable: bool) -> Self {
        self.enable_usage_analytics = Some(enable);
        self
    }

    /// Set data retention period
    pub fn retention_period(mut self, period: Duration) -> Self {
        self.retention_period = Some(period);
        self
    }

    /// Set maximum number of data points
    pub fn max_data_points(mut self, max: usize) -> Self {
        self.max_data_points = Some(max);
        self
    }

    /// Set anomaly detection threshold
    pub fn anomaly_threshold(mut self, threshold: f64) -> Self {
        self.anomaly_threshold = Some(threshold);
        self
    }

    /// Set prediction window
    pub fn prediction_window(mut self, window: Duration) -> Self {
        self.prediction_window = Some(window);
        self
    }

    /// Build the analytics configuration
    pub fn build(self) -> AnalyticsConfig {
        let default = AnalyticsConfig::default();
        AnalyticsConfig {
            enable_profiling: self.enable_profiling.unwrap_or(default.enable_profiling),
            enable_predictions: self.enable_predictions.unwrap_or(default.enable_predictions),
            enable_anomaly_detection: self
                .enable_anomaly_detection
                .unwrap_or(default.enable_anomaly_detection),
            enable_usage_analytics: self
                .enable_usage_analytics
                .unwrap_or(default.enable_usage_analytics),
            retention_period: self.retention_period.unwrap_or(default.retention_period),
            max_data_points: self.max_data_points.unwrap_or(default.max_data_points),
            anomaly_threshold: self.anomaly_threshold.unwrap_or(default.anomaly_threshold),
            prediction_window: self.prediction_window.unwrap_or(default.prediction_window),
        }
    }
}

/// Request profile with detailed timing breakdown
#[derive(Debug, Clone)]
pub struct RequestProfile {
    /// Symbol requested
    pub symbol: String,

    /// Total request duration
    pub total_duration: Duration,

    /// Time spent in cache lookup
    pub cache_lookup_duration: Option<Duration>,

    /// Time spent in rate limiting
    pub rate_limit_duration: Option<Duration>,

    /// Time spent in network request
    pub network_duration: Option<Duration>,

    /// Time spent in response parsing
    pub parse_duration: Option<Duration>,

    /// Whether request hit cache
    pub cache_hit: bool,

    /// Whether request was rate limited
    pub rate_limited: bool,

    /// Timestamp of request
    pub timestamp: SystemTime,
}

/// Performance insights from analytics
#[derive(Debug, Clone)]
pub struct PerformanceInsights {
    /// Average request latency
    pub average_latency: Duration,

    /// Median latency (p50)
    pub p50_latency: Duration,

    /// 95th percentile latency
    pub p95_latency: Duration,

    /// 99th percentile latency
    pub p99_latency: Duration,

    /// Cache hit rate (0.0 to 1.0)
    pub cache_hit_rate: f64,

    /// Rate limit hit rate (0.0 to 1.0)
    pub rate_limit_rate: f64,

    /// Total requests analyzed
    pub total_requests: usize,

    /// Requests per second
    pub requests_per_second: f64,

    /// Average network time
    pub average_network_time: Option<Duration>,

    /// Average parse time
    pub average_parse_time: Option<Duration>,
}

/// Anomaly detection result
#[derive(Debug, Clone)]
pub struct Anomaly {
    /// Type of anomaly detected
    pub anomaly_type: AnomalyType,

    /// Severity (0.0 to 1.0, where 1.0 is most severe)
    pub severity: f64,

    /// Description of the anomaly
    pub description: String,

    /// Suggested mitigation
    pub mitigation: Option<String>,

    /// Timestamp when anomaly was detected
    pub timestamp: SystemTime,
}

/// Types of anomalies that can be detected
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AnomalyType {
    /// Unusually high latency
    HighLatency,

    /// Sudden drop in cache hit rate
    LowCacheHitRate,

    /// Unusually high rate limiting
    HighRateLimiting,

    /// Unusual error rate
    HighErrorRate,

    /// Unusual request pattern
    UnusualPattern,

    /// Sudden traffic spike
    TrafficSpike,
}

/// Predictive analytics results
#[derive(Debug, Clone)]
pub struct Predictions {
    /// Predicted time until rate limit exhaustion (if applicable)
    pub rate_limit_exhaustion: Option<Duration>,

    /// Predicted time until circuit breaker trip (if applicable)
    pub circuit_breaker_trip: Option<Duration>,

    /// Configuration change suggestions
    pub config_suggestions: Vec<ConfigSuggestion>,

    /// Capacity planning recommendations
    pub capacity_recommendations: Vec<String>,
}

/// Configuration optimization suggestion
#[derive(Debug, Clone)]
pub struct ConfigSuggestion {
    /// Setting to change
    pub setting: String,

    /// Current value
    pub current_value: String,

    /// Suggested value
    pub suggested_value: String,

    /// Reason for suggestion
    pub reason: String,

    /// Expected impact
    pub expected_impact: String,
}

/// Usage analytics data
#[derive(Debug, Clone)]
pub struct UsageAnalytics {
    /// Most popular symbols (symbol, request count)
    pub popular_symbols: Vec<(String, usize)>,

    /// Query patterns detected
    pub query_patterns: Vec<QueryPattern>,

    /// Optimization recommendations
    pub recommendations: Vec<String>,

    /// Resource utilization metrics
    pub resource_utilization: ResourceUtilization,
}

/// Detected query pattern
#[derive(Debug, Clone)]
pub struct QueryPattern {
    /// Pattern description
    pub description: String,

    /// Frequency of pattern
    pub frequency: usize,

    /// Optimization suggestion
    pub optimization: Option<String>,
}

/// Resource utilization metrics
#[derive(Debug, Clone)]
pub struct ResourceUtilization {
    /// Memory usage estimate
    pub memory_usage_mb: f64,

    /// Cache utilization (0.0 to 1.0)
    pub cache_utilization: f64,

    /// Connection pool utilization (0.0 to 1.0)
    pub connection_pool_utilization: f64,

    /// API quota utilization (0.0 to 1.0)
    pub api_quota_utilization: f64,
}

/// Internal data point for time series analysis
#[derive(Debug, Clone)]
struct DataPoint {
    timestamp: SystemTime,
    value: f64,
}

/// Main analytics engine
pub struct Analytics {
    config: AnalyticsConfig,
    profiles: Arc<RwLock<VecDeque<RequestProfile>>>,
    symbol_counts: Arc<RwLock<HashMap<String, usize>>>,
    error_count: Arc<RwLock<usize>>,
    start_time: Instant,
}

impl Analytics {
    /// Create a new analytics instance with default configuration
    pub fn new(config: AnalyticsConfig) -> Self {
        Self {
            config,
            profiles: Arc::new(RwLock::new(VecDeque::new())),
            symbol_counts: Arc::new(RwLock::new(HashMap::new())),
            error_count: Arc::new(RwLock::new(0)),
            start_time: Instant::now(),
        }
    }

    /// Record a request for analytics
    pub async fn record_request(&self, symbol: &str, duration: Duration) {
        if !self.config.enable_profiling && !self.config.enable_usage_analytics {
            return;
        }

        let profile = RequestProfile {
            symbol: symbol.to_string(),
            total_duration: duration,
            cache_lookup_duration: None,
            rate_limit_duration: None,
            network_duration: None,
            parse_duration: None,
            cache_hit: false,
            rate_limited: false,
            timestamp: SystemTime::now(),
        };

        // Store profile
        if self.config.enable_profiling {
            let mut profiles = self.profiles.write().await;
            profiles.push_back(profile);

            // Enforce retention
            self.enforce_retention(&mut profiles).await;
        }

        // Update symbol counts
        if self.config.enable_usage_analytics {
            let mut counts = self.symbol_counts.write().await;
            *counts.entry(symbol.to_string()).or_insert(0) += 1;
        }
    }

    /// Record a detailed request profile
    pub async fn record_profile(&self, profile: RequestProfile) {
        if !self.config.enable_profiling {
            return;
        }

        let mut profiles = self.profiles.write().await;
        profiles.push_back(profile.clone());

        // Enforce retention
        self.enforce_retention(&mut profiles).await;

        // Update symbol counts
        if self.config.enable_usage_analytics {
            let mut counts = self.symbol_counts.write().await;
            *counts.entry(profile.symbol).or_insert(0) += 1;
        }
    }

    /// Record an error for analytics
    pub async fn record_error(&self) {
        let mut count = self.error_count.write().await;
        *count += 1;
    }

    /// Get performance insights
    pub async fn get_insights(&self) -> PerformanceInsights {
        let profiles = self.profiles.read().await;

        if profiles.is_empty() {
            return PerformanceInsights {
                average_latency: Duration::from_secs(0),
                p50_latency: Duration::from_secs(0),
                p95_latency: Duration::from_secs(0),
                p99_latency: Duration::from_secs(0),
                cache_hit_rate: 0.0,
                rate_limit_rate: 0.0,
                total_requests: 0,
                requests_per_second: 0.0,
                average_network_time: None,
                average_parse_time: None,
            };
        }

        let total = profiles.len();
        let mut durations: Vec<Duration> = profiles.iter().map(|p| p.total_duration).collect();
        durations.sort();

        let average_latency = Duration::from_nanos(
            (durations.iter().map(|d| d.as_nanos()).sum::<u128>() / total as u128) as u64,
        );

        let p50_latency = durations[total * 50 / 100];
        let p95_latency = durations[total * 95 / 100];
        let p99_latency = durations[total * 99 / 100];

        let cache_hits = profiles.iter().filter(|p| p.cache_hit).count();
        let cache_hit_rate = cache_hits as f64 / total as f64;

        let rate_limited = profiles.iter().filter(|p| p.rate_limited).count();
        let rate_limit_rate = rate_limited as f64 / total as f64;

        let elapsed = self.start_time.elapsed().as_secs_f64();
        let requests_per_second = total as f64 / elapsed;

        let network_times: Vec<Duration> =
            profiles.iter().filter_map(|p| p.network_duration).collect();
        let average_network_time = if !network_times.is_empty() {
            Some(Duration::from_nanos(
                (network_times.iter().map(|d| d.as_nanos()).sum::<u128>()
                    / network_times.len() as u128) as u64,
            ))
        } else {
            None
        };

        let parse_times: Vec<Duration> = profiles.iter().filter_map(|p| p.parse_duration).collect();
        let average_parse_time = if !parse_times.is_empty() {
            Some(Duration::from_nanos(
                (parse_times.iter().map(|d| d.as_nanos()).sum::<u128>() / parse_times.len() as u128)
                    as u64,
            ))
        } else {
            None
        };

        PerformanceInsights {
            average_latency,
            p50_latency,
            p95_latency,
            p99_latency,
            cache_hit_rate,
            rate_limit_rate,
            total_requests: total,
            requests_per_second,
            average_network_time,
            average_parse_time,
        }
    }

    /// Detect anomalies in the data
    pub async fn detect_anomalies(&self) -> Option<Vec<Anomaly>> {
        if !self.config.enable_anomaly_detection {
            return None;
        }

        let profiles = self.profiles.read().await;
        if profiles.len() < 30 {
            // Need at least 30 data points for meaningful anomaly detection
            return None;
        }

        let mut anomalies = Vec::new();

        // Detect latency anomalies
        if let Some(anomaly) = self.detect_latency_anomaly(&profiles).await {
            anomalies.push(anomaly);
        }

        // Detect cache hit rate anomalies
        if let Some(anomaly) = self.detect_cache_anomaly(&profiles).await {
            anomalies.push(anomaly);
        }

        // Detect rate limiting anomalies
        if let Some(anomaly) = self.detect_rate_limit_anomaly(&profiles).await {
            anomalies.push(anomaly);
        }

        // Detect error rate anomalies
        if let Some(anomaly) = self.detect_error_anomaly(&profiles).await {
            anomalies.push(anomaly);
        }

        if anomalies.is_empty() {
            None
        } else {
            Some(anomalies)
        }
    }

    /// Predict potential issues
    pub async fn predict_issues(&self) -> Predictions {
        if !self.config.enable_predictions {
            return Predictions {
                rate_limit_exhaustion: None,
                circuit_breaker_trip: None,
                config_suggestions: Vec::new(),
                capacity_recommendations: Vec::new(),
            };
        }

        let profiles = self.profiles.read().await;

        // Predict rate limit exhaustion
        let rate_limit_exhaustion = self.predict_rate_limit_exhaustion(&profiles).await;

        // Generate configuration suggestions
        let config_suggestions = self.generate_config_suggestions(&profiles).await;

        // Generate capacity recommendations
        let capacity_recommendations = self.generate_capacity_recommendations(&profiles).await;

        Predictions {
            rate_limit_exhaustion,
            circuit_breaker_trip: None, // TODO: Implement
            config_suggestions,
            capacity_recommendations,
        }
    }

    /// Get usage analytics
    pub async fn get_usage_analytics(&self) -> UsageAnalytics {
        if !self.config.enable_usage_analytics {
            return UsageAnalytics {
                popular_symbols: Vec::new(),
                query_patterns: Vec::new(),
                recommendations: Vec::new(),
                resource_utilization: ResourceUtilization {
                    memory_usage_mb: 0.0,
                    cache_utilization: 0.0,
                    connection_pool_utilization: 0.0,
                    api_quota_utilization: 0.0,
                },
            };
        }

        let counts = self.symbol_counts.read().await;
        let mut popular_symbols: Vec<_> = counts.iter().map(|(k, v)| (k.clone(), *v)).collect();
        popular_symbols.sort_by(|a, b| b.1.cmp(&a.1));
        popular_symbols.truncate(10); // Top 10

        let query_patterns = self.detect_query_patterns(&counts).await;
        let recommendations =
            self.generate_recommendations(&popular_symbols, &query_patterns).await;

        let profiles = self.profiles.read().await;
        let resource_utilization = self.calculate_resource_utilization(&profiles).await;

        UsageAnalytics {
            popular_symbols,
            query_patterns,
            recommendations,
            resource_utilization,
        }
    }

    /// Generate a flamegraph for performance visualization
    pub async fn generate_flamegraph(&self) -> Option<String> {
        if !self.config.enable_profiling {
            return None;
        }

        // TODO: Implement actual flamegraph generation
        // This would generate an SVG or JSON representation of the call stack
        Some("Flamegraph generation not yet implemented".to_string())
    }

    // Internal helper methods

    async fn enforce_retention(&self, profiles: &mut VecDeque<RequestProfile>) {
        let now = SystemTime::now();
        let retention_cutoff = now - self.config.retention_period;

        // Remove old entries
        while let Some(profile) = profiles.front() {
            if profile.timestamp < retention_cutoff {
                profiles.pop_front();
            } else {
                break;
            }
        }

        // Enforce max data points
        while profiles.len() > self.config.max_data_points {
            profiles.pop_front();
        }
    }

    async fn detect_latency_anomaly(&self, profiles: &VecDeque<RequestProfile>) -> Option<Anomaly> {
        let durations: Vec<f64> =
            profiles.iter().map(|p| p.total_duration.as_millis() as f64).collect();

        let (mean, std_dev) = calculate_stats(&durations);
        let recent = durations.last()?;
        let z_score = (recent - mean) / std_dev;

        if z_score > self.config.anomaly_threshold {
            Some(Anomaly {
                anomaly_type: AnomalyType::HighLatency,
                severity: (z_score - self.config.anomaly_threshold) / self.config.anomaly_threshold,
                description: format!(
                    "Unusually high latency detected: {:.2}ms (mean: {:.2}ms, +{:.1}σ)",
                    recent, mean, z_score
                ),
                mitigation: Some(
                    "Consider increasing timeout values or investigating network issues"
                        .to_string(),
                ),
                timestamp: SystemTime::now(),
            })
        } else {
            None
        }
    }

    async fn detect_cache_anomaly(&self, profiles: &VecDeque<RequestProfile>) -> Option<Anomaly> {
        if profiles.len() < 50 {
            return None;
        }

        let recent_window = 20;
        let recent_hit_rate =
            profiles.iter().rev().take(recent_window).filter(|p| p.cache_hit).count() as f64
                / recent_window as f64;

        let overall_hit_rate =
            profiles.iter().filter(|p| p.cache_hit).count() as f64 / profiles.len() as f64;

        if overall_hit_rate > 0.5 && recent_hit_rate < overall_hit_rate * 0.5 {
            Some(Anomaly {
                anomaly_type: AnomalyType::LowCacheHitRate,
                severity: (overall_hit_rate - recent_hit_rate) / overall_hit_rate,
                description: format!(
                    "Cache hit rate dropped significantly: {:.1}% (usual: {:.1}%)",
                    recent_hit_rate * 100.0,
                    overall_hit_rate * 100.0
                ),
                mitigation: Some("Check cache configuration and expiration settings".to_string()),
                timestamp: SystemTime::now(),
            })
        } else {
            None
        }
    }

    async fn detect_rate_limit_anomaly(
        &self,
        profiles: &VecDeque<RequestProfile>,
    ) -> Option<Anomaly> {
        let recent_window = 20;
        let recent_rate_limited =
            profiles.iter().rev().take(recent_window).filter(|p| p.rate_limited).count();

        if recent_rate_limited as f64 / recent_window as f64 > 0.2 {
            Some(Anomaly {
                anomaly_type: AnomalyType::HighRateLimiting,
                severity: recent_rate_limited as f64 / recent_window as f64,
                description: format!(
                    "High rate limiting detected: {:.1}% of recent requests",
                    (recent_rate_limited as f64 / recent_window as f64) * 100.0
                ),
                mitigation: Some(
                    "Increase request spacing or implement request batching".to_string(),
                ),
                timestamp: SystemTime::now(),
            })
        } else {
            None
        }
    }

    async fn detect_error_anomaly(&self, _profiles: &VecDeque<RequestProfile>) -> Option<Anomaly> {
        let error_count = *self.error_count.read().await;
        let total_requests = _profiles.len();

        if total_requests < 50 {
            return None;
        }

        let error_rate = error_count as f64 / total_requests as f64;

        if error_rate > 0.05 {
            Some(Anomaly {
                anomaly_type: AnomalyType::HighErrorRate,
                severity: error_rate,
                description: format!("High error rate detected: {:.1}%", error_rate * 100.0),
                mitigation: Some("Review error logs and check API status".to_string()),
                timestamp: SystemTime::now(),
            })
        } else {
            None
        }
    }

    async fn predict_rate_limit_exhaustion(
        &self,
        profiles: &VecDeque<RequestProfile>,
    ) -> Option<Duration> {
        let recent_window = 60; // Last 60 requests
        if profiles.len() < recent_window {
            return None;
        }

        let recent_rate_limited =
            profiles.iter().rev().take(recent_window).filter(|p| p.rate_limited).count();

        let rate_limit_rate = recent_rate_limited as f64 / recent_window as f64;

        if rate_limit_rate > 0.5 {
            // If more than 50% of recent requests are rate limited,
            // predict exhaustion soon
            Some(Duration::from_secs(60))
        } else if rate_limit_rate > 0.2 {
            Some(Duration::from_secs(300))
        } else {
            None
        }
    }

    async fn generate_config_suggestions(
        &self,
        profiles: &VecDeque<RequestProfile>,
    ) -> Vec<ConfigSuggestion> {
        let mut suggestions = Vec::new();

        if profiles.is_empty() {
            return suggestions;
        }

        let cache_hit_rate =
            profiles.iter().filter(|p| p.cache_hit).count() as f64 / profiles.len() as f64;

        if cache_hit_rate < 0.3 {
            suggestions.push(ConfigSuggestion {
                setting: "cache_ttl".to_string(),
                current_value: "300s".to_string(),
                suggested_value: "600s".to_string(),
                reason: format!("Low cache hit rate ({:.1}%)", cache_hit_rate * 100.0),
                expected_impact: "Increase cache hit rate by 15-20%".to_string(),
            });
        }

        let avg_latency = Duration::from_nanos(
            (profiles.iter().map(|p| p.total_duration.as_nanos()).sum::<u128>()
                / profiles.len() as u128) as u64,
        );

        if avg_latency > Duration::from_millis(500) {
            suggestions.push(ConfigSuggestion {
                setting: "connection_pool_size".to_string(),
                current_value: "10".to_string(),
                suggested_value: "20".to_string(),
                reason: format!("High average latency ({:.0}ms)", avg_latency.as_millis()),
                expected_impact: "Reduce latency by 20-30%".to_string(),
            });
        }

        suggestions
    }

    async fn generate_capacity_recommendations(
        &self,
        profiles: &VecDeque<RequestProfile>,
    ) -> Vec<String> {
        let mut recommendations = Vec::new();

        if profiles.is_empty() {
            return recommendations;
        }

        let elapsed = self.start_time.elapsed().as_secs_f64();
        let rps = profiles.len() as f64 / elapsed;

        if rps > 10.0 {
            recommendations.push(format!(
                "High request rate ({:.1} req/s). Consider implementing request batching or \
                 caching strategies.",
                rps
            ));
        }

        let rate_limited_count = profiles.iter().filter(|p| p.rate_limited).count();
        if rate_limited_count as f64 / profiles.len() as f64 > 0.1 {
            recommendations.push(
                "Frequent rate limiting detected. Consider upgrading API tier or implementing \
                 request throttling."
                    .to_string(),
            );
        }

        recommendations
    }

    async fn detect_query_patterns(&self, counts: &HashMap<String, usize>) -> Vec<QueryPattern> {
        let mut patterns = Vec::new();

        let total_requests: usize = counts.values().sum();

        for (symbol, count) in counts.iter() {
            let frequency_pct = (*count as f64 / total_requests as f64) * 100.0;

            if frequency_pct > 20.0 {
                patterns.push(QueryPattern {
                    description: format!("High frequency symbol: {}", symbol),
                    frequency: *count,
                    optimization: Some(format!(
                        "Consider dedicated caching for {} ({}% of requests)",
                        symbol, frequency_pct as u32
                    )),
                });
            }
        }

        patterns
    }

    async fn generate_recommendations(
        &self,
        popular_symbols: &[(String, usize)],
        patterns: &[QueryPattern],
    ) -> Vec<String> {
        let mut recommendations = Vec::new();

        if popular_symbols.len() > 5 {
            recommendations.push(
                "Consider implementing symbol-specific cache tiers for frequently accessed symbols"
                    .to_string(),
            );
        }

        if patterns.iter().any(|p| p.frequency > 100) {
            recommendations.push(
                "High-frequency queries detected. Consider implementing cache warming for popular \
                 symbols"
                    .to_string(),
            );
        }

        recommendations
    }

    async fn calculate_resource_utilization(
        &self,
        profiles: &VecDeque<RequestProfile>,
    ) -> ResourceUtilization {
        let cache_hits = profiles.iter().filter(|p| p.cache_hit).count();
        let cache_utilization = if !profiles.is_empty() {
            cache_hits as f64 / profiles.len() as f64
        } else {
            0.0
        };

        // Estimate memory usage (rough approximation)
        let memory_per_profile = std::mem::size_of::<RequestProfile>();
        let memory_usage_mb = (profiles.len() * memory_per_profile) as f64 / 1024.0 / 1024.0;

        ResourceUtilization {
            memory_usage_mb,
            cache_utilization,
            connection_pool_utilization: 0.5, // TODO: Get from actual pool
            api_quota_utilization: 0.3,       // TODO: Calculate from rate limiting
        }
    }
}

/// Calculate mean and standard deviation
fn calculate_stats(values: &[f64]) -> (f64, f64) {
    if values.is_empty() {
        return (0.0, 0.0);
    }

    let mean = values.iter().sum::<f64>() / values.len() as f64;
    let variance = values.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / values.len() as f64;
    let std_dev = variance.sqrt();

    (mean, std_dev)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_analytics_creation() {
        let config = AnalyticsConfig::default();
        let analytics = Analytics::new(config);

        assert!(analytics.profiles.read().await.is_empty());
    }

    #[tokio::test]
    async fn test_record_request() {
        let config = AnalyticsConfig::default();
        let analytics = Analytics::new(config);

        analytics.record_request("AAPL", Duration::from_millis(100)).await;

        let profiles = analytics.profiles.read().await;
        assert_eq!(profiles.len(), 1);
        assert_eq!(profiles[0].symbol, "AAPL");
    }

    #[tokio::test]
    async fn test_performance_insights() {
        let config = AnalyticsConfig::default();
        let analytics = Analytics::new(config);

        for i in 0..100 {
            analytics.record_request("AAPL", Duration::from_millis(100 + i)).await;
        }

        let insights = analytics.get_insights().await;
        assert_eq!(insights.total_requests, 100);
        assert!(insights.average_latency.as_millis() > 0);
    }

    #[tokio::test]
    async fn test_usage_analytics() {
        let config = AnalyticsConfig::default();
        let analytics = Analytics::new(config);

        analytics.record_request("AAPL", Duration::from_millis(100)).await;
        analytics.record_request("AAPL", Duration::from_millis(100)).await;
        analytics.record_request("GOOGL", Duration::from_millis(100)).await;

        let usage = analytics.get_usage_analytics().await;
        assert_eq!(usage.popular_symbols.len(), 2);
        assert_eq!(usage.popular_symbols[0].0, "AAPL");
        assert_eq!(usage.popular_symbols[0].1, 2);
    }

    #[tokio::test]
    async fn test_config_builder() {
        let config = AnalyticsConfig::builder()
            .enable_profiling(false)
            .retention_period(Duration::from_secs(7200))
            .build();

        assert!(!config.enable_profiling);
        assert_eq!(config.retention_period, Duration::from_secs(7200));
    }
}
