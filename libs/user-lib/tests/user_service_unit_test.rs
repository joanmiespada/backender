use std::sync::Arc;
use async_trait::async_trait;
use mockall::mock;
use uuid::Uuid;

use user_lib::entities::PaginationParams;
use user_lib::repository::errors::UserRepositoryError;
use user_lib::repository::models::{RoleRow, UserRow, UserRoleMapping};
use user_lib::repository::traits::{RoleRepositoryTrait, UserRepositoryTrait, UserRoleRepositoryTrait};
use user_lib::errors_service::UserServiceError;
use user_lib::user_service::UserService;

mock! {
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

fn create_test_service(
    user_repo: MockUserRepo,
    role_repo: MockRoleRepo,
    user_role_repo: MockUserRoleRepo,
) -> UserService<MockUserRepo, MockRoleRepo, MockUserRoleRepo> {
    UserService::with_repos(
        Arc::new(user_repo),
        Arc::new(role_repo),
        Arc::new(user_role_repo),
    )
}

// ==================== CREATE USER TESTS ====================

#[tokio::test]
async fn test_create_user_success() {
    let mut user_repo = MockUserRepo::new();
    let role_repo = MockRoleRepo::new();
    let user_role_repo = MockUserRoleRepo::new();

    let user_id = Uuid::new_v4();
    let keycloak_id = "kc-user-12345";

    user_repo
        .expect_create_user()
        .withf(|kc_id| kc_id == "kc-user-12345")
        .times(1)
        .returning(move |kc_id| {
            Ok(UserRow {
                id: user_id.to_string(),
                keycloak_id: kc_id.to_string(),
            })
        });

    let service = create_test_service(user_repo, role_repo, user_role_repo);
    let result = service.create_user(keycloak_id).await;

    assert!(result.is_ok());
    let user = result.unwrap();
    assert_eq!(user.id, user_id);
    assert_eq!(user.keycloak_id, keycloak_id);
    assert!(user.roles.is_empty());
}

// ==================== GET USER TESTS ====================

#[tokio::test]
async fn test_get_user_success() {
    let mut user_repo = MockUserRepo::new();
    let mut role_repo = MockRoleRepo::new();
    let user_role_repo = MockUserRoleRepo::new();

    let user_id = Uuid::new_v4();
    let role_id = Uuid::new_v4();
    let keycloak_id = "kc-user-jane";

    user_repo
        .expect_get_user()
        .withf(move |id| *id == user_id)
        .times(1)
        .returning(move |_| {
            Ok(Some(UserRow {
                id: user_id.to_string(),
                keycloak_id: keycloak_id.to_string(),
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

    let service = create_test_service(user_repo, role_repo, user_role_repo);
    let result = service.get_user(user_id).await;

    assert!(result.is_ok());
    let user = result.unwrap().unwrap();
    assert_eq!(user.id, user_id);
    assert_eq!(user.keycloak_id, keycloak_id);
    assert_eq!(user.roles.len(), 1);
    assert_eq!(user.roles[0].name, "admin");
}

#[tokio::test]
async fn test_get_user_not_found() {
    let mut user_repo = MockUserRepo::new();
    let role_repo = MockRoleRepo::new();
    let user_role_repo = MockUserRoleRepo::new();

    let user_id = Uuid::new_v4();

    user_repo
        .expect_get_user()
        .times(1)
        .returning(|_| Ok(None));

    let service = create_test_service(user_repo, role_repo, user_role_repo);
    let result = service.get_user(user_id).await;

    assert!(result.is_ok());
    assert!(result.unwrap().is_none());
}

#[tokio::test]
async fn test_get_user_by_keycloak_id_success() {
    let mut user_repo = MockUserRepo::new();
    let mut role_repo = MockRoleRepo::new();
    let user_role_repo = MockUserRoleRepo::new();

    let user_id = Uuid::new_v4();
    let keycloak_id = "kc-user-lookup";

    user_repo
        .expect_get_user_by_keycloak_id()
        .withf(|kc_id| kc_id == "kc-user-lookup")
        .times(1)
        .returning(move |kc_id| {
            Ok(Some(UserRow {
                id: user_id.to_string(),
                keycloak_id: kc_id.to_string(),
            }))
        });

    role_repo
        .expect_get_roles_for_user()
        .times(1)
        .returning(|_| Ok(vec![]));

    let service = create_test_service(user_repo, role_repo, user_role_repo);
    let result = service.get_user_by_keycloak_id(keycloak_id).await;

    assert!(result.is_ok());
    let user = result.unwrap().unwrap();
    assert_eq!(user.keycloak_id, keycloak_id);
}

// ==================== DELETE USER TESTS ====================

#[tokio::test]
async fn test_delete_user_success() {
    let mut user_repo = MockUserRepo::new();
    let role_repo = MockRoleRepo::new();
    let user_role_repo = MockUserRoleRepo::new();

    let user_id = Uuid::new_v4();

    user_repo
        .expect_delete_user()
        .withf(move |id| *id == user_id)
        .times(1)
        .returning(|_| Ok(()));

    let service = create_test_service(user_repo, role_repo, user_role_repo);
    let result = service.delete_user(user_id).await;

    assert!(result.is_ok());
}

// ==================== ROLE ASSIGNMENT TESTS ====================

#[tokio::test]
async fn test_assign_role_success() {
    let user_repo = MockUserRepo::new();
    let role_repo = MockRoleRepo::new();
    let mut user_role_repo = MockUserRoleRepo::new();

    let user_id = Uuid::new_v4();
    let role_id = Uuid::new_v4();

    user_role_repo
        .expect_assign_role()
        .withf(move |uid, rid| {
            uid == &user_id.to_string() && rid == &role_id.to_string()
        })
        .times(1)
        .returning(|_, _| Ok(()));

    let service = create_test_service(user_repo, role_repo, user_role_repo);
    let result = service.assign_role(user_id, role_id).await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn test_assign_role_already_assigned() {
    let user_repo = MockUserRepo::new();
    let role_repo = MockRoleRepo::new();
    let mut user_role_repo = MockUserRoleRepo::new();

    let user_id = Uuid::new_v4();
    let role_id = Uuid::new_v4();

    user_role_repo
        .expect_assign_role()
        .times(1)
        .returning(|_, _| Err(UserRepositoryError::UserAlreadyHasRole));

    let service = create_test_service(user_repo, role_repo, user_role_repo);
    let result = service.assign_role(user_id, role_id).await;

    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), UserServiceError::UserAlreadyHasRole));
}

#[tokio::test]
async fn test_unassign_role_success() {
    let user_repo = MockUserRepo::new();
    let role_repo = MockRoleRepo::new();
    let mut user_role_repo = MockUserRoleRepo::new();

    let user_id = Uuid::new_v4();
    let role_id = Uuid::new_v4();

    user_role_repo
        .expect_unassign_role()
        .withf(move |uid, rid| {
            uid == &user_id.to_string() && rid == &role_id.to_string()
        })
        .times(1)
        .returning(|_, _| Ok(()));

    let service = create_test_service(user_repo, role_repo, user_role_repo);
    let result = service.unassign_role(user_id, role_id).await;

    assert!(result.is_ok());
}

// ==================== CREATE ROLE TESTS ====================

#[tokio::test]
async fn test_create_role_success() {
    let user_repo = MockUserRepo::new();
    let mut role_repo = MockRoleRepo::new();
    let user_role_repo = MockUserRoleRepo::new();

    let role_id = Uuid::new_v4();

    role_repo
        .expect_create_role()
        .withf(|name| name == "editor")
        .times(1)
        .returning(move |_| {
            Ok(RoleRow {
                id: role_id.to_string(),
                name: "editor".to_string(),
            })
        });

    let service = create_test_service(user_repo, role_repo, user_role_repo);
    let result = service.create_role("editor").await;

    assert!(result.is_ok());
    let role = result.unwrap();
    assert_eq!(role.id, role_id);
    assert_eq!(role.name, "editor");
}

#[tokio::test]
async fn test_create_role_name_already_exists() {
    let user_repo = MockUserRepo::new();
    let mut role_repo = MockRoleRepo::new();
    let user_role_repo = MockUserRoleRepo::new();

    role_repo
        .expect_create_role()
        .times(1)
        .returning(|_| Err(UserRepositoryError::RoleNameAlreadyExists));

    let service = create_test_service(user_repo, role_repo, user_role_repo);
    let result = service.create_role("admin").await;

    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), UserServiceError::RoleNameAlreadyExists));
}

// ==================== GET ROLE TESTS ====================

#[tokio::test]
async fn test_get_role_success() {
    let user_repo = MockUserRepo::new();
    let mut role_repo = MockRoleRepo::new();
    let user_role_repo = MockUserRoleRepo::new();

    let role_id = Uuid::new_v4();

    role_repo
        .expect_get_role()
        .withf(move |id| *id == role_id)
        .times(1)
        .returning(move |_| {
            Ok(Some(RoleRow {
                id: role_id.to_string(),
                name: "viewer".to_string(),
            }))
        });

    let service = create_test_service(user_repo, role_repo, user_role_repo);
    let result = service.get_role(role_id).await;

    assert!(result.is_ok());
    let role = result.unwrap().unwrap();
    assert_eq!(role.id, role_id);
    assert_eq!(role.name, "viewer");
}

#[tokio::test]
async fn test_get_role_not_found() {
    let user_repo = MockUserRepo::new();
    let mut role_repo = MockRoleRepo::new();
    let user_role_repo = MockUserRoleRepo::new();

    role_repo
        .expect_get_role()
        .times(1)
        .returning(|_| Ok(None));

    let service = create_test_service(user_repo, role_repo, user_role_repo);
    let result = service.get_role(Uuid::new_v4()).await;

    assert!(result.is_ok());
    assert!(result.unwrap().is_none());
}

// ==================== UPDATE ROLE TESTS ====================

#[tokio::test]
async fn test_update_role_success() {
    let user_repo = MockUserRepo::new();
    let mut role_repo = MockRoleRepo::new();
    let user_role_repo = MockUserRoleRepo::new();

    let role_id = Uuid::new_v4();

    role_repo
        .expect_update_role()
        .withf(move |id, name| *id == role_id && name == "super-admin")
        .times(1)
        .returning(move |_, _| {
            Ok(RoleRow {
                id: role_id.to_string(),
                name: "super-admin".to_string(),
            })
        });

    let service = create_test_service(user_repo, role_repo, user_role_repo);
    let result = service.update_role(role_id, "super-admin").await;

    assert!(result.is_ok());
    let role = result.unwrap();
    assert_eq!(role.name, "super-admin");
}

// ==================== DELETE ROLE TESTS ====================

#[tokio::test]
async fn test_delete_role_success() {
    let user_repo = MockUserRepo::new();
    let mut role_repo = MockRoleRepo::new();
    let user_role_repo = MockUserRoleRepo::new();

    let role_id = Uuid::new_v4();

    role_repo
        .expect_delete_role()
        .withf(move |id| *id == role_id)
        .times(1)
        .returning(|_| Ok(()));

    let service = create_test_service(user_repo, role_repo, user_role_repo);
    let result = service.delete_role(role_id).await;

    assert!(result.is_ok());
}

// ==================== GET USERS TESTS ====================

#[tokio::test]
async fn test_get_users_success() {
    let mut user_repo = MockUserRepo::new();
    let mut role_repo = MockRoleRepo::new();
    let user_role_repo = MockUserRoleRepo::new();

    let user1_id = Uuid::new_v4();
    let user2_id = Uuid::new_v4();
    let role_id = Uuid::new_v4();

    let user1_id_clone = user1_id;
    user_repo
        .expect_get_users_paginated()
        .times(1)
        .returning(move |_| {
            Ok((vec![
                UserRow {
                    id: user1_id.to_string(),
                    keycloak_id: "kc-user-1".to_string(),
                },
                UserRow {
                    id: user2_id.to_string(),
                    keycloak_id: "kc-user-2".to_string(),
                },
            ], 2))
        });

    role_repo
        .expect_get_roles_for_users()
        .times(1)
        .returning(move |_| {
            Ok(vec![UserRoleMapping {
                user_id: user1_id_clone.to_string(),
                role_id: role_id.to_string(),
                role_name: "admin".to_string(),
            }])
        });

    let service = create_test_service(user_repo, role_repo, user_role_repo);
    let result = service.get_users(PaginationParams::default()).await;

    assert!(result.is_ok());
    let paginated = result.unwrap();
    assert_eq!(paginated.items.len(), 2);
    assert_eq!(paginated.total, 2);
    assert_eq!(paginated.page, 1);
    assert_eq!(paginated.items[0].keycloak_id, "kc-user-1");
    assert_eq!(paginated.items[0].roles.len(), 1);
    assert_eq!(paginated.items[0].roles[0].name, "admin");
    assert_eq!(paginated.items[1].keycloak_id, "kc-user-2");
    assert_eq!(paginated.items[1].roles.len(), 0);
}

#[tokio::test]
async fn test_get_users_empty() {
    let mut user_repo = MockUserRepo::new();
    let role_repo = MockRoleRepo::new();
    let user_role_repo = MockUserRoleRepo::new();

    user_repo
        .expect_get_users_paginated()
        .times(1)
        .returning(|_| Ok((vec![], 0)));

    let service = create_test_service(user_repo, role_repo, user_role_repo);
    let result = service.get_users(PaginationParams::default()).await;

    assert!(result.is_ok());
    let paginated = result.unwrap();
    assert!(paginated.items.is_empty());
    assert_eq!(paginated.total, 0);
}

// ==================== GET ROLES TESTS ====================

#[tokio::test]
async fn test_get_roles_success() {
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
                    name: "user".to_string(),
                },
            ], 2))
        });

    let service = create_test_service(user_repo, role_repo, user_role_repo);
    let result = service.get_roles(PaginationParams::default()).await;

    assert!(result.is_ok());
    let paginated = result.unwrap();
    assert_eq!(paginated.items.len(), 2);
    assert_eq!(paginated.total, 2);
    assert_eq!(paginated.items[0].name, "admin");
    assert_eq!(paginated.items[1].name, "user");
}

