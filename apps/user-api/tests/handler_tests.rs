use std::sync::Arc;
use std::time::Duration;
use async_trait::async_trait;
use axum::{http::StatusCode, response::IntoResponse};
use mockall::mock;
use uuid::Uuid;

use user_lib::entities::PaginationParams;
use user_lib::repository::errors::UserRepositoryError;
use user_lib::repository::models::{RoleRow, UserRow, UserRoleMapping};
use user_lib::repository::traits::{RoleRepositoryTrait, UserRepositoryTrait, UserRoleRepositoryTrait};
use user_lib::user_service::UserService;

use user_api::cache::{CacheConfig, CachedUserService, RedisCache};
use user_api::state::AppState;
use user_api::methods::entities::{
    CreateUserRequest, UserResponse, RoleResponse, PaginationQuery,
};

// ==================== MOCKS ====================

mock! {
    pub UserRepo {}

    #[async_trait]
    impl UserRepositoryTrait for UserRepo {
        async fn create_user(&self, name: &str, email: &str) -> Result<UserRow, UserRepositoryError>;
        async fn get_user(&self, user_id: Uuid) -> Result<Option<UserRow>, UserRepositoryError>;
        async fn update_user(&self, user_id: Uuid, name: &str, email: &str) -> Result<UserRow, UserRepositoryError>;
        async fn delete_user(&self, user_id: Uuid) -> Result<(), UserRepositoryError>;
        async fn get_users_paginated(&self, pagination: PaginationParams) -> Result<(Vec<UserRow>, u64), UserRepositoryError>;
        async fn get_users_by_role_paginated(&self, role_id: Uuid, pagination: PaginationParams) -> Result<(Vec<UserRow>, u64), UserRepositoryError>;
    }
}

mock! {
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
    pub UserRoleRepo {}

    #[async_trait]
    impl UserRoleRepositoryTrait for UserRoleRepo {
        async fn assign_role(&self, user_id: &str, role_id: &str) -> Result<(), UserRepositoryError>;
        async fn unassign_role(&self, user_id: &str, role_id: &str) -> Result<(), UserRepositoryError>;
    }
}

// ==================== TEST HELPERS ====================

type TestAppState = AppState<MockUserRepo, MockRoleRepo, MockUserRoleRepo>;

fn create_disabled_cache_config() -> CacheConfig {
    CacheConfig {
        enabled: false,
        redis_host: "localhost".to_string(),
        redis_port: 6379,
        redis_db: 0,
        user_ttl: Duration::from_secs(300),
        role_ttl: Duration::from_secs(600),
        list_ttl: Duration::from_secs(60),
    }
}

async fn create_test_service(
    user_repo: MockUserRepo,
    role_repo: MockRoleRepo,
    user_role_repo: MockUserRoleRepo,
) -> CachedUserService<MockUserRepo, MockRoleRepo, MockUserRoleRepo> {
    let inner = UserService::with_repos(
        Arc::new(user_repo),
        Arc::new(role_repo),
        Arc::new(user_role_repo),
    );
    // Create a disabled cache for tests (no Redis needed)
    let cache_config = create_disabled_cache_config();
    let cache = RedisCache::new(&cache_config).await;
    CachedUserService::new(Arc::new(inner), cache, cache_config)
}

async fn create_test_state(
    user_repo: MockUserRepo,
    role_repo: MockRoleRepo,
    user_role_repo: MockUserRoleRepo,
    env: &str,
) -> TestAppState {
    let service = create_test_service(user_repo, role_repo, user_role_repo).await;
    TestAppState {
        user_service: Arc::new(service),
        env: env.to_string(),
    }
}

// ==================== CREATE USER HANDLER TESTS ====================

