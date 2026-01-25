pub const SERVICE: &str = "user-api";
pub const ENV: &str = "ENV";

//pub const PRODUCTION_ENV : &str = "prod01";
//pub const TEST_ENV : &str = "test01";
//pub const DEVELOPMENT_ENV : &str = "dev01";
pub const LOCAL_ENV: &str = "local";

pub const DATABASE_URL: &str = "DATABASE_URL";
pub const ELASTIC_URL: &str = "ELASTIC_URL";

pub const USER_API_PORT: &str = "USER_API_PORT";

// Redis configuration
pub const REDIS_HOST: &str = "REDIS_HOST";
pub const REDIS_PORT: &str = "REDIS_PORT";
pub const REDIS_DB: &str = "REDIS_DB";

// Cache configuration
pub const CACHE_ENABLED: &str = "CACHE_ENABLED";
pub const CACHE_USER_TTL_SECS: &str = "CACHE_USER_TTL_SECS";
pub const CACHE_ROLE_TTL_SECS: &str = "CACHE_ROLE_TTL_SECS";
pub const CACHE_LIST_TTL_SECS: &str = "CACHE_LIST_TTL_SECS";

// Middleware configuration
pub const RATE_LIMIT_PER_MINUTE: &str = "RATE_LIMIT_PER_MINUTE";
pub const RATE_LIMIT_BURST: &str = "RATE_LIMIT_BURST";
pub const REQUEST_TIMEOUT_SECS: &str = "REQUEST_TIMEOUT_SECS";
pub const CORS_ALLOWED_ORIGINS: &str = "CORS_ALLOWED_ORIGINS";
pub const MAX_BODY_SIZE_BYTES: &str = "MAX_BODY_SIZE_BYTES";
pub const IP_ALLOWLIST: &str = "IP_ALLOWLIST";
pub const IP_BLOCKLIST: &str = "IP_BLOCKLIST";
pub const SHUTDOWN_TIMEOUT_SECS: &str = "SHUTDOWN_TIMEOUT_SECS";