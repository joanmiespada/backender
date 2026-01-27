use cucumber::given;
use uuid::Uuid;

use user_lib::entities::{Role, User};
use user_lib::repository::models::{RoleRow, UserRow};

use crate::support::world::TestWorld;

#[given("a clean user database")]
pub async fn clean_database(world: &mut TestWorld) {
    *world = TestWorld::default();
}

#[given(expr = "a user exists with keycloak_id {string}")]
pub async fn user_exists(world: &mut TestWorld, keycloak_id: String) {
    let user_id = Uuid::new_v4();
    world.current_user_id = Some(user_id);
    world.current_user = Some(User {
        id: user_id,
        keycloak_id: keycloak_id.clone(),
        roles: vec![],
    });
    world.stored_users.push(UserRow {
        id: user_id.to_string(),
        keycloak_id,
    });
}

#[given(expr = "a role exists with name {string}")]
pub async fn role_exists(world: &mut TestWorld, name: String) {
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
pub async fn user_has_role(world: &mut TestWorld, role_name: String) {
    if let (Some(user), Some(role_id)) = (&mut world.current_user, world.current_role_id) {
        user.roles.push(Role {
            id: role_id,
            name: role_name,
        });
    }
}

#[given("the following users exist:")]
pub async fn users_exist(world: &mut TestWorld, step: &cucumber::gherkin::Step) {
    if let Some(table) = &step.table {
        for row in table.rows.iter().skip(1) {
            let keycloak_id = row.first().map(|s| s.as_str()).unwrap_or("");
            world.stored_users.push(UserRow {
                id: Uuid::new_v4().to_string(),
                keycloak_id: keycloak_id.to_string(),
            });
        }
    }
}

#[given("the following roles exist:")]
pub async fn roles_exist(world: &mut TestWorld, step: &cucumber::gherkin::Step) {
    if let Some(table) = &step.table {
        for row in table.rows.iter().skip(1) {
            let name = row.first().map(|s| s.as_str()).unwrap_or("");
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
pub async fn users_have_role(
    world: &mut TestWorld,
    _role_name: String,
    step: &cucumber::gherkin::Step,
) {
    world.users_with_current_role.clear();

    if let Some(table) = &step.table {
        for row in table.rows.iter().skip(1) {
            let keycloak_id = row.first().map(|s| s.as_str()).unwrap_or("");
            let user_row = UserRow {
                id: Uuid::new_v4().to_string(),
                keycloak_id: keycloak_id.to_string(),
            };
            world.stored_users.push(user_row.clone());
            world.users_with_current_role.push(user_row);
        }
    }
}
