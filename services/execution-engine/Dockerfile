# Build stage
FROM rust:slim-bookworm AS builder
WORKDIR /app

# Dependencies
RUN apt-get update && apt-get install -y pkg-config libssl-dev g++ make && rm -rf /var/lib/apt/lists/*

# Copy workspace root manifest and lockfile (needed by common crate)
COPY Cargo.toml Cargo.lock ./

# Copy all workspace member Cargo.toml files (common needs workspace resolution)
COPY services/common/Cargo.toml ./services/common/Cargo.toml
COPY services/backtest-engine/Cargo.toml ./services/backtest-engine/Cargo.toml
COPY services/data-engine/Cargo.toml ./services/data-engine/Cargo.toml
COPY services/gateway/Cargo.toml ./services/gateway/Cargo.toml
COPY services/strategy-engine/Cargo.toml ./services/strategy-engine/Cargo.toml
COPY services/strategy-generator/Cargo.toml ./services/strategy-generator/Cargo.toml

# Copy execution-engine Cargo.toml (excluded from workspace, builds standalone)
COPY services/execution-engine/Cargo.toml ./services/execution-engine/Cargo.toml

# Create dummy source files for workspace members and execution-engine
RUN mkdir -p services/common/src && echo "pub fn dummy() {}" > services/common/src/lib.rs && \
    mkdir -p services/backtest-engine/src && echo "pub fn dummy() {}" > services/backtest-engine/src/lib.rs && \
    mkdir -p services/data-engine/src && echo "fn main() {}" > services/data-engine/src/main.rs && \
    mkdir -p services/data-engine/benches && echo "fn main() {}" > services/data-engine/benches/parser_benchmarks.rs && echo "fn main() {}" > services/data-engine/benches/storage_benchmarks.rs && \
    mkdir -p services/gateway/src && echo "fn main() {}" > services/gateway/src/main.rs && \
    mkdir -p services/strategy-engine/src && echo "fn main() {}" > services/strategy-engine/src/main.rs && \
    mkdir -p services/strategy-generator/src && echo "fn main() {}" > services/strategy-generator/src/main.rs && \
    mkdir -p services/execution-engine/src && echo "fn main() {}" > services/execution-engine/src/main.rs

# Pre-build dependencies (execution-engine is excluded from workspace, built via manifest-path)
RUN cargo build --release --manifest-path services/execution-engine/Cargo.toml 2>/dev/null || true

# Remove dummy files and cached artifacts for path dependencies
RUN rm -rf services/execution-engine/src services/common/src && \
    rm -rf services/execution-engine/target/release/.fingerprint/common-* services/execution-engine/target/release/.fingerprint/execution_engine-* && \
    rm -rf services/execution-engine/target/release/deps/libcommon* services/execution-engine/target/release/deps/execution_engine*

# Copy actual source code
COPY services/common ./services/common
COPY services/execution-engine ./services/execution-engine

# Build
RUN cargo build --release --manifest-path services/execution-engine/Cargo.toml

# Runtime stage
FROM debian:bookworm-slim

RUN set -eux; \
    for i in 1 2 3 4 5; do \
    apt-get update -o Acquire::Retries=5 && \
    apt-get install -y --no-install-recommends -o Acquire::Retries=5 \
    ca-certificates \
    curl \
    && rm -rf /var/lib/apt/lists/* \
    && break; \
    echo "apt-get failed (attempt $i/5), retrying..." >&2; \
    sleep 5; \
    done

# Create non-root user
RUN groupadd -r appuser && useradd -r -g appuser appuser

COPY --from=builder /app/services/execution-engine/target/release/execution-engine /usr/local/bin/

USER appuser
CMD ["execution-engine"]