#[tokio::test]
async fn test_create_user_handler_success() {
    let mut user_repo = MockUserRepo::new();
    let role_repo = MockRoleRepo::new();
    let user_role_repo = MockUserRoleRepo::new();

    let user_id = Uuid::new_v4();
    user_repo
        .expect_create_user()
        .withf(|name, email| name == "John Doe" && email == "john@example.com")
        .times(1)
        .returning(move |name, email| {
            Ok(UserRow {
                id: user_id.to_string(),
                name: name.to_string(),
                email: email.to_string(),
            })
        });

    let state = create_test_state(user_repo, role_repo, user_role_repo, "test").await;
    let payload = CreateUserRequest {
        name: "John Doe".to_string(),
        email: "john@example.com".to_string(),
    };

    let result = state.user_service
        .create_user(&payload.name, &payload.email)
        .await;

    assert!(result.is_ok());
    let user = result.unwrap();
    assert_eq!(user.name, "John Doe");
    assert_eq!(user.email, "john@example.com");
    assert_eq!(user.id, user_id);
}

#[tokio::test]
async fn test_create_user_handler_email_conflict() {
    let mut user_repo = MockUserRepo::new();
    let role_repo = MockRoleRepo::new();
    let user_role_repo = MockUserRoleRepo::new();

    user_repo
        .expect_create_user()
        .times(1)
        .returning(|_, _| Err(UserRepositoryError::EmailAlreadyExists));

    let state = create_test_state(user_repo, role_repo, user_role_repo, "test").await;

    let result = state.user_service
        .create_user("John Doe", "existing@example.com")
        .await;

    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(matches!(err, user_lib::errors_service::UserServiceError::EmailAlreadyExists));
}

#[tokio::test]
async fn test_create_user_handler_validation_error_empty_name() {
    let user_repo = MockUserRepo::new();
    let role_repo = MockRoleRepo::new();
    let user_role_repo = MockUserRoleRepo::new();

    let state = create_test_state(user_repo, role_repo, user_role_repo, "test").await;

    let result = state.user_service
        .create_user("", "test@example.com")
        .await;

    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(matches!(err, user_lib::errors_service::UserServiceError::Validation(_)));
}

#[tokio::test]
async fn test_create_user_handler_validation_error_invalid_email() {
    let user_repo = MockUserRepo::new();
    let role_repo = MockRoleRepo::new();
    let user_role_repo = MockUserRoleRepo::new();

    let state = create_test_state(user_repo, role_repo, user_role_repo, "test").await;

    let result = state.user_service
        .create_user("John Doe", "invalid-email")
        .await;

    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(matches!(err, user_lib::errors_service::UserServiceError::Validation(_)));
}

// ==================== GET USER BY ID HANDLER TESTS ====================

#[tokio::test]
async fn test_get_user_by_id_handler_success() {
    let mut user_repo = MockUserRepo::new();
    let mut role_repo = MockRoleRepo::new();
    let user_role_repo = MockUserRoleRepo::new();

    let user_id = Uuid::new_v4();
    let role_id = Uuid::new_v4();

    user_repo
        .expect_get_user()
        .times(1)
        .returning(move |_| {
            Ok(Some(UserRow {
                id: user_id.to_string(),
                name: "Jane Doe".to_string(),
                email: "jane@example.com".to_string(),
            }))
        });

    role_repo
        .expect_get_roles_for_user()
        .times(1)
        .returning(move |_| {
            Ok(vec![RoleRow {
                id: role_id.to_string(),
                name: "admin".to_string(),
            }])
        });

    let state = create_test_state(user_repo, role_repo, user_role_repo, "test").await;

    let result = state.user_service.get_user(user_id).await;

    assert!(result.is_ok());
    let user = result.unwrap().unwrap();
    assert_eq!(user.name, "Jane Doe");
    assert_eq!(user.email, "jane@example.com");
    assert_eq!(user.roles.len(), 1);
    assert_eq!(user.roles[0].name, "admin");
}

#[tokio::test]
async fn test_get_user_by_id_handler_not_found() {
    let mut user_repo = MockUserRepo::new();
    let role_repo = MockRoleRepo::new();
    let user_role_repo = MockUserRoleRepo::new();

    user_repo
        .expect_get_user()
        .times(1)
        .returning(|_| Ok(None));

    let state = create_test_state(user_repo, role_repo, user_role_repo, "test").await;
    let user_id = Uuid::new_v4();

    let result = state.user_service.get_user(user_id).await;

    assert!(result.is_ok());
    assert!(result.unwrap().is_none());
}

