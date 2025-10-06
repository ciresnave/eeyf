//! Compression support for bandwidth optimization
//!
//! This module provides gzip and brotli compression support to reduce
//! bandwidth usage for large API responses.

use std::io::{Read, Write};
use flate2::{read::GzDecoder, write::GzEncoder, Compression};

/// Compression format
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompressionFormat {
    /// Gzip compression
    Gzip,
    /// Brotli compression (future support)
    Brotli,
    /// No compression
    None,
}

impl CompressionFormat {
    /// Get the Accept-Encoding header value
    pub fn accept_encoding(&self) -> &'static str {
        match self {
            CompressionFormat::Gzip => "gzip",
            CompressionFormat::Brotli => "br",
            CompressionFormat::None => "identity",
        }
    }
}

/// Compression configuration
#[derive(Debug, Clone)]
pub struct CompressionConfig {
    /// Enable compression
    pub enabled: bool,
    
    /// Compression format
    pub format: CompressionFormat,
    
    /// Compression level (0-9 for gzip)
    pub level: u32,
    
    /// Minimum size in bytes to compress (responses smaller than this won't be compressed)
    pub min_size: usize,
}

impl Default for CompressionConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            format: CompressionFormat::Gzip,
            level: 6, // Default compression level
            min_size: 1024, // 1 KB minimum
        }
    }
}

impl CompressionConfig {
    /// Create a new compression config with default values
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Enable or disable compression
    pub fn with_enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }
    
    /// Set the compression format
    pub fn with_format(mut self, format: CompressionFormat) -> Self {
        self.format = format;
        self
    }
    
    /// Set the compression level (0-9)
    pub fn with_level(mut self, level: u32) -> Self {
        self.level = level.min(9);
        self
    }
    
    /// Set the minimum size to compress
    pub fn with_min_size(mut self, size: usize) -> Self {
        self.min_size = size;
        self
    }
}

/// Compression metrics
#[derive(Debug, Clone, Default)]
pub struct CompressionMetrics {
    /// Total bytes compressed
    pub bytes_compressed: u64,
    
    /// Total bytes before compression
    pub bytes_before: u64,
    
    /// Total bytes after compression
    pub bytes_after: u64,
    
    /// Number of compression operations
    pub compressions: u64,
    
    /// Number of decompression operations
    pub decompressions: u64,
}

impl CompressionMetrics {
    /// Create a new metrics instance
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Record a compression operation
    pub fn record_compression(&mut self, before: u64, after: u64) {
        self.compressions += 1;
        self.bytes_before += before;
        self.bytes_after += after;
        self.bytes_compressed += before;
    }
    
    /// Record a decompression operation
    pub fn record_decompression(&mut self) {
        self.decompressions += 1;
    }
    
    /// Get compression ratio (0.0 - 1.0)
    pub fn compression_ratio(&self) -> f64 {
        if self.bytes_before == 0 {
            return 0.0;
        }
        self.bytes_after as f64 / self.bytes_before as f64
    }
    
    /// Get bandwidth savings (0.0 - 1.0)
    pub fn bandwidth_savings(&self) -> f64 {
        if self.bytes_before == 0 {
            return 0.0;
        }
        1.0 - self.compression_ratio()
    }
    
    /// Get average compression ratio
    pub fn average_compression_ratio(&self) -> f64 {
        if self.compressions == 0 {
            return 0.0;
        }
        self.compression_ratio()
    }
    
    /// Get total bytes saved
    pub fn bytes_saved(&self) -> u64 {
        if self.bytes_before > self.bytes_after {
            self.bytes_before - self.bytes_after
        } else {
            0
        }
    }
    
    /// Reset all metrics
    pub fn reset(&mut self) {
        *self = Self::default();
    }
}

/// Compress data using gzip
pub fn compress_gzip(data: &[u8], level: u32) -> std::io::Result<Vec<u8>> {
    let mut encoder = GzEncoder::new(Vec::new(), Compression::new(level));
    encoder.write_all(data)?;
    encoder.finish()
}

/// Decompress gzip data
pub fn decompress_gzip(data: &[u8]) -> std::io::Result<Vec<u8>> {
    let mut decoder = GzDecoder::new(data);
    let mut decompressed = Vec::new();
    decoder.read_to_end(&mut decompressed)?;
    Ok(decompressed)
}

/// Check if data should be compressed based on configuration
pub fn should_compress(data: &[u8], config: &CompressionConfig) -> bool {
    config.enabled && data.len() >= config.min_size
}

