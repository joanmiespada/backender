use std::sync::Arc;
use user_lib::user_service::UserService;

#[derive(Clone, Debug)]
pub struct AppState {
    pub user_service: Arc<UserService>,
    pub env: String,
}

