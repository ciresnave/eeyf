//! Fallback strategies for enhanced reliability
//!
//! This module provides fallback mechanisms when the primary Yahoo Finance API
//! is unavailable or returns errors. Strategies include:
//! - Cached response fallback
//! - Degraded mode operation
//! - Alternative data sources
//!
//! # Example
//!
//! ```no_run
//! use eeyf::fallback::{FallbackStrategy, FallbackConfig};
//!
//! let fallback = FallbackConfig::new()
//!     .with_cached_fallback(true)
//!     .with_degraded_mode(true)
//!     .build();
//! ```

use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Fallback configuration for reliability
#[derive(Debug, Clone)]
pub struct FallbackConfig {
    /// Enable cached response fallback
    pub use_cached_fallback: bool,
    
    /// Maximum age of cached data to use as fallback
    pub max_cache_age: Duration,
    
    /// Enable degraded mode (limited functionality)
    pub degraded_mode: bool,
    
    /// Alternative data sources
    pub alternative_sources: Vec<AlternativeSource>,
    
    /// Fallback timeout
    pub fallback_timeout: Duration,
}

/// Alternative data source configuration
#[derive(Debug, Clone)]
pub struct AlternativeSource {
    /// Source name
    pub name: String,
    
    /// Source URL or endpoint
    pub endpoint: String,
    
    /// Priority (lower = higher priority)
    pub priority: u8,
    
    /// Whether this source is currently healthy
    pub healthy: bool,
    
    /// Data transformation needed
    pub transform: Option<DataTransform>,
}

/// Data transformation type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DataTransform {
    /// No transformation needed
    None,
    
    /// Transform from Alpha Vantage format
    AlphaVantage,
    
    /// Transform from IEX Cloud format
    IexCloud,
    
    /// Transform from Polygon.io format
    Polygon,
    
    /// Custom transformation
    Custom,
}

/// Fallback strategy to use
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FallbackStrategy {
    /// Use cached data if available
    CachedData,
    
    /// Use alternative data source
    AlternativeSource,
    
    /// Return degraded response (limited data)
    DegradedMode,
    
    /// Fail immediately
    Fail,
}

/// Result of a fallback operation
#[derive(Debug, Clone)]
pub struct FallbackResult<T> {
    /// The data returned
    pub data: Option<T>,
    
    /// Strategy that was used
    pub strategy: FallbackStrategy,
    
    /// Whether the fallback was successful
    pub success: bool,
    
    /// Age of the data (if from cache)
    pub data_age: Option<Duration>,
    
    /// Source that provided the data
    pub source: Option<String>,
}

/// Degraded mode response with limited data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DegradedResponse {
    /// Ticker symbol
    pub symbol: String,
    
    /// Last known price (may be stale)
    pub last_price: Option<f64>,
    
    /// Timestamp of last known price
    pub last_updated: Option<u64>,
    
    /// Warning message
    pub warning: String,
    
    /// Degraded mode indicator
    pub degraded: bool,
}

impl Default for FallbackConfig {
    fn default() -> Self {
        Self {
            use_cached_fallback: true,
            max_cache_age: Duration::from_secs(3600), // 1 hour
            degraded_mode: true,
            alternative_sources: Vec::new(),
            fallback_timeout: Duration::from_secs(5),
        }
    }
}

impl FallbackConfig {
    /// Create a new fallback configuration
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Enable or disable cached fallback
    pub fn with_cached_fallback(mut self, enabled: bool) -> Self {
        self.use_cached_fallback = enabled;
        self
    }
    
    /// Set maximum cache age for fallback
    pub fn with_max_cache_age(mut self, age: Duration) -> Self {
        self.max_cache_age = age;
        self
    }
    
    /// Enable or disable degraded mode
    pub fn with_degraded_mode(mut self, enabled: bool) -> Self {
        self.degraded_mode = enabled;
        self
    }
    
    /// Add an alternative data source
    pub fn add_alternative_source(mut self, source: AlternativeSource) -> Self {
        self.alternative_sources.push(source);
        self.alternative_sources.sort_by_key(|s| s.priority);
        self
    }
    
