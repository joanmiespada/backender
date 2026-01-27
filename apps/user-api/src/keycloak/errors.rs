use std::fmt;

#[derive(Debug)]
pub enum KeycloakError {
    /// Failed to obtain access token
    TokenError(String),
    /// User not found in Keycloak
    UserNotFound(String),
    /// User already exists in Keycloak
    UserAlreadyExists(String),
    /// HTTP request failed
    RequestFailed(String),
    /// Invalid response from Keycloak
    InvalidResponse(String),
    /// Keycloak is not configured
    NotConfigured,
    /// Internal error
    #[allow(dead_code)]
    Internal(String),
}

impl fmt::Display for KeycloakError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            KeycloakError::TokenError(msg) => write!(f, "token error: {msg}"),
            KeycloakError::UserNotFound(id) => write!(f, "user not found in keycloak: {id}"),
            KeycloakError::UserAlreadyExists(email) => {
                write!(f, "user already exists in keycloak: {email}")
            }
            KeycloakError::RequestFailed(msg) => write!(f, "keycloak request failed: {msg}"),
            KeycloakError::InvalidResponse(msg) => {
                write!(f, "invalid response from keycloak: {msg}")
            }
            KeycloakError::NotConfigured => write!(f, "keycloak is not configured"),
            KeycloakError::Internal(msg) => write!(f, "internal keycloak error: {msg}"),
        }
    }
}

impl std::error::Error for KeycloakError {}

impl From<reqwest::Error> for KeycloakError {
    fn from(err: reqwest::Error) -> Self {
        KeycloakError::RequestFailed(err.to_string())
    }
}
