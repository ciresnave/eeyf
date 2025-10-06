//! Advanced Connection Pooling for EEYF
//!
//! This module provides intelligent HTTP connection pooling and management
//! for optimal performance with Yahoo Finance API:
//!
//! - Dynamic connection pool scaling based on load
//! - Connection health monitoring and automatic recovery
//! - Request load balancing across available connections
//! - Connection lifecycle management with graceful shutdown
//! - Connection pooling analytics and performance monitoring

use crate::yahoo_error::YahooError;
use reqwest::{Client, ClientBuilder};
use std::collections::VecDeque;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::{Arc, Weak};
use std::time::{Duration, Instant, SystemTime};
use tokio::sync::{Mutex, RwLock, Semaphore};
use tokio::time::sleep;

/// Connection pool configuration
#[derive(Debug, Clone)]
pub struct ConnectionPoolConfig {
    /// Minimum number of connections to maintain
    pub min_connections: usize,
    /// Maximum number of connections allowed
    pub max_connections: usize,
    /// Connection timeout duration
    pub connection_timeout: Duration,
    /// Request timeout duration
    pub request_timeout: Duration,
    /// Idle connection timeout
    pub idle_timeout: Duration,
    /// Health check interval
    pub health_check_interval: Duration,
    /// Maximum connection lifetime
    pub max_connection_lifetime: Duration,
    /// Enable connection pooling analytics
    pub enable_analytics: bool,
    /// Connection keep-alive settings
    pub keep_alive_config: KeepAliveConfig,
}

/// Keep-alive configuration for connections
#[derive(Debug, Clone)]
pub struct KeepAliveConfig {
    /// Enable TCP keep-alive
    pub enabled: bool,
    /// Keep-alive timeout
    pub timeout: Duration,
    /// Keep-alive interval
    pub interval: Duration,
    /// Keep-alive probe count
    pub probe_count: u32,
}

/// Connection health status
#[derive(Debug, Clone, PartialEq)]
pub enum ConnectionHealth {
    /// Connection is healthy and ready
    Healthy,
    /// Connection is degraded but usable
    Degraded,
    /// Connection is unhealthy and should be replaced
    Unhealthy,
    /// Connection is being tested
    Testing,
}

/// Pooled connection wrapper
#[derive(Debug)]
pub struct PooledConnection {
    /// The underlying HTTP client
    pub client: Client,
    /// Connection ID for tracking
    pub id: u64,
    /// When this connection was created
    pub created_at: Instant,
    /// Last successful request time
    pub last_used: Instant,
    /// Number of requests made with this connection
    pub request_count: AtomicU64,
    /// Current health status
    pub health: Arc<RwLock<ConnectionHealth>>,
    /// Connection-specific configuration
    pub config: ConnectionConfig,
}

/// Per-connection configuration
#[derive(Debug, Clone)]
pub struct ConnectionConfig {
    /// User agent string
    pub user_agent: String,
    /// Custom headers
    pub headers: Vec<(String, String)>,
    /// Proxy configuration
    pub proxy: Option<String>,
    /// TLS configuration
    pub tls_config: TlsConfig,
}

/// TLS configuration for connections
#[derive(Debug, Clone)]
pub struct TlsConfig {
    /// Accept invalid certificates (for testing)
    pub accept_invalid_certs: bool,
    /// Accept invalid hostnames
    pub accept_invalid_hostnames: bool,
    /// Minimum TLS version
    pub min_tls_version: TlsVersion,
}

/// TLS version specification
#[derive(Debug, Clone, PartialEq)]
pub enum TlsVersion {
    TlsV1_0,
    TlsV1_1,
    TlsV1_2,
    TlsV1_3,
}

