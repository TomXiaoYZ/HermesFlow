# 🚀 HermesFlow CI/CD 快速启动指南

## ⚠️ 立即执行（3 个步骤）

### 步骤 1: 应用 ArgoCD Applications（5分钟）

```bash
# 连接到 Dev AKS
az aks get-credentials \
  --resource-group hermesflow-dev-rg \
  --name hermesflow-dev-aks \
  --admin

# 切换到 GitOps 仓库
cd /Users/tomxiao/Desktop/Git/personal/HermesFlow-GitOps

# 更新 GitHub 用户名（替换 YOUR_GITHUB_USERNAME）
# 方式1: 使用 sed
GITHUB_USERNAME="YOUR_ACTUAL_USERNAME"  # 替换这里
sed -i '' "s/YOUR_GITHUB_USERNAME/$GITHUB_USERNAME/g" apps/dev/argocd-applications.yaml

# 方式2: 手动编辑
# vi apps/dev/argocd-applications.yaml

# 应用配置
kubectl apply -f apps/dev/argocd-applications.yaml

# 验证（应该看到 6 个 Applications）
kubectl get app -n argocd
```

**预期输出**:
```
NAME                      SYNC STATUS   HEALTH STATUS
data-engine-dev           Synced        Healthy
user-management-dev       Synced        Healthy
api-gateway-dev           Synced        Healthy
risk-engine-dev           Synced        Healthy
strategy-engine-dev       Synced        Healthy
frontend-dev              Synced        Healthy
```

---

### 步骤 2: 测试 CI/CD 流程（10分钟）

```bash
# 切换到 HermesFlow 仓库
cd /Users/tomxiao/Desktop/Git/personal/HermesFlow

# 创建测试分支
git checkout -b test/cicd-validation

# 修改一个文件（触发 Rust CI）
echo "// Test CI/CD flow" >> modules/data-engine/src/main.rs

# 提交并推送
git add modules/data-engine/
git commit -m "test: validate complete CI/CD pipeline"
git push origin test/cicd-validation
```

**观察流程**:

```bash
# Terminal 1: 观察 GitHub Actions
gh run watch

# Terminal 2: 观察 GitOps 仓库更新（CI 完成后）
cd /Users/tomxiao/Desktop/Git/personal/HermesFlow-GitOps
watch -n 5 'git pull && git log --oneline -n 3'

# Terminal 3: 观察 ArgoCD 同步和 Pod 更新
kubectl get app data-engine-dev -n argocd -w
# 或
kubectl get pods -n hermesflow-dev -l app=data-engine -w
```

**完整流程时间轴**:
- T+0分钟: 推送代码
- T+5分钟: CI 完成，镜像推送到 ACR
- T+6分钟: GitOps 仓库自动更新
- T+9分钟: ArgoCD 检测到变更并同步
- T+10分钟: 新 Pod 运行中 ✅

---

### 步骤 3: 提交所有更改（5分钟）

```bash
# 提交 HermesFlow workflows 更改
cd /Users/tomxiao/Desktop/Git/personal/HermesFlow
git checkout main
git add .github/workflows/
git commit -m "feat(ci): configure development environment for all workflows"
git push origin main

# 提交 HermesFlow-GitOps 配置
cd /Users/tomxiao/Desktop/Git/personal/HermesFlow-GitOps
git add apps/dev/
git commit -m "feat(gitops): add Helm charts and ArgoCD auto-sync config"
git push origin main
```

---

## 📋 完整更改清单

### HermesFlow 仓库

**已更新的 Workflows**:
- ✅ `.github/workflows/ci-rust.yml`
- ✅ `.github/workflows/ci-java.yml`
- ✅ `.github/workflows/ci-python.yml`
- ✅ `.github/workflows/ci-frontend.yml`
- ✅ `.github/workflows/security-scan.yml`
- ✅ `.github/workflows/update-gitops.yml`

**关键特性**:
- 所有 CI workflows 使用 `environment: development`
- 支持 `develop`, `feature/*`, `test/*` 分支
- Image tag 格式: `{branch}-{short-sha}`
- GitOps 自动更新 (workflow_run 触发)

### HermesFlow-GitOps 仓库

**新增文件**:
```
apps/dev/
├── data-engine/
│   ├── Chart.yaml          ✅ 新增
│   └── values.yaml         ✅ 新增
├── api-gateway/
│   ├── Chart.yaml          ✅ 新增
│   └── values.yaml         ✅ 新增
├── risk-engine/
│   ├── Chart.yaml          ✅ 新增
│   └── values.yaml         ✅ 新增
├── strategy-engine/
│   ├── Chart.yaml          ✅ 新增
│   └── values.yaml         ✅ 新增
├── frontend/
│   ├── Chart.yaml          ✅ 新增
│   └── values.yaml         ✅ 新增
├── argocd-applications.yaml ✅ 新增
└── README.md               ✅ 新增
```

