# Build stage
FROM rust:1.83-slim as builder

WORKDIR /app

# Install dependencies
RUN apt-get update && \
    apt-get install -y pkg-config libssl-dev && \
    rm -rf /var/lib/apt/lists/*

# Copy manifests
COPY Cargo.toml ./
COPY sync-service/Cargo.toml ./sync-service/

# Copy source code
COPY sync-service/src ./sync-service/src
COPY sync-service/migrations ./sync-service/migrations

# Build the application
RUN cargo build --release -p sync-service

# Runtime stage
FROM debian:bookworm-slim

WORKDIR /app

# Install runtime dependencies
RUN apt-get update && \
    apt-get install -y ca-certificates libssl3 && \
    rm -rf /var/lib/apt/lists/*

# Copy binary from builder
COPY --from=builder /app/target/release/sync-service /app/sync-service

# Copy migrations
COPY sync-service/migrations /app/migrations

# Create directories
RUN mkdir -p /app/config /app/cache

# Expose port
EXPOSE 8080

# Run the application
CMD ["/app/sync-service"]
