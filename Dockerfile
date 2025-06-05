# Use Rust as build environment
FROM rust:1.84.0 as builder

# Install required system dependencies for Diesel with PostgreSQL
RUN apt-get update && \
    apt-get install -y \
    libpq-dev \
    pkg-config \
    && rm -rf /var/lib/apt/lists/*

# Install Diesel CLI (for migration tools)
RUN cargo install diesel_cli --no-default-features --features postgres

WORKDIR /app

# Copy manifests
COPY Cargo.toml Cargo.lock ./

# Dummy build to cache dependencies
RUN mkdir -p src && \
    echo "fn main() {}" > src/main.rs && \
    cargo build --release

# Copy source code and migrations
COPY src ./src
COPY src/storage/migrations ./src/storage/migrations
COPY .env ./

# Run Diesel migrations
RUN diesel migration run

# Build async_worker and webapp binaries
RUN cargo build --release --bin async_worker --bin webapp

# Use a minimal debian image for runtime
FROM debian:bookworm-slim

# Install runtime dependencies for PostgreSQL
RUN apt-get update && \
    apt-get install -y \
    libpq5 \
    && rm -rf /var/lib/apt/lists/*

# Copy the built binaries from the builder stage
COPY --from=builder /app/target/release/async_worker /usr/local/bin/async_worker
COPY --from=builder /app/target/release/webapp /usr/local/bin/webapp
COPY --from=builder /app/.env /app/.env

WORKDIR /app

# Expose port for webapp
EXPOSE 8080

# Use docker-compose or command override to select which binary to run.
# By default, run webapp (listening on port 8080)
CMD ["webapp"]