/// Connection pool statistics
#[derive(Debug, Clone)]
pub struct PoolStats {
    /// Total active connections
    pub active_connections: usize,
    /// Total idle connections
    pub idle_connections: usize,
    /// Total requests served
    pub total_requests: u64,
    /// Failed requests count
    pub failed_requests: u64,
    /// Average request duration
    pub avg_request_duration_ms: f64,
    /// Pool efficiency (0.0 - 1.0)
    pub pool_efficiency: f64,
    /// Connection health distribution
    pub health_distribution: HealthDistribution,
    /// Pool utilization over time
    pub utilization_history: VecDeque<UtilizationSnapshot>,
}

/// Health distribution across connections
#[derive(Debug, Clone)]
pub struct HealthDistribution {
    pub healthy_count: usize,
    pub degraded_count: usize,
    pub unhealthy_count: usize,
    pub testing_count: usize,
}

/// Point-in-time utilization snapshot
#[derive(Debug, Clone)]
pub struct UtilizationSnapshot {
    pub timestamp: SystemTime,
    pub active_connections: usize,
    pub pending_requests: usize,
    pub throughput_rps: f64,
}

/// Advanced connection pool manager
pub struct ConnectionPool {
    /// Pool configuration
    config: ConnectionPoolConfig,
    /// Available connections
    connections: Arc<Mutex<VecDeque<Arc<PooledConnection>>>>,
    /// Active (in-use) connections tracking
    active_connections: Arc<AtomicUsize>,
    /// Semaphore for connection limiting
    connection_semaphore: Arc<Semaphore>,
    /// Connection ID generator
    connection_id_counter: Arc<AtomicU64>,
    /// Pool statistics
    stats: Arc<RwLock<PoolStats>>,
    /// Health monitor handle
    health_monitor: Arc<HealthMonitor>,
    /// Pool lifecycle state
    is_shutdown: Arc<AtomicUsize>, // 0 = running, 1 = shutting down, 2 = shutdown
}

/// Connection health monitoring system
pub struct HealthMonitor {
    /// Connections being monitored
    monitored_connections: Arc<RwLock<Vec<Weak<PooledConnection>>>>,
    /// Health check interval
    check_interval: Duration,
    /// Health statistics
    health_stats: Arc<RwLock<HealthStats>>,
}

/// Health monitoring statistics
#[derive(Debug, Clone)]
pub struct HealthStats {
    /// Total health checks performed
    pub total_checks: u64,
    /// Health checks that passed
    pub passed_checks: u64,
    /// Health checks that failed
    pub failed_checks: u64,
    /// Average health check duration
    pub avg_check_duration_ms: f64,
    /// Recently unhealthy connections
    pub recent_failures: VecDeque<HealthFailure>,
}

/// Health check failure record
#[derive(Debug, Clone)]
pub struct HealthFailure {
    pub connection_id: u64,
    pub timestamp: SystemTime,
    pub error_message: String,
    pub failure_type: HealthFailureType,
}

/// Types of health check failures
#[derive(Debug, Clone, PartialEq)]
pub enum HealthFailureType {
    /// Connection timeout
    Timeout,
    /// Network error
    NetworkError,
    /// DNS resolution failure
    DnsFailure,
    /// TLS handshake failure
    TlsFailure,
    /// HTTP error response
    HttpError(u16),
}

impl Default for ConnectionPoolConfig {
    fn default() -> Self {
        Self {
            min_connections: 5,
            max_connections: 50,
            connection_timeout: Duration::from_secs(30),
            request_timeout: Duration::from_secs(60),
            idle_timeout: Duration::from_secs(300), // 5 minutes
            health_check_interval: Duration::from_secs(30),
            max_connection_lifetime: Duration::from_secs(3600), // 1 hour
            enable_analytics: true,
            keep_alive_config: KeepAliveConfig {
                enabled: true,
                timeout: Duration::from_secs(60),
                interval: Duration::from_secs(10),
                probe_count: 3,
            },
        }
    }
}

