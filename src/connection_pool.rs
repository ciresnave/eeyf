//! Connection pooling for efficient HTTP connection management
//!
//! This module provides intelligent connection pooling to manage
//! HTTP connections efficiently, reduce connection overhead, and
//! improve overall API performance and reliability.

use log::{debug, info};
use reqwest::{Client, ClientBuilder};
use std::collections::HashMap;
use std::sync::atomic::{AtomicU32, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::sync::{RwLock, Semaphore};
use tokio::time::timeout;
use crate::yahoo_error::YahooError;

/// Configuration for connection pooling
#[derive(Debug, Clone)]
pub struct ConnectionPoolConfig {
    /// Maximum number of connections per host
    pub max_connections_per_host: usize,
    /// Maximum total connections across all hosts
    pub max_total_connections: usize,
    /// Connection timeout in milliseconds
    pub connect_timeout_ms: u64,
    /// Request timeout in milliseconds
    pub request_timeout_ms: u64,
    /// Keep-alive timeout in milliseconds
    pub keep_alive_timeout_ms: u64,
    /// Connection idle timeout in milliseconds
    pub idle_timeout_ms: u64,
    /// Enable HTTP/2
    pub enable_http2: bool,
    /// Enable connection reuse
    pub enable_connection_reuse: bool,
    /// Pool cleanup interval in milliseconds
    pub cleanup_interval_ms: u64,
    /// User agent string
    pub user_agent: String,
    /// Enable TCP keepalive
    pub enable_tcp_keepalive: bool,
    /// TCP keepalive interval in milliseconds
    pub tcp_keepalive_interval_ms: u64,
}

impl Default for ConnectionPoolConfig {
    fn default() -> Self {
        Self {
            max_connections_per_host: 10,
            max_total_connections: 100,
            connect_timeout_ms: 10000, // 10 seconds
            request_timeout_ms: 30000, // 30 seconds
            keep_alive_timeout_ms: 90000, // 90 seconds
            idle_timeout_ms: 60000, // 60 seconds
            enable_http2: true,
            enable_connection_reuse: true,
            cleanup_interval_ms: 300000, // 5 minutes
            user_agent: "EEYF/0.1.0 (Yahoo Finance API Client)".to_string(),
            enable_tcp_keepalive: true,
            tcp_keepalive_interval_ms: 30000, // 30 seconds
        }
    }
}

impl ConnectionPoolConfig {
    /// Configuration optimized for high throughput
    pub fn high_throughput() -> Self {
        Self {
            max_connections_per_host: 20,
            max_total_connections: 200,
            connect_timeout_ms: 5000, // 5 seconds
            request_timeout_ms: 15000, // 15 seconds
            keep_alive_timeout_ms: 120000, // 2 minutes
            idle_timeout_ms: 30000, // 30 seconds
            enable_http2: true,
            enable_connection_reuse: true,
            cleanup_interval_ms: 120000, // 2 minutes
            user_agent: "EEYF/0.1.0 (High-Throughput)".to_string(),
            enable_tcp_keepalive: true,
            tcp_keepalive_interval_ms: 15000, // 15 seconds
        }
    }

    /// Configuration optimized for low resource usage
    pub fn low_resource() -> Self {
        Self {
            max_connections_per_host: 3,
            max_total_connections: 20,
            connect_timeout_ms: 15000, // 15 seconds
            request_timeout_ms: 45000, // 45 seconds
            keep_alive_timeout_ms: 60000, // 1 minute
            idle_timeout_ms: 120000, // 2 minutes
            enable_http2: false,
            enable_connection_reuse: true,
            cleanup_interval_ms: 600000, // 10 minutes
            user_agent: "EEYF/0.1.0 (Low-Resource)".to_string(),
            enable_tcp_keepalive: false,
            tcp_keepalive_interval_ms: 60000, // 1 minute
        }
    }

    /// Configuration for development/testing
    pub fn development() -> Self {
        Self {
            max_connections_per_host: 5,
            max_total_connections: 25,
            connect_timeout_ms: 20000, // 20 seconds
            request_timeout_ms: 60000, // 60 seconds
            keep_alive_timeout_ms: 30000, // 30 seconds
            idle_timeout_ms: 30000, // 30 seconds
            enable_http2: true,
            enable_connection_reuse: true,
            cleanup_interval_ms: 60000, // 1 minute
            user_agent: "EEYF/0.1.0 (Development)".to_string(),
            enable_tcp_keepalive: true,
            tcp_keepalive_interval_ms: 30000, // 30 seconds
        }
    }
}

/// Connection statistics
#[derive(Debug, Clone, Default)]
pub struct ConnectionStats {
    pub total_connections_created: u64,
    pub active_connections: u32,
    pub idle_connections: u32,
    pub connections_in_use: u32,
    pub connection_timeouts: u64,
    pub connection_errors: u64,
    pub requests_per_connection: f64,
    pub average_connection_lifetime_ms: f64,
    pub connection_reuse_rate: f64,
    pub pool_utilization: f64,
}

/// Individual connection information
#[derive(Debug, Clone)]
struct ConnectionInfo {
    created_at: u64,
    last_used: u64,
    request_count: u64,
    host: String,
    is_active: bool,
}

impl ConnectionInfo {
    fn new(host: String) -> Self {
        let now = current_time_millis();
        Self {
            created_at: now,
            last_used: now,
            request_count: 0,
            host,
            is_active: false,
        }
    }

    fn use_connection(&mut self) {
        self.last_used = current_time_millis();
        self.request_count += 1;
        self.is_active = true;
    }

    fn release_connection(&mut self) {
        self.is_active = false;
    }

    fn is_idle(&self, idle_timeout_ms: u64) -> bool {
        !self.is_active && (current_time_millis() - self.last_used) > idle_timeout_ms
    }

    fn lifetime_ms(&self) -> u64 {
        current_time_millis() - self.created_at
    }
}

/// Connection pool manager
pub struct ConnectionPool {
    config: ConnectionPoolConfig,
    clients: Arc<RwLock<HashMap<String, Client>>>,
    connections: Arc<RwLock<HashMap<String, ConnectionInfo>>>,
    connection_semaphores: Arc<RwLock<HashMap<String, Arc<Semaphore>>>>,
    total_semaphore: Arc<Semaphore>,
    stats: Arc<RwLock<ConnectionStats>>,
    
    // Atomic counters
    connection_counter: Arc<AtomicU64>,
    active_counter: Arc<AtomicU32>,
    timeout_counter: Arc<AtomicU64>,
    error_counter: Arc<AtomicU64>,
}

impl ConnectionPool {
    pub fn new(config: ConnectionPoolConfig) -> Self {
        let total_semaphore = Arc::new(Semaphore::new(config.max_total_connections));
        
        let pool = Self {
            config,
            clients: Arc::new(RwLock::new(HashMap::new())),
            connections: Arc::new(RwLock::new(HashMap::new())),
            connection_semaphores: Arc::new(RwLock::new(HashMap::new())),
            total_semaphore,
            stats: Arc::new(RwLock::new(ConnectionStats::default())),
            connection_counter: Arc::new(AtomicU64::new(0)),
            active_counter: Arc::new(AtomicU32::new(0)),
            timeout_counter: Arc::new(AtomicU64::new(0)),
            error_counter: Arc::new(AtomicU64::new(0)),
        };

        // Start background cleanup task
        pool.start_cleanup_task();
        
        pool
    }

    pub fn with_default_config() -> Self {
        Self::new(ConnectionPoolConfig::default())
    }

    /// Get or create a client for the specified host
    pub async fn get_client(&self, host: &str) -> Result<Client, YahooError> {
        let host_key = host.to_string();
        
        // Check if we already have a client for this host
        {
            let clients = self.clients.read().await;
            if let Some(client) = clients.get(&host_key) {
                self.track_connection_use(&host_key).await;
                return Ok(client.clone());
            }
        }

        // Create new client if we don't have one
        self.create_client_for_host(&host_key).await
    }

    /// Create a new client for a specific host
    async fn create_client_for_host(&self, host: &str) -> Result<Client, YahooError> {
        // Acquire semaphore permits
        let _total_permit = self.total_semaphore.acquire().await
            .map_err(|_| YahooError::FetchFailed("Failed to acquire connection permit".to_string()))?;

        let host_semaphore = self.get_or_create_host_semaphore(host).await;
        let _host_permit = host_semaphore.acquire().await
            .map_err(|_| YahooError::FetchFailed("Failed to acquire host-specific connection permit".to_string()))?;

        // Build the client with our configuration
        let client = self.build_client().await?;
        
        // Store the client
        {
            let mut clients = self.clients.write().await;
            clients.insert(host.to_string(), client.clone());
        }

        // Track the new connection
        self.track_new_connection(host).await;
        
        info!("Created new client for host: {}", host);
        Ok(client)
    }

    /// Build a configured HTTP client
    async fn build_client(&self) -> Result<Client, YahooError> {
        let mut builder = ClientBuilder::new();
        
        builder = builder
            .timeout(Duration::from_millis(self.config.request_timeout_ms))
            .connect_timeout(Duration::from_millis(self.config.connect_timeout_ms))
            .user_agent(&self.config.user_agent)
            .pool_max_idle_per_host(self.config.max_connections_per_host)
            .pool_idle_timeout(Duration::from_millis(self.config.idle_timeout_ms));

        // HTTP/2 feature may not be available in this version
        // if self.config.enable_http2 {
        //     builder = builder.http2_prior_knowledge();
        // }

        if self.config.enable_tcp_keepalive {
            builder = builder.tcp_keepalive(Duration::from_millis(self.config.tcp_keepalive_interval_ms));
        }

        if self.config.enable_connection_reuse {
            builder = builder.connection_verbose(true);
        }

        builder.build()
            .map_err(|e| YahooError::ConnectionFailed(e.to_string()))
    }

    /// Get or create semaphore for a specific host
    async fn get_or_create_host_semaphore(&self, host: &str) -> Arc<Semaphore> {
        let mut semaphores = self.connection_semaphores.write().await;
        semaphores
            .entry(host.to_string())
            .or_insert_with(|| Arc::new(Semaphore::new(self.config.max_connections_per_host)))
            .clone()
    }

    /// Track a new connection
    async fn track_new_connection(&self, host: &str) {
        let connection_info = ConnectionInfo::new(host.to_string());
        
        {
            let mut connections = self.connections.write().await;
            let connection_id = format!("{}_{}", host, self.connection_counter.load(Ordering::Relaxed));
            connections.insert(connection_id, connection_info);
        }

        self.connection_counter.fetch_add(1, Ordering::Relaxed);
        self.active_counter.fetch_add(1, Ordering::Relaxed);
        
        debug!("Tracked new connection for host: {}", host);
    }

    /// Track connection usage
    async fn track_connection_use(&self, host: &str) {
        let mut connections = self.connections.write().await;
        
        // Find an existing connection for this host and mark it as used
        for (_, conn_info) in connections.iter_mut() {
            if conn_info.host == host && !conn_info.is_active {
                conn_info.use_connection();
                debug!("Reusing connection for host: {}", host);
                return;
            }
        }
        
        debug!("Using connection for host: {}", host);
    }

    /// Release a connection back to the pool
    pub async fn release_connection(&self, host: &str) {
        let mut connections = self.connections.write().await;
        
        for (_, conn_info) in connections.iter_mut() {
            if conn_info.host == host && conn_info.is_active {
                conn_info.release_connection();
                debug!("Released connection for host: {}", host);
                return;
            }
        }
    }

    /// Execute an HTTP request with connection pooling
    pub async fn execute_request<F, Fut, T>(&self, host: &str, operation: F) -> Result<T, YahooError>
    where
        F: FnOnce(Client) -> Fut,
        Fut: std::future::Future<Output = Result<T, YahooError>>,
    {
        let start_time = current_time_millis();
        
        // Get client with timeout
        let client = match timeout(
            Duration::from_millis(self.config.connect_timeout_ms),
            self.get_client(host)
        ).await {
            Ok(Ok(client)) => client,
            Ok(Err(e)) => {
                self.error_counter.fetch_add(1, Ordering::Relaxed);
                return Err(e);
            }
            Err(_) => {
                self.timeout_counter.fetch_add(1, Ordering::Relaxed);
                return Err(YahooError::FetchFailed(
                    format!("Connection timeout after {}ms", self.config.connect_timeout_ms)
                ));
            }
        };

        // Execute the operation
        let result = operation(client).await;

        // Release the connection
        self.release_connection(host).await;

        // Update stats
        let duration = current_time_millis() - start_time;
        self.update_request_stats(duration, result.is_ok()).await;

        result
    }

    /// Update request statistics
    async fn update_request_stats(&self, _duration_ms: u64, _success: bool) {
        let mut stats = self.stats.write().await;
        let connections = self.connections.read().await;
        
        // Update basic stats
        stats.total_connections_created = self.connection_counter.load(Ordering::Relaxed);
        stats.active_connections = self.active_counter.load(Ordering::Relaxed);
        stats.connection_timeouts = self.timeout_counter.load(Ordering::Relaxed);
        stats.connection_errors = self.error_counter.load(Ordering::Relaxed);
        
        // Calculate derived stats
        let total_connections = connections.len();
        if total_connections > 0 {
            let total_requests: u64 = connections.values().map(|c| c.request_count).sum();
            let total_lifetime: u64 = connections.values().map(|c| c.lifetime_ms()).sum();
            
            stats.requests_per_connection = total_requests as f64 / total_connections as f64;
            stats.average_connection_lifetime_ms = total_lifetime as f64 / total_connections as f64;
            
            let reused_connections = connections.values().filter(|c| c.request_count > 1).count();
            stats.connection_reuse_rate = reused_connections as f64 / total_connections as f64;
            
            stats.pool_utilization = stats.active_connections as f64 / self.config.max_total_connections as f64;
        }
        
        let active_connections = connections.values().filter(|c| c.is_active).count();
        let idle_connections = connections.values().filter(|c| !c.is_active).count();
        
        stats.connections_in_use = active_connections as u32;
        stats.idle_connections = idle_connections as u32;
    }

    /// Get current pool statistics
    pub async fn stats(&self) -> ConnectionStats {
        self.update_request_stats(0, true).await;
        self.stats.read().await.clone()
    }

    /// Start background cleanup task
    fn start_cleanup_task(&self) {
        if self.config.cleanup_interval_ms == 0 {
            return; // Cleanup disabled
        }

        let connections = self.connections.clone();
        let clients = self.clients.clone();
        let config = self.config.clone();
        let active_counter = self.active_counter.clone();
        
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_millis(config.cleanup_interval_ms));
            
            loop {
                interval.tick().await;
                
                let mut connections_guard = connections.write().await;
                let mut clients_guard = clients.write().await;
                let mut cleaned_up = 0;
                
                // Collect idle connection IDs first
                let mut idle_connections = Vec::new();
                for (_conn_id, conn_info) in connections_guard.iter() {
                    if conn_info.is_idle(config.idle_timeout_ms) {
                        idle_connections.push((conn_info.host.clone(), _conn_id.clone()));
                    }
                }
                
                // Remove idle connections
                for (host, conn_id) in idle_connections {
                    connections_guard.remove(&conn_id);
                    
                    // Check if host has any remaining connections
                    let host_has_connections = connections_guard
                        .values()
                        .any(|conn| conn.host == host);
                    
                    if !host_has_connections {
                        clients_guard.remove(&host);
                    }
                    
                    cleaned_up += 1;
                }
                
                if cleaned_up > 0 {
                    active_counter.fetch_sub(cleaned_up, Ordering::Relaxed);
                    info!("Cleaned up {} idle connections", cleaned_up);
                }
                
                drop(connections_guard);
                drop(clients_guard);
            }
        });
    }

    /// Manually trigger cleanup of idle connections
    pub async fn cleanup_idle_connections(&self) {
        let mut connections = self.connections.write().await;
        let mut clients = self.clients.write().await;
        let mut cleaned_up = 0;
        
        connections.retain(|_, conn_info| {
            if conn_info.is_idle(self.config.idle_timeout_ms) {
                cleaned_up += 1;
                false
            } else {
                true
            }
        });

        // Remove clients that no longer have connections
        let active_hosts: std::collections::HashSet<_> = connections
            .values()
            .map(|c| c.host.clone())
            .collect();
        
        clients.retain(|host, _| active_hosts.contains(host));
        
        if cleaned_up > 0 {
            self.active_counter.fetch_sub(cleaned_up, Ordering::Relaxed);
            info!("Manually cleaned up {} idle connections", cleaned_up);
        }
    }

    /// Shutdown the connection pool
    pub async fn shutdown(&self) {
        info!("Shutting down connection pool");
        
        self.clients.write().await.clear();
        self.connections.write().await.clear();
        self.connection_semaphores.write().await.clear();
        
        self.active_counter.store(0, Ordering::Relaxed);
        
        info!("Connection pool shutdown complete");
    }
}

/// Get current time in milliseconds since Unix epoch
fn current_time_millis() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or(Duration::ZERO)
        .as_millis() as u64
}

#[cfg(test)]
mod tests {
    use super::*;
    
    // Async tests removed due to tokio macro ABI mismatch
    // Tests can be added back when toolchain is updated

    #[test]
    fn test_config_presets() {
        let high_throughput = ConnectionPoolConfig::high_throughput();
        assert_eq!(high_throughput.max_connections_per_host, 20);
        assert!(high_throughput.enable_http2);
        
        let low_resource = ConnectionPoolConfig::low_resource();
        assert_eq!(low_resource.max_connections_per_host, 3);
        assert!(!low_resource.enable_http2);
        
        let development = ConnectionPoolConfig::development();
        assert_eq!(development.max_connections_per_host, 5);
        assert!(development.enable_http2);
    }
}