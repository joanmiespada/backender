//! # Secrets Management Library
//!
//! A flexible secrets management abstraction supporting multiple providers
//! with automatic fallback to environment variables.
//!
//! ## Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────┐
//! │                     SecretsClient                           │
//! │  ┌─────────────────────────────────────────────────────┐   │
//! │  │  1. Try Infisical (or other configured provider)    │   │
//! │  │  2. If fails → Try Environment Variable             │   │
//! │  │  3. If empty → Panic with clear error message       │   │
//! │  └─────────────────────────────────────────────────────┘   │
//! └─────────────────────────────────────────────────────────────┘
//! ```
//!
//! ## Usage
//!
//! ```rust,ignore
//! use secrets::{SecretsClient, SecretsConfig};
//!
//! #[tokio::main]
//! async fn main() {
//!     let config = SecretsConfig::from_env();
//!     let client = SecretsClient::new(config).await;
//!
//!     // Get a secret (tries Infisical first, then env var, then panics)
//!     let db_password = client.get_secret("DATABASE_PASSWORD").await;
//!
//!     // Get optional secret (returns None instead of panicking)
//!     let optional = client.get_secret_optional("OPTIONAL_KEY").await;
//! }
//! ```

mod config;
mod error;
mod provider;

pub mod providers;

pub use config::{InfisicalConfig, SecretsConfig};
pub use error::SecretsError;
pub use provider::SecretsProvider;

use providers::{EnvProvider, InfisicalProvider};
use secrecy::{ExposeSecret, Secret};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

/// Main client for accessing secrets with automatic fallback
pub struct SecretsClient {
    /// Primary provider (e.g., Infisical)
    primary: Option<Arc<dyn SecretsProvider>>,
    /// Fallback provider (environment variables)
    fallback: Arc<dyn SecretsProvider>,
    /// Cache for secrets (optional, reduces API calls)
    cache: Arc<RwLock<std::collections::HashMap<String, Secret<String>>>>,
    /// Whether caching is enabled
    cache_enabled: bool,
}

impl SecretsClient {
    /// Create a new secrets client with the given configuration
    pub async fn new(config: SecretsConfig) -> Self {
        let primary: Option<Arc<dyn SecretsProvider>> = if config.infisical.is_configured() {
            match InfisicalProvider::new(config.infisical.clone()).await {
                Ok(provider) => {
                    info!("Infisical provider initialized successfully");
                    Some(Arc::new(provider))
                }
                Err(e) => {
                    warn!(
                        error = %e,
                        "Failed to initialize Infisical provider, will use env vars only"
                    );
                    None
                }
            }
        } else {
            debug!("Infisical not configured, using environment variables only");
            None
        };

        let fallback: Arc<dyn SecretsProvider> = Arc::new(EnvProvider::new());

        Self {
            primary,
            fallback,
            cache: Arc::new(RwLock::new(std::collections::HashMap::new())),
            cache_enabled: config.cache_enabled,
        }
    }

    /// Create a client that only uses environment variables (for testing/simple setups)
    pub fn env_only() -> Self {
        Self {
            primary: None,
            fallback: Arc::new(EnvProvider::new()),
            cache: Arc::new(RwLock::new(std::collections::HashMap::new())),
            cache_enabled: false,
        }
    }

    /// Get a secret by key. Panics if not found in any provider.
    ///
    /// Order of resolution:
    /// 1. Check cache (if enabled)
    /// 2. Try primary provider (Infisical)
    /// 3. Try fallback provider (environment variables)
    /// 4. Panic with descriptive error
    pub async fn get_secret(&self, key: &str) -> Secret<String> {
        self.get_secret_optional(key).await.unwrap_or_else(|| {
            panic!(
                "FATAL: Secret '{key}' not found in any provider (Infisical, env vars). \
                 Please ensure the secret is configured in Infisical or set as environment variable."
            )
        })
    }

