use axum::Json;
use uuid::Uuid;
use user_lib::errors_service::UserServiceError;
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
) -> Result<Json<RoleResponse>, (axum::http::StatusCode, String)> {
    let user_service = state.user_service.clone();
    let env = state.env.clone();
    let prod_like = state.is_prod_like();

    let parsed_id = Uuid::parse_str(&id)
        .map_err(|_| (axum::http::StatusCode::BAD_REQUEST, "invalid uuid".to_string()))?;

    user_service
        .update_role(parsed_id, &payload.name)
        .await
        .map(|role| Json(RoleResponse::from(role)))
        .map_err(|e| match e {
            UserServiceError::Validation(msg) => (axum::http::StatusCode::BAD_REQUEST, msg),
            UserServiceError::NotFound => {
                (axum::http::StatusCode::NOT_FOUND, "role not found".to_string())
            }
            UserServiceError::RoleNameAlreadyExists => {
                (axum::http::StatusCode::CONFLICT, e.to_string())
            }
            other => {
                tracing::error!(env = %env, error = ?other, "update_role failed");
                if prod_like {
                    (axum::http::StatusCode::INTERNAL_SERVER_ERROR, "internal server error".to_string())
                } else {
                    (axum::http::StatusCode::INTERNAL_SERVER_ERROR, other.to_string())
                }
            }
        })
}