// ==================== GET USERS (PAGINATION) HANDLER TESTS ====================

#[tokio::test]
async fn test_get_users_handler_success() {
    let mut user_repo = MockUserRepo::new();
    let mut role_repo = MockRoleRepo::new();
    let user_role_repo = MockUserRoleRepo::new();

    let user1_id = Uuid::new_v4();
    let user2_id = Uuid::new_v4();

    user_repo
        .expect_get_users_paginated()
        .times(1)
        .returning(move |_| {
            Ok((vec![
                UserRow {
                    id: user1_id.to_string(),
                    name: "User One".to_string(),
                    email: "user1@example.com".to_string(),
                },
                UserRow {
                    id: user2_id.to_string(),
                    name: "User Two".to_string(),
                    email: "user2@example.com".to_string(),
                },
            ], 5))
        });

    role_repo
        .expect_get_roles_for_users()
        .times(1)
        .returning(|_| Ok(vec![]));

    let state = create_test_state(user_repo, role_repo, user_role_repo, "test").await;
    let pagination = PaginationParams { page: 1, page_size: 2 };

    let result = state.user_service.get_users(pagination).await;

    assert!(result.is_ok());
    let paginated = result.unwrap();
    assert_eq!(paginated.items.len(), 2);
    assert_eq!(paginated.total, 5);
    assert_eq!(paginated.page, 1);
    assert_eq!(paginated.page_size, 2);
    assert_eq!(paginated.total_pages, 3);
}

#[tokio::test]
async fn test_get_users_handler_empty() {
    let mut user_repo = MockUserRepo::new();
    let role_repo = MockRoleRepo::new();
    let user_role_repo = MockUserRoleRepo::new();

    user_repo
        .expect_get_users_paginated()
        .times(1)
        .returning(|_| Ok((vec![], 0)));

    let state = create_test_state(user_repo, role_repo, user_role_repo, "test").await;
    let pagination = PaginationParams::default();

    let result = state.user_service.get_users(pagination).await;

    assert!(result.is_ok());
    let paginated = result.unwrap();
    assert!(paginated.items.is_empty());
    assert_eq!(paginated.total, 0);
}

// ==================== UPDATE USER HANDLER TESTS ====================

#[tokio::test]
async fn test_update_user_handler_success() {
    let mut user_repo = MockUserRepo::new();
    let mut role_repo = MockRoleRepo::new();
    let user_role_repo = MockUserRoleRepo::new();

    let user_id = Uuid::new_v4();

    user_repo
        .expect_update_user()
        .times(1)
        .returning(move |_, name, email| {
            Ok(UserRow {
                id: user_id.to_string(),
                name: name.to_string(),
                email: email.to_string(),
            })
        });

    role_repo
        .expect_get_roles_for_user()
        .times(1)
        .returning(|_| Ok(vec![]));

    let state = create_test_state(user_repo, role_repo, user_role_repo, "test").await;

    let result = state.user_service
        .update_user(user_id, "Updated Name", "updated@example.com")
        .await;

    assert!(result.is_ok());
    let user = result.unwrap();
    assert_eq!(user.name, "Updated Name");
    assert_eq!(user.email, "updated@example.com");
}

#[tokio::test]
async fn test_update_user_handler_email_conflict() {
    let mut user_repo = MockUserRepo::new();
    let role_repo = MockRoleRepo::new();
    let user_role_repo = MockUserRoleRepo::new();

    user_repo
        .expect_update_user()
        .times(1)
        .returning(|_, _, _| Err(UserRepositoryError::EmailAlreadyExists));

    let state = create_test_state(user_repo, role_repo, user_role_repo, "test").await;
    let user_id = Uuid::new_v4();

    let result = state.user_service
        .update_user(user_id, "Name", "taken@example.com")
        .await;

    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), user_lib::errors_service::UserServiceError::EmailAlreadyExists));
}

