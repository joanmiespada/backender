use crate::error::{handle_integrated_service_error, ApiError};
use crate::methods::entities::UserResponse;
use crate::methods::routes::USERS_BY_ID_PATH;
use crate::state::AppState;
use axum::Json;
use uuid::Uuid;

#[utoipa::path(
    get,
    path = USERS_BY_ID_PATH,
    tag = "users",
    params(
        ("id" = String, Path, description = "User ID (UUID)")
    ),
    responses(
        (status = 200, description = "User found", body = UserResponse),
        (status = 400, description = "Invalid UUID"),
        (status = 404, description = "User not found"),
        (status = 500, description = "Internal server error"),
    )
)]
pub async fn get_user_by_id(
    axum::extract::Path(id): axum::extract::Path<String>,
    axum::extract::State(state): axum::extract::State<AppState>,
) -> Result<Json<UserResponse>, ApiError> {
    let parsed_id = Uuid::parse_str(&id).map_err(|_| ApiError::invalid_uuid())?;

    state
        .user_service
        .get_user(parsed_id)
        .await
        .map_err(|e| handle_integrated_service_error(e, &state.env, "get_user"))?
        .map(|user| Json(UserResponse::from(user)))
        .ok_or_else(ApiError::user_not_found)
}
