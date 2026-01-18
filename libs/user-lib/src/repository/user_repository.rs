use sqlx::{query, query_as, MySqlPool};
use uuid::Uuid;
use crate::repository::models::UserRow;
use crate::repository::errors::UserRepositoryError;

#[derive(Debug, Clone)]
pub struct UserRepository {
    pub pool: MySqlPool,
}

impl UserRepository {
    pub fn new(pool: MySqlPool) -> Self {
        Self { pool }
    }

    pub async fn create_user(&self, name: &str, email: &str) -> Result<UserRow, UserRepositoryError> {
        let user_id = Uuid::new_v4();

        query(
            r#"
            INSERT INTO users (id, name, email)
            VALUES (?, ?, ?)
            "#
        )
        .bind(user_id.to_string())
        .bind(name)
        .bind(email)
        .execute(&self.pool)
        .await
        .map_err(UserRepositoryError::from)?;

        let user = query_as::<_, UserRow>(
            r#"
            SELECT id, name, email FROM users WHERE id = ?
            "#
        )
        .bind(user_id.to_string())
        .fetch_one(&self.pool)
        .await
        .map_err(UserRepositoryError::from)?;

        Ok(user)
    }

    pub async fn get_user(&self, user_id: Uuid) -> Result<Option<UserRow>, UserRepositoryError> {
        let user = query_as::<_, UserRow>(
            r#"
            SELECT id, name, email FROM users WHERE id = ?
            "#
        )
        .bind(user_id.to_string())
        .fetch_optional(&self.pool)
        .await
        .map_err(UserRepositoryError::from)?;

        Ok(user)
    }

    pub async fn update_user(&self, user_id: Uuid, name: &str, email: &str) -> Result<UserRow, UserRepositoryError> {
        query(
            r#"
            UPDATE users
            SET name = ?, email = ?
            WHERE id = ?
            "#
        )
        .bind(name)
        .bind(email)
        .bind(user_id.to_string())
        .execute(&self.pool)
        .await
        .map_err(UserRepositoryError::from)?;

        let user = query_as::<_, UserRow>(
            r#"
            SELECT id, name, email FROM users WHERE id = ?
            "#
        )
        .bind(user_id.to_string())
        .fetch_one(&self.pool)
        .await
        .map_err(UserRepositoryError::from)?;

        Ok(user)
    }

    pub async fn delete_user(&self, user_id: Uuid) -> Result<(), UserRepositoryError> {
        query(
            r#"
            DELETE FROM users WHERE id = ?
            "#
        )
        .bind(user_id.to_string())
        .execute(&self.pool)
        .await
        .map_err(UserRepositoryError::from)?;

        Ok(())
    }

    pub async fn get_users(&self) -> Result<Vec<UserRow>, UserRepositoryError> {
        let users = query_as::<_, UserRow>(
            r#"
            SELECT id, name, email FROM users
            "#
        )
        .fetch_all(&self.pool)
        .await
        .map_err(UserRepositoryError::from)?;

        Ok(users)
    }
    pub async fn get_users_by_role(&self, role_id: Uuid) -> Result<Vec<UserRow>, UserRepositoryError> {
        let users = query_as::<_, UserRow>(
            r#"
            SELECT u.id, u.name, u.email
            FROM users u
            JOIN user_roles ur ON u.id = ur.user_id
            WHERE ur.role_id = ?
            "#
        )
        .bind(role_id.to_string())
        .fetch_all(&self.pool)
        .await
        .map_err(UserRepositoryError::from)?;

        Ok(users)
    }
}