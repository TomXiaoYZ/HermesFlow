# GitOps 最佳实践指南

本文档详细描述 HermesFlow 项目的 GitOps 工作流最佳实践、故障排查方法和运维建议。

## 目录

- [1. GitOps 核心原则](#1-gitops-核心原则)
- [2. 仓库结构最佳实践](#2-仓库结构最佳实践)
- [3. Helm Chart 管理](#3-helm-chart-管理)
- [4. 环境管理策略](#4-环境管理策略)
- [5. 密钥管理](#5-密钥管理)
- [6. 变更管理流程](#6-变更管理流程)
- [7. 回滚策略](#7-回滚策略)
- [8. 监控与告警](#8-监控与告警)
- [9. 故障排查](#9-故障排查)
- [10. 安全最佳实践](#10-安全最佳实践)
- [11. 性能优化](#11-性能优化)
- [12. 团队协作](#12-团队协作)

---

## 1. GitOps 核心原则

### 1.1 声明式配置

**原则**：所有系统状态都通过声明式配置定义。

**实践**：

```yaml
# ✅ 好的实践：声明期望状态
apiVersion: apps/v1
kind: Deployment
spec:
  replicas: 3
  template:
    spec:
      containers:
      - name: data-engine
        image: hermesflow-dev-acr.azurecr.io/data-engine:abc123def
        resources:
          limits:
            cpu: 2000m
            memory: 2Gi

# ❌ 坏的实践：命令式操作
# kubectl scale deployment data-engine --replicas=3
# kubectl set image deployment/data-engine data-engine=xxx:abc123def
```

### 1.2 Git 作为唯一真实来源

**原则**：Git 仓库是系统状态的唯一权威来源。

**实践**：

- ✅ 所有配置变更必须通过 Git commit
- ✅ 使用 Pull Request 进行审核
- ✅ 保留完整的审计日志
- ❌ 避免直接在集群中手动修改资源

### 1.3 自动化同步

**原则**：系统自动检测并应用 Git 中的配置变更。

**ArgoCD 配置**：

```yaml
syncPolicy:
  automated:
    prune: true      # 自动删除不再需要的资源
    selfHeal: true   # 自动修复配置漂移
  syncOptions:
    - CreateNamespace=true
  retry:
    limit: 5
    backoff:
      duration: 5s
      factor: 2
      maxDuration: 3m
```

### 1.4 版本控制与回滚

**原则**：通过 Git 历史轻松回滚到任意版本。

**实践**：

```bash
# 查看部署历史
git log --oneline apps/dev/data-engine/values.yaml

# 回滚到特定版本
git revert <commit-hash>
git push

# ArgoCD 自动同步回滚
```

---

## 2. 仓库结构最佳实践

### 2.1 推荐的目录结构

```
HermesFlow-GitOps/
├── base-charts/                    # 基础 Chart 模板
│   └── microservice/
│       ├── Chart.yaml
│       ├── values.yaml            # 默认值
│       └── templates/
│           ├── deployment.yaml
│           ├── service.yaml
│           ├── hpa.yaml
│           ├── ingress.yaml
│           ├── configmap.yaml
│           └── _helpers.tpl
│
├── apps/                           # 环境特定配置
│   ├── dev/
│   │   ├── _namespace.yaml        # Namespace 配置
│   │   ├── data-engine/
│   │   │   ├── Chart.yaml         # 依赖 base-charts
│   │   │   ├── values.yaml        # 环境特定值
│   │   │   └── secrets.yaml       # Sealed Secrets
│   │   ├── strategy-engine/
│   │   └── ...
│   │
│   └── main/
│       └── ...
│
├── argocd/                         # ArgoCD 配置
│   ├── applications/
│   │   ├── hermesflow-dev.yaml
│   │   └── hermesflow-main.yaml
│   └── projects/
│       └── hermesflow.yaml
│
├── docs/                           # 文档
│   ├── README.md
│   ├── deployment-guide.md
│   └── troubleshooting.md
│
└── .github/
    └── workflows/
        └── update-values.yml       # 自动更新 workflow
```

### 2.2 Chart 依赖管理

**base-charts/microservice/Chart.yaml**：

```yaml
apiVersion: v2
name: microservice
description: HermesFlow 微服务基础 Chart
type: application
version: 1.0.0
appVersion: "1.0"
```

**apps/dev/data-engine/Chart.yaml**：

```yaml
apiVersion: v2
name: data-engine-dev
description: Data Engine for Dev Environment
type: application
version: 0.1.0

# 依赖基础 Chart
dependencies:
  - name: microservice
    version: "1.0.0"
    repository: "file://../../base-charts/microservice"
```

### 2.3 Values 文件层次

```
base-charts/microservice/values.yaml    ← 默认值（所有环境通用）
        ↓
apps/dev/data-engine/values.yaml        ← 环境特定值（覆盖默认值）
```

**覆盖策略**：

```yaml
# base-charts/microservice/values.yaml (默认)
replicaCount: 1
resources:
  limits:
    cpu: 1000m
    memory: 1Gi

# apps/dev/data-engine/values.yaml (覆盖)
replicaCount: 2  # 覆盖为 2
resources:
  limits:
    cpu: 2000m   # 覆盖 CPU
    memory: 2Gi  # 覆盖内存
```

---

## 3. Helm Chart 管理

### 3.1 Chart 版本管理

**语义化版本**：

- **Major (1.x.x)**: 不兼容的 API 变更
- **Minor (x.1.x)**: 向后兼容的功能新增
- **Patch (x.x.1)**: 向后兼容的 Bug 修复

**示例**：

```yaml
# Chart.yaml
version: 1.2.3
# 1: Major (破坏性变更)
# 2: Minor (新功能)
# 3: Patch (Bug修复)
```

### 3.2 模板最佳实践

**使用 _helpers.tpl 定义复用函数**：

```yaml
{{/* _helpers.tpl */}}

{{/*
生成完整的应用名称
*/}}
{{- define "microservice.fullname" -}}
{{- if .Values.fullnameOverride }}
{{- .Values.fullnameOverride | trunc 63 | trimSuffix "-" }}
{{- else }}
{{- $name := default .Chart.Name .Values.nameOverride }}
{{- printf "%s-%s" .Release.Name $name | trunc 63 | trimSuffix "-" }}
{{- end }}
{{- end }}

{{/*
生成标签选择器
*/}}
{{- define "microservice.selectorLabels" -}}
app.kubernetes.io/name: {{ include "microservice.name" . }}
app.kubernetes.io/instance: {{ .Release.Name }}
{{- end }}
```

**在模板中使用**：

```yaml
# deployment.yaml
metadata:
  name: {{ include "microservice.fullname" . }}
  labels:
    {{- include "microservice.labels" . | nindent 4 }}
spec:
  selector:
    matchLabels:
      {{- include "microservice.selectorLabels" . | nindent 6 }}
```

### 3.3 条件资源

**根据配置动态启用资源**：

```yaml
# values.yaml
autoscaling:
  enabled: true
  minReplicas: 2
  maxReplicas: 10

ingress:
  enabled: false
  host: data-engine.hermesflow.com
```

```yaml
# hpa.yaml
{{- if .Values.autoscaling.enabled }}
apiVersion: autoscaling/v2
kind: HorizontalPodAutoscaler
metadata:
  name: {{ include "microservice.fullname" . }}
spec:
  scaleTargetRef:
    apiVersion: apps/v1
    kind: Deployment
    name: {{ include "microservice.fullname" . }}
  minReplicas: {{ .Values.autoscaling.minReplicas }}
  maxReplicas: {{ .Values.autoscaling.maxReplicas }}
{{- end }}
```

---

## 4. 环境管理策略

### 4.1 环境隔离

| 环境 | 命名空间 | ACR | 分支 | 自动部署 |
|------|---------|-----|------|---------|
| dev | hermesflow-dev | hermesflow-dev-acr | dev | ✅ |
| staging | hermesflow-staging | hermesflow-staging-acr | staging | ✅ |
| main | hermesflow-prod | hermesflow-prod-acr | main | ✅ |

### 4.2 环境特定配置

**dev 环境（宽松配置）**：

```yaml
# apps/dev/data-engine/values.yaml
replicaCount: 1
resources:
  limits:
    cpu: 2000m
    memory: 2Gi
autoscaling:
  enabled: false

env:
  - name: RUST_LOG
    value: "debug"  # 开发环境详细日志
  - name: RUST_BACKTRACE
    value: "1"
```

**main 环境（严格配置）**：

```yaml
# apps/main/data-engine/values.yaml
replicaCount: 3
resources:
  limits:
    cpu: 4000m
    memory: 4Gi
  requests:
    cpu: 2000m
    memory: 2Gi
autoscaling:
  enabled: true
  minReplicas: 3
  maxReplicas: 10

env:
  - name: RUST_LOG
    value: "info"  # 生产环境减少日志
```

### 4.3 环境提升流程

```
dev 环境测试
    ↓
创建 Pull Request: dev → staging
    ↓
Code Review + 自动化测试
    ↓
合并到 staging 分支
    ↓
staging 环境自动部署
    ↓
Smoke Test + 集成测试
    ↓
创建 Pull Request: staging → main
    ↓
需要 2 人审核 + 必须通过 CI
    ↓
合并到 main 分支
    ↓
main 环境自动部署
    ↓
监控 + 金丝雀发布（可选）
```

---

## 5. 密钥管理

### 5.1 使用 Sealed Secrets

**为什么使用 Sealed Secrets？**

- ✅ 加密后的 Secret 可以安全地存储在 Git 中
- ✅ 只有 Kubernetes 集群可以解密
- ✅ 支持 GitOps 工作流

**安装 Sealed Secrets Controller**：

```bash
kubectl apply -f https://github.com/bitnami-labs/sealed-secrets/releases/download/v0.24.0/controller.yaml
```

**创建 Sealed Secret**：

```bash
# 1. 创建普通 Secret
kubectl create secret generic postgres-secret \
  --from-literal=connection-string='postgresql://user:pass@host:5432/db' \
  --dry-run=client -o yaml > secret.yaml

# 2. 使用 kubeseal 加密
kubeseal --format yaml < secret.yaml > sealed-secret.yaml

# 3. 提交到 Git
git add apps/dev/data-engine/sealed-secret.yaml
git commit -m "Add PostgreSQL sealed secret for dev"
git push
```

**Sealed Secret 示例**：

```yaml
# apps/dev/data-engine/sealed-secret.yaml
apiVersion: bitnami.com/v1alpha1
kind: SealedSecret
metadata:
  name: postgres-secret
  namespace: hermesflow-dev
spec:
  encryptedData:
    connection-string: AgA...encrypted...data...
  template:
    metadata:
      name: postgres-secret
      namespace: hermesflow-dev
    type: Opaque
```

### 5.2 External Secrets Operator（推荐）

**从 Azure Key Vault 自动同步**：

```yaml
# apps/dev/data-engine/external-secret.yaml
apiVersion: external-secrets.io/v1beta1
kind: ExternalSecret
metadata:
  name: postgres-secret
  namespace: hermesflow-dev
spec:
  refreshInterval: 1h
  secretStoreRef:
    name: azure-keyvault-store
    kind: SecretStore
  target:
    name: postgres-secret
  data:
  - secretKey: connection-string
    remoteRef:
      key: hermesflow-dev-postgres-connection-string
```

**配置 SecretStore**：

```yaml
apiVersion: external-secrets.io/v1beta1
kind: SecretStore
metadata:
  name: azure-keyvault-store
  namespace: hermesflow-dev
spec:
  provider:
    azurekv:
      authType: WorkloadIdentity
      vaultUrl: "https://hermesflow-dev-kv.vault.azure.net"
```

### 5.3 密钥轮换

**定期轮换策略**：

| 密钥类型 | 轮换周期 | 方法 |
|---------|---------|------|
| 数据库密码 | 90天 | Azure Key Vault 自动轮换 |
| API密钥 | 180天 | 手动轮换 + External Secrets 自动同步 |
| TLS证书 | Let's Encrypt自动更新 (90天) | cert-manager |
| Service Principal | 365天 | Azure AD 轮换 |

---

## 6. 变更管理流程

### 6.1 变更类型

| 类型 | 审批要求 | 自动部署 | 示例 |
|------|---------|---------|------|
| 🟢 Low Risk | 1人审核 | ✅ | 日志级别调整、资源限制调整 |
| 🟡 Medium Risk | 2人审核 | ✅ | 镜像版本更新、副本数调整 |
| 🔴 High Risk | 2人审核 + QA | ❌ 手动 | 数据库迁移、架构变更 |

### 6.2 Pull Request 模板

```markdown
## 变更描述
简要描述此次变更的目的和内容。

## 变更类型
- [ ] 🟢 Low Risk: 配置调整
- [ ] 🟡 Medium Risk: 版本更新
- [ ] 🔴 High Risk: 架构变更

## 影响范围
- [ ] dev 环境
- [ ] staging 环境
- [ ] main 环境

## 测试计划
- [ ] 单元测试通过
- [ ] 集成测试通过
- [ ] 在 dev 环境验证
- [ ] Smoke Test 通过

## 回滚计划
如果部署失败，回滚步骤：
1. ...
2. ...

## Checklist
- [ ] values.yaml 已更新
- [ ] 密钥已配置（如需要）
- [ ] 文档已更新
- [ ] 监控告警已配置
```

### 6.3 自动化检查

**.github/workflows/pr-validation.yml**：

```yaml
name: PR Validation

on:
  pull_request:
    branches: [dev, staging, main]

jobs:
  validate:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      
      - name: Install Helm
        uses: azure/setup-helm@v3
        with:
          version: '3.12.0'
      
      - name: Validate Helm Charts
        run: |
          for chart in apps/*/*/; do
            helm lint "$chart"
            helm template "$chart" --debug
          done
      
      - name: Check for Secrets in Plain Text
        run: |
          if git diff --name-only origin/main | xargs grep -E 'password|secret|token' | grep -v 'sealed-secret.yaml'; then
            echo "❌ Found plain text secrets!"
            exit 1
          fi
      
      - name: Validate YAML Syntax
        run: |
          find apps -name '*.yaml' -exec yamllint {} \;
```

---

## 7. 回滚策略

### 7.1 回滚决策矩阵

| 故障严重程度 | 回滚方法 | 预计时间 | 触发条件 |
|------------|---------|---------|---------|
| **P0 Critical** | Kubernetes 原生回滚 | ~30秒 | 服务完全不可用 |
| **P1 High** | ArgoCD 回滚 | ~2分钟 | 错误率 >5% |
| **P2 Medium** | GitOps 回滚 | ~3分钟 | 性能下降 >20% |
| **P3 Low** | 修复后重新部署 | 正常流程 | 非关键功能异常 |

### 7.2 方法1：Kubernetes 原生回滚（最快）

```bash
# 立即回滚到上一个版本
kubectl rollout undo deployment/data-engine -n hermesflow-dev

# 查看回滚历史
kubectl rollout history deployment/data-engine -n hermesflow-dev

# 回滚到特定版本
kubectl rollout undo deployment/data-engine --to-revision=3 -n hermesflow-dev

# 实时查看回滚状态
kubectl rollout status deployment/data-engine -n hermesflow-dev -w
```

**⚠️ 注意**：Kubernetes 原生回滚会造成配置漂移（Git 与集群状态不一致），需要后续更新 Git。

### 7.3 方法2：ArgoCD 回滚（推荐）

```bash
# CLI 回滚
argocd app rollback hermesflow-dev <revision-number>

# 查看历史版本
argocd app history hermesflow-dev

# Web UI 回滚
# 1. 访问 ArgoCD Dashboard
# 2. 选择 Application: hermesflow-dev
# 3. 点击 "History" 标签
# 4. 选择要回滚的版本
# 5. 点击 "Rollback"
```

**优点**：
- ✅ 保持 GitOps 一致性
- ✅ 审计日志完整
- ✅ 支持部分回滚（特定模块）

### 7.4 方法3：GitOps 回滚（最安全）

```bash
# 进入 GitOps 仓库
cd HermesFlow-GitOps

# 查看变更历史
git log --oneline apps/dev/data-engine/values.yaml

# 方法A：使用 git revert（推荐，保留历史）
git revert <commit-hash>
git push

# 方法B：直接重置到旧版本（慎用）
git checkout <old-commit-hash> -- apps/dev/data-engine/values.yaml
git commit -m "⏪ Rollback data-engine to previous version"
git push

# ArgoCD 自动检测并同步（约3分钟）
```

### 7.5 自动回滚（金丝雀发布）

**使用 Argo Rollouts 实现自动回滚**：

```yaml
apiVersion: argoproj.io/v1alpha1
kind: Rollout
metadata:
  name: data-engine
spec:
  replicas: 5
  strategy:
    canary:
      steps:
      - setWeight: 20
      - pause: {duration: 5m}
      - analysis:
          templates:
          - templateName: error-rate-analysis
      - setWeight: 50
      - pause: {duration: 5m}
      
      # 自动回滚条件
      analysisTemplateRef:
        name: error-rate-analysis
      
      # 如果分析失败，自动回滚
      abortScaleDownDelaySeconds: 30

---
apiVersion: argoproj.io/v1alpha1
kind: AnalysisTemplate
metadata:
  name: error-rate-analysis
spec:
  metrics:
  - name: error-rate
    interval: 1m
    successCondition: result < 0.05  # 错误率 < 5%
    failureLimit: 3
    provider:
      prometheus:
        address: http://prometheus:9090
        query: |
          sum(rate(http_requests_total{status=~"5..", app="data-engine"}[5m])) /
          sum(rate(http_requests_total{app="data-engine"}[5m]))
```

---

## 8. 监控与告警

### 8.1 关键指标

**部署健康指标**：

```promql
# ArgoCD 同步状态
argocd_app_sync_status{project="hermesflow"} != 1

# 同步失败次数
argocd_app_sync_total{phase="Failed"}

# Pod 重启次数
rate(kube_pod_container_status_restarts_total{namespace="hermesflow-dev"}[5m]) > 0

# 镜像拉取失败
kube_pod_container_status_waiting_reason{reason="ImagePullBackOff"} > 0

# 部署时间
argocd_app_reconcile_duration_seconds
```

### 8.2 告警规则

**prometheus-alerts.yaml**：

```yaml
groups:
- name: gitops
  interval: 30s
  rules:
  
  # ArgoCD 同步失败
  - alert: ArgoCDSyncFailed
    expr: argocd_app_sync_status{project="hermesflow"} != 1
    for: 5m
    labels:
      severity: warning
      team: platform
    annotations:
      summary: "ArgoCD sync failed for {{ $labels.name }}"
      description: "Application {{ $labels.name }} has been out of sync for 5 minutes"
  
  # Pod 持续重启
  - alert: PodCrashLooping
    expr: rate(kube_pod_container_status_restarts_total[15m]) > 0
    for: 5m
    labels:
      severity: critical
      team: {{ $labels.namespace }}
    annotations:
      summary: "Pod {{ $labels.pod }} is crash looping"
      description: "Pod has restarted {{ $value }} times in the last 15 minutes"
  
  # 镜像拉取失败
  - alert: ImagePullFailed
    expr: kube_pod_container_status_waiting_reason{reason="ImagePullBackOff"} > 0
    for: 2m
    labels:
      severity: critical
    annotations:
      summary: "Failed to pull image for {{ $labels.pod }}"
      description: "Check ACR connectivity and image tag"
  
  # 部署耗时过长
  - alert: SlowDeployment
    expr: argocd_app_reconcile_duration_seconds > 600
    labels:
      severity: warning
    annotations:
      summary: "Deployment taking too long for {{ $labels.name }}"
      description: "Deployment has been running for {{ $value }}s (>10m)"
```

### 8.3 Grafana 仪表盘

**推荐的仪表盘**：

1. **ArgoCD 官方仪表盘** (ID: 14584)
   - 应用同步状态
   - 同步历史
   - 资源健康状况

2. **Kubernetes Deployment 仪表盘** (ID: 8588)
   - Pod 状态
   - 副本数
   - 资源使用率

3. **自定义 GitOps 仪表盘**：

```json
{
  "dashboard": {
    "title": "HermesFlow GitOps Overview",
    "panels": [
      {
        "title": "Deployment Frequency",
        "targets": [
          {
            "expr": "sum(increase(argocd_app_sync_total[1h]))"
          }
        ]
      },
      {
        "title": "Mean Time to Recovery (MTTR)",
        "targets": [
          {
            "expr": "histogram_quantile(0.95, argocd_app_reconcile_duration_seconds_bucket)"
          }
        ]
      },
      {
        "title": "Change Failure Rate",
        "targets": [
          {
            "expr": "sum(argocd_app_sync_total{phase=\"Failed\"}) / sum(argocd_app_sync_total)"
          }
        ]
      }
    ]
  }
}
```

---

## 9. 故障排查

### 9.1 常见问题与解决方案

#### 问题1：ArgoCD 同步卡住

**症状**：

```
Application: hermesflow-dev
Status: Syncing... (超过10分钟)
```

**排查步骤**：

```bash
# 1. 查看 ArgoCD 日志
kubectl logs -n argocd -l app.kubernetes.io/name=argocd-application-controller --tail=100

# 2. 检查应用状态
argocd app get hermesflow-dev

# 3. 查看同步详情
argocd app sync hermesflow-dev --dry-run

# 4. 强制刷新
argocd app sync hermesflow-dev --force --prune

# 5. 如果仍然卡住，重启 ArgoCD
kubectl rollout restart deployment argocd-application-controller -n argocd
```

#### 问题2：Helm Chart 渲染失败

**症状**：

```
Error: template: microservice/templates/deployment.yaml:15:24: 
executing "microservice/templates/deployment.yaml" at <.Values.image.tag>: 
nil pointer evaluating interface {}.tag
```

**排查步骤**：

```bash
# 1. 验证 values.yaml 语法
helm lint apps/dev/data-engine/

# 2. 渲染模板查看输出
helm template data-engine apps/dev/data-engine/ --debug

# 3. 检查必需的值是否存在
yq eval '.image.tag' apps/dev/data-engine/values.yaml

# 4. 使用 --dry-run 测试
helm install data-engine apps/dev/data-engine/ --dry-run --debug
```

#### 问题3：镜像拉取失败

**症状**：

```
Warning  Failed     2m (x4 over 4m)  kubelet  Failed to pull image 
"hermesflow-dev-acr.azurecr.io/data-engine:abc123def": 
rpc error: code = Unknown desc = failed to pull and unpack image: 
failed to resolve reference: pull access denied
```

**排查步骤**：

```bash
# 1. 验证 ACR 连接性
az aks check-acr \
  --name hermesflow-aks-dev \
  --resource-group hermesflow-rg \
  --acr hermesflow-dev-acr.azurecr.io

# 2. 检查 AKS 与 ACR 的集成
az aks show --name hermesflow-aks-dev --resource-group hermesflow-rg \
  --query "identityProfile.kubeletidentity.objectId" -o tsv

# 3. 重新附加 ACR
az aks update \
  --name hermesflow-aks-dev \
  --resource-group hermesflow-rg \
  --attach-acr hermesflow-dev-acr

# 4. 手动测试拉取
az acr login --name hermesflow-dev-acr
docker pull hermesflow-dev-acr.azurecr.io/data-engine:abc123def
```

#### 问题4：配置漂移检测

**症状**：

```
ArgoCD 显示 "OutOfSync" 但 Git 中的配置是最新的
```

**排查步骤**：

```bash
# 1. 查看差异
argocd app diff hermesflow-dev

# 2. 检查是否有手动修改
kubectl get deployment data-engine -n hermesflow-dev -o yaml | \
  grep -A 5 "kubernetes.io/change-cause"

# 3. 启用 selfHeal 自动修复
kubectl patch application hermesflow-dev -n argocd --type merge \
  -p '{"spec":{"syncPolicy":{"automated":{"selfHeal":true}}}}'

# 4. 手动同步
argocd app sync hermesflow-dev --force
```

### 9.2 调试工具箱

**必备工具**：

```bash
# ArgoCD CLI
brew install argocd
argocd login argocd.hermesflow.com

# Helm
brew install helm

# kubectl plugins
kubectl krew install neat      # 清理 YAML 输出
kubectl krew install tree      # 查看资源树
kubectl krew install tail      # 多 Pod 日志聚合

# yq (YAML 处理)
brew install yq

# k9s (交互式 K8s 管理)
brew install k9s
```

**调试脚本**：

```bash
#!/bin/bash
# debug-deployment.sh - 快速诊断部署问题

NAMESPACE=$1
DEPLOYMENT=$2

echo "🔍 Debugging Deployment: $DEPLOYMENT in $NAMESPACE"

echo "\n📦 Deployment Status:"
kubectl get deployment $DEPLOYMENT -n $NAMESPACE

echo "\n🏃 Pod Status:"
kubectl get pods -n $NAMESPACE -l app=$DEPLOYMENT

echo "\n📋 Recent Events:"
kubectl get events -n $NAMESPACE --field-selector involvedObject.name=$DEPLOYMENT --sort-by='.lastTimestamp'

echo "\n📊 Resource Usage:"
kubectl top pods -n $NAMESPACE -l app=$DEPLOYMENT

echo "\n📜 Pod Logs (last 50 lines):"
kubectl logs -n $NAMESPACE -l app=$DEPLOYMENT --tail=50

echo "\n🔄 Rollout History:"
kubectl rollout history deployment/$DEPLOYMENT -n $NAMESPACE

echo "\n✅ Health Checks:"
kubectl describe deployment $DEPLOYMENT -n $NAMESPACE | grep -A 10 "Conditions:"
```

---

## 10. 安全最佳实践

### 10.1 最小权限原则

**ArgoCD RBAC 配置**：

```yaml
# argocd-rbac-cm.yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: argocd-rbac-cm
  namespace: argocd
data:
  policy.csv: |
    # 开发团队：只能部署到 dev 环境
    p, role:dev-team, applications, sync, */dev/*, allow
    p, role:dev-team, applications, get, */dev/*, allow
    g, dev-team@hermesflow.com, role:dev-team
    
    # 运维团队：可以部署到所有环境
    p, role:ops-team, applications, *, */*, allow
    g, ops-team@hermesflow.com, role:ops-team
    
    # 只读角色：所有人可查看
    p, role:readonly, applications, get, */*, allow
    g, *@hermesflow.com, role:readonly
```

### 10.2 审计日志

**启用 ArgoCD 审计**：

```yaml
# argocd-cmd-params-cm.yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: argocd-cmd-params-cm
  namespace: argocd
data:
  server.log.level: "info"
  server.log.format: "json"
  
  # 启用审计日志
  server.audit.log.enabled: "true"
  server.audit.log.format: "json"
```

**查询审计日志**：

```bash
# 查看最近的同步操作
kubectl logs -n argocd -l app.kubernetes.io/name=argocd-server | \
  jq 'select(.audit == true and .method == "ApplicationService.Sync")'

# 查看特定用户的操作
kubectl logs -n argocd -l app.kubernetes.io/name=argocd-server | \
  jq 'select(.audit == true and .user == "tom@hermesflow.com")'
```

### 10.3 镜像签名验证

**使用 Cosign 签名镜像**：

```bash
# 1. 签名镜像
cosign sign hermesflow-dev-acr.azurecr.io/data-engine:abc123def \
  --key cosign.key

# 2. 在 Kubernetes 中验证签名（使用 admission webhook）
kubectl apply -f - <<EOF
apiVersion: admissionregistration.k8s.io/v1
kind: ValidatingWebhookConfiguration
metadata:
  name: cosign-webhook
webhooks:
- name: cosign.hermesflow.com
  rules:
  - operations: ["CREATE", "UPDATE"]
    apiGroups: [""]
    apiVersions: ["v1"]
    resources: ["pods"]
  clientConfig:
    service:
      name: cosign-webhook
      namespace: cosign-system
EOF
```

---

## 11. 性能优化

### 11.1 ArgoCD 性能调优

**增加并发同步数**：

```yaml
# argocd-cmd-params-cm.yaml
data:
  application.sync.workers: "10"           # 默认 5
  repo.server.parallelism.limit: "5"      # 默认 0 (无限制)
  controller.status.processors: "20"       # 默认 20
  controller.operation.processors: "10"    # 默认 10
```

**减少轮询频率（大规模集群）**：

```yaml
syncPolicy:
  automated:
    # 减少轮询频率（默认 3分钟）
    pollingInterval: "5m"
```

### 11.2 Helm Chart 优化

**减少不必要的模板渲染**：

```yaml
# ❌ 坏的实践：每次都渲染所有资源
{{- range .Values.services }}
  ...
{{- end }}

# ✅ 好的实践：使用条件判断
{{- if .Values.serviceEnabled }}
  ...
{{- end }}
```

**使用 Helm 依赖缓存**：

```bash
# 构建依赖缓存
helm dependency build apps/dev/data-engine/

# 使用缓存部署
helm upgrade --install data-engine apps/dev/data-engine/ \
  --reuse-values
```

### 11.3 Git 仓库优化

**使用 Shallow Clone**：

```yaml
# ArgoCD Application
spec:
  source:
    repoURL: https://github.com/tomxiao/HermesFlow-GitOps.git
    targetRevision: HEAD
    # 仅克隆最近的 commit（加快同步速度）
    helm:
      skipCrds: false
    # 启用 Git LFS（如果有大文件）
    gitLfs: true
```

**定期清理历史**：

```bash
# 清理超过 180 天的 Git 历史（可选）
git filter-branch --prune-empty --subdirectory-filter apps/dev -- --all
git reflog expire --expire=now --all
git gc --prune=now --aggressive
```

---

## 12. 团队协作

### 12.1 文档规范

**README.md 模板**：

```markdown
# [模块名称] - [环境]

## 概述
简要描述此模块的功能。

## 镜像信息
- **Registry**: hermesflow-dev-acr.azurecr.io
- **Image**: data-engine
- **Current Tag**: abc123def456

## 配置文件
- `values.yaml`: 环境特定配置
- `sealed-secret.yaml`: 加密的密钥

## 部署流程
1. 更新 `values.yaml` 中的 `image.tag`
2. 创建 Pull Request
3. Code Review
4. 合并后自动部署

## 回滚
```bash
kubectl rollout undo deployment/data-engine -n hermesflow-dev
```

## 联系人
- **Owner**: Data Team
- **Slack**: #team-data
- **On-call**: https://pagerduty.com/data-team
```

### 12.2 Code Review Checklist

**审查要点**：

- [ ] Commit Message 清晰描述了变更
- [ ] values.yaml 中的镜像 tag 已更新
- [ ] 没有硬编码的密钥或密码
- [ ] 资源限制（CPU/Memory）合理
- [ ] 副本数符合环境要求（dev: 1, main: 3+）
- [ ] 健康检查配置正确
- [ ] 环境变量配置完整
- [ ] 文档已更新
- [ ] 测试计划已提供

### 12.3 沟通渠道

**部署通知集成**：

```yaml
# ArgoCD Notifications
apiVersion: v1
kind: ConfigMap
metadata:
  name: argocd-notifications-cm
  namespace: argocd
data:
  service.slack: |
    token: $slack-token
  
  template.app-deployed: |
    message: |
      🚀 Application {{.app.metadata.name}} deployed successfully!
      Environment: {{.app.metadata.labels.env}}
      Version: {{.app.status.sync.revision}}
      User: {{.app.status.operationState.operation.initiatedBy.username}}
  
  trigger.on-deployed: |
    - when: app.status.operationState.phase in ['Succeeded']
      send: [app-deployed]
```

**Slack 通知示例**：

```
🚀 Deployment Notification

Application: data-engine-dev
Environment: dev
Status: ✅ Succeeded
Revision: abc123def456
Duration: 2m 38s
Initiated by: tom@hermesflow.com

View in ArgoCD: https://argocd.hermesflow.com/applications/data-engine-dev
```

---

## 附录

### A. 常用命令速查

```bash
# ArgoCD
argocd app list
argocd app get <app-name>
argocd app sync <app-name> --prune
argocd app rollback <app-name> <revision>
argocd app history <app-name>

# Helm
helm list -n <namespace>
helm upgrade --install <release> <chart> -n <namespace>
helm rollback <release> <revision> -n <namespace>
helm history <release> -n <namespace>

# Kubectl
kubectl get applications -n argocd
kubectl rollout status deployment/<name> -n <namespace>
kubectl rollout history deployment/<name> -n <namespace>
kubectl rollout undo deployment/<name> -n <namespace>
```

### B. 参考资料

- [GitOps Principles](https://opengitops.dev/)
- [ArgoCD Best Practices](https://argo-cd.readthedocs.io/en/stable/user-guide/best_practices/)
- [Helm Best Practices](https://helm.sh/docs/chart_best_practices/)
- [Kubernetes Security Best Practices](https://kubernetes.io/docs/concepts/security/security-checklist/)

---

**最后更新**: 2024-12-20  
**维护团队**: HermesFlow Platform Team  
**反馈渠道**: Slack #gitops-support

