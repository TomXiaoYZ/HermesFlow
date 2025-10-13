# 开发者快速开始指南

> **从零到第一个 Commit** | **目标时间**: < 1 小时

---

## 🎯 本指南目标

帮助新开发者快速：
1. ✅ 搭建本地开发环境（30-40分钟）
2. ✅ 理解项目结构（10分钟）
3. ✅ 完成第一个 Commit（10分钟）

---

## 📋 前置要求

### 必装工具

- **Git**: 版本控制
  ```bash
  git --version  # 检查是否已安装
  ```

- **Docker Desktop**: 本地服务
  ```bash
  docker --version
  docker-compose --version
  ```

- **IDE**: 任选其一
  - VS Code（推荐，轻量级）
  - IntelliJ IDEA（Java 开发推荐）
  - PyCharm（Python 开发推荐）

### 按技术栈安装

#### 🦀 Rust 开发者

```bash
# 安装 Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env

# 验证
rustc --version  # 应为 1.75+
cargo --version

# 安装工具
cargo install cargo-watch  # 热重载
cargo install cargo-audit   # 安全审计
```

#### ☕ Java 开发者

```bash
# macOS
brew install openjdk@21

# Linux (Ubuntu/Debian)
sudo apt install openjdk-21-jdk

# Windows
# 下载并安装 Oracle JDK 21 或 OpenJDK 21

# 验证
java --version  # 应为 21+
```

#### 🐍 Python 开发者

```bash
# macOS
brew install python@3.12

# Linux
pyenv install 3.12.0

# 验证
python3 --version  # 应为 3.12+

# 安装 Poetry
curl -sSL https://install.python-poetry.org | python3 -
```

---

## 🚀 5 步开始开发

### 步骤 1: 克隆代码（2分钟）

```bash
# 克隆主仓库
git clone <your-repo-url>/HermesFlow.git
cd HermesFlow

# (可选) 克隆 GitOps 仓库（DevOps 需要）
cd ..
git clone <your-repo-url>/HermesFlow-GitOps.git
```

### 步骤 2: 启动依赖服务（5分钟）

```bash
cd HermesFlow

# 启动 PostgreSQL, Redis, ClickHouse, Kafka
docker-compose up -d

# 检查服务状态
docker-compose ps

# 应该看到所有服务状态为 "Up"
```

**验证服务**:
```bash
# PostgreSQL
docker-compose exec postgres psql -U hermesflow -c "SELECT version();"

# Redis
docker-compose exec redis redis-cli ping  # 应返回 PONG

# ClickHouse
curl http://localhost:8123/ping  # 应返回 Ok.
```

### 步骤 3: 配置开发环境（20-30分钟）

#### 🦀 Rust 开发（数据引擎）

```bash
cd modules/data-engine

# 安装依赖
cargo build

# 运行测试
cargo test

# 启动开发服务器
cargo run
```

**配置 VS Code**:
```json
// .vscode/settings.json
{
  "rust-analyzer.checkOnSave.command": "clippy",
  "rust-analyzer.cargo.features": "all",
  "editor.formatOnSave": true
}
```

**推荐插件**:
- `rust-analyzer`（Rust 语言支持）
- `crates`（Cargo.toml 依赖管理）

---

#### ☕ Java 开发（交易/用户/风控）

```bash
cd modules/trading-engine  # 或 user-management, risk-engine

# 安装依赖并运行测试
./mvnw clean test

# 启动服务
./mvnw spring-boot:run
```

**配置 IntelliJ IDEA**:
1. 打开项目: `File → Open → 选择 modules/trading-engine`
2. 启用 Lombok: `Settings → Plugins → 安装 Lombok`
3. 启用 Annotation Processing: `Settings → Build → Compiler → Annotation Processors → 勾选 Enable`

**推荐插件**:
- Lombok
- Spring Boot Assistant
- SonarLint（代码质量）

---

#### 🐍 Python 开发（策略引擎）

```bash
cd modules/strategy-engine

# 安装依赖
poetry install

# 激活虚拟环境
poetry shell

# 运行测试
pytest

# 启动服务
python main.py
```

**配置 VS Code**:
```json
// .vscode/settings.json
{
  "python.linting.enabled": true,
  "python.linting.pylintEnabled": true,
  "python.formatting.provider": "black",
  "editor.formatOnSave": true,
  "python.testing.pytestEnabled": true
}
```

**推荐插件**:
- Python（Microsoft）
- Pylance（类型检查）
- Black Formatter

---

### 步骤 4: 验证环境（5分钟）

#### 健康检查

```bash
# API Gateway (如果运行)
curl http://localhost:8080/actuator/health

# 数据引擎 (Rust)
curl http://localhost:8081/health

# 策略引擎 (Python)
curl http://localhost:8082/health

# 交易引擎 (Java)
curl http://localhost:8083/actuator/health
```

