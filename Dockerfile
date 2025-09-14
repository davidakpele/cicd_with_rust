# ------------------------
# Stage 1: Builder
# ------------------------
FROM rust:slim-bullseye AS builder

WORKDIR /app

# Install build tools and Node.js
RUN apt-get update && apt-get install -y --no-install-recommends \
    curl \
    wget \
    git \
    build-essential \
    pkg-config \
    libssl-dev \
    nodejs npm \
    && rm -rf /var/lib/apt/lists/*

# Install sqlx-cli for database migrations
RUN cargo install sqlx-cli --no-default-features --features postgres --locked

# Copy project files including migrations
COPY . .

# Build Rust application
RUN cargo build --release

# Optional: build React project if package.json exists
RUN if [ -f package.json ] && grep -q "\"react\"" package.json; then \
        npm install && npm run build; \
    fi

# ------------------------
# Stage 2: Production
# ------------------------
FROM debian:bookworm-slim AS production

WORKDIR /app

# Install minimal runtimes
RUN apt-get update && apt-get install -y --no-install-recommends \
    nodejs npm \
    libssl3 \
    curl \
    wget \
    git \
    && rm -rf /var/lib/apt/lists/*

# Copy Rust binary from builder
COPY --from=builder /app/target/release/ci-cd-pipeline ./ci-cd-pipeline

# Copy sqlx binary for runtime migrations
COPY --from=builder /usr/local/cargo/bin/sqlx /usr/local/bin/sqlx

# Copy migrations folder
COPY --from=builder /app/migrations ./migrations

# Ensure deployments folder exists
RUN mkdir -p /app/deployments

# Expose only Rust service port
EXPOSE 8022

# Default command: run migrations and start Rust app
CMD ["sh", "-c", "sqlx migrate run --source ./migrations && ./ci-cd-pipeline"]
