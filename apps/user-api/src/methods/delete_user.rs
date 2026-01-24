use axum::http::StatusCode;
use uuid::Uuid;
use crate::error::{ApiError, handle_service_error};
use crate::state::AppState;
use crate::methods::routes::USERS_BY_ID_PATH;

#[utoipa::path(
    delete,
    path = USERS_BY_ID_PATH,
    tag = "users",
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
) -> Result<StatusCode, ApiError> {
    let parsed_id = Uuid::parse_str(&id).map_err(|_| ApiError::invalid_uuid())?;

    state.user_service
        .delete_user(parsed_id)
        .await
        .map(|_| StatusCode::NO_CONTENT)
        .map_err(|e| handle_service_error(e, &state.env, "delete_user"))
}
