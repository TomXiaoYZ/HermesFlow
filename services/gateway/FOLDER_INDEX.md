# gateway - FOLDER_INDEX

> API gateway and WebSocket router. Reverse proxies all service APIs, handles JWT authentication, CORS, rate limiting, and ClickHouse log queries.

## Module Map

```
src/
  main.rs              # Entry point: Axum router, route groups (public/protected/ws),
                       #   CORS config, rate limiting, proxy handlers, WebSocket handler
  jwt_auth.rs          # JWT middleware (HS256, opt-in via JWT_SECRET env var)
                       #   - jwt_middleware(): Axum middleware for Bearer token validation
                       #   - validate_ws_token(): WebSocket query-param auth
  auth_handler.rs      # Auth proxy handler (forwards to user-management service)
                       #   - Body size limit (1MB auth, 10MB proxy)
                       #   - Error info leakage prevention
  health_checker.rs    # Background health polling for downstream services
  metrics.rs           # Prometheus metrics endpoint
```

## Route Groups

| Group | Auth | Rate Limit | Routes |
|-------|------|------------|--------|
| Public | None | None | `/health`, `/metrics` |
| Auth | None | 3 req/s burst 10 | `/api/auth/*` |
| Protected | JWT (if configured) | None | `/api/v1/*` (proxy to services) |
| WebSocket | Query-param token | None | `/ws` |

## Proxy Targets

| Route Pattern | Target Service | Port |
|---------------|----------------|------|
| `/api/auth/*` | user-management | 8086 |
| `/api/v1/data/*` | data-engine | 8080 |
| `/api/v1/strategies/*` | strategy-engine | 8082 |
| `/api/v1/generator/*` | strategy-generator | 8084 |
| `/api/v1/execution/*` | execution-engine | 8083 |
| `/api/v1/logs/*` | ClickHouse | 8123 |

## Dependencies
- `common` (workspace crate)
- `jsonwebtoken`, `tower_governor`, `tower-http` (CORS)
- Redis (for health checks), ClickHouse (log queries), TimescaleDB
