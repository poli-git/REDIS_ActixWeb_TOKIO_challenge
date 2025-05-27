# Use Rust 1.81.0 as a build environment
FROM rust:1.84.0 as builder

# Install required system dependencies for Diesel with PostgreSQL
RUN apt-get update && \
    apt-get install -y \
    libpq-dev \
    pkg-config \
    && rm -rf /var/lib/apt/lists/*


# Install Diesel CLI (for migration tools)
RUN cargo install diesel_cli --no-default-features --features postgres
    

# Create a new empty shell project
WORKDIR /app
RUN cargo init --bin

# Copy your Cargo.toml and Cargo.lock
COPY Cargo.toml Cargo.lock ./

# This is a dummy build to cache dependencies
RUN mkdir -p src && \
    echo "fn main() {}" > src/main.rs && \
    cargo build --release

# Copy your source code
COPY src ./src
COPY storage/migrations .storage/migrations
COPY .env ./

# Touch main.rs to prevent cached release build
RUN touch src/main.rs

# Build for release
RUN cargo build --release

# Use a minimal debian image for runtime
FROM debian:bookworm-slim

# Install runtime dependencies for PostgreSQL
RUN apt-get update && \
    apt-get install -y \
    libpq5 \
    && rm -rf /var/lib/apt/lists/*

# Copy the built binary from the builder stage
COPY --from=builder /usr/local/cargo/bin/diesel /usr/local/bin/
COPY --from=builder /app/target/release/fever_challenge /usr/local/bin/fever_challenge
COPY --from=builder /app/.env /app/.env

# Set the working directory
WORKDIR /app

# Run the application
CMD ["fever_challenge"]