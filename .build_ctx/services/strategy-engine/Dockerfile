# Build stage
FROM rust:slim-bookworm AS builder
WORKDIR /app

# Dependencies
RUN apt-get update && apt-get install -y pkg-config libssl-dev g++ make && rm -rf /var/lib/apt/lists/*

# Copy Source Code (simplest path for now to handle local deps)
COPY services/common ./services/common
COPY services/backtest-engine ./services/backtest-engine
COPY services/strategy-engine ./services/strategy-engine

# Build
WORKDIR /app/services/strategy-engine
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates libssl3 && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/services/strategy-engine/target/release/strategy-engine /usr/local/bin/

# Environment variables should be set in docker-compose
CMD ["strategy-engine"]
