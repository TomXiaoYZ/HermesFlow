---
description: Git workflow and commit conventions
globs: ["**/*"]
---

# Git Workflow

## Commit Messages
Use conventional commits:
- `feat:` - New feature
- `fix:` - Bug fix
- `refactor:` - Code restructuring
- `test:` - Adding/updating tests
- `docs:` - Documentation only
- `chore:` - Build, CI, dependencies
- `perf:` - Performance improvement

Format: `type: concise description of WHY, not what`

## Before Committing
1. `cargo clippy --workspace -- -D warnings` passes
2. `cargo test --workspace` passes
3. `cargo fmt --all` applied
4. No hardcoded secrets in diff
5. No debug println!/console.log in production code

## Pull Request Workflow
1. Analyze full commit history with `git diff main...HEAD`
2. Keep PR title under 70 characters
3. Use description for details
4. Include test plan

## Branch Naming
- `feat/description` - Features
- `fix/description` - Bug fixes
- `refactor/description` - Refactoring

## Module Completion Workflow (MANDATORY)

When a module/service is modified, follow this sequence **before moving to the next module**:

1. **Lint & Test**: Run `cargo clippy -p {service} -- -D warnings` and `cargo test -p {service}`
2. **Local Deploy**: Build and start the service via Docker to verify runtime behavior
   ```bash
   docker compose build {service}        # Build the modified service image
   docker compose up -d {service}        # Start/restart the service
   docker compose logs {service} --tail 50  # Check startup logs for errors
   curl -sf http://localhost:{port}/health  # Verify health endpoint responds
   ```
3. **Smoke Test**: Verify the specific functionality that was changed (e.g., check `/metrics` for new Prometheus labels, verify WebSocket reconnect, check data flow via Redis)
4. **Commit**: Write a conventional commit message describing the changes
5. **Push**: `git push` to remote after each successful module commit

Service ports for health checks:
- data-engine: 8081 (maps to internal 8080)
- gateway: 8080
- strategy-engine: internal 8082
- execution-engine: internal 8083
- strategy-generator: 8082 (maps to internal 8084)
- user-management: 8086
- futu-bridge: 8088
- web: 3000

## Rules
- Commit after completing each logical unit of work
- Never force push to main
- Never commit .env files, secrets, or credentials
