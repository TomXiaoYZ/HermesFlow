.PHONY: help setup lint test build up down clean logs web-setup web-dev web-build audit

help:
	@echo "HermesFlow Development Commands"
	@echo "==============================="
	@echo "Dev Environment:"
	@echo "  make setup      - Verify Rust toolchain and install frontend deps"
	@echo "  make lint       - Run clippy on all Rust crates"
	@echo "  make test       - Run tests across all Rust crates"
	@echo "Web (Frontend):"
	@echo "  make web-setup  - Install frontend dependencies"
	@echo "  make web-dev    - Start frontend dev server"
	@echo "  make web-build  - Build frontend for production"
	@echo "Docker:"
	@echo "  make build      - Build all docker images"
	@echo "  make up         - Start services locally"
	@echo "  make down       - Stop services"
	@echo "  make logs       - View logs"
	@echo "  make clean      - Stop services, remove volumes and build artifacts"
	@echo "Security:"
	@echo "  make audit      - Run cargo-audit and npm audit"

setup:
	@echo ">>> Verifying Rust Toolchain..."
	cargo check --workspace
	@$(MAKE) web-setup
	@echo "Setup complete."

web-setup:
	@echo ">>> Setting up Frontend..."
	cd services/web && npm install

web-dev:
	@echo ">>> Starting Frontend Dev Server..."
	cd services/web && npm run dev

web-build:
	@echo ">>> Building Frontend..."
	cd services/web && npm run build

lint:
	@echo ">>> Linting Rust..."
	cargo clippy --workspace -- -D warnings
	@echo "Rust lint complete."

test:
	@echo ">>> Testing Rust..."
	cargo test --workspace
	@echo "Rust tests complete."

build:
	docker compose build

up:
	docker compose up -d

down:
	docker compose down

clean:
	docker compose down -v
	find . -type d -name "__pycache__" -exec rm -rf {} +
	find . -type d -name "target" -exec rm -rf {} +
	rm -rf services/web/node_modules services/web/.next

logs:
	docker compose logs -f

audit:
	@echo ">>> Running Security Audit..."
	@command -v cargo-audit >/dev/null 2>&1 || cargo install cargo-audit
	cargo audit
	cd services/web && npm audit --audit-level=moderate
	@echo "Security audit complete."