#[tokio::test]
async fn test_update_user_handler_validation_error() {
    let user_repo = MockUserRepo::new();
    let role_repo = MockRoleRepo::new();
    let user_role_repo = MockUserRoleRepo::new();

    let state = create_test_state(user_repo, role_repo, user_role_repo, "test").await;
    let user_id = Uuid::new_v4();

    let result = state.user_service
        .update_user(user_id, "", "test@example.com")
        .await;

    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), user_lib::errors_service::UserServiceError::Validation(_)));
}

// ==================== DELETE USER HANDLER TESTS ====================

#[tokio::test]
async fn test_delete_user_handler_success() {
    let mut user_repo = MockUserRepo::new();
    let role_repo = MockRoleRepo::new();
    let user_role_repo = MockUserRoleRepo::new();

    user_repo
        .expect_delete_user()
        .times(1)
        .returning(|_| Ok(()));

    let state = create_test_state(user_repo, role_repo, user_role_repo, "test").await;
    let user_id = Uuid::new_v4();

    let result = state.user_service.delete_user(user_id).await;

    assert!(result.is_ok());
}

// ==================== CREATE ROLE HANDLER TESTS ====================

#[tokio::test]
async fn test_create_role_handler_success() {
    let user_repo = MockUserRepo::new();
    let mut role_repo = MockRoleRepo::new();
    let user_role_repo = MockUserRoleRepo::new();

    let role_id = Uuid::new_v4();

    role_repo
        .expect_create_role()
        .withf(|name| name == "admin")
        .times(1)
        .returning(move |name| {
            Ok(RoleRow {
                id: role_id.to_string(),
                name: name.to_string(),
            })
        });

    let state = create_test_state(user_repo, role_repo, user_role_repo, "test").await;

    let result = state.user_service.create_role("admin").await;

    assert!(result.is_ok());
    let role = result.unwrap();
    assert_eq!(role.name, "admin");
    assert_eq!(role.id, role_id);
}

#[tokio::test]
async fn test_create_role_handler_name_conflict() {
    let user_repo = MockUserRepo::new();
    let mut role_repo = MockRoleRepo::new();
    let user_role_repo = MockUserRoleRepo::new();

    role_repo
        .expect_create_role()
        .times(1)
        .returning(|_| Err(UserRepositoryError::RoleNameAlreadyExists));

    let state = create_test_state(user_repo, role_repo, user_role_repo, "test").await;

    let result = state.user_service.create_role("admin").await;

    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), user_lib::errors_service::UserServiceError::RoleNameAlreadyExists));
}

#[tokio::test]
async fn test_create_role_handler_validation_error() {
    let user_repo = MockUserRepo::new();
    let role_repo = MockRoleRepo::new();
    let user_role_repo = MockUserRoleRepo::new();

    let state = create_test_state(user_repo, role_repo, user_role_repo, "test").await;

    let result = state.user_service.create_role("").await;

    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), user_lib::errors_service::UserServiceError::Validation(_)));
}

// ==================== GET ROLE BY ID HANDLER TESTS ====================

#[tokio::test]
async fn test_get_role_by_id_handler_success() {
    let user_repo = MockUserRepo::new();
    let mut role_repo = MockRoleRepo::new();
    let user_role_repo = MockUserRoleRepo::new();

    let role_id = Uuid::new_v4();

    role_repo
        .expect_get_role()
        .times(1)
        .returning(move |_| {
            Ok(Some(RoleRow {
                id: role_id.to_string(),
                name: "editor".to_string(),
            }))
        });

    let state = create_test_state(user_repo, role_repo, user_role_repo, "test").await;

    let result = state.user_service.get_role(role_id).await;

    assert!(result.is_ok());
    let role = result.unwrap().unwrap();
    assert_eq!(role.name, "editor");
    assert_eq!(role.id, role_id);
}

