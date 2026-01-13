# Build stage
FROM rust:slim-bookworm AS builder
WORKDIR /app

# Make Cargo network a bit more resilient in CI/spotty networks
ENV CARGO_HTTP_TIMEOUT=600
ENV CARGO_NET_RETRY=10

# Install build dependencies
RUN apt-get update && apt-get install -y pkg-config libssl-dev g++ make && rm -rf /var/lib/apt/lists/*

# Optimize Build Cache: Copy manifests only
COPY services/data-engine/Cargo.toml services/data-engine/Cargo.lock* ./services/data-engine/

# Switch to component directory
WORKDIR /app/services/data-engine

# Create dummy main.rs to cache dependencies
RUN mkdir src && echo "fn main() {}" > src/main.rs
# Create dummy benchmark files to satisfy Cargo.toml
RUN mkdir benches && \
    echo "fn main() {}" > benches/parser_benchmarks.rs && \
    echo "fn main() {}" > benches/storage_benchmarks.rs

# Build dependencies only
RUN cargo build --release

# Remove dummy files
RUN rm -rf src benches target/release/deps/data_engine*

# Copy actual source code AND Infrastructure (for migrations)
WORKDIR /app
COPY infrastructure ./infrastructure
COPY services/data-engine ./services/data-engine

# Build the application
WORKDIR /app/services/data-engine
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies including Chrome for headless browsing
RUN set -eux; \
    for i in 1 2 3 4 5; do \
      apt-get update -o Acquire::Retries=5 && \
      apt-get install -y --no-install-recommends -o Acquire::Retries=5 \
        ca-certificates \
        curl \
        chromium \
        chromium-driver \
      && rm -rf /var/lib/apt/lists/* \
      && break; \
      echo "apt-get failed (attempt $i/5), retrying..." >&2; \
      sleep 5; \
    done

# Set Chrome path for headless_chrome
ENV CHROME_PATH=/usr/bin/chromium

# Copy the binary from builder
COPY --from=builder /app/services/data-engine/target/release/data-engine /usr/local/bin/

# Copy config directory
COPY --from=builder /app/services/data-engine/config /app/config
WORKDIR /app

EXPOSE 8080
CMD ["data-engine"]
