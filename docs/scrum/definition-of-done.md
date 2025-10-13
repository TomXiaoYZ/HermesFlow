# Definition of Done (DoD)

> **HermesFlow 项目完成标准** | **版本**: v1.0

---

## 📋 什么是 Definition of Done？

**Definition of Done (DoD)** 是团队对"完成"的共识。只有满足所有 DoD 标准的 Story 才能被认为是"Done"，才能在 Sprint Review 中 Demo，才能计入速度。

DoD 确保：
- ✅ 质量标准一致
- ✅ 减少技术债务
- ✅ 团队对"完成"有共同理解
- ✅ 减少后期返工

---

## 🎯 HermesFlow Definition of Done

### Story 级别 DoD

每个 Story 必须满足以下所有标准：

---

## 1. 代码层面

### 1.1 代码审查

- [ ] **至少 1 人 Code Review 通过**
  - Reviewer 必须是不同于作者的开发者
  - 所有 Review 意见已处理
  - Reviewer 明确批准（GitHub Approve）
  
- [ ] **代码符合编码规范**
  - 遵循 [编码规范](../development/coding-standards.md)
  - 命名清晰、一致
  - 代码可读性强
  - 适当的注释（复杂逻辑）

### 1.2 Linter 检查

- [ ] **Rust: `cargo clippy` 无警告**
  ```bash
  cargo clippy -- -D warnings
  ```
  - 通过标准: 0 warnings, 0 errors
  
- [ ] **Java: `checkstyle` 通过**
  ```bash
  ./mvnw checkstyle:check
  ```
  - 通过标准: 0 violations
  
- [ ] **Python: `pylint` 评分 ≥ 8.0**
  ```bash
  poetry run pylint src/ --fail-under=8.0
  ```
  - 通过标准: Score ≥ 8.0/10

### 1.3 安全扫描

- [ ] **Trivy 安全扫描无高危漏洞**
  ```bash
  trivy fs --severity HIGH,CRITICAL .
  ```
  - 允许: LOW, MEDIUM
  - 不允许: HIGH, CRITICAL
  
- [ ] **依赖安全检查**
  - Rust: `cargo audit`
  - Java: `mvn dependency:check`
  - Python: `safety check`

---

## 2. 测试层面

### 2.1 单元测试

- [ ] **单元测试通过**
  - Rust: `cargo test`
  - Java: `./mvnw test`
  - Python: `poetry run pytest`
  
- [ ] **覆盖率达标**
  - **Rust**: ≥ 85%
  - **Java**: ≥ 80%
  - **Python**: ≥ 75%
  
  **检查命令**:
  ```bash
  # Rust
  cargo tarpaulin --out Html --output-dir target/coverage
  
  # Java
  ./mvnw jacoco:report
  # 查看: target/site/jacoco/index.html
  
  # Python
  poetry run pytest --cov=src --cov-report=html
  # 查看: htmlcov/index.html
  ```

- [ ] **关键路径有测试**
  - Happy Path（正常流程）
  - Error Handling（错误处理）
  - Edge Cases（边界情况）

### 2.2 集成测试

- [ ] **集成测试通过**（如适用）
  - 跨模块集成测试
  - 数据库集成测试
  - API 集成测试
  - 第三方服务集成测试（使用 Mock）
  
  **何时需要集成测试**:
  - 新增 API 端点
  - 数据库 Schema 变更
  - 跨服务调用（Rust ↔ Java ↔ Python）
  - 外部服务集成（Binance, IBKR 等）

### 2.3 性能测试

- [ ] **性能测试通过**（如适用）
  - 性能基线达标
  - 无性能回归
  - 负载测试通过（如涉及高并发）
  
  **何时需要性能测试**:
  - 数据处理模块（吞吐量测试）
  - API 端点（延迟测试）
  - 数据库查询优化
  - 缓存策略
  
  **HermesFlow 性能基线**:
  - Data Service: 吞吐量 > 10万行/秒
  - API P95 延迟: < 500ms
  - API P99 延迟: < 1s

### 2.4 安全测试

- [ ] **安全测试通过**（如适用）
  - SQL 注入防护
  - XSS 防护
  - 认证和授权测试
  - RBAC 权限测试
  - 多租户隔离测试
  
  **何时需要安全测试**:
  - 涉及认证/授权
  - 涉及多租户数据
  - 用户输入处理
  - 敏感数据处理
  
  **参考**: [安全测试用例](../testing/high-risk-access-testing.md)

---

## 3. 文档层面

### 3.1 代码文档

