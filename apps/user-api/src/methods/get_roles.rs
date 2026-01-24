use axum::{extract::Query, Json};
use crate::error::{ApiError, handle_service_error};
use crate::methods::entities::{PaginatedResponse, PaginationQuery, RoleResponse};
use crate::state::AppState;
use crate::methods::routes::ROLES_PATH;

#[utoipa::path(
    get,
    path = ROLES_PATH,
    tag = "roles",
    params(PaginationQuery),
    responses(
        (status = 200, description = "List of roles", body = PaginatedResponse<RoleResponse>),
        (status = 500, description = "Internal server error"),
    )
)]
pub async fn get_roles(
    axum::extract::State(state): axum::extract::State<AppState>,
    Query(pagination): Query<PaginationQuery>,
) -> Result<Json<PaginatedResponse<RoleResponse>>, ApiError> {
    state.user_service
        .get_roles(pagination.into())
        .await
        .map(|result| Json(PaginatedResponse::from(result)))
        .map_err(|e| handle_service_error(e, &state.env, "get_roles"))
}
