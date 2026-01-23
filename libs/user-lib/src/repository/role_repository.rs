use async_trait::async_trait;
use sqlx::{query, query_as, MySqlPool};
use uuid::Uuid;

use crate::repository::errors::{map_sqlx_error, UserRepositoryError};
use crate::repository::models::RoleRow;
use crate::repository::traits::RoleRepositoryTrait;

#[derive(Debug, Clone)]
pub struct RoleRepository {
    pub pool: MySqlPool,
}

impl RoleRepository {
    pub fn new(pool: MySqlPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl RoleRepositoryTrait for RoleRepository {
    async fn create_role(&self, name: &str) -> Result<RoleRow, UserRepositoryError> {
        let id = Uuid::new_v4();
        query(
            r#"
            INSERT INTO roles (id, name)
            VALUES (?, ?)
            "#
        )
        .bind(id.to_string())
        .bind(name)
        .execute(&self.pool)
        .await
        .map_err(map_sqlx_error)?;

        let role = query_as::<_, RoleRow>(
            r#"SELECT id, name FROM roles WHERE id = ? "#
        )
        .bind(id.to_string())
        .fetch_one(&self.pool)
        .await
        .map_err(map_sqlx_error)?;
        Ok(role)
    }

    async fn get_role(&self, role_id: Uuid) -> Result<Option<RoleRow>, UserRepositoryError> {
        let role = query_as::<_, RoleRow>(
            r#"
            SELECT id, name FROM roles WHERE id = ?
            "#
        )
        .bind(role_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(map_sqlx_error)?;
        Ok(role)
    }

    async fn update_role(&self, role_id: Uuid, name: &str) -> Result<RoleRow, UserRepositoryError> {
        query(
            r#"
            UPDATE roles
            SET name = ?
            WHERE id = ?
            "#
        )
        .bind(name)
        .bind(role_id)
        .execute(&self.pool)
        .await
        .map_err(map_sqlx_error)?;

        let role = query_as::<_, RoleRow>(
            r#"SELECT id, name FROM roles WHERE id = ? "#
        )
        .bind(role_id)
        .fetch_one(&self.pool)
        .await
        .map_err(map_sqlx_error)?;
        Ok(role)
    }

    async fn delete_role(&self, role_id: Uuid) -> Result<(), UserRepositoryError> {
        sqlx::query(
            r#"
            DELETE FROM roles WHERE id = ?
            "#
        )
        .bind(role_id)
        .execute(&self.pool)
        .await
        .map_err(map_sqlx_error)?;
        Ok(())
    }

    async fn get_roles_for_user(&self, user_id: Uuid) -> Result<Vec<RoleRow>, UserRepositoryError> {
        let roles = query_as::<_, RoleRow>(
            r#"
            SELECT r.id, r.name
            FROM roles r
            INNER JOIN user_roles ur ON ur.role_id = r.id
            WHERE ur.user_id = ?
            "#
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await
        .map_err(map_sqlx_error)?;
        Ok(roles)
    }

    async fn get_roles(&self) -> Result<Vec<RoleRow>, UserRepositoryError> {
        let roles = query_as::<_, RoleRow>(
            r#"
            SELECT id, name FROM roles
            "#
        )
        .fetch_all(&self.pool)
        .await
        .map_err(map_sqlx_error)?;
        Ok(roles)
    }
}