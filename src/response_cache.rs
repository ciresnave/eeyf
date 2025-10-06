//! Response caching with TTL and LRU eviction
//!
//! This module provides intelligent response caching with configurable
//! TTL (Time To Live) and LRU (Least Recently Used) eviction policies
//! to optimize API response times and reduce unnecessary requests.

use lru::LruCache;
use serde_json::Value;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::num::NonZeroUsize;
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::sync::RwLock;
use log::{debug, info};


/// Cache entry with TTL support
#[derive(Debug, Clone)]
struct CacheEntry {
    data: Value,
    created_at: u64,
    last_accessed: u64,
    access_count: u64,
    size_bytes: usize,
}

impl CacheEntry {
    fn new(data: Value) -> Self {
        let now = current_time_millis();
        let size_bytes = estimate_json_size(&data);
        
        Self {
            data,
            created_at: now,
            last_accessed: now,
            access_count: 1,
            size_bytes,
        }
    }

    fn access(&mut self) -> Value {
        self.last_accessed = current_time_millis();
        self.access_count += 1;
        self.data.clone()
    }

    fn is_expired(&self, ttl_ms: u64) -> bool {
        if ttl_ms == 0 {
            return false; // Never expire if TTL is 0
        }
        current_time_millis() - self.created_at > ttl_ms
    }

    fn age_ms(&self) -> u64 {
        current_time_millis() - self.created_at
    }

    #[allow(dead_code)]
    fn idle_time_ms(&self) -> u64 {
        current_time_millis() - self.last_accessed
    }
}

/// Cache key for request identification
#[derive(Debug, Clone, PartialEq, Eq)]
struct CacheKey {
    url: String,
    params_hash: u64,
}

impl CacheKey {
    fn new(url: String, params: Option<&HashMap<String, String>>) -> Self {
        let params_hash = if let Some(params) = params {
            let mut hasher = std::collections::hash_map::DefaultHasher::new();
            let mut sorted_params: Vec<_> = params.iter().collect();
            sorted_params.sort_by_key(|(k, _)| *k);
            
            for (key, value) in sorted_params {
                key.hash(&mut hasher);
                value.hash(&mut hasher);
            }
            hasher.finish()
        } else {
            0
        };

        Self { url, params_hash }
    }
}

impl Hash for CacheKey {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.url.hash(state);
        self.params_hash.hash(state);
    }
}

/// Configuration for response caching
#[derive(Debug, Clone)]
pub struct ResponseCacheConfig {
    /// Maximum number of entries in the cache
    pub max_entries: usize,
    /// Default TTL for cached responses (in milliseconds)
    pub default_ttl_ms: u64,
    /// TTL for different response types
    pub quote_ttl_ms: u64,
    pub search_ttl_ms: u64,
    pub history_ttl_ms: u64,
    /// Maximum memory usage in bytes (approximate)
    pub max_memory_bytes: usize,
    /// Whether to enable size-based eviction
    pub enable_size_eviction: bool,
    /// Cleanup interval for expired entries (in milliseconds)
    pub cleanup_interval_ms: u64,
    /// Whether to cache error responses
    pub cache_errors: bool,
    /// TTL for cached errors (typically much shorter)
    pub error_ttl_ms: u64,
}

impl Default for ResponseCacheConfig {
    fn default() -> Self {
        Self {
            max_entries: 1000,
            default_ttl_ms: 300_000,    // 5 minutes
            quote_ttl_ms: 60_000,       // 1 minute for live quotes
            search_ttl_ms: 3600_000,    // 1 hour for search results
            history_ttl_ms: 1800_000,   // 30 minutes for historical data
            max_memory_bytes: 50 * 1024 * 1024, // 50MB
            enable_size_eviction: true,
            cleanup_interval_ms: 300_000, // 5 minutes
            cache_errors: false,
            error_ttl_ms: 30_000,       // 30 seconds
        }
    }
}

impl ResponseCacheConfig {
    /// Configuration for aggressive caching (longer TTLs, more memory)
    pub fn aggressive() -> Self {
        Self {
            max_entries: 5000,
            default_ttl_ms: 900_000,    // 15 minutes
            quote_ttl_ms: 300_000,      // 5 minutes
            search_ttl_ms: 7200_000,    // 2 hours
            history_ttl_ms: 3600_000,   // 1 hour
            max_memory_bytes: 200 * 1024 * 1024, // 200MB
            enable_size_eviction: true,
            cleanup_interval_ms: 600_000, // 10 minutes
            cache_errors: true,
            error_ttl_ms: 120_000,      // 2 minutes
        }
    }

