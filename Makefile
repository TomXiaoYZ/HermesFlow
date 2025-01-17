.PHONY: help dev prod test clean lint format

help: ## 显示帮助信息
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | sort | awk 'BEGIN {FS = ":.*?## "}; {printf "\033[36m%-30s\033[0m %s\n", $$1, $$2}'

dev: ## 启动开发环境
	docker-compose -f docker-compose.dev.yml up --build

prod: ## 启动生产环境
	docker-compose -f docker-compose.prod.yml up --build -d

test: ## 运行测试
	cd src/backend && cargo test
	cd src/backend && go test ./...
	cd src/backend && python -m pytest
	cd src/frontend && npm test

clean: ## 清理构建文件
	find . -type d -name "target" -exec rm -rf {} +
	find . -type d -name "node_modules" -exec rm -rf {} +
	find . -type d -name "dist" -exec rm -rf {} +
	find . -type d -name "__pycache__" -exec rm -rf {} +
	find . -type f -name "*.pyc" -delete

lint: ## 运行代码检查
	cd src/backend && cargo clippy
	cd src/backend && golangci-lint run
	cd src/backend && flake8
	cd src/frontend && npm run lint

format: ## 格式化代码
	cd src/backend && cargo fmt
	cd src/backend && gofmt -w .
	cd src/backend && black .
	cd src/frontend && npm run format

install-dev: ## 安装开发依赖
	cd src/frontend && npm install
	cd src/backend && cargo build
	cd src/backend && go mod download
	cd src/backend && pip install -r requirements.txt

build: ## 构建项目
	cd src/frontend && npm run build
	cd src/backend && cargo build --release
	cd src/backend && go build
	cd src/backend && python setup.py build

deploy-dev: ## 部署到开发环境
	kubectl apply -f infrastructure/k8s/dev/

deploy-prod: ## 部署到生产环境
	kubectl apply -f infrastructure/k8s/prod/

logs: ## 查看日志
	docker-compose -f docker-compose.dev.yml logs -f

db-migrate: ## 运行数据库迁移
	cd src/backend && alembic upgrade head

db-rollback: ## 回滚数据库迁移
	cd src/backend && alembic downgrade -1

generate-docs: ## 生成文档
	cd docs && mkdocs build

serve-docs: ## 本地服务文档
	cd docs && mkdocs serve 