    /// Get a secret by key, returning None if not found
    pub async fn get_secret_optional(&self, key: &str) -> Option<Secret<String>> {
        // Check cache first
        if self.cache_enabled {
            let cache = self.cache.read().await;
            if let Some(cached) = cache.get(key) {
                debug!(key = %key, "Secret retrieved from cache");
                return Some(Secret::new(cached.expose_secret().clone()));
            }
        }

        // Try primary provider (Infisical)
        if let Some(ref primary) = self.primary {
            match primary.get_secret(key).await {
                Ok(Some(value)) => {
                    debug!(key = %key, provider = "infisical", "Secret retrieved");
                    self.cache_secret(key, &value).await;
                    return Some(value);
                }
                Ok(None) => {
                    debug!(key = %key, provider = "infisical", "Secret not found, trying fallback");
                }
                Err(e) => {
                    warn!(
                        key = %key,
                        error = %e,
                        "Failed to get secret from Infisical, trying fallback"
                    );
                }
            }
        }

        // Try fallback provider (env vars)
        match self.fallback.get_secret(key).await {
            Ok(Some(value)) => {
                debug!(key = %key, provider = "env", "Secret retrieved from environment");
                self.cache_secret(key, &value).await;
                Some(value)
            }
            Ok(None) => {
                debug!(key = %key, "Secret not found in any provider");
                None
            }
            Err(e) => {
                error!(key = %key, error = %e, "Failed to get secret from environment");
                None
            }
        }
    }

    /// Get a secret and expose its value (convenience method)
    pub async fn get_secret_value(&self, key: &str) -> String {
        self.get_secret(key).await.expose_secret().clone()
    }

    /// Get an optional secret's value
    pub async fn get_secret_value_optional(&self, key: &str) -> Option<String> {
        self.get_secret_optional(key)
            .await
            .map(|s| s.expose_secret().clone())
    }

    /// Check if a secret exists in any provider
    pub async fn has_secret(&self, key: &str) -> bool {
        self.get_secret_optional(key).await.is_some()
    }

    /// Clear the cache (useful for secret rotation)
    pub async fn clear_cache(&self) {
        let mut cache = self.cache.write().await;
        cache.clear();
        info!("Secrets cache cleared");
    }

    /// Invalidate a specific cached secret
    pub async fn invalidate(&self, key: &str) {
        let mut cache = self.cache.write().await;
        cache.remove(key);
        debug!(key = %key, "Secret invalidated from cache");
    }

    /// Cache a secret value
    async fn cache_secret(&self, key: &str, value: &Secret<String>) {
        if self.cache_enabled {
            let mut cache = self.cache.write().await;
            cache.insert(key.to_string(), Secret::new(value.expose_secret().clone()));
        }
    }

    /// Check if the primary provider (Infisical) is available
    pub fn has_primary_provider(&self) -> bool {
        self.primary.is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_env_only_client() {
        std::env::set_var("TEST_SECRET_123", "test_value");

        let client = SecretsClient::env_only();
        let secret = client.get_secret_optional("TEST_SECRET_123").await;

        assert!(secret.is_some());
        assert_eq!(secret.unwrap().expose_secret(), "test_value");

        std::env::remove_var("TEST_SECRET_123");
    }

    #[tokio::test]
    async fn test_missing_secret_returns_none() {
        let client = SecretsClient::env_only();
        let secret = client
            .get_secret_optional("NONEXISTENT_SECRET_KEY_12345")
            .await;

        assert!(secret.is_none());
    }

    #[tokio::test]
    async fn test_has_secret() {
        std::env::set_var("TEST_HAS_SECRET", "value");

        let client = SecretsClient::env_only();

        assert!(client.has_secret("TEST_HAS_SECRET").await);
        assert!(!client.has_secret("NONEXISTENT_KEY").await);

        std::env::remove_var("TEST_HAS_SECRET");
    }
}
