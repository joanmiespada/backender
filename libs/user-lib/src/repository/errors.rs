#[derive(Debug)]
pub enum UserRepositoryError {
    EmailAlreadyExists,
    RoleNameAlreadyExists,
    UserAlreadyHasRole,
    NotFound,
    Sqlx(sqlx::Error),
}

impl std::fmt::Display for UserRepositoryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UserRepositoryError::EmailAlreadyExists => write!(f, "email already exists"),
            UserRepositoryError::RoleNameAlreadyExists => write!(f, "role name already exists"),
            UserRepositoryError::UserAlreadyHasRole => write!(f, "user already has role"),
            UserRepositoryError::NotFound => write!(f, "not found"),
            UserRepositoryError::Sqlx(e) => write!(f, "{e}"),
        }
    }
}

impl std::error::Error for UserRepositoryError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            UserRepositoryError::EmailAlreadyExists => None,
            UserRepositoryError::RoleNameAlreadyExists => None,
            UserRepositoryError::UserAlreadyHasRole => None,
            UserRepositoryError::NotFound => None,
            UserRepositoryError::Sqlx(e) => Some(e),
        }
    }
}

impl From<sqlx::Error> for UserRepositoryError {
    fn from(value: sqlx::Error) -> Self {
        map_sqlx_error(value)
    }
}

fn extract_mysql_key_name(msg_lower: &str) -> Option<String> {
    // msg_lower is already lowercased
    let marker = "for key '";
    let start = msg_lower.find(marker)? + marker.len();
    let rest = &msg_lower[start..];
    let end = rest.find('\'')?;
    Some(rest[..end].to_string())
}

pub fn map_sqlx_error(err: sqlx::Error) -> UserRepositoryError {
    const USER_EMAIL_UNIQUE: &str = "user_email_unique";
    const ROLE_NAME_UNIQUE: &str = "role_name_unique";
    const USER_ROLES_PK: &str = "user_roles_pk";

    if let sqlx::Error::Database(db_err) = &err {
        // MySQL duplicate key violations typically surface as:
        // - SQLSTATE code: 23000 (integrity constraint violation)
        // - message: "Duplicate entry '...' for key '...'"
        //tracing::info!("Database error: {:?}", db_err);

        let msg = db_err.message().to_lowercase();
        let is_duplicate_key = db_err.code().as_deref() == Some("23000")
            && msg.contains("duplicate entry")
            && msg.contains("for key");

        if is_duplicate_key {
            // Example message:
            // "Duplicate entry 'user12@user.com' for key 'users.user_email_unique'"
            // We extract the key name between "for key '" and the next "'".
            let key = extract_mysql_key_name(&msg).unwrap_or_default();

            //tracing::info!("Duplicate key: {}", key);
            //tracing::info!("Error Message: {}", msg);

            // Prefer deterministic matching on named constraints.
            // MySQL may prefix with table name (e.g., "users.user_email_unique"), so we use `ends_with`.
            if key.ends_with(USER_EMAIL_UNIQUE) || msg.contains(USER_EMAIL_UNIQUE) {
                return UserRepositoryError::EmailAlreadyExists;
            }

            if key.ends_with(ROLE_NAME_UNIQUE) || msg.contains(ROLE_NAME_UNIQUE) {
                return UserRepositoryError::RoleNameAlreadyExists;
            }

            if key.ends_with(USER_ROLES_PK) || msg.contains(USER_ROLES_PK) {
                return UserRepositoryError::UserAlreadyHasRole;
            }
        }
    }

    UserRepositoryError::Sqlx(err)
}
