# Build stage
FROM rust:slim-bookworm AS builder
WORKDIR /app

# Dependencies
RUN apt-get update && apt-get install -y pkg-config libssl-dev g++ make && rm -rf /var/lib/apt/lists/*

# Copy Source Code
COPY services/common ./services/common
COPY services/execution-engine ./services/execution-engine

# Build
WORKDIR /app/services/execution-engine
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates libssl3 && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/services/execution-engine/target/release/execution-engine /usr/local/bin/

CMD ["execution-engine"]
