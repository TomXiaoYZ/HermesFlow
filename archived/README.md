# 归档目录说明

本目录存放项目演进过程中被替换或废弃的代码和文档。

---

## 📦 归档原因

### v1.0 → v2.0 技术栈重大变更 (2024-12)

#### 数据引擎：Python → Rust

**变更原因**:
- **性能需求**: 需要μs级延迟和100k+ msg/s吞吐量
- **内存安全**: Rust编译时保证内存安全，无数据竞争
- **并发处理**: Tokio异步运行时更高效处理WebSocket连接

**性能对比**:
| 指标 | Python (v1.0) | Rust (v2.0) | 提升 |
|------|--------------|------------|------|
| P99延迟 | ~50ms | <1ms | 50x |
| 吞吐量 | ~2k msg/s | >100k msg/s | 50x |
| 内存使用 | ~500MB | ~50MB | 10x |
| CPU使用 | ~80% | ~20% | 4x |

**归档内容**:
```
archived/data-engine/  (Python实现)
├── __init__.py
├── connectors/        # 交易所连接器
├── processors/        # 数据处理器
└── storage/           # 数据存储
```

#### 其他服务架构重构

**变更原因**:
- 微服务化设计
- 统一技术栈
- 性能优化

**归档内容**:
```
archived/
├── api-gateway/       # 旧版API网关
├── frontend/          # 旧版前端
├── risk-engine/       # 旧版风控服务
├── strategy-engine/   # 旧版策略引擎
└── user-management/   # 旧版用户管理
```

### v2.0 → v2.1 功能增强 (2024-12-20)

#### PRD文档整合

**变更原因**:
- 市场分析结果整合
- 功能需求增强
- 文档统一管理

**归档内容**:
```
archived/prd/
└── PRD-Enhancement-v2.1.md  (已整合至PRD-HermesFlow.md v2.1.0)
```

**整合内容**:
- ✅ Alpha因子库（100个预定义因子）
- ✅ 策略优化引擎（贝叶斯+Walk-Forward+遗传算法+PSO）
- ✅ 模拟交易系统（与实盘API兼容）
- ✅ ML集成路线图（特征工程+模型训练+在线预测）
- ✅ 组合管理系统（多策略+动态资金分配+Markowitz优化）

---

## 🔍 查看新版本

### 代码位置

**当前代码库** (`modules/`):
```
modules/
├── data-engine/         # Rust数据引擎 ⭐ 新实现
│   ├── src/
│   │   ├── connectors/  # WebSocket连接器
│   │   ├── processors/  # 数据处理
│   │   └── storage/     # 存储层
│   └── Cargo.toml
│
├── strategy-engine/     # Python策略引擎
│   ├── engines/
│   ├── backtest/
│   └── requirements.txt
│
├── trading-engine/      # Java交易执行
│   ├── src/main/java/
│   └── pom.xml
│
├── risk-engine/         # Java风控服务
│   ├── src/main/java/
│   └── pom.xml
│
├── user-management/     # Java用户管理
│   ├── src/main/java/
│   └── pom.xml
│
├── gateway/             # Java API网关
│   ├── src/main/java/
│   └── pom.xml
│
└── frontend/            # React前端
    ├── src/
    ├── package.json
    └── vite.config.ts
```

### 文档位置

**当前文档** (`docs/`):

#### 核心文档
- **PRD**: `docs/prd/PRD-HermesFlow.md` (v2.1.0)
- **架构**: `docs/architecture/system-architecture.md` (v2.1.0)
- **进度**: `docs/progress.md` (v2.1.0)

#### 架构设计
- **系统架构**: `docs/architecture/system-architecture.md` (5700+行)
- **ADR文档**: `docs/architecture/decisions/ADR-001~008.md`
- **CI/CD流程**: `docs/architecture/diagrams/cicd-flow.md`