    /// Set fallback timeout
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.fallback_timeout = timeout;
        self
    }
    
    /// Build the configuration
    pub fn build(self) -> Self {
        self
    }
    
    /// Determine which fallback strategy to use
    pub fn select_strategy(&self, error_type: ErrorType) -> FallbackStrategy {
        match error_type {
            ErrorType::NetworkError | ErrorType::Timeout => {
                if self.use_cached_fallback {
                    FallbackStrategy::CachedData
                } else if !self.alternative_sources.is_empty() {
                    FallbackStrategy::AlternativeSource
                } else if self.degraded_mode {
                    FallbackStrategy::DegradedMode
                } else {
                    FallbackStrategy::Fail
                }
            }
            
            ErrorType::RateLimit => {
                if !self.alternative_sources.is_empty() {
                    FallbackStrategy::AlternativeSource
                } else if self.use_cached_fallback {
                    FallbackStrategy::CachedData
                } else if self.degraded_mode {
                    FallbackStrategy::DegradedMode
                } else {
                    FallbackStrategy::Fail
                }
            }
            
            ErrorType::ServiceUnavailable => {
                if !self.alternative_sources.is_empty() {
                    FallbackStrategy::AlternativeSource
                } else if self.use_cached_fallback {
                    FallbackStrategy::CachedData
                } else {
                    FallbackStrategy::Fail
                }
            }
            
            ErrorType::InvalidResponse => {
                if self.use_cached_fallback {
                    FallbackStrategy::CachedData
                } else {
                    FallbackStrategy::Fail
                }
            }
            
            ErrorType::CircuitBreakerOpen => {
                if self.use_cached_fallback {
                    FallbackStrategy::CachedData
                } else if self.degraded_mode {
                    FallbackStrategy::DegradedMode
                } else {
                    FallbackStrategy::Fail
                }
            }
        }
    }
    
    /// Get the next available alternative source
    pub fn next_alternative_source(&self) -> Option<&AlternativeSource> {
        self.alternative_sources.iter().find(|s| s.healthy)
    }
}

/// Error type for fallback decision
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorType {
    /// Network connectivity error
    NetworkError,
    
    /// Request timeout
    Timeout,
    
    /// Rate limit exceeded
    RateLimit,
    
    /// Service unavailable (5xx errors)
    ServiceUnavailable,
    
    /// Invalid or malformed response
    InvalidResponse,
    
    /// Circuit breaker is open
    CircuitBreakerOpen,
}

impl AlternativeSource {
    /// Create a new alternative source
    pub fn new(name: impl Into<String>, endpoint: impl Into<String>, priority: u8) -> Self {
        Self {
            name: name.into(),
            endpoint: endpoint.into(),
            priority,
            healthy: true,
            transform: None,
        }
    }
    
    /// Set the data transformation type
    pub fn with_transform(mut self, transform: DataTransform) -> Self {
        self.transform = Some(transform);
        self
    }
    
    /// Mark source as healthy or unhealthy
    pub fn set_healthy(&mut self, healthy: bool) {
        self.healthy = healthy;
    }
    
    /// Check if source is healthy
    pub fn is_healthy(&self) -> bool {
        self.healthy
    }
}

impl<T> FallbackResult<T> {
    /// Create a successful fallback result
    pub fn success(data: T, strategy: FallbackStrategy) -> Self {
        Self {
            data: Some(data),
            strategy,
            success: true,
            data_age: None,
            source: None,
        }
    }
    
    /// Create a failed fallback result
    pub fn failure(strategy: FallbackStrategy) -> Self {
        Self {
            data: None,
            strategy,
            success: false,
            data_age: None,
            source: None,
        }
    }
    
    /// Set the data age
    pub fn with_age(mut self, age: Duration) -> Self {
        self.data_age = Some(age);
        self
    }
    
    /// Set the source
    pub fn with_source(mut self, source: impl Into<String>) -> Self {
        self.source = Some(source.into());
        self
    }
    
