# Security Policy

## Supported Versions

We actively support the following versions with security updates:

| Version | Supported          |
| ------- | ------------------ |
| 0.1.x   | :white_check_mark: |
| < 0.1   | :x:                |

**Note**: As we approach 1.0, we will provide long-term support for 1.x releases.

## Reporting a Vulnerability

We take security vulnerabilities seriously. If you discover a security issue, please report it responsibly:

### How to Report

**DO NOT** open a public GitHub issue for security vulnerabilities.

Instead, please report security issues via one of these methods:

1. **Preferred**: Use GitHub's private security advisory feature
   - Go to: <https://github.com/YOUR_USERNAME/EEYF/security/advisories>
   - Click "New draft security advisory"
   - Provide detailed information about the vulnerability

2. **Alternative**: Email security concerns to:
   - Email: `security@[your-domain].com` (or your email)
   - Subject: `[SECURITY] EEYF Vulnerability Report`

### What to Include

Please provide as much information as possible:

- **Type of vulnerability** (e.g., authentication bypass, injection, denial of service)
- **Affected component(s)** (e.g., specific module, function, or feature)
- **Attack vector** (e.g., network, local, physical)
- **Impact assessment** (what can an attacker achieve?)
- **Steps to reproduce** (detailed proof of concept)
- **Suggested fix** (if you have one)
- **Your contact information** for follow-up

### What to Expect

After you submit a vulnerability report:

1. **Acknowledgment**: Within 48 hours
   - We'll confirm receipt of your report
   - We'll assign a severity level

2. **Investigation**: Within 7 days
   - We'll validate and reproduce the issue
   - We'll determine affected versions
   - We'll develop a fix

3. **Resolution**: Within 30 days (for critical issues)
   - We'll release a patched version
   - We'll publish a security advisory
   - We'll credit you (unless you prefer anonymity)

### Severity Levels

We use the following severity classifications:

- **Critical**: Actively exploited, remote code execution, authentication bypass
  - Response time: 24-48 hours
  - Patch release: Within 7 days

- **High**: Significant impact, requires special conditions
  - Response time: 2-3 days
  - Patch release: Within 14 days

- **Medium**: Limited impact, difficult to exploit
  - Response time: 1 week
  - Patch release: Within 30 days

- **Low**: Minimal impact, theoretical issue
  - Response time: 2 weeks
  - Patch release: Next regular release

## Security Best Practices

### For Users

When using EEYF in your application:

1. **Keep Dependencies Updated**

   ```toml
   [dependencies]
   eeyf = "0.1"  # Use latest stable version
   ```

2. **Use Secure Configuration**

   ```rust
   // Always use TLS for production
   let connector = YahooConnector::builder()
       .timeout(Duration::from_secs(30))  // Set reasonable timeouts
       .build()?;
   ```

3. **Monitor Security Advisories**
   - Watch this repository for security updates
   - Subscribe to GitHub security advisories
   - Run `cargo audit` regularly

4. **Protect API Keys and Credentials**
   - Never hardcode credentials
   - Use environment variables or secret management
   - Rotate credentials regularly

5. **Rate Limiting**
   - Use built-in rate limiting to avoid IP blocks
   - Respect Yahoo Finance's Terms of Service
   - Monitor rate limit status

### For Contributors

When contributing to EEYF:

1. **Run Security Checks**

   ```bash
   # Check for known vulnerabilities
   cargo audit
   
   # Run clippy with security lints
   cargo clippy -- -W clippy::suspicious
   
   # Check for unsafe code
   cargo geiger
   ```

2. **Avoid Unsafe Code**
   - Justify all uses of `unsafe` with detailed comments
   - Minimize unsafe blocks
   - Document safety invariants

3. **Input Validation**
   - Validate all external inputs
   - Sanitize user-provided data
   - Use type system for validation where possible

4. **Error Handling**
   - Never panic on user input
   - Provide informative error messages
   - Don't leak sensitive information in errors

5. **Dependencies**
   - Minimize dependencies
   - Audit new dependencies
   - Keep dependencies up to date

## Known Security Considerations

### Rate Limiting

EEYF includes built-in rate limiting to prevent:

- IP blocking by Yahoo Finance
- Accidental denial of service
- Terms of Service violations

Users should still:

- Monitor their request rates
- Implement additional rate limiting at the application level
- Use appropriate presets for their use case

### Data Validation

Yahoo Finance API responses are validated, but users should:

- Validate critical financial data independently
- Implement sanity checks for trading decisions
- Not rely solely on this library for production trading systems

### Network Security

EEYF uses HTTPS for all API calls, but users should:

- Verify TLS certificates in production
- Use secure network configurations
- Implement additional security layers for sensitive applications

### Caching

Cached data may become stale. Users should:

- Configure appropriate cache TTLs
- Validate cached data before use in critical operations
- Implement cache invalidation strategies

## Security Update Process

1. **Discovery**: Vulnerability identified or reported
2. **Verification**: Security team validates the issue
3. **Assessment**: Determine severity and impact
4. **Development**: Create and test fix
5. **Release**: Publish patched version
6. **Advisory**: Publish security advisory
7. **Notification**: Notify users via GitHub and crates.io

## Responsible Disclosure

We follow responsible disclosure principles:

- We will acknowledge your contribution in the security advisory (unless you prefer anonymity)
- We will not take legal action against security researchers who follow this policy
- We will work with you to understand and resolve the issue
- We will credit you for your discovery (if desired)

## Security Hall of Fame

We appreciate security researchers who help keep EEYF secure:

<!-- This section will be populated with names of researchers who responsibly disclose vulnerabilities -->

Currently no reported vulnerabilities (as of October 2025).

## Contact

For non-security issues, please use:

- GitHub Issues: <https://github.com/YOUR_USERNAME/EEYF/issues>
- Discussions: <https://github.com/YOUR_USERNAME/EEYF/discussions>

For security issues, follow the reporting process above.

---

**Thank you for helping keep EEYF and its users safe!**

## Last Updated

October 5, 2025