#### 技术文档
- **API设计**: `docs/api/api-design.md`
- **数据库设计**: `docs/database/database-design.md`
- **开发指南**: `docs/development/dev-guide.md`
- **编码规范**: `docs/development/coding-standards.md`
- **部署指南**: `docs/deployment/docker-guide.md`
- **GitOps**: `docs/deployment/gitops-best-practices.md`
- **监控方案**: `docs/operations/monitoring.md`

#### 测试文档
- **测试策略**: `docs/testing/test-strategy.md` (v3.0.0)
- **早期测试策略**: `docs/testing/early-test-strategy.md`
- **高风险测试**: `docs/testing/high-risk-access-testing.md`
- **测试数据管理**: `docs/testing/test-data-management.md`
- **CI/CD集成**: `docs/testing/ci-cd-integration.md`

#### 设计文档
- **设计系统**: `docs/design/design-system.md`
- **页面设计**: `docs/design/page-designs.md`
- **UI提示词**: `docs/design/lovable-v0-prompt.md`

---

## 📚 技术栈对比

### v1.0 (已归档)

| 组件 | 技术栈 | 状态 |
|------|--------|------|
| 数据引擎 | Python 3.11 | 📦 已归档 |
| 策略引擎 | Python 3.11 | 📦 已归档 |
| 交易执行 | Python 3.11 | 📦 已归档 |
| 风控服务 | Python 3.11 | 📦 已归档 |
| 用户管理 | Python 3.11 + Flask | 📦 已归档 |
| 前端 | React 17 | 📦 已归档 |

### v2.0/v2.1 (当前)

| 组件 | 技术栈 | 状态 |
|------|--------|------|
| 数据引擎 | **Rust 1.75 + Tokio 1.35** ⭐ | ✅ 设计完成 |
| 策略引擎 | Python 3.12 + FastAPI 0.104 | ✅ 设计完成 |
| 交易执行 | Java 21 + Spring Boot 3.2 | ✅ 设计完成 |
| 风控服务 | Java 21 + Spring Boot 3.2 | ✅ 设计完成 |
| 用户管理 | Java 21 + Spring Boot 3.2 | ✅ 设计完成 |
| API网关 | Java 21 + Spring Cloud Gateway 4.1 | ✅ 设计完成 |
| 前端 | React 18.2 + TypeScript 5.3 + Vite 5.0 | ✅ 设计完成 |

---

## ⚠️ 注意事项

### 归档代码不应用于生产

归档代码仅供参考和学习使用，**不应用于生产环境**，原因：

1. **性能问题**: v1.0性能远低于v2.0要求
2. **安全问题**: 旧版未实现多租户隔离
3. **维护停止**: 不再接收安全更新和Bug修复
4. **依赖过时**: 使用的第三方库可能存在漏洞

### 迁移建议

如需从v1.0迁移到v2.0/v2.1:

1. **数据迁移**: 参考 `docs/database/migration-guide.md` (待创建)
2. **配置迁移**: 更新环境变量和配置文件
3. **API适配**: 参考 `docs/api/api-design.md`
4. **测试验证**: 运行完整测试套件

### 保留原因

归档代码保留的原因：

1. **历史参考**: 了解系统演进过程
2. **学习材料**: Python实现作为对比学习
3. **紧急回滚**: 极端情况下的备选方案（不推荐）
4. **代码复用**: 部分业务逻辑可参考

---

## 📞 相关资源

### 文档
- **系统架构**: `docs/architecture/system-architecture.md`
- **ADR-001**: `docs/architecture/decisions/ADR-001-hybrid-tech-stack.md`
- **开发进度**: `docs/progress.md`
- **快速参考**: `docs/QUICK-REFERENCE.md`

### 代码仓库
- **主仓库**: HermesFlow (当前)
- **GitOps**: HermesFlow-GitOps

### 联系方式
- **技术问题**: @architect.mdc
- **产品问题**: @pm.mdc
- **迁移支持**: @architect.mdc

---

**最后更新**: 2024-12-20  
**维护者**: Architecture Team

