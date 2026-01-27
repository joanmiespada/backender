//! Configuration for secrets providers

use serde::Deserialize;

/// Configuration for the secrets client
#[derive(Debug, Clone, Default)]
pub struct SecretsConfig {
    /// Infisical configuration
    pub infisical: InfisicalConfig,
    /// Whether to cache secrets in memory
    pub cache_enabled: bool,
}

impl SecretsConfig {
    /// Load configuration from environment variables
    pub fn from_env() -> Self {
        Self {
            infisical: InfisicalConfig::from_env(),
            cache_enabled: std::env::var("SECRETS_CACHE_ENABLED")
                .map(|v| v.to_lowercase() == "true")
                .unwrap_or(true),
        }
    }

    /// Create a config for env-only mode (no external provider)
    pub fn env_only() -> Self {
        Self {
            infisical: InfisicalConfig::default(),
            cache_enabled: false,
        }
    }
}

/// Configuration for Infisical provider
#[derive(Debug, Clone, Default, Deserialize)]
pub struct InfisicalConfig {
    /// Infisical API URL (e.g., https://app.infisical.com or self-hosted URL)
    pub url: Option<String>,
    /// Client ID for machine identity authentication
    pub client_id: Option<String>,
    /// Client secret for machine identity authentication
    pub client_secret: Option<String>,
    /// Project ID (workspace)
    pub project_id: Option<String>,
    /// Environment (e.g., dev, staging, prod)
    pub environment: Option<String>,
    /// Secret path (folder path within the project)
    pub secret_path: Option<String>,
}

impl InfisicalConfig {
    /// Load Infisical configuration from environment variables
    pub fn from_env() -> Self {
        Self {
            url: std::env::var("INFISICAL_URL").ok(),
            client_id: std::env::var("INFISICAL_CLIENT_ID").ok(),
            client_secret: std::env::var("INFISICAL_CLIENT_SECRET").ok(),
            project_id: std::env::var("INFISICAL_PROJECT_ID").ok(),
            environment: std::env::var("INFISICAL_ENVIRONMENT").ok(),
            secret_path: std::env::var("INFISICAL_SECRET_PATH").ok(),
        }
    }

    /// Check if Infisical is properly configured
    pub fn is_configured(&self) -> bool {
        self.url.is_some()
            && self.client_id.is_some()
            && self.client_secret.is_some()
            && self.project_id.is_some()
            && self.environment.is_some()
    }

    /// Get the API URL, defaulting to Infisical Cloud
    pub fn api_url(&self) -> String {
        self.url
            .clone()
            .unwrap_or_else(|| "https://app.infisical.com".to_string())
    }

    /// Get the secret path, defaulting to root
    pub fn path(&self) -> String {
        self.secret_path.clone().unwrap_or_else(|| "/".to_string())
    }
}
