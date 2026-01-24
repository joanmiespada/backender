use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;
use user_lib::errors_service::UserServiceError;

#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub error: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

#[derive(Debug)]
pub enum ApiError {
    BadRequest(String),
    NotFound(String),
    Conflict(String),
    Internal(String),
}

impl ApiError {
    pub fn invalid_uuid() -> Self {
        ApiError::BadRequest("invalid uuid".to_string())
    }

    pub fn invalid_user_uuid() -> Self {
        ApiError::BadRequest("invalid user uuid".to_string())
    }

    pub fn invalid_role_uuid() -> Self {
        ApiError::BadRequest("invalid role uuid".to_string())
    }

    pub fn user_not_found() -> Self {
        ApiError::NotFound("user not found".to_string())
    }

    pub fn role_not_found() -> Self {
        ApiError::NotFound("role not found".to_string())
    }

}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, error, message) = match self {
            ApiError::BadRequest(msg) => (StatusCode::BAD_REQUEST, "bad_request", Some(msg)),
            ApiError::NotFound(msg) => (StatusCode::NOT_FOUND, "not_found", Some(msg)),
            ApiError::Conflict(msg) => (StatusCode::CONFLICT, "conflict", Some(msg)),
            ApiError::Internal(msg) => (StatusCode::INTERNAL_SERVER_ERROR, "internal_error", Some(msg)),
        };

        let body = ErrorResponse {
            error: error.to_string(),
            message,
        };

        (status, Json(body)).into_response()
    }
}

impl From<UserServiceError> for ApiError {
    fn from(err: UserServiceError) -> Self {
        match err {
            UserServiceError::Validation(msg) => ApiError::BadRequest(msg),
            UserServiceError::NotFound => ApiError::NotFound("resource not found".to_string()),
            UserServiceError::EmailAlreadyExists => ApiError::Conflict("email already exists".to_string()),
            UserServiceError::RoleNameAlreadyExists => ApiError::Conflict("role name already exists".to_string()),
            UserServiceError::UserAlreadyHasRole => ApiError::Conflict("user already has this role".to_string()),
            UserServiceError::InvalidUuid(msg) => ApiError::BadRequest(format!("invalid uuid: {}", msg)),
            UserServiceError::Internal(err) => ApiError::Internal(err.to_string()),
            _ => ApiError::Internal("unexpected error".to_string()),
        }
    }
}

/// Check if environment is production-like (prod, prod01, prod02, etc.)
pub fn is_prod_like(env: &str) -> bool {
    env.to_lowercase().starts_with("prod")
}

/// Converts a service error to an ApiError, logging internal errors.
/// In production, internal error details are hidden.
pub fn handle_service_error(err: UserServiceError, env: &str, operation: &str) -> ApiError {
    match &err {
        UserServiceError::Internal(_) | UserServiceError::InvalidUuid(_) => {
            tracing::error!(env = %env, error = ?err, operation = %operation, "service error");
            if is_prod_like(env) {
                ApiError::Internal("internal server error".to_string())
            } else {
                ApiError::from(err)
            }
        }
        _ => ApiError::from(err),
    }
}
