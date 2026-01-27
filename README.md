# Backender

A production-ready backend platform built with Rust, featuring user management, role-based access control, Keycloak integration, Redis caching, and comprehensive observability.

## Features

- **User & Role Management**: Complete CRUD operations with role assignments
- **Keycloak Integration**: OAuth2/OIDC authentication and identity management
- **Redis Caching**: Cache-aside pattern with configurable TTLs
- **API Middleware**: Rate limiting, timeouts, CORS, request tracing, body size limits, IP filtering
- **Observability**: Logging (ELK), metrics (Prometheus/Grafana), tracing
- **BDD Testing**: Cucumber/Gherkin feature tests with 19 scenarios
- **API Documentation**: Auto-generated Swagger/OpenAPI docs
- **Graceful Shutdown**: Handles in-flight requests during shutdown
- **Database Migrations**: Version-controlled schema with sqlx
- **Feature Flags**: Unleash integration for feature toggles

## Prerequisites

### Required

- **Docker**: Docker Desktop or Colima for running containers
- **Rust**: Install from [rustup.rs](https://rustup.rs)
- **Just**: Command runner - `brew install just` (macOS) or [install guide](https://github.com/casey/just)

### Optional (Recommended)

- **mysql-client**: For database health checks - `brew install mysql-client` (macOS)
- **cargo-watch**: For hot-reload during development - `cargo install cargo-watch`

## Quick Setup

Bootstrap the entire environment with a single command:

```bash
# Complete setup: Docker + Keycloak + Migrations + Root User
just setup
```

This command will:
1. Install git hooks for code quality checks (fmt + lint)
2. Start all Docker containers (MySQL, Redis, Keycloak, etc.)
3. Wait for MySQL to be ready
4. Configure Keycloak service account and update `.env.local` with the client secret
5. Run database migrations
6. Create the root user in both Keycloak and database

After setup completes, start the API:

```bash
# Start API with hot-reload
just dev

# Or without hot-reload
just run
```

The API will be available at `http://localhost:3333`

## Manual Setup

If you prefer step-by-step setup or need to run individual commands:

### 1. Start Docker Containers

```bash
just up
```

### 2. Setup Keycloak Service Account

```bash
just setup-keycloak
```

This creates a service account in Keycloak and automatically updates `KEYCLOAK_CLIENT_SECRET` in `.env.local`.

### 3. Run Database Migrations

```bash
just migrate
```

### 4. Initialize Root User

```bash
just init-root
```

Creates the root user (configured in `.env.local`) in both Keycloak and the database with admin role.

Default credentials (from `.env.local`):
- Email: `root@mail.com`
- Password: `root`

### 5. Start the API

```bash
just dev    # with hot-reload
just run    # without hot-reload
```

## Included Services

| Service | Purpose | Local URL |
|---------|---------|-----------|
| **MySQL** | Primary database | `localhost:3306` |
| **phpMyAdmin** | Database admin UI | `http://localhost:8082` |
| **Redis** | Caching and streams | `localhost:6379` |
| **Keycloak** | Identity & access management | `http://localhost:18080` |
| **Unleash** | Feature toggles | `http://unleash.127.0.0.1.nip.io` |
| **ElasticSearch + Kibana** | Logging and search | `http://kibana.127.0.0.1.nip.io` |
| **Prometheus + Grafana** | Metrics and monitoring | `http://grafana.127.0.0.1.nip.io` |
| **Postgres** | Database for supporting services | Internal |

### Service URLs

- **User API**: http://localhost:3333
- **phpMyAdmin**: http://localhost:8082
- **Keycloak**: http://localhost:18080
- **Unleash**: http://unleash.127.0.0.1.nip.io
- **Grafana**: http://grafana.127.0.0.1.nip.io
- **Prometheus**: http://prometheus.127.0.0.1.nip.io
- **Kibana**: http://kibana.127.0.0.1.nip.io
- **Traefik Dashboard**: http://traefik.127.0.0.1.nip.io


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

### Configuration

All configuration is managed through `.env.local`. The file is pre-configured with sensible defaults.

#### Core Settings

| Variable | Default | Description |
|----------|---------|-------------|
| `DATABASE_URL` | `mysql://user:pass@127.0.0.1:3306/mydb` | MySQL connection string |
| `ENV` | `local` | Environment name |
| `USER_API_PORT` | `3333` | API server port |
| `RUST_LOG` | `debug` | Logging level |

#### Keycloak Settings

| Variable | Default | Description |
|----------|---------|-------------|
| `KEYCLOAK_URL` | `http://localhost:18080` | Keycloak server URL |
| `KEYCLOAK_REALM` | `master` | Keycloak realm |
| `KEYCLOAK_CLIENT_ID` | `user-api-service` | Service account client ID |
| `KEYCLOAK_CLIENT_SECRET` | Auto-generated by `just setup-keycloak` | Client secret |

#### Root User Settings

| Variable | Default | Description |
|----------|---------|-------------|
| `ROOT_USER_EMAIL` | `root@mail.com` | Root user email |
| `ROOT_USER_PASSWORD` | `root` | Root user password |
| `ROOT_USER_FIRST_NAME` | `Root` | First name |
| `ROOT_USER_LAST_NAME` | `User` | Last name |

#### API Middleware

| Variable | Default | Description |
|----------|---------|-------------|
| `RATE_LIMIT_PER_MINUTE` | `100` | Rate limit per IP |
| `RATE_LIMIT_BURST` | `150` | Burst capacity |
| `REQUEST_TIMEOUT_SECS` | `30` | Request timeout |
| `CORS_ALLOWED_ORIGINS` | `*` | CORS origins |
| `MAX_BODY_SIZE_BYTES` | `1048576` | Max request body (1MB) |
| `IP_ALLOWLIST` | Empty | Comma-separated allowed IPs |
| `IP_BLOCKLIST` | Empty | Comma-separated blocked IPs |

#### Cache Settings

| Variable | Default | Description |
|----------|---------|-------------|
| `CACHE_ENABLED` | `true` | Enable Redis caching |
| `CACHE_USER_TTL_SECS` | `300` | User cache TTL (5 min) |
| `CACHE_ROLE_TTL_SECS` | `600` | Role cache TTL (10 min) |
| `CACHE_LIST_TTL_SECS` | `60` | List cache TTL (1 min) |

### Graceful Shutdown

The service handles SIGTERM and SIGINT signals for graceful shutdown, allowing in-flight requests to complete before terminating.

## Testing

The project includes comprehensive test coverage across multiple layers.

### Quick Test Commands

```bash
just test              # Run all tests (unit + handler + BDD)
just test-watch        # Run tests with hot-reload
just test-bdd          # Run BDD/Cucumber tests only
just test-integration  # Run integration tests (requires Docker)
```

### Test Suite Overview

| Package | Test Type | Count | Coverage |
|---------|-----------|-------|----------|
| **user-api** | Unit | 7 | Middleware (IP filter, circuit breaker) |
| **user-api** | Handler | 45 | All API endpoints with mocked services |
| **user-api** | OpenAPI | 2 | Swagger spec validation |
| **user-lib** | BDD | 19 scenarios | Business logic (94 Cucumber steps) |
| **user-lib** | Integration | 1 | Full service flow with real database |

### BDD Tests (Cucumber/Gherkin)

Behavior-driven tests in `libs/user-lib/tests/features/`:

#### User Management (`user_management.feature`)
- Create, retrieve, update, delete users
- Duplicate email prevention
- Pagination with page/page_size
- Input validation (required name, email format)
- Case-insensitive email matching

#### Role Management (`role_management.feature`)
- Create and delete roles
- Assign/unassign roles to users
- Multiple roles per user
- Duplicate role name prevention
- Role-based permissions

Run BDD tests:
```bash
just test-bdd
# Or directly:
cargo test --package user-lib --test bdd
```

### Handler Tests

Located in `apps/user-api/tests/handler_tests.rs`:
- CRUD operations for users and roles
- Error handling (400 Bad Request, 404 Not Found, 409 Conflict)
- Query parameter validation (pagination)
- Response transformation (DTOs)
- Mock service layer

### Integration Tests

Full end-to-end test with real MySQL container:
```bash
just test-integration
# Or directly:
cargo test --package user-lib --test user_service_test
```

## Development Commands

This project uses [just](https://github.com/casey/just) as a command runner. Run `just` without arguments to see all available commands.

### Setup & Bootstrap

| Command | Description |
|---------|-------------|
| `just setup` | Complete setup: git hooks + Docker + Keycloak + migrations + root user |
| `just install-hooks` | Install git hooks (runs automatically with `just setup`) |
| `just setup-keycloak` | Configure Keycloak service account and update `.env.local` |
| `just migrate` | Run database migrations |
| `just init-root` | Create root user in Keycloak and database |

### Running the Application

| Command | Description |
|---------|-------------|
| `just dev` | Start Docker + API with hot-reload (recommended for development) |
| `just run` | Start Docker + API without hot-reload |

### Docker Management

| Command | Description |
|---------|-------------|
| `just up` | Start all Docker containers |
| `just down` | Stop all Docker containers |
| `just clean` | Stop containers and remove volumes |
| `just logs` | Show logs from all containers |
| `just logs-api` | Show logs from API only |

### Testing

| Command | Description |
|---------|-------------|
| `just test` | Run all tests (unit + handler + BDD) |
| `just test-watch` | Run tests with hot-reload on file changes |
| `just test-bdd` | Run BDD/Cucumber tests only |
| `just test-integration` | Run integration tests (requires Docker) |
| `just test-verbose` | Run tests with output |

### Code Quality

| Command | Description |
|---------|-------------|
| `just fmt` | Format code with rustfmt |
| `just lint` | Run clippy linter |
| `just check` | Check code without building |
| `just build` | Build all packages (debug) |
| `just build-release` | Build optimized release binaries |

## CLI Tool (backcli)

The `backcli` tool provides administrative commands for managing the platform.

### Usage Examples

```bash
# Run migrations for user-lib
cargo run -p backcli -- --migrations --user-lib

# Setup Keycloak service account
cargo run -p backcli -- --setup-keycloak

# Initialize root user
cargo run -p backcli -- --init-root

# Revert migrations (cleanup)
cargo run -p backcli -- --delete --user-lib
```

All commands respect the `DATABASE_URL` environment variable from `.env.local`.

## Project Structure

```
backender/
├── apps/
│   ├── user-api/        # REST API service (Axum)
│   └── backcli/         # CLI tool for admin operations
├── libs/
│   └── user-lib/        # Core business logic, repositories, domain models
├── compose/             # Docker Compose configurations
└── justfile            # Command runner recipes
```

### Architecture

- **API Layer** (`user-api`): Axum-based REST API with middleware stack
- **Business Logic** (`user-lib`): Domain models, repositories, service layer
- **CLI Tool** (`backcli`): Database migrations, Keycloak setup, user initialization
- **External Services**: Keycloak (auth), Redis (cache), MySQL (persistence)

## Common Workflows

### Starting Fresh

```bash
# Complete teardown and fresh start
just clean          # Remove all containers and volumes
just setup          # Bootstrap everything from scratch
just dev            # Start developing
```

### Adding a New Feature

```bash
just dev            # Start with hot-reload
# Make your changes...
just test           # Run tests
just fmt            # Format code
just lint           # Check for issues
```

### Resetting the Database

```bash
cargo run -p backcli -- --delete --user-lib   # Revert migrations
just migrate                                   # Re-apply migrations
just init-root                                 # Recreate root user
```

## After Setup

Once setup is complete, you can:

1. **Access the API**: http://localhost:3333
   - Swagger documentation: http://localhost:3333/docs
   - Health check: http://localhost:3333/health

2. **Login with root user**:
   - Email: `root@mail.com` (configurable in `.env.local`)
   - Password: `root` (configurable in `.env.local`)

3. **Manage the database**:
   - phpMyAdmin: http://localhost:8082
   - User: `user`, Password: `pass` (from `.env.local`)

4. **Access Keycloak admin**:
   - Keycloak console: http://localhost:18080
   - Default admin credentials are configured in Docker Compose

## Troubleshooting

### Docker Connection Issues

If you're using **Colima**, uncomment this line in `.env.local`:
```bash
DOCKER_HOST=unix://${HOME}/.colima/default/docker.sock
```

For **Docker Desktop**, the default configuration should work.

### MySQL Connection Failed

Ensure MySQL is fully started before running migrations:
```bash
just up
just wait-db  # This happens automatically in 'just setup'
just migrate
```

### Keycloak Setup Failed

If `just setup-keycloak` fails:
1. Ensure Keycloak is running: `docker ps | grep keycloak`
2. Check Keycloak logs: `just logs | grep keycloak`
3. Wait a few more seconds for Keycloak to fully initialize
4. Retry: `just setup-keycloak`

### Port Already in Use

If ports 3306, 3333, or 18080 are in use:
1. Stop conflicting services
2. Or update port numbers in `.env.local`
3. Restart with `just down && just up`

### Cache Issues

To clear Redis cache:
```bash
docker exec -it compose-redis-1 redis-cli FLUSHALL
```

## Git Hooks

The project includes pre-commit hooks to maintain code quality. Hooks are automatically installed when you run `just setup`.

### Manual Installation

If you cloned the repo without running setup:

```bash
just install-hooks
```

### Pre-Commit Hook

Runs automatically before every commit:

1. **Format Check**: `cargo fmt --all --check` - Ensures code is properly formatted
2. **Linting**: `cargo clippy --workspace -- -D warnings` - Catches common issues and enforces best practices

If the hook fails:
```bash
just fmt    # Fix formatting
just lint   # Check and fix linting issues
```

### Bypassing Hooks

Only in exceptional cases (not recommended):
```bash
git commit --no-verify -m "message"
```

## Development Best Practices

### Code Style

- Follow Rust conventions and idioms
- Git hooks automatically check formatting and linting before commits
- All tests must pass: `just test`
- Address all clippy warnings before committing

### Adding New Features

1. Write BDD tests first in `libs/user-lib/tests/features/`
2. Implement business logic in `user-lib`
3. Add handler tests in `user-api/tests/`
4. Implement API endpoints
5. Update OpenAPI documentation
6. Run full test suite

### Database Changes

1. Create migration files in `libs/user-lib/migrations/`
2. Name format: `XXXX_description.sql` and `XXXX_description.down.sql`
3. Test migrations: `just migrate`
4. Test rollback: `cargo run -p backcli -- --delete --user-lib`

### Commit Messages

Follow conventional commits:
- `feat: add user export endpoint`
- `fix: resolve cache invalidation issue`
- `docs: update setup instructions`
- `test: add integration tests for roles`

## Quick Reference

### Essential Commands

```bash
# First time setup
just setup                                        # Complete bootstrap (includes git hooks)
just install-hooks                                # Install git hooks only

# Daily development
just dev                                          # Start with hot-reload

# Code quality (automatically run by git hooks)
just fmt                                          # Format code
just lint                                         # Run linter
just test                                         # Run tests

# Database operations
just migrate                                      # Run migrations
cargo run -p backcli -- --delete --user-lib      # Revert migrations
just init-root                                    # Create root user

# Docker management
just up                                           # Start services
just down                                         # Stop services
just clean                                        # Remove everything
just logs                                         # View logs
```

### Default Credentials

| Service | URL | Username | Password |
|---------|-----|----------|----------|
| **User API** | http://localhost:3333 | N/A | N/A |
| **Root User** | - | `root@mail.com` | `root` |
| **phpMyAdmin** | http://localhost:8082 | `user` | `pass` |
| **MySQL** | localhost:3306 | `user` | `pass` |
| **Redis** | localhost:6379 | - | - |
| **Keycloak** | http://localhost:18080 | See Docker Compose | - |

### Key Files

- `.env.local` - Environment configuration
- `justfile` - Command definitions
- `compose/docker-compose.local.yml` - Docker services
- `apps/user-api/src/main.rs` - API entrypoint
- `apps/backcli/src/main.rs` - CLI tool
- `libs/user-lib/src/` - Core business logic

---

Built with Rust, Axum, SQLx, Keycloak, Redis, and lots of ☕

