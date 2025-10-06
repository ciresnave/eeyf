//! Resource limits and backpressure handling
//!
//! This module provides configurable resource limits to prevent resource
//! exhaustion and maintain system stability.

use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{RwLock, Semaphore};

/// Resource limit errors
#[derive(Debug, thiserror::Error)]
pub enum LimitError {
    /// Concurrent request limit exceeded
    #[error("Concurrent request limit exceeded: {current}/{max}")]
    ConcurrentRequests { current: usize, max: usize },
    
    /// Memory limit exceeded
    #[error("Memory limit exceeded: {current_mb}MB/{max_mb}MB")]
    MemoryLimit { current_mb: usize, max_mb: usize },
    
    /// Cache size limit exceeded
    #[error("Cache size limit exceeded: {current_mb}MB/{max_mb}MB")]
    CacheSize { current_mb: usize, max_mb: usize },
    
    /// Queue size limit exceeded
    #[error("Queue size limit exceeded: {current}/{max}")]
    QueueSize { current: usize, max: usize },
    
    /// Acquisition timeout
    #[error("Failed to acquire resource within timeout")]
    Timeout,
}

/// Resource limits configuration
#[derive(Debug, Clone)]
pub struct ResourceLimits {
    /// Maximum concurrent requests
    pub max_concurrent_requests: usize,
    
    /// Maximum memory usage in MB
    pub max_memory_mb: usize,
    
    /// Maximum cache size in MB
    pub max_cache_size_mb: usize,
    
    /// Maximum queue size
    pub max_queue_size: usize,
    
    /// Connection pool size
    pub connection_pool_size: usize,
    
    /// Request timeout
    pub request_timeout: Duration,
    
    /// Enable backpressure
    pub enable_backpressure: bool,
}

impl Default for ResourceLimits {
    fn default() -> Self {
        Self {
            max_concurrent_requests: 100,
            max_memory_mb: 512,
            max_cache_size_mb: 128,
            max_queue_size: 1000,
            connection_pool_size: 10,
            request_timeout: Duration::from_secs(30),
            enable_backpressure: true,
        }
    }
}

impl ResourceLimits {
    /// Create a new resource limits config
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Set maximum concurrent requests
    pub fn with_max_concurrent_requests(mut self, max: usize) -> Self {
        self.max_concurrent_requests = max;
        self
    }
    
    /// Set maximum memory usage
    pub fn with_max_memory_mb(mut self, mb: usize) -> Self {
        self.max_memory_mb = mb;
        self
    }
    
    /// Set maximum cache size
    pub fn with_max_cache_size_mb(mut self, mb: usize) -> Self {
        self.max_cache_size_mb = mb;
        self
    }
    
    /// Set maximum queue size
    pub fn with_max_queue_size(mut self, size: usize) -> Self {
        self.max_queue_size = size;
        self
    }
    
    /// Set connection pool size
    pub fn with_connection_pool_size(mut self, size: usize) -> Self {
        self.connection_pool_size = size;
        self
    }
    
    /// Set request timeout
    pub fn with_request_timeout(mut self, timeout: Duration) -> Self {
        self.request_timeout = timeout;
        self
    }
    
    /// Enable or disable backpressure
    pub fn with_backpressure(mut self, enabled: bool) -> Self {
        self.enable_backpressure = enabled;
        self
    }
}

/// Resource limiter for enforcing limits
pub struct ResourceLimiter {
    /// Configuration
    config: ResourceLimits,
    
    /// Concurrent request semaphore
    request_semaphore: Arc<Semaphore>,
    
    /// Current memory usage in bytes
    memory_usage: Arc<RwLock<usize>>,
    
    /// Current cache size in bytes
    cache_size: Arc<RwLock<usize>>,
    
    /// Current queue size
    queue_size: Arc<RwLock<usize>>,
}

impl ResourceLimiter {
    /// Create a new resource limiter
    pub fn new(config: ResourceLimits) -> Self {
        Self {
            request_semaphore: Arc::new(Semaphore::new(config.max_concurrent_requests)),
            memory_usage: Arc::new(RwLock::new(0)),
            cache_size: Arc::new(RwLock::new(0)),
            queue_size: Arc::new(RwLock::new(0)),
            config,
        }
    }
    
