# Sprint 1 Mid-Dev QA 检查 - 问题清单

**生成日期**: 2025-10-14  
**QA 工程师**: @qa.mdc  
**总问题数**: 8 个 (0 P0, 3 P1, 5 P2)  

---

## 🔴 P0 - 阻塞性问题 (0)

**无阻塞性问题** ✅

---

## 🟡 P1 - 重要问题 (3)

### Issue #1: 缺少成本监控配置

**ID**: SPRINT01-ISSUE-001  
**优先级**: P1 (High)  
**类别**: 监控  
**负责人**: DevOps Lead  
**预计工作量**: 1 小时  
**截止日期**: 2025-10-18  

**描述**:
未配置 Azure Cost Management 预算警报和成本监控，存在意外超支风险。

**影响**:
- 无法及时发现成本异常
- 可能导致预算超支
- 缺少成本趋势分析

**建议解决方案**:
1. 配置月度预算警报 ($1000, 80% 警告阈值)
2. 启用每日成本报告
3. 配置 Action Group 发送成本警报
4. 启用 Azure Advisor 成本建议

**验收标准**:
- [ ] 创建预算资源 `hermesflow-dev-budget`
- [ ] 配置 80% 和 100% 警告阈值
- [ ] 设置邮件通知到 devops@hermesflow.io
- [ ] 验证收到测试警报

**参考命令**:
```bash
az consumption budget create \
  --amount 1000 \
  --budget-name hermesflow-dev-budget \
  --resource-group hermesflow-dev-rg \
  --time-grain Monthly \
  --time-period start-date=2025-10-01 \
  --notification true 80 hermesflow-dev-action-group \
  --notification true 100 hermesflow-dev-action-group
```

---

### Issue #2: 冗余 GitHub Workflows

**ID**: SPRINT01-ISSUE-002  
**优先级**: P1 (Medium)  
**类别**: CI/CD  
**负责人**: DevOps Engineer  
**预计工作量**: 2 小时  
**截止日期**: 2025-10-21  

**描述**:
`.github/workflows/` 目录中发现 11 个 workflow 文件，但 Sprint 1 仅计划创建 7 个。以下 4 个文件用途不明：
- `deploy.yml`
- `main.yml`
- `module-cicd.yml`
- `test.yml`

**影响**:
- 可能导致团队混淆
- 增加维护成本
- 潜在冲突风险

**建议解决方案**:
1. 审查每个 workflow 的来源和用途
2. 删除不需要的测试/旧文件
3. 将有用但不活跃的 workflows 移动到 `.github/workflows/archive/`
4. 更新 `README.md` 列出活跃 workflows

**验收标准**:
- [ ] 审查 4 个未知 workflows
- [ ] 删除或归档不需要的文件
- [ ] 更新 `README.md` 的 Workflows 章节
- [ ] 在 PR 中说明每个文件的处理决定

---

### Issue #3: 测试执行不完整

**ID**: SPRINT01-ISSUE-003  
**优先级**: P1 (High)  
**类别**: 测试  
**负责人**: QA Lead  
**预计工作量**: 4 小时  
**截止日期**: 2025-10-18  

**描述**:
Sprint 1 测试计划包含 32 个测试用例，但仅执行了约 50%。以下测试未完成：
- AKS 集群连接测试 (需要 kubelogin)
- PostgreSQL 实际连接测试 (从 AKS Pod)
- ACR 推送/拉取测试
- CI/CD workflows 触发测试
- 性能基准测试
- 灾难恢复测试

**影响**:
- 潜在问题未被发现
- 无法确认端到端功能
- 缺少性能基线

**建议解决方案**:
1. 安装 kubelogin: `brew install Azure/kubelogin/kubelogin`
2. 连接 AKS 集群并验证 nodes
3. 部署测试 Pod 并测试 PostgreSQL 连接
4. 推送测试镜像到 ACR
5. 触发至少一个 CI workflow 并验证成功
6. 执行基础性能测试

**验收标准**:
- [ ] ✅ AKS 集群可访问 (kubectl get nodes)
- [ ] ✅ PostgreSQL 可从 AKS Pod 连接
- [ ] ✅ ACR 可推送和拉取镜像
- [ ] ✅ 至少触发并通过 1 个 CI workflow
- [ ] ✅ 记录性能基线（CPU, Memory, I/O）
- [ ] 更新测试执行记录到 `sprint-01-test-cases.md`

---

## 🟢 P2 - 次要问题 (5)

### Issue #4: NSG HTTP 规则过于宽松

