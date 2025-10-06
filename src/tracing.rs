//! Distributed tracing implementation for EEYF
//!
//! This module provides comprehensive tracing capabilities using the `tracing`
//! crate, enabling distributed request tracking and span correlation across
//! the EEYF library components.

use crate::yahoo_error::YahooError;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;
use uuid::Uuid;

#[cfg(feature = "tracing")]
use tracing::{Level, Span, debug, error, field::Empty, info, instrument, span, warn};

#[cfg(feature = "tracing")]
use tracing_subscriber::{
    EnvFilter, Registry,
    fmt::{self, format::FmtSpan},
    layer::SubscriberExt,
};

/// Tracing configuration
#[derive(Debug, Clone)]
pub struct TracingConfig {
    /// Enable tracing
    pub enabled: bool,
    /// Trace level (error, warn, info, debug, trace)
    pub level: TraceLevel,
    /// Service name for tracing
    pub service_name: String,
    /// Service version
    pub service_version: String,
    /// Environment (production, staging, development)
    pub environment: String,
    /// Export traces to Jaeger
    pub jaeger_endpoint: Option<String>,
    /// Export traces to Zipkin
    pub zipkin_endpoint: Option<String>,
    /// Sample rate (0.0 to 1.0)
    pub sample_rate: f64,
    /// Maximum spans per trace
    pub max_spans_per_trace: usize,
}

impl Default for TracingConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            level: TraceLevel::Info,
            service_name: "eeyf".to_string(),
            service_version: env!("CARGO_PKG_VERSION").to_string(),
            environment: "development".to_string(),
            jaeger_endpoint: None,
            zipkin_endpoint: None,
            sample_rate: 1.0,
            max_spans_per_trace: 100,
        }
    }
}

/// Trace levels
#[derive(Debug, Clone)]
pub enum TraceLevel {
    Error,
    Warn,
    Info,
    Debug,
    Trace,
}

#[cfg(feature = "tracing")]
impl From<TraceLevel> for Level {
    fn from(level: TraceLevel) -> Self {
        match level {
            TraceLevel::Error => Level::ERROR,
            TraceLevel::Warn => Level::WARN,
            TraceLevel::Info => Level::INFO,
            TraceLevel::Debug => Level::DEBUG,
            TraceLevel::Trace => Level::TRACE,
        }
    }
}

/// Request context for tracing
#[derive(Debug, Clone)]
pub struct RequestContext {
    /// Unique request ID
    pub request_id: String,
    /// Parent trace ID (if part of larger trace)
    pub trace_id: Option<String>,
    /// Symbol being requested
    pub symbol: String,
    /// Endpoint being called
    pub endpoint: String,
    /// Start time
    pub start_time: Instant,
    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

impl RequestContext {
    /// Create a new request context
    pub fn new(symbol: &str, endpoint: &str) -> Self {
        Self {
            request_id: Uuid::new_v4().to_string(),
            trace_id: None,
            symbol: symbol.to_string(),
            endpoint: endpoint.to_string(),
            start_time: Instant::now(),
            metadata: HashMap::new(),
        }
    }

    /// Create a child context from parent trace
    pub fn with_parent_trace(symbol: &str, endpoint: &str, parent_trace_id: &str) -> Self {
        let mut ctx = Self::new(symbol, endpoint);
        ctx.trace_id = Some(parent_trace_id.to_string());
        ctx
    }

    /// Add metadata to the context
    pub fn with_metadata(mut self, key: &str, value: &str) -> Self {
        self.metadata.insert(key.to_string(), value.to_string());
        self
    }

    /// Get elapsed time since request start
    pub fn elapsed(&self) -> std::time::Duration {
        self.start_time.elapsed()
    }
}

/// Tracing manager
#[derive(Debug)]
pub struct TracingManager {
    #[allow(dead_code)]
    config: TracingConfig,
    active_spans: Arc<std::sync::RwLock<HashMap<String, RequestContext>>>,
}

impl TracingManager {
    /// Create a new tracing manager
    pub fn new(config: TracingConfig) -> Self {
        Self {
            config,
            active_spans: Arc::new(std::sync::RwLock::new(HashMap::new())),
        }
    }

    /// Initialize tracing subscriber
    #[cfg(feature = "tracing")]
    pub fn init_tracing(&self) -> Result<(), YahooError> {
        let filter = EnvFilter::try_from_default_env()
            .or_else(|_| {
                let level = match self.config.level {
                    TraceLevel::Error => "error",
                    TraceLevel::Warn => "warn",
                    TraceLevel::Info => "info",
                    TraceLevel::Debug => "debug",
                    TraceLevel::Trace => "trace",
                };
                EnvFilter::try_new(format!("eeyf={}", level))
            })
            .map_err(|e| YahooError::InvalidStatusCode(format!("Invalid tracing filter: {}", e)))?;

        let fmt_layer = fmt::layer()
            .with_target(true)
            .with_level(true)
            .with_thread_ids(true)
            .with_span_events(FmtSpan::CLOSE);

        let subscriber = Registry::default().with(filter).with(fmt_layer);

        // OpenTelemetry integration disabled due to version conflicts
        // TODO: Re-enable with compatible versions
        let subscriber = subscriber;

        tracing::subscriber::set_global_default(subscriber).map_err(|e| {
            YahooError::InvalidStatusCode(format!("Failed to set tracing subscriber: {}", e))
        })?;

        info!(
            "🔍 Tracing initialized for service: {}",
            self.config.service_name
        );
        Ok(())
    }

