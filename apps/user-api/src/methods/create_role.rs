use axum::Json;
use user_lib::errors_service::UserServiceError;
use crate::methods::entities::{CreateRoleRequest, RoleResponse};
use crate::state::AppState;
use crate::methods::routes::ROLES_PATH;

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
) -> Result<Json<RoleResponse>, (axum::http::StatusCode, String)> {
    let user_service = state.user_service.clone();
    let env = state.env.clone();
    let prod_like = state.is_prod_like();

    user_service
        .create_role(&payload.name)
        .await
        .map(|role| Json(RoleResponse::from(role)))
        .map_err(|e| match e {
            UserServiceError::Validation(msg) => (axum::http::StatusCode::BAD_REQUEST, msg),
            UserServiceError::RoleNameAlreadyExists => {
                (axum::http::StatusCode::CONFLICT, e.to_string())
            }
            other => {
                tracing::error!(env = %env, error = ?other, "create_role failed");
                if prod_like {
                    (axum::http::StatusCode::INTERNAL_SERVER_ERROR, "internal server error".to_string())
                } else {
                    (axum::http::StatusCode::INTERNAL_SERVER_ERROR, other.to_string())
                }
            }
        })
}
