use std::sync::Arc;
use uuid::Uuid;

use user_lib::entities::{PaginatedResult, PaginationParams, Role, User};
use user_lib::errors_service::UserServiceError;
use user_lib::repository::traits::{RoleRepositoryTrait, UserRepositoryTrait, UserRoleRepositoryTrait};
use user_lib::user_service::UserService;

use super::client::RedisCache;
use super::config::CacheConfig;
use super::keys;

#[derive(Clone, Debug)]
pub struct CachedUserService<U, R, UR>
where
    U: UserRepositoryTrait + Send + Sync + 'static,
    R: RoleRepositoryTrait + Send + Sync + 'static,
    UR: UserRoleRepositoryTrait + Send + Sync + 'static,
{
    inner: Arc<UserService<U, R, UR>>,
    cache: RedisCache,
    config: CacheConfig,
}

impl<U, R, UR> CachedUserService<U, R, UR>
where
    U: UserRepositoryTrait + Send + Sync + 'static,
    R: RoleRepositoryTrait + Send + Sync + 'static,
    UR: UserRoleRepositoryTrait + Send + Sync + 'static,
{
    pub fn new(inner: Arc<UserService<U, R, UR>>, cache: RedisCache, config: CacheConfig) -> Self {
        Self {
            inner,
            cache,
            config,
        }
    }

    // ========== User Read Operations ==========

    pub async fn get_user(&self, user_id: Uuid) -> Result<Option<User>, UserServiceError> {
        if !self.cache.is_enabled() {
            return self.inner.get_user(user_id).await;
        }

        let cache_key = keys::user_key(user_id);

        // Try cache first
        if let Some(user) = self.cache.get::<User>(&cache_key).await {
            return Ok(Some(user));
        }

        // Cache miss - fetch from DB
        let result = self.inner.get_user(user_id).await?;

        // Cache the result if found
        if let Some(ref user) = result {
            self.cache.set(&cache_key, user, self.config.user_ttl).await;
        }

        Ok(result)
    }

    pub async fn get_users(
        &self,
        pagination: PaginationParams,
    ) -> Result<PaginatedResult<User>, UserServiceError> {
        if !self.cache.is_enabled() {
            return self.inner.get_users(pagination).await;
        }

        let cache_key = keys::users_list_key(pagination.page, pagination.page_size);

        // Try cache first
        if let Some(result) = self.cache.get::<PaginatedResult<User>>(&cache_key).await {
            return Ok(result);
        }

        // Cache miss - fetch from DB
        let result = self.inner.get_users(pagination).await?;

        // Cache the result
        self.cache.set(&cache_key, &result, self.config.list_ttl).await;

        Ok(result)
    }

    // ========== User Write Operations ==========

    pub async fn create_user(&self, name: &str, email: &str) -> Result<User, UserServiceError> {
        let user = self.inner.create_user(name, email).await?;

        // Invalidate users list cache
        if self.cache.is_enabled() {
            self.cache.delete_pattern(&keys::users_pattern()).await;
        }

        Ok(user)
    }

    pub async fn update_user(
        &self,
        user_id: Uuid,
        name: &str,
        email: &str,
    ) -> Result<User, UserServiceError> {
        let user = self.inner.update_user(user_id, name, email).await?;

        // Invalidate specific user and users list cache
        if self.cache.is_enabled() {
            self.cache.delete(&keys::user_key(user_id)).await;
            self.cache.delete_pattern(&keys::users_pattern()).await;
        }

        Ok(user)
    }

    pub async fn delete_user(&self, user_id: Uuid) -> Result<(), UserServiceError> {
        self.inner.delete_user(user_id).await?;

        // Invalidate specific user and users list cache
        if self.cache.is_enabled() {
            self.cache.delete(&keys::user_key(user_id)).await;
            self.cache.delete_pattern(&keys::users_pattern()).await;
        }

        Ok(())
    }

    // ========== Role Read Operations ==========

    pub async fn get_role(&self, role_id: Uuid) -> Result<Option<Role>, UserServiceError> {
        if !self.cache.is_enabled() {
            return self.inner.get_role(role_id).await;
        }

        let cache_key = keys::role_key(role_id);

        // Try cache first
        if let Some(role) = self.cache.get::<Role>(&cache_key).await {
            return Ok(Some(role));
        }

        // Cache miss - fetch from DB
        let result = self.inner.get_role(role_id).await?;

        // Cache the result if found
        if let Some(ref role) = result {
            self.cache.set(&cache_key, role, self.config.role_ttl).await;
        }

        Ok(result)
    }

    pub async fn get_roles(
        &self,
        pagination: PaginationParams,
    ) -> Result<PaginatedResult<Role>, UserServiceError> {
        if !self.cache.is_enabled() {
            return self.inner.get_roles(pagination).await;
        }

        let cache_key = keys::roles_list_key(pagination.page, pagination.page_size);

        // Try cache first
        if let Some(result) = self.cache.get::<PaginatedResult<Role>>(&cache_key).await {
            return Ok(result);
        }

        // Cache miss - fetch from DB
        let result = self.inner.get_roles(pagination).await?;

        // Cache the result
        self.cache.set(&cache_key, &result, self.config.list_ttl).await;

        Ok(result)
    }

    // ========== Role Write Operations ==========

    pub async fn create_role(&self, name: &str) -> Result<Role, UserServiceError> {
        let role = self.inner.create_role(name).await?;

        // Invalidate roles list cache
        if self.cache.is_enabled() {
            self.cache.delete_pattern(&keys::roles_pattern()).await;
        }

        Ok(role)
    }

    pub async fn update_role(&self, role_id: Uuid, name: &str) -> Result<Role, UserServiceError> {
        let role = self.inner.update_role(role_id, name).await?;

        // Invalidate role-related caches (role changes affect users who have this role)
        if self.cache.is_enabled() {
            self.cache.delete(&keys::role_key(role_id)).await;
            self.cache.delete_pattern(&keys::roles_pattern()).await;
            // User caches might contain stale role data
            self.cache.delete_pattern(&keys::user_pattern()).await;
            self.cache.delete_pattern(&keys::users_pattern()).await;
        }

        Ok(role)
    }

    pub async fn delete_role(&self, role_id: Uuid) -> Result<(), UserServiceError> {
        self.inner.delete_role(role_id).await?;

        // Invalidate role-related caches (role deletion affects users who had this role)
        if self.cache.is_enabled() {
            self.cache.delete(&keys::role_key(role_id)).await;
            self.cache.delete_pattern(&keys::roles_pattern()).await;
            // User caches might contain stale role data
            self.cache.delete_pattern(&keys::user_pattern()).await;
            self.cache.delete_pattern(&keys::users_pattern()).await;
        }

        Ok(())
    }

    // ========== Role Assignment Operations ==========

    pub async fn assign_role(&self, user_id: Uuid, role_id: Uuid) -> Result<(), UserServiceError> {
        self.inner.assign_role(user_id, role_id).await?;

        // Invalidate user cache (user's roles changed)
        if self.cache.is_enabled() {
            self.cache.delete(&keys::user_key(user_id)).await;
            self.cache.delete_pattern(&keys::users_pattern()).await;
        }

        Ok(())
    }

    pub async fn unassign_role(
        &self,
        user_id: Uuid,
        role_id: Uuid,
    ) -> Result<(), UserServiceError> {
        self.inner.unassign_role(user_id, role_id).await?;

        // Invalidate user cache (user's roles changed)
        if self.cache.is_enabled() {
            self.cache.delete(&keys::user_key(user_id)).await;
            self.cache.delete_pattern(&keys::users_pattern()).await;
        }

        Ok(())
    }

    // ========== Additional Operations (pass-through without caching) ==========

    pub async fn get_roles_for_user(&self, user_id: Uuid) -> Result<Vec<Role>, UserServiceError> {
        self.inner.get_roles_for_user(user_id).await
    }

    pub async fn get_users_by_role(
        &self,
        role_id: Uuid,
        pagination: PaginationParams,
    ) -> Result<PaginatedResult<User>, UserServiceError> {
        self.inner.get_users_by_role(role_id, pagination).await
    }
}