impl Default for ConnectionConfig {
    fn default() -> Self {
        Self {
            user_agent: "EEYF/1.0".to_string(),
            headers: vec![
                ("Accept".to_string(), "application/json".to_string()),
                ("Accept-Encoding".to_string(), "gzip, deflate".to_string()),
            ],
            proxy: None,
            tls_config: TlsConfig {
                accept_invalid_certs: false,
                accept_invalid_hostnames: false,
                min_tls_version: TlsVersion::TlsV1_2,
            },
        }
    }
}

impl ConnectionPool {
    /// Create a new connection pool with the given configuration
    pub async fn new(config: ConnectionPoolConfig) -> Result<Self, YahooError> {
        let semaphore = Arc::new(Semaphore::new(config.max_connections));
        let health_monitor = Arc::new(HealthMonitor::new(config.health_check_interval));

        let pool = Self {
            connections: Arc::new(Mutex::new(VecDeque::new())),
            active_connections: Arc::new(AtomicUsize::new(0)),
            connection_semaphore: semaphore,
            connection_id_counter: Arc::new(AtomicU64::new(1)),
            stats: Arc::new(RwLock::new(PoolStats::default())),
            health_monitor,
            is_shutdown: Arc::new(AtomicUsize::new(0)),
            config,
        };

        // Pre-warm the pool with minimum connections
        pool.ensure_min_connections().await?;

        // Update statistics to reflect the initial connections
        pool.update_pool_statistics().await?;

        // Start health monitoring
        pool.start_health_monitoring().await;

        Ok(pool)
    }

    /// Get a connection from the pool
    pub async fn get_connection(&self) -> Result<Arc<PooledConnection>, YahooError> {
        if self.is_shutdown.load(Ordering::Relaxed) > 0 {
            return Err(YahooError::ConnectionFailed(
                "Pool is shutting down".to_string(),
            ));
        }

        // Try to get an existing connection
        if let Some(conn) = self.try_get_existing_connection().await? {
            return Ok(conn);
        }

        // Acquire semaphore permit for new connection
        let _permit = self.connection_semaphore.acquire().await.map_err(|_| {
            YahooError::ConnectionFailed("Failed to acquire connection permit".to_string())
        })?;

        // Create new connection if under max limit
        if self.active_connections.load(Ordering::Relaxed) < self.config.max_connections {
            let conn = self.create_connection().await?;
            self.active_connections.fetch_add(1, Ordering::Relaxed);
            return Ok(conn);
        }

        Err(YahooError::ConnectionFailed(
            "Connection pool exhausted".to_string(),
        ))
    }

    /// Return a connection to the pool
    pub async fn return_connection(
        &self,
        connection: Arc<PooledConnection>,
    ) -> Result<(), YahooError> {
        // Update connection usage statistics
        let health = connection.health.read().await;
        if *health == ConnectionHealth::Healthy {
            // Return healthy connections to pool
            drop(health);
            let mut connections = self.connections.lock().await;
            connections.push_back(connection);
            // Moving from active to idle
            self.active_connections.fetch_sub(1, Ordering::Relaxed);
        } else {
            // Don't return unhealthy connections - they're being destroyed
            self.active_connections.fetch_sub(1, Ordering::Relaxed);
        }

        Ok(())
    }

    /// Get pool statistics
    pub async fn get_stats(&self) -> PoolStats {
        self.stats.read().await.clone()
    }

    /// Perform pool maintenance (cleanup, optimization)
    pub async fn maintenance(&self) -> Result<(), YahooError> {
        // Remove idle connections that have exceeded idle timeout
        self.cleanup_idle_connections().await?;

        // Replace connections that have exceeded max lifetime
        self.replace_old_connections().await?;

        // Ensure minimum connection count
        self.ensure_min_connections().await?;

        // Update pool statistics
        self.update_pool_statistics().await?;

        Ok(())
    }

