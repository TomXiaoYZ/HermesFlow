---
description: Python conventions for the futu-bridge service
globs: ["services/futu-bridge/**/*.py", "**/*.py"]
---

# Python Conventions

## Standards
- Python 3.11+
- PEP 8 compliance
- Type annotations on all function signatures

## Framework
- FastAPI for HTTP endpoints
- Pydantic for data validation
- httpx for async HTTP client

## Error Handling
- Specific exception types, no bare `except:`
- FastAPI exception handlers for HTTP errors
- Structured logging, not print()

## Testing
- pytest with pytest-asyncio
- httpx.AsyncClient for API tests
- Fixtures for test setup/teardown

## Formatting
- black for code formatting
- isort for import sorting
- ruff for linting
