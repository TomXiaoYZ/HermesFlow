# Strategy Engine Service

HermesFlow 策略引擎服务 - 使用 Python 和 FastAPI 构建的交易策略服务。

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
python -m uvicorn strategy_engine.main:app --reload --port 8020
```

## 🧪 测试

```bash
PYTHONPATH=src pytest
```

## 🐳 Docker

```bash
docker build -t strategy-engine:latest .
docker run -p 8020:8020 strategy-engine:latest
```
