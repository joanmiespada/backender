use axum::Json;
use uuid::Uuid;
use crate::error::{ApiError, handle_service_error};
use crate::methods::entities::{UpdateUserRequest, UserResponse};
use crate::state::AppState;
use crate::methods::routes::USERS_BY_ID_PATH;

#[utoipa::path(
    put,
    path = USERS_BY_ID_PATH,
    tag = "users",
    params(
        ("id" = String, Path, description = "User ID (UUID)")
    ),
    request_body = UpdateUserRequest,
    responses(
        (status = 200, description = "User updated successfully", body = UserResponse),
        (status = 400, description = "Invalid UUID or validation error"),
        (status = 404, description = "User not found"),
        (status = 409, description = "Email already exists"),
        (status = 500, description = "Internal server error"),
    )
)]
pub async fn update_user(
    axum::extract::Path(id): axum::extract::Path<String>,
    axum::extract::State(state): axum::extract::State<AppState>,
    Json(payload): Json<UpdateUserRequest>,
) -> Result<Json<UserResponse>, ApiError> {
    let parsed_id = Uuid::parse_str(&id).map_err(|_| ApiError::invalid_uuid())?;

    state.user_service
        .update_user(parsed_id, &payload.name, &payload.email)
        .await
        .map(|user| Json(UserResponse::from(user)))
        .map_err(|e| handle_service_error(e, &state.env, "update_user"))
}
