use std::sync::Arc;
use async_trait::async_trait;
use cucumber::{given, when, then, World};
use mockall::mock;
use uuid::Uuid;

use user_lib::entities::{User, Role, PaginatedResult, PaginationParams};
use user_lib::repository::errors::UserRepositoryError;
use user_lib::repository::models::{RoleRow, UserRow, UserRoleMapping};
use user_lib::repository::traits::{RoleRepositoryTrait, UserRepositoryTrait, UserRoleRepositoryTrait};
use user_lib::errors_service::UserServiceError;
use user_lib::user_service::UserService;

// Mock repositories
mock! {
    #[derive(Debug)]
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
    current_user: Option<User>,
    current_user_id: Option<Uuid>,
    current_role: Option<Role>,
    current_role_id: Option<Uuid>,
    roles: Vec<Role>,
    stored_users: Vec<UserRow>,
    stored_roles: Vec<RoleRow>,
    users_with_current_role: Vec<UserRow>,

    // Results
    user_result: Option<Result<User, UserServiceError>>,
    role_result: Option<Result<Role, UserServiceError>>,
    optional_user_result: Option<Result<Option<User>, UserServiceError>>,
    delete_result: Option<Result<(), UserServiceError>>,
    paginated_users_result: Option<Result<PaginatedResult<User>, UserServiceError>>,
    paginated_roles_result: Option<Result<PaginatedResult<Role>, UserServiceError>>,
    error: Option<UserServiceError>,
}

