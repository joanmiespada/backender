use crate::error::{handle_integrated_service_error, ApiError};
use crate::methods::entities::{CreateRoleRequest, RoleResponse};
use crate::methods::routes::ROLES_PATH;
use crate::state::AppState;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use validator::Validate;

#[utoipa::path(
    post,
    path = ROLES_PATH,
    tag = "roles",
    request_body = CreateRoleRequest,
    responses(
        (status = 201, description = "Role created successfully", body = RoleResponse),
        (status = 400, description = "Validation error"),
        (status = 409, description = "Role name already exists"),
        (status = 500, description = "Internal server error"),
    )
)]
pub async fn create_role(
    axum::extract::State(state): axum::extract::State<AppState>,
    Json(payload): Json<CreateRoleRequest>,
) -> Result<impl IntoResponse, ApiError> {
    // Validate input
    payload.validate()?;

    state
        .user_service
        .create_role(&payload.name)
        .await
        .map(|role| (StatusCode::CREATED, Json(RoleResponse::from(role))))
        .map_err(|e| handle_integrated_service_error(e, &state.env, "create_role"))
}
