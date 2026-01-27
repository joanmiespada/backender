//! Environment variable secrets provider

use async_trait::async_trait;
use secrecy::Secret;

use crate::{SecretsError, SecretsProvider};

/// Provider that reads secrets from environment variables
pub struct EnvProvider;

impl EnvProvider {
    pub fn new() -> Self {
        Self
    }
}

impl Default for EnvProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl SecretsProvider for EnvProvider {
    async fn get_secret(&self, key: &str) -> Result<Option<Secret<String>>, SecretsError> {
        match std::env::var(key) {
            Ok(value) if !value.is_empty() => Ok(Some(Secret::new(value))),
            Ok(_) => Ok(None), // Empty value treated as not found
            Err(std::env::VarError::NotPresent) => Ok(None),
            Err(std::env::VarError::NotUnicode(_)) => Err(SecretsError::EnvError(format!(
                "Environment variable '{key}' contains invalid UTF-8"
            ))),
        }
    }

    fn name(&self) -> &'static str {
        "environment"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use secrecy::ExposeSecret;

    #[tokio::test]
    async fn test_get_existing_env_var() {
        std::env::set_var("TEST_ENV_PROVIDER_VAR", "secret_value");

        let provider = EnvProvider::new();
        let result = provider.get_secret("TEST_ENV_PROVIDER_VAR").await;

        assert!(result.is_ok());
        let secret = result.unwrap();
        assert!(secret.is_some());
        assert_eq!(secret.unwrap().expose_secret(), "secret_value");

        std::env::remove_var("TEST_ENV_PROVIDER_VAR");
    }

    #[tokio::test]
    async fn test_get_missing_env_var() {
        let provider = EnvProvider::new();
        let result = provider
            .get_secret("NONEXISTENT_ENV_VAR_12345")
            .await
            .unwrap();

        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_empty_env_var_returns_none() {
        std::env::set_var("TEST_EMPTY_VAR", "");

        let provider = EnvProvider::new();
        let result = provider.get_secret("TEST_EMPTY_VAR").await.unwrap();

        assert!(result.is_none());

        std::env::remove_var("TEST_EMPTY_VAR");
    }
}
