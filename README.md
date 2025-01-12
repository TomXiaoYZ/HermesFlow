# HermesFlow

HermesFlow 是一个现代化的量化交易平台，支持多交易所（CEX/DEX）数据接入、策略开发、回测和实盘交易。系统采用微服务架构，提供高性能、可扩展的交易解决方案。

## 🌟 主要特点

- 多交易所支持：同时接入 CEX（Binance、OKX、Bitget）和 DEX（GMGN、Uniswap）
- 高性能交易引擎：使用 Rust 开发的低延迟交易核心
- 灵活的策略开发：支持 Python、Rust、Go 等多语言策略开发
- 完整的回测系统：支持历史数据回测和链上回测
- 实时风控系统：多维度风险监控和自动化风控
- 可视化分析：直观的数据展示和策略分析工具

## 🚀 快速开始

### 环境要求

- Docker Desktop
- Python 3.8+
- Rust 1.70+
- Go 1.20+

### 本地开发环境搭建

1. 克隆仓库
```bash
git clone https://github.com/yourusername/HermesFlow.git
cd HermesFlow
```

2. 安装依赖
```bash
# 安装 Python 依赖
pip install -r requirements.txt

# 安装 Rust 依赖
cd trading-engine
cargo build

# 安装 Go 依赖
cd ../api-gateway
go mod download
```

3. 启动本地开发环境
```bash
docker-compose up -d
```

4. 访问服务
- Web UI: http://localhost:3000
- API 文档: http://localhost:8080/docs

## 📚 文档

- [架构设计](./architecture.md)
- [开发进度](./progress.md)
- [API 文档](./docs/api/README.md)
- [部署指南](./docs/deployment/README.md)

## 🔧 开发规范

- 遵循各语言官方代码规范
- 提交前运行测试套件
- 遵循 Git 分支管理策略

## 🤝 贡献指南

1. Fork 本仓库
2. 创建功能分支 (`git checkout -b feature/AmazingFeature`)
3. 提交更改 (`git commit -m 'Add some AmazingFeature'`)
4. 推送到分支 (`git push origin feature/AmazingFeature`)
5. 开启 Pull Request

## 📄 许可证

本项目采用 MIT 许可证 - 详见 [LICENSE](LICENSE) 文件

## 👥 作者

- 作者名字 - [@yourgithub](https://github.com/yourgithub)

## 🙏 致谢

感谢所有为本项目做出贡献的开发者！