# Story 3: ArgoCD GitOps 部署（成本优化版）

**Story ID**: DEVOPS-003  
**Epic**: DevOps Foundation  
**Priority**: P1 (High)  
**Estimate**: 8 Story Points (16 hours)  
**Sprint**: Sprint 1 (2025-01-10 ~ 2025-01-24)  
**Status**: ✅ Approved  
**Created**: 2025-10-14  
**Created By**: @sm.mdc  
**Validated By**: @po.mdc (2025-10-14)

---

## 📖 User Story

**作为** DevOps 工程师  
**我想要** 通过 Terraform 部署 ArgoCD 到 Dev AKS，代码在 GitOps 仓库管理  
**以便** 实现声明式 GitOps 工作流，自动化应用部署，同时保持成本最优（个人使用场景）

---

## 🎯 验收标准 (Acceptance Criteria)

### 1. ArgoCD 部署到现有 Dev AKS

```gherkin
Scenario: ArgoCD 成功部署到 Dev AKS
  Given Dev AKS 集群已运行（来自 DEVOPS-002）
  And Terraform 配置在 HermesFlow-GitOps 仓库
  When 执行 terraform apply
  Then 系统应该:
    - 在 argocd namespace 部署 ArgoCD
    - 资源占用 < 2GB RAM, < 1 CPU
    - 单副本配置（成本优化）
    - 禁用不需要的组件 (Dex, Notifications, ApplicationSet)
  And ArgoCD UI 可通过 port-forward 访问
  And Admin 密码存储在 Azure Key Vault
```

### 2. 代码架构分离

- [ ] Terraform 代码在 **HermesFlow-GitOps** 仓库
- [ ] 位置: `infrastructure/argocd/terraform/`
- [ ] 使用 Helm Provider 部署 ArgoCD Chart
- [ ] 连接配置通过环境变量传递（避免硬编码）
- [ ] 支持未来迁移到独立管理集群

### 3. 成本优化配置

```yaml
验收标准:
- ArgoCD Server: 1 副本, 100m CPU, 128Mi RAM
- Repo Server: 1 副本, 100m CPU, 128Mi RAM  
- Controller: 1 副本, 250m CPU, 256Mi RAM
- Redis: 50m CPU, 64Mi RAM
- 总资源占用: ~1 core CPU, ~1.5GB RAM
- 适配 Standard_B2s (2vCPU, 4GB) ✅
```

### 4. GitOps 仓库连接

- [ ] ArgoCD 连接到自身 GitOps 仓库
- [ ] 使用 GitHub PAT 或 Deploy Key 认证
- [ ] 配置默认 AppProject: `hermesflow`
- [ ] 支持管理多环境 (dev/main)

### 5. 访问和认证（个人简化版）

- [ ] Admin 密码存储在 Key Vault: `argocd-admin-password`
- [ ] 通过 kubectl port-forward 访问 UI
- [ ] 不需要 Ingress 配置
- [ ] 不需要 Azure AD 集成
- [ ] 不需要多用户 RBAC

### 6. 基础 Application 部署

- [ ] 创建示例 Application CRD (data-engine-dev)
- [ ] 自动同步策略 (automated sync)
- [ ] 支持 prune 和 selfHeal
- [ ] 命名空间自动创建

---

## 🔧 技术任务分解 (Technical Tasks)

### Task 3.0: [可选] AKS 成本优化 (2h)

**负责人**: DevOps Lead

**任务**: 将现有 AKS 降级到最便宜的 B 系列 VM

**文件修改**:
```hcl
# infrastructure/terraform/modules/aks/variables.tf
variable "system_node_pool_vm_size" {
  default = "Standard_B2s"  # 从 D4s_v3 降低
}

variable "system_node_pool_count" {
  default = 1  # 从 2 降低到 1
}

variable "user_node_pool_vm_size" {
  default = "Standard_B2ms"  # 从 D8s_v3 降低
}

variable "user_node_pool_min_count" {
  default = 0  # 可完全关闭
}
```

