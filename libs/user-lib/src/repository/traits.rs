use async_trait::async_trait;
use uuid::Uuid;

use crate::repository::errors::UserRepositoryError;
use crate::repository::models::{RoleRow, UserRow};

#[async_trait]
pub trait UserRepositoryTrait: Send + Sync {
    async fn create_user(&self, name: &str, email: &str) -> Result<UserRow, UserRepositoryError>;
    async fn get_user(&self, user_id: Uuid) -> Result<Option<UserRow>, UserRepositoryError>;
    async fn update_user(&self, user_id: Uuid, name: &str, email: &str) -> Result<UserRow, UserRepositoryError>;
    async fn delete_user(&self, user_id: Uuid) -> Result<(), UserRepositoryError>;
    async fn get_users(&self) -> Result<Vec<UserRow>, UserRepositoryError>;
    async fn get_users_by_role(&self, role_id: Uuid) -> Result<Vec<UserRow>, UserRepositoryError>;
}

#[async_trait]
pub trait RoleRepositoryTrait: Send + Sync {
    async fn create_role(&self, name: &str) -> Result<RoleRow, UserRepositoryError>;
    async fn get_role(&self, role_id: Uuid) -> Result<Option<RoleRow>, UserRepositoryError>;
    async fn update_role(&self, role_id: Uuid, name: &str) -> Result<RoleRow, UserRepositoryError>;
    async fn delete_role(&self, role_id: Uuid) -> Result<(), UserRepositoryError>;
    async fn get_roles_for_user(&self, user_id: Uuid) -> Result<Vec<RoleRow>, UserRepositoryError>;
    async fn get_roles(&self) -> Result<Vec<RoleRow>, UserRepositoryError>;
}

#[async_trait]
pub trait UserRoleRepositoryTrait: Send + Sync {
    async fn assign_role(&self, user_id: &str, role_id: &str) -> Result<(), UserRepositoryError>;
    async fn unassign_role(&self, user_id: &str, role_id: &str) -> Result<(), UserRepositoryError>;
}
