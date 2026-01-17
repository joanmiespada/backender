
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

pub fn map_sqlx_error(err: sqlx::Error) -> UserRepositoryError {
    if let sqlx::Error::Database(db_err) = &err {
        let code_is_duplicate = db_err.code().as_deref() == Some("1062");
        if code_is_duplicate {
            // MySQL typically reports the violated index/constraint name as the "key".
            // For unnamed UNIQUE constraints, the key often matches the column name.
            let msg = db_err.message().to_lowercase();
            let constraint = db_err
                .constraint()
                .map(|c| c.to_lowercase())
                .unwrap_or_default();

            // Helper: does the message mention a given key name?
            let mentions_key = |k: &str| msg.contains("for key") && msg.contains(k);

            // users.email has UNIQUE, so key is commonly `email`
            if constraint == "email" || mentions_key("'email'") || mentions_key("`email`") {
                return UserRepositoryError::EmailAlreadyExists;
            }

            // roles.name has UNIQUE, so key is commonly `name`
            if constraint == "name" || mentions_key("'name'") || mentions_key("`name`") {
                return UserRepositoryError::RoleNameAlreadyExists;
            }

            // user_roles has PRIMARY KEY(user_id, role_id). Duplicate assignment => key PRIMARY
            if constraint == "primary" || mentions_key("'primary'") || mentions_key("`primary`") {
                return UserRepositoryError::UserAlreadyHasRole;
            }
        }
    }

    UserRepositoryError::Sqlx(err)
}
