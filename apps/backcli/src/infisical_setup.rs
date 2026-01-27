//! Infisical setup for local development
//!
//! This module handles the automated setup of Infisical for local development:
//! 1. Create admin user (first signup)
//! 2. Create organization
//! 3. Create project
//! 4. Create machine identity with universal auth
//! 5. Return client credentials for .env.local

use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;

const DEFAULT_INFISICAL_URL: &str = "http://localhost:8888";
const DEFAULT_ADMIN_EMAIL: &str = "admin@backender.local";
const DEFAULT_ADMIN_PASSWORD: &str = "AdminPassword123!";
const DEFAULT_ORG_NAME: &str = "Backender";
const DEFAULT_PROJECT_NAME: &str = "backender-secrets";
const DEFAULT_ENVIRONMENT: &str = "dev";

// ============================================================================
// API Request/Response Types
// ============================================================================

#[derive(Debug, Serialize)]
struct SignupRequest {
    email: String,
    password: String,
    #[serde(rename = "firstName")]
    first_name: String,
    #[serde(rename = "lastName")]
    last_name: String,
}

#[derive(Debug, Serialize)]
struct LoginRequest {
    email: String,
    password: String,
}

#[derive(Debug, Deserialize)]
struct LoginResponse {
    token: String,
}

#[derive(Debug, Serialize)]
struct CreateOrgRequest {
    name: String,
}

#[derive(Debug, Deserialize)]
struct Organization {
    id: String,
}

#[derive(Debug, Deserialize)]
struct CreateOrgResponse {
    organization: Organization,
}

#[derive(Debug, Serialize)]
struct CreateProjectRequest {
    #[serde(rename = "projectName")]
    project_name: String,
    #[serde(rename = "organizationId")]
    organization_id: String,
}

#[derive(Debug, Deserialize)]
struct Project {
    id: String,
}

#[derive(Debug, Deserialize)]
struct CreateProjectResponse {
    project: Project,
}

#[derive(Debug, Serialize)]
struct CreateIdentityRequest {
    name: String,
    #[serde(rename = "organizationId")]
    organization_id: String,
    role: String,
}

#[derive(Debug, Deserialize)]
struct Identity {
    id: String,
}

#[derive(Debug, Deserialize)]
struct CreateIdentityResponse {
    identity: Identity,
}

#[derive(Debug, Serialize)]
struct CreateUniversalAuthRequest {
    #[serde(rename = "identityId")]
    identity_id: String,
    #[serde(rename = "clientSecretTrustedIps")]
    client_secret_trusted_ips: Vec<TrustedIp>,
    #[serde(rename = "accessTokenTrustedIps")]
    access_token_trusted_ips: Vec<TrustedIp>,
    #[serde(rename = "accessTokenTTL")]
    access_token_ttl: u64,
    #[serde(rename = "accessTokenMaxTTL")]
    access_token_max_ttl: u64,
    #[serde(rename = "accessTokenNumUsesLimit")]
    access_token_num_uses_limit: u64,
}

#[derive(Debug, Serialize, Clone)]
struct TrustedIp {
    #[serde(rename = "ipAddress")]
    ip_address: String,
}

#[derive(Debug, Deserialize)]
struct UniversalAuthResponse {
    #[serde(rename = "clientId")]
    client_id: String,
}

#[derive(Debug, Serialize)]
struct CreateClientSecretRequest {
    #[serde(rename = "identityId")]
    identity_id: String,
    description: String,
    #[serde(rename = "numUsesLimit")]
    num_uses_limit: u64,
    ttl: u64,
}

#[derive(Debug, Deserialize)]
struct ClientSecretResponse {
    #[serde(rename = "clientSecret")]
    client_secret: String,
}

#[derive(Debug, Serialize)]
struct AddIdentityToProjectRequest {
    #[serde(rename = "identityId")]
    identity_id: String,
    role: String,
}

#[derive(Debug, Serialize)]
struct CreateSecretRequest {
    #[serde(rename = "workspaceId")]
    workspace_id: String,
    environment: String,
    #[serde(rename = "secretKey")]
    secret_key: String,
    #[serde(rename = "secretValue")]
    secret_value: String,
    #[serde(rename = "secretPath")]
    secret_path: String,
    #[serde(rename = "type")]
    secret_type: String,
}

#[derive(Debug, Serialize)]
struct UpdateSecretRequest {
    #[serde(rename = "workspaceId")]
    workspace_id: String,
    environment: String,
    #[serde(rename = "secretValue")]
    secret_value: String,
    #[serde(rename = "secretPath")]
    secret_path: String,
}

