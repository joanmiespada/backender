# -------- Builder --------
FROM rust:1.92-alpine AS builder

RUN apk add --no-cache \
    musl-dev \
    build-base \
    pkgconf \
    openssl-dev \
    curl

WORKDIR /workspace

# Copy workspace manifests first (better cache) — include ALL workspace members
COPY Cargo.toml Cargo.lock ./
COPY apps/user-api/Cargo.toml apps/user-api/Cargo.toml
COPY apps/backcli/Cargo.toml apps/backcli/Cargo.toml
COPY libs/user-lib/Cargo.toml libs/user-lib/Cargo.toml

# Dummy build to cache deps
# Cargo loads workspace members during dependency resolution; each member must have at least one target file.
RUN mkdir -p apps/user-api/src \
 && echo "fn main() {}" > apps/user-api/src/main.rs \
 && mkdir -p apps/backcli/src \
 && echo "fn main() {}" > apps/backcli/src/main.rs \
 && mkdir -p libs/user-lib/src \
 && echo "pub fn _dummy() {}" > libs/user-lib/src/lib.rs
RUN cargo build -p user-api --release
RUN rm -rf apps/user-api/src apps/backcli/src libs/user-lib/src

# Copy real sources
COPY apps ./apps
COPY libs ./libs

# Build real binary
RUN cargo build -p user-api --release

# -------- Runtime --------
FROM alpine:3.19

RUN apk add --no-cache ca-certificates

WORKDIR /app

COPY --from=builder /workspace/target/release/user-api /app/user-api

RUN addgroup -S app && adduser -S app -G app
USER app

EXPOSE 3333
CMD ["./user-api"]