use reqwest::{Client, StatusCode};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

use super::config::KeycloakConfig;
use super::errors::KeycloakError;
use super::models::{
    CreateKeycloakUserRequest, KeycloakCredential, KeycloakUser, TokenResponse,
    UpdateKeycloakUserRequest,
};

/// Token with expiration tracking
struct CachedToken {
    access_token: String,
    expires_at: Instant,
}

impl CachedToken {
    fn new(token: String, expires_in: u64) -> Self {
        // Subtract 30 seconds buffer to refresh before actual expiration
        let buffer = 30;
        let expires_in = if expires_in > buffer {
            expires_in - buffer
        } else {
            expires_in
        };
        Self {
            access_token: token,
            expires_at: Instant::now() + Duration::from_secs(expires_in),
        }
    }

    fn is_valid(&self) -> bool {
        Instant::now() < self.expires_at
    }
}

pub struct KeycloakClient {
    config: KeycloakConfig,
    http: Client,
    token: Arc<RwLock<Option<CachedToken>>>,
}

impl KeycloakClient {
    pub fn new(config: KeycloakConfig) -> Self {
        let http = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .expect("failed to create HTTP client");

        Self {
            config,
            http,
            token: Arc::new(RwLock::new(None)),
        }
    }

    pub fn is_configured(&self) -> bool {
        self.config.is_configured()
    }

    pub fn profile_cache_ttl(&self) -> Duration {
        self.config.profile_cache_ttl
    }

    /// Get a valid access token, refreshing if necessary
    async fn get_token(&self) -> Result<String, KeycloakError> {
        if !self.is_configured() {
            return Err(KeycloakError::NotConfigured);
        }

        // Check if we have a valid cached token
        {
            let token_guard = self.token.read().await;
            if let Some(ref cached) = *token_guard {
                if cached.is_valid() {
                    return Ok(cached.access_token.clone());
                }
            }
        }

        // Need to refresh token
        let new_token = self.fetch_token().await?;
        let token_string = new_token.access_token.clone();

        {
            let mut token_guard = self.token.write().await;
            *token_guard = Some(CachedToken::new(new_token.access_token, new_token.expires_in));
        }

        Ok(token_string)
    }

    /// Fetch a new token from Keycloak
    async fn fetch_token(&self) -> Result<TokenResponse, KeycloakError> {
        let response = self
            .http
            .post(&self.config.token_url())
            .form(&[
                ("grant_type", "client_credentials"),
                ("client_id", &self.config.client_id),
                ("client_secret", &self.config.client_secret),
            ])
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(KeycloakError::TokenError(format!(
                "status {}: {}",
                status, body
            )));
        }

        response
            .json::<TokenResponse>()
            .await
            .map_err(|e| KeycloakError::InvalidResponse(e.to_string()))
    }

    /// Get a user by Keycloak ID
    pub async fn get_user_by_id(&self, keycloak_id: &str) -> Result<Option<KeycloakUser>, KeycloakError> {
        let token = self.get_token().await?;

        let response = self
            .http
            .get(&self.config.admin_user_url(keycloak_id))
            .bearer_auth(&token)
            .send()
            .await?;

        match response.status() {
            StatusCode::OK => {
                let user = response
                    .json::<KeycloakUser>()
                    .await
                    .map_err(|e| KeycloakError::InvalidResponse(e.to_string()))?;
                Ok(Some(user))
            }
            StatusCode::NOT_FOUND => Ok(None),
            status => {
                let body = response.text().await.unwrap_or_default();
                Err(KeycloakError::RequestFailed(format!(
                    "get user failed with status {}: {}",
                    status, body
                )))
            }
        }
    }

    /// Create a new user in Keycloak
    pub async fn create_user(
        &self,
        email: &str,
        first_name: Option<&str>,
        last_name: Option<&str>,
        password: Option<&str>,
    ) -> Result<String, KeycloakError> {
        let token = self.get_token().await?;

        let credentials = password.map(|pwd| {
            vec![KeycloakCredential {
                credential_type: "password".to_string(),
                value: pwd.to_string(),
                temporary: false,
            }]
        });

        let request = CreateKeycloakUserRequest {
            username: email.to_string(),
            email: Some(email.to_string()),
            first_name: first_name.map(String::from),
            last_name: last_name.map(String::from),
            enabled: true,
            credentials,
        };

        let response = self
            .http
            .post(&self.config.admin_users_url())
            .bearer_auth(&token)
            .json(&request)
            .send()
            .await?;

        match response.status() {
            StatusCode::CREATED => {
                // Extract the user ID from the Location header
                if let Some(location) = response.headers().get("location") {
                    let location_str = location.to_str().unwrap_or_default();
                    // Location format: http://host/admin/realms/{realm}/users/{id}
                    if let Some(id) = location_str.rsplit('/').next() {
                        return Ok(id.to_string());
                    }
                }
                Err(KeycloakError::InvalidResponse(
                    "missing Location header in create response".to_string(),
                ))
            }
            StatusCode::CONFLICT => Err(KeycloakError::UserAlreadyExists(email.to_string())),
            status => {
                let body = response.text().await.unwrap_or_default();
                Err(KeycloakError::RequestFailed(format!(
                    "create user failed with status {}: {}",
                    status, body
                )))
            }
        }
    }

    /// Update a user in Keycloak
    pub async fn update_user(
        &self,
        keycloak_id: &str,
        first_name: Option<&str>,
        last_name: Option<&str>,
    ) -> Result<(), KeycloakError> {
        let token = self.get_token().await?;

        let request = UpdateKeycloakUserRequest {
            first_name: first_name.map(String::from),
            last_name: last_name.map(String::from),
            email: None,
        };

        let response = self
            .http
            .put(&self.config.admin_user_url(keycloak_id))
            .bearer_auth(&token)
            .json(&request)
            .send()
            .await?;

        match response.status() {
            StatusCode::NO_CONTENT => Ok(()),
            StatusCode::NOT_FOUND => Err(KeycloakError::UserNotFound(keycloak_id.to_string())),
            status => {
                let body = response.text().await.unwrap_or_default();
                Err(KeycloakError::RequestFailed(format!(
                    "update user failed with status {}: {}",
                    status, body
                )))
            }
        }
    }

    /// Delete a user from Keycloak
    pub async fn delete_user(&self, keycloak_id: &str) -> Result<(), KeycloakError> {
        let token = self.get_token().await?;

        let response = self
            .http
            .delete(&self.config.admin_user_url(keycloak_id))
            .bearer_auth(&token)
            .send()
            .await?;

        match response.status() {
            StatusCode::NO_CONTENT => Ok(()),
            StatusCode::NOT_FOUND => {
                // User already deleted, treat as success
                Ok(())
            }
            status => {
                let body = response.text().await.unwrap_or_default();
                Err(KeycloakError::RequestFailed(format!(
                    "delete user failed with status {}: {}",
                    status, body
                )))
            }
        }
    }

    /// Get users by email (for lookup during sync)
    pub async fn get_users_by_email(&self, email: &str) -> Result<Vec<KeycloakUser>, KeycloakError> {
        let token = self.get_token().await?;

        let url = format!("{}?email={}&exact=true", self.config.admin_users_url(), email);

        let response = self.http.get(&url).bearer_auth(&token).send().await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(KeycloakError::RequestFailed(format!(
                "search users failed with status {}: {}",
                status, body
            )));
        }

        response
            .json::<Vec<KeycloakUser>>()
            .await
            .map_err(|e| KeycloakError::InvalidResponse(e.to_string()))
    }
}
