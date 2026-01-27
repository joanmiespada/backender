use cucumber::when;
use uuid::Uuid;

use user_lib::entities::PaginationParams;
use user_lib::repository::errors::UserRepositoryError;
use user_lib::repository::models::RoleRow;

use crate::support::world::TestWorld;

#[when(expr = "I create a role with name {string}")]
pub async fn create_role(world: &mut TestWorld, name: String) {
    let role_id = Uuid::new_v4();
    let name_clone = name.clone();

    let service = world.create_service_with_mocks(
        |_| {},
        move |role_repo| {
            let n = name_clone.clone();
            role_repo.expect_create_role().returning(move |_| {
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
pub async fn try_create_role(world: &mut TestWorld, name: String) {
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
pub async fn assign_role(world: &mut TestWorld) {
    let user_id = world.current_user_id.expect("User ID should be set");
    let role_id = world.current_role_id.expect("Role ID should be set");
    let role = world.current_role.clone().expect("Role should exist");

    let service = world.create_service_with_mocks(
        |_| {},
        |_| {},
        |user_role_repo| {
            user_role_repo.expect_assign_role().returning(|_, _| Ok(()));
        },
    );

    world.delete_result = Some(service.assign_role(user_id, role_id).await);

    if let Some(user) = &mut world.current_user {
        user.roles.push(role);
    }
}

#[when(expr = "I try to assign the role {string} to the user again")]
pub async fn try_assign_role_again(world: &mut TestWorld, _role_name: String) {
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
pub async fn unassign_role(world: &mut TestWorld) {
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

    if let Some(user) = &mut world.current_user {
        user.roles.clear();
    }
}

#[when("I assign all roles to the user")]
pub async fn assign_all_roles(world: &mut TestWorld) {
    let user_id = world.current_user_id.expect("User ID should be set");
    let roles = world.roles.clone();

    for role in &roles {
        let role_id = role.id;
        let service = world.create_service_with_mocks(
            |_| {},
            |_| {},
            |user_role_repo| {
                user_role_repo.expect_assign_role().returning(|_, _| Ok(()));
            },
        );

        let _ = service.assign_role(user_id, role_id).await;
        if let Some(user) = &mut world.current_user {
            user.roles.push(role.clone());
        }
    }
}

#[when(expr = "I request roles page {int} with page size {int}")]
pub async fn request_roles_page(world: &mut TestWorld, page: u32, page_size: u32) {
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

#[when("I delete the role")]
pub async fn delete_role(world: &mut TestWorld) {
    let role_id = world.current_role_id.expect("Role ID should be set");

    let service = world.create_service_with_mocks(
        |_| {},
        |role_repo| {
            role_repo.expect_delete_role().returning(|_| Ok(()));
        },
        |_| {},
    );

    world.delete_result = Some(service.delete_role(role_id).await);
}