#### 运行全部测试

```bash
# Rust
cd modules/data-engine && cargo test

# Java
cd modules/trading-engine && ./mvnw test

# Python
cd modules/strategy-engine && poetry run pytest
```

✅ **如果所有测试通过，您的环境已就绪！**

---

### 步骤 5: 第一个 Commit（10分钟）

#### 5.1 找一个 "Good First Issue"

在 GitHub Issues 中找一个标记为 `good-first-issue` 的任务。

**示例**: 修复文档中的拼写错误

#### 5.2 创建功能分支

```bash
# 确保在 main 分支
git checkout main
git pull origin main

# 创建新分支
git checkout -b fix/update-readme-typo
```

**分支命名规范**:
- `feat/xxx` - 新功能
- `fix/xxx` - Bug 修复
- `docs/xxx` - 文档更新
- `test/xxx` - 测试相关
- `refactor/xxx` - 代码重构

#### 5.3 进行修改

**示例**: 修复 README.md 中的拼写错误

```bash
# 编辑文件
vim README.md

# 查看修改
git diff
```

#### 5.4 提交代码

```bash
# 添加修改
git add README.md

# 提交（使用 Conventional Commits 规范）
git commit -m "docs: fix typo in README.md"

# 推送到远程
git push origin fix/update-readme-typo
```

**Commit 消息规范**:
```
<type>(<scope>): <subject>

<body>

<footer>
```

**Type**:
- `feat`: 新功能
- `fix`: Bug 修复
- `docs`: 文档更新
- `test`: 测试相关
- `refactor`: 代码重构
- `perf`: 性能优化
- `chore`: 构建/工具链更新

**示例**:
```bash
git commit -m "feat(data-engine): add Binance WebSocket connector

- Implement real-time market data subscription
- Add reconnection logic with exponential backoff
- Add unit tests with 90% coverage

Closes #123"
```

#### 5.5 创建 Pull Request

1. 访问 GitHub 仓库
2. 点击 "Compare & pull request"
3. 填写 PR 描述:

```markdown
## 📝 描述
修复 README.md 中的拼写错误

## 🔗 相关 Issue
Closes #xxx

## ✅ 检查清单
- [x] 代码符合编码规范
- [x] 通过了所有测试
- [x] 更新了相关文档
- [x] 添加了测试（如适用）

## 📸 截图（如适用）
```

4. 请求 Code Review
5. 等待审查和合并

🎉 **恭喜！您完成了第一个 Commit！**

---

## 📚 代码审查流程

### 提交 PR 前自查

使用 [代码审查清单](./CODE-REVIEW-CHECKLIST.md):

```bash
自查清单：
- [ ] 代码符合编码规范（运行 linter）
- [ ] 添加了单元测试
- [ ] 测试覆盖率达标（Rust≥85%, Java≥80%, Python≥75%）
- [ ] 更新了相关文档
- [ ] Commit 消息符合规范
- [ ] 通过了 CI/CD 检查
```

### 运行 Linter

```bash
# Rust
cargo clippy -- -D warnings

# Java
./mvnw checkstyle:check

# Python
poetry run pylint src/
poetry run black --check src/
```

### 等待 Code Review

- ⏰ **平均时间**: < 4 小时（团队目标）
- 👥 **Reviewer**: 至少 1 人审查
- ✅ **批准后**: 自动合并（或手动合并）

### 处理 Review 意见

```bash
# 修改代码
vim src/your_file.rs

# 提交修改
git add .
git commit -m "refactor: address code review feedback"
git push origin your-branch

# PR 会自动更新
```

---

## 🛠️ 常用命令

### Git 命令

```bash
# 查看状态
git status

# 查看分支
git branch -a

# 切换分支
git checkout <branch-name>

# 拉取最新代码
git pull origin main

# 查看提交历史
git log --oneline --graph

# 撤销工作区修改
git checkout -- <file>

# 撤销最后一次提交（保留修改）
git reset --soft HEAD~1

# 变基（合并 commits）
git rebase -i HEAD~3
```

### Docker 命令

```bash
# 启动所有服务
docker-compose up -d

# 查看日志
docker-compose logs -f [service-name]

# 停止所有服务
docker-compose down

# 重建服务
docker-compose up -d --build

# 进入容器
docker-compose exec <service> bash

# 清理所有容器和卷
docker-compose down -v
```

### 测试命令

