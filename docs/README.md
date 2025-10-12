# HermesFlow 文档导航

**版本**: v2.0.0  
**最后更新**: 2024-12-20  

欢迎来到HermesFlow量化交易平台文档中心！本文档提供完整的系统文档导航。

---

## 📚 文档结构

```
docs/
├── prd/                          # 产品需求文档
│   ├── PRD-HermesFlow.md        # 🌟 主PRD文档 (50-80页)
│   ├── modules/                  # 各模块详细需求
│   └── user-stories/             # 用户故事集
├── api/                          # API设计文档
│   ├── api-design.md            # 🌟 API设计总览
│   ├── rest-api-spec.yaml       # OpenAPI规范
│   └── grpc-proto/              # gRPC协议定义
├── database/                     # 数据库设计文档
│   ├── database-design.md       # 🌟 数据库设计总览
│   └── schema/                   # DDL脚本
├── development/                  # 开发文档
│   ├── dev-guide.md             # 开发指南
│   ├── coding-standards.md      # 编码规范
│   └── module-templates/         # 模块开发模板
├── deployment/                   # 部署文档
│   ├── deployment-guide.md      # 部署指南
│   └── docker-guide.md          # Docker部署
├── testing/                      # 测试文档
│   ├── test-plan.md             # 测试计划
│   └── test-strategy.md         # 测试策略
├── operations/                   # 运维文档
│   ├── monitoring.md            # 监控方案
│   └── troubleshooting.md       # 故障排查
├── architecture.md               # 🌟 系统架构文档
├── progress.md                   # 🌟 开发进度跟踪
└── QUICK-REFERENCE.md           # 快速参考指南
```

---

## 🚀 快速开始

### 对于产品经理

1. 📖 [主PRD文档](./prd/PRD-HermesFlow.md) - 了解产品全貌
2. 📊 [开发进度跟踪](./progress.md) - 查看当前进度
3. 🏗️ [系统架构文档](./architecture.md) - 理解技术架构

### 对于开发者

1. 🛠️ [开发指南](./development/dev-guide.md) - 配置开发环境
2. 📝 [编码规范](./development/coding-standards.md) - 遵循代码规范
3. 🔌 [API设计文档](./api/api-design.md) - API接口规范
4. 💾 [数据库设计](./database/database-design.md) - 数据库结构

### 对于测试工程师

1. 📋 [测试计划](./testing/test-plan.md) - 测试范围与目标
2. 🧪 [测试策略](./testing/test-strategy.md) - 测试方法论
3. ✅ [测试指南](./testing/) - 各类测试指南

### 对于运维工程师

1. 🚀 [部署指南](./deployment/deployment-guide.md) - 部署流程
2. 📊 [监控方案](./operations/monitoring.md) - 监控配置
3. 🔧 [故障排查](./operations/troubleshooting.md) - 问题诊断

---

## 📖 核心文档

### 产品需求文档 (PRD)

#### 主文档
- **[PRD-HermesFlow.md](./prd/PRD-HermesFlow.md)** 🌟
  - 产品概述与愿景
  - 用户画像与核心价值
  - 系统架构概览
  - 8大模块功能需求
  - 非功能需求与优先级路线图
  - 约80页详细文档

#### 模块详细需求 (每个6-10页)
1. **[数据模块 (Rust)](./prd/modules/01-data-module.md)** ⭐ **技术栈变更**
   - 多数据源接入（加密货币、美股、期权、舆情、宏观）
   - 超低延迟数据处理（μs级）
   - 高性能数据分发（>100k msg/s）
   - 完整的Epic、用户故事、验收标准

2. **[策略模块 (Python)](./prd/modules/02-strategy-module.md)**
   - 策略开发框架
   - 回测引擎
   - 策略执行与优化

3. **[执行模块 (Java)](./prd/modules/03-execution-module.md)**
   - 订单管理
   - 智能路由
   - 多交易所集成

4. **[风控模块 (Java)](./prd/modules/04-risk-module.md)**
   - 实时风险监控
   - 风控规则引擎
   - 清算保护

5. **[账户模块 (Java)](./prd/modules/05-account-module.md)**
   - 多账户管理
   - API密钥管理
   - 资金划拨

6. **[安全模块](./prd/modules/06-security-module.md)**
   - 认证授权
   - 数据加密
   - 审计日志

7. **[报表模块](./prd/modules/07-report-module.md)**
   - 交易报表
   - 风险报表
   - 土狗评分

8. **[用户体验模块](./prd/modules/08-ux-module.md)**
   - 可视化仪表盘
   - 通知系统
   - 移动端支持

