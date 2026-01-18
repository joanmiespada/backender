
use crate::methods::routes::SERVICE_HEALTH_PATH;

#[utoipa::path(
    get,
    path = SERVICE_HEALTH_PATH,
    responses(
        (status = 200, description = "System is healthy", body = String),
        (status = 500, description = "Internal server error"),
    )
)]
pub async fn health_check() -> &'static str {
    "OK"
}