// ============================================================================
// Infisical Setup Implementation
// ============================================================================

pub struct InfisicalSetup {
    base_url: String,
    admin_email: String,
    admin_password: String,
    org_name: String,
    project_name: String,
    environment: String,
    http: Client,
}

pub struct InfisicalCredentials {
    pub url: String,
    pub client_id: String,
    pub client_secret: String,
    pub project_id: String,
    pub environment: String,
}

impl InfisicalSetup {
    pub fn from_env() -> Result<Self, String> {
        let base_url = std::env::var("INFISICAL_SETUP_URL")
            .unwrap_or_else(|_| DEFAULT_INFISICAL_URL.to_string());

        let admin_email = std::env::var("INFISICAL_ADMIN_EMAIL")
            .unwrap_or_else(|_| DEFAULT_ADMIN_EMAIL.to_string());

        let admin_password = std::env::var("INFISICAL_ADMIN_PASSWORD")
            .unwrap_or_else(|_| DEFAULT_ADMIN_PASSWORD.to_string());

        let org_name =
            std::env::var("INFISICAL_ORG_NAME").unwrap_or_else(|_| DEFAULT_ORG_NAME.to_string());

        let project_name = std::env::var("INFISICAL_PROJECT_NAME")
            .unwrap_or_else(|_| DEFAULT_PROJECT_NAME.to_string());

        let environment = std::env::var("INFISICAL_ENVIRONMENT")
            .unwrap_or_else(|_| DEFAULT_ENVIRONMENT.to_string());

        let http = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .map_err(|e| format!("Failed to create HTTP client: {e}"))?;

        Ok(Self {
            base_url,
            admin_email,
            admin_password,
            org_name,
            project_name,
            environment,
            http,
        })
    }

    /// Try to signup a new user (will fail silently if user exists)
    async fn signup(&self) -> Result<(), String> {
        let url = format!("{}/api/v1/signup", self.base_url);

        let request = SignupRequest {
            email: self.admin_email.clone(),
            password: self.admin_password.clone(),
            first_name: "Admin".to_string(),
            last_name: "User".to_string(),
        };

        let response = self
            .http
            .post(&url)
            .json(&request)
            .send()
            .await
            .map_err(|e| format!("Failed to signup: {e}"))?;

        // 200 = success, 400 = user already exists (both are OK)
        if response.status().is_success() || response.status() == reqwest::StatusCode::BAD_REQUEST {
            Ok(())
        } else {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            Err(format!("Signup failed ({status}): {body}"))
        }
    }

    /// Login and get access token
    async fn login(&self) -> Result<String, String> {
        let url = format!("{}/api/v1/auth/login1", self.base_url);

        let request = LoginRequest {
            email: self.admin_email.clone(),
            password: self.admin_password.clone(),
        };

        let response = self
            .http
            .post(&url)
            .json(&request)
            .send()
            .await
            .map_err(|e| format!("Failed to login: {e}"))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(format!("Login failed ({status}): {body}"));
        }

        let login_response: LoginResponse = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse login response: {e}"))?;

