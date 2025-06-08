

use sqlx::{query, Error, MySqlPool};

#[derive(Debug, Clone)]
pub struct UserRoleRepository {
    pub pool: MySqlPool,
}

impl UserRoleRepository {
    pub fn new(pool: MySqlPool) -> Self {
        Self { pool }
    }

    pub async fn assign_role(&self, user_id: &str, role_id: &str) -> Result<(), Error> {
        query(
            r#"
            INSERT INTO user_roles (user_id, role_id)
            VALUES (?, ?)
            "#,
        )
        .bind(user_id)
        .bind(role_id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn unassign_role(&self, user_id: &str, role_id: &str) -> Result<(), Error> {
        query(
            r#"
            DELETE FROM user_roles
            WHERE user_id = ? AND role_id = ?
            "#,
        )
        .bind(user_id)
        .bind(role_id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }
}