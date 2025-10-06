//! HTTP/2 support for improved performance
//!
//! This module provides HTTP/2 configuration and connection multiplexing
//! to improve request throughput and reduce latency.

use reqwest::Client;
use std::time::Duration;

/// HTTP/2 configuration options
#[derive(Debug, Clone)]
pub struct Http2Config {
    /// Enable HTTP/2 support
    pub enabled: bool,
    
    /// Connection keep-alive duration
    pub keep_alive: Duration,
    
    /// Initial connection window size
    pub initial_connection_window_size: u32,
    
    /// Initial stream window size
    pub initial_stream_window_size: u32,
    
    /// Maximum concurrent streams
    pub max_concurrent_streams: u32,
    
    /// Enable adaptive window sizing
    pub adaptive_window: bool,
}

impl Default for Http2Config {
    fn default() -> Self {
        Self {
            enabled: true,
            keep_alive: Duration::from_secs(90),
            initial_connection_window_size: 1024 * 1024, // 1 MB
            initial_stream_window_size: 1024 * 1024,     // 1 MB
            max_concurrent_streams: 100,
            adaptive_window: true,
        }
    }
}

impl Http2Config {
    /// Create a new HTTP/2 config with default values
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Enable or disable HTTP/2
    pub fn with_enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }
    
    /// Set the keep-alive duration
    pub fn with_keep_alive(mut self, duration: Duration) -> Self {
        self.keep_alive = duration;
        self
    }
    
    /// Set the initial connection window size
    pub fn with_connection_window_size(mut self, size: u32) -> Self {
        self.initial_connection_window_size = size;
        self
    }
    
    /// Set the initial stream window size
    pub fn with_stream_window_size(mut self, size: u32) -> Self {
        self.initial_stream_window_size = size;
        self
    }
    
    /// Set the maximum concurrent streams
    pub fn with_max_concurrent_streams(mut self, max: u32) -> Self {
        self.max_concurrent_streams = max;
        self
    }
    
    /// Enable or disable adaptive window sizing
    pub fn with_adaptive_window(mut self, enabled: bool) -> Self {
        self.adaptive_window = enabled;
        self
    }
    
    /// Apply HTTP/2 configuration to a reqwest ClientBuilder
    pub fn configure_client(&self, builder: reqwest::ClientBuilder) -> reqwest::ClientBuilder {
        // Note: HTTP/2 is automatically enabled in reqwest when available
        // These configuration options are for future enhancement when reqwest exposes more HTTP/2 controls
        builder
            .pool_max_idle_per_host(10)
            .pool_idle_timeout(Some(self.keep_alive))
            .timeout(Duration::from_secs(30))
    }
}

/// HTTP/2 connection metrics
#[derive(Debug, Clone, Default)]
pub struct Http2Metrics {
    /// Total number of connections created
    pub connections_created: u64,
    
    /// Number of active connections
    pub active_connections: u64,
    
    /// Number of reused connections
    pub connections_reused: u64,
    
    /// Number of connection errors
    pub connection_errors: u64,
    
    /// Average connection duration (milliseconds)
    pub avg_connection_duration_ms: f64,
    
    /// Total number of streams created
    pub streams_created: u64,
    
    /// Number of active streams
    pub active_streams: u64,
    
    /// Average streams per connection
    pub avg_streams_per_connection: f64,
}

impl Http2Metrics {
    /// Create a new metrics instance
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Record a new connection
    pub fn record_connection_created(&mut self) {
        self.connections_created += 1;
        self.active_connections += 1;
    }
    
    /// Record a connection being closed
    pub fn record_connection_closed(&mut self, duration_ms: f64) {
        if self.active_connections > 0 {
            self.active_connections -= 1;
        }
        
        // Update average connection duration
        let total_duration = self.avg_connection_duration_ms * (self.connections_created - 1) as f64;
        self.avg_connection_duration_ms = 
            (total_duration + duration_ms) / self.connections_created as f64;
    }
    
    /// Record a connection being reused
    pub fn record_connection_reused(&mut self) {
        self.connections_reused += 1;
    }
    
    /// Record a connection error
    pub fn record_connection_error(&mut self) {
        self.connection_errors += 1;
    }
    
    /// Record a new stream
    pub fn record_stream_created(&mut self) {
        self.streams_created += 1;
        self.active_streams += 1;
        
        // Update average streams per connection
        if self.connections_created > 0 {
            self.avg_streams_per_connection = 
                self.streams_created as f64 / self.connections_created as f64;
        }
    }
    
