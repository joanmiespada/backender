use std::sync::Arc;
use user_lib::user_service::UserService;

#[derive(Clone, Debug)]
pub struct AppState {
    pub user_service: Arc<UserService>,
    pub env: String,
}

impl AppState {
    pub fn is_prod_like(&self) -> bool {
        // Treat any environment starting with "prod" as production-like (prod01, prod02, ...)
        self.env.to_lowercase().starts_with("prod")
    }
}