**验收**:
- [ ] Terraform plan 显示节点规格变更
- [ ] 执行 apply 后 AKS 成功调整
- [ ] 月成本从 $626 降到 ~$96 (节省 $530/月)
- [ ] 节点健康检查通过

---

### Task 3.1: 准备 HermesFlow-GitOps 仓库结构 (1h)

**负责人**: DevOps Lead

**创建目录结构**:
```bash
HermesFlow-GitOps/
├── infrastructure/
│   └── argocd/
│       ├── terraform/
│       │   ├── main.tf
│       │   ├── providers.tf
│       │   ├── variables.tf
│       │   ├── backend.tf
│       │   └── README.md
│       ├── values/
│       │   └── argocd-values.yaml
│       ├── README.md
│       ├── COST_OPTIMIZATION.md
│       └── MIGRATION_GUIDE.md
└── apps/
    ├── dev/
    └── main/
```

**验收**:
- [ ] 目录结构创建完成
- [ ] README.md 包含部署说明
- [ ] .gitignore 配置正确

---

### Task 3.2: 创建 ArgoCD Terraform 模块（低资源配置）(4h)

**负责人**: DevOps Engineer

**关键文件 1: providers.tf**
```hcl
terraform {
  required_version = ">= 1.5.0"
  
  required_providers {
    kubernetes = {
      source  = "hashicorp/kubernetes"
      version = "~> 2.23.0"
    }
    helm = {
      source  = "hashicorp/helm"
      version = "~> 2.11.0"
    }
  }
}

provider "kubernetes" {
  host                   = var.aks_host
  cluster_ca_certificate = base64decode(var.aks_ca_certificate)
  client_certificate     = base64decode(var.aks_client_certificate)
  client_key             = base64decode(var.aks_client_key)
}

provider "helm" {
  kubernetes {
    host                   = var.aks_host
    cluster_ca_certificate = base64decode(var.aks_ca_certificate)
    client_certificate     = base64decode(var.aks_client_certificate)
    client_key             = base64decode(var.aks_client_key)
  }
}
```

**关键文件 2: main.tf**
```hcl
resource "helm_release" "argocd" {
  name             = "argocd"
  repository       = "https://argoproj.github.io/argo-helm"
  chart            = "argo-cd"
  namespace        = "argocd"
  create_namespace = true
  version          = "5.51.0"

  values = [file("${path.module}/../values/argocd-values.yaml")]

  set_sensitive {
    name  = "configs.secret.argocdServerAdminPassword"
    value = var.admin_password_bcrypt
  }
}

resource "kubernetes_manifest" "hermesflow_project" {
  manifest = {
    apiVersion = "argoproj.io/v1alpha1"
    kind       = "AppProject"
    metadata = {
      name      = "hermesflow"
      namespace = "argocd"
    }
    spec = {
      description = "HermesFlow Applications"
      sourceRepos = ["*"]
      destinations = [{
        namespace = "*"
        server    = "https://kubernetes.default.svc"
      }]
      clusterResourceWhitelist = [{
        group = "*"
        kind  = "*"
      }]
    }
  }
  
  depends_on = [helm_release.argocd]
}
```

**关键文件 3: values/argocd-values.yaml**
```yaml
# 成本优化配置 - 适配 B2s (2vCPU, 4GB)
global:
  image:
    tag: "v2.9.3"

server:
  replicas: 1
  resources:
    limits:
      cpu: 200m
      memory: 256Mi
    requests:
      cpu: 100m
      memory: 128Mi
  service:
    type: ClusterIP

repoServer:
  replicas: 1
  resources:
    limits:
      cpu: 200m
      memory: 256Mi
    requests:
      cpu: 100m
      memory: 128Mi

controller:
  replicas: 1
  resources:
    limits:
      cpu: 500m
      memory: 512Mi
    requests:
      cpu: 250m
      memory: 256Mi

# 禁用不需要的组件（个人使用）
dex:
  enabled: false

notifications:
  enabled: false

applicationSet:
  enabled: false

# Redis
redis:
  enabled: true
  resources:
    limits:
      cpu: 100m
      memory: 128Mi
    requests:
      cpu: 50m
      memory: 64Mi

# 配置 GitOps 仓库
configs:
  repositories:
    hermesflow-gitops:
      url: https://github.com/hermesflow/HermesFlow-GitOps
      type: git
      # PAT 通过 secret 配置
```