        Ok(login_response.token)
    }

    /// Get existing organizations or create a new one
    async fn get_or_create_organization(&self, token: &str) -> Result<String, String> {
        // First, try to get existing organizations
        let url = format!("{}/api/v2/organizations", self.base_url);

        let response = self
            .http
            .get(&url)
            .header("Authorization", format!("Bearer {token}"))
            .send()
            .await
            .map_err(|e| format!("Failed to get organizations: {e}"))?;

        if response.status().is_success() {
            #[derive(Deserialize)]
            struct OrgList {
                organizations: Vec<Organization>,
            }

            let orgs: OrgList = response
                .json()
                .await
                .map_err(|e| format!("Failed to parse organizations: {e}"))?;

            if let Some(org) = orgs.organizations.first() {
                return Ok(org.id.clone());
            }
        }

        // Create new organization
        let create_url = format!("{}/api/v2/organizations", self.base_url);
        let request = CreateOrgRequest {
            name: self.org_name.clone(),
        };

        let response = self
            .http
            .post(&create_url)
            .header("Authorization", format!("Bearer {token}"))
            .json(&request)
            .send()
            .await
            .map_err(|e| format!("Failed to create organization: {e}"))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(format!("Failed to create organization ({status}): {body}"));
        }

        let org_response: CreateOrgResponse = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse organization response: {e}"))?;

        Ok(org_response.organization.id)
    }

    /// Get existing project or create a new one
    async fn get_or_create_project(&self, token: &str, org_id: &str) -> Result<String, String> {
        // First, try to get existing projects in the organization
        let url = format!(
            "{}/api/v2/organizations/{}/workspaces",
            self.base_url, org_id
        );

        let response = self
            .http
            .get(&url)
            .header("Authorization", format!("Bearer {token}"))
            .send()
            .await
            .map_err(|e| format!("Failed to get projects: {e}"))?;

        if response.status().is_success() {
            #[derive(Deserialize)]
            struct ProjectList {
                workspaces: Vec<Project>,
            }

            let projects: ProjectList = response
                .json()
                .await
                .map_err(|e| format!("Failed to parse projects: {e}"))?;

            if let Some(project) = projects.workspaces.first() {
                return Ok(project.id.clone());
            }
        }

        // Create new project
        let create_url = format!("{}/api/v2/workspace", self.base_url);
        let request = CreateProjectRequest {
            project_name: self.project_name.clone(),
            organization_id: org_id.to_string(),
        };

        let response = self
            .http
            .post(&create_url)
            .header("Authorization", format!("Bearer {token}"))
            .json(&request)
            .send()
            .await
            .map_err(|e| format!("Failed to create project: {e}"))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(format!("Failed to create project ({status}): {body}"));
        }

        let project_response: CreateProjectResponse = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse project response: {e}"))?;

        Ok(project_response.project.id)
    }

    /// Create a machine identity for the project
    async fn create_machine_identity(&self, token: &str, org_id: &str) -> Result<String, String> {
        let url = format!("{}/api/v1/identities", self.base_url);

        let request = CreateIdentityRequest {
            name: "user-api-service".to_string(),
            organization_id: org_id.to_string(),
            role: "admin".to_string(),
        };

        let response = self
            .http
            .post(&url)
            .header("Authorization", format!("Bearer {token}"))
            .json(&request)
            .send()
            .await
            .map_err(|e| format!("Failed to create identity: {e}"))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(format!("Failed to create identity ({status}): {body}"));
        }

        let identity_response: CreateIdentityResponse = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse identity response: {e}"))?;

        Ok(identity_response.identity.id)
    }

    /// Setup universal auth for the identity
    async fn setup_universal_auth(&self, token: &str, identity_id: &str) -> Result<String, String> {
        let url = format!(
            "{}/api/v1/auth/universal-auth/identities/{}",
            self.base_url, identity_id
        );

        // Allow all IPs for local development
        let trusted_ip = TrustedIp {
            ip_address: "0.0.0.0/0".to_string(),
        };

        let request = CreateUniversalAuthRequest {
            identity_id: identity_id.to_string(),
            client_secret_trusted_ips: vec![trusted_ip.clone()],
            access_token_trusted_ips: vec![trusted_ip],
            access_token_ttl: 7200,         // 2 hours
            access_token_max_ttl: 86400,    // 24 hours
            access_token_num_uses_limit: 0, // unlimited
        };

        let response = self
            .http
            .post(&url)
            .header("Authorization", format!("Bearer {token}"))
            .json(&request)
            .send()
            .await
            .map_err(|e| format!("Failed to setup universal auth: {e}"))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(format!("Failed to setup universal auth ({status}): {body}"));
        }

        let auth_response: UniversalAuthResponse = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse universal auth response: {e}"))?;

        Ok(auth_response.client_id)
    }

    /// Create a client secret for the identity
    async fn create_client_secret(&self, token: &str, identity_id: &str) -> Result<String, String> {
        let url = format!(
            "{}/api/v1/auth/universal-auth/identities/{}/client-secrets",
            self.base_url, identity_id
        );

        let request = CreateClientSecretRequest {
            identity_id: identity_id.to_string(),
            description: "user-api-service credentials".to_string(),
            num_uses_limit: 0, // unlimited
            ttl: 0,            // never expires
        };

        let response = self
            .http
            .post(&url)
            .header("Authorization", format!("Bearer {token}"))
            .json(&request)
            .send()
            .await
            .map_err(|e| format!("Failed to create client secret: {e}"))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(format!("Failed to create client secret ({status}): {body}"));
        }

        let secret_response: ClientSecretResponse = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse client secret response: {e}"))?;

        Ok(secret_response.client_secret)
    }

    /// Add identity to project with read/write access
    async fn add_identity_to_project(
        &self,
        token: &str,
        project_id: &str,
        identity_id: &str,
    ) -> Result<(), String> {
        let url = format!(
            "{}/api/v2/workspace/{}/identity-memberships/{}",
            self.base_url, project_id, identity_id
        );

        let request = AddIdentityToProjectRequest {
            identity_id: identity_id.to_string(),
            role: "admin".to_string(),
        };

        let response = self
            .http
            .post(&url)
            .header("Authorization", format!("Bearer {token}"))
            .json(&request)
            .send()
            .await
            .map_err(|e| format!("Failed to add identity to project: {e}"))?;

        // 200 = success, 400 = already added (both OK)
        if response.status().is_success() || response.status() == reqwest::StatusCode::BAD_REQUEST {
            Ok(())
        } else {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            Err(format!(
                "Failed to add identity to project ({status}): {body}"
            ))
        }
    }

    /// Main setup function - creates everything needed and returns credentials
    pub async fn setup(&self) -> Result<InfisicalCredentials, String> {
        println!("Setting up Infisical for local development...\n");

        // Step 1: Signup (or verify user exists)
        println!("Creating admin user...");
        self.signup().await?;
        println!("  Admin user ready");

        // Step 2: Login
        println!("Authenticating...");
        let token = self.login().await?;
        println!("  Authenticated successfully");

        // Step 3: Get or create organization
        println!("Setting up organization...");
        let org_id = self.get_or_create_organization(&token).await?;
        println!("  Organization ID: {org_id}");

        // Step 4: Get or create project
        println!("Setting up project...");
        let project_id = self.get_or_create_project(&token, &org_id).await?;
        println!("  Project ID: {project_id}");

        // Step 5: Create machine identity
        println!("Creating machine identity...");
        let identity_id = self.create_machine_identity(&token, &org_id).await?;
        println!("  Identity ID: {identity_id}");

        // Step 6: Setup universal auth
        println!("Setting up universal auth...");
        let client_id = self.setup_universal_auth(&token, &identity_id).await?;
        println!("  Client ID: {client_id}");

        // Step 7: Create client secret
        println!("Creating client secret...");
        let client_secret = self.create_client_secret(&token, &identity_id).await?;
        println!("  Client secret created");

        // Step 8: Add identity to project
        println!("Adding identity to project...");
        self.add_identity_to_project(&token, &project_id, &identity_id)
            .await?;
        println!("  Identity added to project");

        Ok(InfisicalCredentials {
            url: self.base_url.clone(),
            client_id,
            client_secret,
            project_id,
            environment: self.environment.clone(),
        })
    }

    /// Store a secret in Infisical
    /// Creates the secret if it doesn't exist, updates it if it does
    pub async fn store_secret(&self, key: &str, value: &str) -> Result<(), String> {
        // Login first
        let token = self.login().await?;

        // Get organization
        let org_id = self.get_or_create_organization(&token).await?;

        // Get project
        let project_id = self.get_or_create_project(&token, &org_id).await?;

        // Try to create the secret first
        let create_url = format!("{}/api/v3/secrets/raw/{}", self.base_url, key);

        let create_request = CreateSecretRequest {
            workspace_id: project_id.clone(),
            environment: self.environment.clone(),
            secret_key: key.to_string(),
            secret_value: value.to_string(),
            secret_path: "/".to_string(),
            secret_type: "shared".to_string(),
        };

        let response = self
            .http
            .post(&create_url)
            .header("Authorization", format!("Bearer {token}"))
            .json(&create_request)
            .send()
            .await
            .map_err(|e| format!("Failed to create secret: {e}"))?;

        if response.status().is_success() {
            return Ok(());
        }

        // If creation failed (likely because secret exists), try to update
        let update_url = format!("{}/api/v3/secrets/raw/{}", self.base_url, key);

        let update_request = UpdateSecretRequest {
            workspace_id: project_id,
            environment: self.environment.clone(),
            secret_value: value.to_string(),
            secret_path: "/".to_string(),
        };

        let response = self
            .http
            .patch(&update_url)
            .header("Authorization", format!("Bearer {token}"))
            .json(&update_request)
            .send()
            .await
            .map_err(|e| format!("Failed to update secret: {e}"))?;

        if response.status().is_success() {
            Ok(())
        } else {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            Err(format!("Failed to store secret ({status}): {body}"))
        }
    }
}
