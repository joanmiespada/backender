use serde::{Deserialize, Serialize};
use user_lib::entities::Role;
use uuid::Uuid;

/// User representation from Keycloak Admin API
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KeycloakUser {
    pub id: String,
    pub username: String,
    #[serde(default)]
    pub email: Option<String>,
    #[serde(default)]
    pub first_name: Option<String>,
    #[serde(default)]
    pub last_name: Option<String>,
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub email_verified: bool,
}

impl KeycloakUser {
    /// Build display name from first_name and last_name
    pub fn display_name(&self) -> String {
        match (&self.first_name, &self.last_name) {
            (Some(first), Some(last)) => format!("{} {}", first, last),
            (Some(first), None) => first.clone(),
            (None, Some(last)) => last.clone(),
            (None, None) => self.username.clone(),
        }
    }
}

/// Request body for creating a user in Keycloak
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateKeycloakUserRequest {
    pub username: String,
    pub email: Option<String>,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub enabled: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub credentials: Option<Vec<KeycloakCredential>>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct KeycloakCredential {
    #[serde(rename = "type")]
    pub credential_type: String,
    pub value: String,
    pub temporary: bool,
}

/// Request body for updating a user in Keycloak
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateKeycloakUserRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub first_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
}

/// Token response from Keycloak
#[derive(Debug, Deserialize)]
pub struct TokenResponse {
    pub access_token: String,
    pub expires_in: u64,
    #[serde(default)]
    pub refresh_token: Option<String>,
    pub token_type: String,
}

/// Merged user data (local DB + Keycloak profile)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FullUser {
    pub id: Uuid,
    pub keycloak_id: String,
    pub name: String,
    pub email: Option<String>,
    pub roles: Vec<Role>,
    pub email_verified: bool,
    pub enabled: bool,
}
