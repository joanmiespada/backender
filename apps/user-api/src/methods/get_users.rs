use axum::{extract::Query, Json};
use crate::error::{ApiError, handle_service_error};
use crate::methods::entities::{PaginatedResponse, PaginationQuery, UserResponse};
use crate::state::AppState;
use crate::methods::routes::USERS_PATH;

#[utoipa::path(
    get,
    path = USERS_PATH,
    tag = "users",
    params(PaginationQuery),
    responses(
        (status = 200, description = "List of users", body = PaginatedResponse<UserResponse>),
        (status = 500, description = "Internal server error"),
    )
)]
pub async fn get_users(
    axum::extract::State(state): axum::extract::State<AppState>,
    Query(pagination): Query<PaginationQuery>,
) -> Result<Json<PaginatedResponse<UserResponse>>, ApiError> {
    state.user_service
        .get_users(pagination.into())
        .await
        .map(|result| Json(PaginatedResponse::from(result)))
        .map_err(|e| handle_service_error(e, &state.env, "get_users"))
}
