use axum::Json;
use uuid::Uuid;
use crate::methods::entities::RoleResponse;
use crate::state::AppState;
use crate::methods::routes::ROLES_BY_ID_PATH;

#[utoipa::path(
    get,
    path = ROLES_BY_ID_PATH,
    params(
        ("id" = String, Path, description = "Role ID (UUID)")
    ),
    responses(
        (status = 200, description = "Role found", body = RoleResponse),
        (status = 404, description = "Role not found"),
        (status = 400, description = "Invalid UUID"),
        (status = 500, description = "Internal server error"),
    )
)]
pub async fn get_role_by_id(
    axum::extract::Path(id): axum::extract::Path<String>,
    axum::extract::State(state): axum::extract::State<AppState>,
) -> Result<Json<RoleResponse>, (axum::http::StatusCode, String)> {
    let user_service = state.user_service.clone();
    let env = state.env.clone();
    let prod_like = state.is_prod_like();

    let parsed_id = Uuid::parse_str(&id)
        .map_err(|_| (axum::http::StatusCode::BAD_REQUEST, "invalid uuid".to_string()))?;

    match user_service.get_role(parsed_id).await {
        Ok(Some(role)) => Ok(Json(RoleResponse::from(role))),
        Ok(None) => Err((axum::http::StatusCode::NOT_FOUND, "role not found".to_string())),
        Err(e) => {
            tracing::error!(env = %env, error = ?e, "get_role failed");
            if prod_like {
                Err((axum::http::StatusCode::INTERNAL_SERVER_ERROR, "internal server error".to_string()))
            } else {
                Err((axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
            }
        }
    }
}
