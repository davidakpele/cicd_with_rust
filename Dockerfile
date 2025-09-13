FROM rust:latest

WORKDIR /app

COPY . .

# Install sqlx-cli with only postgres feature
RUN cargo install sqlx-cli --no-default-features --features postgres

# Build the application
RUN cargo build --release

EXPOSE 8055

# Run migrations before starting the app
CMD sqlx migrate run && ./target/release/ci-cd-pipeline