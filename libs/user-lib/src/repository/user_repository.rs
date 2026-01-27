use crate::entities::PaginationParams;
use crate::repository::errors::UserRepositoryError;
use crate::repository::models::UserRow;
use crate::repository::traits::UserRepositoryTrait;
use async_trait::async_trait;
use sqlx::{query, query_as, query_scalar, MySqlPool};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct UserRepository {
    pub pool: MySqlPool,
}

impl UserRepository {
    pub fn new(pool: MySqlPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl UserRepositoryTrait for UserRepository {
    async fn create_user(&self, keycloak_id: &str) -> Result<UserRow, UserRepositoryError> {
        let user_id = Uuid::new_v4();

        query(
            r#"
            INSERT INTO users (id, keycloak_id)
            VALUES (?, ?)
            "#,
        )
        .bind(user_id.to_string())
        .bind(keycloak_id)
        .execute(&self.pool)
        .await
        .map_err(UserRepositoryError::from)?;

        let user = query_as::<_, UserRow>(
            r#"
            SELECT id, keycloak_id FROM users WHERE id = ?
            "#,
        )
        .bind(user_id.to_string())
        .fetch_one(&self.pool)
        .await
        .map_err(UserRepositoryError::from)?;

        Ok(user)
    }

    async fn get_user(&self, user_id: Uuid) -> Result<Option<UserRow>, UserRepositoryError> {
        let user = query_as::<_, UserRow>(
            r#"
            SELECT id, keycloak_id FROM users WHERE id = ?
            "#,
        )
        .bind(user_id.to_string())
        .fetch_optional(&self.pool)
        .await
        .map_err(UserRepositoryError::from)?;

        Ok(user)
    }

    async fn get_user_by_keycloak_id(
        &self,
        keycloak_id: &str,
    ) -> Result<Option<UserRow>, UserRepositoryError> {
        let user = query_as::<_, UserRow>(
            r#"
            SELECT id, keycloak_id FROM users WHERE keycloak_id = ?
            "#,
        )
        .bind(keycloak_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(UserRepositoryError::from)?;

        Ok(user)
    }

    async fn delete_user(&self, user_id: Uuid) -> Result<(), UserRepositoryError> {
        query(
            r#"
            DELETE FROM users WHERE id = ?
            "#,
        )
        .bind(user_id.to_string())
        .execute(&self.pool)
        .await
        .map_err(UserRepositoryError::from)?;

        Ok(())
    }

    async fn get_users_paginated(
        &self,
        pagination: PaginationParams,
    ) -> Result<(Vec<UserRow>, u64), UserRepositoryError> {
        let total: i64 = query_scalar("SELECT COUNT(*) FROM users")
            .fetch_one(&self.pool)
            .await
            .map_err(UserRepositoryError::from)?;

        let users = query_as::<_, UserRow>(
            r#"
            SELECT id, keycloak_id FROM users
            ORDER BY id
            LIMIT ? OFFSET ?
            "#,
        )
        .bind(pagination.limit())
        .bind(pagination.offset())
        .fetch_all(&self.pool)
        .await
        .map_err(UserRepositoryError::from)?;

        Ok((users, total as u64))
    }

    async fn get_users_by_role_paginated(
        &self,
        role_id: Uuid,
        pagination: PaginationParams,
    ) -> Result<(Vec<UserRow>, u64), UserRepositoryError> {
        let total: i64 = query_scalar(
            r#"
            SELECT COUNT(*)
            FROM users u
            JOIN user_roles ur ON u.id = ur.user_id
            WHERE ur.role_id = ?
            "#,
        )
        .bind(role_id.to_string())
        .fetch_one(&self.pool)
        .await
        .map_err(UserRepositoryError::from)?;

        let users = query_as::<_, UserRow>(
            r#"
            SELECT u.id, u.keycloak_id
            FROM users u
            JOIN user_roles ur ON u.id = ur.user_id
            WHERE ur.role_id = ?
            ORDER BY u.id
            LIMIT ? OFFSET ?
            "#,
        )
        .bind(role_id.to_string())
        .bind(pagination.limit())
        .bind(pagination.offset())
        .fetch_all(&self.pool)
        .await
        .map_err(UserRepositoryError::from)?;

        Ok((users, total as u64))
    }
}
