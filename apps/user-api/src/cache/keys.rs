use uuid::Uuid;

const PREFIX: &str = "user-api";

pub fn user_key(user_id: Uuid) -> String {
    format!("{PREFIX}:user:{user_id}")
}

pub fn users_list_key(page: u32, page_size: u32) -> String {
    format!("{PREFIX}:users:page:{page}:size:{page_size}")
}

pub fn role_key(role_id: Uuid) -> String {
    format!("{PREFIX}:role:{role_id}")
}

pub fn roles_list_key(page: u32, page_size: u32) -> String {
    format!("{PREFIX}:roles:page:{page}:size:{page_size}")
}

pub fn users_pattern() -> String {
    format!("{PREFIX}:users:*")
}

pub fn user_pattern() -> String {
    format!("{PREFIX}:user:*")
}

pub fn roles_pattern() -> String {
    format!("{PREFIX}:roles:*")
}

#[allow(dead_code)]
pub fn keycloak_profile_key(keycloak_id: &str) -> String {
    format!("{PREFIX}:kc:profile:{keycloak_id}")
}

#[allow(dead_code)]
pub fn keycloak_profiles_pattern() -> String {
    format!("{PREFIX}:kc:*")
}
