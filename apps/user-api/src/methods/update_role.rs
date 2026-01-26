use axum::Json;
use uuid::Uuid;
use crate::error::{ApiError, handle_integrated_service_error};
use crate::methods::entities::{UpdateRoleRequest, RoleResponse};
use crate::state::AppState;
use crate::methods::routes::ROLES_BY_ID_PATH;

#[utoipa::path(
    put,
    path = ROLES_BY_ID_PATH,
    tag = "roles",
    params(
        ("id" = String, Path, description = "Role ID (UUID)")
    ),
    request_body = UpdateRoleRequest,
    responses(
        (status = 200, description = "Role updated successfully", body = RoleResponse),
        (status = 400, description = "Invalid UUID or validation error"),
        (status = 404, description = "Role not found"),
        (status = 409, description = "Role name already exists"),
        (status = 500, description = "Internal server error"),
    )
)]
pub async fn update_role(
    axum::extract::Path(id): axum::extract::Path<String>,
    axum::extract::State(state): axum::extract::State<AppState>,
    Json(payload): Json<UpdateRoleRequest>,
) -> Result<Json<RoleResponse>, ApiError> {
    let parsed_id = Uuid::parse_str(&id).map_err(|_| ApiError::invalid_uuid())?;

    state.user_service
        .update_role(parsed_id, &payload.name)
        .await
        .map(|role| Json(RoleResponse::from(role)))
        .map_err(|e| handle_integrated_service_error(e, &state.env, "update_role"))
}
