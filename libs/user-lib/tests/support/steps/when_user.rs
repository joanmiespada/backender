use cucumber::when;
use uuid::Uuid;

use user_lib::entities::PaginationParams;
use user_lib::repository::errors::UserRepositoryError;
use user_lib::repository::models::UserRow;

use crate::support::world::TestWorld;

#[when(expr = "I create a user with name {string} and email {string}")]
pub async fn create_user(world: &mut TestWorld, name: String, email: String) {
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
pub async fn try_create_user(world: &mut TestWorld, name: String, email: String) {
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
pub async fn retrieve_user_by_id(world: &mut TestWorld) {
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
pub async fn retrieve_random_user(world: &mut TestWorld) {
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
pub async fn update_user(world: &mut TestWorld, new_name: String, new_email: String) {
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
pub async fn delete_user(world: &mut TestWorld) {
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
pub async fn request_users_page(world: &mut TestWorld, page: u32, page_size: u32) {
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

#[when(expr = "I get users with role {string}")]
pub async fn get_users_with_role(world: &mut TestWorld, _role_name: String) {
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