#[tokio::test]
async fn test_get_role_by_id_handler_not_found() {
    let user_repo = MockUserRepo::new();
    let mut role_repo = MockRoleRepo::new();
    let user_role_repo = MockUserRoleRepo::new();

    role_repo
        .expect_get_role()
        .times(1)
        .returning(|_| Ok(None));

    let state = create_test_state(user_repo, role_repo, user_role_repo, "test").await;
    let role_id = Uuid::new_v4();

    let result = state.user_service.get_role(role_id).await;

    assert!(result.is_ok());
    assert!(result.unwrap().is_none());
}

// ==================== GET ROLES (PAGINATION) HANDLER TESTS ====================

#[tokio::test]
async fn test_get_roles_handler_success() {
    let user_repo = MockUserRepo::new();
    let mut role_repo = MockRoleRepo::new();
    let user_role_repo = MockUserRoleRepo::new();

    let role1_id = Uuid::new_v4();
    let role2_id = Uuid::new_v4();

    role_repo
        .expect_get_roles_paginated()
        .times(1)
        .returning(move |_| {
            Ok((vec![
                RoleRow {
                    id: role1_id.to_string(),
                    name: "admin".to_string(),
                },
                RoleRow {
                    id: role2_id.to_string(),
                    name: "editor".to_string(),
                },
            ], 2))
        });

    let state = create_test_state(user_repo, role_repo, user_role_repo, "test").await;
    let pagination = PaginationParams::default();

    let result = state.user_service.get_roles(pagination).await;

    assert!(result.is_ok());
    let paginated = result.unwrap();
    assert_eq!(paginated.items.len(), 2);
    assert_eq!(paginated.total, 2);
}

// ==================== UPDATE ROLE HANDLER TESTS ====================

#[tokio::test]
async fn test_update_role_handler_success() {
    let user_repo = MockUserRepo::new();
    let mut role_repo = MockRoleRepo::new();
    let user_role_repo = MockUserRoleRepo::new();

    let role_id = Uuid::new_v4();

    role_repo
        .expect_update_role()
        .times(1)
        .returning(move |_, name| {
            Ok(RoleRow {
                id: role_id.to_string(),
                name: name.to_string(),
            })
        });

    let state = create_test_state(user_repo, role_repo, user_role_repo, "test").await;

    let result = state.user_service.update_role(role_id, "super-admin").await;

    assert!(result.is_ok());
    let role = result.unwrap();
    assert_eq!(role.name, "super-admin");
}

#[tokio::test]
async fn test_update_role_handler_name_conflict() {
    let user_repo = MockUserRepo::new();
    let mut role_repo = MockRoleRepo::new();
    let user_role_repo = MockUserRoleRepo::new();

    role_repo
        .expect_update_role()
        .times(1)
        .returning(|_, _| Err(UserRepositoryError::RoleNameAlreadyExists));

    let state = create_test_state(user_repo, role_repo, user_role_repo, "test").await;
    let role_id = Uuid::new_v4();

    let result = state.user_service.update_role(role_id, "taken-name").await;

    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), user_lib::errors_service::UserServiceError::RoleNameAlreadyExists));
}

#[tokio::test]
async fn test_update_role_handler_validation_error() {
    let user_repo = MockUserRepo::new();
    let role_repo = MockRoleRepo::new();
    let user_role_repo = MockUserRoleRepo::new();

    let state = create_test_state(user_repo, role_repo, user_role_repo, "test").await;
    let role_id = Uuid::new_v4();

    let result = state.user_service.update_role(role_id, "   ").await;

    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), user_lib::errors_service::UserServiceError::Validation(_)));
}

// ==================== DELETE ROLE HANDLER TESTS ====================

#[tokio::test]
async fn test_delete_role_handler_success() {
    let user_repo = MockUserRepo::new();
    let mut role_repo = MockRoleRepo::new();
    let user_role_repo = MockUserRoleRepo::new();

    role_repo
        .expect_delete_role()
        .times(1)
        .returning(|_| Ok(()));

    let state = create_test_state(user_repo, role_repo, user_role_repo, "test").await;
    let role_id = Uuid::new_v4();

    let result = state.user_service.delete_role(role_id).await;

    assert!(result.is_ok());
}

