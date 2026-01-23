
mod state;
mod constants;
mod methods;

use axum::{
    routing::get,
    Router,
};
use tokio;
use tower_http::trace::{DefaultMakeSpan, DefaultOnResponse, TraceLayer};
use tracing_subscriber::EnvFilter;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use user_lib::{user_service::UserService, util::connect_with_retry};
use user_lib::repository::role_repository::RoleRepository;
use user_lib::repository::user_repository::UserRepository;
use user_lib::repository::user_role_repository::UserRoleRepository;
use tokio::net::TcpListener;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;
use std::sync::Arc;

use crate::{constants::{LOCAL_ENV,USER_API_PORT, SERVICE}};
use crate::methods::health_check::health_check;
use crate::methods::create_user::create_user;
use crate::methods::create_user::__path_create_user;
use crate::methods::get_user_by_id::get_user_by_id;
use crate::methods::get_user_by_id::__path_get_user_by_id;
use crate::methods::get_users::get_users;
use crate::methods::get_users::__path_get_users;
use crate::methods::entities::{CreateUserRequest, UserResponse, PaginatedResponse};
use crate::state::AppState;
use crate::methods::routes::{USERS_PATH, USERS_BY_ID_PATH, SERVICE_HEALTH_PATH, SERVICE_DOCS_PATH};
use crate::constants::{ENV, ELASTIC_URL, DATABASE_URL};


#[derive(OpenApi)]
#[openapi(
    paths(create_user, get_user_by_id, get_users),
    components(schemas(CreateUserRequest, UserResponse, PaginatedResponse<UserResponse>)),
    tags(
        (name = "users", description = "User management endpoints")
    )
)]
struct ApiDoc;


#[tokio::main]
async fn main() {
    // Setup tracing subscriber
    // - Always: JSON logs to STDOUT (so they can be shipped to Elasticsearch)
    // - Additionally on local: pretty logs to STDERR (no duplicates in STDOUT)
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    let env = std::env::var(ENV).expect( format!("{} must be set", ENV).as_str());

    // We don't send logs directly to Elasticsearch from the app.
    // In shared envs, a shipper (Elastic Agent / Filebeat / Fluent Bit) forwards STDOUT to Elasticsearch.
    let _elastic_url = std::env::var(ELASTIC_URL).ok();

    let registry = tracing_subscriber::registry().with(filter);

    let json_layer = tracing_subscriber::fmt::layer()
        .json()
        .with_current_span(true)
        .with_span_list(true);

    if env == LOCAL_ENV {
        let pretty_layer = tracing_subscriber::fmt::layer()
            .with_writer(std::io::stderr)
            .pretty();
        registry.with(json_layer).with(pretty_layer).init();
        //registry.with(pretty_layer).init();
    } else {
        registry.with(json_layer).init();
    }

    tracing::info!(service=SERVICE, env = %env, "tracing initialized");

    // Setup database pool
    let database_url = std::env::var(DATABASE_URL).expect( format!("{} must be set", DATABASE_URL).as_str());

    let pool = connect_with_retry(&database_url, 10).await;
    // Create shared service
    let user_service = UserService::new(
        UserRepository::new(pool.clone()),
        RoleRepository::new(pool.clone()),
        UserRoleRepository::new(pool.clone()),
    );

    // Build application with routes
    let app = Router::new()
        .route(SERVICE_HEALTH_PATH, get(health_check))
        .route(USERS_PATH, get(get_users).post(create_user))
        .route(USERS_BY_ID_PATH, get(get_user_by_id))
        .merge(SwaggerUi::new(SERVICE_DOCS_PATH).url("/api-doc/openapi.json", ApiDoc::openapi()))
        .with_state(AppState { user_service: Arc::new(user_service), env: env.clone() })
        // Log one line per request/response at DEBUG (method, path, status, latency)
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(DefaultMakeSpan::new().level(tracing::Level::DEBUG))
                .on_response(DefaultOnResponse::new().level(tracing::Level::DEBUG)),
        );

    // Read port from env (default to 3333)
    let port: u16 = std::env::var(USER_API_PORT)
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