    /// Start a new trace for a request
    #[cfg(feature = "tracing")]
    #[instrument(
        skip(self),
        fields(
            request_id = %context.request_id,
            symbol = %context.symbol,
            endpoint = %context.endpoint,
            trace_id = context.trace_id.as_deref().unwrap_or("none")
        )
    )]
    pub async fn start_request_trace(&self, context: RequestContext) -> RequestSpan {
        let span = span!(
            Level::INFO,
            "eeyf_request",
            request_id = %context.request_id,
            symbol = %context.symbol,
            endpoint = %context.endpoint,
            trace_id = context.trace_id.as_deref().unwrap_or("none"),
            duration_ms = Empty,
            status = Empty,
            error_type = Empty
        );

        // Store active span
        {
            let mut active_spans = self.active_spans.write().unwrap();
            active_spans.insert(context.request_id.clone(), context.clone());
        }

        info!(
            request_id = %context.request_id,
            "Starting request for symbol: {} on endpoint: {}",
            context.symbol,
            context.endpoint
        );

        RequestSpan::new(span, context, Arc::clone(&self.active_spans))
    }

    /// Log enterprise flow span
    #[cfg(feature = "tracing")]
    #[instrument(skip(self))]
    pub fn trace_enterprise_flow(&self, request_id: &str, component: &str, operation: &str) {
        debug!(
            request_id = %request_id,
            component = %component,
            operation = %operation,
            "Enterprise flow step"
        );
    }

    /// Log rate limiter activity
    #[cfg(feature = "tracing")]
    #[instrument(skip(self))]
    pub fn trace_rate_limiter(&self, request_id: &str, wait_time_ms: u64, tokens_remaining: f64) {
        if wait_time_ms > 0 {
            warn!(
                request_id = %request_id,
                wait_time_ms = %wait_time_ms,
                tokens_remaining = %tokens_remaining,
                "Rate limit triggered, waiting"
            );
        } else {
            debug!(
                request_id = %request_id,
                tokens_remaining = %tokens_remaining,
                "Rate limit check passed"
            );
        }
    }

    /// Log circuit breaker activity  
    #[cfg(feature = "tracing")]
    #[instrument(skip(self))]
    pub fn trace_circuit_breaker(&self, request_id: &str, state: &str, action: &str) {
        match state {
            "open" => error!(
                request_id = %request_id,
                state = %state,
                action = %action,
                "Circuit breaker is open, rejecting request"
            ),
            "half-open" => warn!(
                request_id = %request_id,
                state = %state,
                action = %action,
                "Circuit breaker is half-open, allowing probe request"
            ),
            _ => debug!(
                request_id = %request_id,
                state = %state,
                action = %action,
                "Circuit breaker state change"
            ),
        }
    }

    /// Log cache activity
    #[cfg(feature = "tracing")]
    #[instrument(skip(self))]
    pub fn trace_cache_activity(&self, request_id: &str, operation: &str, hit: bool, key: &str) {
        if hit {
            debug!(
                request_id = %request_id,
                operation = %operation,
                key = %key,
                "Cache hit"
            );
        } else {
            debug!(
                request_id = %request_id,
                operation = %operation,
                key = %key,
                "Cache miss"
            );
        }
    }

    /// Log retry attempt
    #[cfg(feature = "tracing")]
    #[instrument(skip(self))]
    pub fn trace_retry_attempt(&self, request_id: &str, attempt: u32, delay_ms: u64, error: &str) {
        warn!(
            request_id = %request_id,
            attempt = %attempt,
            delay_ms = %delay_ms,
            error = %error,
            "Retrying request after failure"
        );
    }

    /// Get active span count
    pub fn active_span_count(&self) -> usize {
        self.active_spans.read().unwrap().len()
    }

    /// Cleanup old spans (should be called periodically)
    pub fn cleanup_old_spans(&self, max_age_secs: u64) {
        let mut active_spans = self.active_spans.write().unwrap();
        let cutoff = std::time::Duration::from_secs(max_age_secs);

        active_spans.retain(|_, context| context.start_time.elapsed() < cutoff);
    }
}

/// Request span wrapper
pub struct RequestSpan {
    #[cfg(feature = "tracing")]
    span: Span,
    context: RequestContext,
    active_spans: Arc<std::sync::RwLock<HashMap<String, RequestContext>>>,
}

impl RequestSpan {
    #[cfg(feature = "tracing")]
    fn new(
        span: Span,
        context: RequestContext,
        active_spans: Arc<std::sync::RwLock<HashMap<String, RequestContext>>>,
    ) -> Self {
        Self {
            span,
            context,
            active_spans,
        }
    }

