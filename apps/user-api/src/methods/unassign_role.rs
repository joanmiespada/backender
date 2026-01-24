use axum::http::StatusCode;
use uuid::Uuid;
use crate::state::AppState;
use crate::methods::routes::USER_ROLES_PATH;
use crate::methods::assign_role::UserRolePath;

#[utoipa::path(
    delete,
    path = USER_ROLES_PATH,
    tag = "users",
    params(
        ("user_id" = String, Path, description = "User ID (UUID)"),
        ("role_id" = String, Path, description = "Role ID (UUID)")
    ),
    responses(
        (status = 204, description = "Role unassigned successfully"),
        (status = 400, description = "Invalid UUID"),
        (status = 404, description = "User or role not found"),
        (status = 500, description = "Internal server error"),
    )
)]
pub async fn unassign_role(
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
        .unassign_role(user_id, role_id)
        .await
        .map(|_| StatusCode::NO_CONTENT)
        .map_err(|e| {
            tracing::error!(env = %env, error = ?e, "unassign_role failed");
            if prod_like {
                (StatusCode::INTERNAL_SERVER_ERROR, "internal server error".to_string())
            } else {
                (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
            }
        })
}
