use sqlx::FromRow;

#[derive(Debug, Clone, FromRow)]
pub struct UserRow {
    pub id: String,
    pub name: String,
    pub email: String,
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