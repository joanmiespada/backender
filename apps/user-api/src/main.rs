use axum::{
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use tokio;
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;
use user_lib::{user_service::UserService, util::connect_with_retry};
use user_lib::repository::role_repository::RoleRepository;
use user_lib::repository::user_repository::UserRepository;
use user_lib::repository::user_role_repository::UserRoleRepository;
use user_lib::entities::User;
use uuid::Uuid;
use utoipa::ToSchema;
use tokio::net::TcpListener;

use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

#[derive(OpenApi)]
#[openapi(
    paths(create_user, get_user_by_id),
    components(schemas(CreateUserRequest, User)),
    tags(
        (name = "users", description = "User management endpoints")
    )
)]
struct ApiDoc;


#[tokio::main]
async fn main() {
    // Setup tracing subscriber
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();

    tracing::subscriber::set_global_default(subscriber)
        .expect("setting default subscriber failed");

        // Setup database pool
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let pool = connect_with_retry(&database_url, 10).await;
    // Create shared service
    let user_service = UserService::new(
        UserRepository::new(pool.clone()),
        RoleRepository::new(pool.clone()),
        UserRoleRepository::new(pool.clone()),
    );

    // Build application with routes
    let app = Router::new()
        .route("/health", get(health_check))
        .route("/users", post(create_user))
        .route("/users/:id", get(get_user_by_id))
        .merge(SwaggerUi::new("/docs").url("/api-doc/openapi.json", ApiDoc::openapi()))
        .with_state(user_service);

    // Run server
    let listener = TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
    
}

async fn health_check() -> &'static str {
    "OK"
}

#[derive(Debug, Deserialize, Serialize, ToSchema)]
struct CreateUserRequest {
    name: String,
    email: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct User {
    pub id: Uuid,
    pub name: String,
    pub email: String,
}


#[utoipa::path(
    post,
    path = "/users",
    request_body = CreateUserRequest,
    responses(
        (status = 200, description = "User created successfully", body = User),
        (status = 500, description = "Internal server error"),
    )
)]
async fn create_user(
    axum::extract::State(user_service): axum::extract::State<UserService>,
    Json(payload): Json<CreateUserRequest>,
) -> Result<Json<User>, axum::http::StatusCode> {
    user_service.create_user(&payload.name, &payload.email)
        .await
        .map(Json)
        .map_err(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR)
}

#[utoipa::path(
    get,
    path = "/users/{id}",
    params(
        ("id" = String, Path, description = "User ID (UUID)")
    ),
    responses(
        (status = 200, description = "User found", body = User),
        (status = 404, description = "User not found"),
        (status = 400, description = "Invalid UUID"),
        (status = 500, description = "Internal server error"),
    )
)]
async fn get_user_by_id(
    axum::extract::Path(id): axum::extract::Path<String>,
    axum::extract::State(user_service): axum::extract::State<UserService>,
) -> Result<Json<User>, axum::http::StatusCode> {
    match Uuid::parse_str(&id) {
        Ok(parsed_id) => {
            match user_service.get_user(parsed_id).await {
                Ok(Some(user)) => Ok(Json(user)),
                Ok(None) => Err(axum::http::StatusCode::NOT_FOUND),
                Err(_) => Err(axum::http::StatusCode::INTERNAL_SERVER_ERROR),
            }
        }
        Err(_) => Err(axum::http::StatusCode::BAD_REQUEST),
    }
}