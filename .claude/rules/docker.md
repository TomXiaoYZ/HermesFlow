---
description: Docker build and deployment conventions
globs: ["**/Dockerfile", "docker-compose*.yml", "docker-compose*.yaml"]
---

# Docker Conventions

## Build Context
- All builds use root directory as context: `context: .` in compose
- Dockerfiles live in `services/{name}/Dockerfile`
- This allows accessing workspace Cargo.toml and shared crates

## Multi-Stage Builds
- Builder stage: compile with dependencies cached
- Runtime stage: minimal image (distroless or alpine)
- Copy only the compiled binary, not source

## Health Checks
- Every service must have a health check endpoint
- Docker compose health checks configured
- Dependencies use `depends_on` with `condition: service_healthy`

## Environment
- Use `.env` locally for secrets
- Never bake secrets into images
- Use build args only for non-sensitive config
- Runtime config via environment variables

## Workspace Builds (Rust)
- Cache cargo registry and target directory
- Use `SQLX_OFFLINE=true` for builds without database
- execution-engine builds separately (not in workspace)