**验收**:
- [ ] Terraform 配置语法验证通过
- [ ] 资源配置总和 < 2GB RAM
- [ ] Helm chart 版本固定
- [ ] Values 适配小内存环境

---

### Task 3.3: 配置 Providers 和 AKS 连接 (2h)

**负责人**: DevOps Engineer

**获取 AKS 连接信息**:

**步骤 1**: 在 HermesFlow 项目添加输出
```hcl
# infrastructure/terraform/environments/dev/outputs.tf
output "aks_kube_config" {
  value = {
    host                   = module.aks.kube_config.0.host
    ca_certificate         = module.aks.kube_config.0.cluster_ca_certificate
    client_certificate     = module.aks.kube_config.0.client_certificate
    client_key             = module.aks.kube_config.0.client_key
  }
  sensitive = true
}
```

**步骤 2**: 导出为环境变量
```bash
# 脚本: export-aks-config.sh
cd HermesFlow/infrastructure/terraform/environments/dev
terraform output -json aks_kube_config > /tmp/aks_config.json

export TF_VAR_aks_host=$(jq -r '.host' /tmp/aks_config.json)
export TF_VAR_aks_ca_certificate=$(jq -r '.ca_certificate' /tmp/aks_config.json)
export TF_VAR_aks_client_certificate=$(jq -r '.client_certificate' /tmp/aks_config.json)
export TF_VAR_aks_client_key=$(jq -r '.client_key' /tmp/aks_config.json)
```

**验收**:
- [ ] AKS 输出配置正确
- [ ] 环境变量脚本可执行
- [ ] Terraform providers 可连接到 AKS
- [ ] kubectl context 可切换

---

### Task 3.4: 部署 ArgoCD 到 Dev AKS (2h)

**负责人**: DevOps Engineer

**部署步骤**:
```bash
# 1. 导出 AKS 配置
cd HermesFlow/infrastructure/terraform/environments/dev
source export-aks-config.sh

# 2. 生成 admin 密码 bcrypt hash
ADMIN_PASSWORD=$(openssl rand -base64 32)
ADMIN_PASSWORD_BCRYPT=$(htpasswd -nbBC 10 "" $ADMIN_PASSWORD | tr -d ':\n' | sed 's/$2y/$2a/')

# 3. 存储密码到 Key Vault
az keyvault secret set \
  --vault-name hermesflow-dev-kv \
  --name argocd-admin-password \
  --value "$ADMIN_PASSWORD"

# 4. 部署 ArgoCD
cd HermesFlow-GitOps/infrastructure/argocd/terraform
export TF_VAR_admin_password_bcrypt="$ADMIN_PASSWORD_BCRYPT"

terraform init
terraform plan
terraform apply

# 5. 验证部署
kubectl get pods -n argocd
kubectl get svc -n argocd
```

**验收**:
- [ ] ArgoCD pods 全部 Running
- [ ] 资源占用符合预期 (< 2GB RAM)
- [ ] Admin 密码存储在 Key Vault
- [ ] Port-forward 可访问 UI

---

### Task 3.5: 配置 GitOps 仓库连接 (2h)

**负责人**: DevOps Engineer

**配置 GitHub 访问**:

