use std::time::Duration;

const KEYCLOAK_URL: &str = "KEYCLOAK_URL";
const KEYCLOAK_REALM: &str = "KEYCLOAK_REALM";
const KEYCLOAK_CLIENT_ID: &str = "KEYCLOAK_CLIENT_ID";
const KEYCLOAK_CLIENT_SECRET: &str = "KEYCLOAK_CLIENT_SECRET";
const KEYCLOAK_PROFILE_CACHE_TTL_SECS: &str = "KEYCLOAK_PROFILE_CACHE_TTL_SECS";

const DEFAULT_KEYCLOAK_URL: &str = "http://localhost:18080";
const DEFAULT_KEYCLOAK_REALM: &str = "master";
const DEFAULT_KEYCLOAK_CLIENT_ID: &str = "user-api-service";
const DEFAULT_PROFILE_CACHE_TTL_SECS: u64 = 300;

#[derive(Debug, Clone)]
pub struct KeycloakConfig {
    pub base_url: String,
    pub realm: String,
    pub client_id: String,
    pub client_secret: String,
    pub profile_cache_ttl: Duration,
}

impl KeycloakConfig {
    pub fn from_env() -> Self {
        let base_url =
            std::env::var(KEYCLOAK_URL).unwrap_or_else(|_| DEFAULT_KEYCLOAK_URL.to_string());
        let realm =
            std::env::var(KEYCLOAK_REALM).unwrap_or_else(|_| DEFAULT_KEYCLOAK_REALM.to_string());
        let client_id = std::env::var(KEYCLOAK_CLIENT_ID)
            .unwrap_or_else(|_| DEFAULT_KEYCLOAK_CLIENT_ID.to_string());
        let client_secret = std::env::var(KEYCLOAK_CLIENT_SECRET).unwrap_or_default();
        let profile_cache_ttl_secs: u64 = std::env::var(KEYCLOAK_PROFILE_CACHE_TTL_SECS)
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(DEFAULT_PROFILE_CACHE_TTL_SECS);

        Self {
            base_url,
            realm,
            client_id,
            client_secret,
            profile_cache_ttl: Duration::from_secs(profile_cache_ttl_secs),
        }
    }

    pub fn token_url(&self) -> String {
        format!(
            "{}/realms/{}/protocol/openid-connect/token",
            self.base_url, self.realm
        )
    }

    pub fn admin_users_url(&self) -> String {
        format!("{}/admin/realms/{}/users", self.base_url, self.realm)
    }

    pub fn admin_user_url(&self, keycloak_id: &str) -> String {
        format!(
            "{}/admin/realms/{}/users/{}",
            self.base_url, self.realm, keycloak_id
        )
    }

    pub fn is_configured(&self) -> bool {
        !self.client_secret.is_empty()
    }
}
