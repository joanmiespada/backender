use sqlx::{query, MySqlPool};

use crate::repository::errors::{map_sqlx_error, UserRepositoryError};

#[derive(Debug, Clone)]
pub struct UserRoleRepository {
    pub pool: MySqlPool,
}

impl UserRoleRepository {
    pub fn new(pool: MySqlPool) -> Self {
        Self { pool }
    }

    pub async fn assign_role(&self, user_id: &str, role_id: &str) -> Result<(), UserRepositoryError> {
        query(
            r#"
            INSERT INTO user_roles (user_id, role_id)
            VALUES (?, ?)
            "#,
        )
        .bind(user_id)
        .bind(role_id)
        .execute(&self.pool)
        .await
        .map_err(map_sqlx_error)?;

        Ok(())
    }

    pub async fn unassign_role(&self, user_id: &str, role_id: &str) -> Result<(), UserRepositoryError> {
        query(
            r#"
            DELETE FROM user_roles
            WHERE user_id = ? AND role_id = ?
            "#,
        )
        .bind(user_id)
        .bind(role_id)
        .execute(&self.pool)
        .await
        .map_err(map_sqlx_error)?;

        Ok(())
    }
}