// ==================== GET ROLES FOR USER TESTS ====================

#[tokio::test]
async fn test_get_roles_for_user_success() {
    let user_repo = MockUserRepo::new();
    let mut role_repo = MockRoleRepo::new();
    let user_role_repo = MockUserRoleRepo::new();

    let user_id = Uuid::new_v4();
    let role_id = Uuid::new_v4();

    role_repo
        .expect_get_roles_for_user()
        .withf(move |id| *id == user_id)
        .times(1)
        .returning(move |_| {
            Ok(vec![RoleRow {
                id: role_id.to_string(),
                name: "member".to_string(),
            }])
        });

    let service = create_test_service(user_repo, role_repo, user_role_repo);
    let result = service.get_roles_for_user(user_id).await;

    assert!(result.is_ok());
    let roles = result.unwrap();
    assert_eq!(roles.len(), 1);
    assert_eq!(roles[0].name, "member");
}

// ==================== GET USERS BY ROLE TESTS ====================

#[tokio::test]
async fn test_get_users_by_role_success() {
    let mut user_repo = MockUserRepo::new();
    let mut role_repo = MockRoleRepo::new();
    let user_role_repo = MockUserRoleRepo::new();

    let role_id = Uuid::new_v4();
    let user_id = Uuid::new_v4();

    user_repo
        .expect_get_users_by_role_paginated()
        .withf(move |id, _| *id == role_id)
        .times(1)
        .returning(move |_, _| {
            Ok((vec![UserRow {
                id: user_id.to_string(),
                keycloak_id: "kc-admin-user".to_string(),
            }], 1))
        });

    role_repo
        .expect_get_roles_for_users()
        .times(1)
        .returning(move |_| {
            Ok(vec![UserRoleMapping {
                user_id: user_id.to_string(),
                role_id: role_id.to_string(),
                role_name: "admin".to_string(),
            }])
        });

    let service = create_test_service(user_repo, role_repo, user_role_repo);
    let result = service.get_users_by_role(role_id, PaginationParams::default()).await;

    assert!(result.is_ok());
    let paginated = result.unwrap();
    assert_eq!(paginated.items.len(), 1);
    assert_eq!(paginated.total, 1);
    assert_eq!(paginated.items[0].keycloak_id, "kc-admin-user");
    assert_eq!(paginated.items[0].roles.len(), 1);
}

// ==================== VALIDATION TESTS ====================

#[tokio::test]
async fn test_create_role_empty_name() {
    let user_repo = MockUserRepo::new();
    let role_repo = MockRoleRepo::new();
    let user_role_repo = MockUserRoleRepo::new();

    let service = create_test_service(user_repo, role_repo, user_role_repo);
    let result = service.create_role("").await;

    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), UserServiceError::Validation(_)));
}

#[tokio::test]
async fn test_update_role_empty_name() {
    let user_repo = MockUserRepo::new();
    let role_repo = MockRoleRepo::new();
    let user_role_repo = MockUserRoleRepo::new();

    let role_id = Uuid::new_v4();
    let service = create_test_service(user_repo, role_repo, user_role_repo);
    let result = service.update_role(role_id, "   ").await;

    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), UserServiceError::Validation(_)));
}
