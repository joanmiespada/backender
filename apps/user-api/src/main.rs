mod cache;
mod config;
mod constants;
mod error;
mod keycloak;
mod methods;
mod middleware;
mod services;
mod shutdown;
mod state;

use axum::{
    http::{header, HeaderName, Method, StatusCode},
    middleware::from_fn,
    routing::{get, post},
    Extension, Router,
};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpListener;
use tower_governor::{governor::GovernorConfigBuilder, GovernorLayer};
use tower_http::{
    cors::{Any, CorsLayer},
    limit::RequestBodyLimitLayer,
    request_id::{MakeRequestUuid, PropagateRequestIdLayer, SetRequestIdLayer},
    timeout::TimeoutLayer,
    trace::{DefaultMakeSpan, DefaultOnResponse, TraceLayer},
};
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::EnvFilter;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use secrets::SecretsConfig;
use user_lib::repository::role_repository::RoleRepository;
use user_lib::repository::user_repository::UserRepository;
use user_lib::repository::user_role_repository::UserRoleRepository;
use user_lib::user_service::UserService;
use user_lib::util::connect_with_retry;

use crate::cache::{CacheConfig, CachedUserService, RedisCache};
use crate::config::MiddlewareConfig;
use crate::constants::{DATABASE_URL, ELASTIC_URL, ENV, LOCAL_ENV, SERVICE, USER_API_PORT};
use crate::keycloak::{KeycloakClient, KeycloakConfig};
use crate::methods::assign_role::__path_assign_role;
use crate::methods::assign_role::assign_role;
use crate::methods::create_role::__path_create_role;
use crate::methods::create_role::create_role;
use crate::methods::create_user::__path_create_user;
use crate::methods::create_user::create_user;
use crate::methods::delete_role::__path_delete_role;
use crate::methods::delete_role::delete_role;
use crate::methods::delete_user::__path_delete_user;
use crate::methods::delete_user::delete_user;
use crate::methods::entities::{
    CreateRoleRequest, CreateUserRequest, PaginatedResponse, RoleResponse, UpdateRoleRequest,
    UpdateUserRequest, UserResponse,
};
use crate::methods::get_role_by_id::__path_get_role_by_id;
use crate::methods::get_role_by_id::get_role_by_id;
use crate::methods::get_roles::__path_get_roles;
use crate::methods::get_roles::get_roles;
use crate::methods::get_user_by_id::__path_get_user_by_id;
use crate::methods::get_user_by_id::get_user_by_id;
use crate::methods::get_users::__path_get_users;
use crate::methods::get_users::get_users;
use crate::methods::health_check::health_check;
use crate::methods::routes::{
    API_V1_PREFIX, ROLES_BY_ID_PATH, ROLES_PATH, SERVICE_DOCS_PATH, SERVICE_HEALTH_PATH,
    USERS_BY_ID_PATH, USERS_PATH, USER_ROLES_PATH,
};
use crate::methods::unassign_role::__path_unassign_role;
use crate::methods::unassign_role::unassign_role;
use crate::methods::update_role::__path_update_role;
use crate::methods::update_role::update_role;
use crate::methods::update_user::__path_update_user;
use crate::methods::update_user::update_user;
use crate::middleware::ip_filter::{ip_filter_middleware, IpFilterConfig};
use crate::services::IntegratedUserService;
use crate::shutdown::shutdown_signal;
use crate::state::AppState;

#[derive(OpenApi)]
#[openapi(
    paths(
        create_user, get_user_by_id, get_users, update_user, delete_user,
        create_role, get_role_by_id, get_roles, update_role, delete_role,
        assign_role, unassign_role
    ),
    components(schemas(
        CreateUserRequest, UpdateUserRequest, UserResponse,
        CreateRoleRequest, UpdateRoleRequest, RoleResponse,
        PaginatedResponse<UserResponse>, PaginatedResponse<RoleResponse>
    )),
    tags(
        (name = "users", description = "User management endpoints"),
        (name = "roles", description = "Role management endpoints")
    )
)]
struct ApiDoc;

#[tokio::main]
async fn main() {
    if let Err(e) = run().await {
        eprintln!("Fatal error: {e}");
        std::process::exit(1);
    }
}