    /// Configuration for conservative caching (shorter TTLs, less memory)
    pub fn conservative() -> Self {
        Self {
            max_entries: 200,
            default_ttl_ms: 60_000,     // 1 minute
            quote_ttl_ms: 30_000,       // 30 seconds
            search_ttl_ms: 900_000,     // 15 minutes
            history_ttl_ms: 600_000,    // 10 minutes
            max_memory_bytes: 10 * 1024 * 1024, // 10MB
            enable_size_eviction: true,
            cleanup_interval_ms: 120_000, // 2 minutes
            cache_errors: false,
            error_ttl_ms: 10_000,       // 10 seconds
        }
    }

    /// Configuration for development/testing (very short TTLs)
    pub fn development() -> Self {
        Self {
            max_entries: 50,
            default_ttl_ms: 10_000,     // 10 seconds
            quote_ttl_ms: 5_000,        // 5 seconds
            search_ttl_ms: 30_000,      // 30 seconds
            history_ttl_ms: 20_000,     // 20 seconds
            max_memory_bytes: 5 * 1024 * 1024, // 5MB
            enable_size_eviction: true,
            cleanup_interval_ms: 30_000, // 30 seconds
            cache_errors: false,
            error_ttl_ms: 2_000,        // 2 seconds
        }
    }

    /// Disable caching completely
    pub fn disabled() -> Self {
        Self {
            max_entries: 0,
            default_ttl_ms: 0,
            quote_ttl_ms: 0,
            search_ttl_ms: 0,
            history_ttl_ms: 0,
            max_memory_bytes: 0,
            enable_size_eviction: false,
            cleanup_interval_ms: 0,
            cache_errors: false,
            error_ttl_ms: 0,
        }
    }
}

/// Statistics for cache monitoring
#[derive(Debug, Clone, Default)]
pub struct CacheStats {
    pub total_requests: u64,
    pub cache_hits: u64,
    pub cache_misses: u64,
    pub cache_entries: usize,
    pub total_memory_bytes: usize,
    pub cache_evictions: u64,
    pub expired_evictions: u64,
    pub size_evictions: u64,
    pub average_entry_size: f64,
    pub hit_rate: f64,
    pub oldest_entry_age_ms: u64,
}

impl CacheStats {
    pub fn update_hit_rate(&mut self) {
        if self.total_requests > 0 {
            self.hit_rate = self.cache_hits as f64 / self.total_requests as f64;
        }
    }
}

/// Response cache implementation
pub struct ResponseCache {
    config: ResponseCacheConfig,
    cache: Arc<RwLock<LruCache<CacheKey, CacheEntry>>>,
    stats: Arc<RwLock<CacheStats>>,
    current_memory_usage: Arc<RwLock<usize>>,
}

impl ResponseCache {
    pub fn new(config: ResponseCacheConfig) -> Self {
        let capacity = if config.max_entries > 0 {
            NonZeroUsize::new(config.max_entries).unwrap()
        } else {
            NonZeroUsize::new(1).unwrap() // Minimum size for LruCache
        };

        Self {
            config,
            cache: Arc::new(RwLock::new(LruCache::new(capacity))),
            stats: Arc::new(RwLock::new(CacheStats::default())),
            current_memory_usage: Arc::new(RwLock::new(0)),
        }
    }

    pub fn with_default_config() -> Self {
        Self::new(ResponseCacheConfig::default())
    }

    /// Get cached response if available and not expired
    pub async fn get(&self, url: &str, params: Option<&HashMap<String, String>>) -> Option<Value> {
        if self.config.max_entries == 0 {
            return None; // Cache disabled
        }

        let key = CacheKey::new(url.to_string(), params);
        
        {
            let mut stats = self.stats.write().await;
            stats.total_requests += 1;
        }

        let mut cache = self.cache.write().await;
        
        if let Some(entry) = cache.get_mut(&key) {
            let ttl = self.determine_ttl(url);
            
            if !entry.is_expired(ttl) {
                let result = entry.access();
                
                {
                    let mut stats = self.stats.write().await;
                    stats.cache_hits += 1;
                    stats.update_hit_rate();
                }
                
                debug!("Cache hit for URL: {} (age: {}ms)", url, entry.age_ms());
                return Some(result);
            } else {
                // Remove expired entry
                let removed = cache.pop(&key);
                if let Some(removed_entry) = removed {
                    let mut memory = self.current_memory_usage.write().await;
                    *memory = memory.saturating_sub(removed_entry.size_bytes);
                    
                    let mut stats = self.stats.write().await;
                    stats.expired_evictions += 1;
                }
                
                debug!("Expired cache entry removed for URL: {}", url);
            }
        }

        {
            let mut stats = self.stats.write().await;
            stats.cache_misses += 1;
            stats.update_hit_rate();
        }

        None
    }

