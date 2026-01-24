use serde::{Deserialize, Serialize};
use user_lib::entities::{User, Role, PaginatedResult, PaginationParams};
use uuid::Uuid;
use utoipa::{ToSchema, IntoParams};

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct CreateUserRequest {
    pub name: String,
    pub email: String,
}

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct UpdateUserRequest {
    pub name: String,
    pub email: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct UserResponse {
    pub id: Uuid,
    pub name: String,
    pub email: String,
    pub roles: Vec<RoleResponse>,
}

impl From<User> for UserResponse {
    fn from(user: User) -> Self {
        UserResponse {
            id: user.id,
            name: user.name,
            email: user.email,
            roles: user.roles.into_iter().map(RoleResponse::from).collect(),
        }
    }
}

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct CreateRoleRequest {
    pub name: String,
}

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct UpdateRoleRequest {
    pub name: String,
}

#[derive(Debug, Serialize, ToSchema, Clone)]
pub struct RoleResponse {
    pub id: Uuid,
    pub name: String,
}

impl From<Role> for RoleResponse {
    fn from(role: Role) -> Self {
        RoleResponse {
            id: role.id,
            name: role.name,
        }
    }
}

#[derive(Debug, Deserialize, IntoParams)]
pub struct PaginationQuery {
    pub page: Option<u32>,
    pub page_size: Option<u32>,
}

impl From<PaginationQuery> for PaginationParams {
    fn from(query: PaginationQuery) -> Self {
        PaginationParams::new(query.page, query.page_size)
    }
}

#[derive(Debug, Serialize, ToSchema)]
pub struct PaginatedResponse<T> {
    pub items: Vec<T>,
    pub total: u64,
    pub page: u32,
    pub page_size: u32,
    pub total_pages: u32,
}

impl<T, U> From<PaginatedResult<T>> for PaginatedResponse<U>
where
    U: From<T>,
{
    fn from(result: PaginatedResult<T>) -> Self {
        PaginatedResponse {
            items: result.items.into_iter().map(U::from).collect(),
            total: result.total,
            page: result.page,
            page_size: result.page_size,
            total_pages: result.total_pages,
        }
    }
}

