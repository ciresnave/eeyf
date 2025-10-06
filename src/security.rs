//! Security features for the EEYF library
//!
//! This module provides security enhancements including:
//! - TLS configuration
//! - Certificate pinning
//! - Proxy authentication
//! - IP rotation
//! - Secrets management
//!
//! # Example
//!
//! ```no_run
//! use std::time::Duration;
//!
//! use eeyf::security::{ProxyAuth, SecurityConfig, TlsConfig};
//!
//! let security = SecurityConfig::new()
//!     .with_tls_config(TlsConfig::default())
//!     .with_proxy_auth(ProxyAuth::basic("user", "pass"))
//!     .build();
//! ```

use std::time::Duration;

/// Security configuration for the Yahoo Finance connector
#[derive(Debug, Clone)]
pub struct SecurityConfig {
    /// TLS/SSL configuration
    pub tls_config: Option<TlsConfig>,

    /// Certificate pinning configuration
    pub cert_pinning: Option<CertificatePinning>,

    /// Proxy authentication
    pub proxy_auth: Option<ProxyAuth>,

    /// IP rotation configuration
    pub ip_rotation: Option<IpRotation>,

    /// Secrets provider
    pub secrets_provider: Option<SecretsProvider>,
}

/// TLS/SSL configuration options
#[derive(Debug, Clone)]
pub struct TlsConfig {
    /// Minimum TLS version to use
    pub min_tls_version: TlsVersion,

    /// Maximum TLS version to use
    pub max_tls_version: Option<TlsVersion>,

    /// Allowed cipher suites (empty = use defaults)
    pub cipher_suites: Vec<String>,

    /// Verify server certificates
    pub verify_certificates: bool,

    /// Accept invalid certificates (DANGEROUS - use only for testing)
    pub accept_invalid_certs: bool,

    /// Accept invalid hostnames (DANGEROUS - use only for testing)
    pub accept_invalid_hostnames: bool,
}

/// TLS protocol version
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum TlsVersion {
    /// TLS 1.0 (deprecated, not recommended)
    Tls10,
    /// TLS 1.1 (deprecated, not recommended)
    Tls11,
    /// TLS 1.2 (minimum recommended)
    Tls12,
    /// TLS 1.3 (preferred)
    Tls13,
}

/// Certificate pinning configuration
#[derive(Debug, Clone)]
pub struct CertificatePinning {
    /// SHA-256 hashes of pinned certificates
    pub pinned_certs: Vec<String>,

    /// Whether to fail requests if pinning validation fails
    pub fail_on_mismatch: bool,

    /// Allow pinning rotation (grace period for new certificates)
    pub rotation_enabled: bool,

    /// Grace period for certificate rotation
    pub rotation_grace_period: Option<Duration>,
}

/// Proxy authentication methods
#[derive(Debug, Clone)]
pub enum ProxyAuth {
    /// No authentication
    None,

    /// Basic HTTP authentication
    Basic {
        /// Username
        username: String,
        /// Password
        password: String,
    },

    /// Bearer token authentication
    Bearer {
        /// Authentication token
        token: String,
    },

    /// Custom authentication header
    Custom {
        /// Header name
        header: String,
        /// Header value
        value: String,
    },
}

/// IP rotation configuration
#[derive(Debug, Clone)]
pub struct IpRotation {
    /// List of IP addresses to rotate through
    pub addresses: Vec<String>,

    /// Rotate on rate limit
    pub rotate_on_rate_limit: bool,

    /// Rotate on error
    pub rotate_on_error: bool,

    /// Health check interval
    pub health_check_interval: Option<Duration>,

    /// Current index (internal state)
    current_index: std::sync::Arc<std::sync::atomic::AtomicUsize>,
}

/// Secrets provider for sensitive configuration
#[derive(Debug, Clone)]
pub enum SecretsProvider {
    /// Environment variables
    Environment {
        /// Prefix for environment variables
        prefix: String,
    },

    /// AWS Secrets Manager
    #[cfg(feature = "aws-secrets")]
    AwsSecretsManager {
        /// Secret name/ARN
        secret_name: String,
        /// Region
        region: String,
    },

    /// HashiCorp Vault
    #[cfg(feature = "vault")]
    Vault {
        /// Vault address
        address: String,
        /// Token or authentication method
        token: String,
        /// Secret path
        path: String,
    },

