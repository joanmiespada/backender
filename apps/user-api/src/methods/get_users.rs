use axum::{extract::Query, Json};
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
) -> Result<Json<PaginatedResponse<UserResponse>>, (axum::http::StatusCode, String)> {
    let user_service = state.user_service.clone();
    let env = state.env.clone();
    let prod_like = state.is_prod_like();

    user_service
        .get_users(pagination.into())
        .await
        .map(|result| Json(PaginatedResponse::from(result)))
        .map_err(|e| {
            tracing::error!(env = %env, error = ?e, "get_users failed");
            if prod_like {
                (axum::http::StatusCode::INTERNAL_SERVER_ERROR, "internal server error".to_string())
            } else {
                (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
            }
        })
}