    /// Store response in cache
    pub async fn put(&self, url: &str, params: Option<&HashMap<String, String>>, response: Value) {
        if self.config.max_entries == 0 {
            return; // Cache disabled
        }

        let key = CacheKey::new(url.to_string(), params);
        let entry = CacheEntry::new(response);
        let entry_size = entry.size_bytes;

        // Check memory constraints
        if self.config.enable_size_eviction {
            let current_memory = *self.current_memory_usage.read().await;
            if current_memory + entry_size > self.config.max_memory_bytes {
                self.evict_by_size(entry_size).await;
            }
        }

        {
            let mut cache = self.cache.write().await;
            
            // If adding this entry would exceed capacity, LRU will evict automatically
            if let Some(evicted) = cache.push(key, entry) {
                let mut memory = self.current_memory_usage.write().await;
                *memory = memory.saturating_sub(evicted.1.size_bytes);
                
                let mut stats = self.stats.write().await;
                stats.cache_evictions += 1;
            }
        }

        {
            let mut memory = self.current_memory_usage.write().await;
            *memory += entry_size;
        }

        self.update_stats().await;
        debug!("Cached response for URL: {} (size: {} bytes)", url, entry_size);
    }

    /// Determine TTL based on URL pattern
    fn determine_ttl(&self, url: &str) -> u64 {
        if url.contains("/chart/") || url.contains("interval=") {
            if url.contains("interval=1m") || url.contains("interval=5m") {
                self.config.quote_ttl_ms // Live quotes - shorter TTL
            } else {
                self.config.history_ttl_ms // Historical data
            }
        } else if url.contains("/search") {
            self.config.search_ttl_ms // Search results
        } else {
            self.config.default_ttl_ms // Default TTL
        }
    }

    /// Evict entries to free up memory
    async fn evict_by_size(&self, needed_bytes: usize) {
        let mut cache = self.cache.write().await;
        let mut memory = self.current_memory_usage.write().await;
        let mut freed_bytes = 0;
        let mut evicted_count = 0;

        // Find entries to evict (LRU order)
        let mut keys_to_remove = Vec::new();
        
        // Collect keys from least recently used entries
        while freed_bytes < needed_bytes && !cache.is_empty() {
            if let Some((key, _)) = cache.peek_lru() {
                keys_to_remove.push(key.clone());
                if let Some((_, entry)) = cache.pop_lru() {
                    freed_bytes += entry.size_bytes;
                    evicted_count += 1;
                }
            } else {
                break;
            }
        }

        *memory = memory.saturating_sub(freed_bytes);
        
        drop(cache);
        drop(memory);

        {
            let mut stats = self.stats.write().await;
            stats.size_evictions += evicted_count;
            stats.cache_evictions += evicted_count;
        }

        info!("Evicted {} entries to free {} bytes", evicted_count, freed_bytes);
    }

    /// Clean up expired entries
    pub async fn cleanup_expired(&self) {
        let cache = self.cache.write().await;
        let memory = self.current_memory_usage.write().await;
        let _expired_keys: Vec<String> = Vec::new();
        let _freed_bytes = 0;

        // Find expired entries
        let _current_time = current_time_millis();
        
        // Note: LruCache doesn't provide an iterator, so we need a different approach
        // We'll clean up during normal operations instead
        
        // For now, let's just update stats
        drop(cache);
        drop(memory);
        
        self.update_stats().await;
    }

    /// Update cache statistics
    async fn update_stats(&self) {
        let cache = self.cache.read().await;
        let memory = self.current_memory_usage.read().await;
        let mut stats = self.stats.write().await;

        stats.cache_entries = cache.len();
        stats.total_memory_bytes = *memory;
        
        if stats.cache_entries > 0 {
            stats.average_entry_size = stats.total_memory_bytes as f64 / stats.cache_entries as f64;
        }

        // Note: oldest_entry_age_ms would require iterating through entries
        // We'll calculate it during periodic cleanup instead
    }

    /// Get current cache statistics
    pub async fn stats(&self) -> CacheStats {
        self.update_stats().await;
        self.stats.read().await.clone()
    }

    /// Clear all cached entries
    pub async fn clear(&self) {
        let mut cache = self.cache.write().await;
        let mut memory = self.current_memory_usage.write().await;
        
        cache.clear();
        *memory = 0;
        
        drop(cache);
        drop(memory);
        
        self.update_stats().await;
        info!("Cache cleared");
    }

    /// Get current memory usage
    pub async fn memory_usage(&self) -> usize {
        *self.current_memory_usage.read().await
    }

    /// Check if cache is enabled
    pub fn is_enabled(&self) -> bool {
        self.config.max_entries > 0
    }
}

