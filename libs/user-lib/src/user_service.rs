use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;
use crate::entities::{User, Role, PaginatedResult, PaginationParams};
use crate::repository::{RoleRepository, UserRepository, UserRoleRepository};
use crate::repository::models::{RoleRow, UserRow, UserRoleMapping};
use crate::repository::traits::{RoleRepositoryTrait, UserRepositoryTrait, UserRoleRepositoryTrait};
use crate::errors_service::UserServiceError;

fn parse_uuid(s: &str) -> Result<Uuid, UserServiceError> {
    Uuid::parse_str(s).map_err(|_| UserServiceError::InvalidUuid(s.to_string()))
}

const MAX_NAME_LENGTH: usize = 255;
const MAX_EMAIL_LENGTH: usize = 255;
const MAX_ROLE_NAME_LENGTH: usize = 255;

fn validate_name(name: &str) -> Result<(), UserServiceError> {
    let name = name.trim();
    if name.is_empty() {
        return Err(UserServiceError::Validation("name cannot be empty".to_string()));
    }
    if name.len() > MAX_NAME_LENGTH {
        return Err(UserServiceError::Validation(
            format!("name cannot exceed {} characters", MAX_NAME_LENGTH)
        ));
    }
    Ok(())
}

fn validate_email(email: &str) -> Result<(), UserServiceError> {
    let email = email.trim();
    if email.is_empty() {
        return Err(UserServiceError::Validation("email cannot be empty".to_string()));
    }
    if email.len() > MAX_EMAIL_LENGTH {
        return Err(UserServiceError::Validation(
            format!("email cannot exceed {} characters", MAX_EMAIL_LENGTH)
        ));
    }

    // Must contain exactly one @
    let at_count = email.matches('@').count();
    if at_count != 1 {
        return Err(UserServiceError::Validation("email must contain exactly one @".to_string()));
    }

    let parts: Vec<&str> = email.split('@').collect();
    let local = parts[0];
    let domain = parts[1];

    // Validate local part
    if local.is_empty() || local.len() > 64 {
        return Err(UserServiceError::Validation("email local part is invalid".to_string()));
    }
    if local.starts_with('.') || local.ends_with('.') {
        return Err(UserServiceError::Validation("email local part cannot start or end with a dot".to_string()));
    }
    if local.contains("..") {
        return Err(UserServiceError::Validation("email local part cannot contain consecutive dots".to_string()));
    }
    // Allow alphanumeric, dots, hyphens, underscores, and plus signs in local part
    for c in local.chars() {
        if !c.is_ascii_alphanumeric() && !".+-_".contains(c) {
            return Err(UserServiceError::Validation(format!(
                "email local part contains invalid character: {}", c
            )));
        }
    }

    // Validate domain part
    if domain.is_empty() || domain.len() > 255 {
        return Err(UserServiceError::Validation("email domain is invalid".to_string()));
    }
    if !domain.contains('.') {
        return Err(UserServiceError::Validation("email domain must contain a dot".to_string()));
    }
    if domain.starts_with('.') || domain.ends_with('.') {
        return Err(UserServiceError::Validation("email domain cannot start or end with a dot".to_string()));
    }
    if domain.starts_with('-') || domain.ends_with('-') {
        return Err(UserServiceError::Validation("email domain cannot start or end with a hyphen".to_string()));
    }
    if domain.contains("..") {
        return Err(UserServiceError::Validation("email domain cannot contain consecutive dots".to_string()));
    }

    // Validate TLD (at least 2 characters)
    let tld = domain.rsplit('.').next().unwrap_or("");
    if tld.len() < 2 {
        return Err(UserServiceError::Validation("email domain must have a valid TLD".to_string()));
    }

    // Allow alphanumeric, dots, and hyphens in domain
    for c in domain.chars() {
        if !c.is_ascii_alphanumeric() && !".-".contains(c) {
            return Err(UserServiceError::Validation(format!(
                "email domain contains invalid character: {}", c
            )));
        }
    }

    Ok(())
}