    /// Gracefully shutdown the connection pool
    pub async fn shutdown(&self) -> Result<(), YahooError> {
        // Set shutdown flag
        self.is_shutdown.store(1, Ordering::Relaxed);

        // Wait for active connections to finish
        let mut retries = 0;
        while self.active_connections.load(Ordering::Relaxed) > 0 && retries < 30 {
            sleep(Duration::from_secs(1)).await;
            retries += 1;
        }

        // Force close remaining connections
        let mut connections = self.connections.lock().await;
        connections.clear();

        // Mark as fully shutdown
        self.is_shutdown.store(2, Ordering::Relaxed);

        Ok(())
    }

    // Private helper methods
    async fn try_get_existing_connection(
        &self,
    ) -> Result<Option<Arc<PooledConnection>>, YahooError> {
        let mut connections = self.connections.lock().await;

        while let Some(conn) = connections.pop_front() {
            let health = conn.health.read().await.clone();
            if health == ConnectionHealth::Healthy {
                // Check if connection is still within lifetime limits
                if conn.created_at.elapsed() < self.config.max_connection_lifetime {
                    // Moving from idle to active
                    self.active_connections.fetch_add(1, Ordering::Relaxed);
                    return Ok(Some(conn));
                }
            }
            // Connection is unhealthy or too old, let it drop
            // (No need to decrement active_connections since it was idle)
        }

        Ok(None)
    }

    async fn create_connection(&self) -> Result<Arc<PooledConnection>, YahooError> {
        let connection_id = self.connection_id_counter.fetch_add(1, Ordering::Relaxed);
        let conn_config = ConnectionConfig::default();

        // Build HTTP client with connection-specific configuration
        let mut client_builder = ClientBuilder::new()
            .timeout(self.config.request_timeout)
            .connect_timeout(self.config.connection_timeout)
            .tcp_keepalive(self.config.keep_alive_config.timeout)
            .user_agent(&conn_config.user_agent);

        // Apply TLS configuration
        match conn_config.tls_config.min_tls_version {
            TlsVersion::TlsV1_2 => {
                client_builder = client_builder.min_tls_version(reqwest::tls::Version::TLS_1_2);
            }
            TlsVersion::TlsV1_3 => {
                client_builder = client_builder.min_tls_version(reqwest::tls::Version::TLS_1_3);
            }
            _ => {} // Use default
        }

        let client = client_builder
            .build()
            .map_err(|e| YahooError::ConnectionFailed(format!("Failed to create client: {}", e)))?;

        let connection = Arc::new(PooledConnection {
            client,
            id: connection_id,
            created_at: Instant::now(),
            last_used: Instant::now(),
            request_count: AtomicU64::new(0),
            health: Arc::new(RwLock::new(ConnectionHealth::Healthy)),
            config: conn_config,
        });

        // Register with health monitor
        self.health_monitor
            .register_connection(Arc::downgrade(&connection))
            .await;

        Ok(connection)
    }

    async fn cleanup_idle_connections(&self) -> Result<(), YahooError> {
        let mut connections = self.connections.lock().await;
        let now = Instant::now();

        // Remove connections that have been idle too long
        connections.retain(|conn| {
            let idle_time = now.duration_since(conn.last_used);
            if idle_time > self.config.idle_timeout {
                // Don't decrement active_connections - these are idle connections in the pool
                false
            } else {
                true
            }
        });

        Ok(())
    }

    async fn replace_old_connections(&self) -> Result<(), YahooError> {
        let mut connections = self.connections.lock().await;
        let now = Instant::now();

        // Replace connections that have exceeded max lifetime
        let old_connections: Vec<_> = connections
            .iter()
            .enumerate()
            .filter(|(_, conn)| {
                now.duration_since(conn.created_at) > self.config.max_connection_lifetime
            })
            .map(|(i, _)| i)
            .collect();

        // Remove old connections (they'll be replaced by ensure_min_connections)
        for &index in old_connections.iter().rev() {
            connections.remove(index);
            // Don't decrement active_connections - these are idle connections in the pool
        }

        Ok(())
    }

