use cucumber::then;

use user_lib::errors_service::UserServiceError;

use crate::support::world::TestWorld;

#[then("the role should be created successfully")]
pub async fn role_created_successfully(world: &mut TestWorld) {
    assert!(world.role_result.as_ref().unwrap().is_ok());
    world.current_role = world.role_result.as_ref().unwrap().as_ref().ok().cloned();
}

#[then(expr = "the role should have name {string}")]
pub async fn role_has_name(world: &mut TestWorld, expected_name: String) {
    let role = world.current_role.as_ref().expect("Role should exist");
    assert_eq!(role.name, expected_name);
}

#[then("I should receive a role name already exists error")]
pub async fn role_name_already_exists_error(world: &mut TestWorld) {
    assert!(matches!(world.error, Some(UserServiceError::RoleNameAlreadyExists)));
}

#[then("the assignment should be successful")]
pub async fn assignment_successful(world: &mut TestWorld) {
    assert!(world.delete_result.as_ref().unwrap().is_ok());
}

#[then(expr = "the user should have the role {string}")]
pub async fn user_has_role_check(world: &mut TestWorld, role_name: String) {
    let user = world.current_user.as_ref().expect("User should exist");
    assert!(user.roles.iter().any(|r| r.name == role_name));
}

#[then("I should receive a user already has role error")]
pub async fn user_already_has_role_error(world: &mut TestWorld) {
    assert!(matches!(world.error, Some(UserServiceError::UserAlreadyHasRole)));
}

#[then("the unassignment should be successful")]
pub async fn unassignment_successful(world: &mut TestWorld) {
    assert!(world.delete_result.as_ref().unwrap().is_ok());
}

#[then(expr = "the user should not have the role {string}")]
pub async fn user_does_not_have_role(world: &mut TestWorld, role_name: String) {
    let user = world.current_user.as_ref().expect("User should exist");
    assert!(!user.roles.iter().any(|r| r.name == role_name));
}

#[then(expr = "the user should have {int} roles")]
pub async fn user_has_roles_count(world: &mut TestWorld, count: usize) {
    let user = world.current_user.as_ref().expect("User should exist");
    assert_eq!(user.roles.len(), count);
}

#[then(expr = "I should receive {int} roles")]
pub async fn receive_roles_count(world: &mut TestWorld, count: usize) {
    let result = world.paginated_roles_result.as_ref().expect("Result should exist");
    assert!(result.is_ok());
    let paginated = result.as_ref().unwrap();
    assert_eq!(paginated.items.len(), count);
}

#[then(expr = "the total roles count should be {int}")]
pub async fn total_roles_count(world: &mut TestWorld, count: u64) {
    let result = world.paginated_roles_result.as_ref().expect("Result should exist");
    let paginated = result.as_ref().unwrap();
    assert_eq!(paginated.total, count);
}

#[then("the role deletion should be successful")]
pub async fn role_deletion_successful(world: &mut TestWorld) {
    assert!(world.delete_result.as_ref().unwrap().is_ok());
}