fn validate_role_name(name: &str) -> Result<(), UserServiceError> {
    let name = name.trim();
    if name.is_empty() {
        return Err(UserServiceError::Validation("role name cannot be empty".to_string()));
    }
    if name.len() > MAX_ROLE_NAME_LENGTH {
        return Err(UserServiceError::Validation(
            format!("role name cannot exceed {} characters", MAX_ROLE_NAME_LENGTH)
        ));
    }
    Ok(())
}

fn role_from_row(row: RoleRow) -> Result<Role, UserServiceError> {
    Ok(Role {
        id: parse_uuid(&row.id)?,
        name: row.name,
    })
}

fn role_from_mapping(mapping: UserRoleMapping) -> Result<(String, Role), UserServiceError> {
    let role = Role {
        id: parse_uuid(&mapping.role_id)?,
        name: mapping.role_name,
    };
    Ok((mapping.user_id, role))
}

fn user_from_row(row: UserRow, roles: Vec<Role>) -> Result<User, UserServiceError> {
    Ok(User {
        id: parse_uuid(&row.id)?,
        name: row.name,
        email: row.email,
        roles,
    })
}

#[derive(Debug, Clone)]
pub struct UserService<U = UserRepository, R = RoleRepository, UR = UserRoleRepository>
where
    U: UserRepositoryTrait,
    R: RoleRepositoryTrait,
    UR: UserRoleRepositoryTrait,
{
    pub user_repo: Arc<U>,
    pub role_repo: Arc<R>,
    pub user_role_repo: Arc<UR>,
}

impl UserService<UserRepository, RoleRepository, UserRoleRepository> {
    pub fn new(user_repo: UserRepository, role_repo: RoleRepository, user_role_repo: UserRoleRepository) -> Self {
        Self {
            user_repo: Arc::new(user_repo),
            role_repo: Arc::new(role_repo),
            user_role_repo: Arc::new(user_role_repo),
        }
    }
}