/// Compress data based on configuration
pub fn compress(data: &[u8], config: &CompressionConfig) -> std::io::Result<Vec<u8>> {
    if !should_compress(data, config) {
        return Ok(data.to_vec());
    }
    
    match config.format {
        CompressionFormat::Gzip => compress_gzip(data, config.level),
        CompressionFormat::Brotli => {
            // Brotli support not yet implemented
            Ok(data.to_vec())
        }
        CompressionFormat::None => Ok(data.to_vec()),
    }
}

/// Decompress data based on format
pub fn decompress(data: &[u8], format: CompressionFormat) -> std::io::Result<Vec<u8>> {
    match format {
        CompressionFormat::Gzip => decompress_gzip(data),
        CompressionFormat::Brotli => {
            // Brotli support not yet implemented
            Ok(data.to_vec())
        }
        CompressionFormat::None => Ok(data.to_vec()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_compression_config_default() {
        let config = CompressionConfig::default();
        assert!(config.enabled);
        assert_eq!(config.format, CompressionFormat::Gzip);
        assert_eq!(config.level, 6);
        assert_eq!(config.min_size, 1024);
    }
    
    #[test]
    fn test_compression_config_builder() {
        let config = CompressionConfig::new()
            .with_enabled(false)
            .with_format(CompressionFormat::Brotli)
            .with_level(9)
            .with_min_size(2048);
        
        assert!(!config.enabled);
        assert_eq!(config.format, CompressionFormat::Brotli);
        assert_eq!(config.level, 9);
        assert_eq!(config.min_size, 2048);
    }
    
    #[test]
    fn test_compression_format_accept_encoding() {
        assert_eq!(CompressionFormat::Gzip.accept_encoding(), "gzip");
        assert_eq!(CompressionFormat::Brotli.accept_encoding(), "br");
        assert_eq!(CompressionFormat::None.accept_encoding(), "identity");
    }
    
    #[test]
    fn test_gzip_compression() {
        // Use larger data that will definitely compress
        let data = b"Hello, World! This is a test string that should compress well. ".repeat(10);
        let compressed = compress_gzip(&data, 6).unwrap();
        
        // Compressed data should be smaller
        assert!(compressed.len() < data.len());
        
        // Decompress and verify
        let decompressed = decompress_gzip(&compressed).unwrap();
        assert_eq!(decompressed, data);
    }
    
    #[test]
    fn test_should_compress() {
        let config = CompressionConfig::new().with_min_size(10);
        
        // Small data should not be compressed
        assert!(!should_compress(b"small", &config));
        
        // Large data should be compressed
        assert!(should_compress(b"This is a longer string that should be compressed", &config));
        
        // Disabled compression
        let config = config.with_enabled(false);
        assert!(!should_compress(b"This is a longer string", &config));
    }
    
    #[test]
    fn test_compression_metrics() {
        let mut metrics = CompressionMetrics::new();
        
        // Record compression: 1000 bytes -> 500 bytes
        metrics.record_compression(1000, 500);
        assert_eq!(metrics.compressions, 1);
        assert_eq!(metrics.bytes_before, 1000);
        assert_eq!(metrics.bytes_after, 500);
        assert_eq!(metrics.bytes_saved(), 500);
        assert_eq!(metrics.compression_ratio(), 0.5);
        assert_eq!(metrics.bandwidth_savings(), 0.5);
        
        // Record another compression
        metrics.record_compression(2000, 1000);
        assert_eq!(metrics.compressions, 2);
        assert_eq!(metrics.bytes_before, 3000);
        assert_eq!(metrics.bytes_after, 1500);
        assert_eq!(metrics.compression_ratio(), 0.5);
    }
    
    #[test]
    fn test_compression_roundtrip() {
        let original = b"The quick brown fox jumps over the lazy dog. ".repeat(100);
        let config = CompressionConfig::default();
        
        let compressed = compress(&original, &config).unwrap();
        let decompressed = decompress(&compressed, CompressionFormat::Gzip).unwrap();
        
        assert_eq!(decompressed, original);
        assert!(compressed.len() < original.len());
    }
    
    #[test]
    fn test_small_data_not_compressed() {
        let small_data = b"tiny";
        let config = CompressionConfig::new().with_min_size(1024);
        
        let result = compress(small_data, &config).unwrap();
        assert_eq!(result, small_data);
    }
}
