# Data Engine Service

HermesFlow 数据引擎服务 - 使用 Rust 和 Axum 构建的高性能数据处理服务。

## 🚀 功能

- ✅ 健康检查端点 (`/health`)
- ✅ 异步处理
- ✅ 高性能 HTTP 服务器

## 🛠️ 技术栈

- **语言**: Rust 1.75+
- **框架**: Axum 0.7
- **异步运行时**: Tokio

## 📦 本地开发

```bash
cargo run
```

## 🧪 测试

```bash
cargo test
```

## 🐳 Docker

```bash
docker build -t data-engine:latest .
docker run -p 8080:8080 data-engine:latest
```
