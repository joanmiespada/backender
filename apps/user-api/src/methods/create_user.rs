use axum::Json;
use user_lib::errors_service::UserServiceError;
use crate::methods::entities::{CreateUserRequest, UserResponse};
use crate::state::AppState;
use crate::methods::routes::USERS_PATH;

#[utoipa::path(
    post,
    path = USERS_PATH,
    request_body = CreateUserRequest,
    responses(
        (status = 200, description = "User created successfully", body = UserResponse),
        (status = 400, description = "Validation error"),
        (status = 409, description = "Email already exists"),
        (status = 500, description = "Internal server error"),
    )
)]

pub async fn create_user(
    axum::extract::State(state): axum::extract::State<AppState>,
    Json(payload): Json<CreateUserRequest>,
) -> Result<Json<UserResponse>, (axum::http::StatusCode, String)> {
    let user_service = state.user_service.clone();
    let env = state.env.clone();
    let prod_like = state.is_prod_like();
    user_service
        .create_user(&payload.name, &payload.email)
        .await
        .map(|user| Json(UserResponse::from(user)))
        .map_err(|e| {
            match e {
                UserServiceError::Validation(msg) => {
                    (axum::http::StatusCode::BAD_REQUEST, msg)
                }
                UserServiceError::EmailAlreadyExists => {
                    (axum::http::StatusCode::CONFLICT, UserServiceError::EmailAlreadyExists.to_string())
                }
                UserServiceError::RoleNameAlreadyExists => (axum::http::StatusCode::CONFLICT, e.to_string()),
                UserServiceError::UserAlreadyHasRole => (axum::http::StatusCode::CONFLICT, e.to_string()),
                other => {
                    tracing::error!(env = %env, error = ?other, "create_user failed");
                    if prod_like {
                        (axum::http::StatusCode::INTERNAL_SERVER_ERROR, "internal server error".to_string())
                    } else {
                        (axum::http::StatusCode::INTERNAL_SERVER_ERROR, other.to_string())
                    }
                }
            }
        })
}

