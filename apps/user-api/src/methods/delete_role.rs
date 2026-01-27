use crate::error::{handle_integrated_service_error, ApiError};
use crate::methods::routes::ROLES_BY_ID_PATH;
use crate::state::AppState;
use axum::http::StatusCode;
use uuid::Uuid;

#[utoipa::path(
    delete,
    path = ROLES_BY_ID_PATH,
    tag = "roles",
    params(
        ("id" = String, Path, description = "Role ID (UUID)")
    ),
    responses(
        (status = 204, description = "Role deleted successfully"),
        (status = 400, description = "Invalid UUID"),
        (status = 404, description = "Role not found"),
        (status = 500, description = "Internal server error"),
    )
)]
pub async fn delete_role(
    axum::extract::Path(id): axum::extract::Path<String>,
    axum::extract::State(state): axum::extract::State<AppState>,
) -> Result<StatusCode, ApiError> {
    let parsed_id = Uuid::parse_str(&id).map_err(|_| ApiError::invalid_uuid())?;

    state
        .user_service
        .delete_role(parsed_id)
        .await
        .map(|_| StatusCode::NO_CONTENT)
        .map_err(|e| handle_integrated_service_error(e, &state.env, "delete_role"))
}