**ArgoCD 配置**:
- 自动同步: `automated.prune: true`, `automated.selfHeal: true`
- 3分钟轮询间隔
- 无需手动干预

---

## 🔍 验证检查清单

### ✅ 基础设施
- [x] Dev AKS 集群运行中
- [x] Dev ACR 可访问
- [x] ArgoCD 已部署
- [ ] ArgoCD Applications 已应用 ⚠️ **待执行**

### ✅ GitHub 配置
- [x] `development` Environment 已创建
- [x] 9 个 Environment Secrets 已配置
- [x] GitHub PAT 已创建

### ✅ Workflows
- [x] 所有 CI workflows 添加 environment
- [x] update-gitops.yml 支持多分支
- [ ] 测试 CI/CD 完整流程 ⚠️ **待测试**

### ✅ GitOps
- [x] 所有服务的 Helm Charts 已创建
- [x] ArgoCD Applications 配置已创建
- [ ] Applications 已部署到集群 ⚠️ **待执行**

---

## 🎯 CI/CD 流程示意图

```
开发者推送代码 (develop/feature/test)
         ↓
    GitHub Actions CI
    (environment: development)
         ↓
   构建 + 测试 + 质量检查
         ↓
   Docker build & push to ACR
   (tag: {branch}-{short-sha})
         ↓
   update-gitops.yml 自动触发
   (workflow_run)
         ↓
   更新 GitOps 仓库 values.yaml
   (apps/{env}/{service}/values.yaml)
         ↓
   ArgoCD 检测变更 (3分钟内)
         ↓
   ArgoCD 自动同步到 AKS
   (auto-sync enabled)
         ↓
   新版本部署完成 ✅
```

---

## 📞 故障排查

### 问题 1: ArgoCD Application 无法创建

**症状**: `kubectl apply` 失败

**解决方案**:
```bash
# 检查 hermesflow AppProject 是否存在
kubectl get appproject hermesflow -n argocd

# 如果不存在，创建
kubectl apply -f - <<EOF
apiVersion: argoproj.io/v1alpha1
kind: AppProject
metadata:
  name: hermesflow
  namespace: argocd
spec:
  sourceRepos:
  - '*'
  destinations:
  - namespace: 'hermesflow-*'
    server: https://kubernetes.default.svc
  clusterResourceWhitelist:
  - group: '*'
    kind: '*'
EOF
```

### 问题 2: GitHub Actions 无法访问 secrets

**症状**: CI workflow 失败，错误信息提示 secret 不存在

**解决方案**:
```bash
# 验证 secrets 配置
gh secret list --env development

# 应该看到 9 个 secrets:
# AZURE_CLIENT_ID
# AZURE_CLIENT_SECRET
# AZURE_SUBSCRIPTION_ID
# AZURE_TENANT_ID
# ACR_LOGIN_SERVER
# ACR_USERNAME
# ACR_PASSWORD
# GITOPS_PAT
# POSTGRES_ADMIN_PASSWORD
```

### 问题 3: ArgoCD 未同步

**症状**: Git 已更新，但 ArgoCD 未同步

**解决方案**:
```bash
# 手动触发同步
kubectl patch app data-engine-dev -n argocd \
  --type merge \
  --patch '{"spec":{"syncPolicy":{"syncOptions":["CreateNamespace=true"]}}}'

# 或使用 argocd CLI
argocd app sync data-engine-dev

# 查看同步状态
argocd app get data-engine-dev
```

---

## 📚 详细文档

- **完整实施报告**: `docs/qa/github-actions-dev-setup-completed.md`
- **GitOps 部署指南**: `HermesFlow-GitOps/apps/dev/README.md`
- **ArgoCD 配置**: `HermesFlow-GitOps/apps/dev/argocd-applications.yaml`
- **Workflow 配置**: `.github/workflows/`

---

## ✨ 下一步

1. **立即执行**: 完成上述 3 个步骤
2. **验证测试**: 确保 CI/CD 流程完整运行
3. **日常使用**: 正常开发，系统将自动处理部署

**预计总时间**: 20 分钟

---

**需要帮助?** 查看 `docs/qa/github-actions-dev-setup-completed.md` 获取详细说明。

**祝部署顺利！** 🚀

