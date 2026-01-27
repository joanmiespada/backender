use sqlx::FromRow;

#[derive(Debug, Clone, FromRow)]
pub struct UserRow {
    pub id: String,
    pub keycloak_id: String,
}

#[derive(Debug, Clone, FromRow)]
pub struct RoleRow {
    pub id: String,
    pub name: String,
}

#[derive(Debug, Clone, FromRow)]
pub struct UserRoleRow {
    pub user_id: String,
    pub role_id: String,
}

#[derive(Debug, Clone, FromRow)]
pub struct UserRoleMapping {
    pub user_id: String,
    pub role_id: String,
    pub role_name: String,
}
