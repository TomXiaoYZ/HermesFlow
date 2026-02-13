---
description: General coding style rules applied across all languages in HermesFlow
globs: ["**/*.rs", "**/*.ts", "**/*.tsx", "**/*.py", "**/*.java"]
---

# Coding Style

## File Organization
- Feature-based organization, not file-type based
- Target 200-400 lines per file, 800 line maximum
- One public type per file in Rust
- Functions under 50 lines
- Nesting depth <= 4 levels

## Naming
- Rust: snake_case for functions/variables, PascalCase for types/traits
- TypeScript: camelCase for functions/variables, PascalCase for types/components
- Python: snake_case for functions/variables, PascalCase for classes
- Java: camelCase for methods/variables, PascalCase for classes

## Error Handling
- Handle errors at every level - never silently swallow
- User-facing: friendly messages, no internal details
- Server-side: detailed logging with context
- Rust: `thiserror` + `?` operator, two-tier (DataError + ServiceError)
- TypeScript: try-catch with proper error types
- Python: specific exception types, no bare `except:`

## Input Validation
- Validate at system boundaries (API handlers, message consumers)
- Use schema-based validation (Axum extractors, Zod, Pydantic)
- Never trust external data

## Immutability
- Prefer immutable data structures
- Rust: default immutable, explicit `mut` when needed
- TypeScript: `const`, spread operator for updates
- Python: `@dataclass(frozen=True)`, `NamedTuple`

## No Debug Output in Production
- No `println!` in Rust production code (use `tracing`)
- No `console.log` in TypeScript production code
- No `print()` in Python production code (use `logging`)
