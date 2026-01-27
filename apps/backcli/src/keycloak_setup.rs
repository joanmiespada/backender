use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Debug, Serialize)]
struct ClientRepresentation {
    #[serde(rename = "clientId")]
    client_id: String,
    #[serde(rename = "serviceAccountsEnabled")]
    service_accounts_enabled: bool,
    #[serde(rename = "directAccessGrantsEnabled")]
    direct_access_grants_enabled: bool,
    #[serde(rename = "publicClient")]
    public_client: bool,
    protocol: String,
    #[serde(rename = "standardFlowEnabled")]
    standard_flow_enabled: bool,
}

#[derive(Debug, Deserialize)]
struct TokenResponse {
    access_token: String,
}

#[derive(Debug, Deserialize)]
struct ClientSecret {
    value: String,
}

#[derive(Debug, Deserialize)]
struct ClientDetails {
    id: String,
}

pub struct KeycloakSetup {
    base_url: String,
    realm: String,
    admin_user: String,
    admin_password: String,
    http: Client,
}

impl KeycloakSetup {
    pub fn from_env() -> Result<Self, String> {
        let base_url =
            std::env::var("KEYCLOAK_URL").unwrap_or_else(|_| "http://localhost:18080".to_string());

        let realm = std::env::var("KEYCLOAK_REALM").unwrap_or_else(|_| "master".to_string());

        let admin_user =
            std::env::var("KEYCLOAK_ADMIN_USER").unwrap_or_else(|_| "admin".to_string());

        let admin_password =
            std::env::var("KEYCLOAK_ADMIN_PASSWORD").unwrap_or_else(|_| "admin".to_string());

        let http = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .map_err(|e| format!("Failed to create HTTP client: {e}"))?;

        Ok(Self {
            base_url,
            realm,
            admin_user,
            admin_password,
            http,
        })
    }

    async fn get_admin_token(&self) -> Result<String, String> {
        let token_url = format!(
            "{}/realms/{}/protocol/openid-connect/token",
            self.base_url, self.realm
        );

        let params = [
            ("grant_type", "password"),
            ("client_id", "admin-cli"),
            ("username", &self.admin_user),
            ("password", &self.admin_password),
        ];

        let response = self
            .http
            .post(&token_url)
            .form(&params)
            .send()
            .await
            .map_err(|e| format!("Failed to request admin token: {e}"))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(format!("Failed to get admin token ({status}): {body}"));
        }

        let token_response: TokenResponse = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse token response: {e}"))?;

