use std::sync::Arc;
use user_lib::repository::role_repository::RoleRepository;
use user_lib::repository::user_repository::UserRepository;
use user_lib::repository::user_role_repository::UserRoleRepository;
use user_lib::repository::traits::{RoleRepositoryTrait, UserRepositoryTrait, UserRoleRepositoryTrait};
use user_lib::user_service::UserService;

#[derive(Clone, Debug)]
pub struct AppState<U = UserRepository, R = RoleRepository, UR = UserRoleRepository>
where
    U: UserRepositoryTrait + Send + Sync + 'static,
    R: RoleRepositoryTrait + Send + Sync + 'static,
    UR: UserRoleRepositoryTrait + Send + Sync + 'static,
{
    pub user_service: Arc<UserService<U, R, UR>>,
    pub env: String,
}
