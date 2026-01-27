//! Infisical secrets provider
//!
//! Uses machine identity authentication to fetch secrets from Infisical.
//! See: https://infisical.com/docs/documentation/platform/identities/machine-identities

use async_trait::async_trait;
use reqwest::Client;
use secrecy::{ExposeSecret, Secret};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::debug;

use crate::{InfisicalConfig, SecretsError, SecretsProvider};

/// Infisical secrets provider using machine identity authentication
pub struct InfisicalProvider {
    client: Client,
    config: InfisicalConfig,
    /// Cached access token
    access_token: Arc<RwLock<Option<AccessToken>>>,
}

#[derive(Debug, Clone)]
struct AccessToken {
    token: Secret<String>,
    expires_at: std::time::Instant,
}

impl AccessToken {
    fn is_expired(&self) -> bool {
        // Consider expired 30 seconds before actual expiry for safety
        self.expires_at
            .checked_sub(std::time::Duration::from_secs(30))
            .map(|t| std::time::Instant::now() > t)
            .unwrap_or(true)
    }
}

// API Response types

#[derive(Debug, Deserialize)]
struct AuthResponse {
    #[serde(rename = "accessToken")]
    access_token: String,
    #[serde(rename = "expiresIn")]
    expires_in: u64,
}

#[derive(Debug, Serialize)]
struct AuthRequest {
    #[serde(rename = "clientId")]
    client_id: String,
    #[serde(rename = "clientSecret")]
    client_secret: String,
}

#[derive(Debug, Deserialize)]
struct SecretEntry {
    #[serde(rename = "secretValue")]
    secret_value: String,
}

#[derive(Debug, Deserialize)]
struct SingleSecretResponse {
    secret: SecretEntry,
}

impl InfisicalProvider {
    /// Create a new Infisical provider with the given configuration
    ///
    /// This will authenticate with Infisical and cache the access token.
    pub async fn new(config: InfisicalConfig) -> Result<Self, SecretsError> {
        if !config.is_configured() {
            return Err(SecretsError::InvalidConfig(
                "Infisical configuration is incomplete. Required: url, client_id, client_secret, project_id, environment".to_string()
            ));
        }

        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .map_err(|e| SecretsError::ConnectionFailed(e.to_string()))?;

        let provider = Self {
            client,
            config,
            access_token: Arc::new(RwLock::new(None)),
        };

        // Authenticate on creation to fail fast if credentials are wrong
        provider.authenticate().await?;

        Ok(provider)
    }

    /// Authenticate with Infisical using machine identity credentials
    async fn authenticate(&self) -> Result<Secret<String>, SecretsError> {
        // Check if we have a valid cached token
        {
            let token_guard = self.access_token.read().await;
            if let Some(ref token) = *token_guard {
                if !token.is_expired() {
                    return Ok(Secret::new(token.token.expose_secret().clone()));
                }
            }
        }

        // Need to get a new token
        debug!("Authenticating with Infisical");

        let auth_url = format!("{}/api/v1/auth/universal-auth/login", self.config.api_url());

        let auth_request =
            AuthRequest {
                client_id: self
                    .config
                    .client_id
                    .clone()
                    .ok_or_else(|| SecretsError::InvalidConfig("Missing client_id".to_string()))?,
                client_secret: self.config.client_secret.clone().ok_or_else(|| {
                    SecretsError::InvalidConfig("Missing client_secret".to_string())
                })?,
            };

        let response = self
            .client
            .post(&auth_url)
            .json(&auth_request)
            .send()
            .await
            .map_err(|e| SecretsError::ConnectionFailed(e.to_string()))?;

        if response.status() == reqwest::StatusCode::UNAUTHORIZED {
            return Err(SecretsError::AuthenticationFailed(
                "Invalid client credentials".to_string(),
            ));
        }

        if response.status() == reqwest::StatusCode::TOO_MANY_REQUESTS {
            return Err(SecretsError::RateLimited(
                "Too many authentication attempts".to_string(),
            ));
        }

        if !response.status().is_success() {
            let status = response.status();
            let body = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(SecretsError::AuthenticationFailed(format!(
                "HTTP {status}: {body}"
            )));
        }

        let auth_response: AuthResponse = response.json().await?;

        let access_token = AccessToken {
            token: Secret::new(auth_response.access_token.clone()),
            expires_at: std::time::Instant::now()
                + std::time::Duration::from_secs(auth_response.expires_in),
        };

        // Cache the token
        {
            let mut token_guard = self.access_token.write().await;
            *token_guard = Some(access_token);
        }

        debug!("Successfully authenticated with Infisical");
        Ok(Secret::new(auth_response.access_token))
    }

    /// Get a single secret by key from Infisical
    async fn fetch_secret(&self, key: &str) -> Result<Option<Secret<String>>, SecretsError> {
        let token = self.authenticate().await?;

        let project_id = self
            .config
            .project_id
            .as_ref()
            .ok_or_else(|| SecretsError::InvalidConfig("Missing project_id".to_string()))?;

        let environment = self
            .config
            .environment
            .as_ref()
            .ok_or_else(|| SecretsError::InvalidConfig("Missing environment".to_string()))?;

        let secret_path = self.config.path();

        // Use the single secret endpoint
        let url = format!(
            "{}/api/v3/secrets/raw/{}?workspaceId={}&environment={}&secretPath={}",
            self.config.api_url(),
            urlencoding::encode(key),
            urlencoding::encode(project_id),
            urlencoding::encode(environment),
            urlencoding::encode(&secret_path)
        );

        let response = self
            .client
            .get(&url)
            .header("Authorization", format!("Bearer {}", token.expose_secret()))
            .send()
            .await
            .map_err(|e| SecretsError::ConnectionFailed(e.to_string()))?;

        match response.status() {
            reqwest::StatusCode::OK => {
                let secret_response: SingleSecretResponse = response.json().await?;
                Ok(Some(Secret::new(secret_response.secret.secret_value)))
            }
            reqwest::StatusCode::NOT_FOUND => Ok(None),
            reqwest::StatusCode::UNAUTHORIZED => {
                // Token might have expired, clear cache and retry once
                {
                    let mut token_guard = self.access_token.write().await;
                    *token_guard = None;
                }
                Err(SecretsError::AuthenticationFailed(
                    "Token expired or invalid".to_string(),
                ))
            }
            reqwest::StatusCode::FORBIDDEN => Err(SecretsError::PermissionDenied(format!(
                "Access denied for secret '{key}'"
            ))),
            reqwest::StatusCode::TOO_MANY_REQUESTS => {
                Err(SecretsError::RateLimited("Rate limit exceeded".to_string()))
            }
            status => {
                let body = response
                    .text()
                    .await
                    .unwrap_or_else(|_| "Unknown error".to_string());
                Err(SecretsError::Internal(format!("HTTP {status}: {body}")))
            }
        }
    }
}

#[async_trait]
impl SecretsProvider for InfisicalProvider {
    async fn get_secret(&self, key: &str) -> Result<Option<Secret<String>>, SecretsError> {
        self.fetch_secret(key).await
    }

    fn name(&self) -> &'static str {
        "infisical"
    }

    async fn health_check(&self) -> Result<(), SecretsError> {
        // Try to authenticate - if it works, the provider is healthy
        self.authenticate().await.map(|_| ())
    }
}
