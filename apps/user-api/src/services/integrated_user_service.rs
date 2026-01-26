use secrecy::Secret;
use std::sync::Arc;
use uuid::Uuid;

use user_lib::entities::{PaginatedResult, PaginationParams, Role};
use user_lib::errors_service::UserServiceError;
use user_lib::repository::traits::{RoleRepositoryTrait, UserRepositoryTrait, UserRoleRepositoryTrait};

use crate::cache::{CachedUserService, RedisCache};
use crate::keycloak::{FullUser, KeycloakClient, KeycloakError, KeycloakUser};

/// Cache key for Keycloak profiles
fn keycloak_profile_key(keycloak_id: &str) -> String {
    format!("user-api:kc:profile:{}", keycloak_id)
}

/// Pattern for all Keycloak profile keys
pub fn keycloak_profiles_pattern() -> String {
    "user-api:kc:profile:*".to_string()
}

/// Request for creating a user
pub struct CreateUserRequest {
    pub email: String,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub password: Option<Secret<String>>,
}

/// Request for updating a user profile
pub struct UpdateUserRequest {
    pub first_name: Option<String>,
    pub last_name: Option<String>,
}

/// Service error that combines user service and keycloak errors
#[derive(Debug)]
pub enum IntegratedServiceError {
    User(UserServiceError),
    Keycloak(KeycloakError),
}

impl std::fmt::Display for IntegratedServiceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IntegratedServiceError::User(e) => write!(f, "{}", e),
            IntegratedServiceError::Keycloak(e) => write!(f, "{}", e),
        }
    }
}

impl std::error::Error for IntegratedServiceError {}

impl From<UserServiceError> for IntegratedServiceError {
    fn from(err: UserServiceError) -> Self {
        IntegratedServiceError::User(err)
    }
}

impl From<KeycloakError> for IntegratedServiceError {
    fn from(err: KeycloakError) -> Self {
        IntegratedServiceError::Keycloak(err)
    }
}

/// Integrated user service that wraps CachedUserService and KeycloakClient
pub struct IntegratedUserService<U, R, UR>
where
    U: UserRepositoryTrait + Send + Sync + 'static,
    R: RoleRepositoryTrait + Send + Sync + 'static,
    UR: UserRoleRepositoryTrait + Send + Sync + 'static,
{
    inner: Arc<CachedUserService<U, R, UR>>,
    keycloak: Arc<KeycloakClient>,
    redis: RedisCache,
}