// ==================== ASSIGN ROLE HANDLER TESTS ====================

#[tokio::test]
async fn test_assign_role_handler_success() {
    let user_repo = MockUserRepo::new();
    let role_repo = MockRoleRepo::new();
    let mut user_role_repo = MockUserRoleRepo::new();

    user_role_repo
        .expect_assign_role()
        .times(1)
        .returning(|_, _| Ok(()));

    let state = create_test_state(user_repo, role_repo, user_role_repo, "test").await;
    let user_id = Uuid::new_v4();
    let role_id = Uuid::new_v4();

    let result = state.user_service.assign_role(user_id, role_id).await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn test_assign_role_handler_already_assigned() {
    let user_repo = MockUserRepo::new();
    let role_repo = MockRoleRepo::new();
    let mut user_role_repo = MockUserRoleRepo::new();

    user_role_repo
        .expect_assign_role()
        .times(1)
        .returning(|_, _| Err(UserRepositoryError::UserAlreadyHasRole));

    let state = create_test_state(user_repo, role_repo, user_role_repo, "test").await;
    let user_id = Uuid::new_v4();
    let role_id = Uuid::new_v4();

    let result = state.user_service.assign_role(user_id, role_id).await;

    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), user_lib::errors_service::UserServiceError::UserAlreadyHasRole));
}

// ==================== UNASSIGN ROLE HANDLER TESTS ====================

#[tokio::test]
async fn test_unassign_role_handler_success() {
    let user_repo = MockUserRepo::new();
    let role_repo = MockRoleRepo::new();
    let mut user_role_repo = MockUserRoleRepo::new();

    user_role_repo
        .expect_unassign_role()
        .times(1)
        .returning(|_, _| Ok(()));

    let state = create_test_state(user_repo, role_repo, user_role_repo, "test").await;
    let user_id = Uuid::new_v4();
    let role_id = Uuid::new_v4();

    let result = state.user_service.unassign_role(user_id, role_id).await;

    assert!(result.is_ok());
}

// ==================== API ERROR MAPPING TESTS ====================

