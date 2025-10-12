# 开发指南

**版本**: v2.0.0  
**最后更新**: 2024-12-20

---

## 目录

1. [项目结构](#1-项目结构)
2. [技术栈](#2-技术栈)
3. [开发环境要求](#3-开发环境要求)
4. [开发流程](#4-开发流程)
5. [常见任务](#5-常见任务)

---

## 1. 项目结构

```
HermesFlow/
├── docs/                        # 文档
├── modules/                     # 微服务模块
│   ├── data-engine/            # 数据采集服务 (Rust) ⭐
│   ├── strategy-engine/        # 策略引擎 (Python)
│   ├── trading-engine/         # 交易执行 (Java)
│   ├── risk-engine/            # 风控服务 (Java)
│   ├── user-management/        # 用户管理 (Java)
│   ├── backtest-engine/        # 回测引擎 (Python)
│   ├── notification-center/    # 通知中心 (Java)
│   ├── gateway/                # API网关 (Java)
│   └── frontend/               # 前端 (React + TS)
├── infrastructure/             # 基础设施代码
│   └── terraform/              # Terraform配置
├── scripts/                    # 构建和部署脚本
└── tests/                      # 集成测试
```

### 模块职责划分

| 模块 | 技术栈 | 端口 | 职责 |
|------|--------|------|------|
| data-engine | **Rust** | 18001-18002 | 数据采集、处理、分发 ⭐ |
| strategy-engine | Python | 18020-18021 | 策略开发、回测、执行 |
| trading-engine | Java | 18030 | 订单管理、交易执行 |
| risk-engine | Java | 18040 | 风险监控、风控规则 |
| user-management | Java | 18010 | 用户、租户、权限 |
| gateway | Java | 18000 | API网关、路由 |
| frontend | React+TS | 3000 | Web界面 |

---

## 2. 技术栈

### 2.1 Rust服务（数据模块）⭐

**核心依赖**

```toml
[dependencies]
# 异步运行时
tokio = { version = "1.35", features = ["full"] }

# Web框架
actix-web = "4.4"
# 或 axum = "0.7"

# WebSocket
tokio-tungstenite = "0.21"

# 序列化
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

# 数据库
sqlx = { version = "0.7", features = ["postgres", "runtime-tokio"] }
redis = { version = "0.24", features = ["tokio-comp"] }

# 消息队列
rdkafka = "0.36"

# gRPC
tonic = "0.10"
prost = "0.12"

# 日志
tracing = "0.1"
tracing-subscriber = "0.3"

# 错误处理
anyhow = "1.0"
thiserror = "1.0"

# 性能监控
prometheus = "0.13"
```

### 2.2 Java服务

- **JDK**: 21 (Virtual Threads)
- **框架**: Spring Boot 3.x, Spring WebFlux
- **构建工具**: Maven 3.9+
- **数据库**: Spring Data JPA, R2DBC
- **消息队列**: Spring Kafka
- **监控**: Micrometer + Prometheus

### 2.3 Python服务

- **版本**: Python 3.12
- **框架**: FastAPI
- **依赖管理**: Poetry
- **数据分析**: Pandas, NumPy
- **异步**: asyncio, aiohttp

---

## 3. 开发环境要求

### 3.1 必需工具

#### Rust工具链 ⭐

```bash
# 安装Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# 配置环境变量
source $HOME/.cargo/env

# 验证安装
rustc --version  # 应该 >= 1.75
cargo --version

# 安装常用工具
cargo install cargo-watch  # 文件监控自动重新编译
cargo install cargo-edit   # 依赖管理
cargo install cargo-tarpaulin  # 代码覆盖率
```

#### Java工具链

```bash
# 安装JDK 21
# macOS
brew install openjdk@21

# 验证
java --version  # 应该是 21.x
javac --version
```

#### Python工具链

```bash
# 安装Python 3.12
# macOS
brew install python@3.12

# 安装Poetry
curl -sSL https://install.python-poetry.org | python3 -

# 验证
python3.12 --version
poetry --version
```

#### Docker

```bash
# 安装Docker Desktop
# macOS: 从 https://www.docker.com/products/docker-desktop 下载

# 验证
docker --version
docker-compose --version
```

### 3.2 IDE配置

#### VS Code（推荐用于Rust开发）⭐

安装插件：
- **rust-analyzer**: Rust语言支持
- **CodeLLDB**: Rust调试
- **crates**: 依赖管理
- **Even Better TOML**: TOML文件支持

配置 `.vscode/settings.json`:

```json
{
    "rust-analyzer.checkOnSave.command": "clippy",
    "rust-analyzer.cargo.features": "all",
    "[rust]": {
        "editor.formatOnSave": true,
        "editor.defaultFormatter": "rust-lang.rust-analyzer"
    }
}
```

#### IntelliJ IDEA（推荐用于Java开发）

- 启用 Spring Boot 支持
- 安装 Lombok 插件
- 配置 Maven 自动导入

---

## 4. 开发流程

### 4.1 获取代码

```bash
# 克隆主仓库
git clone https://github.com/yourusername/HermesFlow.git
cd HermesFlow

# 克隆GitOps仓库
git clone https://github.com/yourusername/HermesFlow-GitOps.git
```

### 4.2 启动基础设施

```bash
# 启动PostgreSQL, Redis, ClickHouse, Kafka
docker-compose -f infrastructure/docker-compose.local.yml up -d

# 查看状态
docker-compose -f infrastructure/docker-compose.local.yml ps
```

### 4.3 开发Rust服务 ⭐

```bash
cd modules/data-engine

# 检查代码
cargo check

# 运行测试
cargo test

# 运行服务（开发模式）
cargo run

# 监控文件变化自动重新编译
cargo watch -x run

# 构建发布版本
cargo build --release

# 运行基准测试
cargo bench
```

### 4.4 开发Java服务

```bash
cd modules/trading-engine

# 编译
./mvnw clean compile

# 运行测试
./mvnw test

# 运行服务
./mvnw spring-boot:run

# 打包
./mvnw package
```

### 4.5 开发Python服务

```bash
cd modules/strategy-engine

# 安装依赖
poetry install

# 激活虚拟环境
poetry shell

# 运行测试
pytest

# 运行服务
uvicorn main:app --reload --port 18020
```

---

## 5. 常见任务

### 5.1 添加新的Rust依赖 ⭐

```bash
# 方法1：手动编辑Cargo.toml
[dependencies]
new-crate = "1.0"

# 方法2：使用cargo-edit
cargo add new-crate
cargo add --dev test-crate  # 开发依赖
```

### 5.2 运行集成测试

```bash
# Rust集成测试
cd modules/data-engine
cargo test --test integration_tests

# Java集成测试
cd modules/trading-engine
./mvnw verify

# Python集成测试
cd modules/strategy-engine
pytest tests/integration/
```

### 5.3 调试

#### Rust调试 ⭐

```bash
# 使用lldb
rust-lldb target/debug/data-engine

# 或在VS Code中按F5启动调试
```

#### Java调试

在IntelliJ IDEA中设置断点，按Debug运行

#### Python调试

```python
# 使用pdb
import pdb; pdb.set_trace()

# 或在VS Code中设置断点
```

### 5.4 性能分析

#### Rust性能分析 ⭐

```bash
# 使用flamegraph
cargo install flamegraph
cargo flamegraph

# 使用perf（Linux）
perf record -g target/release/data-engine
perf report

# 基准测试
cargo bench
```

### 5.5 代码覆盖率

```bash
# Rust
cargo tarpaulin --out Html

# Java
./mvnw jacoco:report

# Python
pytest --cov=. --cov-report=html
```

---

## 6. 故障排查

### 6.1 Rust常见问题 ⭐

**编译错误：借用检查失败**

```rust
// 错误示例
fn main() {
    let s = String::from("hello");
    let r1 = &s;
    let r2 = &mut s;  // 错误：不能同时有不可变和可变引用
}

// 正确示例
fn main() {
    let mut s = String::from("hello");
    {
        let r1 = &s;
        println!("{}", r1);
    }  // r1离开作用域
    let r2 = &mut s;  // 现在可以创建可变引用
}
```

**异步运行时错误**

```rust
// 错误：在非异步上下文中使用.await
fn main() {
    let result = async_function().await;  // 错误
}

// 正确：使用tokio::main
#[tokio::main]
async fn main() {
    let result = async_function().await;  // 正确
}
```

### 6.2 Java常见问题

**Virtual Threads不工作**

确保使用JDK 21，并正确配置：

```java
@Configuration
public class AsyncConfig {
    @Bean
    public Executor taskExecutor() {
        return Executors.newVirtualThreadPerTaskExecutor();
    }
}
```

### 6.3 Python常见问题

**依赖冲突**

```bash
# 清理缓存
poetry cache clear . --all

# 重新安装
poetry install
```

---

## 7. 参考资源

### Rust资源 ⭐
- [The Rust Programming Language](https://doc.rust-lang.org/book/)
- [Rust by Example](https://doc.rust-lang.org/rust-by-example/)
- [Tokio Tutorial](https://tokio.rs/tokio/tutorial)
- [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)

### Java资源
- [Spring Boot Reference](https://docs.spring.io/spring-boot/docs/current/reference/html/)
- [Project Reactor](https://projectreactor.io/docs/core/release/reference/)

### Python资源
- [FastAPI Documentation](https://fastapi.tiangolo.com/)
- [Pandas Documentation](https://pandas.pydata.org/docs/)

---

**文档维护者**: Development Team  
**最后更新**: 2024-12-20