    /// Acquire a request permit
    pub async fn acquire_request_permit(&self) -> Result<RequestPermit, LimitError> {
        let permit = tokio::time::timeout(
            self.config.request_timeout,
            self.request_semaphore.clone().acquire_owned(),
        )
        .await
        .map_err(|_| LimitError::Timeout)?
        .map_err(|_| LimitError::ConcurrentRequests {
            current: self.config.max_concurrent_requests,
            max: self.config.max_concurrent_requests,
        })?;
        
        Ok(RequestPermit { permit })
    }
    
    /// Check if memory limit would be exceeded
    pub async fn check_memory_limit(&self, additional_bytes: usize) -> Result<(), LimitError> {
        let current = *self.memory_usage.read().await;
        let new_total = current + additional_bytes;
        let max_bytes = self.config.max_memory_mb * 1024 * 1024;
        
        if new_total > max_bytes {
            return Err(LimitError::MemoryLimit {
                current_mb: new_total / (1024 * 1024),
                max_mb: self.config.max_memory_mb,
            });
        }
        
        Ok(())
    }
    
    /// Add memory usage
    pub async fn add_memory_usage(&self, bytes: usize) {
        let mut usage = self.memory_usage.write().await;
        *usage += bytes;
    }
    
    /// Remove memory usage
    pub async fn remove_memory_usage(&self, bytes: usize) {
        let mut usage = self.memory_usage.write().await;
        if *usage >= bytes {
            *usage -= bytes;
        } else {
            *usage = 0;
        }
    }
    
    /// Check if cache size limit would be exceeded
    pub async fn check_cache_limit(&self, additional_bytes: usize) -> Result<(), LimitError> {
        let current = *self.cache_size.read().await;
        let new_total = current + additional_bytes;
        let max_bytes = self.config.max_cache_size_mb * 1024 * 1024;
        
        if new_total > max_bytes {
            return Err(LimitError::CacheSize {
                current_mb: new_total / (1024 * 1024),
                max_mb: self.config.max_cache_size_mb,
            });
        }
        
        Ok(())
    }
    
    /// Add cache size
    pub async fn add_cache_size(&self, bytes: usize) {
        let mut size = self.cache_size.write().await;
        *size += bytes;
    }
    
    /// Remove cache size
    pub async fn remove_cache_size(&self, bytes: usize) {
        let mut size = self.cache_size.write().await;
        if *size >= bytes {
            *size -= bytes;
        } else {
            *size = 0;
        }
    }
    
    /// Check if queue size limit would be exceeded
    pub async fn check_queue_limit(&self) -> Result<(), LimitError> {
        let current = *self.queue_size.read().await;
        
        if current >= self.config.max_queue_size {
            return Err(LimitError::QueueSize {
                current,
                max: self.config.max_queue_size,
            });
        }
        
        Ok(())
    }
    
    /// Increment queue size
    pub async fn increment_queue_size(&self) {
        let mut size = self.queue_size.write().await;
        *size += 1;
    }
    
    /// Decrement queue size
    pub async fn decrement_queue_size(&self) {
        let mut size = self.queue_size.write().await;
        if *size > 0 {
            *size -= 1;
        }
    }
    
    /// Get current memory usage in bytes
    pub async fn memory_usage(&self) -> usize {
        *self.memory_usage.read().await
    }
    
    /// Get current cache size in bytes
    pub async fn cache_size(&self) -> usize {
        *self.cache_size.read().await
    }
    
    /// Get current queue size
    pub async fn queue_size(&self) -> usize {
        *self.queue_size.read().await
    }
    
    /// Get available request permits
    pub fn available_permits(&self) -> usize {
        self.request_semaphore.available_permits()
    }
}

/// RAII guard for request permit
pub struct RequestPermit {
    permit: tokio::sync::OwnedSemaphorePermit,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_resource_limits_default() {
        let limits = ResourceLimits::default();
        assert_eq!(limits.max_concurrent_requests, 100);
        assert_eq!(limits.max_memory_mb, 512);
        assert_eq!(limits.max_cache_size_mb, 128);
        assert_eq!(limits.max_queue_size, 1000);
        assert!(limits.enable_backpressure);
    }
    
