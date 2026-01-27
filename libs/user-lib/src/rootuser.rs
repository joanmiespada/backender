/// Root user initialization module
///
/// Handles creation of the root administrative user in the database.
/// This module is designed to be called during application initialization.
use crate::entities::{Role, User};
use crate::errors_service::UserServiceError;
use crate::repository::traits::{
    RoleRepositoryTrait, UserRepositoryTrait, UserRoleRepositoryTrait,
};
use uuid::Uuid;

/// Configuration for root user initialization
#[derive(Debug, Clone)]
pub struct RootUserConfig {
    pub email: String,
    pub first_name: String,
    pub last_name: String,
    /// Keycloak ID for the root user (obtained from Keycloak after user creation)
    pub keycloak_id: String,
}

impl RootUserConfig {
    /// Load root user configuration from environment variables
    pub fn from_env() -> Result<Self, String> {
        let email = std::env::var("ROOT_USER_EMAIL")
            .map_err(|_| "ROOT_USER_EMAIL environment variable not set")?;

        let first_name =
            std::env::var("ROOT_USER_FIRST_NAME").unwrap_or_else(|_| "Root".to_string());

        let last_name = std::env::var("ROOT_USER_LAST_NAME").unwrap_or_else(|_| "User".to_string());

        if email.is_empty() {
            return Err("ROOT_USER_EMAIL cannot be empty".to_string());
        }

        Ok(Self {
            email,
            first_name,
            last_name,
            keycloak_id: String::new(), // Will be set after Keycloak creation
        })
    }

    /// Get the root user password from environment
    pub fn password_from_env() -> Result<String, String> {
        let password = std::env::var("ROOT_USER_PASSWORD")
            .map_err(|_| "ROOT_USER_PASSWORD environment variable not set")?;

        if password.is_empty() {
            return Err("ROOT_USER_PASSWORD cannot be empty".to_string());
        }

        Ok(password)
    }
}

/// Initialize root user in the database
///
/// This function:
/// 1. Creates the root user record in the database
/// 2. Finds or creates the 'admin' role
/// 3. Assigns the admin role to the root user
///
/// Note: The Keycloak user must be created BEFORE calling this function,
/// and the keycloak_id must be provided in the config.
pub async fn initialize_root_user<U, R, UR>(
    user_repo: &U,
    role_repo: &R,
    user_role_repo: &UR,
    config: &RootUserConfig,
) -> Result<User, UserServiceError>
where
    U: UserRepositoryTrait,
    R: RoleRepositoryTrait,
    UR: UserRoleRepositoryTrait,
{
    if config.keycloak_id.is_empty() {
        return Err(UserServiceError::Validation(
            "Keycloak ID must be set before creating local user record".to_string(),
        ));
    }

    // Check if root user already exists by keycloak_id
    if let Some(existing) = user_repo
        .get_user_by_keycloak_id(&config.keycloak_id)
        .await
        .map_err(|e| UserServiceError::Internal(e.into()))?
    {
        tracing::info!(
            user_id = %existing.id,
            keycloak_id = %existing.keycloak_id,
            "Root user already exists in database"
        );

        // Parse the UUID
        let user_id = Uuid::parse_str(&existing.id)
            .map_err(|e| UserServiceError::InvalidUuid(e.to_string()))?;

        // Get roles for the user
        let role_rows = role_repo
            .get_roles_for_user(user_id)
            .await
            .map_err(|e| UserServiceError::Internal(e.into()))?;

        let roles: Vec<Role> = role_rows
            .into_iter()
            .filter_map(|row| {
                let id = Uuid::parse_str(&row.id).ok()?;
                Some(Role { id, name: row.name })
            })
            .collect();

        return Ok(User {
            id: user_id,
            keycloak_id: existing.keycloak_id,
            roles,
        });
    }

    // Create user in database
    tracing::info!(
        keycloak_id = %config.keycloak_id,
        email = %config.email,
        "Creating root user in database"
    );

    let user_row = user_repo
        .create_user(&config.keycloak_id)
        .await
        .map_err(|e| UserServiceError::Internal(e.into()))?;

    let user_id =
        Uuid::parse_str(&user_row.id).map_err(|e| UserServiceError::InvalidUuid(e.to_string()))?;

    // Find or verify admin role exists
    tracing::info!("Looking up admin role");
    let admin_role_id = find_admin_role(role_repo).await?;

    // Assign admin role to root user
    tracing::info!(
        user_id = %user_id,
        role_id = %admin_role_id,
        "Assigning admin role to root user"
    );

    user_role_repo
        .assign_role(&user_id.to_string(), &admin_role_id.to_string())
        .await
        .map_err(|e| match e {
            crate::repository::errors::UserRepositoryError::UserAlreadyHasRole => {
                // Already has role, that's fine
                tracing::info!("Root user already has admin role");
                UserServiceError::UserAlreadyHasRole
            }
            _ => UserServiceError::Internal(e.into()),
        })?;

    tracing::info!(
        user_id = %user_id,
        keycloak_id = %config.keycloak_id,
        "Root user initialized successfully"
    );

    Ok(User {
        id: user_id,
        keycloak_id: user_row.keycloak_id,
        roles: vec![Role {
            id: admin_role_id,
            name: "admin".to_string(),
        }],
    })
}

/// Find the admin role ID
/// The admin role should be seeded during migrations
async fn find_admin_role<R: RoleRepositoryTrait>(role_repo: &R) -> Result<Uuid, UserServiceError> {
    use crate::entities::PaginationParams;

    // Get all roles and find admin
    let (roles, _) = role_repo
        .get_roles_paginated(PaginationParams {
            page: 1,
            page_size: 100,
        })
        .await
        .map_err(|e| UserServiceError::Internal(e.into()))?;

    let admin_role = roles
        .iter()
        .find(|r| r.name.to_lowercase() == "admin")
        .ok_or_else(|| UserServiceError::NotFound)?;

    Uuid::parse_str(&admin_role.id).map_err(|e| UserServiceError::InvalidUuid(e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_root_user_config_validation() {
        std::env::set_var("ROOT_USER_EMAIL", "test@example.com");
        std::env::set_var("ROOT_USER_FIRST_NAME", "Test");
        std::env::set_var("ROOT_USER_LAST_NAME", "User");

        let config = RootUserConfig::from_env().unwrap();
        assert_eq!(config.email, "test@example.com");
        assert_eq!(config.first_name, "Test");
        assert_eq!(config.last_name, "User");
    }

    #[test]
    fn test_root_user_config_defaults() {
        std::env::set_var("ROOT_USER_EMAIL", "test@example.com");
        std::env::remove_var("ROOT_USER_FIRST_NAME");
        std::env::remove_var("ROOT_USER_LAST_NAME");

        let config = RootUserConfig::from_env().unwrap();
        assert_eq!(config.first_name, "Root");
        assert_eq!(config.last_name, "User");
    }

    #[test]
    fn test_root_user_config_missing_email() {
        std::env::remove_var("ROOT_USER_EMAIL");

        let result = RootUserConfig::from_env();
        assert!(result.is_err());
    }

    #[test]
    fn test_root_user_config_empty_email() {
        std::env::set_var("ROOT_USER_EMAIL", "");

        let result = RootUserConfig::from_env();
        assert!(result.is_err());
    }
}
