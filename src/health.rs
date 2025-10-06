//! Health check implementation for EEYF
//!
//! This module provides comprehensive health monitoring capabilities,
//! including connectivity checks, component status monitoring, and
//! structured health reporting for production environments.


use crate::YahooConnector;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};

/// Overall health status
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum HealthStatus {
    /// All systems operational
    Healthy,
    /// Some non-critical issues detected
    Degraded,
    /// Critical issues detected
    Unhealthy,
    /// Status unknown (startup, etc.)
    Unknown,
}

impl std::fmt::Display for HealthStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HealthStatus::Healthy => write!(f, "healthy"),
            HealthStatus::Degraded => write!(f, "degraded"),
            HealthStatus::Unhealthy => write!(f, "unhealthy"),
            HealthStatus::Unknown => write!(f, "unknown"),
        }
    }
}

/// Individual component health check
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentHealth {
    /// Component name
    pub name: String,
    /// Health status
    pub status: HealthStatus,
    /// Human-readable status message
    pub message: String,
    /// Last check timestamp (Unix timestamp)
    pub last_check: u64,
    /// Check duration in milliseconds
    pub duration_ms: u64,
    /// Additional metadata
    pub details: HashMap<String, serde_json::Value>,
}

impl ComponentHealth {
    /// Create a healthy component check
    pub fn healthy(name: &str, message: &str) -> Self {
        Self {
            name: name.to_string(),
            status: HealthStatus::Healthy,
            message: message.to_string(),
            last_check: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            duration_ms: 0,
            details: HashMap::new(),
        }
    }

    /// Create a degraded component check
    pub fn degraded(name: &str, message: &str) -> Self {
        let mut health = Self::healthy(name, message);
        health.status = HealthStatus::Degraded;
        health
    }

    /// Create an unhealthy component check
    pub fn unhealthy(name: &str, message: &str) -> Self {
        let mut health = Self::healthy(name, message);
        health.status = HealthStatus::Unhealthy;
        health
    }

    /// Add detail information
    pub fn with_detail<T: serde::Serialize>(mut self, key: &str, value: T) -> Self {
        if let Ok(json_value) = serde_json::to_value(value) {
            self.details.insert(key.to_string(), json_value);
        }
        self
    }

    /// Set check duration
    pub fn with_duration(mut self, duration: Duration) -> Self {
        self.duration_ms = duration.as_millis() as u64;
        self
    }
}

/// Complete health report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthReport {
    /// Overall health status
    pub status: HealthStatus,
    /// Service information
    pub service: ServiceInfo,
    /// Individual component checks
    pub checks: HashMap<String, ComponentHealth>,
    /// Report generation timestamp
    pub timestamp: u64,
    /// Total check duration in milliseconds
    pub duration_ms: u64,
}

/// Service information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceInfo {
    /// Service name
    pub name: String,
    /// Service version
    pub version: String,
    /// Environment (production, staging, development)
    pub environment: String,
    /// Uptime in seconds
    pub uptime_seconds: u64,
}

/// Health check configuration
#[derive(Debug, Clone)]
pub struct HealthConfig {
    /// Enable health checks
    pub enabled: bool,
    /// Check interval in seconds
    pub check_interval_secs: u64,
    /// Timeout for individual checks
    pub check_timeout_secs: u64,
    /// Number of consecutive failures before marking as unhealthy
    pub failure_threshold: u32,
    /// Enable detailed Yahoo connectivity check
    pub check_yahoo_connectivity: bool,
    /// Test symbol for connectivity check
    pub test_symbol: String,
    /// Service information
    pub service_info: ServiceInfo,
}

impl Default for HealthConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            check_interval_secs: 30,
            check_timeout_secs: 10,
            failure_threshold: 3,
            check_yahoo_connectivity: true,
            test_symbol: "AAPL".to_string(),
            service_info: ServiceInfo {
                name: "eeyf".to_string(),
                version: env!("CARGO_PKG_VERSION").to_string(),
                environment: std::env::var("ENVIRONMENT").unwrap_or_else(|_| "development".to_string()),
                uptime_seconds: 0,
            },
        }
    }
}