    #[test]
    fn test_resource_limits_builder() {
        let limits = ResourceLimits::new()
            .with_max_concurrent_requests(50)
            .with_max_memory_mb(256)
            .with_max_cache_size_mb(64)
            .with_max_queue_size(500)
            .with_connection_pool_size(20)
            .with_request_timeout(Duration::from_secs(60))
            .with_backpressure(false);
        
        assert_eq!(limits.max_concurrent_requests, 50);
        assert_eq!(limits.max_memory_mb, 256);
        assert_eq!(limits.max_cache_size_mb, 64);
        assert_eq!(limits.max_queue_size, 500);
        assert_eq!(limits.connection_pool_size, 20);
        assert_eq!(limits.request_timeout, Duration::from_secs(60));
        assert!(!limits.enable_backpressure);
    }
    
    #[tokio::test]
    async fn test_request_permit_acquisition() {
        let limits = ResourceLimits::new().with_max_concurrent_requests(2);
        let limiter = ResourceLimiter::new(limits);
        
        assert_eq!(limiter.available_permits(), 2);
        
        let permit1 = limiter.acquire_request_permit().await.unwrap();
        assert_eq!(limiter.available_permits(), 1);
        
        let permit2 = limiter.acquire_request_permit().await.unwrap();
        assert_eq!(limiter.available_permits(), 0);
        
        drop(permit1);
        assert_eq!(limiter.available_permits(), 1);
        
        drop(permit2);
        assert_eq!(limiter.available_permits(), 2);
    }
    
    #[tokio::test]
    async fn test_memory_limit_enforcement() {
        let limits = ResourceLimits::new().with_max_memory_mb(1); // 1 MB = 1048576 bytes
        let limiter = ResourceLimiter::new(limits);
        
        // Should succeed (512 KB)
        let result = limiter.check_memory_limit(512 * 1024).await;
        assert!(result.is_ok());
        
        limiter.add_memory_usage(512 * 1024).await;
        
        // Should still succeed (total would be 768 KB, under 1 MB)
        let result = limiter.check_memory_limit(256 * 1024).await;
        assert!(result.is_ok());
        
        // Should fail (total would be 512 KB + 600 KB = 1112 KB, over 1 MB)
        let result = limiter.check_memory_limit(600 * 1024).await;
        assert!(result.is_err());
    }
    
    #[tokio::test]
    async fn test_cache_limit_enforcement() {
        let limits = ResourceLimits::new().with_max_cache_size_mb(1); // 1 MB
        let limiter = ResourceLimiter::new(limits);
        
        // Should succeed
        let result = limiter.check_cache_limit(512 * 1024).await;
        assert!(result.is_ok());
        
        limiter.add_cache_size(512 * 1024).await;
        assert_eq!(limiter.cache_size().await, 512 * 1024);
        
        // Should fail
        let result = limiter.check_cache_limit(600 * 1024).await;
        assert!(result.is_err());
        
        // Remove some cache
        limiter.remove_cache_size(256 * 1024).await;
        assert_eq!(limiter.cache_size().await, 256 * 1024);
        
        // Should succeed now
        let result = limiter.check_cache_limit(512 * 1024).await;
        assert!(result.is_ok());
    }
    
    #[tokio::test]
    async fn test_queue_limit_enforcement() {
        let limits = ResourceLimits::new().with_max_queue_size(2);
        let limiter = ResourceLimiter::new(limits);
        
        // Should succeed
        assert!(limiter.check_queue_limit().await.is_ok());
        limiter.increment_queue_size().await;
        
        assert!(limiter.check_queue_limit().await.is_ok());
        limiter.increment_queue_size().await;
        
        // Should fail
        assert!(limiter.check_queue_limit().await.is_err());
        
        // Decrement and try again
        limiter.decrement_queue_size().await;
        assert!(limiter.check_queue_limit().await.is_ok());
    }
}