### 技术设计文档

#### API文档
- **[API设计总览](./api/api-design.md)** 🌟
  - RESTful设计原则
  - 认证授权机制
  - 统一响应格式
  - 错误处理规范
  - 限流策略

- **[REST API规范](./api/rest-api-spec.yaml)**
  - OpenAPI 3.0格式
  - 所有服务API定义
  - 请求/响应示例

- **[gRPC协议定义](./api/grpc-proto/)**
  - market_data.proto - 实时数据流
  - strategy.proto - 策略执行信号
  - execution.proto - 订单执行通知

- **[API使用示例](./api/api-examples.md)**
  - curl命令示例
  - Python/Java/Rust客户端示例

#### 数据库文档
- **[数据库设计总览](./database/database-design.md)** 🌟
  - 三层数据架构（热/温/冷）
  - PostgreSQL主数据库设计
  - ClickHouse时序数据库设计
  - Redis缓存结构设计
  - 数据生命周期管理
  - 备份与恢复策略

- **[数据库Schema](./database/schema/)**
  - PostgreSQL DDL脚本
  - ClickHouse DDL脚本
  - Redis数据结构文档

- **[ER图](./database/er-diagram.md)**
  - 实体关系图
  - 数据流向图

#### 架构文档
- **[系统架构文档](./architecture.md)** 🌟
  - 混合技术栈架构（Rust + Java + Python）
  - 服务拓扑与通信
  - 多租户架构设计
  - 技术选型说明

- **[开发进度跟踪](./progress.md)** 🌟
  - 各模块开发状态
  - 功能点详细清单
  - 测试用例与验收标准
  - 归档代码说明

---

## 🛠️ 开发文档

### 开发环境
- **[开发指南](./development/dev-guide.md)**
  - 项目结构说明
  - 开发环境要求（Rust/Java/Python/Docker）
  - IDE配置建议
  - 调试技巧

- **[本地环境搭建](./development/local-setup.md)**
  - Rust工具链安装
  - JDK 21安装
  - Python 3.12安装
  - Docker Desktop配置
  - 服务启动指南

### 开发规范
- **[编码规范](./development/coding-standards.md)**
  - **Rust编码规范** ⭐
    - rustfmt、clippy使用
    - 错误处理最佳实践
    - 异步编程规范
  - Java编码规范（Google Style）
  - Python编码规范（PEP 8）
  - TypeScript编码规范

- **[Git工作流](./development/git-workflow.md)**
  - 分支策略
  - Commit消息规范
  - Pull Request流程
  - Code Review指南

### 开发模板
- **[Rust服务模板](./development/module-templates/rust-service-template.md)** ⭐
  - Cargo.toml配置
  - 项目结构
  - 日志与错误处理
  - 测试结构
  - Dockerfile模板

- **[Java服务模板](./development/module-templates/java-service-template.md)**
  - Maven配置
  - Spring Boot结构
  - 配置管理

- **[Python服务模板](./development/module-templates/python-service-template.md)**
  - Poetry配置
  - FastAPI结构
  - 虚拟环境管理

---

## 🚀 部署文档

### 部署指南
- **[部署指南](./deployment/deployment-guide.md)**
  - 部署架构概览
  - 环境清单（local/dev/prod）
  - 部署步骤
  - 回滚步骤

- **[Docker部署](./deployment/docker-guide.md)**
  - **Rust多阶段构建** ⭐
  - docker-compose配置
  - 镜像优化技巧

- **[Kubernetes部署](./deployment/kubernetes-guide.md)**
  - Helm Charts说明
  - Rust服务资源配置
  - ArgoCD GitOps流程

### 配置管理
- **[环境变量文档](./deployment/env-variables.md)**
  - 所有环境变量清单
  - Rust服务环境变量
  - 敏感信息管理

- **[基础设施搭建](./deployment/infrastructure-setup.md)**
  - Azure AKS集群
  - PostgreSQL/Redis/Kafka配置
  - 网络配置

---

## 🧪 测试文档

### 测试策略
- **[测试计划](./testing/test-plan.md)**
  - 测试目标与范围
  - Rust服务测试重点
  - 测试环境
  - 测试时间表

- **[测试策略](./testing/test-strategy.md)**
  - 测试金字塔
  - 覆盖率要求（Rust >85%）
  - 自动化测试策略

### 测试指南
- **[单元测试指南](./testing/unit-test-guide.md)**
  - **Rust单元测试** ⭐
    - cargo test使用
    - Mock框架（mockall）
    - 异步测试（tokio::test）
    - 基准测试（criterion）
  - JUnit 5使用指南
  - pytest使用指南

