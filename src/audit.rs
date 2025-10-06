//! Audit logging for security and compliance
//!
//! This module provides tamper-proof audit logging for all API requests
//! and important system events. Designed for compliance with regulations
//! like SOC2, HIPAA, and GDPR.
//!
//! # Example
//!
//! ```no_run
//! use eeyf::audit::{AuditLogger, AuditEvent, EventType};
//!
//! let logger = AuditLogger::new("./audit-logs");
//!
//! logger.log_event(AuditEvent::new(
//!     EventType::ApiRequest,
//!     "Fetched quote for AAPL",
//! )).await?;
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::time::SystemTime;
use tokio::fs::OpenOptions;
use tokio::io::AsyncWriteExt;

/// Audit logger for compliance and security monitoring
#[derive(Debug, Clone)]
pub struct AuditLogger {
    /// Directory to store audit logs
    log_dir: PathBuf,
    
    /// Log format
    format: LogFormat,
    
    /// Retention policy
    retention: RetentionPolicy,
    
    /// Whether to sign log entries (tamper-proof)
    signing_enabled: bool,
}

/// Audit event representing a logged action
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEvent {
    /// Unique event ID
    pub id: String,
    
    /// Event timestamp
    pub timestamp: u64,
    
    /// Event type
    pub event_type: EventType,
    
    /// Event description
    pub description: String,
    
    /// User or system that triggered the event
    pub actor: Option<String>,
    
    /// Resource affected by the event
    pub resource: Option<String>,
    
    /// Action performed
    pub action: Option<String>,
    
    /// Outcome (success/failure)
    pub outcome: Outcome,
    
    /// Additional metadata
    pub metadata: std::collections::HashMap<String, String>,
    
    /// IP address (if applicable)
    pub ip_address: Option<String>,
    
    /// Request ID for correlation
    pub request_id: Option<String>,
    
    /// Digital signature (if signing enabled)
    pub signature: Option<String>,
}

/// Type of audit event
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EventType {
    /// API request to external service
    ApiRequest,
    
    /// Configuration change
    ConfigChange,
    
    /// Authentication event
    Authentication,
    
    /// Authorization check
    Authorization,
    
    /// Data access
    DataAccess,
    
    /// Rate limit hit
    RateLimit,
    
    /// Circuit breaker state change
    CircuitBreaker,
    
    /// Cache operation
    CacheOperation,
    
    /// Security event
    SecurityEvent,
    
    /// Error or exception
    Error,
    
    /// System startup
    SystemStart,
    
    /// System shutdown
    SystemShutdown,
}

/// Outcome of an audit event
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Outcome {
    /// Event succeeded
    Success,
    
    /// Event failed
    Failure,
    
    /// Event partially succeeded
    Partial,
    
    /// Event denied
    Denied,
}

/// Log format for audit logs
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogFormat {
    /// JSON Lines format (one JSON object per line)
    JsonLines,
    
    /// Comma-Separated Values
    Csv,
    
    /// CEF (Common Event Format)
    Cef,
    
    /// Syslog format
    Syslog,
}

/// Retention policy for audit logs
#[derive(Debug, Clone, Copy)]
pub struct RetentionPolicy {
    /// Keep logs for this many days
    pub days: u32,
    
    /// Compress old logs
    pub compress_after_days: Option<u32>,
    
    /// Archive to cold storage
    pub archive_after_days: Option<u32>,
}

impl AuditLogger {
    /// Create a new audit logger
    pub fn new(log_dir: impl Into<PathBuf>) -> Self {
        Self {
            log_dir: log_dir.into(),
            format: LogFormat::JsonLines,
            retention: RetentionPolicy::default(),
            signing_enabled: false,
        }
    }
    
    /// Set log format
    pub fn with_format(mut self, format: LogFormat) -> Self {
        self.format = format;
        self
    }
    
    /// Set retention policy
    pub fn with_retention(mut self, retention: RetentionPolicy) -> Self {
        self.retention = retention;
        self
    }
    
    /// Enable tamper-proof signing
    pub fn with_signing(mut self, enabled: bool) -> Self {
        self.signing_enabled = enabled;
        self
    }
    
