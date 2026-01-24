# Default recipe - show available commands
default:
    @just --list

# Source env file helper
export-env := "set -a && source .env.local && set +a"

# Start local stack (Docker) and run API with hot-reload
dev: up wait-db migrate
    #!/usr/bin/env bash
    set -a && source .env.local && set +a
    # Expand DATABASE_URL variables
    export DATABASE_URL="mysql://${MYSQL_USER}:${MYSQL_PASSWORD}@127.0.0.1:${MYSQL_PORT}/${MYSQL_DATABASE}"
    cargo watch -x "run --bin user-api"

# Start local stack and run API (no hot-reload)
run: up wait-db migrate
    #!/usr/bin/env bash
    set -a && source .env.local && set +a
    export DATABASE_URL="mysql://${MYSQL_USER}:${MYSQL_PASSWORD}@127.0.0.1:${MYSQL_PORT}/${MYSQL_DATABASE}"
    cargo run --bin user-api

# Start Docker containers
up:
    docker compose -f compose/docker-compose.local.yml up -d --build

# Stop Docker containers
down:
    docker compose -f compose/docker-compose.local.yml down

# Stop and remove volumes
clean:
    docker compose -f compose/docker-compose.local.yml down -v

# Wait for MySQL to be ready
wait-db:
    #!/usr/bin/env bash
    echo "Waiting for MySQL on 127.0.0.1:${MYSQL_PORT:-3306}..."
    for i in {1..60}; do
        # Try mysqladmin if available, otherwise use docker exec
        if command -v mysqladmin &>/dev/null; then
            if mysqladmin ping -h 127.0.0.1 -P"${MYSQL_PORT:-3306}" -u"${MYSQL_USER:-user}" -p"${MYSQL_PASSWORD:-pass}" --silent 2>/dev/null; then
                echo "MySQL is up."
                echo "phpMyAdmin available at http://127.0.0.1:${PHPMYADMIN_PORT:-8082}/"
                exit 0
            fi
        else
            if docker exec compose-mysql-1 mysqladmin ping -u"${MYSQL_USER:-user}" -p"${MYSQL_PASSWORD:-pass}" --silent 2>/dev/null; then
                echo "MySQL is up."
                echo "phpMyAdmin available at http://127.0.0.1:${PHPMYADMIN_PORT:-8082}/"
                exit 0
            fi
        fi
        sleep 1
    done
    echo "ERROR: MySQL did not become ready in time." >&2
    exit 1

# Run database migrations
migrate:
    #!/usr/bin/env bash
    set -a && source .env.local && set +a
    export DATABASE_URL="mysql://${MYSQL_USER}:${MYSQL_PASSWORD}@127.0.0.1:${MYSQL_PORT}/${MYSQL_DATABASE}"
    cargo run --bin backcli -- --migrations --user-lib

# Run all tests
test:
    cargo test --workspace

# Run tests with output
test-verbose:
    cargo test --workspace -- --nocapture

# Run BDD tests only
test-bdd:
    cargo test --package user-lib --test bdd

# Build all packages
build:
    cargo build --workspace

# Build release
build-release:
    cargo build --workspace --release

# Check code without building
check:
    cargo check --workspace

# Format code
fmt:
    cargo fmt --all

# Lint with clippy
lint:
    cargo clippy --workspace -- -D warnings

# Watch tests (re-run on file changes)
test-watch:
    cargo watch -x "test --workspace"

# Show logs from Docker containers
logs:
    docker compose -f compose/docker-compose.local.yml logs -f

# Show API logs only
logs-api:
    docker compose -f compose/docker-compose.local.yml logs -f user-api