```bash
# Rust - 运行所有测试
cargo test

# Rust - 运行特定测试
cargo test test_function_name

# Rust - 显示输出
cargo test -- --nocapture

# Rust - 测试覆盖率
cargo tarpaulin --out Html

# Java - 运行所有测试
./mvnw test

# Java - 运行特定测试类
./mvnw test -Dtest=YourTestClass

# Java - 跳过测试
./mvnw install -DskipTests

# Python - 运行所有测试
poetry run pytest

# Python - 运行特定测试文件
poetry run pytest tests/test_specific.py

# Python - 显示覆盖率
poetry run pytest --cov=src --cov-report=html
```

### 调试命令

```bash
# Rust - 打印调试信息
RUST_LOG=debug cargo run

# Rust - 运行单个示例
cargo run --example example_name

# Java - 调试模式启动
./mvnw spring-boot:run -Dspring-boot.run.jvmArguments="-Xdebug -Xrunjdwp:transport=dt_socket,server=y,suspend=y,address=5005"

# Python - 调试模式
poetry run python -m pdb main.py
```

---

## 💡 开发技巧

### 1. 使用热重载

```bash
# Rust
cargo watch -x run

# Java (Spring Boot DevTools 已配置)
# 只需在 IDE 中修改代码，自动重启

# Python
# 使用 uvicorn 的 --reload 选项
poetry run uvicorn main:app --reload
```

### 2. 使用 Git Hooks

```bash
# 安装 pre-commit hooks
# .git/hooks/pre-commit

#!/bin/bash
echo "Running tests before commit..."
cargo test || exit 1
./mvnw test || exit 1
poetry run pytest || exit 1
```

### 3. 使用别名

```bash
# 添加到 ~/.bashrc 或 ~/.zshrc

# Git 别名
alias gs='git status'
alias gc='git commit'
alias gp='git push'
alias gl='git log --oneline --graph'

# Docker 别名
alias dcu='docker-compose up -d'
alias dcd='docker-compose down'
alias dcl='docker-compose logs -f'

# Rust 别名
alias ct='cargo test'
alias cr='cargo run'
alias cw='cargo watch -x run'

# 重新加载配置
source ~/.bashrc  # or ~/.zshrc
```

### 4. 使用多个终端

建议同时打开多个终端窗口：

```
终端 1: Git 操作和编辑
终端 2: 运行服务（cargo run / ./mvnw spring-boot:run）
终端 3: 运行测试（cargo test --watch）
终端 4: Docker 日志（docker-compose logs -f）
```

---

## 🆘 常见问题

### Q1: Docker 服务启动失败

**错误**: `Error: port is already allocated`

**解决**:
```bash
# 查找占用端口的进程
lsof -i :5432  # PostgreSQL
lsof -i :6379  # Redis

# 停止占用的进程
kill -9 <PID>

# 或修改 docker-compose.yml 中的端口映射
```

---

### Q2: Rust 编译很慢

**解决**:
```bash
# 使用 sccache 缓存编译
cargo install sccache
export RUSTC_WRAPPER=sccache

# 使用 mold 链接器（Linux）
sudo apt install mold
echo "[build]\nlinker = \"clang\"\nrustflags = [\"-C\", \"link-arg=-fuse-ld=mold\"]" >> ~/.cargo/config.toml
```

---

### Q3: Java OOM (Out of Memory)

**解决**:
```bash
# 增加 JVM 堆内存
export MAVEN_OPTS="-Xmx2048m"
./mvnw spring-boot:run
```

---

### Q4: Python 依赖冲突

**解决**:
```bash
# 删除 poetry.lock 重新安装
rm poetry.lock
poetry install

# 或使用虚拟环境重新开始
rm -rf .venv
poetry install
```

---

### Q5: 测试失败

**解决**:
```bash
# 查看详细错误信息
cargo test -- --nocapture  # Rust
./mvnw test -X  # Java
poetry run pytest -v  # Python

# 清理并重新运行
cargo clean && cargo test
./mvnw clean test
poetry run pytest --cache-clear
```

---

## 📖 下一步

根据您的角色，深入学习：

### 开发者
- 🦀 [Rust 开发者完整指南](./RUST-DEVELOPER-GUIDE.md)
- ☕ [Java 开发者完整指南](./JAVA-DEVELOPER-GUIDE.md)
- 🐍 [Python 开发者完整指南](./PYTHON-DEVELOPER-GUIDE.md)

### 必读文档
- 📝 [编码规范](./coding-standards.md)
- 📚 [开发指南](./dev-guide.md)
- 🏗️ [系统架构](../architecture/system-architecture.md)
- 🔍 [代码审查清单](./CODE-REVIEW-CHECKLIST.md)

### 获取帮助
- 💬 Slack: `#hermesflow-dev`
- 📖 [FAQ](../FAQ.md)
- 🔧 [故障排查手册](../operations/troubleshooting.md)

---

**祝您开发愉快！** 🚀

---

**最后更新**: 2025-01-13  
**维护者**: @pm.mdc

