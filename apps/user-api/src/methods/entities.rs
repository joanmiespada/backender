use secrecy::Secret;
use serde::{Deserialize, Serialize};
use user_lib::entities::{PaginatedResult, PaginationParams, Role};
use utoipa::{IntoParams, ToSchema};
use uuid::Uuid;

use crate::keycloak::FullUser;

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct CreateUserRequest {
    pub email: String,
    #[serde(default)]
    pub first_name: Option<String>,
    #[serde(default)]
    pub last_name: Option<String>,
    #[serde(default, skip_serializing)]
    pub password: Option<Secret<String>>,
}

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct UpdateUserRequest {
    #[serde(default)]
    pub first_name: Option<String>,
    #[serde(default)]
    pub last_name: Option<String>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct UserResponse {
    pub id: Uuid,
    pub keycloak_id: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
    pub roles: Vec<RoleResponse>,
    pub email_verified: bool,
    pub enabled: bool,
}

impl From<FullUser> for UserResponse {
    fn from(user: FullUser) -> Self {
        UserResponse {
            id: user.id,
            keycloak_id: user.keycloak_id,
            name: user.name,
            email: user.email,
            roles: user.roles.into_iter().map(RoleResponse::from).collect(),
            email_verified: user.email_verified,
            enabled: user.enabled,
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
