use axum::http::StatusCode;
use uuid::Uuid;
use user_lib::errors_service::UserServiceError;
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
) -> Result<StatusCode, (StatusCode, String)> {
    let user_service = state.user_service.clone();
    let env = state.env.clone();
    let prod_like = state.is_prod_like();

    let user_id = Uuid::parse_str(&path.user_id)
        .map_err(|_| (StatusCode::BAD_REQUEST, "invalid user uuid".to_string()))?;
    let role_id = Uuid::parse_str(&path.role_id)
        .map_err(|_| (StatusCode::BAD_REQUEST, "invalid role uuid".to_string()))?;

    user_service
        .assign_role(user_id, role_id)
        .await
        .map(|_| StatusCode::NO_CONTENT)
        .map_err(|e| match e {
            UserServiceError::NotFound => {
                (StatusCode::NOT_FOUND, "user or role not found".to_string())
            }
            UserServiceError::UserAlreadyHasRole => {
                (StatusCode::CONFLICT, e.to_string())
            }
            other => {
                tracing::error!(env = %env, error = ?other, "assign_role failed");
                if prod_like {
                    (StatusCode::INTERNAL_SERVER_ERROR, "internal server error".to_string())
                } else {
                    (StatusCode::INTERNAL_SERVER_ERROR, other.to_string())
                }
            }
        })
}