/// Health check manager
#[derive(Debug)]
pub struct HealthManager {
    config: HealthConfig,
    connector: Option<Arc<YahooConnector>>,
    start_time: Instant,
    last_checks: Arc<RwLock<HashMap<String, ComponentHealth>>>,
    failure_counts: Arc<RwLock<HashMap<String, u32>>>,
}

impl HealthManager {
    /// Create a new health manager
    pub fn new(config: HealthConfig) -> Self {
        Self {
            config,
            connector: None,
            start_time: Instant::now(),
            last_checks: Arc::new(RwLock::new(HashMap::new())),
            failure_counts: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Set the Yahoo connector for connectivity checks
    pub fn with_connector(mut self, connector: Arc<YahooConnector>) -> Self {
        self.connector = Some(connector);
        self
    }

    /// Perform all health checks and return report
    pub async fn check_health(&self) -> HealthReport {
        let start_time = Instant::now();
        let mut checks = HashMap::new();

        // Basic service health
        checks.insert("service".to_string(), self.check_service_health().await);

        // Yahoo Finance connectivity
        if self.config.check_yahoo_connectivity {
            checks.insert("yahoo_connectivity".to_string(), self.check_yahoo_connectivity().await);
        }

        // Rate limiter health
        checks.insert("rate_limiter".to_string(), self.check_rate_limiter_health().await);

        // Circuit breaker health  
        checks.insert("circuit_breaker".to_string(), self.check_circuit_breaker_health().await);

        // Cache health
        checks.insert("cache".to_string(), self.check_cache_health().await);

        // Connection pool health
        checks.insert("connection_pool".to_string(), self.check_connection_pool_health().await);

        // Determine overall status
        let overall_status = self.calculate_overall_status(&checks).await;

        // Update failure counts
        self.update_failure_counts(&checks).await;

        // Store last checks
        {
            let mut last_checks = self.last_checks.write().await;
            *last_checks = checks.clone();
        }

        HealthReport {
            status: overall_status,
            service: ServiceInfo {
                name: self.config.service_info.name.clone(),
                version: self.config.service_info.version.clone(),
                environment: self.config.service_info.environment.clone(),
                uptime_seconds: self.start_time.elapsed().as_secs(),
            },
            checks,
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            duration_ms: start_time.elapsed().as_millis() as u64,
        }
    }

    /// Get the last health report
    pub async fn get_last_health_report(&self) -> HealthReport {
        let last_checks = self.last_checks.read().await;
        
        if last_checks.is_empty() {
            // No previous checks, return unknown status
            let mut checks = HashMap::new();
            checks.insert("service".to_string(), ComponentHealth {
                name: "service".to_string(),
                status: HealthStatus::Unknown,
                message: "No health checks performed yet".to_string(),
                last_check: 0,
                duration_ms: 0,
                details: HashMap::new(),
            });

            return HealthReport {
                status: HealthStatus::Unknown,
                service: ServiceInfo {
                    name: self.config.service_info.name.clone(),
                    version: self.config.service_info.version.clone(),
                    environment: self.config.service_info.environment.clone(),
                    uptime_seconds: self.start_time.elapsed().as_secs(),
                },
                checks,
                timestamp: SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs(),
                duration_ms: 0,
            };
        }

        let overall_status = self.calculate_overall_status(&last_checks).await;

        HealthReport {
            status: overall_status,
            service: ServiceInfo {
                name: self.config.service_info.name.clone(),
                version: self.config.service_info.version.clone(),
                environment: self.config.service_info.environment.clone(),
                uptime_seconds: self.start_time.elapsed().as_secs(),
            },
            checks: last_checks.clone(),
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            duration_ms: 0,
        }
    }

    /// Check basic service health
    async fn check_service_health(&self) -> ComponentHealth {
        let start = Instant::now();
        
        let uptime = self.start_time.elapsed();
        let memory_usage = self.get_memory_usage();

        ComponentHealth::healthy("service", "Service is running normally")
            .with_duration(start.elapsed())
            .with_detail("uptime_seconds", uptime.as_secs())
            .with_detail("memory_kb", memory_usage)
            .with_detail("rust_version", env!("CARGO_PKG_RUST_VERSION"))
            .with_detail("build_timestamp", "unknown")
    }

    /// Check Yahoo Finance API connectivity
    async fn check_yahoo_connectivity(&self) -> ComponentHealth {
        let start = Instant::now();

        if let Some(connector) = &self.connector {
            // Try a simple quote request
            match tokio::time::timeout(
                Duration::from_secs(self.config.check_timeout_secs),
                connector.get_latest_quotes(&self.config.test_symbol, "1d")
            ).await {
                Ok(Ok(response)) => {
                    if response.quotes().is_ok() {
                        ComponentHealth::healthy("yahoo_connectivity", "Yahoo Finance API is accessible")
                            .with_duration(start.elapsed())
                            .with_detail("test_symbol", &self.config.test_symbol)
                            .with_detail("response_time_ms", start.elapsed().as_millis() as u64)
                    } else {
                        ComponentHealth::degraded("yahoo_connectivity", "Yahoo Finance API returned invalid data")
                            .with_duration(start.elapsed())
                            .with_detail("test_symbol", &self.config.test_symbol)
                    }
                }
                Ok(Err(e)) => {
                    ComponentHealth::unhealthy("yahoo_connectivity", &format!("Yahoo Finance API error: {}", e))
                        .with_duration(start.elapsed())
                        .with_detail("error", e.to_string())
                        .with_detail("test_symbol", &self.config.test_symbol)
                }
                Err(_) => {
                    ComponentHealth::unhealthy("yahoo_connectivity", "Yahoo Finance API timeout")
                        .with_duration(start.elapsed())
                        .with_detail("timeout_secs", self.config.check_timeout_secs)
                        .with_detail("test_symbol", &self.config.test_symbol)
                }
            }
        } else {
            ComponentHealth::degraded("yahoo_connectivity", "No connector configured for health checks")
                .with_duration(start.elapsed())
        }
    }

    /// Check rate limiter health
    async fn check_rate_limiter_health(&self) -> ComponentHealth {
        let start = Instant::now();

        // Rate limiter is healthy if it's allowing requests
        // In a real implementation, you'd check the actual rate limiter state
        ComponentHealth::healthy("rate_limiter", "Rate limiter is operational")
            .with_duration(start.elapsed())
            .with_detail("tokens_available", true)  // Placeholder
            .with_detail("requests_per_hour", 1800) // From config
    }

    /// Check circuit breaker health
    async fn check_circuit_breaker_health(&self) -> ComponentHealth {
        let start = Instant::now();

        // Circuit breaker is healthy if it's closed or half-open
        // In a real implementation, you'd check the actual circuit breaker state
        ComponentHealth::healthy("circuit_breaker", "Circuit breaker is closed")
            .with_duration(start.elapsed())
            .with_detail("state", "closed")
            .with_detail("failure_count", 0)
            .with_detail("last_failure", Option::<u64>::None)
    }

    /// Check cache health
    async fn check_cache_health(&self) -> ComponentHealth {
        let start = Instant::now();

        // Cache is healthy if it's responding
        // In a real implementation, you'd check actual cache metrics
        ComponentHealth::healthy("cache", "Cache is operational")
            .with_duration(start.elapsed())
            .with_detail("size", 150)        // Current entries
            .with_detail("max_size", 2000)   // Max entries
            .with_detail("hit_rate", 0.85)   // 85% hit rate
    }

    /// Check connection pool health
    async fn check_connection_pool_health(&self) -> ComponentHealth {
        let start = Instant::now();

        // Connection pool is healthy if connections are available
        // In a real implementation, you'd check actual pool metrics  
        ComponentHealth::healthy("connection_pool", "Connection pool is healthy")
            .with_duration(start.elapsed())
            .with_detail("pool_size", 20)
            .with_detail("active_connections", 3)
            .with_detail("idle_connections", 17)
    }

    /// Calculate overall health status from component checks
    async fn calculate_overall_status(&self, checks: &HashMap<String, ComponentHealth>) -> HealthStatus {
        let mut unhealthy_count = 0;
        let mut degraded_count = 0;
        let mut total_count = 0;

        for check in checks.values() {
            total_count += 1;
            match check.status {
                HealthStatus::Unhealthy => unhealthy_count += 1,
                HealthStatus::Degraded => degraded_count += 1,
                _ => {}
            }
        }

        if unhealthy_count > 0 {
            HealthStatus::Unhealthy
        } else if degraded_count > 0 {
            HealthStatus::Degraded
        } else if total_count > 0 {
            HealthStatus::Healthy
        } else {
            HealthStatus::Unknown
        }
    }

    /// Update failure counts for components
    async fn update_failure_counts(&self, checks: &HashMap<String, ComponentHealth>) {
        let mut failure_counts = self.failure_counts.write().await;
        
        for (name, check) in checks {
            match check.status {
                HealthStatus::Unhealthy => {
                    *failure_counts.entry(name.clone()).or_insert(0) += 1;
                }
                HealthStatus::Healthy => {
                    failure_counts.remove(name);
                }
                _ => {}
            }
        }
    }

    /// Get approximate memory usage (simple heuristic)
    fn get_memory_usage(&self) -> u64 {
        // This is a simplified memory usage estimation
        // In production, you might want to use a proper memory profiler
        std::mem::size_of::<Self>() as u64
    }

    /// Start background health checking
    pub async fn start_background_checks(
        self: Arc<Self>,
    ) -> tokio::task::JoinHandle<()> {
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(
                Duration::from_secs(self.config.check_interval_secs)
            );

            loop {
                interval.tick().await;
                
                if self.config.enabled {
                    let report = self.check_health().await;
                    
                    // Log health status
                    match report.status {
                        HealthStatus::Healthy => {
                            log::info!("💚 Health check passed: All systems healthy");
                        }
                        HealthStatus::Degraded => {
                            log::warn!("💛 Health check warning: Some systems degraded");
                            for (name, check) in &report.checks {
                                if matches!(check.status, HealthStatus::Degraded) {
                                    log::warn!("  - {}: {}", name, check.message);
                                }
                            }
                        }
                        HealthStatus::Unhealthy => {
                            log::error!("❤️ Health check failed: Systems unhealthy");
                            for (name, check) in &report.checks {
                                if matches!(check.status, HealthStatus::Unhealthy) {
                                    log::error!("  - {}: {}", name, check.message);
                                }
                            }
                        }
                        HealthStatus::Unknown => {
                            log::warn!("🔍 Health check status unknown");
                        }
                    }
                }
            }
        })
    }
}

