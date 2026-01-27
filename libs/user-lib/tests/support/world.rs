use async_trait::async_trait;
use cucumber::World;
use mockall::mock;
use std::sync::Arc;
use uuid::Uuid;

use user_lib::entities::{PaginatedResult, PaginationParams, Role, User};
use user_lib::errors_service::UserServiceError;
use user_lib::repository::errors::UserRepositoryError;
use user_lib::repository::models::{RoleRow, UserRoleMapping, UserRow};
use user_lib::repository::traits::{
    RoleRepositoryTrait, UserRepositoryTrait, UserRoleRepositoryTrait,
};
use user_lib::user_service::UserService;

// Mock repositories
mock! {
    #[derive(Debug)]
    pub UserRepo {}

    #[async_trait]
    impl UserRepositoryTrait for UserRepo {
        async fn create_user(&self, keycloak_id: &str) -> Result<UserRow, UserRepositoryError>;
        async fn get_user(&self, user_id: Uuid) -> Result<Option<UserRow>, UserRepositoryError>;
        async fn get_user_by_keycloak_id(&self, keycloak_id: &str) -> Result<Option<UserRow>, UserRepositoryError>;
        async fn delete_user(&self, user_id: Uuid) -> Result<(), UserRepositoryError>;
        async fn get_users_paginated(&self, pagination: PaginationParams) -> Result<(Vec<UserRow>, u64), UserRepositoryError>;
        async fn get_users_by_role_paginated(&self, role_id: Uuid, pagination: PaginationParams) -> Result<(Vec<UserRow>, u64), UserRepositoryError>;
    }
}

mock! {
    #[derive(Debug)]
    pub RoleRepo {}

    #[async_trait]
    impl RoleRepositoryTrait for RoleRepo {
        async fn create_role(&self, name: &str) -> Result<RoleRow, UserRepositoryError>;
        async fn get_role(&self, role_id: Uuid) -> Result<Option<RoleRow>, UserRepositoryError>;
        async fn update_role(&self, role_id: Uuid, name: &str) -> Result<RoleRow, UserRepositoryError>;
        async fn delete_role(&self, role_id: Uuid) -> Result<(), UserRepositoryError>;
        async fn get_roles_for_user(&self, user_id: Uuid) -> Result<Vec<RoleRow>, UserRepositoryError>;
        async fn get_roles_for_users(&self, user_ids: &[String]) -> Result<Vec<UserRoleMapping>, UserRepositoryError>;
        async fn get_roles_paginated(&self, pagination: PaginationParams) -> Result<(Vec<RoleRow>, u64), UserRepositoryError>;
    }
}

mock! {
    #[derive(Debug)]
    pub UserRoleRepo {}

    #[async_trait]
    impl UserRoleRepositoryTrait for UserRoleRepo {
        async fn assign_role(&self, user_id: &str, role_id: &str) -> Result<(), UserRepositoryError>;
        async fn unassign_role(&self, user_id: &str, role_id: &str) -> Result<(), UserRepositoryError>;
    }
}

#[derive(Debug, Default, World)]
pub struct TestWorld {
    // State
    pub current_user: Option<User>,
    pub current_user_id: Option<Uuid>,
    pub current_role: Option<Role>,
    pub current_role_id: Option<Uuid>,
    pub roles: Vec<Role>,
    pub stored_users: Vec<UserRow>,
    pub stored_roles: Vec<RoleRow>,
    pub users_with_current_role: Vec<UserRow>,

    // Results
    pub user_result: Option<Result<User, UserServiceError>>,
    pub role_result: Option<Result<Role, UserServiceError>>,
    pub optional_user_result: Option<Result<Option<User>, UserServiceError>>,
    pub delete_result: Option<Result<(), UserServiceError>>,
    pub paginated_users_result: Option<Result<PaginatedResult<User>, UserServiceError>>,
    pub paginated_roles_result: Option<Result<PaginatedResult<Role>, UserServiceError>>,
    pub error: Option<UserServiceError>,
}

impl TestWorld {
    pub fn create_service_with_mocks(
        &self,
        setup_user_repo: impl FnOnce(&mut MockUserRepo),
        setup_role_repo: impl FnOnce(&mut MockRoleRepo),
        setup_user_role_repo: impl FnOnce(&mut MockUserRoleRepo),
    ) -> UserService<MockUserRepo, MockRoleRepo, MockUserRoleRepo> {
        let mut user_repo = MockUserRepo::new();
        let mut role_repo = MockRoleRepo::new();
        let mut user_role_repo = MockUserRoleRepo::new();

        setup_user_repo(&mut user_repo);
        setup_role_repo(&mut role_repo);
        setup_user_role_repo(&mut user_role_repo);

        UserService::with_repos(
            Arc::new(user_repo),
            Arc::new(role_repo),
            Arc::new(user_role_repo),
        )
    }
}
