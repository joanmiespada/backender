use sqlx::{query, query_as, MySqlPool, Error};
use uuid::Uuid;
use crate::repository::models::RoleRow;

#[derive(Debug, Clone)]
pub struct RoleRepository {
    pub pool: MySqlPool,
}

impl RoleRepository {
    pub fn new(pool: MySqlPool) -> Self {
        Self { pool }
    }

    pub async fn create_role(&self, name: &str) -> Result<RoleRow, Error> {
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
        .await?;

        let role = query_as::<_, RoleRow>(
            r#"SELECT id, name FROM roles WHERE id = ? "#
        )
        .bind(id.to_string())
        .fetch_one(&self.pool)
        .await?;
        Ok(role)
    }

    pub async fn get_role(&self, role_id: Uuid) -> Result<Option<RoleRow>, Error> {
        let role = query_as::<_, RoleRow>(
            r#"
            SELECT id, name FROM roles WHERE id = ?
            "#
        )
        .bind(role_id)
        .fetch_optional(&self.pool)
        .await?;
        Ok(role)
    }

    pub async fn update_role(&self, role_id: Uuid, name: &str) -> Result<RoleRow, Error> {
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
        .await?;

        let role = query_as::<_, RoleRow>(
            r#"SELECT id, name FROM roles WHERE id = ? "#
        )
        .bind(role_id)
        .fetch_one(&self.pool)
        .await?;
        Ok(role)
    }

    pub async fn delete_role(&self, role_id: Uuid) -> Result<(), Error> {
        sqlx::query(
            r#"
            DELETE FROM roles WHERE id = ?
            "#
        )
        .bind(role_id)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn get_roles_for_user(&self, user_id: Uuid) -> Result<Vec<RoleRow>, Error> {
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
        .await?;
        Ok(roles)
    }

    pub async fn get_roles(&self) -> Result<Vec<RoleRow>, Error> {
        let roles = query_as::<_, RoleRow>(
            r#"
            SELECT id, name FROM roles
            "#
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(roles)
    }
}