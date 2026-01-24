use utoipa::OpenApi;

use user_api::methods::entities::{
    CreateUserRequest, UpdateUserRequest, UserResponse,
    CreateRoleRequest, UpdateRoleRequest, RoleResponse,
    PaginatedResponse
};

#[derive(OpenApi)]
#[openapi(
    paths(
        user_api::methods::create_user::create_user,
        user_api::methods::get_user_by_id::get_user_by_id,
        user_api::methods::get_users::get_users,
        user_api::methods::update_user::update_user,
        user_api::methods::delete_user::delete_user,
        user_api::methods::create_role::create_role,
        user_api::methods::get_role_by_id::get_role_by_id,
        user_api::methods::get_roles::get_roles,
        user_api::methods::update_role::update_role,
        user_api::methods::delete_role::delete_role,
        user_api::methods::assign_role::assign_role,
        user_api::methods::unassign_role::unassign_role
    ),
    components(schemas(
        CreateUserRequest, UpdateUserRequest, UserResponse,
        CreateRoleRequest, UpdateRoleRequest, RoleResponse,
        PaginatedResponse<UserResponse>, PaginatedResponse<RoleResponse>
    )),
    tags(
        (name = "users", description = "User management endpoints"),
        (name = "roles", description = "Role management endpoints")
    )
)]
struct ApiDoc;

#[test]
fn test_openapi_spec_has_all_endpoints() {
    let spec = ApiDoc::openapi();
    let json = spec.to_pretty_json().expect("Failed to generate OpenAPI JSON");

    // Verify paths exist
    let paths = spec.paths.paths;

    // User endpoints
    assert!(paths.contains_key("/users"), "Missing /users path");
    assert!(paths.contains_key("/users/{id}"), "Missing /users/{{id}} path");

    // Role endpoints
    assert!(paths.contains_key("/roles"), "Missing /roles path");
    assert!(paths.contains_key("/roles/{id}"), "Missing /roles/{{id}} path");

    // User-role assignment endpoints
    assert!(paths.contains_key("/users/{user_id}/roles/{role_id}"), "Missing user-role assignment path");

    // Verify HTTP methods for /users
    let users_path = paths.get("/users").unwrap();
    assert!(users_path.get.is_some(), "Missing GET /users");
    assert!(users_path.post.is_some(), "Missing POST /users");

    // Verify HTTP methods for /users/{id}
    let user_by_id_path = paths.get("/users/{id}").unwrap();
    assert!(user_by_id_path.get.is_some(), "Missing GET /users/{{id}}");
    assert!(user_by_id_path.put.is_some(), "Missing PUT /users/{{id}}");
    assert!(user_by_id_path.delete.is_some(), "Missing DELETE /users/{{id}}");

    // Verify HTTP methods for /roles
    let roles_path = paths.get("/roles").unwrap();
    assert!(roles_path.get.is_some(), "Missing GET /roles");
    assert!(roles_path.post.is_some(), "Missing POST /roles");

    // Verify HTTP methods for /roles/{id}
    let role_by_id_path = paths.get("/roles/{id}").unwrap();
    assert!(role_by_id_path.get.is_some(), "Missing GET /roles/{{id}}");
    assert!(role_by_id_path.put.is_some(), "Missing PUT /roles/{{id}}");
    assert!(role_by_id_path.delete.is_some(), "Missing DELETE /roles/{{id}}");

    // Verify user-role assignment methods
    let user_roles_path = paths.get("/users/{user_id}/roles/{role_id}").unwrap();
    assert!(user_roles_path.post.is_some(), "Missing POST /users/{{user_id}}/roles/{{role_id}}");
    assert!(user_roles_path.delete.is_some(), "Missing DELETE /users/{{user_id}}/roles/{{role_id}}");

    // Verify schemas exist
    let schemas = &spec.components.as_ref().unwrap().schemas;
    assert!(schemas.contains_key("CreateUserRequest"), "Missing CreateUserRequest schema");
    assert!(schemas.contains_key("UpdateUserRequest"), "Missing UpdateUserRequest schema");
    assert!(schemas.contains_key("UserResponse"), "Missing UserResponse schema");
    assert!(schemas.contains_key("CreateRoleRequest"), "Missing CreateRoleRequest schema");
    assert!(schemas.contains_key("UpdateRoleRequest"), "Missing UpdateRoleRequest schema");
    assert!(schemas.contains_key("RoleResponse"), "Missing RoleResponse schema");

    // Print the full spec for manual verification
    println!("OpenAPI Spec:\n{}", json);
}

#[test]
fn test_openapi_json_contains_tags() {
    let spec = ApiDoc::openapi();
    let json = spec.to_pretty_json().expect("Failed to generate OpenAPI JSON");

    // Check tags are present in the JSON
    assert!(json.contains("\"users\""), "Missing 'users' tag in JSON");
    assert!(json.contains("\"roles\""), "Missing 'roles' tag in JSON");
}
