use crate::error::{handle_integrated_service_error, ApiError};
use crate::methods::entities::{CreateUserRequest, UserResponse};
use crate::methods::routes::USERS_PATH;
use crate::services::integrated_user_service::CreateUserRequest as ServiceCreateUserRequest;
use crate::state::AppState;
use axum::Json;

#[utoipa::path(
    post,
    path = USERS_PATH,
    tag = "users",
    request_body = CreateUserRequest,
    responses(
        (status = 201, description = "User created successfully", body = UserResponse),
        (status = 400, description = "Validation error"),
        (status = 409, description = "Email already exists"),
        (status = 500, description = "Internal server error"),
    )
)]
pub async fn create_user(
    axum::extract::State(state): axum::extract::State<AppState>,
    Json(payload): Json<CreateUserRequest>,
) -> Result<Json<UserResponse>, ApiError> {
    let request = ServiceCreateUserRequest {
        email: payload.email,
        first_name: payload.first_name,
        last_name: payload.last_name,
        password: payload.password,
    };

    state
        .user_service
        .create_user(request)
        .await
        .map(|user| Json(UserResponse::from(user)))
        .map_err(|e| handle_integrated_service_error(e, &state.env, "create_user"))
}
