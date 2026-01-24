use axum::http::StatusCode;
use uuid::Uuid;
use crate::state::AppState;
use crate::methods::routes::USERS_BY_ID_PATH;

#[utoipa::path(
    delete,
    path = USERS_BY_ID_PATH,
    params(
        ("id" = String, Path, description = "User ID (UUID)")
    ),
    responses(
        (status = 204, description = "User deleted successfully"),
        (status = 400, description = "Invalid UUID"),
        (status = 404, description = "User not found"),
        (status = 500, description = "Internal server error"),
    )
)]
pub async fn delete_user(
    axum::extract::Path(id): axum::extract::Path<String>,
    axum::extract::State(state): axum::extract::State<AppState>,
) -> Result<StatusCode, (StatusCode, String)> {
    let user_service = state.user_service.clone();
    let env = state.env.clone();
    let prod_like = state.is_prod_like();

    let parsed_id = Uuid::parse_str(&id)
        .map_err(|_| (StatusCode::BAD_REQUEST, "invalid uuid".to_string()))?;

    user_service
        .delete_user(parsed_id)
        .await
        .map(|_| StatusCode::NO_CONTENT)
        .map_err(|e| {
            tracing::error!(env = %env, error = ?e, "delete_user failed");
            if prod_like {
                (StatusCode::INTERNAL_SERVER_ERROR, "internal server error".to_string())
            } else {
                (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
            }
        })
}
