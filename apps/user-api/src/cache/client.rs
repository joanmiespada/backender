use deadpool_redis::{Config, Connection, Pool, Runtime};
use redis::AsyncCommands;
use serde::{de::DeserializeOwned, Serialize};
use std::time::Duration;

use super::config::CacheConfig;

#[derive(Clone)]
pub struct RedisCache {
    pool: Option<Pool>,
}

impl std::fmt::Debug for RedisCache {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RedisCache")
            .field("connected", &self.pool.is_some())
            .finish()
    }
}

impl RedisCache {
    pub async fn new(config: &CacheConfig) -> Self {
        if !config.enabled {
            tracing::info!("Cache disabled by configuration");
            return Self { pool: None };
        }

        let redis_url = config.redis_url();
        tracing::info!(redis_url = %redis_url, "Connecting to Redis");

        let cfg = Config::from_url(&redis_url);
        match cfg.create_pool(Some(Runtime::Tokio1)) {
            Ok(pool) => {
                // Test connection
                match pool.get().await {
                    Ok(mut conn) => {
                        let ping_result: Result<String, _> =
                            redis::cmd("PING").query_async(&mut conn).await;
                        match ping_result {
                            Ok(_) => {
                                tracing::info!("Redis connection established");
                                Self { pool: Some(pool) }
                            }
                            Err(e) => {
                                tracing::warn!(
                                    error = %e,
                                    "Redis PING failed, cache disabled"
                                );
                                Self { pool: None }
                            }
                        }
                    }
                    Err(e) => {
                        tracing::warn!(
                            error = %e,
                            "Failed to get Redis connection, cache disabled"
                        );
                        Self { pool: None }
                    }
                }
            }
            Err(e) => {
                tracing::warn!(
                    error = %e,
                    "Failed to create Redis pool, cache disabled"
                );
                Self { pool: None }
            }
        }
    }

    pub fn is_enabled(&self) -> bool {
        self.pool.is_some()
    }

    async fn get_conn(&self) -> Option<Connection> {
        let pool = self.pool.as_ref()?;
        match pool.get().await {
            Ok(conn) => Some(conn),
            Err(e) => {
                tracing::error!(error = %e, "Failed to get Redis connection from pool");
                None
            }
        }
    }

    pub async fn get<T: DeserializeOwned>(&self, key: &str) -> Option<T> {
        let mut conn = self.get_conn().await?;

        let result: Result<Option<String>, _> = conn.get(key).await;
        match result {
            Ok(Some(data)) => match serde_json::from_str(&data) {
                Ok(value) => {
                    tracing::debug!(key = %key, "Cache hit");
                    Some(value)
                }
                Err(e) => {
                    tracing::error!(key = %key, error = %e, "Cache deserialize error - data corrupted");
                    None
                }
            },
            Ok(None) => {
                tracing::debug!(key = %key, "Cache miss");
                None
            }
            Err(e) => {
                tracing::error!(key = %key, error = %e, "Redis GET command failed");
                None
            }
        }
    }

    pub async fn set<T: Serialize>(&self, key: &str, value: &T, ttl: Duration) {
        let Some(mut conn) = self.get_conn().await else {
            return;
        };

        let data = match serde_json::to_string(value) {
            Ok(d) => d,
            Err(e) => {
                tracing::error!(key = %key, error = %e, "Cache serialize error - failed to encode value");
                return;
            }
        };

        let ttl_secs = ttl.as_secs() as i64;
        let result: Result<(), _> = conn.set_ex(key, data, ttl_secs as u64).await;
        if let Err(e) = result {
            tracing::error!(key = %key, error = %e, "Redis SETEX command failed");
        } else {
            tracing::debug!(key = %key, ttl_secs = ttl_secs, "Cache set");
        }
    }

    pub async fn delete(&self, key: &str) {
        let Some(mut conn) = self.get_conn().await else {
            return;
        };

        let result: Result<i64, _> = conn.del(key).await;
        if let Err(e) = result {
            tracing::error!(key = %key, error = %e, "Redis DEL command failed");
        } else {
            tracing::debug!(key = %key, "Cache key deleted");
        }
    }

    pub async fn delete_pattern(&self, pattern: &str) {
        let Some(mut conn) = self.get_conn().await else {
            return;
        };

        let keys: Result<Vec<String>, _> = conn.keys(pattern).await;
        match keys {
            Ok(keys) if !keys.is_empty() => {
                let result: Result<i64, _> = conn.del(&keys).await;
                match result {
                    Ok(count) => {
                        tracing::debug!(pattern = %pattern, count = count, "Cache pattern deleted");
                    }
                    Err(e) => {
                        tracing::error!(pattern = %pattern, error = %e, "Redis DEL command failed for pattern keys");
                    }
                }
            }
            Ok(_) => {
                tracing::debug!(pattern = %pattern, "No keys matched pattern");
            }
            Err(e) => {
                tracing::error!(pattern = %pattern, error = %e, "Redis KEYS command failed");
            }
        }
    }
}
