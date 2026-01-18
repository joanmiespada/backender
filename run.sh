#!/usr/bin/env bash
set -euo pipefail

# Run from repo root regardless of where the script is invoked from
ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$ROOT_DIR"


COMPOSE_FILE="./compose/docker-compose.local.yml"

ENV_FILE="./.env.local"

# Load local environment variables (MYSQL_*, etc.) if present
if [[ -f "$ENV_FILE" ]]; then
  # Export all variables defined in the env file
  set -a
  # shellcheck disable=SC1090
  source "$ENV_FILE"
  set +a
fi

# Prefer Docker Compose v2 (`docker compose`), but fall back to legacy `docker-compose` if needed
if command -v docker >/dev/null 2>&1 && docker compose version >/dev/null 2>&1; then
  DC=(docker compose)
elif command -v docker-compose >/dev/null 2>&1; then
  DC=(docker-compose)
else
  echo "ERROR: Docker Compose not found. Install Docker Desktop (includes 'docker compose') or docker-compose." >&2
  exit 1
fi

# Bring the local stack up in the background
if [[ -f "$ENV_FILE" ]]; then
  "${DC[@]}" --env-file "$ENV_FILE" -f "$COMPOSE_FILE" up -d --build
else
  "${DC[@]}" -f "$COMPOSE_FILE" up -d --build
fi

# Optional: wait for MySQL to be reachable on localhost (requires mysqladmin installed)
# If you don't have mysql client tools installed, this step is skipped.
if command -v mysqladmin >/dev/null 2>&1; then
  echo "Waiting for MySQL on 127.0.0.1..."
  for i in {1..60}; do
    if mysqladmin ping -h 127.0.0.1 -P"${MYSQL_PORT:-3306}" -u"${MYSQL_USER:-user}" -p"${MYSQL_PASSWORD:-pass}" --silent >/dev/null 2>&1; then
      echo "MySQL is up."
      echo "phpMyAdmin available at http://127.0.0.1:${PHPMYADMIN_PORT:-8082}/"
      break
    fi
    sleep 1
    if [[ $i -eq 60 ]]; then
      echo "ERROR: MySQL did not become ready in time." >&2
      exit 1
    fi
  done
else
  echo "mysqladmin not found; skipping MySQL readiness check."
fi

# Run DB migrations before starting the API.
# Safe to run repeatedly: sqlx migrations are tracked and only unapplied ones run.
echo "Running database migrations (backcli)..."
cargo run --bin backcli -- --migrations --user-lib

# Run the Rust API locally (talking to MySQL via the published port)
#cargo run --bin user-api
cargo watch -x "run --bin user-api" 
