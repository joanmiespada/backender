

use uuid::Uuid;
use crate::entities::{User, Role};
use crate::repository::{RoleRepository, UserRepository, UserRoleRepository};
use sqlx::Error;

pub struct UserService {
    pub user_repo: UserRepository,
    pub role_repo: RoleRepository,
    pub user_role_repo: UserRoleRepository,
}

impl UserService {
    pub fn new(user_repo: UserRepository, role_repo: RoleRepository, user_role_repo: UserRoleRepository) -> Self {
        Self { user_repo, role_repo, user_role_repo }
    }

    pub async fn create_user(&self, name: &str, email: &str) -> Result<User, Error> {
        let row = self.user_repo.create_user(name, email).await?;
        Ok(User {
            id: Uuid::parse_str(&row.id).expect("Invalid UUID format"),
            name: row.name,
            email: row.email,
            roles: vec![],
        })
    }

    pub async fn get_user(&self, user_id: Uuid) -> Result<Option<User>, Error> {
        let user_row = self.user_repo.get_user(user_id).await?;
        if let Some(row) = user_row {
            let roles = self.role_repo.get_roles_for_user(  
                        Uuid::parse_str(&row.id).expect("invalid UUID format") 
                    ).await?
                .into_iter()
                .map(|r| Role { id: Uuid::parse_str(&r.id).expect("Invalid UUID format"), name: r.name })
                .collect();
            Ok(Some(User {
                id: Uuid::parse_str(&row.id).expect("Invalid UUID format"),
                name: row.name,
                email: row.email,
                roles,
            }))
        } else {
            Ok(None)
        }
    }

    pub async fn update_user(&self, user_id: Uuid, name: &str, email: &str) -> Result<User, Error> {
        let row = self.user_repo.update_user(user_id, name, email).await?;
        let roles = self.role_repo.get_roles_for_user(
                        Uuid::parse_str(&row.id).expect("invalid UUID format") 
                    ).await?
            .into_iter()
            .map(|r| Role { id: Uuid::parse_str(&r.id).expect("Invalid UUID format"), name: r.name })
            .collect();
        Ok(User {
            id: Uuid::parse_str(&row.id).expect("Invalid UUID format"),
            name: row.name,
            email: row.email,
            roles,
        })
    }

    pub async fn delete_user(&self, user_id: Uuid) -> Result<(), Error> {
        self.user_repo.delete_user(user_id).await
    }

    pub async fn assign_role(&self, user_id: Uuid, role_id: Uuid) -> Result<(), Error> {
        self.user_role_repo.assign_role(&user_id.to_string(), &role_id.to_string()).await
    }

    pub async fn unassign_role(&self, user_id: Uuid, role_id: Uuid) -> Result<(), Error> {
        self.user_role_repo.unassign_role(&user_id.to_string(), &role_id.to_string()).await
    }
    pub async fn get_roles_for_user(&self, user_id: Uuid) -> Result<Vec<Role>, Error> {
        let role_rows = self.role_repo.get_roles_for_user(user_id).await?;
        Ok(role_rows.into_iter().map(|r| Role { id: Uuid::parse_str(&r.id).expect("Invalid UUID format"), name: r.name }).collect())
    }
    pub async fn create_role(&self, name: &str) -> Result<Role, Error> {
        let row = self.role_repo.create_role(name).await?;
        Ok(Role {
            id: Uuid::parse_str(&row.id).expect("Invalid UUID format"),
            name: row.name,
        })
    }
    pub async fn get_role(&self, role_id: Uuid) -> Result<Option<Role>, Error> {
        let role_row = self.role_repo.get_role(role_id).await?;
        if let Some(row) = role_row {
            Ok(Some(Role {
                id: Uuid::parse_str(&row.id).expect("Invalid UUID format"),
                name: row.name,
            }))
        } else {
            Ok(None)
        }
    }
    pub async fn update_role(&self, role_id: Uuid, name: &str) -> Result<Role, Error> {
        let row = self.role_repo.update_role(role_id, name).await?;
        Ok(Role {
            id: Uuid::parse_str(&row.id).expect("Invalid UUID format"),
            name: row.name,
        })
    }
    pub async fn delete_role(&self, role_id: Uuid) -> Result<(), Error> {
        self.role_repo.delete_role(role_id).await
    }
    pub async fn get_users(&self) -> Result<Vec<User>, Error> {
        let user_rows = self.user_repo.get_users().await?;
        let mut users = Vec::new();
        for row in user_rows {
            let roles = self.role_repo.get_roles_for_user(
                            Uuid::parse_str(&row.id).expect("invalid UUID format") 
                        ).await?
                .into_iter()
                .map(|r| Role { id: Uuid::parse_str(&r.id).expect("Invalid UUID format"), name: r.name })
                .collect();
            users.push(User {
                id: Uuid::parse_str(&row.id).expect("Invalid UUID format"),
                name: row.name,
                email: row.email,
                roles,
            });
        }
        Ok(users)
    }
    pub async fn get_roles(&self) -> Result<Vec<Role>, Error> {
        let role_rows = self.role_repo.get_roles().await?;
        Ok(role_rows.into_iter().map(|r| Role { id: Uuid::parse_str(&r.id).expect("Invalid UUID format"), name: r.name }).collect())
    }
    pub async fn get_users_by_role(&self, role_id: Uuid) -> Result<Vec<User>, Error> {
        let user_rows = self.user_repo.get_users_by_role(role_id).await?;
        let mut users = Vec::new();
        for row in user_rows {
            let roles = self.role_repo.get_roles_for_user(
                            Uuid::parse_str(&row.id).expect("invalid UUID format") 
                        ).await?
                .into_iter()
                .map(|r| Role { id: Uuid::parse_str(&r.id).expect("Invalid UUID format"), name: r.name })
                .collect();
            users.push(User {
                id: Uuid::parse_str(&row.id).expect("Invalid UUID format"),
                name: row.name,
                email: row.email,
                roles,
            });
        }
        Ok(users)
    }

}