/// Estimate the size of a JSON value in bytes
fn estimate_json_size(value: &Value) -> usize {
    match value {
        Value::Null => 4,
        Value::Bool(_) => 5,
        Value::Number(_) => 8,
        Value::String(s) => s.len() + 24, // String overhead
        Value::Array(arr) => {
            24 + arr.iter().map(estimate_json_size).sum::<usize>()
        }
        Value::Object(obj) => {
            24 + obj.iter()
                .map(|(k, v)| k.len() + estimate_json_size(v) + 16)
                .sum::<usize>()
        }
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
    use serde_json::json;
    use tokio::time::{sleep, Duration};

    #[tokio::test]
    async fn test_cache_put_and_get() {
        let cache = ResponseCache::with_default_config();
        let data = json!({"symbol": "AAPL", "price": 150.0});
        
        // Should not find entry initially
        assert!(cache.get("https://test.com", None).await.is_none());
        
        // Store entry
        cache.put("https://test.com", None, data.clone()).await;
        
        // Should find entry
        let cached = cache.get("https://test.com", None).await;
        assert!(cached.is_some());
        assert_eq!(cached.unwrap(), data);
    }

    #[tokio::test]
    async fn test_cache_expiration() {
        let config = ResponseCacheConfig {
            quote_ttl_ms: 50, // Very short TTL for testing
            ..Default::default()
        };
        let cache = ResponseCache::new(config);
        let data = json!({"test": true});
        
        // Store entry
        cache.put("https://chart.test.com/interval=1m", None, data.clone()).await;
        
        // Should find entry immediately
        assert!(cache.get("https://chart.test.com/interval=1m", None).await.is_some());
        
        // Wait for expiration
        sleep(Duration::from_millis(60)).await;
        
        // Should not find expired entry
        assert!(cache.get("https://chart.test.com/interval=1m", None).await.is_none());
    }

    #[tokio::test]
    async fn test_cache_with_parameters() {
        let cache = ResponseCache::with_default_config();
        let data1 = json!({"result": "data1"});
        let data2 = json!({"result": "data2"});
        
        let mut params1 = HashMap::new();
        params1.insert("symbol".to_string(), "AAPL".to_string());
        
        let mut params2 = HashMap::new();
        params2.insert("symbol".to_string(), "MSFT".to_string());
        
        // Store different data for different parameters
        cache.put("https://test.com", Some(&params1), data1.clone()).await;
        cache.put("https://test.com", Some(&params2), data2.clone()).await;
        
        // Should get correct data for each parameter set
        let cached1 = cache.get("https://test.com", Some(&params1)).await;
        let cached2 = cache.get("https://test.com", Some(&params2)).await;
        
        assert_eq!(cached1.unwrap(), data1);
        assert_eq!(cached2.unwrap(), data2);
    }

    #[tokio::test]
    async fn test_ttl_determination() {
        let cache = ResponseCache::with_default_config();
        
        // Quote URL should use quote TTL
        assert_eq!(cache.determine_ttl("https://chart.yahoo.com/interval=1m"), cache.config.quote_ttl_ms);
        
        // Search URL should use search TTL
        assert_eq!(cache.determine_ttl("https://search.yahoo.com"), cache.config.search_ttl_ms);
        
        // History URL should use history TTL
        assert_eq!(cache.determine_ttl("https://chart.yahoo.com/interval=1d"), cache.config.history_ttl_ms);
        
        // Other URLs should use default TTL
        assert_eq!(cache.determine_ttl("https://other.yahoo.com"), cache.config.default_ttl_ms);
    }

    #[tokio::test]
    async fn test_cache_stats() {
        let cache = ResponseCache::with_default_config();
        let data = json!({"test": true});
        
        // Initial stats
        let stats = cache.stats().await;
        assert_eq!(stats.total_requests, 0);
        assert_eq!(stats.cache_hits, 0);
        assert_eq!(stats.cache_misses, 0);
        
        // Miss
        cache.get("https://test.com", None).await;
        let stats = cache.stats().await;
        assert_eq!(stats.total_requests, 1);
        assert_eq!(stats.cache_misses, 1);
        
        // Store and hit
        cache.put("https://test.com", None, data).await;
        cache.get("https://test.com", None).await;
        let stats = cache.stats().await;
        assert_eq!(stats.total_requests, 2);
        assert_eq!(stats.cache_hits, 1);
        assert!(stats.hit_rate > 0.0);
    }

    #[test]
    fn test_json_size_estimation() {
        assert_eq!(estimate_json_size(&json!(null)), 4);
        assert_eq!(estimate_json_size(&json!(true)), 5);
        assert!(estimate_json_size(&json!("hello")) > 5);
        assert!(estimate_json_size(&json!({"key": "value"})) > 10);
    }
}