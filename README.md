# HermesFlow

HermesFlow 是一个高性能的量化交易系统，支持多交易所、多策略的实时交易。系统采用微服务架构，各个组件之间通过消息队列和缓存进行解耦，保证系统的可扩展性和可维护性。

## 功能特点

- 多交易所支持
  - Binance
  - OKX
  - Bitget（开发中）
- 实时行情数据
  - WebSocket实时数据流
  - REST API数据补充
  - 数据质量监控
- 策略引擎
  - 策略框架
  - 回测系统
  - 实时交易
- 风控系统
  - 账户风控
  - 策略风控
  - 系统风控
- 监控分析
  - 性能监控
  - 交易分析
  - 报表系统

## 快速开始

### 环境要求

- Python 3.11+
- Docker
- Redis
- PostgreSQL
- Kafka
- ClickHouse

### 安装

1. 克隆仓库
```bash
git clone https://github.com/yourusername/hermesflow.git
cd hermesflow
```

2. 安装Poetry
```bash
curl -sSL https://install.python-poetry.org | python3 -
```

3. 安装依赖
```bash
poetry install
```

4. 启动开发环境
```bash
docker-compose -f docker/docker-compose.dev.yml up -d
```

### 配置

1. 复制环境变量模板
```bash
cp config/development/.env.example config/development/.env
```

2. 编辑环境变量
```bash
vim config/development/.env
```

### 运行测试

```bash
poetry run pytest
```

## 文档

- [架构设计](docs/architecture.md)
- [API文档](docs/api/README.md)
- [使用指南](docs/guides/README.md)

## 贡献

1. Fork 项目
2. 创建特性分支 (`git checkout -b feature/amazing-feature`)
3. 提交更改 (`git commit -m 'feat: add some amazing feature'`)
4. 推送到分支 (`git push origin feature/amazing-feature`)
5. 创建Pull Request

## 许可证

本项目采用 MIT 许可证 - 查看 [LICENSE](LICENSE) 文件了解详情 