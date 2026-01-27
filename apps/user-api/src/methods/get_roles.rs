use crate::error::{handle_integrated_service_error, ApiError};
use crate::methods::entities::{PaginatedResponse, PaginationQuery, RoleResponse};
use crate::methods::routes::ROLES_PATH;
use crate::state::AppState;
use axum::{extract::Query, Json};

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
    state
        .user_service
        .get_roles(pagination.into())
        .await
        .map(|result| Json(PaginatedResponse::from(result)))
        .map_err(|e| handle_integrated_service_error(e, &state.env, "get_roles"))
}
