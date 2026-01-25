use crate::constants::{
    CACHE_DEFAULT_TTL_SECS, CACHE_ENABLED, CACHE_LIST_TTL_SECS, CACHE_POOL_SIZE,
    CACHE_ROLE_TTL_SECS, CACHE_USER_TTL_SECS, REDIS_DB, REDIS_HOST, REDIS_PORT,
};
use std::time::Duration;

#[derive(Clone, Debug)]
pub struct CacheConfig {
    pub enabled: bool,
    pub redis_host: String,
    pub redis_port: u16,
    pub redis_db: i64,
    pub pool_size: usize,
    pub default_ttl: Duration,
    pub user_ttl: Duration,
    pub role_ttl: Duration,
    pub list_ttl: Duration,
}

impl CacheConfig {
    pub fn from_env() -> Self {
        let enabled = std::env::var(CACHE_ENABLED)
            .map(|v| v.to_lowercase() == "true")
            .unwrap_or(false);

        let redis_host = std::env::var(REDIS_HOST).unwrap_or_else(|_| "localhost".to_string());

        let redis_port = std::env::var(REDIS_PORT)
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(6379);

        let redis_db = std::env::var(REDIS_DB)
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(0);

        let pool_size = std::env::var(CACHE_POOL_SIZE)
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(10);

        let default_ttl_secs = std::env::var(CACHE_DEFAULT_TTL_SECS)
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(300);

        let user_ttl_secs = std::env::var(CACHE_USER_TTL_SECS)
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(300);

        let role_ttl_secs = std::env::var(CACHE_ROLE_TTL_SECS)
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(600);

        let list_ttl_secs = std::env::var(CACHE_LIST_TTL_SECS)
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(60);

        Self {
            enabled,
            redis_host,
            redis_port,
            redis_db,
            pool_size,
            default_ttl: Duration::from_secs(default_ttl_secs),
            user_ttl: Duration::from_secs(user_ttl_secs),
            role_ttl: Duration::from_secs(role_ttl_secs),
            list_ttl: Duration::from_secs(list_ttl_secs),
        }
    }

    pub fn redis_url(&self) -> String {
        format!(
            "redis://{}:{}/{}",
            self.redis_host, self.redis_port, self.redis_db
        )
    }
}
