# Security Guide

This guide covers security best practices and features in EEYF for production deployments.

---

## Table of Contents

1. [API Key Management](#api-key-management)
2. [Audit Logging](#audit-logging)
3. [Rate Limiting Security](#rate-limiting-security)
4. [Network Security](#network-security)
5. [Data Protection](#data-protection)
6. [Security Checklist](#security-checklist)

---

## API Key Management

### Environment Variables (Recommended)

Store API keys in environment variables, never hardcode them:

```rust
use std::env;
use eeyf::YahooConnector;

fn create_connector() -> Result<YahooConnector, Box<dyn std::error::Error>> {
    // Read API key from environment
    let api_key = env::var("YAHOO_API_KEY")
        .expect("YAHOO_API_KEY must be set");
    
    let connector = YahooConnector::new()?;
    // Use api_key in requests if needed
    Ok(connector)
}
```

### .env Files for Development

Use `.env` files for local development (add to `.gitignore`):

```bash
# .env
YAHOO_API_KEY=your_api_key_here
YAHOO_API_SECRET=your_secret_here
```

Load with `dotenv` crate:

```rust
use dotenv::dotenv;

fn main() {
    dotenv().ok();
    let api_key = std::env::var("YAHOO_API_KEY").unwrap();
}
```

### Secret Management Services

For production, use proper secret management:

#### AWS Secrets Manager

```rust
use aws_sdk_secretsmanager::Client as SecretsClient;

async fn get_api_key() -> String {
    let config = aws_config::load_from_env().await;
    let client = SecretsClient::new(&config);
    
    let resp = client
        .get_secret_value()
        .secret_id("yahoo-finance-api-key")
        .send()
        .await
        .unwrap();
    
    resp.secret_string().unwrap().to_string()
}
```

#### HashiCorp Vault

```rust
use vaultrs::client::{VaultClient, VaultClientSettingsBuilder};

async fn get_api_key() -> String {
    let client = VaultClient::new(
        VaultClientSettingsBuilder::default()
            .address("https://vault.example.com")
            .token("your-token")
            .build()
            .unwrap()
    ).unwrap();
    
    let secret: String = vaultrs::kv2::read(&client, "mount", "yahoo-api-key")
        .await
        .unwrap();
    
    secret
}
```

---

## Audit Logging

Enable audit logging to track API usage and detect anomalies:

```rust
use eeyf::{YahooConnector, AuditLogger, AuditConfig};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Configure audit logging
    let audit_config = AuditConfig {
        log_all_requests: true,
        log_failures: true,
        log_rate_limits: true,
        log_security_events: true,
        output_format: OutputFormat::Json,
    };
    
    let audit_logger = AuditLogger::new(audit_config);
    
    let connector = YahooConnector::builder()
        .with_audit_logger(audit_logger)
        .build()?;
    
    // All requests are now logged
    let response = connector.get_latest_quotes("AAPL", "1d").await?;
    
    Ok(())
}
```

### Audit Log Format

Audit logs are written in JSON format:

```json
{
  "timestamp": "2025-10-05T10:30:00Z",
  "event_type": "api_request",
  "user_id": "user@example.com",
  "action": "get_quotes",
  "resource": "AAPL",
  "status": "success",
  "duration_ms": 245,
  "ip_address": "192.168.1.100",
  "user_agent": "eeyf/0.1.0"
}
```

### Security Events Logged

- API key usage
- Rate limit violations
- Failed authentication attempts
- Unusual request patterns
- Circuit breaker trips
- Cache misses/hits

### Log Storage Options

#### Local Files

```rust
let audit_logger = AuditLogger::builder()
    .file_output("./logs/audit.log")
    .rotate_daily()
    .max_file_size_mb(100)
    .build();
```

#### Syslog

```rust
let audit_logger = AuditLogger::builder()
    .syslog_output("localhost:514")
    .facility(Facility::Local0)
    .build();
```

#### Cloud Services

```rust
// AWS CloudWatch
let audit_logger = AuditLogger::builder()
    .cloudwatch_output("log-group-name", "log-stream-name")
    .build();

// Elasticsearch
let audit_logger = AuditLogger::builder()
    .elasticsearch_output("https://es.example.com:9200")
    .index_name("eeyf-audit")
    .build();
```

---

## Rate Limiting Security

### Configure Aggressive Rate Limits

Protect against abuse with strict rate limits:

```rust
use eeyf::{YahooConnector, RateLimitConfig};

let rate_limit_config = RateLimitConfig {
    requests_per_second: 2,
    requests_per_minute: 100,
    requests_per_hour: 2000,
    burst_size: 5,
};

let connector = YahooConnector::builder()
    .with_rate_limit(rate_limit_config)
    .build()?;
```

### IP-Based Rate Limiting

Track rate limits per IP address:

```rust
use eeyf::{YahooConnector, IpRateLimiter};

let ip_limiter = IpRateLimiter::new()
    .limit_per_ip(100) // 100 requests per hour per IP
    .ban_duration(Duration::from_secs(3600)) // Ban for 1 hour
    .whitelist(vec!["192.168.1.0/24"]) // Whitelist internal network
    .build();

let connector = YahooConnector::builder()
    .with_ip_rate_limiter(ip_limiter)
    .build()?;
```

### User-Based Rate Limiting

Track rate limits per authenticated user:

```rust
use eeyf::{YahooConnector, UserRateLimiter};

let user_limiter = UserRateLimiter::new()
    .limit_per_user(500) // 500 requests per hour per user
    .premium_tier_limit(5000) // Higher limit for premium users
    .build();

let connector = YahooConnector::builder()
    .with_user_rate_limiter(user_limiter)
    .build()?;
```

---

## Network Security

### HTTPS Enforcement

Always use HTTPS for API requests:

```rust
let connector = YahooConnector::builder()
    .enforce_https(true) // Reject HTTP connections
    .build()?;
```

### Certificate Validation

Enable strict certificate validation:

```rust
use eeyf::{YahooConnector, TlsConfig};

let tls_config = TlsConfig {
    verify_certificates: true,
    verify_hostname: true,
    min_tls_version: TlsVersion::Tls12,
    allowed_ciphers: vec![
        "TLS_ECDHE_RSA_WITH_AES_128_GCM_SHA256",
        "TLS_ECDHE_RSA_WITH_AES_256_GCM_SHA384",
    ],
};

let connector = YahooConnector::builder()
    .with_tls_config(tls_config)
    .build()?;
```

### Connection Timeouts

Set aggressive timeouts to prevent hanging connections:

```rust
use std::time::Duration;

let connector = YahooConnector::builder()
    .connect_timeout(Duration::from_secs(5))
    .request_timeout(Duration::from_secs(10))
    .idle_timeout(Duration::from_secs(30))
    .build()?;
```

### Proxy Configuration

Use corporate proxies when required:

```rust
use eeyf::{YahooConnector, ProxyConfig};

let proxy = ProxyConfig::new("https://proxy.example.com:8080")
    .with_auth("username", "password")
    .no_proxy(vec!["localhost", "127.0.0.1"]);

let connector = YahooConnector::builder()
    .with_proxy(proxy)
    .build()?;
```

---

## Data Protection

### Sensitive Data Handling

Never log sensitive data:

```rust
use eeyf::{YahooConnector, LogFilter};

let log_filter = LogFilter::new()
    .redact_field("api_key")
    .redact_field("session_token")
    .redact_field("user_email")
    .redact_pattern(r"\d{16}"); // Credit card numbers

let connector = YahooConnector::builder()
    .with_log_filter(log_filter)
    .build()?;
```

### Data Encryption at Rest

Encrypt cached data:

```rust
use eeyf::{YahooConnector, EncryptionConfig};

let encryption = EncryptionConfig {
    algorithm: EncryptionAlgorithm::Aes256Gcm,
    key_source: KeySource::EnvVar("CACHE_ENCRYPTION_KEY"),
    rotate_keys: true,
    rotation_interval: Duration::from_secs(86400 * 30), // 30 days
};

let connector = YahooConnector::builder()
    .with_cache_encryption(encryption)
    .build()?;
```

### Data Retention Policies

Configure automatic data deletion:

```rust
use eeyf::{YahooConnector, RetentionPolicy};

let retention = RetentionPolicy {
    cache_ttl: Duration::from_secs(3600), // 1 hour
    log_retention_days: 90,
    audit_log_retention_days: 365,
    auto_purge: true,
};

let connector = YahooConnector::builder()
    .with_retention_policy(retention)
    .build()?;
```

---

## Security Checklist

### Pre-Production

- [ ] API keys stored in environment variables or secret manager
- [ ] `.env` files added to `.gitignore`
- [ ] Audit logging enabled
- [ ] Rate limiting configured
- [ ] HTTPS enforcement enabled
- [ ] Certificate validation enabled
- [ ] Timeouts configured
- [ ] Sensitive data redaction implemented
- [ ] Cache encryption enabled (if using cache)
- [ ] Data retention policies configured

### Production Deployment

- [ ] Rotate API keys regularly (every 90 days)
- [ ] Monitor audit logs for suspicious activity
- [ ] Set up alerts for rate limit violations
- [ ] Configure IP whitelisting/blacklisting
- [ ] Enable DDoS protection (at infrastructure level)
- [ ] Use TLS 1.3 where possible
- [ ] Implement request signing
- [ ] Set up security scanning in CI/CD
- [ ] Configure SIEM integration
- [ ] Document incident response procedures

### Ongoing Maintenance

- [ ] Review audit logs weekly
- [ ] Update dependencies monthly for security patches
- [ ] Conduct security audits quarterly
- [ ] Test disaster recovery procedures
- [ ] Review and update rate limits based on usage
- [ ] Rotate encryption keys per policy
- [ ] Update TLS configuration annually
- [ ] Review access controls and permissions
- [ ] Test security controls with penetration testing
- [ ] Keep security documentation up to date

---

## Vulnerability Reporting

If you discover a security vulnerability in EEYF:

1. **Do NOT** open a public GitHub issue
2. Email security@eeyf.dev with details
3. Include steps to reproduce
4. Allow 90 days for patch development
5. Coordinate disclosure timing

We follow responsible disclosure practices and appreciate security researchers who help keep EEYF secure.

---

## Security Resources

- [OWASP Top 10](https://owasp.org/www-project-top-ten/)
- [Rust Security Guidelines](https://anssi-fr.github.io/rust-guide/)
- [CIS Benchmarks](https://www.cisecurity.org/cis-benchmarks/)
- [NIST Cybersecurity Framework](https://www.nist.gov/cyberframework)

---

## Additional Security Measures

### Web Application Firewall (WAF)

Deploy a WAF in front of your application:

```yaml
# Example: AWS WAF configuration
Rules:
  - Name: RateLimitRule
    Priority: 1
    Action: Block
    Condition: RequestRate > 2000 per 5 minutes
  
  - Name: GeoBlockRule
    Priority: 2
    Action: Block
    Condition: SourceCountry in [BLOCKED_COUNTRIES]
```

### DDoS Protection

Enable DDoS protection at the infrastructure level:

- Use AWS Shield, Cloudflare, or similar services
- Configure rate-based rules
- Enable automatic mitigation
- Set up alerting for attacks

### Input Validation

Always validate user inputs:

```rust
use eeyf::validate_ticker;

fn process_ticker(ticker: &str) -> Result<(), String> {
    // Validate ticker format
    if !validate_ticker(ticker) {
        return Err("Invalid ticker format".to_string());
    }
    
    // Sanitize for logging
    let safe_ticker = ticker.chars()
        .filter(|c| c.is_alphanumeric())
        .collect::<String>();
    
    // Process...
    Ok(())
}
```

---

## Compliance Considerations

### GDPR (EU)

- Log user consent for data processing
- Implement data deletion on request
- Encrypt personal data
- Maintain data processing records

### SOC 2

- Enable comprehensive audit logging
- Implement access controls
- Monitor security events
- Maintain security policies

### PCI DSS (if handling payment data)

- Never store full card numbers
- Use tokenization
- Encrypt data in transit and at rest
- Maintain firewall rules

---

## Contact

For security questions or concerns:
- Email: security@eeyf.dev
- Security Policy: [SECURITY.md](../SECURITY.md)
- Bug Bounty Program: (Coming Soon)
