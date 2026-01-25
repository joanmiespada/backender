use uuid::Uuid;

const PREFIX: &str = "user-api";

pub fn user_key(user_id: Uuid) -> String {
    format!("{}:user:{}", PREFIX, user_id)
}

pub fn users_list_key(page: u32, page_size: u32) -> String {
    format!("{}:users:page:{}:size:{}", PREFIX, page, page_size)
}

pub fn role_key(role_id: Uuid) -> String {
    format!("{}:role:{}", PREFIX, role_id)
}

pub fn roles_list_key(page: u32, page_size: u32) -> String {
    format!("{}:roles:page:{}:size:{}", PREFIX, page, page_size)
}

pub fn users_pattern() -> String {
    format!("{}:users:*", PREFIX)
}

pub fn user_pattern() -> String {
    format!("{}:user:*", PREFIX)
}

pub fn roles_pattern() -> String {
    format!("{}:roles:*", PREFIX)
}
