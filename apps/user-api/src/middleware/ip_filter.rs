use axum::{
    body::Body,
    extract::ConnectInfo,
    http::{Request, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;
use std::net::SocketAddr;

#[derive(Debug, Serialize)]
struct ForbiddenResponse {
    error: &'static str,
    message: &'static str,
}

#[derive(Clone, Debug)]
pub struct IpFilterConfig {
    pub allowlist: Vec<String>,
    pub blocklist: Vec<String>,
}

impl IpFilterConfig {
    pub fn new(allowlist: Vec<String>, blocklist: Vec<String>) -> Self {
        Self {
            allowlist,
            blocklist,
        }
    }

    pub fn is_allowed(&self, ip: &str) -> bool {
        // If blocklist contains the IP, deny
        if self.blocklist.iter().any(|blocked| blocked == ip) {
            return false;
        }

        // If allowlist is configured, IP must be in it
        if !self.allowlist.is_empty() {
            return self.allowlist.iter().any(|allowed| allowed == ip);
        }

        // No allowlist configured and not in blocklist - allow
        true
    }
}

pub async fn ip_filter_middleware(
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    request: Request<Body>,
    next: Next,
) -> Response {
    let config = request.extensions().get::<IpFilterConfig>().cloned();

    if let Some(config) = config {
        let client_ip = addr.ip().to_string();

        if !config.is_allowed(&client_ip) {
            tracing::warn!(client_ip = %client_ip, "IP address blocked by filter");
            return (
                StatusCode::FORBIDDEN,
                Json(ForbiddenResponse {
                    error: "forbidden",
                    message: "IP address not allowed",
                }),
            )
                .into_response();
        }
    }

    next.run(request).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ip_filter_empty_lists() {
        let config = IpFilterConfig::new(vec![], vec![]);
        assert!(config.is_allowed("192.168.1.1"));
        assert!(config.is_allowed("10.0.0.1"));
    }

    #[test]
    fn test_ip_filter_blocklist() {
        let config = IpFilterConfig::new(vec![], vec!["192.168.1.1".to_string()]);
        assert!(!config.is_allowed("192.168.1.1"));
        assert!(config.is_allowed("192.168.1.2"));
    }

    #[test]
    fn test_ip_filter_allowlist() {
        let config = IpFilterConfig::new(vec!["10.0.0.1".to_string()], vec![]);
        assert!(config.is_allowed("10.0.0.1"));
        assert!(!config.is_allowed("10.0.0.2"));
    }

    #[test]
    fn test_ip_filter_blocklist_takes_precedence() {
        let config = IpFilterConfig::new(
            vec!["192.168.1.1".to_string()],
            vec!["192.168.1.1".to_string()],
        );
        // Blocklist should take precedence
        assert!(!config.is_allowed("192.168.1.1"));
    }
}