    async fn ensure_min_connections(&self) -> Result<(), YahooError> {
        let connections_count = self.connections.lock().await.len();
        let needed = self
            .config
            .min_connections
            .saturating_sub(connections_count);

        for _ in 0..needed {
            if let Ok(conn) = self.create_connection().await {
                self.connections.lock().await.push_back(conn);
                // Don't increment active_connections - these are idle connections in the pool
            }
        }

        Ok(())
    }

    async fn start_health_monitoring(&self) {
        let health_monitor = Arc::clone(&self.health_monitor);
        let pool_weak = Arc::downgrade(&self.stats); // Use stats as a proxy for pool lifetime

        tokio::spawn(async move {
            while let Some(_stats) = pool_weak.upgrade() {
                if let Err(e) = health_monitor.perform_health_checks().await {
                    eprintln!("Health check failed: {}", e);
                }
                sleep(health_monitor.check_interval).await;
            }
        });
    }

    async fn update_pool_statistics(&self) -> Result<(), YahooError> {
        let mut stats = self.stats.write().await;
        let connections_guard = self.connections.lock().await;

        stats.active_connections = self.active_connections.load(Ordering::Relaxed);
        stats.idle_connections = connections_guard.len();

        // Calculate health distribution
        let mut healthy = 0;
        let mut degraded = 0;
        let mut unhealthy = 0;
        let mut testing = 0;

        for conn in connections_guard.iter() {
            let health = futures::executor::block_on(conn.health.read());
            match *health {
                ConnectionHealth::Healthy => healthy += 1,
                ConnectionHealth::Degraded => degraded += 1,
                ConnectionHealth::Unhealthy => unhealthy += 1,
                ConnectionHealth::Testing => testing += 1,
            }
        }

        stats.health_distribution = HealthDistribution {
            healthy_count: healthy,
            degraded_count: degraded,
            unhealthy_count: unhealthy,
            testing_count: testing,
        };

        // Calculate pool efficiency
        let total_connections = stats.active_connections + stats.idle_connections;
        stats.pool_efficiency = if total_connections > 0 {
            stats.active_connections as f64 / total_connections as f64
        } else {
            0.0
        };

        Ok(())
    }
}

impl HealthMonitor {
    pub fn new(check_interval: Duration) -> Self {
        Self {
            monitored_connections: Arc::new(RwLock::new(Vec::new())),
            check_interval,
            health_stats: Arc::new(RwLock::new(HealthStats::default())),
        }
    }

    pub async fn register_connection(&self, connection: Weak<PooledConnection>) {
        let mut connections = self.monitored_connections.write().await;
        connections.push(connection);
    }

    pub async fn perform_health_checks(&self) -> Result<(), YahooError> {
        let connections = self.monitored_connections.read().await;
        let mut active_connections = Vec::new();

        for weak_conn in connections.iter() {
            if let Some(conn) = weak_conn.upgrade() {
                active_connections.push(conn);
            }
        }
        drop(connections);

        // Update monitored connections list to remove dead references
        let mut connections = self.monitored_connections.write().await;
        connections.retain(|weak| weak.strong_count() > 0);
        drop(connections);

        // Perform health checks on active connections
        for conn in active_connections {
            self.check_connection_health(conn).await?;
        }

        Ok(())
    }

    async fn check_connection_health(
        &self,
        connection: Arc<PooledConnection>,
    ) -> Result<(), YahooError> {
        let start_time = Instant::now();
        let mut health_stats = self.health_stats.write().await;
        health_stats.total_checks += 1;
        drop(health_stats);

        // Set connection to testing state
        {
            let mut health = connection.health.write().await;
            *health = ConnectionHealth::Testing;
        }

        // Perform actual health check (simplified - could make a lightweight request)
        let health_result = self.perform_lightweight_check(&connection).await;

        // Update connection health based on result
        let new_health = match health_result {
            Ok(true) => {
                let mut stats = self.health_stats.write().await;
                stats.passed_checks += 1;
                ConnectionHealth::Healthy
            }
            Ok(false) => {
                let mut stats = self.health_stats.write().await;
                stats.failed_checks += 1;
                ConnectionHealth::Degraded
            }
            Err(_) => {
                let mut stats = self.health_stats.write().await;
                stats.failed_checks += 1;
                ConnectionHealth::Unhealthy
            }
        };

        {
            let mut health = connection.health.write().await;
            *health = new_health;
        }

        // Update health check duration statistics
        let check_duration = start_time.elapsed();
        let mut health_stats = self.health_stats.write().await;
        health_stats.avg_check_duration_ms = (health_stats.avg_check_duration_ms
            * (health_stats.total_checks - 1) as f64
            + check_duration.as_secs_f64() * 1000.0)
            / health_stats.total_checks as f64;

        Ok(())
    }