    /// Check if fallback was successful
    pub fn is_success(&self) -> bool {
        self.success && self.data.is_some()
    }
}

impl DegradedResponse {
    /// Create a new degraded response
    pub fn new(symbol: impl Into<String>, warning: impl Into<String>) -> Self {
        Self {
            symbol: symbol.into(),
            last_price: None,
            last_updated: None,
            warning: warning.into(),
            degraded: true,
        }
    }
    
    /// Set the last known price
    pub fn with_last_price(mut self, price: f64, timestamp: u64) -> Self {
        self.last_price = Some(price);
        self.last_updated = Some(timestamp);
        self
    }
}

/// Fallback executor that applies strategies
pub struct FallbackExecutor {
    config: FallbackConfig,
}

impl FallbackExecutor {
    /// Create a new fallback executor
    pub fn new(config: FallbackConfig) -> Self {
        Self { config }
    }
    
    /// Execute a fallback strategy
    pub async fn execute<T, F, E>(
        &self,
        strategy: FallbackStrategy,
        fallback_fn: F,
    ) -> Result<FallbackResult<T>, E>
    where
        F: std::future::Future<Output = Result<T, E>>,
    {
        // Apply timeout to fallback operation
        let timeout_result = tokio::time::timeout(self.config.fallback_timeout, fallback_fn).await;
        
        match timeout_result {
            Ok(Ok(data)) => Ok(FallbackResult::success(data, strategy)),
            Ok(Err(e)) => Err(e),
            Err(_) => Ok(FallbackResult::failure(strategy)),
        }
    }
    
    /// Get the configuration
    pub fn config(&self) -> &FallbackConfig {
        &self.config
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_fallback_config() {
        let config = FallbackConfig::new()
            .with_cached_fallback(true)
            .with_max_cache_age(Duration::from_secs(1800))
            .with_degraded_mode(false)
            .build();
        
        assert!(config.use_cached_fallback);
        assert_eq!(config.max_cache_age, Duration::from_secs(1800));
        assert!(!config.degraded_mode);
    }
    
    #[test]
    fn test_strategy_selection() {
        let config = FallbackConfig::new()
            .with_cached_fallback(true)
            .with_degraded_mode(true);
        
        assert_eq!(
            config.select_strategy(ErrorType::NetworkError),
            FallbackStrategy::CachedData
        );
        
        assert_eq!(
            config.select_strategy(ErrorType::CircuitBreakerOpen),
            FallbackStrategy::CachedData
        );
    }
    
    #[test]
    fn test_alternative_source() {
        let mut source = AlternativeSource::new("Backup API", "https://backup.api", 1)
            .with_transform(DataTransform::AlphaVantage);
        
        assert!(source.is_healthy());
        
        source.set_healthy(false);
        assert!(!source.is_healthy());
    }
    
    #[test]
    fn test_fallback_result() {
        let result = FallbackResult::success("data", FallbackStrategy::CachedData)
            .with_age(Duration::from_secs(300))
            .with_source("cache");
        
        assert!(result.is_success());
        assert_eq!(result.data, Some("data"));
        assert_eq!(result.data_age, Some(Duration::from_secs(300)));
    }
    
    #[test]
    fn test_degraded_response() {
        let response = DegradedResponse::new("AAPL", "Service temporarily unavailable")
            .with_last_price(150.25, 1234567890);
        
        assert_eq!(response.symbol, "AAPL");
        assert_eq!(response.last_price, Some(150.25));
        assert!(response.degraded);
    }
    
    #[test]
    fn test_source_priority_sorting() {
        let config = FallbackConfig::new()
            .add_alternative_source(AlternativeSource::new("Source3", "url3", 3))
            .add_alternative_source(AlternativeSource::new("Source1", "url1", 1))
            .add_alternative_source(AlternativeSource::new("Source2", "url2", 2));
        
        assert_eq!(config.alternative_sources[0].priority, 1);
        assert_eq!(config.alternative_sources[1].priority, 2);
        assert_eq!(config.alternative_sources[2].priority, 3);
    }
}
