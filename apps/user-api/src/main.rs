#[derive(Clone, Debug)]
struct AppState {
    user_service: Arc<UserService>,
    env: String,
}

impl AppState {
    fn is_prod_like(&self) -> bool {
        // Treat any environment starting with "prod" as production-like (prod01, prod02, ...)
        self.env.to_lowercase().starts_with("prod")
    }
}
use axum::{
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use tokio;
use tower_http::trace::{DefaultMakeSpan, DefaultOnResponse, TraceLayer};
use tracing_subscriber::EnvFilter;
use user_lib::{user_service::UserService, util::connect_with_retry};
use user_lib::repository::role_repository::RoleRepository;
use user_lib::repository::user_repository::UserRepository;
use user_lib::repository::user_role_repository::UserRoleRepository;
use user_lib::entities::User;
use user_lib::errors_service::UserServiceError;
use uuid::Uuid;
use utoipa::ToSchema;
use tokio::net::TcpListener;

use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;
use std::sync::Arc;

#[derive(OpenApi)]
#[openapi(
    paths(create_user, get_user_by_id),
    components(schemas(CreateUserRequest, UserResponse)),
    tags(
        (name = "users", description = "User management endpoints")
    )
)]
struct ApiDoc;


#[tokio::main]
async fn main() {
    // Setup tracing subscriber
    // Use RUST_LOG if set, otherwise default to INFO for app + DEBUG for HTTP tracing.
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info"));

    let subscriber = tracing_subscriber::fmt()
        .with_env_filter(filter)
        //.with_max_level(Level::DEBUG)
        .finish();

    tracing::subscriber::set_global_default(subscriber)
        .expect("setting default subscriber failed");

    //Global environment variable
    let env = std::env::var("ENV").expect("ENV must be set");

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
        .route("/users/{id}", get(get_user_by_id))
        .merge(SwaggerUi::new("/docs").url("/api-doc/openapi.json", ApiDoc::openapi()))
        .with_state(AppState { user_service: Arc::new(user_service), env: env.clone() })
        // Log one line per request/response at DEBUG (method, path, status, latency)
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(DefaultMakeSpan::new().level(tracing::Level::DEBUG))
                .on_response(DefaultOnResponse::new().level(tracing::Level::DEBUG)),
        );

    // Read port from env (default to 3333)
    let port: u16 = std::env::var("USER_API_PORT")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(3333);

    let addr = format!("0.0.0.0:{}", port);
    let public_url = format!("http://127.0.0.1:{}", port);

    let listener = TcpListener::bind(&addr).await.unwrap();

    tracing::info!(
        "user-api is ready to accept requests at: {}",
        public_url
    );

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
pub struct UserResponse {
    pub id: Uuid,
    pub name: String,
    pub email: String,
}

impl From<User> for UserResponse {
    fn from(user: User) -> Self {
        UserResponse {
            id: user.id,
            name: user.name,
            email: user.email,
        }
    }
}


#[utoipa::path(
    post,
    path = "/users",
    request_body = CreateUserRequest,
    responses(
        (status = 200, description = "User created successfully", body = UserResponse),
        (status = 409, description = "Email already exists"),
        (status = 500, description = "Internal server error"),
    )
)]

async fn create_user(
    axum::extract::State(state): axum::extract::State<AppState>,
    Json(payload): Json<CreateUserRequest>,
) -> Result<Json<UserResponse>, (axum::http::StatusCode, String)> {
    let user_service = state.user_service.clone();
    let env = state.env.clone();
    let prod_like = state.is_prod_like();
    user_service
        .create_user(&payload.name, &payload.email)
        .await
        .map(|user| Json(UserResponse::from(user)))
        .map_err(|e| {
            match e {
                UserServiceError::EmailAlreadyExists => {
                    (axum::http::StatusCode::CONFLICT, UserServiceError::EmailAlreadyExists.to_string())
                }
                UserServiceError::RoleNameAlreadyExists => (axum::http::StatusCode::CONFLICT, e.to_string()),
                UserServiceError::UserAlreadyHasRole => (axum::http::StatusCode::CONFLICT, e.to_string()),
                other => {
                    tracing::error!(env = %env, error = ?other, "create_user failed");
                    if prod_like {
                        (axum::http::StatusCode::INTERNAL_SERVER_ERROR, "internal server error".to_string())
                    } else {
                        (axum::http::StatusCode::INTERNAL_SERVER_ERROR, other.to_string())
                    }
                }
            }
        })
}

#[utoipa::path(
    get,
    path = "/users/{id}",
    params(
        ("id" = String, Path, description = "User ID (UUID)")
    ),
    responses(
        (status = 200, description = "User found", body = UserResponse),
        (status = 404, description = "User not found"),
        (status = 400, description = "Invalid UUID"),
        (status = 500, description = "Internal server error"),
    )
)]
async fn get_user_by_id(
    axum::extract::Path(id): axum::extract::Path<String>,
    axum::extract::State(state): axum::extract::State<AppState>,
) -> Result<Json<UserResponse>, (axum::http::StatusCode, String)> {
    let user_service = state.user_service.clone();
    let env = state.env.clone();
    let prod_like = state.is_prod_like();
    match Uuid::parse_str(&id) {
        Ok(parsed_id) => {
            match user_service.get_user(parsed_id).await {
                Ok(Some(user)) => Ok(Json(UserResponse::from(user))),
                Ok(None) => Err((axum::http::StatusCode::NOT_FOUND, "user not found".to_string())),
                Err(e) => {
                    tracing::error!(env = %env, error = ?e, "get_user failed");
                    if prod_like {
                        Err((axum::http::StatusCode::INTERNAL_SERVER_ERROR, "internal server error".to_string()))
                    } else {
                        Err((axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
                    }
                }
            }
        }
        Err(_) => Err((axum::http::StatusCode::BAD_REQUEST, "invalid uuid".to_string())),
    }
}