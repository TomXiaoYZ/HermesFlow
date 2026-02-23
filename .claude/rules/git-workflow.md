---
description: Git workflow and commit conventions — enforces dev-agent-skills plugin workflows
globs: ["**/*"]
---

# Git Workflow

## MANDATORY: Use dev-agent-skills for All Git Operations

Every git commit, PR creation, PR review, and PR merge **MUST** use the corresponding `github-workflow` skill. These are not optional suggestions — they are the required process.

| Operation | Required Skill | Trigger |
|-----------|---------------|---------|
| Committing changes | `github-workflow:git-commit` | Any `git commit` |
| Creating a PR | `github-workflow:github-pr-creation` | Any PR creation |
| Reviewing PR comments | `github-workflow:github-pr-review` | Any PR review feedback |
| Merging a PR | `github-workflow:github-pr-merge` | Any PR merge |

## Commit Messages

Use Conventional Commits with **required scope** (kebab-case):

```
type(scope): subject
```

- `feat(scope):` - New feature
- `fix(scope):` - Bug fix
- `refactor(scope):` - Code restructuring
- `test(scope):` - Adding/updating tests
- `docs(scope):` - Documentation only
- `chore(scope):` - Build, CI, dependencies
- `perf(scope):` - Performance improvement
- `security(scope):` - Vulnerability fixes or hardening

### Commit Rules
- Scope is **required** — use service/module name: `gateway`, `data-engine`, `execution-engine`, `web`, `strategy-generator`, etc.
- Subject: present tense imperative verb, no period, max 50 chars
- **NEVER** use generic messages ("update code", "fix bug", "changes")
- Use HEREDOC for multi-line commits
- Group related changes into a single focused commit

## Before Committing
1. `cargo clippy --workspace -- -D warnings` passes
2. `cargo test --workspace` passes
3. `cargo fmt --all` applied
4. No hardcoded secrets in diff
5. No debug println!/console.log in production code

## Pull Request Workflow

**MUST use `github-workflow:github-pr-creation` skill**, which enforces:
1. Confirm target branch with user
2. Search for task documentation
3. Analyze commits and verify task completion
4. Run tests (must pass before creating PR)
5. Generate Conventional Commits title: `type(scope): description`
6. Generate PR body from template
7. Check available labels with `gh label list` and suggest matches
8. Check open milestones and assign if applicable
9. Show full PR content for user approval before creating

## PR Review Workflow

**MUST use `github-workflow:github-pr-review` skill**, which enforces:
- Severity-based comment classification: CRITICAL > HIGH > MEDIUM > LOW
- CRITICAL/HIGH: must fix (separate commits per fix)
- MEDIUM/LOW: batch into single `style:` commit
- Reply to every thread with standard templates (no emojis)
- Run tests before pushing
- Submit formal review via `gh pr review`

## PR Merge Workflow

**MUST use `github-workflow:github-pr-merge` skill**, which enforces:
- Verify all review comments have replies (STOP if unreplied)
- Check milestone assignment (warn if missing)
- Run tests, lint, CI checks (all must pass)
- Confirm with user before merging
- Use merge commit (`--merge`), never squash/rebase
- Delete feature branch after merge
- Check milestone completion after merge

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