        Ok(token_response.access_token)
    }

    async fn client_exists(&self, token: &str, client_id: &str) -> Result<Option<String>, String> {
        let url = format!("{}/admin/realms/{}/clients", self.base_url, self.realm);

        let response = self
            .http
            .get(&url)
            .bearer_auth(token)
            .query(&[("clientId", client_id)])
            .send()
            .await
            .map_err(|e| format!("Failed to check if client exists: {e}"))?;

        if !response.status().is_success() {
            return Err(format!("Failed to query clients: {}", response.status()));
        }

        let clients: Vec<ClientDetails> = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse clients response: {e}"))?;

        Ok(clients.first().map(|c| c.id.clone()))
    }

    async fn create_client(&self, token: &str, client_id: &str) -> Result<String, String> {
        let url = format!("{}/admin/realms/{}/clients", self.base_url, self.realm);

        let client = ClientRepresentation {
            client_id: client_id.to_string(),
            service_accounts_enabled: true,
            direct_access_grants_enabled: false,
            public_client: false,
            protocol: "openid-connect".to_string(),
            standard_flow_enabled: false,
        };

        let response = self
            .http
            .post(&url)
            .bearer_auth(token)
            .json(&client)
            .send()
            .await
            .map_err(|e| format!("Failed to create client: {e}"))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(format!("Failed to create client ({status}): {body}"));
        }

        // Get the created client's ID from the location header
        if let Some(location) = response.headers().get("location") {
            let location_str = location.to_str().map_err(|_| "Invalid location header")?;
            let client_uuid = location_str
                .split('/')
                .next_back()
                .ok_or("Could not extract client ID from location")?;
            return Ok(client_uuid.to_string());
        }

        // Fallback: query for the client
        self.client_exists(token, client_id)
            .await?
            .ok_or_else(|| "Client was created but could not retrieve its ID".to_string())
    }

    async fn get_client_secret(&self, token: &str, client_uuid: &str) -> Result<String, String> {
        let url = format!(
            "{}/admin/realms/{}/clients/{}/client-secret",
            self.base_url, self.realm, client_uuid
        );

        let response = self
            .http
            .get(&url)
            .bearer_auth(token)
            .send()
            .await
            .map_err(|e| format!("Failed to get client secret: {e}"))?;

        if !response.status().is_success() {
            return Err(format!(
                "Failed to retrieve client secret: {}",
                response.status()
            ));
        }

        let secret: ClientSecret = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse client secret: {e}"))?;

        Ok(secret.value)
    }

    async fn get_realm_management_client_id(&self, token: &str) -> Result<String, String> {
        let url = format!("{}/admin/realms/{}/clients", self.base_url, self.realm);

        // In the master realm, the management client is "master-realm"
        // In other realms, it's "realm-management"
        let client_id = if self.realm == "master" {
            "master-realm"
        } else {
            "realm-management"
        };

        let response = self
            .http
            .get(&url)
            .bearer_auth(token)
            .query(&[("clientId", client_id)])
            .send()
            .await
            .map_err(|e| format!("Failed to get {client_id} client: {e}"))?;

        if !response.status().is_success() {
            return Err(format!(
                "Failed to query {} client: {}",
                client_id,
                response.status()
            ));
        }

        let clients: Vec<ClientDetails> = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse {client_id} response: {e}"))?;

        clients
            .first()
            .map(|c| c.id.clone())
            .ok_or_else(|| format!("{client_id} client not found"))
    }

    async fn get_service_account_user_id(
        &self,
        token: &str,
        client_uuid: &str,
    ) -> Result<String, String> {
        let url = format!(
            "{}/admin/realms/{}/clients/{}/service-account-user",
            self.base_url, self.realm, client_uuid
        );

        let response = self
            .http
            .get(&url)
            .bearer_auth(token)
            .send()
            .await
            .map_err(|e| format!("Failed to get service account user: {e}"))?;

        if !response.status().is_success() {
            return Err(format!(
                "Failed to retrieve service account user: {}",
                response.status()
            ));
        }

        #[derive(Deserialize)]
        struct ServiceAccountUser {
            id: String,
        }

        let user: ServiceAccountUser = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse service account user: {e}"))?;

        Ok(user.id)
    }

    async fn get_client_roles(
        &self,
        token: &str,
        realm_mgmt_uuid: &str,
    ) -> Result<Vec<(String, String)>, String> {
        let url = format!(
            "{}/admin/realms/{}/clients/{}/roles",
            self.base_url, self.realm, realm_mgmt_uuid
        );

        let response = self
            .http
            .get(&url)
            .bearer_auth(token)
            .send()
            .await
            .map_err(|e| format!("Failed to get client roles: {e}"))?;

        if !response.status().is_success() {
            return Err(format!(
                "Failed to retrieve client roles: {}",
                response.status()
            ));
        }

        #[derive(Deserialize)]
        struct RoleRepresentation {
            id: String,
            name: String,
        }

        let roles: Vec<RoleRepresentation> = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse roles: {e}"))?;

        Ok(roles.into_iter().map(|r| (r.id, r.name)).collect())
    }

    async fn assign_client_roles(
        &self,
        token: &str,
        user_id: &str,
        realm_mgmt_uuid: &str,
        role_ids: Vec<(String, String)>,
    ) -> Result<(), String> {
        let url = format!(
            "{}/admin/realms/{}/users/{}/role-mappings/clients/{}",
            self.base_url, self.realm, user_id, realm_mgmt_uuid
        );

        #[derive(Serialize)]
        struct RoleMapping {
            id: String,
            name: String,
        }

        let mappings: Vec<RoleMapping> = role_ids
            .into_iter()
            .map(|(id, name)| RoleMapping { id, name })
            .collect();

        let response = self
            .http
            .post(&url)
            .bearer_auth(token)
            .json(&mappings)
            .send()
            .await
            .map_err(|e| format!("Failed to assign roles: {e}"))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(format!("Failed to assign roles ({status}): {body}"));
        }

        Ok(())
    }

    pub async fn setup_service_account(&self, client_id: &str) -> Result<String, String> {
        println!("Authenticating with Keycloak admin API...");
        let token = self.get_admin_token().await?;

        println!("Checking if client '{client_id}' exists...");
        let client_uuid = match self.client_exists(&token, client_id).await? {
            Some(id) => {
                println!("✓ Client already exists");
                id
            }
            None => {
                println!("Creating client '{client_id}'...");
                let id = self.create_client(&token, client_id).await?;
                println!("✓ Client created successfully");
                id
            }
        };

        println!("Configuring service account permissions...");

        // Get realm-management client ID
        let realm_mgmt_uuid = self.get_realm_management_client_id(&token).await?;

        // Get service account user ID
        let sa_user_id = self
            .get_service_account_user_id(&token, &client_uuid)
            .await?;

        // Get available roles from realm-management client
        let roles = self.get_client_roles(&token, &realm_mgmt_uuid).await?;

        // Find manage-users and view-users roles
        let required_roles: Vec<(String, String)> = roles
            .into_iter()
            .filter(|(_, name)| name == "manage-users" || name == "view-users")
            .collect();

        if required_roles.is_empty() {
            return Err("Required roles (manage-users, view-users) not found".to_string());
        }

        // Assign roles to service account
        self.assign_client_roles(&token, &sa_user_id, &realm_mgmt_uuid, required_roles)
            .await?;
        println!("✓ Assigned manage-users and view-users roles");

        println!("Retrieving client secret...");
        let secret = self.get_client_secret(&token, &client_uuid).await?;

        Ok(secret)
    }
}
