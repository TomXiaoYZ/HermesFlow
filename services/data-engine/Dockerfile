# Build stage
FROM rust:slim-bookworm AS builder
WORKDIR /app

# Make Cargo network a bit more resilient in CI/spotty networks
ENV CARGO_HTTP_TIMEOUT=600
ENV CARGO_NET_RETRY=10

# Install build dependencies
RUN apt-get update && apt-get install -y pkg-config libssl-dev g++ make && rm -rf /var/lib/apt/lists/*

# Copy workspace root manifest, lockfile, and all member Cargo.toml files
COPY Cargo.toml Cargo.lock ./
COPY services/common/Cargo.toml ./services/common/Cargo.toml
COPY services/backtest-engine/Cargo.toml ./services/backtest-engine/Cargo.toml
COPY services/data-engine/Cargo.toml ./services/data-engine/Cargo.toml
COPY services/gateway/Cargo.toml ./services/gateway/Cargo.toml
COPY services/strategy-engine/Cargo.toml ./services/strategy-engine/Cargo.toml
COPY services/strategy-generator/Cargo.toml ./services/strategy-generator/Cargo.toml

# Create dummy source files for all workspace members (dependency caching)
RUN mkdir -p services/common/src && echo "pub fn dummy() {}" > services/common/src/lib.rs && \
    mkdir -p services/backtest-engine/src && echo "pub fn dummy() {}" > services/backtest-engine/src/lib.rs && \
    mkdir -p services/data-engine/src && echo "fn main() {}" > services/data-engine/src/main.rs && \
    mkdir -p services/data-engine/src/bin && echo "fn main() {}" > services/data-engine/src/bin/backfill.rs && echo "fn main() {}" > services/data-engine/src/bin/sync-market.rs && \
    mkdir -p services/data-engine/benches && echo "fn main() {}" > services/data-engine/benches/parser_benchmarks.rs && echo "fn main() {}" > services/data-engine/benches/storage_benchmarks.rs && \
    mkdir -p services/gateway/src && echo "fn main() {}" > services/gateway/src/main.rs && \
    mkdir -p services/strategy-engine/src && echo "fn main() {}" > services/strategy-engine/src/main.rs && \
    mkdir -p services/strategy-generator/src && echo "fn main() {}" > services/strategy-generator/src/main.rs

# Build dependencies only (caches layer)
RUN cargo build --release -p data-engine 2>/dev/null || true

# Remove dummy files and cached artifacts for path dependencies
RUN rm -rf services/data-engine/src services/data-engine/benches services/common/src && \
    rm -rf target/release/.fingerprint/common-* target/release/.fingerprint/data_engine-* && \
    rm -rf target/release/deps/libcommon* target/release/deps/data_engine*

# Copy actual source code
COPY services/common ./services/common
COPY services/data-engine ./services/data-engine
COPY infrastructure ./infrastructure

# Build the application
RUN find /app -name ".git" -type d -prune -exec rm -rf {} + || true
RUN cargo build --release -p data-engine --bin data-engine

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
COPY --from=builder /app/target/release/data-engine /usr/local/bin/

# Copy config directory
COPY --from=builder /app/services/data-engine/config /app/config
WORKDIR /app

EXPOSE 8080
CMD ["data-engine"]