**ID**: SPRINT01-ISSUE-004  
**优先级**: P2 (Medium)  
**类别**: 安全  
**负责人**: Security Engineer  
**预计工作量**: 30 分钟  
**截止日期**: 2025-10-25  

**描述**:
AKS NSG 的 HTTP 规则允许所有源 IP (`*`) 访问端口 80，存在潜在安全风险。

**当前配置**:
```
Rule: AllowHTTP
Priority: 110
Source: * (任意)
Destination Port: 80
```

**建议解决方案**:
修改 `infrastructure/terraform/modules/networking/main.tf`，将源限制为：
- 选项 1: 仅允许 AKS 子网 (`10.0.1.0/24`)
- 选项 2: 限制特定公网 IP 范围
- 选项 3: 完全禁用 HTTP，仅允许 HTTPS

**验收标准**:
- [ ] 修改 Terraform NSG 规则
- [ ] 执行 terraform plan 验证更改
- [ ] 执行 terraform apply
- [ ] 验证 HTTP 访问受限
- [ ] 文档化 NSG 规则决策

---

### Issue #5: 缺少 Terraform versions.tf

**ID**: SPRINT01-ISSUE-005  
**优先级**: P2 (Low)  
**类别**: 基础设施代码  
**负责人**: DevOps Engineer  
**预计工作量**: 1 小时  
**截止日期**: 2025-10-28  

**描述**:
所有 6 个 Terraform 模块均缺少 `versions.tf` 文件，未锁定 provider 版本，可能导致版本不一致。

**影响**:
- 不同环境可能使用不同 provider 版本
- 无法保证可重复构建
- 升级风险增加

**建议解决方案**:
为每个模块创建 `versions.tf`：

```hcl
# versions.tf
terraform {
  required_version = ">= 1.5.0"
  
  required_providers {
    azurerm = {
      source  = "hashicorp/azurerm"
      version = "~> 3.80.0"
    }
    random = {
      source  = "hashicorp/random"
      version = "~> 3.5.0"
    }
  }
}
```

**验收标准**:
- [ ] 为 6 个模块各创建 `versions.tf`
- [ ] 锁定 azurerm provider 版本
- [ ] 执行 terraform init 验证
- [ ] 更新模块 README 说明版本要求

---

### Issue #6: Key Vault Secrets 无轮换策略

**ID**: SPRINT01-ISSUE-006  
**优先级**: P2 (Medium)  
**类别**: 安全  
**负责人**: Security Engineer  
**预计工作量**: 2 小时  
**截止日期**: 2025-10-25  

**描述**:
Key Vault 中的 4 个 secrets (postgres-admin-password, jwt-secret, redis-password, encryption-key) 均未设置到期时间和轮换策略。

**影响**:
- 长期使用同一 secret 增加泄露风险
- 不符合安全最佳实践
- 缺少自动轮换机制

**建议解决方案**:
1. 设置 secrets 到期时间 (90 天)
2. 创建轮换流程文档
3. 配置到期前通知 (7 天)
4. 考虑使用 Azure Key Vault Secrets Rotation

**验收标准**:
- [ ] 为所有 secrets 设置 90 天到期时间
- [ ] 配置到期前 7 天通知
- [ ] 创建 secrets 轮换 runbook
- [ ] 测试 secret 更新流程
- [ ] 文档化轮换程序

---

### Issue #7: 缺少自动化测试脚本

**ID**: SPRINT01-ISSUE-007  
**优先级**: P2 (Medium)  
**类别**: 测试  
**负责人**: QA Engineer  
**预计工作量**: 8 小时  
**截止日期**: 2025-10-28  

**描述**:
虽然有详细的测试用例文档，但缺少自动化测试脚本，依赖手动验证。

**影响**:
- 测试效率低
- 难以持续验证
- 回归测试成本高

**建议解决方案**:
创建测试脚本目录结构：

```
tests/
├── infrastructure/
│   ├── test-azure-resources.sh      # 验证所有 Azure 资源
│   ├── test-network-connectivity.sh # 网络连接测试
│   ├── test-aks-cluster.sh          # AKS 功能测试
│   └── test-database.sh             # PostgreSQL 测试
├── integration/
│   ├── test-aks-to-postgres.sh      # 数据库连接
│   ├── test-aks-to-acr.sh           # 镜像拉取
│   └── test-keyvault-access.sh      # Secrets 访问
├── performance/
│   ├── benchmark-aks.sh              # AKS 性能基准
│   └── benchmark-postgres.sh         # 数据库性能基准
└── run-all-tests.sh                  # 测试入口脚本
```

