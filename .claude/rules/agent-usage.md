# Agent & Sub-Agent Usage Rules

## Context: Bedrock Cross-Region Inference

This project uses `global.anthropic.claude-opus-4-6-v1` via Amazon Bedrock cross-region inference. Opus models have high first-token latency, and sub-agents (Task tool) create additional API requests that increase 504 timeout risk. These rules minimize timeout failures while preserving efficiency.

## When to Use Sub-Agents

### Explore Agent (LOW risk, ENCOURAGED)
- Codebase search, file discovery, keyword search
- Understanding architecture or code flow
- Answering "where is X?" or "how does Y work?" questions
- Always use `subagent_type: "Explore"` for these tasks

### Haiku-Model Agents (LOW risk, OK)
- Simple, well-scoped tasks (format check, count lines, summarize file)
- Specify `model: "haiku"` to reduce latency and timeout risk
- Keep max_turns low (< 5)

### Background Bash Tasks (NO risk, PREFERRED for parallel work)
- Docker builds, long compilations, test suites
- Use `run_in_background: true` on Bash tool instead of spawning agents
- Check results with `TaskOutput` or `Read` on the output file
- This runs shell commands without model API calls — zero 504 risk

## When NOT to Use Sub-Agents

### Never Use Agent Teams for Build/Deploy
- Docker build, `cargo build`, `npm build` — use parallel Bash background tasks
- Health check verification — use direct `curl` commands
- Lint and test (`cargo clippy`, `cargo test`) — run directly

### Never Use Agents for Code Editing
- Writing, editing, or refactoring code — do directly in main session
- Commit and push workflows — do directly (needs precise control)
- File creation — do directly

### Never Use Agents for Multi-Step Workflows
- Sequences like "build → start → verify → commit" — do directly
- Any task where step N depends on step N-1 output — do directly
- Complex debugging requiring context accumulation — do directly

## Parallel Work Strategy

| Task Type | Method | Rationale |
|-----------|--------|-----------|
| Code search / exploration | Explore sub-agent | Fast, read-only, low API cost |
| Architecture analysis | Explore sub-agent | Read-only, bounded scope |
| Docker builds (parallel) | Multiple Bash `run_in_background` | No model API, zero 504 risk |
| Test suites (parallel) | Multiple Bash `run_in_background` | No model API, zero 504 risk |
| Code writing | Direct (main session) | Needs context, precise control |
| Commit / push | Direct (main session) | Must be sequential, reliable |
| Plan design | Direct (main session) | Needs full conversation context |

## Agent Team Rules

### Avoid Agent Teams Unless Explicitly Requested
- Default to direct execution + Bash background tasks
- Only create teams when the user explicitly asks for team/swarm coordination
- Previous team attempts (rust-linter, docker-strategy) failed silently due to 504

### If Teams Are Needed
- Use `model: "haiku"` for simple teammate tasks
- Keep each teammate's scope narrow (single responsibility)
- Set `run_in_background: true` where possible
- Always have a fallback plan to do the work directly if agents go unresponsive
- Monitor for idle notifications — two consecutive idles without response = likely 504, switch to direct execution

## Error Recovery
- If a sub-agent goes idle without responding after a message: assume 504, do the work directly
- If a sub-agent returns an API error: do not retry the agent, do the work directly
- Never spend more than one retry on an unresponsive agent