#[tokio::test]
async fn test_api_error_bad_request() {
    use user_api::error::ApiError;

    let error = ApiError::BadRequest("invalid input".to_string());
    let response = error.into_response();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_api_error_not_found() {
    use user_api::error::ApiError;

    let error = ApiError::NotFound("user not found".to_string());
    let response = error.into_response();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_api_error_conflict() {
    use user_api::error::ApiError;

    let error = ApiError::Conflict("email already exists".to_string());
    let response = error.into_response();

    assert_eq!(response.status(), StatusCode::CONFLICT);
}

#[tokio::test]
async fn test_api_error_internal() {
    use user_api::error::ApiError;

    let error = ApiError::Internal("database error".to_string());
    let response = error.into_response();

    assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
}

#[tokio::test]
async fn test_api_error_helper_invalid_uuid() {
    use user_api::error::ApiError;

    let error = ApiError::invalid_uuid();
    let response = error.into_response();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_api_error_helper_user_not_found() {
    use user_api::error::ApiError;

    let error = ApiError::user_not_found();
    let response = error.into_response();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_api_error_helper_role_not_found() {
    use user_api::error::ApiError;

    let error = ApiError::role_not_found();
    let response = error.into_response();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

// ==================== IS_PROD_LIKE TESTS ====================

#[tokio::test]
async fn test_is_prod_like_local() {
    use user_api::error::is_prod_like;

    assert!(!is_prod_like("local"));
    assert!(!is_prod_like("LOCAL"));
}

#[tokio::test]
async fn test_is_prod_like_dev() {
    use user_api::error::is_prod_like;

    assert!(!is_prod_like("dev"));
    assert!(!is_prod_like("development"));
}

#[tokio::test]
async fn test_is_prod_like_test() {
    use user_api::error::is_prod_like;

    assert!(!is_prod_like("test"));
    assert!(!is_prod_like("testing"));
}

#[tokio::test]
async fn test_is_prod_like_prod() {
    use user_api::error::is_prod_like;

    assert!(is_prod_like("prod"));
    assert!(is_prod_like("PROD"));
    assert!(is_prod_like("prod01"));
    assert!(is_prod_like("prod-us-east"));
    assert!(is_prod_like("production"));
}

// ==================== HANDLE_SERVICE_ERROR TESTS ====================

#[tokio::test]
async fn test_handle_service_error_validation_always_shown() {
    use user_api::error::handle_service_error;
    use user_lib::errors_service::UserServiceError;

    let err = UserServiceError::Validation("name cannot be empty".to_string());
    let api_err = handle_service_error(err, "prod", "test_op");
    let response = api_err.into_response();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_handle_service_error_email_exists_always_shown() {
    use user_api::error::handle_service_error;
    use user_lib::errors_service::UserServiceError;

    let err = UserServiceError::EmailAlreadyExists;
    let api_err = handle_service_error(err, "prod", "test_op");
    let response = api_err.into_response();

    assert_eq!(response.status(), StatusCode::CONFLICT);
}

#[tokio::test]
async fn test_handle_service_error_role_name_exists_always_shown() {
    use user_api::error::handle_service_error;
    use user_lib::errors_service::UserServiceError;

    let err = UserServiceError::RoleNameAlreadyExists;
    let api_err = handle_service_error(err, "prod", "test_op");
    let response = api_err.into_response();

    assert_eq!(response.status(), StatusCode::CONFLICT);
}

#[tokio::test]
async fn test_handle_service_error_user_already_has_role() {
    use user_api::error::handle_service_error;
    use user_lib::errors_service::UserServiceError;

    let err = UserServiceError::UserAlreadyHasRole;
    let api_err = handle_service_error(err, "prod", "test_op");
    let response = api_err.into_response();

    assert_eq!(response.status(), StatusCode::CONFLICT);
}

#[tokio::test]
async fn test_handle_service_error_not_found() {
    use user_api::error::handle_service_error;
    use user_lib::errors_service::UserServiceError;

    let err = UserServiceError::NotFound;
    let api_err = handle_service_error(err, "prod", "test_op");
    let response = api_err.into_response();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

// ==================== PAGINATION QUERY TESTS ====================

#[tokio::test]
async fn test_pagination_query_defaults() {
    let query = PaginationQuery {
        page: None,
        page_size: None,
    };

    let params: PaginationParams = query.into();
    assert_eq!(params.page, 1);
    assert_eq!(params.page_size, 20); // DEFAULT_PAGE_SIZE from user-lib
}

#[tokio::test]
async fn test_pagination_query_custom_values() {
    let query = PaginationQuery {
        page: Some(3),
        page_size: Some(25),
    };

    let params: PaginationParams = query.into();
    assert_eq!(params.page, 3);
    assert_eq!(params.page_size, 25);
}

// ==================== USER RESPONSE CONVERSION TESTS ====================

#[tokio::test]
async fn test_user_response_from_user() {
    use user_lib::entities::{User, Role};

    let user_id = Uuid::new_v4();
    let role_id = Uuid::new_v4();

    let user = User {
        id: user_id,
        name: "Test User".to_string(),
        email: "test@example.com".to_string(),
        roles: vec![Role {
            id: role_id,
            name: "admin".to_string(),
        }],
    };

    let response = UserResponse::from(user);

    assert_eq!(response.id.to_string(), user_id.to_string());
    assert_eq!(response.name, "Test User");
    assert_eq!(response.email, "test@example.com");
    assert_eq!(response.roles.len(), 1);
    assert_eq!(response.roles[0].name, "admin");
}

// ==================== ROLE RESPONSE CONVERSION TESTS ====================

#[tokio::test]
async fn test_role_response_from_role() {
    use user_lib::entities::Role;

    let role_id = Uuid::new_v4();

    let role = Role {
        id: role_id,
        name: "editor".to_string(),
    };

    let response = RoleResponse::from(role);

    assert_eq!(response.id.to_string(), role_id.to_string());
    assert_eq!(response.name, "editor");
}
