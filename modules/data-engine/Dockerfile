# Build stage
FROM rust:1.75 as builder
WORKDIR /app

# Copy manifests
COPY Cargo.toml Cargo.lock* ./

# Create a dummy main.rs to cache dependencies
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release || true
RUN rm -rf src

# Copy actual source code
COPY src ./src

# Build the application
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*

# Copy the binary from builder
COPY --from=builder /app/target/release/data-engine /usr/local/bin/

EXPOSE 8080
CMD ["data-engine"]