    /// Azure Key Vault
    #[cfg(feature = "azure-keyvault")]
    AzureKeyVault {
        /// Vault URL
        vault_url: String,
        /// Secret name
        secret_name: String,
    },
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            tls_config: Some(TlsConfig::default()),
            cert_pinning: None,
            proxy_auth: None,
            ip_rotation: None,
            secrets_provider: None,
        }
    }
}

impl SecurityConfig {
    /// Create a new security configuration with defaults
    pub fn new() -> SecurityConfigBuilder {
        SecurityConfigBuilder {
            config: Self::default(),
        }
    }

    /// Create a security configuration with no security features enabled
    pub fn insecure() -> Self {
        Self {
            tls_config: Some(TlsConfig::insecure()),
            cert_pinning: None,
            proxy_auth: None,
            ip_rotation: None,
            secrets_provider: None,
        }
    }
}

/// Builder for SecurityConfig
pub struct SecurityConfigBuilder {
    config: SecurityConfig,
}

impl SecurityConfigBuilder {
    /// Set TLS configuration
    pub fn with_tls_config(mut self, tls: TlsConfig) -> Self {
        self.config.tls_config = Some(tls);
        self
    }

    /// Enable certificate pinning
    pub fn with_cert_pinning(mut self, pinning: CertificatePinning) -> Self {
        self.config.cert_pinning = Some(pinning);
        self
    }

    /// Set proxy authentication
    pub fn with_proxy_auth(mut self, auth: ProxyAuth) -> Self {
        self.config.proxy_auth = Some(auth);
        self
    }

    /// Enable IP rotation
    pub fn with_ip_rotation(mut self, rotation: IpRotation) -> Self {
        self.config.ip_rotation = Some(rotation);
        self
    }

    /// Set secrets provider
    pub fn with_secrets_provider(mut self, provider: SecretsProvider) -> Self {
        self.config.secrets_provider = Some(provider);
        self
    }

    /// Build the security configuration
    pub fn build(self) -> SecurityConfig {
        self.config
    }
}

impl Default for TlsConfig {
    fn default() -> Self {
        Self {
            min_tls_version: TlsVersion::Tls12,
            max_tls_version: Some(TlsVersion::Tls13),
            cipher_suites: Vec::new(),
            verify_certificates: true,
            accept_invalid_certs: false,
            accept_invalid_hostnames: false,
        }
    }
}

impl TlsConfig {
    /// Create a secure TLS configuration (recommended for production)
    pub fn secure() -> Self {
        Self::default()
    }

    /// Create an insecure TLS configuration (for testing only)
    pub fn insecure() -> Self {
        Self {
            min_tls_version: TlsVersion::Tls10,
            max_tls_version: None,
            cipher_suites: Vec::new(),
            verify_certificates: false,
            accept_invalid_certs: true,
            accept_invalid_hostnames: true,
        }
    }

    /// Set minimum TLS version
    pub fn with_min_version(mut self, version: TlsVersion) -> Self {
        self.min_tls_version = version;
        self
    }

    /// Set maximum TLS version
    pub fn with_max_version(mut self, version: TlsVersion) -> Self {
        self.max_tls_version = Some(version);
        self
    }

    /// Add allowed cipher suite
    pub fn add_cipher_suite(mut self, suite: impl Into<String>) -> Self {
        self.cipher_suites.push(suite.into());
        self
    }
}

impl CertificatePinning {
    /// Create new certificate pinning configuration
    pub fn new(pinned_certs: Vec<String>) -> Self {
        Self {
            pinned_certs,
            fail_on_mismatch: true,
            rotation_enabled: false,
            rotation_grace_period: None,
        }
    }

    /// Enable rotation with grace period
    pub fn with_rotation(mut self, grace_period: Duration) -> Self {
        self.rotation_enabled = true;
        self.rotation_grace_period = Some(grace_period);
        self
    }
}

impl ProxyAuth {
    /// Create basic authentication
    pub fn basic(username: impl Into<String>, password: impl Into<String>) -> Self {
        Self::Basic {
            username: username.into(),
            password: password.into(),
        }
    }

    /// Create bearer token authentication
    pub fn bearer(token: impl Into<String>) -> Self {
        Self::Bearer {
            token: token.into(),
        }
    }

    /// Create custom header authentication
    pub fn custom(header: impl Into<String>, value: impl Into<String>) -> Self {
        Self::Custom {
            header: header.into(),
            value: value.into(),
        }
    }
}

