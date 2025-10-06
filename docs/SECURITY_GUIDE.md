# Security Guide

This guide covers security best practices and features for using the EEYF library in production environments.

## Table of Contents

- [TLS/SSL Configuration](#tlsssl-configuration)
- [Certificate Pinning](#certificate-pinning)
- [Proxy Authentication](#proxy-authentication)
- [IP Rotation](#ip-rotation)
- [Secrets Management](#secrets-management)
- [Audit Logging](#audit-logging)
- [Rate Limiting](#rate-limiting)
- [Best Practices](#best-practices)

---

## TLS/SSL Configuration

### Secure Configuration (Recommended)

```rust
use eeyf::security::{SecurityConfig, TlsConfig, TlsVersion};

let security = SecurityConfig::new()
    .with_tls_config(
        TlsConfig::secure()
            .with_min_version(TlsVersion::Tls12)
            .with_max_version(TlsVersion::Tls13)
    )
    .build();
```

### Custom TLS Configuration

```rust
let tls_config = TlsConfig::default()
    .with_min_version(TlsVersion::Tls13)
    .add_cipher_suite("TLS_AES_256_GCM_SHA384")
    .add_cipher_suite("TLS_CHACHA20_POLY1305_SHA256");
```

### Testing-Only Insecure Configuration

⚠️ **WARNING**: Only use for testing! Never in production!

```rust
// For local testing only
let insecure = TlsConfig::insecure();
```

---

## Certificate Pinning

Certificate pinning helps prevent man-in-the-middle attacks by validating that the server's certificate matches an expected hash.

### Basic Pinning

```rust
use eeyf::security::CertificatePinning;

let pinning = CertificatePinning::new(vec![
    "sha256/AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=".to_string(),
    "sha256/BBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBB=".to_string(),
]);

let security = SecurityConfig::new()
    .with_cert_pinning(pinning)
    .build();
```

### Pinning with Rotation

```rust
use std::time::Duration;

let pinning = CertificatePinning::new(vec![
    "sha256/primary_cert_hash".to_string(),
    "sha256/backup_cert_hash".to_string(),
])
.with_rotation(Duration::from_days(7)); // 7-day grace period

let security = SecurityConfig::new()
    .with_cert_pinning(pinning)
    .build();
```

### Getting Certificate Hashes

To get the SHA-256 hash of a certificate:

```bash
# Using OpenSSL
openssl s_client -connect query1.finance.yahoo.com:443 < /dev/null | \
  openssl x509 -pubkey -noout | \
  openssl rsa -pubin -outform der | \
  openssl dgst -sha256 -binary | \
  base64
```

---

## Proxy Authentication

### Basic Authentication

```rust
use eeyf::security::ProxyAuth;

let proxy_auth = ProxyAuth::basic("username", "password");

let security = SecurityConfig::new()
    .with_proxy_auth(proxy_auth)
    .build();
```

### Bearer Token

```rust
let proxy_auth = ProxyAuth::bearer("your_bearer_token_here");

let security = SecurityConfig::new()
    .with_proxy_auth(proxy_auth)
    .build();
```

### Custom Header Authentication

```rust
let proxy_auth = ProxyAuth::custom("X-Custom-Auth", "secret_value");

let security = SecurityConfig::new()
    .with_proxy_auth(proxy_auth)
    .build();
```

---

## IP Rotation

IP rotation can help avoid rate limiting and distribute load across multiple network interfaces.

### Basic IP Rotation

```rust
use eeyf::security::IpRotation;

let rotation = IpRotation::new(vec![
    "192.168.1.10".to_string(),
    "192.168.1.11".to_string(),
    "192.168.1.12".to_string(),
]);

let security = SecurityConfig::new()
    .with_ip_rotation(rotation)
    .build();
```

### Rotating IPs Manually

```rust
// Get next IP
let next_ip = rotation.next_ip();

// Get current IP
let current = rotation.current_ip();

// Reset to first IP
rotation.reset();
```

---

## Secrets Management

### Environment Variables

```rust
use eeyf::security::SecretsProvider;

// Set environment variable
std::env::set_var("EEYF_API_KEY", "your_secret_key");

let provider = SecretsProvider::environment("EEYF");

// Retrieve secret
let api_key = provider.get_secret("API_KEY").await?;
```

### AWS Secrets Manager

```rust
#[cfg(feature = "aws-secrets")]
{
    let provider = SecretsProvider::AwsSecretsManager {
        secret_name: "prod/eeyf/api-keys".to_string(),
        region: "us-east-1".to_string(),
    };
    
    let api_key = provider.get_secret("api_key").await?;
}
```

### HashiCorp Vault

```rust
#[cfg(feature = "vault")]
{
    let provider = SecretsProvider::Vault {
        address: "https://vault.example.com".to_string(),
        token: "vault_token".to_string(),
        path: "secret/eeyf".to_string(),
    };
    
    let api_key = provider.get_secret("api_key").await?;
}
```

---

## Audit Logging

Audit logging provides compliance-ready logs for all API requests and security events.

### Basic Setup

```rust
use eeyf::audit::{AuditLogger, AuditEvent, EventType};

let logger = AuditLogger::new("./audit-logs")
    .with_signing(true); // Enable tamper detection

// Log an API request
logger.log_api_request(
    "GET",
    "https://query1.finance.yahoo.com/v7/finance/quote?symbols=AAPL",
    Outcome::Success,
    metadata,
).await?;
```

### Log Formats

#### JSON Lines (Default)

```rust
use eeyf::audit::LogFormat;

let logger = AuditLogger::new("./audit-logs")
    .with_format(LogFormat::JsonLines);
```

#### CEF (Common Event Format)

```rust
let logger = AuditLogger::new("./audit-logs")
    .with_format(LogFormat::Cef);
```

#### Syslog

```rust
let logger = AuditLogger::new("./audit-logs")
    .with_format(LogFormat::Syslog);
```

### Retention Policies

```rust
use eeyf::audit::RetentionPolicy;

// Compliance (7 years)
let retention = RetentionPolicy::compliance();

// Short term (30 days)
let retention = RetentionPolicy::short();

// Custom
let retention = RetentionPolicy {
    days: 365,
    compress_after_days: Some(90),
    archive_after_days: Some(180),
};

let logger = AuditLogger::new("./audit-logs")
    .with_retention(retention);
```

### Custom Events

```rust
let event = AuditEvent::new(EventType::SecurityEvent, "Suspicious activity detected")
    .with_actor("system")
    .with_ip_address("192.168.1.100")
    .with_outcome(Outcome::Denied)
    .add_metadata("reason", "Too many requests");

logger.log_event(event).await?;
```

---

## Rate Limiting

Rate limiting helps prevent abuse and ensures fair resource usage.

### Basic Rate Limiting

```rust
use eeyf::rate_limiter::RateLimiter;
use std::time::Duration;

let limiter = RateLimiter::new(
    100,                          // 100 requests
    Duration::from_secs(60),      // per minute
);
```

### Endpoint-Specific Limits

```rust
use eeyf::rate_limiter::EndpointLimits;

let limits = EndpointLimits::new()
    .set_limit("/v7/finance/quote", 200, Duration::from_secs(60))
    .set_limit("/v7/finance/chart", 100, Duration::from_secs(60))
    .set_limit("/v1/finance/search", 50, Duration::from_secs(60));
```

---

## Best Practices

### 1. Use TLS 1.2 or Higher

```rust
let tls = TlsConfig::default()
    .with_min_version(TlsVersion::Tls12);
```

### 2. Enable Certificate Verification

Never disable certificate verification in production:

```rust
// ✅ Good
let tls = TlsConfig::secure();

// ❌ Bad (testing only!)
let tls = TlsConfig::insecure();
```

### 3. Store Secrets Securely

Never hardcode secrets:

```rust
// ❌ Bad
let api_key = "hardcoded_secret_key";

// ✅ Good
let provider = SecretsProvider::environment("EEYF");
let api_key = provider.get_secret("API_KEY").await?;
```

### 4. Enable Audit Logging

```rust
let logger = AuditLogger::new("./audit-logs")
    .with_signing(true)
    .with_retention(RetentionPolicy::compliance());
```

### 5. Implement Rate Limiting

```rust
let limiter = RateLimiter::new(100, Duration::from_secs(60));
```

### 6. Use IP Rotation for High-Volume

```rust
let rotation = IpRotation::new(vec![
    "192.168.1.10".to_string(),
    "192.168.1.11".to_string(),
]);
```

### 7. Monitor Security Events

```rust
logger.log_security_event(
    "Unusual access pattern detected",
    Some("192.168.1.100".to_string()),
).await?;
```

---

## Compliance Checklist

### SOC 2

- ✅ Enable audit logging with tamper detection
- ✅ Use retention policy ≥ 1 year
- ✅ Enable TLS 1.2+
- ✅ Implement access controls
- ✅ Monitor security events

### HIPAA

- ✅ Enable audit logging with 7-year retention
- ✅ Encrypt data in transit (TLS 1.2+)
- ✅ Implement authentication
- ✅ Log all data access
- ✅ Enable tamper detection

### GDPR

- ✅ Log data access events
- ✅ Implement data retention policies
- ✅ Secure data in transit
- ✅ Enable audit trails

---

## Security Incident Response

### 1. Detecting Incidents

Review audit logs regularly:

```bash
# Check for denied access
grep "Denied" audit-logs/audit-*.log

# Check for security events
grep "SecurityEvent" audit-logs/audit-*.log
```

### 2. Investigating

```rust
// Analyze specific request ID
let events = audit_log.filter_by_request_id("req-12345");

// Check for suspicious IPs
let events = audit_log.filter_by_ip("192.168.1.100");
```

### 3. Responding

- Rotate compromised secrets immediately
- Update certificate pins if needed
- Block suspicious IPs
- Review and strengthen security configuration

---

## Security Updates

Stay informed about security updates:

1. Monitor the [SECURITY.md](../SECURITY.md) file
2. Subscribe to security advisories
3. Keep dependencies updated
4. Review audit logs regularly

---

## Support

For security issues:
- **DO NOT** open public GitHub issues
- Email: security@example.com
- Use responsible disclosure

For general security questions:
- See [SECURITY.md](../SECURITY.md)
- Check [documentation](../docs/)
- Open a discussion on GitHub
