# Build stage
FROM rust:slim-bookworm AS builder
WORKDIR /app

# Make Cargo network a bit more resilient in CI/spotty networks
ENV CARGO_HTTP_TIMEOUT=600
ENV CARGO_NET_RETRY=10

# Install build dependencies
RUN apt-get update && apt-get install -y pkg-config libssl-dev g++ make && rm -rf /var/lib/apt/lists/*

# Optimize Build Cache: Copy manifests only
COPY services/gateway/Cargo.toml services/gateway/Cargo.lock* ./services/gateway/

# Switch to component directory
WORKDIR /app/services/gateway

# Create dummy main.rs to cache dependencies
RUN mkdir src && echo "fn main() {}" > src/main.rs

# Build dependencies only
RUN cargo build --release

# Remove dummy files
RUN rm -rf src target/release/deps/gateway*

# Copy actual source code AND Infrastructure
WORKDIR /app
COPY infrastructure ./infrastructure
COPY services/gateway ./services/gateway

# Build the application
WORKDIR /app/services/gateway
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Copy the binary from builder
COPY --from=builder /app/services/gateway/target/release/gateway /usr/local/bin/

WORKDIR /app
EXPOSE 8080
CMD ["gateway"]