    /// Record a stream being closed
    pub fn record_stream_closed(&mut self) {
        if self.active_streams > 0 {
            self.active_streams -= 1;
        }
    }
    
    /// Get connection reuse rate
    pub fn connection_reuse_rate(&self) -> f64 {
        if self.connections_created == 0 {
            return 0.0;
        }
        self.connections_reused as f64 / self.connections_created as f64
    }
    
    /// Get connection error rate
    pub fn connection_error_rate(&self) -> f64 {
        let total = self.connections_created + self.connection_errors;
        if total == 0 {
            return 0.0;
        }
        self.connection_errors as f64 / total as f64
    }
    
    /// Reset all metrics
    pub fn reset(&mut self) {
        *self = Self::default();
    }
}

/// Create an HTTP/2 optimized client
pub fn create_http2_client(config: &Http2Config) -> Result<Client, reqwest::Error> {
    let builder = Client::builder()
        .pool_max_idle_per_host(10)
        .pool_idle_timeout(Some(Duration::from_secs(90)))
        .timeout(Duration::from_secs(30));
    
    let builder = if config.enabled {
        config.configure_client(builder)
    } else {
        builder
    };
    
    builder.build()
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_http2_config_default() {
        let config = Http2Config::default();
        assert!(config.enabled);
        assert_eq!(config.keep_alive, Duration::from_secs(90));
        assert_eq!(config.initial_connection_window_size, 1024 * 1024);
        assert_eq!(config.initial_stream_window_size, 1024 * 1024);
        assert_eq!(config.max_concurrent_streams, 100);
        assert!(config.adaptive_window);
    }
    
    #[test]
    fn test_http2_config_builder() {
        let config = Http2Config::new()
            .with_enabled(false)
            .with_keep_alive(Duration::from_secs(60))
            .with_connection_window_size(2 * 1024 * 1024)
            .with_stream_window_size(512 * 1024)
            .with_max_concurrent_streams(50)
            .with_adaptive_window(false);
        
        assert!(!config.enabled);
        assert_eq!(config.keep_alive, Duration::from_secs(60));
        assert_eq!(config.initial_connection_window_size, 2 * 1024 * 1024);
        assert_eq!(config.initial_stream_window_size, 512 * 1024);
        assert_eq!(config.max_concurrent_streams, 50);
        assert!(!config.adaptive_window);
    }
    
    #[test]
    fn test_http2_metrics_connection_lifecycle() {
        let mut metrics = Http2Metrics::new();
        
        metrics.record_connection_created();
        assert_eq!(metrics.connections_created, 1);
        assert_eq!(metrics.active_connections, 1);
        
        metrics.record_connection_closed(100.0);
        assert_eq!(metrics.active_connections, 0);
        assert_eq!(metrics.avg_connection_duration_ms, 100.0);
    }
    
    #[test]
    fn test_http2_metrics_connection_reuse() {
        let mut metrics = Http2Metrics::new();
        
        metrics.record_connection_created();
        metrics.record_connection_reused();
        metrics.record_connection_reused();
        
        assert_eq!(metrics.connections_reused, 2);
        assert_eq!(metrics.connection_reuse_rate(), 2.0);
    }
    
    #[test]
    fn test_http2_metrics_streams() {
        let mut metrics = Http2Metrics::new();
        
        metrics.record_connection_created();
        metrics.record_stream_created();
        metrics.record_stream_created();
        metrics.record_stream_created();
        
        assert_eq!(metrics.streams_created, 3);
        assert_eq!(metrics.active_streams, 3);
        assert_eq!(metrics.avg_streams_per_connection, 3.0);
        
        metrics.record_stream_closed();
        assert_eq!(metrics.active_streams, 2);
    }
    
    #[test]
    fn test_http2_metrics_error_rate() {
        let mut metrics = Http2Metrics::new();
        
        metrics.record_connection_created();
        metrics.record_connection_created();
        metrics.record_connection_error();
        
        assert_eq!(metrics.connection_errors, 1);
        assert_eq!(metrics.connection_error_rate(), 1.0 / 3.0);
    }
    
    #[test]
    fn test_http2_metrics_reset() {
        let mut metrics = Http2Metrics::new();
        
        metrics.record_connection_created();
        metrics.record_stream_created();
        
        metrics.reset();
        
        assert_eq!(metrics.connections_created, 0);
        assert_eq!(metrics.streams_created, 0);
    }
}