**选项 A: 使用 GitHub PAT**
```bash
# 创建 PAT (Settings → Developer settings → PAT)
# Permissions: repo (all)

# 存储到 Kubernetes Secret
kubectl create secret generic github-credentials \
  --namespace argocd \
  --from-literal=type=git \
  --from-literal=url=https://github.com/hermesflow/HermesFlow-GitOps \
  --from-literal=username=hermesflow \
  --from-literal=password=$GITHUB_PAT

# 添加 label
kubectl label secret github-credentials \
  -n argocd argocd.argoproj.io/secret-type=repository
```

**选项 B: 使用 Deploy Key** (推荐)
```bash
# 1. 生成 SSH key
ssh-keygen -t ed25519 -C "argocd@hermesflow" -f ~/.ssh/argocd_ed25519

# 2. 添加到 GitHub repo (Settings → Deploy keys)
cat ~/.ssh/argocd_ed25519.pub

# 3. 创建 Secret
kubectl create secret generic hermesflow-gitops-ssh \
  --namespace argocd \
  --from-file=sshPrivateKey=$HOME/.ssh/argocd_ed25519

kubectl label secret hermesflow-gitops-ssh \
  -n argocd argocd.argoproj.io/secret-type=repository
```

**验收**:
- [ ] ArgoCD 可连接 GitOps 仓库
- [ ] Repository 显示在 UI Settings
- [ ] 连接状态为 "Successful"

---

### Task 3.6: 创建示例 Application (1h)

**负责人**: DevOps Engineer

**创建测试 Application**:

**文件**: `infrastructure/argocd/terraform/example-app.tf`
```hcl
resource "kubernetes_manifest" "data_engine_dev" {
  manifest = {
    apiVersion = "argoproj.io/v1alpha1"
    kind       = "Application"
    metadata = {
      name      = "data-engine-dev"
      namespace = "argocd"
    }
    spec = {
      project = "hermesflow"
      source = {
        repoURL        = "https://github.com/hermesflow/HermesFlow-GitOps"
        targetRevision = "main"
        path           = "apps/dev/data-engine"
      }
      destination = {
        server    = "https://kubernetes.default.svc"
        namespace = "data-engine"
      }
      syncPolicy = {
        automated = {
          prune    = true
          selfHeal = true
        }
        syncOptions = ["CreateNamespace=true"]
      }
    }
  }
  
  depends_on = [kubernetes_manifest.hermesflow_project]
}
```

**验收**:
- [ ] Application 在 UI 中可见
- [ ] Sync 状态显示正常
- [ ] 命名空间自动创建
- [ ] 应用健康检查通过

---

### Task 3.7: 文档和未来迁移指南 (2h)

**负责人**: DevOps Lead

**创建文档**:

**1. infrastructure/argocd/README.md** - 部署指南
**2. infrastructure/argocd/COST_OPTIMIZATION.md** - 成本优化说明
**3. infrastructure/argocd/MIGRATION_GUIDE.md** - 未来迁移到独立集群指南
**4. HermesFlow/docs/stories/sprint-01/sprint-01-dev-notes.md** - Dev Notes
**5. HermesFlow/docs/stories/sprint-01/sprint-01-qa-notes.md** - QA Notes

**验收**:
- [ ] 所有文档创建完成
- [ ] 包含实际可执行命令
- [ ] 未来迁移路径清晰
- [ ] 访问 UI 步骤明确

---

## 📊 测试策略

### 1. 部署测试

**基础功能测试**:
```bash
# 1. 验证 ArgoCD 部署
kubectl get pods -n argocd
kubectl get svc -n argocd

# 2. 验证资源占用
kubectl top pods -n argocd
# 预期: Total < 2GB RAM, < 1 CPU

# 3. 访问 UI
kubectl port-forward svc/argocd-server -n argocd 8080:443

# 4. 获取 admin 密码
az keyvault secret show \
  --vault-name hermesflow-dev-kv \
  --name argocd-admin-password \
  --query value -o tsv

# 5. 登录测试
# UI: https://localhost:8080
# User: admin
# Pass: (from Key Vault)
```

