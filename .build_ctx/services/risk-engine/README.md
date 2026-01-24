# Risk Engine Service

HermesFlow 风险引擎服务 - 使用 Python 和 FastAPI 构建的风险评估服务。

## 🚀 功能

- ✅ 健康检查端点 (`/health`)
- ✅ FastAPI 框架
- ✅ 自动 API 文档

## 🛠️ 技术栈

- **语言**: Python 3.12
- **框架**: FastAPI 0.109

## 📦 本地开发

```bash
export PYTHONPATH=src
python -m uvicorn risk_engine.main:app --reload --port 8030
```

## 🧪 测试

```bash
PYTHONPATH=src pytest
```

## 🐳 Docker

```bash
docker build -t risk-engine:latest .
docker run -p 8030:8030 risk-engine:latest
```