impl<U, R, UR> IntegratedUserService<U, R, UR>
where
    U: UserRepositoryTrait + Send + Sync + 'static,
    R: RoleRepositoryTrait + Send + Sync + 'static,
    UR: UserRoleRepositoryTrait + Send + Sync + 'static,
{
    pub fn new(
        inner: Arc<CachedUserService<U, R, UR>>,
        keycloak: Arc<KeycloakClient>,
        redis: RedisCache,
    ) -> Self {
        Self {
            inner,
            keycloak,
            redis,
        }
    }

    /// Get cached Keycloak profile or fetch from Keycloak
    async fn get_keycloak_profile(&self, keycloak_id: &str) -> Result<Option<KeycloakUser>, KeycloakError> {
        if !self.keycloak.is_configured() {
            return Ok(None);
        }

        let cache_key = keycloak_profile_key(keycloak_id);

        // Try cache first
        if self.redis.is_enabled() {
            if let Some(profile) = self.redis.get::<KeycloakUser>(&cache_key).await {
                return Ok(Some(profile));
            }
        }

        // Fetch from Keycloak
        let profile = self.keycloak.get_user_by_id(keycloak_id).await?;

        // Cache if found
        if let Some(ref p) = profile {
            if self.redis.is_enabled() {
                self.redis.set(&cache_key, p, self.keycloak.profile_cache_ttl()).await;
            }
        }

        Ok(profile)
    }

    /// Invalidate Keycloak profile cache
    async fn invalidate_keycloak_cache(&self, keycloak_id: &str) {
        if self.redis.is_enabled() {
            let cache_key = keycloak_profile_key(keycloak_id);
            self.redis.delete(&cache_key).await;
        }
    }

    /// Merge local user with Keycloak profile
    fn merge_user(&self, local: user_lib::entities::User, kc_profile: Option<KeycloakUser>) -> FullUser {
        match kc_profile {
            Some(kc) => FullUser {
                id: local.id,
                keycloak_id: local.keycloak_id,
                name: kc.display_name(),
                email: kc.email,
                roles: local.roles,
                email_verified: kc.email_verified,
                enabled: kc.enabled,
            },
            None => FullUser {
                id: local.id,
                keycloak_id: local.keycloak_id.clone(),
                name: format!("User {}", &local.keycloak_id[..8.min(local.keycloak_id.len())]),
                email: None,
                roles: local.roles,
                email_verified: false,
                enabled: true,
            },
        }
    }

    // ========== User Operations ==========

    /// Get a user by ID with merged Keycloak profile
    pub async fn get_user(&self, user_id: Uuid) -> Result<Option<FullUser>, IntegratedServiceError> {
        let local = self.inner.get_user(user_id).await?;

        match local {
            Some(user) => {
                let kc_profile = self.get_keycloak_profile(&user.keycloak_id).await.ok().flatten();
                Ok(Some(self.merge_user(user, kc_profile)))
            }
            None => Ok(None),
        }
    }

    /// Get all users with merged Keycloak profiles
    pub async fn get_users(
        &self,
        pagination: PaginationParams,
    ) -> Result<PaginatedResult<FullUser>, IntegratedServiceError> {
        let result = self.inner.get_users(pagination).await?;

        let mut full_users = Vec::with_capacity(result.items.len());
        for user in result.items {
            let kc_profile = self.get_keycloak_profile(&user.keycloak_id).await.ok().flatten();
            full_users.push(self.merge_user(user, kc_profile));
        }

        Ok(PaginatedResult {
            items: full_users,
            total: result.total,
            page: result.page,
            page_size: result.page_size,
            total_pages: result.total_pages,
        })
    }

    /// Create a new user in Keycloak and local DB
    /// Implements compensation transaction: if local DB creation fails, rolls back Keycloak user
    pub async fn create_user(&self, request: CreateUserRequest) -> Result<FullUser, IntegratedServiceError> {
        // Create in Keycloak first
        let keycloak_id = self.keycloak.create_user(
            &request.email,
            request.first_name.as_deref(),
            request.last_name.as_deref(),
            request.password.as_ref(),
        ).await?;

        // Create local record with compensation on failure
        let local = match self.inner.create_user(&keycloak_id).await {
            Ok(user) => user,
            Err(e) => {
                // CRITICAL: Rollback Keycloak user creation
                tracing::error!(
                    keycloak_id = %keycloak_id,
                    email = %request.email,
                    error = ?e,
                    "Failed to create local user record - rolling back Keycloak user"
                );

                // Attempt to delete the Keycloak user to prevent orphaned records
                if let Err(rollback_err) = self.keycloak.delete_user(&keycloak_id).await {
                    tracing::error!(
                        keycloak_id = %keycloak_id,
                        error = ?rollback_err,
                        "CRITICAL: Failed to rollback Keycloak user - ORPHANED USER requires manual cleanup"
                    );
                } else {
                    tracing::info!(
                        keycloak_id = %keycloak_id,
                        "Successfully rolled back Keycloak user creation"
                    );
                }

                return Err(e.into());
            }
        };

        // Fetch the profile from Keycloak
        let kc_profile = self.get_keycloak_profile(&keycloak_id).await.ok().flatten();

        Ok(self.merge_user(local, kc_profile))
    }

    /// Update a user's profile in Keycloak
    pub async fn update_user(
        &self,
        user_id: Uuid,
        request: UpdateUserRequest,
    ) -> Result<FullUser, IntegratedServiceError> {
        // Get local user to find keycloak_id
        let local = self.inner.get_user(user_id).await?
            .ok_or(IntegratedServiceError::User(UserServiceError::NotFound))?;

        // Update in Keycloak
        self.keycloak.update_user(
            &local.keycloak_id,
            request.first_name.as_deref(),
            request.last_name.as_deref(),
        ).await?;

        // Invalidate KC cache
        self.invalidate_keycloak_cache(&local.keycloak_id).await;

        // Fetch fresh profile
        let kc_profile = self.get_keycloak_profile(&local.keycloak_id).await.ok().flatten();

        Ok(self.merge_user(local, kc_profile))
    }

    /// Delete a user from both Keycloak and local DB
    pub async fn delete_user(&self, user_id: Uuid) -> Result<(), IntegratedServiceError> {
        // Get local user to find keycloak_id
        let local = self.inner.get_user(user_id).await?
            .ok_or(IntegratedServiceError::User(UserServiceError::NotFound))?;

        // Delete from Keycloak
        self.keycloak.delete_user(&local.keycloak_id).await?;

        // Invalidate KC cache
        self.invalidate_keycloak_cache(&local.keycloak_id).await;

        // Delete from local DB
        self.inner.delete_user(user_id).await?;

        Ok(())
    }

    /// Sync a user from Keycloak - creates local record if not exists
    pub async fn sync_from_keycloak(&self, keycloak_id: &str) -> Result<FullUser, IntegratedServiceError> {
        // Check if local record exists
        let existing = self.inner.get_user_by_keycloak_id(keycloak_id).await?;

        let local = match existing {
            Some(user) => user,
            None => {
                // Create local record
                self.inner.create_user(keycloak_id).await?
            }
        };

        // Fetch Keycloak profile
        let kc_profile = self.get_keycloak_profile(keycloak_id).await.ok().flatten();

        Ok(self.merge_user(local, kc_profile))
    }

    // ========== Role Operations (passthrough) ==========

    pub async fn get_role(&self, role_id: Uuid) -> Result<Option<Role>, IntegratedServiceError> {
        Ok(self.inner.get_role(role_id).await?)
    }

    pub async fn get_roles(
        &self,
        pagination: PaginationParams,
    ) -> Result<PaginatedResult<Role>, IntegratedServiceError> {
        Ok(self.inner.get_roles(pagination).await?)
    }

    pub async fn create_role(&self, name: &str) -> Result<Role, IntegratedServiceError> {
        Ok(self.inner.create_role(name).await?)
    }

    pub async fn update_role(&self, role_id: Uuid, name: &str) -> Result<Role, IntegratedServiceError> {
        Ok(self.inner.update_role(role_id, name).await?)
    }

    pub async fn delete_role(&self, role_id: Uuid) -> Result<(), IntegratedServiceError> {
        Ok(self.inner.delete_role(role_id).await?)
    }

    pub async fn assign_role(&self, user_id: Uuid, role_id: Uuid) -> Result<(), IntegratedServiceError> {
        self.inner.assign_role(user_id, role_id).await?;

        // Also invalidate user caches since role assignment affects the user
        if let Ok(Some(user)) = self.inner.get_user(user_id).await {
            self.invalidate_keycloak_cache(&user.keycloak_id).await;
        }

        Ok(())
    }

    pub async fn unassign_role(&self, user_id: Uuid, role_id: Uuid) -> Result<(), IntegratedServiceError> {
        self.inner.unassign_role(user_id, role_id).await?;

        // Also invalidate user caches since role unassignment affects the user
        if let Ok(Some(user)) = self.inner.get_user(user_id).await {
            self.invalidate_keycloak_cache(&user.keycloak_id).await;
        }

        Ok(())
    }
}
