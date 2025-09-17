# HermesFlow 文档中心

## 项目简介
HermesFlow 是面向个人和小团队的多租户量化交易平台，采用混合微服务架构，支持多交易所、链上链下数据、策略开发与回测、自动化运维。

## 文档导航
- [系统架构与技术选型](architecture.md)
- [运维与自动化（DevOps）](devops.md)
- [数据库DDL目录结构](../db/README.md)

## 快速上手
1. 克隆代码仓库
2. 参考 [devops.md](devops.md) 配置环境变量、数据库、自动化脚本
3. 本地开发环境下，运行 `./scripts/deploy.sh --env local` 或 `docker-compose -f docker-compose.local.yml up -d` 一键容器化部署所有服务
4. 访问前端界面或API服务

---
文档维护：HermesFlow开发团队 