use crate::error::{handle_integrated_service_error, ApiError};
use crate::methods::entities::{PaginatedResponse, PaginationQuery, UserResponse};
use crate::methods::routes::USERS_PATH;
use crate::state::AppState;
use axum::{extract::Query, Json};

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
    state
        .user_service
        .get_users(pagination.into())
        .await
        .map(|result| Json(PaginatedResponse::from(result)))
        .map_err(|e| handle_integrated_service_error(e, &state.env, "get_users"))
}
