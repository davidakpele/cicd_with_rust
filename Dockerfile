# ------------------------
# Stage 1: Builder
# ------------------------
FROM rust:slim-bullseye AS builder

WORKDIR /app

# Install build dependencies
RUN apt-get update && apt-get install -y --no-install-recommends \
    curl \
    wget \
    git \
    build-essential \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Install Node.js 20.x from NodeSource
RUN curl -fsSL https://deb.nodesource.com/setup_20.x | bash - \
    && apt-get install -y nodejs

# Verify Node.js version
RUN node --version && npm --version

# Install sqlx-cli for database migrations
RUN cargo install sqlx-cli --no-default-features --features postgres --locked

# Copy project files including migrations
COPY . .

# Build Rust application
RUN cargo build --release

# ------------------------
# Stage 2: Production
# ------------------------
FROM debian:bookworm-slim AS production

WORKDIR /app

# Install runtime dependencies - ADD GIT HERE!
RUN apt-get update && apt-get install -y --no-install-recommends \
    git \ 
    curl \
    libssl3 \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Install Node.js 20.x in production stage
RUN curl -fsSL https://deb.nodesource.com/setup_20.x | bash - \
    && apt-get install -y nodejs

# Verify Node.js version
RUN node --version && npm --version

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