impl<U, R, UR> UserService<U, R, UR>
where
    U: UserRepositoryTrait,
    R: RoleRepositoryTrait,
    UR: UserRoleRepositoryTrait,
{
    pub fn with_repos(user_repo: Arc<U>, role_repo: Arc<R>, user_role_repo: Arc<UR>) -> Self {
        Self { user_repo, role_repo, user_role_repo }
    }

    async fn fetch_roles_for_user(&self, user_id: Uuid) -> Result<Vec<Role>, UserServiceError> {
        self.role_repo
            .get_roles_for_user(user_id)
            .await
            .map_err(|e| UserServiceError::Internal(e.into()))?
            .into_iter()
            .map(role_from_row)
            .collect()
    }

    async fn build_users_with_roles(&self, user_rows: Vec<UserRow>) -> Result<Vec<User>, UserServiceError> {
        if user_rows.is_empty() {
            return Ok(vec![]);
        }

        let user_ids: Vec<String> = user_rows.iter().map(|r| r.id.clone()).collect();
        let role_mappings = self.role_repo
            .get_roles_for_users(&user_ids)
            .await
            .map_err(|e| UserServiceError::Internal(e.into()))?;

        let mut roles_by_user: HashMap<String, Vec<Role>> = HashMap::new();
        for mapping in role_mappings {
            let (user_id, role) = role_from_mapping(mapping)?;
            roles_by_user.entry(user_id).or_default().push(role);
        }

        user_rows
            .into_iter()
            .map(|row| {
                let roles = roles_by_user.remove(&row.id).unwrap_or_default();
                user_from_row(row, roles)
            })
            .collect()
    }

    pub async fn create_user(&self, name: &str, email: &str) -> Result<User, UserServiceError> {
        validate_name(name)?;
        validate_email(email)?;
        let row = self.user_repo
            .create_user(name.trim(), email.trim())
            .await
            .map_err(UserServiceError::from)?;
        user_from_row(row, vec![])
    }

    pub async fn get_user(&self, user_id: Uuid) -> Result<Option<User>, UserServiceError> {
        let user_row = self.user_repo.get_user(user_id).await.map_err(UserServiceError::from)?;
        match user_row {
            Some(row) => {
                let roles = self.fetch_roles_for_user(parse_uuid(&row.id)?).await?;
                Ok(Some(user_from_row(row, roles)?))
            }
            None => Ok(None),
        }
    }

    pub async fn update_user(&self, user_id: Uuid, name: &str, email: &str) -> Result<User, UserServiceError> {
        validate_name(name)?;
        validate_email(email)?;
        let row = self.user_repo.update_user(user_id, name.trim(), email.trim()).await.map_err(UserServiceError::from)?;
        let roles = self.fetch_roles_for_user(parse_uuid(&row.id)?).await?;
        user_from_row(row, roles)
    }

    pub async fn delete_user(&self, user_id: Uuid) -> Result<(), UserServiceError> {
        self.user_repo.delete_user(user_id).await.map_err(UserServiceError::from)
    }

    pub async fn assign_role(&self, user_id: Uuid, role_id: Uuid) -> Result<(), UserServiceError> {
        self.user_role_repo
            .assign_role(&user_id.to_string(), &role_id.to_string())
            .await
            .map_err(UserServiceError::from)
    }

    pub async fn unassign_role(&self, user_id: Uuid, role_id: Uuid) -> Result<(), UserServiceError> {
        self.user_role_repo
            .unassign_role(&user_id.to_string(), &role_id.to_string())
            .await
            .map_err(UserServiceError::from)
    }

    pub async fn get_roles_for_user(&self, user_id: Uuid) -> Result<Vec<Role>, UserServiceError> {
        self.fetch_roles_for_user(user_id).await
    }

    pub async fn create_role(&self, name: &str) -> Result<Role, UserServiceError> {
        validate_role_name(name)?;
        let row = self.role_repo.create_role(name.trim()).await.map_err(UserServiceError::from)?;
        role_from_row(row)
    }

    pub async fn get_role(&self, role_id: Uuid) -> Result<Option<Role>, UserServiceError> {
        let role_row = self.role_repo.get_role(role_id).await.map_err(|e| UserServiceError::Internal(e.into()))?;
        role_row.map(role_from_row).transpose()
    }

    pub async fn update_role(&self, role_id: Uuid, name: &str) -> Result<Role, UserServiceError> {
        validate_role_name(name)?;
        let row = self.role_repo.update_role(role_id, name.trim()).await.map_err(UserServiceError::from)?;
        role_from_row(row)
    }

    pub async fn delete_role(&self, role_id: Uuid) -> Result<(), UserServiceError> {
        self.role_repo.delete_role(role_id).await.map_err(|e| UserServiceError::Internal(e.into()))
    }

    pub async fn get_users(&self, pagination: PaginationParams) -> Result<PaginatedResult<User>, UserServiceError> {
        let (user_rows, total) = self.user_repo
            .get_users_paginated(pagination)
            .await
            .map_err(UserServiceError::from)?;
        let users = self.build_users_with_roles(user_rows).await?;
        Ok(PaginatedResult {
            items: users,
            total,
            page: pagination.page,
            page_size: pagination.page_size,
            total_pages: ((total as f64) / (pagination.page_size as f64)).ceil() as u32,
        })
    }

    pub async fn get_roles(&self, pagination: PaginationParams) -> Result<PaginatedResult<Role>, UserServiceError> {
        let (role_rows, total) = self.role_repo
            .get_roles_paginated(pagination)
            .await
            .map_err(|e| UserServiceError::Internal(e.into()))?;
        let roles: Vec<Role> = role_rows.into_iter().map(role_from_row).collect::<Result<_, _>>()?;
        Ok(PaginatedResult {
            items: roles,
            total,
            page: pagination.page,
            page_size: pagination.page_size,
            total_pages: ((total as f64) / (pagination.page_size as f64)).ceil() as u32,
        })
    }

    pub async fn get_users_by_role(&self, role_id: Uuid, pagination: PaginationParams) -> Result<PaginatedResult<User>, UserServiceError> {
        let (user_rows, total) = self.user_repo
            .get_users_by_role_paginated(role_id, pagination)
            .await
            .map_err(UserServiceError::from)?;
        let users = self.build_users_with_roles(user_rows).await?;
        Ok(PaginatedResult {
            items: users,
            total,
            page: pagination.page,
            page_size: pagination.page_size,
            total_pages: ((total as f64) / (pagination.page_size as f64)).ceil() as u32,
        })
    }
}