**验收标准**:
- [ ] 创建测试目录结构
- [ ] 实现至少 5 个核心测试脚本
- [ ] 所有脚本包含错误处理和日志
- [ ] 创建 CI workflow 运行测试
- [ ] 文档化测试使用方法

---

### Issue #8: 监控告警规则不足

**ID**: SPRINT01-ISSUE-008  
**优先级**: P2 (Medium)  
**类别**: 监控  
**负责人**: DevOps Engineer  
**预计工作量**: 2 小时  
**截止日期**: 2025-10-25  

**描述**:
Log Analytics 仅配置了 saved searches，缺少实际的告警规则，无法主动发现问题。

**当前状态**:
- ✅ Log Analytics Workspace 已创建
- ✅ Container Insights 已启用
- ✅ 2 个 saved searches (HighCPUUsage, PodErrors)
- ❌ 无实际告警规则

**建议解决方案**:
添加以下告警规则到 `infrastructure/terraform/modules/monitoring/main.tf`：

1. **CPU 使用率告警** (> 80% for 5 minutes)
2. **内存使用率告警** (> 85% for 5 minutes)
3. **磁盘使用率告警** (> 90%)
4. **Pod 重启告警** (> 3 restarts in 10 minutes)
5. **节点不可用告警** (Node NotReady)
6. **PostgreSQL 连接失败告警**

**验收标准**:
- [ ] 创建至少 4 个 Alert Rules
- [ ] 配置告警发送到 Action Group
- [ ] 测试告警触发（模拟高 CPU）
- [ ] 验证邮件通知收到
- [ ] 文档化告警阈值和响应程序

---

## 📊 问题统计

### 按优先级

| 优先级 | 数量 | 百分比 |
|--------|------|--------|
| P0 (阻塞) | 0 | 0% |
| P1 (重要) | 3 | 37.5% |
| P2 (次要) | 5 | 62.5% |
| **总计** | **8** | **100%** |

### 按类别

| 类别 | 问题数 |
|------|--------|
| 监控 | 2 (#1, #8) |
| 测试 | 2 (#3, #7) |
| 安全 | 2 (#4, #6) |
| CI/CD | 1 (#2) |
| 基础设施代码 | 1 (#5) |

### 按负责人

| 负责人 | 问题数 | 问题 ID |
|--------|--------|---------|
| DevOps Engineer | 3 | #2, #5, #8 |
| QA Engineer/Lead | 2 | #3, #7 |
| Security Engineer | 2 | #4, #6 |
| DevOps Lead | 1 | #1 |

---

## 🎯 行动计划

### 本周必须完成 (2025-10-18 前)

- [ ] **Issue #1**: 配置成本监控 (1h)
- [ ] **Issue #3**: 完成剩余测试 (4h)

**总工作量**: 5 小时

### 下周完成 (2025-10-25 前)

- [ ] **Issue #2**: 清理冗余 workflows (2h)
- [ ] **Issue #4**: 收紧 NSG 规则 (30min)
- [ ] **Issue #6**: 配置 secrets 轮换 (2h)
- [ ] **Issue #8**: 添加告警规则 (2h)

**总工作量**: 6.5 小时

### 两周内完成 (2025-10-28 前)

- [ ] **Issue #5**: 添加 versions.tf (1h)
- [ ] **Issue #7**: 创建测试脚本 (8h)

**总工作量**: 9 小时

---

## 📋 跟踪

### 状态定义

- **🔴 Open**: 未开始
- **🟡 In Progress**: 进行中
- **🟢 Resolved**: 已解决
- **⚫ Closed**: 已关闭验证

### 当前状态 (2025-10-14)

| Issue ID | 标题 | 状态 | 进度 |
|----------|------|------|------|
| #1 | 成本监控 | 🔴 Open | 0% |
| #2 | 冗余 Workflows | 🔴 Open | 0% |
| #3 | 测试执行 | 🔴 Open | 0% |
| #4 | NSG 规则 | 🔴 Open | 0% |
| #5 | versions.tf | 🔴 Open | 0% |
| #6 | Secrets 轮换 | 🔴 Open | 0% |
| #7 | 测试脚本 | 🔴 Open | 0% |
| #8 | 告警规则 | 🔴 Open | 0% |

---

## 📝 更新日志

### 2025-10-14
- 初始问题清单创建
- 识别 8 个问题 (0 P0, 3 P1, 5 P2)
- 分配负责人和截止日期

---

**文档版本**: 1.0  
**最后更新**: 2025-10-14  
**文档位置**: `docs/qa/sprint-01-issues.md`  
**相关文档**: `docs/qa/sprint-01-mid-dev-qa-report.md`

