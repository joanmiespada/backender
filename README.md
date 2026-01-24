# Local Dev Stack

Spin up the full backend shared platform locally.

## Quick Start

```bash
# Install dependencies (macOS)
brew install just mysql-client
cargo install cargo-watch

# Start dev environment with hot-reload
just dev
```

## Brew dependencies

```bash
brew install mysql-client
brew install dnsmasq
brew install just
```

## Included Services

- Redis (cache and stream)
- MySQL (service DB)
- phpMyAdmin
- Keycloak (auth)
- Unleash (feature toggles)
- ElasticSearch + Kibana (logs)
- Prometheus + Grafana (metrics)
- Postgres as database for dependencies


## Start docker

```bash
docker compose -f docker-compose.local.yml up --build
```

## Start k8

	•	http://unleash.127.0.0.1.nip.io
	•	http://keycloak.127.0.0.1.nip.io
	•	http://grafana.127.0.0.1.nip.io
	•	http://prometheus.127.0.0.1.nip.io
	•	http://kibana.127.0.0.1.nip.io
	•	http://traefik.127.0.0.1.nip.io


## User API

The user-api service provides user and role management endpoints.

### API Versioning

All API endpoints are versioned under `/v1/`:
- `GET /v1/users` - List users
- `POST /v1/users` - Create user
- `GET /v1/users/{id}` - Get user by ID
- `PUT /v1/users/{id}` - Update user
- `DELETE /v1/users/{id}` - Delete user
- `GET /v1/roles` - List roles
- `POST /v1/roles` - Create role
- `GET /v1/roles/{id}` - Get role by ID
- `PUT /v1/roles/{id}` - Update role
- `DELETE /v1/roles/{id}` - Delete role
- `POST /v1/users/{user_id}/roles/{role_id}` - Assign role to user
- `DELETE /v1/users/{user_id}/roles/{role_id}` - Unassign role from user

Root-level endpoints (not versioned):
- `GET /health` - Health check
- `GET /docs` - Swagger UI

### Middleware Stack

The API includes the following middleware (in order of execution):

| Middleware | Description | Default | HTTP Code |
|------------|-------------|---------|-----------|
| Rate Limiting | Limits requests per IP | 100/min, burst 150 | 429 |
| IP Filter | Allowlist/blocklist by IP | Disabled | 403 |
| Timeout | Request timeout | 30s | 408 |
| CORS | Cross-origin requests | Allow all | - |
| Body Limit | Max request body size | 1MB | 413 |
| Request ID | Adds `x-request-id` header | Enabled | - |
| Tracing | Request/response logging | Enabled | - |

### Environment Variables

```bash
# Required
DATABASE_URL=mysql://user:pass@localhost:3306/mydb
ENV=local

# Optional (with defaults)
USER_API_PORT=3333
RATE_LIMIT_PER_MINUTE=100
RATE_LIMIT_BURST=150
REQUEST_TIMEOUT_SECS=30
CORS_ALLOWED_ORIGINS=*
MAX_BODY_SIZE_BYTES=1048576
SHUTDOWN_TIMEOUT_SECS=30
IP_ALLOWLIST=                # comma-separated IPs (optional)
IP_BLOCKLIST=                # comma-separated IPs (optional)
```

### Graceful Shutdown

The service handles SIGTERM and SIGINT signals for graceful shutdown, allowing in-flight requests to complete before terminating.

## Testing

Run all tests with:

```bash
cargo test --workspace
```

### Test Suite Overview

| Package | Test Type | Count | Description |
|---------|-----------|-------|-------------|
| user-api | Unit | 7 | Middleware tests (IP filter, circuit breaker) |
| user-api | Handler | 45 | API endpoint handler tests with mocked services |
| user-api | OpenAPI | 2 | OpenAPI spec validation |
| user-lib | BDD | 19 scenarios | Cucumber feature tests (94 steps) |
| user-lib | Integration | 1 | Full service flow (requires Docker) |

### BDD Tests (Cucumber)

Located in `libs/user-lib/tests/features/`:

**User Management** (`user_management.feature`):
- Create, retrieve, update, delete users
- Duplicate email prevention
- Pagination
- Input validation (name required, email format)

**Role Management** (`role_management.feature`):
- Create and delete roles
- Assign/unassign roles to users
- Multiple roles per user
- Duplicate role name prevention
- Pagination

Run BDD tests only:

```bash
cargo test --package user-lib --test bdd
```

### Handler Tests

Located in `apps/user-api/tests/handler_tests.rs`:
- All CRUD operations for users and roles
- Error handling (validation, not found, conflicts)
- Pagination queries
- Response transformation

### Integration Tests

The integration test (`user_service_test.rs`) requires Docker to spin up a MySQL container:

```bash
docker compose -f docker-compose.local.yml up -d mysql
cargo test --package user-lib --test user_service_test
```

## Development Commands (just)

This project uses [just](https://github.com/casey/just) as a command runner.

```bash
just              # Show all available commands
just dev          # Start Docker + API with hot-reload (cargo watch)
just run          # Start Docker + API (no hot-reload)
just test         # Run all tests
just test-watch   # Run tests with hot-reload
just test-bdd     # Run BDD tests only
just up           # Start Docker containers
just down         # Stop Docker containers
just migrate      # Run database migrations
just logs         # Show Docker logs
just fmt          # Format code
just lint         # Run clippy
just build        # Build all packages
```

## Command line tool

$backcli is the command line to perform operations with services. For example, for running migrations databases:

```bash
DATABASE_URL=mysql://testuser:password@localhost:3306/testdb cargo run -p backcli -- --migrations --user-lib
```

