use sqlx::{query, query_as, MySqlPool, Error};
use uuid::Uuid;
use crate::repository::models::UserRow;

pub struct UserRepository {
    pub pool: MySqlPool,
}

impl UserRepository {
    pub fn new(pool: MySqlPool) -> Self {
        Self { pool }
    }

    pub async fn create_user(&self, name: &str, email: &str) -> Result<UserRow, Error> {
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
        .await?;

        let user = query_as::<_, UserRow>(
            r#"
            SELECT id, name, email FROM users WHERE id = ?
            "#
        )
        .bind(user_id.to_string())
        .fetch_one(&self.pool)
        .await?;

        Ok(user)
    }

    pub async fn get_user(&self, user_id: Uuid) -> Result<Option<UserRow>, Error> {
        let user = query_as::<_, UserRow>(
            r#"
            SELECT id, name, email FROM users WHERE id = ?
            "#
        )
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(user)
    }

    pub async fn update_user(&self, user_id: Uuid, name: &str, email: &str) -> Result<UserRow, Error> {
        query(
            r#"
            UPDATE users
            SET name = ?, email = ?
            WHERE id = ?
            "#
        )
        .bind(name)
        .bind(email)
        .bind(user_id)
        .execute(&self.pool)
        .await?;

        let user = query_as::<_, UserRow>(
            r#"
            SELECT id, name, email FROM users WHERE id = ?
            "#
        )
        .bind(user_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(user)
    }

    pub async fn delete_user(&self, user_id: Uuid) -> Result<(), Error> {
        query(
            r#"
            DELETE FROM users WHERE id = ?
            "#
        )
        .bind(user_id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn get_users(&self) -> Result<Vec<UserRow>, Error> {
        let users = query_as::<_, UserRow>(
            r#"
            SELECT id, name, email FROM users
            "#
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(users)
    }
    pub async fn get_users_by_role(&self, role_id: Uuid) -> Result<Vec<UserRow>, Error> {
        let users = query_as::<_, UserRow>(
            r#"
            SELECT u.id, u.name, u.email
            FROM users u
            JOIN user_roles ur ON u.id = ur.user_id
            WHERE ur.role_id = ?
            "#
        )
        .bind(role_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(users)
    }
}