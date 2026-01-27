//! Trait definition for secrets providers

use async_trait::async_trait;
use secrecy::Secret;

use crate::SecretsError;

/// Trait for secrets providers
///
/// Implement this trait to add support for new secrets backends
/// (e.g., HashiCorp Vault, AWS Secrets Manager, etc.)
#[async_trait]
pub trait SecretsProvider: Send + Sync {
    /// Get a secret by key
    ///
    /// Returns `Ok(Some(secret))` if found, `Ok(None)` if not found,
    /// or `Err` if there was an error accessing the provider.
    async fn get_secret(&self, key: &str) -> Result<Option<Secret<String>>, SecretsError>;

    /// Get the provider name (for logging)
    fn name(&self) -> &'static str;

    /// Check if the provider is healthy/reachable
    async fn health_check(&self) -> Result<(), SecretsError> {
        Ok(())
    }
}