async fn run() -> Result<(), Box<dyn std::error::Error>> {
    // Setup tracing subscriber
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    let env = std::env::var(ENV).map_err(|_| format!("{ENV} environment variable must be set"))?;

    // We don't send logs directly to Elasticsearch from the app.
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
    } else {
        registry.with(json_layer).init();
    }

    tracing::info!(service = SERVICE, env = %env, "tracing initialized");

    // Initialize secrets client (tries Infisical first, falls back to env vars)
    let secrets_config = SecretsConfig::from_env();
    let secrets_client = secrets::SecretsClient::new(secrets_config).await;

    if secrets_client.has_primary_provider() {
        tracing::info!("Secrets client initialized with Infisical provider");
    } else {
        tracing::info!("Secrets client using environment variables only");
    }

    // Load middleware configuration from environment
    let middleware_config = MiddlewareConfig::from_env();
    tracing::info!(
        rate_limit_per_minute = middleware_config.rate_limit_per_minute,
        rate_limit_burst = middleware_config.rate_limit_burst,
        request_timeout_secs = middleware_config.request_timeout.as_secs(),
        max_body_size = middleware_config.max_body_size,
        cors_origins = ?middleware_config.cors_allowed_origins,
        ip_filter_enabled = middleware_config.has_ip_filter(),
        "middleware configuration loaded"
    );

    // Setup database pool
    let database_url = std::env::var(DATABASE_URL)
        .map_err(|_| format!("{DATABASE_URL} environment variable must be set"))?;

    let pool = connect_with_retry(&database_url, 10).await?;

    // Setup cache
    let cache_config = CacheConfig::from_env();
    tracing::info!(
        cache_enabled = cache_config.enabled,
        redis_host = %cache_config.redis_host,
        redis_port = cache_config.redis_port,
        user_ttl_secs = cache_config.user_ttl.as_secs(),
        role_ttl_secs = cache_config.role_ttl.as_secs(),
        list_ttl_secs = cache_config.list_ttl.as_secs(),
        "cache configuration loaded"
    );

    let redis_cache = RedisCache::new(&cache_config).await;
    if cache_config.enabled && !redis_cache.is_enabled() {
        tracing::warn!("Cache was enabled but Redis connection failed - running in DB-only mode");
    }

    // Setup Keycloak client (secrets loaded via secrets client)
    let keycloak_config = KeycloakConfig::from_secrets(&secrets_client).await;
    tracing::info!(
        keycloak_url = %keycloak_config.base_url,
        keycloak_realm = %keycloak_config.realm,
        keycloak_configured = keycloak_config.is_configured(),
        profile_cache_ttl_secs = keycloak_config.profile_cache_ttl.as_secs(),
        "keycloak configuration loaded"
    );

    let keycloak_client = Arc::new(KeycloakClient::new(keycloak_config));

    if !keycloak_client.is_configured() {
        tracing::warn!("Keycloak client secret not set - user creation/update will fail");
    }

    // Create shared service
    let user_service = UserService::new(
        UserRepository::new(pool.clone()),
        RoleRepository::new(pool.clone()),
        UserRoleRepository::new(pool.clone()),
    );

    let cached_service =
        CachedUserService::new(Arc::new(user_service), redis_cache.clone(), cache_config);

    // Create integrated service that wraps cached service + keycloak
    let integrated_service =
        IntegratedUserService::new(Arc::new(cached_service), keycloak_client, redis_cache);

    let app_state = AppState {
        user_service: Arc::new(integrated_service),
        env: env.clone(),
    };

    // Build versioned API routes (v1)
    let v1_routes = Router::new()
        // User endpoints
        .route(USERS_PATH, get(get_users).post(create_user))
        .route(
            USERS_BY_ID_PATH,
            get(get_user_by_id).put(update_user).delete(delete_user),
        )
        // Role endpoints
        .route(ROLES_PATH, get(get_roles).post(create_role))
        .route(
            ROLES_BY_ID_PATH,
            get(get_role_by_id).put(update_role).delete(delete_role),
        )
        // User-role assignment endpoints
        .route(USER_ROLES_PATH, post(assign_role).delete(unassign_role));

    // Build root-level routes (health, docs)
    let root_routes = Router::new()
        .route(SERVICE_HEALTH_PATH, get(health_check))
        .merge(SwaggerUi::new(SERVICE_DOCS_PATH).url("/api-doc/openapi.json", ApiDoc::openapi()));

    // Combine routes: nest v1 under /v1, keep health and docs at root
    let mut app = Router::new()
        .nest(API_V1_PREFIX, v1_routes)
        .merge(root_routes)
        .with_state(app_state);

    // ============================================
    // Middleware stack (applied inner to outer)
    // Order: Request → Rate Limit → IP Filter → Timeout → CORS → Body Limit → Request ID → Trace → Handler
    // ============================================

    // 1. Trace layer (innermost - closest to handler)
    app = app.layer(
        TraceLayer::new_for_http()
            .make_span_with(DefaultMakeSpan::new().level(tracing::Level::DEBUG))
            .on_response(DefaultOnResponse::new().level(tracing::Level::DEBUG)),
    );

    // 2. Request ID layers
    let x_request_id = HeaderName::from_static("x-request-id");
    app = app
        .layer(PropagateRequestIdLayer::new(x_request_id.clone()))
        .layer(SetRequestIdLayer::new(
            x_request_id.clone(),
            MakeRequestUuid,
        ));

    // 3. Body limit layer
    app = app.layer(RequestBodyLimitLayer::new(middleware_config.max_body_size));

    // 4. CORS layer
    let cors_layer = if middleware_config
        .cors_allowed_origins
        .contains(&"*".to_string())
    {
        CorsLayer::new()
            .allow_origin(Any)
            .allow_methods([
                Method::GET,
                Method::POST,
                Method::PUT,
                Method::DELETE,
                Method::OPTIONS,
            ])
            .allow_headers([header::CONTENT_TYPE, header::AUTHORIZATION, x_request_id])
    } else {
        let origins: Vec<_> = middleware_config
            .cors_allowed_origins
            .iter()
            .filter_map(|o| o.parse().ok())
            .collect();
        CorsLayer::new()
            .allow_origin(origins)
            .allow_methods([
                Method::GET,
                Method::POST,
                Method::PUT,
                Method::DELETE,
                Method::OPTIONS,
            ])
            .allow_headers([
                header::CONTENT_TYPE,
                header::AUTHORIZATION,
                HeaderName::from_static("x-request-id"),
            ])
    };
    app = app.layer(cors_layer);

    // 5. Timeout layer (returns 408 Request Timeout)
    app = app.layer(TimeoutLayer::with_status_code(
        StatusCode::REQUEST_TIMEOUT,
        middleware_config.request_timeout,
    ));

    // 6. IP filter middleware (only if configured)
    if middleware_config.has_ip_filter() {
        let ip_config = IpFilterConfig::new(
            middleware_config.ip_allowlist.clone(),
            middleware_config.ip_blocklist.clone(),
        );
        app = app
            .layer(Extension(ip_config))
            .layer(from_fn(ip_filter_middleware));
        tracing::info!("IP filter middleware enabled");
    }

    // 7. Rate limiting layer (outermost)
    // Calculate milliseconds between requests: 60000ms / requests_per_minute
    let replenish_interval_ms = 60_000 / middleware_config.rate_limit_per_minute as u64;
    let governor_conf = Arc::new(
        GovernorConfigBuilder::default()
            .per_millisecond(replenish_interval_ms)
            .burst_size(middleware_config.rate_limit_burst)
            .finish()
            .expect("failed to build governor config"),
    );
    app = app.layer(GovernorLayer {
        config: governor_conf,
    });

    // Read port from env (default to 3333)
    let port: u16 = std::env::var(USER_API_PORT)
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(3333);

    let addr = format!("0.0.0.0:{port}");
    let public_url = format!("http://127.0.0.1:{port}");

    let listener = TcpListener::bind(&addr)
        .await
        .map_err(|e| format!("Failed to bind to {addr}: {e}"))?;

    tracing::info!("user-api is ready to accept requests at: {}", public_url);
    tracing::info!("API v1 endpoints available at: {}/v1", public_url);

    // Serve with graceful shutdown
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .with_graceful_shutdown(shutdown_signal(middleware_config.shutdown_timeout))
    .await
    .map_err(|e| format!("Server error: {e}"))?;

    Ok(())
}
