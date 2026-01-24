.PHONY: help setup lint test build up down clean logs

# Python Virtual Environment
VENV := .venv
PYTHON := $(VENV)/bin/python
PIP := $(VENV)/bin/pip
PYTEST := $(VENV)/bin/pytest
RUFF := $(VENV)/bin/ruff
MYPY := $(VENV)/bin/mypy

help:
	@echo "HermesFlow Development Commands"
	@echo "==============================="
	@echo "Dev Environment (Isolated in $(VENV)):"
	@echo "  make setup      - Create .venv and install all dependencies"
	@echo "  make lint       - Run linters via .venv"
	@echo "  make test       - Run tests via .venv"
	@echo "Web (Frontend):"
	@echo "  make web-setup  - Install frontend dependencies"
	@echo "  make web-dev    - Start frontend dev server"
	@echo "  make web-build  - Build frontend for production"
	@echo "Docker:"
	@echo "  make build      - Build all docker images"
	@echo "  make up         - Start services locally"
	@echo "  make down       - Stop services"
	@echo "  make logs       - View logs"

setup:
	@echo ">>> 🐍 Setting up Virtual Environment ($(VENV))..."
	test -d $(VENV) || python3 -m venv $(VENV)
	@echo ">>> 📦 Installing Dependencies..."
	$(PIP) install --upgrade pip setuptools wheel
	$(PIP) install ruff mypy pytest types-redis
	$(PIP) install -e infrastructure/python/hermes_common
	$(PIP) install -e services/risk-engine

	@echo ">>> 🐦 Installing Twitter Scraper..."
	cd services/twitter-scraper && poetry install || $(PIP) install -r requirements.txt
	@echo ">>> 🦀 Verifying Rust Toolchain..."
	cd services/data-engine && cargo check
	cd services/gateway && cargo check
	@$(MAKE) web-setup
	@echo "✅ Setup Complete. Activate with: source $(VENV)/bin/activate"

web-setup:
	@echo ">>> ⚛️ Setting up Frontend..."
	cd services/web && npm install

web-dev:
	@echo ">>> ⚛️ Starting Frontend Dev Server..."
	cd services/web && npm run dev

web-build:
	@echo ">>> ⚛️ Building Frontend..."
	cd services/web && npm run build

lint:
	@echo ">>> 🐍 Linting Python..."
	$(RUFF) check infrastructure/python/hermes_common services/risk-engine
	$(MYPY) infrastructure/python/hermes_common services/risk-engine
	@echo ">>> 🦀 Linting Rust..."
	cd services/data-engine && cargo clippy -- -D warnings
	cd services/gateway && cargo clippy -- -D warnings
	@echo ">>> ⚛️ Linting Frontend..."
	cd services/web && npm run build -- --emptyOutDir # Basic check

test:
	@echo ">>> 🐍 Testing Python..."
	$(PYTEST) services/risk-engine infrastructure/python/hermes_common
	@echo ">>> 🦀 Testing Rust..."
	cd services/data-engine && cargo test
	cd services/gateway && cargo test
	@echo ">>> ⚛️ Testing Frontend..."
	# cd services/web && npm test

build:
	docker compose build

up:
	docker compose up -d

down:
	docker compose down

clean:
	docker compose down -v
	rm -rf $(VENV)
	find . -type d -name "__pycache__" -exec rm -rf {} +
	find . -type d -name ".pytest_cache" -exec rm -rf {} +
	find . -type d -name ".ruff_cache" -exec rm -rf {} +
	find . -type d -name "target" -exec rm -rf {} +
	rm -rf services/web/node_modules services/web/dist

logs:
	docker compose logs -f