**GitOps 同步测试**:
- [ ] 修改 GitOps 仓库中的应用配置
- [ ] 验证 ArgoCD 自动检测变更
- [ ] 确认自动同步生效
- [ ] 验证 selfHeal 功能

### 2. 成本验证测试

- [ ] 查看 AKS 节点规格: `kubectl get nodes -o wide`
- [ ] 验证节点数量: 1 个 B2s
- [ ] 检查 ArgoCD 资源占用符合预期
- [ ] 计算实际月成本 (Azure Cost Management)

### 3. 未来迁移测试（文档验证）

- [ ] 阅读 MIGRATION_GUIDE.md
- [ ] 验证迁移步骤可行性
- [ ] 确认配置解耦正确

---

## 🔗 依赖关系

**前置依赖**:
- [x] DEVOPS-002 完成 - AKS 集群已部署
- [x] Dev AKS 可访问 (kubectl 连接)
- [ ] HermesFlow-GitOps 仓库已创建
- [ ] GitHub PAT 或 Deploy Key 已准备

**后续依赖**:
- DEVOPS-004 (应用部署) 依赖 ArgoCD 完成
- CI/CD 流程需要更新 GitOps 仓库触发 ArgoCD 同步

**与其他 Story 的关系**:
- 依赖 DEVOPS-001 的 GitOps 更新 workflow
- 依赖 DEVOPS-002 的 AKS 基础设施
- 为未来的应用部署提供 GitOps 平台

---

## 🏗️ 架构决策记录 (ADR)

### ADR-001: 为什么 Terraform 在 GitOps 仓库？

**决策**: ArgoCD 的 Terraform 配置放在 HermesFlow-GitOps 仓库

**理由**:
1. **关注点分离**: GitOps 工具不混在应用基础设施中
2. **逻辑清晰**: ArgoCD 是 GitOps 的一部分，代码应在一起
3. **未来扩展**: 便于迁移到独立管理集群
4. **代码组织**: GitOps 仓库包含所有 CD 相关配置

**影响**:
- 需要跨仓库传递 AKS 连接信息（通过环境变量）
- Terraform State 分离管理
- 部署流程略微复杂（两个仓库）

### ADR-002: 为什么部署在 Dev AKS 而非独立集群？

**决策**: Phase 1 将 ArgoCD 部署在现有 Dev AKS

**理由**:
1. **成本优化**: $0 额外成本（个人使用）
2. **资源复用**: B2s 节点足够运行 ArgoCD + 应用
3. **简化运维**: 一个集群更易管理
4. **未来可迁移**: 架构设计支持平滑迁移

**触发迁移条件**:
- 应用负载增加 (CPU > 80%)
- 管理 3+ 环境
- 预算增加 (可接受 $30/月)

### ADR-003: 为什么使用简化认证？

**决策**: 使用 admin 密码 + port-forward，不配置 Azure AD

**理由**:
1. **个人使用**: 不需要多用户和复杂 SSO
2. **简化部署**: 减少配置复杂度
3. **安全性足够**: 密码存储在 Key Vault，仅本地访问
4. **成本优化**: 不需要 Application Gateway/Ingress

---

## 💰 成本影响分析

### 当前成本 (优化前)
- AKS: 2x D4s_v3 + 1x D8s_v3 = **$560/月**
- 其他资源: **$66/月**
- **总计: $626/月**

### 优化后成本
- AKS: 1x B2s = **$30/月**
- ArgoCD: **$0** (复用 AKS)
- 其他资源: **$66/月**
- **总计: $96/月**

### 节省
- **每月节省: $530** (85% 降低)
- **年节省: $6,360**

### 未来成本（如迁移到独立集群）
- Management AKS: 1x B2s = **+$30/月**
- **总计: $126/月**
- 仍比当前节省 **$500/月** (80%)