- [ ] **公共 API 有文档注释**
  - Rust: `///` 文档注释
  - Java: JavaDoc
  - Python: Docstring
  
  **示例**:
  ```rust
  /// Calculates the RSI (Relative Strength Index) for the given price data.
  ///
  /// # Arguments
  ///
  /// * `prices` - A slice of closing prices
  /// * `period` - The RSI period (typically 14)
  ///
  /// # Returns
  ///
  /// A vector of RSI values (0-100)
  ///
  /// # Example
  ///
  /// ```
  /// let prices = vec![100.0, 102.0, 101.0, 103.0];
  /// let rsi = calculate_rsi(&prices, 14);
  /// ```
  pub fn calculate_rsi(prices: &[f64], period: usize) -> Vec<f64> {
      // ...
  }
  ```

### 3.2 API 文档

- [ ] **API 文档已更新**（如有新 API 或 API 变更）
  - REST API: OpenAPI / Swagger 规范更新
  - gRPC: `.proto` 文件更新
  - API 示例代码更新
  
  **文档位置**: `docs/api/api-design.md`

### 3.3 README 更新

- [ ] **README 已更新**（如有配置变更或依赖变更）
  - 新的环境变量说明
  - 新的依赖说明
  - 新的配置说明
  - 更新的构建/运行步骤
  
  **文档位置**: 各模块的 `README.md`

### 3.4 变更日志

- [ ] **CHANGELOG.md 已更新**
  - 新功能（Added）
  - Bug 修复（Fixed）
  - 变更（Changed）
  - 废弃（Deprecated）
  - 移除（Removed）
  
  **格式**:
  ```markdown
  ## [Unreleased]
  
  ### Added
  - RSI 因子计算功能 (#123)
  - MACD 因子计算功能 (#124)
  
  ### Fixed
  - 修复 Binance WebSocket 重连问题 (#125)
  
  ### Changed
  - 优化数据库连接池配置 (#126)
  ```

---

## 4. 部署层面

### 4.1 CI/CD Pipeline

- [ ] **GitHub Actions 所有检查通过**
  - ✅ Build 成功
  - ✅ Unit Tests 通过
  - ✅ Integration Tests 通过（如适用）
  - ✅ Security Tests 通过（如适用）
  - ✅ Code Quality 检查通过（Linter, Coverage）
  - ✅ Security Scan 通过（Trivy）
  
  **检查位置**: GitHub PR 页面

### 4.2 Docker 镜像

- [ ] **Docker 镜像构建成功**
  - 推送到 Azure Container Registry (ACR)
  - 镜像标签正确（使用 Git Tag）
  - 镜像大小合理（优化多阶段构建）
  
  **检查命令**:
  ```bash
  # 查看镜像
  az acr repository show-tags --name hermesflowregistry --repository data-engine
  ```

### 4.3 Helm Chart

- [ ] **Helm Chart 已更新**（如有配置变更）
  - `values.yaml` 更新
  - ConfigMap / Secret 更新
  - Deployment / Service 更新
  - 版本号递增
  
  **文档位置**: `HermesFlow-GitOps/apps/{env}/{module}/`

### 4.4 Dev 环境验证

- [ ] **在 Dev 环境部署并验证**
  - 部署成功（Pod Running）
  - 健康检查通过（/health 端点返回 200）
  - 烟雾测试通过（关键功能可用）
  - 日志无错误
  
  **检查命令**:
  ```bash
  # 检查 Pod 状态
  kubectl get pods -n hermesflow-dev -l app=data-engine
  
  # 检查健康
  kubectl exec -it <pod-name> -n hermesflow-dev -- curl localhost:8080/health
  
  # 查看日志
  kubectl logs -f deployment/data-engine -n hermesflow-dev --tail=100
  ```

---

## 5. 验收层面

### 5.1 验收标准

- [ ] **所有验收标准满足**
  - Story 中定义的所有 AC (Acceptance Criteria) 都已满足
  - 每个 AC 都有测试覆盖
  - 功能按预期工作

### 5.2 Product Owner 验收

- [ ] **Product Owner 验收通过**
  - PO 已审查功能
  - PO 已在 Dev 环境测试
  - PO 明确批准
  - Story 状态更新为 "Done"

### 5.3 Demo 准备

- [ ] **可以在 Sprint Review 中 Demo**
  - Demo 脚本已准备
  - Demo 数据已准备
  - Dev 环境稳定可用

---

## 📊 DoD 检查清单（快速参考）

### 开发完成后自查

```bash
# 代码层面
- [ ] Code Review 通过（至少 1 人）
- [ ] 编码规范符合（Linter 通过）
- [ ] 安全扫描通过（Trivy 无高危）

# 测试层面
- [ ] 单元测试通过（覆盖率达标）
- [ ] 集成测试通过（如适用）
- [ ] 性能测试通过（如适用）
- [ ] 安全测试通过（如适用）

# 文档层面
- [ ] 代码文档完整（公共 API）
- [ ] API 文档更新（如适用）
- [ ] README 更新（如适用）
- [ ] CHANGELOG 更新

# 部署层面
- [ ] CI/CD Pipeline 通过
- [ ] Docker 镜像构建成功
- [ ] Helm Chart 更新（如适用）
- [ ] Dev 环境验证通过

# 验收层面
- [ ] 验收标准满足
- [ ] Product Owner 验收通过
- [ ] 可以 Demo
```

---

## 🚨 DoD 豁免

在极少数情况下，如果某个 DoD 标准无法满足，需要：

1. **申请豁免**
   - 向 Scrum Master 和 Product Owner 说明原因
   - 评估风险和影响
   - 制定后续补救计划
   
2. **记录技术债务**
   - 在 `progress.md#技术债务` 中记录
   - 创建技术债务 Story
   - 纳入后续 Sprint 计划
   
3. **团队共识**
   - 团队投票同意豁免
   - 不能成为常态

**示例**:
```markdown
## 技术债务 - TD-123

**Story**: [STORY-ID] RSI 因子实现  
**DoD 豁免项**: 性能测试未完成  
**原因**: k6 测试环境暂时不可用  
**风险**: 中等 - 性能可能不达标  
**补救计划**: 下周完成 k6 环境搭建并补充性能测试  
**负责人**: @李明  
**截止日期**: 2025-01-20
```

---

## 📈 DoD 演进

DoD 不是一成不变的，应随项目成熟度提升：

### 当前 DoD（v1.0）

适用于 **Q1 2025 - MVP 阶段**

### 未来 DoD 演进计划

**Q2 2025（成熟期）**:
- [ ] 增加 E2E 测试要求
- [ ] 增加 Accessibility 测试
- [ ] 增加 Load Testing 要求
- [ ] 增加用户文档要求

**Q3 2025（规模化）**:
- [ ] 增加多环境验证（Dev + Staging）
- [ ] 增加 Canary 部署要求
- [ ] 增加监控和告警配置
- [ ] 增加 SLO/SLI 达标要求

---

## 💡 DoD 最佳实践

### 1. 可视化 DoD

在任务看板上显示 DoD 清单，确保团队始终记得：

```
┌─────────────────────────────────┐
│  STORY-123: RSI 因子实现          │
├─────────────────────────────────┤
│  DoD Progress: 8/10 (80%)       │
│  ✅ Code Review                  │
│  ✅ Linter                       │
│  ✅ Unit Tests                   │
│  ✅ Coverage (87%)               │
│  ✅ Integration Tests            │
│  ✅ API Doc                      │
│  ✅ CI/CD                        │
│  ✅ Dev Env                      │
│  ⏸️ Performance Test (待完成)    │
│  ⏸️ PO Approval (待验收)         │
└─────────────────────────────────┘
```

### 2. DoD 作为 PR Template

在 GitHub PR 模板中嵌入 DoD 清单：

```markdown
## DoD Checklist

### Code
- [ ] Code Review 通过
- [ ] Linter 通过
- [ ] Security Scan 通过

### Tests
- [ ] Unit Tests (覆盖率 ≥ 85%/80%/75%)
- [ ] Integration Tests（如适用）
- [ ] Performance Tests（如适用）
- [ ] Security Tests（如适用）

### Docs
- [ ] Code Documentation
- [ ] API Documentation（如适用）
- [ ] README更新（如适用）
- [ ] CHANGELOG更新

### Deployment
- [ ] CI/CD Pipeline 通过
- [ ] Docker Image 构建成功
- [ ] Dev 环境验证通过

### Acceptance
- [ ] 验收标准满足
- [ ] PO 验收通过
```

### 3. 自动化 DoD 检查

使用 CI/CD 自动化尽可能多的 DoD 检查：

```yaml
# .github/workflows/pr-check.yml
name: DoD Check

on: [pull_request]

jobs:
  dod-check:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v2
      
      # Linter
      - name: Run Linter
        run: cargo clippy -- -D warnings
      
      # Tests
      - name: Run Unit Tests
        run: cargo test
      
      # Coverage
      - name: Check Coverage
        run: cargo tarpaulin --out Xml
      - name: Upload Coverage
        uses: codecov/codecov-action@v2
      
      # Security
      - name: Security Scan
        run: trivy fs --severity HIGH,CRITICAL .
      
      # Build
      - name: Build Docker Image
        run: docker build -t test .
```

---

## 📚 相关资源

- [编码规范](../development/coding-standards.md)
- [测试策略](../testing/test-strategy.md)
- [代码审查清单](../development/code-review-checklist.md)
- [CI/CD 集成](../testing/ci-cd-integration.md)
- [高风险访问测试](../testing/high-risk-access-testing.md)

---

## 🎯 记住

> **"Done" means DONE!**
> 
> 如果一个 Story 不满足 DoD，它就不是 "Done"，不能在 Sprint Review 中 Demo，不能计入速度，不能发布到生产环境。
> 
> DoD 保护我们免于技术债务和质量问题。严格执行 DoD 是团队对质量的承诺。

---

**最后更新**: 2025-01-13  
**维护者**: @pm.mdc, @architect.mdc  
**版本**: v1.0  
**下次审查**: 2025-04-01（Q2 开始）

