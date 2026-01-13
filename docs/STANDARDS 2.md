# HermesFlow Engineering Standards

本文档定义了 HermesFlow 项目的工程标准和最佳实践。所有新开发的代码必须遵守以下规范。

## 1. 架构模式

### 1.1 Repository Pattern (Rust)
在 `data-engine` 和 `gateway` 中，数据库交互必须通过 Repository 模式解耦。
- **Trait 定义**: 在 `src/repository` 中定义 Trait (例如 `TradingRepository`).
- **实现**: 具体实现放在 `src/repository/postgres` 等子目录。
- **注入**: 在 `main.rs` 中通过 `Arc<dyn Trait>` 注入到 Service/Handler。

### 1.2 Python Shared Library
Python 服务 (`risk-engine`, `strategy-engine`) **禁止** 重复造轮子。
- **Common Lib**: 通用逻辑（日志、配置、DB连接）必须放在 `infrastructure/python/hermes_common`。
- **引用方式**: 在 `pyproject.toml` 中通过文件路径引用：
  ```toml
  dependencies = ["hermes-common"]
  ```

## 2. 构建与部署

### 2.1 Monorepo Build Context
所有 Docker 构建必须在 **项目根目录** 执行。
- **Dockerfile**: 可以在服务目录下 (`services/xxx/Dockerfile`)。
- **Compose**:
  ```yaml
  build:
    context: .
    dockerfile: services/risk-engine/Dockerfile
  ```
- **优势**: 允许服务在构建时访问 `infrastructure/` 中的共享资源（如 SQL 脚本、Python 库）。

### 2.2 CI/CD
- **GitHub Actions**: 唯一的 CI 流水线定义在 `.github/workflows/ci.yml`。
- **Scripts**: 也就是 `scripts/` 目录已被废弃，**禁止**添加新的 Shell 脚本来处理构建逻辑。

## 3. 配置管理 (12-Factor App)

### 3.1 层次化配置
应用加载配置的优先级：
1. **环境变量** (最高): `DATA_ENGINE__SERVER__PORT`
2. **环境配置文件**: `config/prod.toml`
3. **默认配置**: `config/default.toml`

### 3.2 敏感信息
- **绝对禁止** 将密码/密钥提交到 Git (即使是 `prod.toml`)。
- 请使用 `.env` 文件在本地管理密钥，并在生产环境中使用 Secret Store 注入环境变量。
- 变量命名规范: `{SERVICE_NAME}__{SECTION}__{KEY}` (双下划线分隔层级).

## 4. 数据库管理

### 4.1 Schema Migration
DDL 变更必须以 SQL 文件形式提交：
- **Postgres**: `infrastructure/database/postgres/migrations`
- **ClickHouse**: `infrastructure/database/clickhouse/migrations`
- 文件命名: `XXX_description.sql` (保证顺序).

### 4.2 DDL 引用
Rust 服务中引用 SQL 文件必须使用 **相对路径** 指向 `infrastructure`:
```rust
include_str!("../../../../infrastructure/database/postgres/migrations/001_core.sql")
```
不要将 SQL 文件复制到服务目录下。

## 5. 开发工作流规范 (Strict)

### 5.1 禁止事项 (Prohibited)
1. **禁止** 创建 `scripts/` 目录或添加 `.sh` 脚本。所有构建逻辑放入 `Makefile` 或 `.github/workflows/`。
2. **禁止** 手动运行 `pip install` 或 `cargo build`（除非你清楚自己在做什么）。请使用 `make setup` 和 `make build`。
3. **禁止** 在 Python 微服务中定义独立的 Util 类。如果多个服务需要，必须提取到 `hermes-common`。
4. **禁止** 将 `node_modules` 或 `.venv` 提交到 Git。

### 5.2 新增服务检查清单
如果添加新服务 (e.g. `services/new-service`):
1. [ ] 确保使用 Root Build Context (`docker-compose.yml` 中的 `context: .`)。
2. [ ] Python 服务必须在 `pyproject.toml` 中引用 `hermes-common`。
3. [ ] Rust 服务必须在 `Cargo.toml` 中使用 Workspace 依赖 (如果配置了 Workspace)。
4. [ ] 在 root `Makefile` 的 `setup`, `lint`, `test` 目标中添加该服务的命令。
