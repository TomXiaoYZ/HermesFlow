# HermesFlow 常见问题解答（FAQ）

> **快速查找常见问题的答案** | **最后更新**: 2025-01-13

---

## 📋 目录

1. [新手入门](#新手入门)
2. [开发环境](#开发环境)
3. [编码和开发](#编码和开发)
4. [测试](#测试)
5. [部署和运维](#部署和运维)
6. [性能优化](#性能优化)
7. [故障排查](#故障排查)
8. [文档和流程](#文档和流程)

---

## 新手入门

### Q1: 我是新加入的开发者，应该从哪里开始？

**A**: 按以下顺序开始：

1. ✅ 阅读 [快速开始指南](./quickstart.md)（5分钟）
2. ✅ 根据技术栈选择开发者指南：
   - [Rust 开发者指南](./development/rust-developer-guide.md)
   - [Java 开发者指南](./development/java-developer-guide.md)
   - [Python 开发者指南](./development/python-developer-guide.md)
3. ✅ 搭建本地开发环境（30-40分钟）
4. ✅ 运行所有测试，确保环境正常
5. ✅ 找一个 "good first issue" 开始第一个任务

**相关文档**:
- [快速开始指南](./quickstart.md)
- [文档导航](./README.md)

---

### Q2: 如何找到我需要的文档？

**A**: 使用 [文档导航中心](./README.md)，提供三种浏览方式：

- 📅 **按角色浏览**: Product Manager, Developer, QA, DevOps 等
- 📋 **按开发周期浏览**: Sprint Planning, Development, Testing, Deployment 等
- 📚 **按文档类型浏览**: PRD, 架构, API, 测试文档等

也可以使用 [文档流程图](./document-flow.md) 按场景导航。

---

### Q3: 项目的技术栈是什么？

**A**: HermesFlow 采用混合技术栈：

| 模块 | 语言 | 主要框架 |
|------|------|---------|
| 数据引擎 | **Rust 1.75** | Tokio, Actix-web, Rayon |
| 策略引擎 | **Python 3.12** | FastAPI, NumPy, Pandas |
| 交易/用户/风控 | **Java 21** | Spring Boot 3.2, Virtual Threads |
| 前端 | **TypeScript 5.3** | React 18, Vite 5, TailwindCSS |

**数据存储**:
- PostgreSQL 15（主数据库，RLS 多租户）
- ClickHouse 23.8（分析数据库）
- Redis 7（缓存）
- Kafka 7.5（消息队列）

**相关文档**: [系统架构](./architecture/system-architecture.md)

---

## 开发环境

### Q4: Docker Compose 启动失败，提示端口已被占用？

**A**: 检查并释放被占用的端口：

```bash
# 查找占用端口的进程
lsof -i :5432  # PostgreSQL
lsof -i :6379  # Redis
lsof -i :8123  # ClickHouse
lsof -i :9092  # Kafka

# 停止占用的进程
kill -9 <PID>

# 或者修改 docker-compose.yml 中的端口映射
```

**相关文档**: [故障排查手册](./operations/troubleshooting.md#docker-相关问题)

---

### Q5: Rust 编译很慢，如何加速？

**A**: 使用以下优化：

```bash
# 1. 使用 sccache 缓存编译
cargo install sccache
export RUSTC_WRAPPER=sccache

# 2. 使用 mold 链接器（Linux）
sudo apt install mold
echo "[build]\nlinker = \"clang\"\nrustflags = [\"-C\", \"link-arg=-fuse-ld=mold\"]" >> ~/.cargo/config.toml

# 3. 增加并行编译任务数
echo "[build]\njobs = 8" >> ~/.cargo/config.toml
```

---

### Q6: Java 项目启动时内存不足（OOM）？

**A**: 增加 JVM 堆内存：

```bash
# 方式 1: 环境变量
export MAVEN_OPTS="-Xmx2048m"
./mvnw spring-boot:run

# 方式 2: application.yml
# (已配置 Virtual Threads，通常不需要大量堆内存)
```

---

### Q7: Python 依赖安装失败或冲突？

**A**: 重新创建虚拟环境：

```bash
# 删除现有环境
rm -rf .venv poetry.lock

# 重新安装
poetry install

# 如果仍有问题，更新 Poetry
curl -sSL https://install.python-poetry.org | python3 -
```

---

## 编码和开发

### Q8: 代码提交规范是什么？

**A**: 使用 **Conventional Commits** 规范：

```
<type>(<scope>): <subject>

<body>

<footer>
```

**Type**:
- `feat`: 新功能
- `fix`: Bug 修复
- `docs`: 文档更新
- `test`: 测试相关
- `refactor`: 代码重构
- `perf`: 性能优化
- `chore`: 构建/工具链更新

**示例**:
```bash
git commit -m "feat(data-engine): add Binance WebSocket connector

- Implement real-time market data subscription
- Add reconnection logic with exponential backoff
- Add unit tests with 90% coverage

Closes #123"
```

**相关文档**: [开发指南 - Git 工作流](./development/dev-guide.md#git-工作流)

---

### Q9: 如何进行 Code Review？

**A**: 

**作为提交者**:
1. 提交 PR 前，使用 [代码审查清单](./development/code-review-checklist.md) 自查
2. 运行 Linter 和测试
3. 确保 CI/CD 通过
4. 填写 PR 描述模板

**作为审查者**:
1. 使用 [代码审查清单](./development/code-review-checklist.md) 逐项检查
2. 在 4 小时内提供第一次反馈（团队目标）
3. 提供建设性意见

**相关文档**: [代码审查清单](./development/code-review-checklist.md)

---

### Q10: 多租户隔离如何实现？

**A**: HermesFlow 使用多层隔离策略：

1. **PostgreSQL RLS (Row-Level Security)**:
   ```sql
   -- 自动添加 tenant_id 过滤
   CREATE POLICY tenant_isolation ON orders
   FOR ALL TO PUBLIC
   USING (tenant_id = current_setting('app.current_tenant')::TEXT);
   ```

2. **Redis Key 前缀**:
   ```rust
   // 自动添加租户前缀
   let key = format!("tenant:{}:user:{}", tenant_id, user_id);
   ```

3. **Kafka Topic 分区**:
   - 按 tenant_id 分区

4. **应用层验证**:
   - JWT Token 包含 tenant_id
   - 每次请求验证租户

**相关文档**: 
- [ADR-002: 多租户架构](./architecture/decisions/ADR-002-multi-tenancy-architecture.md)
- [高风险访问测试](./testing/high-risk-access-testing.md)

---

## 测试

### Q11: 测试覆盖率要求是多少？

**A**: 

| 语言 | 最低覆盖率 | 推荐覆盖率 |
|------|-----------|-----------|
| **Rust** | 85% | 90%+ |
| **Java** | 80% | 85%+ |
| **Python** | 75% | 80%+ |

**检查覆盖率**:
```bash
# Rust
cargo tarpaulin --out Html

# Java
./mvnw jacoco:report

# Python
poetry run pytest --cov=src --cov-report=html
```

**相关文档**: [测试策略](./testing/test-strategy.md)

---

### Q12: 如何运行特定的测试？

**A**:

```bash
# Rust - 运行特定测试
cargo test test_function_name
cargo test --test integration_test

# Java - 运行特定测试类
./mvnw test -Dtest=UserServiceTest

# Python - 运行特定测试文件
poetry run pytest tests/test_strategy.py
poetry run pytest tests/test_strategy.py::test_specific_function
```

---

### Q13: 集成测试如何访问数据库？

**A**: 使用 `docker-compose.test.yml` 启动测试环境：

```bash
# 启动测试环境
docker-compose -f docker-compose.test.yml up -d

# 运行集成测试
cargo test --test integration_test  # Rust
./mvnw verify                        # Java
poetry run pytest tests/integration/  # Python

# 清理
docker-compose -f docker-compose.test.yml down -v
```

**相关文档**: [测试数据管理](./testing/test-data-management.md)

---

### Q14: 如何 Mock 外部服务（如 Binance API）？

**A**:

**Rust**:
```rust
#[cfg(test)]
mod tests {
    use mockito::{mock, server_url};
    
    #[tokio::test]
    async fn test_fetch_market_data() {
        let _m = mock("GET", "/api/v3/klines")
            .with_status(200)
            .with_body(r#"[...]"#)
            .create();
        
        let connector = BinanceConnector::new(&server_url());
        let data = connector.fetch_candles("BTC/USDT", "1m").await;
        
        assert!(data.is_ok());
    }
}
```

**相关文档**: [测试策略 - Mock 外部服务](./testing/test-strategy.md#mock-外部服务)

---

## 部署和运维

### Q15: 如何部署到 Dev 环境？

**A**: HermesFlow 使用 GitOps 流程：

1. **推送代码到 GitHub**
2. **GitHub Actions 自动构建**:
   - 运行测试
   - 构建 Docker 镜像
   - 推送到 Azure ACR
3. **触发 GitOps 仓库更新**
4. **ArgoCD 自动同步到 Kubernetes**

**手动触发**:
```bash
# Tag 推送触发构建
git tag data-engine-v1.2.3
git push origin data-engine-v1.2.3
```

**相关文档**: 
- [CI/CD 架构](./architecture/system-architecture.md#第11章-cicd架构)
- [GitOps 最佳实践](./deployment/gitops-best-practices.md)

---

### Q16: 如何查看服务日志？

**A**:

```bash
# Kubernetes
kubectl logs -f deployment/data-engine -n hermesflow-dev

# Docker Compose（本地）
docker-compose logs -f data-engine

# 查看最近 100 行
kubectl logs --tail=100 deployment/data-engine -n hermesflow-dev
```

---

### Q17: 如何回滚到上一个版本？

**A**:

**方式 1: ArgoCD UI**
- 打开 ArgoCD
- 选择应用
- 点击 "History"
- 选择上一个版本
- 点击 "Rollback"

**方式 2: kubectl**
```bash
# 查看部署历史
kubectl rollout history deployment/data-engine -n hermesflow-dev

# 回滚到上一个版本
kubectl rollout undo deployment/data-engine -n hermesflow-dev

# 回滚到特定版本
kubectl rollout undo deployment/data-engine --to-revision=2 -n hermesflow-dev
```

**相关文档**: [GitOps 最佳实践 - 回滚策略](./deployment/gitops-best-practices.md#回滚策略)

---

## 性能优化

### Q18: API 响应时间过长，如何优化？

**A**: 

**1. 识别瓶颈**:
```bash
# 查看 Prometheus 指标
# P95 延迟: http_request_duration_seconds{quantile="0.95"}

# 查看慢查询日志（PostgreSQL）
```

**2. 常见优化**:
- ✅ 添加数据库索引
- ✅ 使用 Redis 缓存
- ✅ 优化 N+1 查询（使用 JOIN FETCH）
- ✅ 启用 gzip 压缩
- ✅ 使用分页

**相关文档**: [故障排查手册 - 性能诊断](./operations/troubleshooting.md#性能诊断)

---

### Q19: 数据库查询慢，如何优化？

**A**:

**1. 分析查询**:
```sql
-- PostgreSQL: 查看执行计划
EXPLAIN ANALYZE SELECT * FROM orders WHERE tenant_id = '123';

-- ClickHouse: 查看查询性能
SYSTEM FLUSH LOGS;
SELECT query, query_duration_ms FROM system.query_log ORDER BY query_duration_ms DESC LIMIT 10;
```

**2. 优化建议**:
- ✅ 添加索引（特别是 tenant_id 和常用查询字段）
- ✅ 使用 LIMIT
- ✅ 避免 SELECT *
- ✅ 使用分区表（ClickHouse）

**相关文档**: [数据库设计](./database/database-design.md)

---

## 故障排查

### Q20: 服务启动失败，如何排查？

**A**:

**步骤 1: 查看日志**
```bash
# Kubernetes
kubectl logs deployment/data-engine -n hermesflow-dev

# Docker
docker-compose logs data-engine
```

**步骤 2: 检查配置**
- 环境变量是否正确
- 数据库连接字符串是否正确
- Redis/Kafka 地址是否正确

**步骤 3: 检查依赖服务**
```bash
# 检查 Pod 状态
kubectl get pods -n hermesflow-dev

# 检查服务健康
kubectl get svc -n hermesflow-dev
```

**相关文档**: [故障排查手册](./operations/troubleshooting.md)

---

### Q21: 数据库连接池耗尽？

**A**:

**症状**: `could not get connection from pool` 错误

**排查**:
```bash
# 查看当前连接数
SELECT count(*) FROM pg_stat_activity WHERE datname = 'hermesflow';

# 查看连接详情
SELECT pid, usename, application_name, client_addr, state 
FROM pg_stat_activity 
WHERE datname = 'hermesflow';
```

**解决**:
1. 增加连接池大小（application.yml 或 config.toml）
2. 检查是否有连接泄漏（未关闭的连接）
3. 优化长查询

---

### Q22: Kafka 消费延迟高？

**A**:

**排查**:
```bash
# 查看消费者组延迟
kafka-consumer-groups.sh --bootstrap-server localhost:9092 \
  --group hermesflow-strategy --describe
```

**解决**:
1. ✅ 增加消费者实例（并行处理）
2. ✅ 增加分区数
3. ✅ 优化消息处理逻辑
4. ✅ 批量处理消息

---

## 文档和流程

### Q23: 如何参加 Sprint Planning？

**A**: 

**会前准备**（作为开发者）:
- 审查 Sprint Backlog
- 理解 Story 需求
- 准备估算（使用 Planning Poker）

**会议中**:
- 参与任务分解
- 提供技术估算
- 识别技术风险和依赖

**相关文档**: [Sprint Planning 清单](./scrum/sprint-planning-checklist.md)

---

### Q24: 如何更新文档？

**A**:

1. **找到对应的文档**（使用 [文档导航](./README.md)）
2. **编辑 Markdown 文件**
3. **提交 PR**:
   ```bash
   git checkout -b docs/update-api-doc
   git add docs/api/api-design.md
   git commit -m "docs(api): update REST API examples"
   git push origin docs/update-api-doc
   ```
4. **请求 Review**（文档维护者）

**文档维护责任**:
- PRD: @pm.mdc
- 架构: @architect.mdc
- 测试: @qa.mdc
- 运维: @architect.mdc

---

### Q25: 在哪里可以找到最新的项目进度？

**A**: 

查看 [项目进度文档](./progress.md)，包含：
- 当前 Sprint 目标
- 已完成的里程碑
- Q1 2025 路线图
- 技术债务追踪
- 关键指标

每周更新一次。

---

## 📞 还有其他问题？

### 寻求帮助

- **技术问题**: Slack `#hermesflow-dev`
- **流程问题**: 联系 Scrum Master (@pm.mdc)
- **文档问题**: 查看 [文档导航](./README.md)
- **紧急问题**: [故障排查手册](./operations/troubleshooting.md)

### 反馈

如果您发现常见问题未在此列出，请：
1. 在 Slack `#hermesflow-dev` 提问
2. 问题解决后，提交 PR 更新本 FAQ

---

**最后更新**: 2025-01-13  
**维护者**: @pm.mdc  
**版本**: v1.0

