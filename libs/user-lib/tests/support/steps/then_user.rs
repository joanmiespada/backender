use cucumber::then;

use user_lib::errors_service::UserServiceError;

use crate::support::world::TestWorld;

#[then("the user should be created successfully")]
pub async fn user_created_successfully(world: &mut TestWorld) {
    assert!(world.user_result.as_ref().unwrap().is_ok(), "User creation should succeed");
    world.current_user = world.user_result.as_ref().unwrap().as_ref().ok().cloned();
}

#[then(expr = "the user should have name {string}")]
pub async fn user_has_name(world: &mut TestWorld, expected_name: String) {
    let user = world.current_user.as_ref().expect("User should exist");
    assert_eq!(user.name, expected_name);
}

#[then(expr = "the user should have email {string}")]
pub async fn user_has_email(world: &mut TestWorld, expected_email: String) {
    let user = world.current_user.as_ref().expect("User should exist");
    assert_eq!(user.email, expected_email);
}

#[then("the user should have no roles")]
pub async fn user_has_no_roles(world: &mut TestWorld) {
    let user = world.current_user.as_ref().expect("User should exist");
    assert!(user.roles.is_empty());
}

#[then("I should receive an email already exists error")]
pub async fn email_already_exists_error(world: &mut TestWorld) {
    assert!(matches!(world.error, Some(UserServiceError::EmailAlreadyExists)));
}

#[then("the user should be found")]
pub async fn user_found(world: &mut TestWorld) {
    let result = world.optional_user_result.as_ref().expect("Result should exist");
    assert!(result.is_ok());
    assert!(result.as_ref().unwrap().is_some());
    world.current_user = result.as_ref().unwrap().clone();
}

#[then("the user should not be found")]
pub async fn user_not_found(world: &mut TestWorld) {
    let result = world.optional_user_result.as_ref().expect("Result should exist");
    assert!(result.is_ok());
    assert!(result.as_ref().unwrap().is_none());
}

#[then("the update should be successful")]
pub async fn update_successful(world: &mut TestWorld) {
    assert!(world.user_result.as_ref().unwrap().is_ok());
    world.current_user = world.user_result.as_ref().unwrap().as_ref().ok().cloned();
}

#[then("the deletion should be successful")]
pub async fn deletion_successful(world: &mut TestWorld) {
    assert!(world.delete_result.as_ref().unwrap().is_ok());
}

#[then("the user should no longer exist")]
pub async fn user_no_longer_exists(world: &mut TestWorld) {
    world.current_user = None;
}

#[then(expr = "I should receive {int} users")]
pub async fn receive_users_count(world: &mut TestWorld, count: usize) {
    let result = world.paginated_users_result.as_ref().expect("Result should exist");
    assert!(result.is_ok());
    let paginated = result.as_ref().unwrap();
    assert_eq!(paginated.items.len(), count);
}

#[then(expr = "the total count should be {int}")]
pub async fn total_count(world: &mut TestWorld, count: u64) {
    let result = world.paginated_users_result.as_ref().expect("Result should exist");
    let paginated = result.as_ref().unwrap();
    assert_eq!(paginated.total, count);
}

#[then(expr = "the total pages should be {int}")]
pub async fn total_pages(world: &mut TestWorld, pages: u32) {
    let result = world.paginated_users_result.as_ref().expect("Result should exist");
    let paginated = result.as_ref().unwrap();
    assert_eq!(paginated.total_pages, pages);
}

#[then("I should receive a validation error")]
pub async fn validation_error(world: &mut TestWorld) {
    assert!(matches!(world.error, Some(UserServiceError::Validation(_))));
}

#[then(expr = "the error message should contain {string}")]
pub async fn error_message_contains(world: &mut TestWorld, expected: String) {
    let error = world.error.as_ref().expect("Error should exist");
    assert!(error.to_string().contains(&expected),
        "Error '{}' should contain '{}'", error, expected);
}

#[then(expr = "I should receive {int} users with that role")]
pub async fn receive_users_with_role_count(world: &mut TestWorld, count: usize) {
    let result = world.paginated_users_result.as_ref().expect("Result should exist");
    assert!(result.is_ok());
    let paginated = result.as_ref().unwrap();
    assert_eq!(paginated.items.len(), count);
}
