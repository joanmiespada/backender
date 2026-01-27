use std::net::IpAddr;
use std::time::Duration;

use crate::constants::{
    CORS_ALLOWED_ORIGINS, IP_ALLOWLIST, IP_BLOCKLIST, MAX_BODY_SIZE_BYTES, RATE_LIMIT_BURST,
    RATE_LIMIT_PER_MINUTE, REQUEST_TIMEOUT_SECS, SHUTDOWN_TIMEOUT_SECS,
};

/// Validate and parse IP addresses from a comma-separated string.
/// Returns only valid IP addresses and logs warnings for invalid ones.
fn parse_ip_list(env_var: &str, value: &str) -> Vec<String> {
    value
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .filter(|s| {
            if s.parse::<IpAddr>().is_ok() {
                true
            } else {
                tracing::warn!(
                    env_var = env_var,
                    invalid_ip = s,
                    "ignoring invalid IP address in configuration"
                );
                false
            }
        })
        .collect()
}

#[derive(Debug, Clone)]
pub struct MiddlewareConfig {
    pub rate_limit_per_minute: u32,
    pub rate_limit_burst: u32,
    pub request_timeout: Duration,
    pub max_body_size: usize,
    pub shutdown_timeout: Duration,
    pub cors_allowed_origins: Vec<String>,
    pub ip_allowlist: Vec<String>,
    pub ip_blocklist: Vec<String>,
}

impl Default for MiddlewareConfig {
    fn default() -> Self {
        Self {
            rate_limit_per_minute: 100,
            rate_limit_burst: 150,
            request_timeout: Duration::from_secs(30),
            max_body_size: 1_048_576, // 1MB
            shutdown_timeout: Duration::from_secs(30),
            cors_allowed_origins: vec!["*".to_string()],
            ip_allowlist: vec![],
            ip_blocklist: vec![],
        }
    }
}

impl MiddlewareConfig {
    pub fn from_env() -> Self {
        let default = Self::default();

        let rate_limit_per_minute = std::env::var(RATE_LIMIT_PER_MINUTE)
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(default.rate_limit_per_minute);

        let rate_limit_burst = std::env::var(RATE_LIMIT_BURST)
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(default.rate_limit_burst);

        let request_timeout_secs: u64 = std::env::var(REQUEST_TIMEOUT_SECS)
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(30);

        let max_body_size = std::env::var(MAX_BODY_SIZE_BYTES)
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(default.max_body_size);

        let shutdown_timeout_secs: u64 = std::env::var(SHUTDOWN_TIMEOUT_SECS)
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(30);

        let cors_allowed_origins = std::env::var(CORS_ALLOWED_ORIGINS)
            .ok()
            .map(|v| v.split(',').map(|s| s.trim().to_string()).collect())
            .unwrap_or(default.cors_allowed_origins);

        let ip_allowlist = std::env::var(IP_ALLOWLIST)
            .ok()
            .map(|v| parse_ip_list(IP_ALLOWLIST, &v))
            .unwrap_or(default.ip_allowlist);

        let ip_blocklist = std::env::var(IP_BLOCKLIST)
            .ok()
            .map(|v| parse_ip_list(IP_BLOCKLIST, &v))
            .unwrap_or(default.ip_blocklist);

        Self {
            rate_limit_per_minute,
            rate_limit_burst,
            request_timeout: Duration::from_secs(request_timeout_secs),
            max_body_size,
            shutdown_timeout: Duration::from_secs(shutdown_timeout_secs),
            cors_allowed_origins,
            ip_allowlist,
            ip_blocklist,
        }
    }

    pub fn has_ip_filter(&self) -> bool {
        !self.ip_allowlist.is_empty() || !self.ip_blocklist.is_empty()
    }
}
