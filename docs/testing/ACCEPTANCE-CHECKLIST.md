# 验收测试清单

> **确保功能满足验收标准的完整检查清单**

---

## 🎯 使用时机

- ✅ 功能开发完成，准备提交测试
- ✅ QA 进行验收测试前
- ✅ Sprint Review Demo 前
- ✅ 发布到生产环境前

---

## 📋 通用验收标准

### 1. 功能完整性

- [ ] **所有用户故事的验收标准都已实现**
  - 逐条检查 PRD 中的验收标准
  - 所有功能点都可演示
  
- [ ] **边界条件处理正确**
  - 空值/null/None 情况
  - 边界值（最小值、最大值）
  - 极端情况（0、负数、超大数）
  
- [ ] **错误处理完善**
  - 提供有意义的错误消息
  - 不暴露敏感信息（如堆栈跟踪）
  - 优雅降级

### 2. 用户体验

- [ ] **界面符合设计规范**
  - 参考 [设计系统](../design/design-system.md)
  - 颜色、字体、间距一致
  
- [ ] **交互流畅**
  - 无卡顿
  - 加载状态（Loading Spinner）
  - 操作反馈（成功/失败提示）
  
- [ ] **响应式设计**（前端）
  - 桌面端（≥1280px）
  - 平板端（768px - 1279px）
  - 移动端（< 768px）

### 3. 性能要求

- [ ] **响应时间达标**
  - API 响应时间：P95 < 500ms
  - 页面加载时间：< 3秒
  - 数据处理吞吐量：符合基线（如数据引擎 10万行/秒）
  
- [ ] **资源使用合理**
  - CPU 使用率 < 80%
  - 内存使用稳定（无内存泄漏）
  - 数据库连接池不耗尽
  
- [ ] **并发处理**
  - 支持预期的并发用户数
  - 无数据竞争和死锁

### 4. 安全要求

- [ ] **认证和授权**
  - JWT Token 验证正确
  - RBAC 权限检查正确
  - 未授权访问被拒绝（401/403）
  
- [ ] **多租户隔离**
  - PostgreSQL RLS 生效
  - Redis Key 使用租户前缀
  - 用户只能访问自己租户的数据
  
- [ ] **输入验证**
  - SQL 注入防护
  - XSS 防护
  - CSRF 防护（如适用）
  
- [ ] **敏感信息保护**
  - 密码不在日志中显示
  - API 响应不包含敏感信息
  - HTTPS 加密传输

### 5. 数据一致性

- [ ] **数据正确性**
  - 创建、读取、更新、删除操作正确
  - 数据格式符合规范
  - 时区处理正确（UTC）
  
- [ ] **事务完整性**
  - 事务边界正确
  - 回滚正确处理
  - 无数据不一致
  
- [ ] **数据库约束**
  - 主键、外键、唯一约束生效
  - 非空约束生效

### 6. 集成和兼容性

- [ ] **API 集成正确**
  - 与上游服务集成正常
  - 与下游服务集成正常
  - API 契约符合规范（OpenAPI/gRPC）
  
- [ ] **第三方服务集成**
  - Binance API 集成正常
  - OKX API 集成正常
  - 其他外部 API 集成正常
  
- [ ] **浏览器兼容**（前端）
  - Chrome（最新版）
  - Firefox（最新版）
  - Safari（最新版）
  - Edge（最新版）

---

## 🧪 测试层级检查

### Level 1: 单元测试

- [ ] **单元测试通过**
  ```bash
  cargo test               # Rust
  ./mvnw test             # Java
  poetry run pytest       # Python
  ```
  
- [ ] **覆盖率达标**
  - Rust: ≥ 85%
  - Java: ≥ 80%
  - Python: ≥ 75%
  
- [ ] **测试质量**
  - 测试关键路径
  - 测试边界条件
  - 使用 Arrange-Act-Assert 模式

### Level 2: 集成测试

- [ ] **集成测试通过**
  - 数据库集成测试
  - Redis 集成测试
  - Kafka 集成测试
  - 第三方 API 集成测试（使用 Mock）
  
