// API v1 routes (nested under /v1)
pub const USERS_PATH: &str = "/users";
pub const USERS_BY_ID_PATH: &str = "/users/{id}";
pub const USER_ROLES_PATH: &str = "/users/{user_id}/roles/{role_id}";
pub const ROLES_PATH: &str = "/roles";
pub const ROLES_BY_ID_PATH: &str = "/roles/{id}";

// Root-level service routes (not versioned)
pub const SERVICE_HEALTH_PATH: &str = "/health";
pub const SERVICE_DOCS_PATH: &str = "/docs";

// API version prefix
pub const API_V1_PREFIX: &str = "/v1";
