# Default recipe - show available commands
default:
    @just --list

# Source env file helper
export-env := "set -a && source .env.local && set +a"

# Install git hooks (run once after cloning the repo)
install-hooks:
    #!/usr/bin/env bash
    echo "Installing git hooks..."
    # Create symlink for pre-commit hook
    ln -sf ../../hooks/pre-commit .git/hooks/pre-commit
    chmod +x .git/hooks/pre-commit
    echo "✓ Git hooks installed successfully!"
    echo ""
    echo "Pre-commit hook will now run:"
    echo "  - cargo fmt --all --check"
    echo "  - cargo clippy --workspace -- -D warnings"
    echo ""
    echo "To bypass hooks (not recommended): git commit --no-verify"

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

# Setup Keycloak service account and update .env.local with client secret
setup-keycloak:
    #!/usr/bin/env bash
    set -a && source .env.local && set +a

    # Run setup and capture output
    OUTPUT=$(cargo run --bin backcli -- --setup-keycloak 2>&1)

    # Check if setup was successful
    if [ $? -ne 0 ]; then
        echo "ERROR: Keycloak setup failed"
        echo "$OUTPUT"
        exit 1
    fi

    # Display the output
    echo "$OUTPUT"

    # Extract the secret from output
    SECRET=$(echo "$OUTPUT" | grep "KEYCLOAK_CLIENT_SECRET=" | tail -1 | cut -d'=' -f2)

    if [ -z "$SECRET" ]; then
        echo "ERROR: Could not extract client secret from output"
        exit 1
    fi

    # Update .env.local file
    if grep -q "^KEYCLOAK_CLIENT_SECRET=" .env.local; then
        # Replace existing secret
        if [[ "$OSTYPE" == "darwin"* ]]; then
            # macOS
            sed -i '' "s/^KEYCLOAK_CLIENT_SECRET=.*/KEYCLOAK_CLIENT_SECRET=$SECRET/" .env.local
        else
            # Linux
            sed -i "s/^KEYCLOAK_CLIENT_SECRET=.*/KEYCLOAK_CLIENT_SECRET=$SECRET/" .env.local
        fi
        echo ""
        echo "✓ Updated KEYCLOAK_CLIENT_SECRET in .env.local"
    else
        echo "ERROR: KEYCLOAK_CLIENT_SECRET not found in .env.local"
        exit 1
    fi

# Initialize root user in Keycloak and database
init-root:
    #!/usr/bin/env bash
    set -a && source .env.local && set +a
    export DATABASE_URL="mysql://${MYSQL_USER}:${MYSQL_PASSWORD}@127.0.0.1:${MYSQL_PORT}/${MYSQL_DATABASE}"
    cargo run --bin backcli -- --init-root

# Complete setup: Keycloak + migrations + root user + git hooks
setup: install-hooks up wait-db setup-keycloak migrate init-root
    @echo ""
    @echo "✓ Complete setup finished!"
    @echo "  - Git hooks installed"
    @echo "  - Keycloak service account configured"
    @echo "  - Database migrations applied"
    @echo "  - Root user initialized"
    @echo ""
    @echo "You can now start the API with: just run"

# Run all tests (excludes integration tests that need Docker)
test:
    cargo test --workspace --exclude user-lib
    cargo test --package user-lib --test bdd

# Run tests with output
test-verbose:
    cargo test --workspace -- --nocapture

# Run BDD tests only
test-bdd:
    cargo test --package user-lib --test bdd

# Run integration tests (requires Docker)
test-integration:
    #!/usr/bin/env bash
    set -a && source .env.local && set +a
    # Auto-detect Docker socket: Colima > Docker Desktop > default
    if [[ -z "$DOCKER_HOST" ]]; then
        if [[ -S "$HOME/.colima/default/docker.sock" ]]; then
            export DOCKER_HOST="unix://$HOME/.colima/default/docker.sock"
        elif [[ -S "$HOME/.docker/run/docker.sock" ]]; then
            export DOCKER_HOST="unix://$HOME/.docker/run/docker.sock"
        fi
    fi
    cargo test --package user-lib --test user_service_test -- --nocapture

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