- [ ] **跨模块集成测试**
  - Rust 数据引擎 ↔ Python 策略引擎
  - Python 策略引擎 ↔ Java 交易引擎
  - Java 交易引擎 ↔ Java 风控引擎

### Level 3: 端到端测试（E2E）

- [ ] **关键用户流程可端到端执行**
  - 用户注册 → 登录 → 创建策略 → 回测 → 查看结果
  - 用户登录 → 下单 → 查看持仓 → 查看订单历史
  
- [ ] **跨服务流程正常**
  - 数据采集 → 数据处理 → 存储 → 策略使用
  - 策略信号 → 订单生成 → 风控检查 → 订单执行

### Level 4: 性能测试

- [ ] **性能基线达标**
  - 使用 k6 进行负载测试
  - 参考 [性能测试脚本](../../tests/performance/load_test.js)
  
- [ ] **性能指标**
  ```javascript
  // k6 thresholds
  http_req_duration: ['p(95)<500'],  // P95 < 500ms
  http_req_failed: ['rate<0.01'],    // 错误率 < 1%
  ```
  
- [ ] **无性能回归**
  - 与上个版本对比
  - 性能指标不下降

### Level 5: 安全测试

- [ ] **SQL 注入测试**
  - 参考 [SQL 注入测试](../../tests/security/test_sql_injection.py)
  - 尝试注入各种 SQL payload
  
- [ ] **XSS 测试**
  - 参考 [XSS 测试](../../tests/security/test_xss.py)
  - 尝试注入各种 XSS payload
  
- [ ] **认证测试**
  - 无 Token 访问被拒绝
  - 过期 Token 被拒绝
  - 伪造 Token 被拒绝
  
- [ ] **RBAC 测试**
  - 参考 [RBAC 测试](../../tests/security/test_rbac.py)
  - 普通用户无法访问管理员功能
  - 用户只能访问自己的资源
  
- [ ] **多租户隔离测试**
  - 参考 [租户隔离测试](../../tests/security/test_tenant_isolation.py)
  - 租户 A 无法访问租户 B 的数据
  - PostgreSQL RLS 策略生效
  - Redis Key 隔离生效

---

## 🔐 高风险访问点测试

### 数据库访问