    async fn perform_lightweight_check(
        &self,
        _connection: &PooledConnection,
    ) -> Result<bool, YahooError> {
        // In a real implementation, this might make a lightweight HTTP request
        // For now, simulate a health check
        Ok(true)
    }
}

impl Default for PoolStats {
    fn default() -> Self {
        Self {
            active_connections: 0,
            idle_connections: 0,
            total_requests: 0,
            failed_requests: 0,
            avg_request_duration_ms: 0.0,
            pool_efficiency: 0.0,
            health_distribution: HealthDistribution {
                healthy_count: 0,
                degraded_count: 0,
                unhealthy_count: 0,
                testing_count: 0,
            },
            utilization_history: VecDeque::new(),
        }
    }
}

impl Default for HealthStats {
    fn default() -> Self {
        Self {
            total_checks: 0,
            passed_checks: 0,
            failed_checks: 0,
            avg_check_duration_ms: 0.0,
            recent_failures: VecDeque::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_connection_pool_creation() {
        let config = ConnectionPoolConfig::default();
        let pool = ConnectionPool::new(config).await.unwrap();

        let stats = pool.get_stats().await;
        assert!(stats.idle_connections >= 5); // At least min_connections
    }

    #[tokio::test]
    async fn test_get_and_return_connection() {
        let config = ConnectionPoolConfig::default();
        let pool = ConnectionPool::new(config).await.unwrap();

        // Get a connection
        let conn = pool.get_connection().await.unwrap();
        assert_eq!(conn.request_count.load(Ordering::Relaxed), 0);

        // Return the connection
        pool.return_connection(conn).await.unwrap();

        let stats = pool.get_stats().await;
        assert!(stats.idle_connections > 0);
    }

    #[tokio::test]
    async fn test_pool_maintenance() {
        let mut config = ConnectionPoolConfig::default();
        config.idle_timeout = Duration::from_millis(100); // Very short for testing

        let pool = ConnectionPool::new(config).await.unwrap();

        // Wait for connections to become idle
        tokio::time::sleep(Duration::from_millis(150)).await;

        // Perform maintenance (should clean up idle connections and create new ones)
        pool.maintenance().await.unwrap();

        // Should still have minimum connections
        let stats = pool.get_stats().await;
        assert!(stats.idle_connections >= 5);
    }

    #[tokio::test]
    async fn test_pool_shutdown() {
        let config = ConnectionPoolConfig::default();
        let pool = ConnectionPool::new(config).await.unwrap();

        // Get a connection to make pool active
        let _conn = pool.get_connection().await.unwrap();

        // Shutdown should succeed
        let result = pool.shutdown().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_health_monitor() {
        let monitor = HealthMonitor::new(Duration::from_millis(100));

        // Create a mock connection
        let config = ConnectionConfig::default();
        let client = ClientBuilder::new().build().unwrap();
        let conn = Arc::new(PooledConnection {
            client,
            id: 1,
            created_at: Instant::now(),
            last_used: Instant::now(),
            request_count: AtomicU64::new(0),
            health: Arc::new(RwLock::new(ConnectionHealth::Healthy)),
            config,
        });

        // Register and check
        monitor.register_connection(Arc::downgrade(&conn)).await;
        let result = monitor.perform_health_checks().await;
        assert!(result.is_ok());
    }
}