- **[集成测试指南](./testing/integration-test-guide.md)**
  - Rust集成测试
  - TestContainers使用
  - API集成测试

- **[性能测试指南](./testing/performance-test-guide.md)**
  - **Rust性能测试工具** ⭐
    - Criterion微基准测试
    - Flamegraph性能分析
  - JMeter使用
  - 性能基线

---

## 📊 运维文档

### 监控与日志
- **[监控方案](./operations/monitoring.md)**
  - Prometheus + Grafana
  - Rust服务指标（prometheus crate）
  - 告警规则配置

- **[日志方案](./operations/logging.md)**
  - ELK Stack架构
  - Rust日志格式（tracing）
  - 日志查询示例

### 故障排查
- **[故障排查手册](./operations/troubleshooting.md)**
  - 常见问题与解决方案
  - Rust服务问题排查
  - 日志分析技巧

- **[运维手册](./operations/runbook.md)**
  - 日常运维任务
  - 应急响应流程
  - 升级与回滚

---

## 🔍 快速参考

- **[快速参考指南](./QUICK-REFERENCE.md)**
  - 常用命令速查
  - Rust开发快速参考
  - API端点速查
  - 数据库连接字符串
  - 常见问题FAQ

---

## 🎯 按角色查找文档

### 产品经理
- [x] [主PRD文档](./prd/PRD-HermesFlow.md)
- [x] [开发进度跟踪](./progress.md)
- [x] [系统架构概览](./architecture.md)
- [x] [各模块详细需求](./prd/modules/)

### 后端开发（Rust）
- [x] [数据模块需求](./prd/modules/01-data-module.md)
- [x] [Rust编码规范](./development/coding-standards.md#rust编码规范)
- [x] [Rust服务模板](./development/module-templates/rust-service-template.md)
- [x] [Rust单元测试指南](./testing/unit-test-guide.md#rust单元测试)
- [x] [API设计文档](./api/api-design.md)

### 后端开发（Java）
- [x] [执行/风控模块需求](./prd/modules/)
- [x] [Java编码规范](./development/coding-standards.md#java编码规范)
- [x] [Java服务模板](./development/module-templates/java-service-template.md)
- [x] [数据库设计](./database/database-design.md)

### 后端开发（Python）
- [x] [策略模块需求](./prd/modules/02-strategy-module.md)
- [x] [Python编码规范](./development/coding-standards.md#python编码规范)
- [x] [Python服务模板](./development/module-templates/python-service-template.md)

### 前端开发
- [x] [用户体验模块需求](./prd/modules/08-ux-module.md)
- [x] [API设计文档](./api/api-design.md)
- [x] [TypeScript编码规范](./development/coding-standards.md#typescript编码规范)

### DevOps工程师
- [x] [部署指南](./deployment/deployment-guide.md)
- [x] [Docker部署](./deployment/docker-guide.md)
- [x] [Kubernetes部署](./deployment/kubernetes-guide.md)
- [x] [监控方案](./operations/monitoring.md)
- [x] [故障排查](./operations/troubleshooting.md)

### 测试工程师
- [x] [测试计划](./testing/test-plan.md)
- [x] [测试策略](./testing/test-strategy.md)
- [x] [各类测试指南](./testing/)

---

## 📌 重要说明

### 技术栈标注

文档中使用以下标注：
- ⭐ **技术栈变更** - 表示该模块采用Rust开发（原Python）
- ⭐ **新增功能** - 表示新增的功能需求（如美股、期权、舆情数据）
- 🌟 - 表示核心文档，建议优先阅读
- 📋 - 表示规划中的功能
- 🚧 - 表示开发中的功能
- ✅ - 表示已完成的功能

### 文档版本控制

- 所有文档版本号与项目版本号同步（当前v2.0.0）
- 重大变更会在文档顶部标注
- 文档变更需通过Pull Request审核

### 文档贡献

如需更新文档，请遵循以下流程：
1. 创建feature分支
2. 更新文档
3. 提交Pull Request
4. Code Review通过后合并

---

## 📞 联系方式

如有文档问题，请联系：
- **产品团队**: product@hermesflow.com
- **技术团队**: dev@hermesflow.com
- **GitHub Issues**: https://github.com/your-org/HermesFlow/issues

---

**文档维护者**: HermesFlow Documentation Team  
**最后更新**: 2024-12-20  
**文档版本**: v2.0.0