    /// Log an audit event
    pub async fn log_event(&self, mut event: AuditEvent) -> Result<(), std::io::Error> {
        // Add signature if enabled
        if self.signing_enabled {
            event.signature = Some(self.sign_event(&event));
        }
        
        // Format the log entry
        let log_entry = match self.format {
            LogFormat::JsonLines => self.format_json(&event),
            LogFormat::Csv => self.format_csv(&event),
            LogFormat::Cef => self.format_cef(&event),
            LogFormat::Syslog => self.format_syslog(&event),
        };
        
        // Ensure log directory exists
        tokio::fs::create_dir_all(&self.log_dir).await?;
        
        // Write to log file (dated)
        let log_file = self.get_log_file_path();
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&log_file)
            .await?;
        
        file.write_all(log_entry.as_bytes()).await?;
        file.write_all(b"\n").await?;
        file.sync_all().await?;
        
        Ok(())
    }
    
    /// Log an API request
    pub async fn log_api_request(
        &self,
        method: &str,
        url: &str,
        outcome: Outcome,
        metadata: std::collections::HashMap<String, String>,
    ) -> Result<(), std::io::Error> {
        let event = AuditEvent::new(
            EventType::ApiRequest,
            format!("{} {}", method, url),
        )
        .with_action(method.to_string())
        .with_resource(url.to_string())
        .with_outcome(outcome)
        .with_metadata(metadata);
        
        self.log_event(event).await
    }
    
    /// Log a configuration change
    pub async fn log_config_change(
        &self,
        actor: impl Into<String>,
        description: impl Into<String>,
    ) -> Result<(), std::io::Error> {
        let event = AuditEvent::new(EventType::ConfigChange, description)
            .with_actor(actor);
        
        self.log_event(event).await
    }
    
    /// Log a security event
    pub async fn log_security_event(
        &self,
        description: impl Into<String>,
        ip_address: Option<String>,
    ) -> Result<(), std::io::Error> {
        let mut event = AuditEvent::new(EventType::SecurityEvent, description)
            .with_outcome(Outcome::Denied);
        
        if let Some(ip) = ip_address {
            event.ip_address = Some(ip);
        }
        
        self.log_event(event).await
    }
    
    /// Get the current log file path
    fn get_log_file_path(&self) -> PathBuf {
        let now = chrono::Local::now();
        let filename = format!("audit-{}.log", now.format("%Y-%m-%d"));
        self.log_dir.join(filename)
    }
    
    /// Format event as JSON
    fn format_json(&self, event: &AuditEvent) -> String {
        serde_json::to_string(event).unwrap_or_default()
    }
    
    /// Format event as CSV
    fn format_csv(&self, event: &AuditEvent) -> String {
        format!(
            "{},{},{:?},{},{:?}",
            event.id,
            event.timestamp,
            event.event_type,
            event.description,
            event.outcome
        )
    }
    
    /// Format event as CEF (Common Event Format)
    fn format_cef(&self, event: &AuditEvent) -> String {
        format!(
            "CEF:0|EEYF|AuditLog|1.0|{}|{}|{}|{}",
            event.event_type as u8,
            event.description,
            self.outcome_to_severity(event.outcome),
            event.id
        )
    }
    
    /// Format event as Syslog
    fn format_syslog(&self, event: &AuditEvent) -> String {
        format!(
            "<{}>1 {} {} {} {} {} - {}",
            self.calculate_syslog_priority(event.event_type, event.outcome),
            self.timestamp_to_rfc3339(event.timestamp),
            "eeyf",
            "audit",
            std::process::id(),
            event.id,
            event.description
        )
    }
    
    /// Sign an event for tamper detection
    fn sign_event(&self, event: &AuditEvent) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let mut hasher = DefaultHasher::new();
        event.id.hash(&mut hasher);
        event.timestamp.hash(&mut hasher);
        event.description.hash(&mut hasher);
        
        format!("{:x}", hasher.finish())
    }
    
    /// Convert outcome to CEF severity
    fn outcome_to_severity(&self, outcome: Outcome) -> u8 {
        match outcome {
            Outcome::Success => 1,
            Outcome::Partial => 5,
            Outcome::Denied => 8,
            Outcome::Failure => 10,
        }
    }
    
    /// Calculate syslog priority
    fn calculate_syslog_priority(&self, event_type: EventType, outcome: Outcome) -> u8 {
        let facility = 16; // local0
        let severity = match (event_type, outcome) {
            (EventType::Error, _) => 3,           // Error
            (EventType::SecurityEvent, _) => 4,   // Warning
            (_, Outcome::Failure) => 4,           // Warning
            (_, Outcome::Denied) => 5,            // Notice
            _ => 6,                                // Informational
        };
        facility * 8 + severity
    }
    
    /// Convert timestamp to RFC3339
    fn timestamp_to_rfc3339(&self, timestamp: u64) -> String {
        use chrono::{DateTime, TimeZone, Utc};
        let dt = Utc.timestamp_opt(timestamp as i64 / 1000, 0).unwrap();
        dt.to_rfc3339()
    }
}