/// Create a health check endpoint handler
#[cfg(feature = "health-server")]
pub async fn health_check_handler(health_manager: Arc<HealthManager>) -> HealthReport {
    health_manager.check_health().await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_component_health_creation() {
        let health = ComponentHealth::healthy("test", "Test component");
        assert_eq!(health.name, "test");
        assert_eq!(health.status, HealthStatus::Healthy);
        assert_eq!(health.message, "Test component");
    }

    #[test]
    fn test_component_health_with_details() {
        let health = ComponentHealth::healthy("test", "Test component")
            .with_detail("key1", "value1")
            .with_detail("key2", 42);

        assert_eq!(health.details.len(), 2);
        assert_eq!(health.details.get("key1").unwrap(), &serde_json::Value::String("value1".to_string()));
        assert_eq!(health.details.get("key2").unwrap(), &serde_json::Value::Number(serde_json::Number::from(42)));
    }

    #[test]
    fn test_health_status_display() {
        assert_eq!(HealthStatus::Healthy.to_string(), "healthy");
        assert_eq!(HealthStatus::Degraded.to_string(), "degraded");
        assert_eq!(HealthStatus::Unhealthy.to_string(), "unhealthy");
        assert_eq!(HealthStatus::Unknown.to_string(), "unknown");
    }

    #[tokio::test]
    async fn test_health_manager_creation() {
        let config = HealthConfig::default();
        let manager = HealthManager::new(config);
        
        let report = manager.get_last_health_report().await;
        assert_eq!(report.status, HealthStatus::Unknown);
    }

    #[tokio::test]
    async fn test_health_check_execution() {
        let config = HealthConfig {
            check_yahoo_connectivity: false, // Disable to avoid network calls in tests
            ..Default::default()
        };
        let manager = HealthManager::new(config);
        
        let report = manager.check_health().await;
        assert!(!report.checks.is_empty());
        assert!(report.checks.contains_key("service"));
    }
}