- [ ] **PostgreSQL RLS 测试**
  - 参考 [高风险访问测试](../testing/high-risk-access-testing.md#postgresql-rls-测试)
  - 15+ 测试用例
  
- [ ] **ClickHouse 隔离测试**
  - 租户数据隔离
  - 查询性能
  
- [ ] **Redis Key 隔离测试**
  - Key 前缀正确
  - 租户数据隔离

### API 安全

- [ ] **JWT Token 验证测试**
  - 10+ 测试用例
  - 参考 [高风险访问测试](../testing/high-risk-access-testing.md#jwt-token-验证)
  
- [ ] **RBAC 权限测试**
  - 12+ 测试用例
  - 不同角色的权限边界

### 外部服务集成

- [ ] **外部服务故障容错测试**
  - Binance API 超时处理
  - Binance API 限流处理
  - 网络故障重试机制

---

## 📦 部署验收

### 构建和部署

- [ ] **Docker 镜像构建成功**
  ```bash
  # 检查镜像
  docker images | grep hermesflow
  ```
  
- [ ] **镜像推送到 ACR 成功**
  ```bash
  # 检查 ACR
  az acr repository list --name hermesflowacr
  ```
  
- [ ] **Helm Chart 正确**
  ```bash
  # Lint Helm Chart
  helm lint HermesFlow-GitOps/apps/dev/data-engine/
  ```

### 环境验证

- [ ] **Dev 环境部署成功**
  ```bash
  # 检查 Pod 状态
  kubectl get pods -n hermesflow-dev
  
  # 检查服务健康
  kubectl get svc -n hermesflow-dev
  ```
  
- [ ] **健康检查通过**
  ```bash
  # 数据引擎
  curl http://data-engine.hermesflow-dev.svc.cluster.local:8081/health
  
  # 交易引擎
  curl http://trading-engine.hermesflow-dev.svc.cluster.local:8083/actuator/health
  ```
  
- [ ] **烟雾测试通过**
  - 核心 API 可访问
  - 数据库连接正常
  - Redis 连接正常
  - Kafka 连接正常

### 监控和日志

- [ ] **Prometheus 指标采集正常**
  ```bash
  # 检查 Metrics
  curl http://data-engine.hermesflow-dev.svc.cluster.local:8081/metrics
  ```
  
- [ ] **Grafana 仪表盘显示正常**
  - 打开 Grafana
  - 检查 HermesFlow Dashboard
  - 数据更新正常
  
- [ ] **日志输出正常**
  ```bash
  # 查看日志
  kubectl logs -f deployment/data-engine -n hermesflow-dev
  ```

---

## 📄 文档验收

### 代码文档

- [ ] **API 文档更新**
  - OpenAPI 规范更新（`docs/api/api-design.md`）
  - gRPC 协议定义更新
  - 示例代码更新
  
- [ ] **README 更新**
  - 新功能使用说明
  - 配置项说明
  - 依赖说明

### 用户文档

- [ ] **用户手册更新**（如适用）
  - 新功能使用指南
  - 截图或 GIF
  
- [ ] **FAQ 更新**（如适用）
  - 常见问题解答

### 变更日志

- [ ] **CHANGELOG.md 更新**
  ```markdown
  ## [v2.1.0] - 2025-01-13
  
  ### Added
  - Alpha 因子库核心功能
  - RSI、MACD、EMA 等 10+ 技术指标
  
  ### Changed
  - 优化数据采集性能（+30%）
  
  ### Fixed
  - 修复多租户查询 bug (#123)
  ```

---

## ✅ 最终验收会签

### 验收参与者

- [ ] **开发者确认**
  - 所有功能已实现
  - 所有测试通过
  - 文档已更新
  
- [ ] **QA 确认**
  - 所有测试用例通过
  - 无阻塞性缺陷
  - 性能和安全测试通过
  
- [ ] **Product Owner 确认**
  - 功能符合预期
  - Demo 成功
  - 验收标准满足
  
- [ ] **DevOps 确认**
  - 部署成功
  - 监控正常
  - 无运维风险

### 验收决策

- [ ] **通过**: 所有检查项都满足 → 可以发布
- [ ] **有条件通过**: 有非阻塞性问题，但可以发布 → 记录技术债务
- [ ] **不通过**: 有阻塞性问题 → 修复后重新验收

---

## 📊 验收报告模板

```markdown
# 验收测试报告

## 基本信息
- **Sprint**: Sprint X
- **功能**: [功能名称]
- **Story ID**: STORY-XXX
- **测试日期**: 2025-01-13
- **测试人员**: [姓名]

## 验收结果
- **结果**: ✅ 通过 / ⚠️ 有条件通过 / ❌ 不通过
- **通过率**: X / Y 检查项通过

## 功能测试
- ✅ 功能 1
- ✅ 功能 2
- ❌ 功能 3（问题描述）

## 性能测试
- ✅ P95 响应时间: 420ms（< 500ms）
- ✅ 吞吐量: 12万行/秒（> 10万行/秒）

## 安全测试
- ✅ SQL 注入测试通过
- ✅ XSS 测试通过
- ✅ 多租户隔离测试通过

## 发现的问题
| ID | 问题描述 | 严重程度 | 状态 |
|----|---------|---------|------|
| BUG-01 | 错误消息不友好 | P2 | 待修复 |

## 建议
- [改进建议 1]
- [改进建议 2]

## 签字确认
- **开发者**: [姓名] - [日期]
- **QA**: [姓名] - [日期]
- **PO**: [姓名] - [日期]
```

---

## 📞 获取帮助

### 测试问题
- **QA Team**: Slack `#qa-team`
- **测试文档**: [测试策略](../testing/test-strategy.md)

### 部署问题
- **DevOps Team**: Slack `#devops`
- **部署文档**: [Docker 部署指南](../deployment/docker-guide.md)

### 一般问题
- **Scrum Master**: 联系 @pm.mdc
- **FAQ**: [常见问题](../FAQ.md)

---

**最后更新**: 2025-01-13  
**维护者**: @qa.mdc  
**版本**: v1.0

