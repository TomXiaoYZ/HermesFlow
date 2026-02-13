---
description: Rust-specific coding conventions for all Rust services
globs: ["**/*.rs", "**/Cargo.toml"]
---

# Rust Conventions

## Error Handling
- Use `thiserror` derive macros for all error types
- Two-tier pattern: low-level errors (e.g., `DataError`) + service-level (e.g., `DataEngineError`)
- Use `?` operator for propagation, never `unwrap()` or `expect()` in production
- `retry_with_backoff()` for network/external service calls

## Architecture
- **Repository Pattern**: Trait in `src/repository/mod.rs`, impl in `src/repository/postgres/`
- **Dependency Injection**: `Arc<dyn RepositoryTrait>` passed in `main.rs`
- **Config**: 12-Factor. Env vars > `config/prod.toml` > `config/default.toml`
- **Env naming**: `{SERVICE_NAME}__{SECTION}__{KEY}` (double underscore)

## Async
- Use `tokio` runtime (workspace version 1.35+)
- No blocking calls in async context - use `tokio::task::spawn_blocking`
- Don't hold `MutexGuard` across `.await` points
- Use `tokio::select!` with cancellation-safe futures only
- Share connection pools via `Arc`

## Performance
- Prefer `&str` over `String` in function params
- Avoid unnecessary `.clone()` - use references and borrowing
- Batch database queries (avoid N+1)
- Use Redis pipelines for multiple commands
- Minimize allocations in hot paths

## Cargo
- Workspace dependencies in root `Cargo.toml`
- `cargo clippy --workspace -- -D warnings` must pass
- `cargo fmt --all` before commit
- **execution-engine** is excluded from workspace (Solana SDK tokio ~1.14 conflict) - build separately

## Testing
- `#[cfg(test)] mod tests` in same file
- `#[tokio::test]` for async tests
- Test file naming: same module structure
- Run: `cargo test --workspace`
