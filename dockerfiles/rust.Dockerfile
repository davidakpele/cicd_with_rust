# Stage 1: Build the Rust binary
FROM rust:1.70.0 AS builder

WORKDIR /usr/src/app

# Copy the source code
COPY . .

# Build the release binary
RUN cargo install --path .

# Stage 2: Create the final lightweight image
FROM alpine:3.18

# Install necessary libraries for the Rust binary (e.g., OpenSSL)
RUN apk add --no-cache libssl3

WORKDIR /usr/bin

# Copy the compiled binary from the builder stage
COPY --from=builder /usr/local/cargo/bin/ci-cd-pipeline .

# Expose the application port
EXPOSE 8022

# Run the application
CMD ["./ci-cd-pipeline"]