---

## 📚 相关文档

**本 Story 文档**:
- [ArgoCD 部署指南](../../../HermesFlow-GitOps/infrastructure/argocd/README.md)
- [成本优化说明](../../../HermesFlow-GitOps/infrastructure/argocd/COST_OPTIMIZATION.md)
- [未来迁移指南](../../../HermesFlow-GitOps/infrastructure/argocd/MIGRATION_GUIDE.md)

**依赖文档**:
- [DEVOPS-001: GitHub Actions CI/CD](./DEVOPS-001-github-actions-cicd.md)
- [DEVOPS-002: Azure Terraform IaC](./DEVOPS-002-azure-terraform-iac.md)
- [GitOps 最佳实践](../../deployment/gitops-best-practices.md)
- [系统架构文档](../../architecture/system-architecture.md)

**外部资源**:
- [ArgoCD 官方文档](https://argo-cd.readthedocs.io/)
- [Terraform Helm Provider](https://registry.terraform.io/providers/hashicorp/helm/latest/docs)
- [Azure AKS 成本优化](https://docs.microsoft.com/azure/aks/cost-optimization)

---

## ✅ Definition of Done

**代码层面**:
- [ ] ArgoCD Terraform 配置在 GitOps 仓库创建
- [ ] Terraform 语法验证通过
- [ ] 成功部署到 Dev AKS
- [ ] 资源占用符合预期 (< 2GB RAM, < 1 CPU)

**测试层面**:
- [ ] ArgoCD UI 可通过 port-forward 访问
- [ ] Admin 登录成功
- [ ] GitOps 仓库连接成功
- [ ] 示例 Application 同步成功
- [ ] 成本优化验证通过

**文档层面**:
- [ ] 部署指南完成 (README.md)
- [ ] 成本优化文档完成
- [ ] 未来迁移指南完成
- [ ] Dev Notes 和 QA Notes 创建
- [ ] Sprint Summary 更新

**架构层面**:
- [ ] 代码仓库分离正确（GitOps vs 应用）
- [ ] 支持未来迁移到独立集群
- [ ] AKS 成本优化完成（可选）

---

## 📝 开发笔记 (Dev/QA Notes)

### 实施进度
- [ ] Task 3.0: AKS 成本优化
- [ ] Task 3.1: GitOps 仓库结构准备
- [ ] Task 3.2: ArgoCD Terraform 创建
- [ ] Task 3.3: Providers 配置
- [ ] Task 3.4: ArgoCD 部署
- [ ] Task 3.5: GitOps 仓库连接
- [ ] Task 3.6: 示例 Application
- [ ] Task 3.7: 文档完成

### 技术决策记录

**待开发团队填写**:
- 选择的认证方式 (PAT vs Deploy Key)
- 实际资源占用情况
- 遇到的问题和解决方案

### 访问信息

**ArgoCD UI 访问**:
```bash
# Port-forward
kubectl port-forward svc/argocd-server -n argocd 8080:443

# 获取密码
az keyvault secret show \
  --vault-name hermesflow-dev-kv \
  --name argocd-admin-password \
  --query value -o tsv

# 访问
open https://localhost:8080
# User: admin
# Pass: (from above)
```

### 成本优化记录

**当前配置**:
- AKS 节点: ___________
- ArgoCD 资源占用: ___________
- 月成本: ___________

**优化措施**:
_记录实际采取的优化措施_

---

## 🔄 Story History

| 日期 | 事件 | 操作人 |
|------|------|--------|
| 2025-10-14 | Story 创建（基于成本优化需求） | @sm.mdc |
| 2025-10-14 | Story 验证通过 (96.25/100, A 级) | @po.mdc |
| 2025-10-14 | Story 批准进入 Sprint Backlog | @po.mdc |

---

**Last Updated**: 2025-10-14  
**Status**: ✅ Approved  
**Next Step**: DevOps Team 开始实施

