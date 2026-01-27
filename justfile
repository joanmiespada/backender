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

# Wait for Infisical to be ready
wait-infisical:
    #!/usr/bin/env bash
    echo "Waiting for Infisical on 127.0.0.1:${INFISICAL_PORT:-8888}..."
    for i in {1..60}; do
        if curl -fsS "http://127.0.0.1:${INFISICAL_PORT:-8888}/api/status" >/dev/null 2>&1; then
            echo "Infisical is up."
            echo "Infisical UI available at http://127.0.0.1:${INFISICAL_PORT:-8888}/"
            exit 0
        fi
        sleep 2
    done
    echo "ERROR: Infisical did not become ready in time." >&2
    exit 1

# Run database migrations
migrate:
    #!/usr/bin/env bash
    set -a && source .env.local && set +a
    export DATABASE_URL="mysql://${MYSQL_USER}:${MYSQL_PASSWORD}@127.0.0.1:${MYSQL_PORT}/${MYSQL_DATABASE}"
    cargo run --bin backcli -- --migrations --user-lib

# Setup Keycloak service account and update .env.local with client secret
# Also stores the secret in Infisical if configured
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

# Store Keycloak secret in Infisical (run after setup-keycloak and setup-infisical)
sync-keycloak-secret:
    #!/usr/bin/env bash
    set -a && source .env.local && set +a

    # Check if Infisical is configured
    if [ -z "$INFISICAL_CLIENT_ID" ] || [ -z "$INFISICAL_CLIENT_SECRET" ]; then
        echo "Skipping Infisical sync - Infisical not configured"
        exit 0
    fi

    # Check if Keycloak secret exists
    if [ -z "$KEYCLOAK_CLIENT_SECRET" ]; then
        echo "ERROR: KEYCLOAK_CLIENT_SECRET not set in .env.local"
        exit 1
    fi

    echo "Storing KEYCLOAK_CLIENT_SECRET in Infisical..."
    cargo run --bin backcli -- --store-secret --key KEYCLOAK_CLIENT_SECRET --value "$KEYCLOAK_CLIENT_SECRET"

    if [ $? -eq 0 ]; then
        echo "✓ KEYCLOAK_CLIENT_SECRET stored in Infisical"
        echo ""
        echo "The user-api will now retrieve KEYCLOAK_CLIENT_SECRET from Infisical."
        echo "You can remove it from .env.local if desired (Infisical is the source of truth)."
    else
        echo "WARNING: Failed to store secret in Infisical"
        echo "The application will fall back to the .env.local value"
    fi

# Setup Infisical secrets manager and update .env.local with credentials
setup-infisical:
    #!/usr/bin/env bash
    set -a && source .env.local && set +a

    # Run setup and capture output
    OUTPUT=$(cargo run --bin backcli -- --setup-infisical 2>&1)

    # Check if setup was successful
    if [ $? -ne 0 ]; then
        echo "ERROR: Infisical setup failed"
        echo "$OUTPUT"
        exit 1
    fi

    # Display the output
    echo "$OUTPUT"

    # Extract credentials from output
    INFISICAL_URL_VAL=$(echo "$OUTPUT" | grep "^INFISICAL_URL=" | cut -d'=' -f2)
    CLIENT_ID=$(echo "$OUTPUT" | grep "^INFISICAL_CLIENT_ID=" | cut -d'=' -f2)
    CLIENT_SECRET=$(echo "$OUTPUT" | grep "^INFISICAL_CLIENT_SECRET=" | cut -d'=' -f2)
    PROJECT_ID=$(echo "$OUTPUT" | grep "^INFISICAL_PROJECT_ID=" | cut -d'=' -f2)
    ENVIRONMENT=$(echo "$OUTPUT" | grep "^INFISICAL_ENVIRONMENT=" | cut -d'=' -f2)

    if [ -z "$CLIENT_ID" ] || [ -z "$CLIENT_SECRET" ] || [ -z "$PROJECT_ID" ]; then
        echo "ERROR: Could not extract credentials from output"
        exit 1
    fi

    # Update .env.local file - add or update each variable
    update_env_var() {
        local key=$1
        local value=$2
        if grep -q "^${key}=" .env.local 2>/dev/null; then
            if [[ "$OSTYPE" == "darwin"* ]]; then
                sed -i '' "s|^${key}=.*|${key}=${value}|" .env.local
            else
                sed -i "s|^${key}=.*|${key}=${value}|" .env.local
            fi
        else
            echo "${key}=${value}" >> .env.local
        fi
    }

    update_env_var "INFISICAL_URL" "$INFISICAL_URL_VAL"
    update_env_var "INFISICAL_CLIENT_ID" "$CLIENT_ID"
    update_env_var "INFISICAL_CLIENT_SECRET" "$CLIENT_SECRET"
    update_env_var "INFISICAL_PROJECT_ID" "$PROJECT_ID"
    update_env_var "INFISICAL_ENVIRONMENT" "$ENVIRONMENT"

    echo ""
    echo "✓ Updated Infisical credentials in .env.local"

# Initialize root user in Keycloak and database
init-root:
    #!/usr/bin/env bash
    set -a && source .env.local && set +a
    export DATABASE_URL="mysql://${MYSQL_USER}:${MYSQL_PASSWORD}@127.0.0.1:${MYSQL_PORT}/${MYSQL_DATABASE}"
    cargo run --bin backcli -- --init-root

# Complete setup: Keycloak + Infisical + migrations + root user + git hooks
# Order: infisical first (to get credentials), then keycloak, then sync keycloak secret to infisical
setup: install-hooks up wait-db wait-infisical setup-infisical setup-keycloak sync-keycloak-secret migrate init-root
    @echo ""
    @echo "✓ Complete setup finished!"
    @echo "  - Git hooks installed"
    @echo "  - Infisical secrets manager configured"
    @echo "  - Keycloak service account configured"
    @echo "  - Keycloak secret stored in Infisical"
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

# Run secrets integration tests (requires running Infisical from docker-compose)
test-integration-secrets:
    #!/usr/bin/env bash
    set -a && source .env.local && set +a
    cargo test --package secrets --test infisical_integration_test -- --nocapture

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
