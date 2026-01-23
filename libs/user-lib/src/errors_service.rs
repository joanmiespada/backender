use crate::repository::errors::UserRepositoryError;

#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum UserServiceError {
    #[error("email already exists")]
    EmailAlreadyExists,

    #[error("role name already exists")]
    RoleNameAlreadyExists,

    #[error("user already has role")]
    UserAlreadyHasRole,

    #[error("resource not found")]
    NotFound,

    #[error("invalid UUID in database: {0}")]
    InvalidUuid(String),

    #[error("validation error: {0}")]
    Validation(String),

    #[error(transparent)]
    Internal(#[from] anyhow::Error),
}

impl From<UserRepositoryError> for UserServiceError {
    fn from(err: UserRepositoryError) -> Self {
        match err {
            UserRepositoryError::EmailAlreadyExists => UserServiceError::EmailAlreadyExists,
            UserRepositoryError::RoleNameAlreadyExists => UserServiceError::RoleNameAlreadyExists,
            UserRepositoryError::UserAlreadyHasRole => UserServiceError::UserAlreadyHasRole,
            UserRepositoryError::NotFound => UserServiceError::NotFound,
            UserRepositoryError::Sqlx(e) => UserServiceError::Internal(e.into()),
        }
    }
}
