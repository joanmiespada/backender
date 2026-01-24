use axum::Json;
use uuid::Uuid;
use crate::error::{ApiError, handle_service_error};
use crate::methods::entities::RoleResponse;
use crate::state::AppState;
use crate::methods::routes::ROLES_BY_ID_PATH;

#[utoipa::path(
    get,
    path = ROLES_BY_ID_PATH,
    tag = "roles",
    params(
        ("id" = String, Path, description = "Role ID (UUID)")
    ),
    responses(
        (status = 200, description = "Role found", body = RoleResponse),
        (status = 400, description = "Invalid UUID"),
        (status = 404, description = "Role not found"),
        (status = 500, description = "Internal server error"),
    )
)]
pub async fn get_role_by_id(
    axum::extract::Path(id): axum::extract::Path<String>,
    axum::extract::State(state): axum::extract::State<AppState>,
) -> Result<Json<RoleResponse>, ApiError> {
    let parsed_id = Uuid::parse_str(&id).map_err(|_| ApiError::invalid_uuid())?;

    state.user_service
        .get_role(parsed_id)
        .await
        .map_err(|e| handle_service_error(e, &state.env, "get_role"))?
        .map(|role| Json(RoleResponse::from(role)))
        .ok_or_else(|| ApiError::role_not_found())
}
