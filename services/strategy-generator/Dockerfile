FROM rust:slim-bookworm as builder

WORKDIR /app

# Install build dependencies
RUN apt-get update && apt-get install -y pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*

# Copy dependencies first (path dependencies)
COPY services/common ./services/common
COPY services/backtest-engine ./services/backtest-engine

# Copy source code
COPY services/strategy-generator ./services/strategy-generator

# Remove lockfiles to force regeneration (solves version mismatch)
# Using find to be robust against missing files
RUN find . -name Cargo.lock -delete

# Build using explicit manifest path
# This avoids WORKDIR confusion
RUN cargo build --release --manifest-path services/strategy-generator/Cargo.toml

# Runtime stage
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y ca-certificates libssl-dev && rm -rf /var/lib/apt/lists/*

WORKDIR /app
# Note: Without a workspace, target dir is relative to manifest
# Since we built strategy-generator, it should be in services/strategy-generator/target/release
COPY --from=builder /app/services/strategy-generator/target/release/strategy-generator .

# Env vars
ENV RUST_LOG=info

CMD ["./strategy-generator"]
