use axum::Json;
use uuid::Uuid;
use user_lib::errors_service::UserServiceError;
use crate::methods::entities::{UpdateUserRequest, UserResponse};
use crate::state::AppState;
use crate::methods::routes::USERS_BY_ID_PATH;

#[utoipa::path(
    put,
    path = USERS_BY_ID_PATH,
    params(
        ("id" = String, Path, description = "User ID (UUID)")
    ),
    request_body = UpdateUserRequest,
    responses(
        (status = 200, description = "User updated successfully", body = UserResponse),
        (status = 400, description = "Invalid UUID or validation error"),
        (status = 404, description = "User not found"),
        (status = 409, description = "Email already exists"),
        (status = 500, description = "Internal server error"),
    )
)]
pub async fn update_user(
    axum::extract::Path(id): axum::extract::Path<String>,
    axum::extract::State(state): axum::extract::State<AppState>,
    Json(payload): Json<UpdateUserRequest>,
) -> Result<Json<UserResponse>, (axum::http::StatusCode, String)> {
    let user_service = state.user_service.clone();
    let env = state.env.clone();
    let prod_like = state.is_prod_like();

    let parsed_id = Uuid::parse_str(&id)
        .map_err(|_| (axum::http::StatusCode::BAD_REQUEST, "invalid uuid".to_string()))?;

    user_service
        .update_user(parsed_id, &payload.name, &payload.email)
        .await
        .map(|user| Json(UserResponse::from(user)))
        .map_err(|e| match e {
            UserServiceError::Validation(msg) => (axum::http::StatusCode::BAD_REQUEST, msg),
            UserServiceError::NotFound => {
                (axum::http::StatusCode::NOT_FOUND, "user not found".to_string())
            }
            UserServiceError::EmailAlreadyExists => {
                (axum::http::StatusCode::CONFLICT, e.to_string())
            }
            other => {
                tracing::error!(env = %env, error = ?other, "update_user failed");
                if prod_like {
                    (axum::http::StatusCode::INTERNAL_SERVER_ERROR, "internal server error".to_string())
                } else {
                    (axum::http::StatusCode::INTERNAL_SERVER_ERROR, other.to_string())
                }
            }
        })
}
