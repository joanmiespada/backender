use axum::http::StatusCode;
use uuid::Uuid;
use crate::error::{ApiError, handle_service_error};
use crate::state::AppState;
use crate::methods::routes::USER_ROLES_PATH;

#[derive(serde::Deserialize)]
pub struct UserRolePath {
    pub user_id: String,
    pub role_id: String,
}

#[utoipa::path(
    post,
    path = USER_ROLES_PATH,
    tag = "users",
    params(
        ("user_id" = String, Path, description = "User ID (UUID)"),
        ("role_id" = String, Path, description = "Role ID (UUID)")
    ),
    responses(
        (status = 204, description = "Role assigned successfully"),
        (status = 400, description = "Invalid UUID"),
        (status = 404, description = "User or role not found"),
        (status = 409, description = "User already has this role"),
        (status = 500, description = "Internal server error"),
    )
)]
pub async fn assign_role(
    axum::extract::Path(path): axum::extract::Path<UserRolePath>,
    axum::extract::State(state): axum::extract::State<AppState>,
) -> Result<StatusCode, ApiError> {
    let user_id = Uuid::parse_str(&path.user_id).map_err(|_| ApiError::invalid_user_uuid())?;
    let role_id = Uuid::parse_str(&path.role_id).map_err(|_| ApiError::invalid_role_uuid())?;

    state.user_service
        .assign_role(user_id, role_id)
        .await
        .map(|_| StatusCode::NO_CONTENT)
        .map_err(|e| handle_service_error(e, &state.env, "assign_role"))
}
