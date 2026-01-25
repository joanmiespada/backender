use std::{str::FromStr, time::Duration};

use sqlx::{mysql::{MySqlConnectOptions, MySqlPoolOptions}, MySqlPool};
use tokio::time::sleep;

/// Database pool configuration
#[derive(Debug, Clone)]
pub struct PoolConfig {
    /// Maximum number of connections in the pool
    pub max_connections: u32,
    /// Minimum number of connections to maintain
    pub min_connections: u32,
    /// Connection acquire timeout
    pub acquire_timeout: Duration,
    /// Idle connection timeout
    pub idle_timeout: Duration,
    /// Maximum connection lifetime
    pub max_lifetime: Duration,
}

impl Default for PoolConfig {
    fn default() -> Self {
        Self {
            max_connections: 10,
            min_connections: 1,
            acquire_timeout: Duration::from_secs(5),
            idle_timeout: Duration::from_secs(600),
            max_lifetime: Duration::from_secs(1800),
        }
    }
}

impl PoolConfig {
    /// Load pool configuration from environment variables
    pub fn from_env() -> Self {
        Self {
            max_connections: std::env::var("DB_MAX_CONNECTIONS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(10),
            min_connections: std::env::var("DB_MIN_CONNECTIONS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(1),
            acquire_timeout: Duration::from_secs(
                std::env::var("DB_ACQUIRE_TIMEOUT_SECS")
                    .ok()
                    .and_then(|v| v.parse().ok())
                    .unwrap_or(5),
            ),
            idle_timeout: Duration::from_secs(
                std::env::var("DB_IDLE_TIMEOUT_SECS")
                    .ok()
                    .and_then(|v| v.parse().ok())
                    .unwrap_or(600),
            ),
            max_lifetime: Duration::from_secs(
                std::env::var("DB_MAX_LIFETIME_SECS")
                    .ok()
                    .and_then(|v| v.parse().ok())
                    .unwrap_or(1800),
            ),
        }
    }
}

/// Error type for database connection failures
#[derive(Debug)]
pub struct ConnectionError {
    pub message: String,
    pub retries: u32,
}

impl std::fmt::Display for ConnectionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for ConnectionError {}

/// Connect to MySQL with retry logic and configurable pool settings
#[allow(dead_code)]
pub async fn connect_with_retry(
    database_url: &str,
    max_retries: u32,
) -> Result<MySqlPool, ConnectionError> {
    connect_with_retry_and_config(database_url, max_retries, PoolConfig::from_env()).await
}

/// Connect to MySQL with retry logic and explicit pool configuration
pub async fn connect_with_retry_and_config(
    database_url: &str,
    max_retries: u32,
    config: PoolConfig,
) -> Result<MySqlPool, ConnectionError> {
    let mut retries = 0;

    let connect_options = MySqlConnectOptions::from_str(database_url)
        .map_err(|e| ConnectionError {
            message: format!("Invalid DATABASE_URL: {}", e),
            retries: 0,
        })?;

    loop {
        match MySqlPoolOptions::new()
            .max_connections(config.max_connections)
            .min_connections(config.min_connections)
            .acquire_timeout(config.acquire_timeout)
            .idle_timeout(config.idle_timeout)
            .max_lifetime(config.max_lifetime)
            .connect_with(connect_options.clone())
            .await
        {
            Ok(pool) => return Ok(pool),
            Err(e) if retries < max_retries => {
                tracing::warn!(
                    attempt = retries + 1,
                    max_retries = max_retries,
                    error = %e,
                    "MySQL not ready yet, retrying..."
                );
                retries += 1;
                sleep(Duration::from_secs(1)).await;
            }
            Err(e) => {
                return Err(ConnectionError {
                    message: format!(
                        "Failed to connect to MySQL after {} retries: {}",
                        max_retries, e
                    ),
                    retries,
                });
            }
        }
    }
}