impl IpRotation {
    /// Create new IP rotation configuration
    pub fn new(addresses: Vec<String>) -> Self {
        Self {
            addresses,
            rotate_on_rate_limit: true,
            rotate_on_error: false,
            health_check_interval: Some(Duration::from_secs(300)),
            current_index: std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0)),
        }
    }

    /// Get the next IP address to use
    pub fn next_ip(&self) -> Option<String> {
        if self.addresses.is_empty() {
            return None;
        }

        let index = self.current_index.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        Some(self.addresses[index % self.addresses.len()].clone())
    }

    /// Get the current IP address
    pub fn current_ip(&self) -> Option<String> {
        if self.addresses.is_empty() {
            return None;
        }

        let index = self.current_index.load(std::sync::atomic::Ordering::SeqCst);
        Some(self.addresses[index % self.addresses.len()].clone())
    }

    /// Reset rotation to first IP
    pub fn reset(&self) {
        self.current_index.store(0, std::sync::atomic::Ordering::SeqCst);
    }
}

impl SecretsProvider {
    /// Create environment-based secrets provider
    pub fn environment(prefix: impl Into<String>) -> Self {
        Self::Environment {
            prefix: prefix.into(),
        }
    }

    /// Get a secret value from the provider
    pub async fn get_secret(&self, key: &str) -> Result<String, String> {
        match self {
            Self::Environment { prefix } => {
                let env_key = if prefix.is_empty() {
                    key.to_string()
                } else {
                    format!("{}_{}", prefix, key)
                };

                std::env::var(&env_key)
                    .map_err(|_| format!("Secret '{}' not found in environment", env_key))
            },

            #[cfg(feature = "aws-secrets")]
            Self::AwsSecretsManager { .. } => {
                Err("AWS Secrets Manager not yet implemented".to_string())
            },

            #[cfg(feature = "vault")]
            Self::Vault { .. } => Err("HashiCorp Vault not yet implemented".to_string()),

            #[cfg(feature = "azure-keyvault")]
            Self::AzureKeyVault { .. } => Err("Azure Key Vault not yet implemented".to_string()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_security_config_builder() {
        let config = SecurityConfig::new()
            .with_tls_config(TlsConfig::secure())
            .with_proxy_auth(ProxyAuth::basic("user", "pass"))
            .build();

        assert!(config.tls_config.is_some());
        assert!(config.proxy_auth.is_some());
    }

    #[test]
    fn test_tls_versions() {
        assert!(TlsVersion::Tls13 > TlsVersion::Tls12);
        assert!(TlsVersion::Tls12 > TlsVersion::Tls11);
        assert!(TlsVersion::Tls11 > TlsVersion::Tls10);
    }

    #[test]
    fn test_ip_rotation() {
        let rotation = IpRotation::new(vec![
            "192.168.1.1".to_string(),
            "192.168.1.2".to_string(),
            "192.168.1.3".to_string(),
        ]);

        assert_eq!(rotation.next_ip(), Some("192.168.1.1".to_string()));
        assert_eq!(rotation.next_ip(), Some("192.168.1.2".to_string()));
        assert_eq!(rotation.next_ip(), Some("192.168.1.3".to_string()));
        assert_eq!(rotation.next_ip(), Some("192.168.1.1".to_string()));

        rotation.reset();
        assert_eq!(rotation.current_ip(), Some("192.168.1.1".to_string()));
    }

    #[test]
    fn test_proxy_auth() {
        let basic = ProxyAuth::basic("user", "password");
        match basic {
            ProxyAuth::Basic { username, password } => {
                assert_eq!(username, "user");
                assert_eq!(password, "password");
            },
            _ => panic!("Expected Basic auth"),
        }

        let bearer = ProxyAuth::bearer("token123");
        match bearer {
            ProxyAuth::Bearer { token } => {
                assert_eq!(token, "token123");
            },
            _ => panic!("Expected Bearer auth"),
        }
    }

    #[tokio::test]
    async fn test_secrets_provider_environment() {
        // SAFETY: Test code setting environment variable in controlled test environment
        unsafe {
            std::env::set_var("TEST_API_KEY", "secret123");
        }

        let provider = SecretsProvider::environment("TEST");
        let secret = provider.get_secret("API_KEY").await;

        assert!(secret.is_ok());
        assert_eq!(secret.unwrap(), "secret123");

        // SAFETY: Test cleanup removing environment variable
        unsafe {
            std::env::remove_var("TEST_API_KEY");
        }
    }
}