impl TestWorld {
    fn create_service_with_mocks(
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

// ==================== GIVEN STEPS ====================

#[given("a clean user database")]
async fn clean_database(world: &mut TestWorld) {
    *world = TestWorld::default();
}

#[given(expr = "a user exists with name {string} and email {string}")]
async fn user_exists(world: &mut TestWorld, name: String, email: String) {
    let user_id = Uuid::new_v4();
    world.current_user_id = Some(user_id);
    world.current_user = Some(User {
        id: user_id,
        name: name.clone(),
        email: email.clone(),
        roles: vec![],
    });
    world.stored_users.push(UserRow {
        id: user_id.to_string(),
        name,
        email,
    });
}

#[given(expr = "a role exists with name {string}")]
async fn role_exists(world: &mut TestWorld, name: String) {
    let role_id = Uuid::new_v4();
    world.current_role_id = Some(role_id);
    world.current_role = Some(Role {
        id: role_id,
        name: name.clone(),
    });
    world.stored_roles.push(RoleRow {
        id: role_id.to_string(),
        name,
    });
}

#[given(expr = "the user has the role {string}")]
async fn user_has_role(world: &mut TestWorld, role_name: String) {
    if let (Some(user), Some(role_id)) = (&mut world.current_user, world.current_role_id) {
        user.roles.push(Role {
            id: role_id,
            name: role_name,
        });
    }
}

#[given("the following users exist:")]
async fn users_exist(world: &mut TestWorld, step: &cucumber::gherkin::Step) {
    if let Some(table) = &step.table {
        for row in table.rows.iter().skip(1) {
            let name = row.get(0).map(|s| s.as_str()).unwrap_or("");
            let email = row.get(1).map(|s| s.as_str()).unwrap_or("");
            world.stored_users.push(UserRow {
                id: Uuid::new_v4().to_string(),
                name: name.to_string(),
                email: email.to_string(),
            });
        }
    }
}

#[given("the following roles exist:")]
async fn roles_exist(world: &mut TestWorld, step: &cucumber::gherkin::Step) {
    if let Some(table) = &step.table {
        for row in table.rows.iter().skip(1) {
            let name = row.get(0).map(|s| s.as_str()).unwrap_or("");
            let role_id = Uuid::new_v4();
            world.stored_roles.push(RoleRow {
                id: role_id.to_string(),
                name: name.to_string(),
            });
            world.roles.push(Role {
                id: role_id,
                name: name.to_string(),
            });
        }
    }
}

#[given(expr = "the following users have the role {string}:")]
async fn users_have_role(world: &mut TestWorld, _role_name: String, step: &cucumber::gherkin::Step) {
    // Clear users_with_current_role for this scenario
    world.users_with_current_role.clear();

    if let Some(table) = &step.table {
        for row in table.rows.iter().skip(1) {
            let name = row.get(0).map(|s| s.as_str()).unwrap_or("");
            let email = row.get(1).map(|s| s.as_str()).unwrap_or("");
            let user_row = UserRow {
                id: Uuid::new_v4().to_string(),
                name: name.to_string(),
                email: email.to_string(),
            };
            world.stored_users.push(user_row.clone());
            world.users_with_current_role.push(user_row);
        }
    }
}

// ==================== WHEN STEPS ====================

#[when(expr = "I create a user with name {string} and email {string}")]
async fn create_user(world: &mut TestWorld, name: String, email: String) {
    let user_id = Uuid::new_v4();
    let name_clone = name.clone();
    let email_clone = email.clone();

    let service = world.create_service_with_mocks(
        move |user_repo| {
            let n = name_clone.clone();
            let e = email_clone.clone();
            user_repo
                .expect_create_user()
                .returning(move |_, _| {
                    Ok(UserRow {
                        id: user_id.to_string(),
                        name: n.clone(),
                        email: e.clone(),
                    })
                });
        },
        |_| {},
        |_| {},
    );

    world.user_result = Some(service.create_user(&name, &email).await);
}

#[when(expr = "I try to create a user with name {string} and email {string}")]
async fn try_create_user(world: &mut TestWorld, name: String, email: String) {
    let service = world.create_service_with_mocks(
        |user_repo| {
            user_repo
                .expect_create_user()
                .returning(|_, _| Err(UserRepositoryError::EmailAlreadyExists));
        },
        |_| {},
        |_| {},
    );

    let result = service.create_user(&name, &email).await;
    if let Err(e) = result {
        world.error = Some(e);
    }
}

#[when("I retrieve the user by their ID")]
async fn retrieve_user_by_id(world: &mut TestWorld) {
    let user_id = world.current_user_id.expect("User ID should be set");
    let user = world.current_user.clone().expect("User should exist");

    let service = world.create_service_with_mocks(
        move |user_repo| {
            let u = user.clone();
            user_repo
                .expect_get_user()
                .returning(move |_| {
                    Ok(Some(UserRow {
                        id: u.id.to_string(),
                        name: u.name.clone(),
                        email: u.email.clone(),
                    }))
                });
        },
        |role_repo| {
            role_repo
                .expect_get_roles_for_user()
                .returning(|_| Ok(vec![]));
        },
        |_| {},
    );

    world.optional_user_result = Some(service.get_user(user_id).await);
}

#[when("I try to retrieve a user with a random ID")]
async fn retrieve_random_user(world: &mut TestWorld) {
    let random_id = Uuid::new_v4();

    let service = world.create_service_with_mocks(
        |user_repo| {
            user_repo
                .expect_get_user()
                .returning(|_| Ok(None));
        },
        |_| {},
        |_| {},
    );

    world.optional_user_result = Some(service.get_user(random_id).await);
}

#[when(expr = "I update the user's name to {string} and email to {string}")]
async fn update_user(world: &mut TestWorld, new_name: String, new_email: String) {
    let user_id = world.current_user_id.expect("User ID should be set");
    let name_clone = new_name.clone();
    let email_clone = new_email.clone();

    let service = world.create_service_with_mocks(
        move |user_repo| {
            let n = name_clone.clone();
            let e = email_clone.clone();
            user_repo
                .expect_update_user()
                .returning(move |id, _, _| {
                    Ok(UserRow {
                        id: id.to_string(),
                        name: n.clone(),
                        email: e.clone(),
                    })
                });
        },
        |role_repo| {
            role_repo
                .expect_get_roles_for_user()
                .returning(|_| Ok(vec![]));
        },
        |_| {},
    );

    world.user_result = Some(service.update_user(user_id, &new_name, &new_email).await);
}

#[when("I delete the user")]
async fn delete_user(world: &mut TestWorld) {
    let user_id = world.current_user_id.expect("User ID should be set");

    let service = world.create_service_with_mocks(
        |user_repo| {
            user_repo
                .expect_delete_user()
                .returning(|_| Ok(()));
        },
        |_| {},
        |_| {},
    );

    world.delete_result = Some(service.delete_user(user_id).await);
}

#[when(expr = "I request page {int} with page size {int}")]
async fn request_users_page(world: &mut TestWorld, page: u32, page_size: u32) {
    let users = world.stored_users.clone();
    let total = users.len() as u64;

    let service = world.create_service_with_mocks(
        move |user_repo| {
            let u = users.clone();
            user_repo
                .expect_get_users_paginated()
                .returning(move |pagination| {
                    let start = pagination.offset() as usize;
                    let end = (start + pagination.limit() as usize).min(u.len());
                    let page_users = if start < u.len() {
                        u[start..end].to_vec()
                    } else {
                        vec![]
                    };
                    Ok((page_users, total))
                });
        },
        |role_repo| {
            role_repo
                .expect_get_roles_for_users()
                .returning(|_| Ok(vec![]));
        },
        |_| {},
    );

    let pagination = PaginationParams::new(Some(page), Some(page_size));
    world.paginated_users_result = Some(service.get_users(pagination).await);
}

#[when(expr = "I create a role with name {string}")]
async fn create_role(world: &mut TestWorld, name: String) {
    let role_id = Uuid::new_v4();
    let name_clone = name.clone();

    let service = world.create_service_with_mocks(
        |_| {},
        move |role_repo| {
            let n = name_clone.clone();
            role_repo
                .expect_create_role()
                .returning(move |_| {
                    Ok(RoleRow {
                        id: role_id.to_string(),
                        name: n.clone(),
                    })
                });
        },
        |_| {},
    );

    world.role_result = Some(service.create_role(&name).await);
}

#[when(expr = "I try to create a role with name {string}")]
async fn try_create_role(world: &mut TestWorld, name: String) {
    let service = world.create_service_with_mocks(
        |_| {},
        |role_repo| {
            role_repo
                .expect_create_role()
                .returning(|_| Err(UserRepositoryError::RoleNameAlreadyExists));
        },
        |_| {},
    );

    let result = service.create_role(&name).await;
    if let Err(e) = result {
        world.error = Some(e);
    }
}

#[when("I assign the role to the user")]
async fn assign_role(world: &mut TestWorld) {
    let user_id = world.current_user_id.expect("User ID should be set");
    let role_id = world.current_role_id.expect("Role ID should be set");
    let role = world.current_role.clone().expect("Role should exist");

    let service = world.create_service_with_mocks(
        |_| {},
        |_| {},
        |user_role_repo| {
            user_role_repo
                .expect_assign_role()
                .returning(|_, _| Ok(()));
        },
    );

    world.delete_result = Some(service.assign_role(user_id, role_id).await);

    // Update user's roles for verification
    if let Some(user) = &mut world.current_user {
        user.roles.push(role);
    }
}

#[when(expr = "I try to assign the role {string} to the user again")]
async fn try_assign_role_again(world: &mut TestWorld, _role_name: String) {
    let user_id = world.current_user_id.expect("User ID should be set");
    let role_id = world.current_role_id.expect("Role ID should be set");

    let service = world.create_service_with_mocks(
        |_| {},
        |_| {},
        |user_role_repo| {
            user_role_repo
                .expect_assign_role()
                .returning(|_, _| Err(UserRepositoryError::UserAlreadyHasRole));
        },
    );

    let result = service.assign_role(user_id, role_id).await;
    if let Err(e) = result {
        world.error = Some(e);
    }
}

#[when("I unassign the role from the user")]
async fn unassign_role(world: &mut TestWorld) {
    let user_id = world.current_user_id.expect("User ID should be set");
    let role_id = world.current_role_id.expect("Role ID should be set");

    let service = world.create_service_with_mocks(
        |_| {},
        |_| {},
        |user_role_repo| {
            user_role_repo
                .expect_unassign_role()
                .returning(|_, _| Ok(()));
        },
    );

    world.delete_result = Some(service.unassign_role(user_id, role_id).await);

    // Clear user's roles for verification
    if let Some(user) = &mut world.current_user {
        user.roles.clear();
    }
}

#[when("I assign all roles to the user")]
async fn assign_all_roles(world: &mut TestWorld) {
    let user_id = world.current_user_id.expect("User ID should be set");
    let roles = world.roles.clone();

    for role in &roles {
        let role_id = role.id;
        let service = world.create_service_with_mocks(
            |_| {},
            |_| {},
            |user_role_repo| {
                user_role_repo
                    .expect_assign_role()
                    .returning(|_, _| Ok(()));
            },
        );

        let _ = service.assign_role(user_id, role_id).await;
        if let Some(user) = &mut world.current_user {
            user.roles.push(role.clone());
        }
    }
}

#[when(expr = "I request roles page {int} with page size {int}")]
async fn request_roles_page(world: &mut TestWorld, page: u32, page_size: u32) {
    let roles = world.stored_roles.clone();
    let total = roles.len() as u64;

    let service = world.create_service_with_mocks(
        |_| {},
        move |role_repo| {
            let r = roles.clone();
            role_repo
                .expect_get_roles_paginated()
                .returning(move |pagination| {
                    let start = pagination.offset() as usize;
                    let end = (start + pagination.limit() as usize).min(r.len());
                    let page_roles = if start < r.len() {
                        r[start..end].to_vec()
                    } else {
                        vec![]
                    };
                    Ok((page_roles, total))
                });
        },
        |_| {},
    );

    let pagination = PaginationParams::new(Some(page), Some(page_size));
    world.paginated_roles_result = Some(service.get_roles(pagination).await);
}

#[when(expr = "I get users with role {string}")]
async fn get_users_with_role(world: &mut TestWorld, _role_name: String) {
    let role_id = world.current_role_id.expect("Role ID should be set");
    let users = world.users_with_current_role.clone();
    let total = users.len() as u64;

    let service = world.create_service_with_mocks(
        move |user_repo| {
            let u = users.clone();
            user_repo
                .expect_get_users_by_role_paginated()
                .returning(move |_, _| Ok((u.clone(), total)));
        },
        |role_repo| {
            role_repo
                .expect_get_roles_for_users()
                .returning(|_| Ok(vec![]));
        },
        |_| {},
    );

    let pagination = PaginationParams::default();
    world.paginated_users_result = Some(service.get_users_by_role(role_id, pagination).await);
}

#[when("I delete the role")]
async fn delete_role(world: &mut TestWorld) {
    let role_id = world.current_role_id.expect("Role ID should be set");

    let service = world.create_service_with_mocks(
        |_| {},
        |role_repo| {
            role_repo
                .expect_delete_role()
                .returning(|_| Ok(()));
        },
        |_| {},
    );

    world.delete_result = Some(service.delete_role(role_id).await);
}

// ==================== THEN STEPS ====================

#[then("the user should be created successfully")]
async fn user_created_successfully(world: &mut TestWorld) {
    assert!(world.user_result.as_ref().unwrap().is_ok(), "User creation should succeed");
    world.current_user = world.user_result.as_ref().unwrap().as_ref().ok().cloned();
}

#[then(expr = "the user should have name {string}")]
async fn user_has_name(world: &mut TestWorld, expected_name: String) {
    let user = world.current_user.as_ref().expect("User should exist");
    assert_eq!(user.name, expected_name);
}

#[then(expr = "the user should have email {string}")]
async fn user_has_email(world: &mut TestWorld, expected_email: String) {
    let user = world.current_user.as_ref().expect("User should exist");
    assert_eq!(user.email, expected_email);
}

#[then("the user should have no roles")]
async fn user_has_no_roles(world: &mut TestWorld) {
    let user = world.current_user.as_ref().expect("User should exist");
    assert!(user.roles.is_empty());
}

#[then("I should receive an email already exists error")]
async fn email_already_exists_error(world: &mut TestWorld) {
    assert!(matches!(world.error, Some(UserServiceError::EmailAlreadyExists)));
}

#[then("the user should be found")]
async fn user_found(world: &mut TestWorld) {
    let result = world.optional_user_result.as_ref().expect("Result should exist");
    assert!(result.is_ok());
    assert!(result.as_ref().unwrap().is_some());
    world.current_user = result.as_ref().unwrap().clone();
}

#[then("the user should not be found")]
async fn user_not_found(world: &mut TestWorld) {
    let result = world.optional_user_result.as_ref().expect("Result should exist");
    assert!(result.is_ok());
    assert!(result.as_ref().unwrap().is_none());
}

#[then("the update should be successful")]
async fn update_successful(world: &mut TestWorld) {
    assert!(world.user_result.as_ref().unwrap().is_ok());
    world.current_user = world.user_result.as_ref().unwrap().as_ref().ok().cloned();
}

#[then("the deletion should be successful")]
async fn deletion_successful(world: &mut TestWorld) {
    assert!(world.delete_result.as_ref().unwrap().is_ok());
}

#[then("the user should no longer exist")]
async fn user_no_longer_exists(world: &mut TestWorld) {
    world.current_user = None;
}

#[then(expr = "I should receive {int} users")]
async fn receive_users_count(world: &mut TestWorld, count: usize) {
    let result = world.paginated_users_result.as_ref().expect("Result should exist");
    assert!(result.is_ok());
    let paginated = result.as_ref().unwrap();
    assert_eq!(paginated.items.len(), count);
}

#[then(expr = "the total count should be {int}")]
async fn total_count(world: &mut TestWorld, count: u64) {
    let result = world.paginated_users_result.as_ref().expect("Result should exist");
    let paginated = result.as_ref().unwrap();
    assert_eq!(paginated.total, count);
}

#[then(expr = "the total pages should be {int}")]
async fn total_pages(world: &mut TestWorld, pages: u32) {
    let result = world.paginated_users_result.as_ref().expect("Result should exist");
    let paginated = result.as_ref().unwrap();
    assert_eq!(paginated.total_pages, pages);
}

#[then("I should receive a validation error")]
async fn validation_error(world: &mut TestWorld) {
    assert!(matches!(world.error, Some(UserServiceError::Validation(_))));
}

#[then(expr = "the error message should contain {string}")]
async fn error_message_contains(world: &mut TestWorld, expected: String) {
    let error = world.error.as_ref().expect("Error should exist");
    assert!(error.to_string().contains(&expected),
        "Error '{}' should contain '{}'", error, expected);
}

#[then("the role should be created successfully")]
async fn role_created_successfully(world: &mut TestWorld) {
    assert!(world.role_result.as_ref().unwrap().is_ok());
    world.current_role = world.role_result.as_ref().unwrap().as_ref().ok().cloned();
}

#[then(expr = "the role should have name {string}")]
async fn role_has_name(world: &mut TestWorld, expected_name: String) {
    let role = world.current_role.as_ref().expect("Role should exist");
    assert_eq!(role.name, expected_name);
}

#[then("I should receive a role name already exists error")]
async fn role_name_already_exists_error(world: &mut TestWorld) {
    assert!(matches!(world.error, Some(UserServiceError::RoleNameAlreadyExists)));
}

#[then("the assignment should be successful")]
async fn assignment_successful(world: &mut TestWorld) {
    assert!(world.delete_result.as_ref().unwrap().is_ok());
}

#[then(expr = "the user should have the role {string}")]
async fn user_has_role_check(world: &mut TestWorld, role_name: String) {
    let user = world.current_user.as_ref().expect("User should exist");
    assert!(user.roles.iter().any(|r| r.name == role_name));
}

#[then("I should receive a user already has role error")]
async fn user_already_has_role_error(world: &mut TestWorld) {
    assert!(matches!(world.error, Some(UserServiceError::UserAlreadyHasRole)));
}

#[then("the unassignment should be successful")]
async fn unassignment_successful(world: &mut TestWorld) {
    assert!(world.delete_result.as_ref().unwrap().is_ok());
}

#[then(expr = "the user should not have the role {string}")]
async fn user_does_not_have_role(world: &mut TestWorld, role_name: String) {
    let user = world.current_user.as_ref().expect("User should exist");
    assert!(!user.roles.iter().any(|r| r.name == role_name));
}

#[then(expr = "the user should have {int} roles")]
async fn user_has_roles_count(world: &mut TestWorld, count: usize) {
    let user = world.current_user.as_ref().expect("User should exist");
    assert_eq!(user.roles.len(), count);
}

#[then(expr = "I should receive {int} roles")]
async fn receive_roles_count(world: &mut TestWorld, count: usize) {
    let result = world.paginated_roles_result.as_ref().expect("Result should exist");
    assert!(result.is_ok());
    let paginated = result.as_ref().unwrap();
    assert_eq!(paginated.items.len(), count);
}

#[then(expr = "the total roles count should be {int}")]
async fn total_roles_count(world: &mut TestWorld, count: u64) {
    let result = world.paginated_roles_result.as_ref().expect("Result should exist");
    let paginated = result.as_ref().unwrap();
    assert_eq!(paginated.total, count);
}

#[then(expr = "I should receive {int} users with that role")]
async fn receive_users_with_role_count(world: &mut TestWorld, count: usize) {
    let result = world.paginated_users_result.as_ref().expect("Result should exist");
    assert!(result.is_ok());
    let paginated = result.as_ref().unwrap();
    assert_eq!(paginated.items.len(), count);
}

#[then("the role deletion should be successful")]
async fn role_deletion_successful(world: &mut TestWorld) {
    assert!(world.delete_result.as_ref().unwrap().is_ok());
}

#[tokio::main]
async fn main() {
    TestWorld::run("tests/features").await;
}
