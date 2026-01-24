use axum::{extract::Query, Json};
use crate::methods::entities::{PaginatedResponse, PaginationQuery, RoleResponse};
use crate::state::AppState;
use crate::methods::routes::ROLES_PATH;

#[utoipa::path(
    get,
    path = ROLES_PATH,
    params(PaginationQuery),
    responses(
        (status = 200, description = "List of roles", body = PaginatedResponse<RoleResponse>),
        (status = 500, description = "Internal server error"),
    )
)]
pub async fn get_roles(
    axum::extract::State(state): axum::extract::State<AppState>,
    Query(pagination): Query<PaginationQuery>,
) -> Result<Json<PaginatedResponse<RoleResponse>>, (axum::http::StatusCode, String)> {
    let user_service = state.user_service.clone();
    let env = state.env.clone();
    let prod_like = state.is_prod_like();

    user_service
        .get_roles(pagination.into())
        .await
        .map(|result| Json(PaginatedResponse::from(result)))
        .map_err(|e| {
            tracing::error!(env = %env, error = ?e, "get_roles failed");
            if prod_like {
                (axum::http::StatusCode::INTERNAL_SERVER_ERROR, "internal server error".to_string())
            } else {
                (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
            }
        })
}