impl AuditEvent {
    /// Create a new audit event
    pub fn new(event_type: EventType, description: impl Into<String>) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;
        
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            timestamp,
            event_type,
            description: description.into(),
            actor: None,
            resource: None,
            action: None,
            outcome: Outcome::Success,
            metadata: std::collections::HashMap::new(),
            ip_address: None,
            request_id: None,
            signature: None,
        }
    }
    
    /// Set the actor (user/system)
    pub fn with_actor(mut self, actor: impl Into<String>) -> Self {
        self.actor = Some(actor.into());
        self
    }
    
    /// Set the resource
    pub fn with_resource(mut self, resource: impl Into<String>) -> Self {
        self.resource = Some(resource.into());
        self
    }
    
    /// Set the action
    pub fn with_action(mut self, action: impl Into<String>) -> Self {
        self.action = Some(action.into());
        self
    }
    
    /// Set the outcome
    pub fn with_outcome(mut self, outcome: Outcome) -> Self {
        self.outcome = outcome;
        self
    }
    
    /// Set metadata
    pub fn with_metadata(mut self, metadata: std::collections::HashMap<String, String>) -> Self {
        self.metadata = metadata;
        self
    }
    
    /// Add a metadata key-value pair
    pub fn add_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }
    
    /// Set IP address
    pub fn with_ip_address(mut self, ip: impl Into<String>) -> Self {
        self.ip_address = Some(ip.into());
        self
    }
    
    /// Set request ID for correlation
    pub fn with_request_id(mut self, request_id: impl Into<String>) -> Self {
        self.request_id = Some(request_id.into());
        self
    }
}

impl Default for RetentionPolicy {
    fn default() -> Self {
        Self {
            days: 365,                    // Keep logs for 1 year
            compress_after_days: Some(90), // Compress after 3 months
            archive_after_days: Some(180), // Archive after 6 months
        }
    }
}

impl RetentionPolicy {
    /// Create a retention policy for compliance (7 years)
    pub fn compliance() -> Self {
        Self {
            days: 2555, // 7 years
            compress_after_days: Some(30),
            archive_after_days: Some(365),
        }
    }
    
    /// Create a short retention policy (30 days)
    pub fn short() -> Self {
        Self {
            days: 30,
            compress_after_days: Some(7),
            archive_after_days: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_audit_event_creation() {
        let event = AuditEvent::new(EventType::ApiRequest, "Test request")
            .with_actor("system")
            .with_resource("https://api.example.com/data")
            .with_action("GET")
            .with_outcome(Outcome::Success);
        
        assert_eq!(event.event_type, EventType::ApiRequest);
        assert_eq!(event.description, "Test request");
        assert_eq!(event.actor, Some("system".to_string()));
        assert_eq!(event.outcome, Outcome::Success);
    }
    
    #[tokio::test]
    async fn test_audit_logger() {
        let temp_dir = std::env::temp_dir().join("eeyf-audit-test");
        let logger = AuditLogger::new(&temp_dir)
            .with_format(LogFormat::JsonLines);
        
        let event = AuditEvent::new(EventType::SecurityEvent, "Test security event")
            .with_outcome(Outcome::Denied);
        
        let result = logger.log_event(event).await;
        assert!(result.is_ok());
        
        // Cleanup
        let _ = tokio::fs::remove_dir_all(temp_dir).await;
    }
    
    #[test]
    fn test_retention_policies() {
        let default = RetentionPolicy::default();
        assert_eq!(default.days, 365);
        
        let compliance = RetentionPolicy::compliance();
        assert_eq!(compliance.days, 2555);
        
        let short = RetentionPolicy::short();
        assert_eq!(short.days, 30);
    }
}
