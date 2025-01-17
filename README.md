# HermesFlow

HermesFlow 是一个高性能的量化交易平台，支持多交易所、多链的数据接入和交易执行，具备完整的策略开发、回测、风控和执行功能。

## 主要特性

- 多交易所支持（CEX & DEX）
- 实时数据处理
- 策略开发与回测
- 风险控制
- 自动化交易执行
- 多账户管理
- 实时监控与报警

## 技术栈

### 后端
- Rust（高性能交易引擎）
- Golang（API服务）
- Python（数据分析和策略回测）

### 前端
- React + TypeScript
- Redux Toolkit
- Ant Design Pro
- TradingView + ECharts

### 基础设施
- Kubernetes (EKS)
- Redis
- ClickHouse
- PostgreSQL
- Kafka
- ELK Stack

## 快速开始

### 环境要求

- Docker Desktop
- Kubernetes
- AWS CLI
- Node.js >= 18
- Rust >= 1.75
- Go >= 1.21
- Python >= 3.11

### 开发环境设置

1. 克隆仓库
```bash
git clone https://github.com/yourusername/HermesFlow.git
cd HermesFlow
```

2. 安装依赖
```bash
# 前端依赖
cd src/frontend
npm install

# 后端依赖
cd ../backend
cargo build
go mod download
pip install -r requirements.txt
```

3. 配置环境变量
```bash
cp .env.example .env.dev
# 编辑 .env.dev 文件，填入必要的配置信息
```

4. 启动开发环境
```bash
# 启动本地开发环境
make dev
```

## 项目结构

```
HermesFlow/
├── src/
│   ├── frontend/          # 前端代码
│   ├── backend/           # 后端服务
│   ├── infrastructure/    # 基础设施配置
│   └── tests/             # 测试代码
├── docs/                  # 文档
├── architecture.md        # 架构文档
├── progress.md           # 进度追踪
└── README.md
```

## 文档

- [架构文档](architecture.md)
- [进度追踪](progress.md)
- [API文档](docs/api/README.md)
- [部署指南](docs/deployment/README.md)

## 贡献指南

1. Fork 项目
2. 创建功能分支 (`git checkout -b feature/AmazingFeature`)
3. 提交更改 (`git commit -m 'Add some AmazingFeature'`)
4. 推送到分支 (`git push origin feature/AmazingFeature`)
5. 开启 Pull Request

## 许可证

[MIT](LICENSE) 