    #[cfg(not(feature = "tracing"))]
    fn new(
        _span: (),
        context: RequestContext,
        active_spans: Arc<std::sync::RwLock<HashMap<String, RequestContext>>>,
    ) -> Self {
        Self {
            context,
            active_spans,
        }
    }

    /// Get the request context
    pub fn context(&self) -> &RequestContext {
        &self.context
    }

    /// Record successful completion
    #[cfg(feature = "tracing")]
    pub fn record_success(self, response_size: usize) {
        let duration_ms = self.context.elapsed().as_millis() as u64;

        self.span.record("duration_ms", duration_ms);
        self.span.record("status", "success");

        info!(
            request_id = %self.context.request_id,
            duration_ms = %duration_ms,
            response_size = %response_size,
            "Request completed successfully"
        );

        // Remove from active spans
        self.active_spans
            .write()
            .unwrap()
            .remove(&self.context.request_id);
    }

    /// Record error completion  
    #[cfg(feature = "tracing")]
    pub fn record_error(self, error: &YahooError) {
        let duration_ms = self.context.elapsed().as_millis() as u64;
        let error_type = crate::metrics::categorize_error(error);

        self.span.record("duration_ms", duration_ms);
        self.span.record("status", "error");
        self.span.record("error_type", error_type);

        error!(
            request_id = %self.context.request_id,
            duration_ms = %duration_ms,
            error_type = %error_type,
            error = %error,
            "Request failed"
        );

        // Remove from active spans
        self.active_spans
            .write()
            .unwrap()
            .remove(&self.context.request_id);
    }

    /// No-op implementations when tracing is disabled
    #[cfg(not(feature = "tracing"))]
    pub fn record_success(self, _response_size: usize) {
        self.active_spans
            .write()
            .unwrap()
            .remove(&self.context.request_id);
    }

    #[cfg(not(feature = "tracing"))]
    pub fn record_error(self, _error: &YahooError) {
        self.active_spans
            .write()
            .unwrap()
            .remove(&self.context.request_id);
    }
}

// No-op implementations when tracing feature is disabled
#[cfg(not(feature = "tracing"))]
impl TracingManager {
    pub fn init_tracing(&self) -> Result<(), YahooError> {
        Ok(())
    }

    pub async fn start_request_trace(&self, context: RequestContext) -> RequestSpan {
        // Store active span even in no-op mode to maintain consistent span counting
        {
            let mut active_spans = self.active_spans.write().unwrap();
            active_spans.insert(context.request_id.clone(), context.clone());
        }
        RequestSpan::new((), context, Arc::clone(&self.active_spans))
    }

    pub fn trace_enterprise_flow(&self, _request_id: &str, _component: &str, _operation: &str) {}
    pub fn trace_rate_limiter(
        &self,
        _request_id: &str,
        _wait_time_ms: u64,
        _tokens_remaining: f64,
    ) {
    }
    pub fn trace_circuit_breaker(&self, _request_id: &str, _state: &str, _action: &str) {}
    pub fn trace_cache_activity(
        &self,
        _request_id: &str,
        _operation: &str,
        _hit: bool,
        _key: &str,
    ) {
    }
    pub fn trace_retry_attempt(
        &self,
        _request_id: &str,
        _attempt: u32,
        _delay_ms: u64,
        _error: &str,
    ) {
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_request_context_creation() {
        let ctx = RequestContext::new("AAPL", "quotes");
        assert_eq!(ctx.symbol, "AAPL");
        assert_eq!(ctx.endpoint, "quotes");
        assert!(ctx.trace_id.is_none());
        assert!(!ctx.request_id.is_empty());
    }

    #[test]
    fn test_request_context_with_parent() {
        let ctx = RequestContext::with_parent_trace("AAPL", "quotes", "parent-123");
        assert_eq!(ctx.trace_id, Some("parent-123".to_string()));
    }

    #[test]
    fn test_request_context_metadata() {
        let ctx = RequestContext::new("AAPL", "quotes")
            .with_metadata("user_id", "123")
            .with_metadata("session", "abc");

        assert_eq!(ctx.metadata.get("user_id"), Some(&"123".to_string()));
        assert_eq!(ctx.metadata.get("session"), Some(&"abc".to_string()));
    }

    #[test]
    fn test_tracing_manager_creation() {
        let config = TracingConfig::default();
        let manager = TracingManager::new(config);
        assert_eq!(manager.active_span_count(), 0);
    }

    #[tokio::test]
    async fn test_span_lifecycle() {
        let config = TracingConfig::default();
        let manager = TracingManager::new(config);

        let context = RequestContext::new("AAPL", "quotes");
        let _request_id = context.request_id.clone();

        // Start span
        let span = manager.start_request_trace(context).await;
        assert_eq!(manager.active_span_count(), 1);

        // Complete span
        span.record_success(1024);
        assert_eq!(manager.active_span_count(), 0);
    }
}
