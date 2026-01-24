use axum::Json;
use uuid::Uuid;
use crate::methods::entities::UserResponse;
use crate::state::AppState;
use crate::methods::routes::USERS_BY_ID_PATH;

#[utoipa::path(
    get,
    path = USERS_BY_ID_PATH,
    tag = "users",
    params(
        ("id" = String, Path, description = "User ID (UUID)")
    ),
    responses(
        (status = 200, description = "User found", body = UserResponse),
        (status = 400, description = "Invalid UUID"),
        (status = 404, description = "User not found"),
        (status = 500, description = "Internal server error"),
    )
)]
pub async fn get_user_by_id(
    axum::extract::Path(id): axum::extract::Path<String>,
    axum::extract::State(state): axum::extract::State<AppState>,
) -> Result<Json<UserResponse>, (axum::http::StatusCode, String)> {
    let user_service = state.user_service.clone();
    let env = state.env.clone();
    let prod_like = state.is_prod_like();
    match Uuid::parse_str(&id) {
        Ok(parsed_id) => {
            match user_service.get_user(parsed_id).await {
                Ok(Some(user)) => Ok(Json(UserResponse::from(user))),
                Ok(None) => Err((axum::http::StatusCode::NOT_FOUND, "user not found".to_string())),
                Err(e) => {
                    tracing::error!(env = %env, error = ?e, "get_user failed");
                    if prod_like {
                        Err((axum::http::StatusCode::INTERNAL_SERVER_ERROR, "internal server error".to_string()))
                    } else {
                        Err((axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
                    }
                }
            }
        }
        Err(_) => Err((axum::http::StatusCode::BAD_REQUEST, "invalid uuid".to_string())),
    }
}