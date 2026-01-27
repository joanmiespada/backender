//! Error types for the secrets library

use thiserror::Error;

/// Errors that can occur when working with secrets
#[derive(Error, Debug)]
pub enum SecretsError {
    /// Failed to connect to the secrets provider
    #[error("Failed to connect to secrets provider: {0}")]
    ConnectionFailed(String),

    /// Authentication failed
    #[error("Authentication failed: {0}")]
    AuthenticationFailed(String),

    /// Secret not found
    #[error("Secret not found: {0}")]
    NotFound(String),

    /// Permission denied
    #[error("Permission denied for secret: {0}")]
    PermissionDenied(String),

    /// Invalid configuration
    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),

    /// HTTP request failed
    #[error("HTTP request failed: {0}")]
    HttpError(#[from] reqwest::Error),

    /// JSON parsing error
    #[error("Failed to parse response: {0}")]
    ParseError(#[from] serde_json::Error),

    /// Environment variable error
    #[error("Environment variable error: {0}")]
    EnvError(String),

    /// Provider not available
    #[error("Provider not available: {0}")]
    ProviderUnavailable(String),

    /// Rate limited
    #[error("Rate limited, retry after: {0}")]
    RateLimited(String),

    /// Generic internal error
    #[error("Internal error: {0}")]
    Internal(String),
}
