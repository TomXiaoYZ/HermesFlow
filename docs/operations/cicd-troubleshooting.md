# CI/CD Troubleshooting Guide

**版本**: v1.0.0  
**最后更新**: 2025-10-21  
**维护者**: DevOps Team

本文档提供 HermesFlow CI/CD 流程中常见问题的诊断和解决方案。

---

## 📋 目录

- [CI Workflow 问题](#ci-workflow-问题)
- [GitOps 更新问题](#gitops-更新问题)
- [ArgoCD 同步问题](#argocd-同步问题)
- [Kubernetes 部署问题](#kubernetes-部署问题)
- [网络和认证问题](#网络和认证问题)
- [回滚流程](#回滚流程)
- [紧急修复步骤](#紧急修复步骤)

---

## CI Workflow 问题

### 问题 1: CI Workflow 未触发

**症状**:
- Git push 后 GitHub Actions 没有运行
- Actions 页面没有新的 workflow run

**可能原因**:
1. Commit message 格式错误
2. 推送的分支不在触发列表中
3. Workflow 文件语法错误

**解决方案**:

```bash
# 1. 检查 commit message
git log -1 --oneline
# 确保包含 [module: xxx]

# 2. 检查当前分支
git branch --show-current
# 应该是 develop 或 main

# 3. 检查 workflow 文件
cat .github/workflows/ci-rust.yml | head -20
# 确认 on.push.branches 包含当前分支

# 4. 重新触发（如果 commit 正确但未触发）
git commit --amend --no-edit
git push -f origin develop
```

---

### 问题 2: CI 构建失败

**症状**:
- GitHub Actions 显示红色 ❌
- 构建在某个步骤失败

**诊断步骤**:

```bash
# 1. 查看 GitHub Actions 日志
# 访问: https://github.com/TomXiaoYZ/HermesFlow/actions
# 点击失败的 workflow run
# 展开失败的步骤查看详细日志

# 2. 本地复现问题
cd modules/<module-name>

# Rust
cargo clean
cargo test
cargo build --release

# Java
mvn clean test
mvn clean package

# Python
poetry install
poetry run pytest

# Frontend
npm install --legacy-peer-deps
npm test
npm run build
```

**常见错误和解决方案**:

#### 错误: 测试失败

```bash
# Rust
# 错误: test result: FAILED. 1 passed; 1 failed
# 解决: 修复失败的测试，或临时跳过
cargo test -- --skip failing_test_name

# Java
# 错误: Tests run: 10, Failures: 1
# 解决: 修复测试或使用 -DskipTests
mvn package -DskipTests

# Python
# 错误: 1 failed, 9 passed
# 解决: 修复测试或使用 -k 选项跳过
poetry run pytest -k "not test_failing"
```

#### 错误: 依赖安装失败

```bash
# Rust
# 错误: failed to resolve dependencies
# 解决: 更新 Cargo.lock
cargo update

# Java
# 错误: Could not resolve dependencies
# 解决: 清理并重新下载
mvn dependency:purge-local-repository
mvn clean install

# Python
# 错误: Unable to find a compatible version
# 解决: 更新 lock 文件
poetry update

# Frontend
# 错误: ERESOLVE unable to resolve dependency tree
# 解决: 使用 legacy peer deps
npm install --legacy-peer-deps
```

#### 错误: Docker 构建失败

```bash
# 检查 Dockerfile
docker build -t test-image -f modules/<module>/Dockerfile .

# 常见问题:
# 1. 基础镜像拉取失败
#    解决: 检查网络，或使用镜像源

# 2. COPY 文件不存在
#    解决: 确保文件路径正确，检查 .dockerignore

# 3. 构建上下文过大
#    解决: 优化 .dockerignore
echo "node_modules/" >> .dockerignore
echo "target/" >> .dockerignore
echo ".git/" >> .dockerignore
```

---

### 问题 3: 镜像推送到 ACR 失败

**症状**:
- CI 构建成功但推送失败
- 错误: `unauthorized: authentication required`

**解决方案**:

```bash
# 1. 检查 GitHub Secrets 配置
# 访问: https://github.com/TomXiaoYZ/HermesFlow/settings/secrets/actions
# 确认以下 secrets 存在:
# - ACR_LOGIN_SERVER
# - ACR_USERNAME
# - ACR_PASSWORD

# 2. 测试 ACR 登录
az acr login --name hermesflowdevacr

# 3. 如果登录失败，重新生成 Service Principal
az ad sp create-for-rbac \
  --name hermesflow-acr-push \
  --role acrpush \
  --scopes /subscriptions/<subscription-id>/resourceGroups/hermesflow-dev-rg/providers/Microsoft.ContainerRegistry/registries/hermesflowdevacr

# 4. 更新 GitHub Secrets
# ACR_USERNAME: 上面命令输出的 appId
# ACR_PASSWORD: 上面命令输出的 password
```

---

## GitOps 更新问题

### 问题 4: GitOps 仓库未更新

**症状**:
- CI 成功但 HermesFlow-GitOps 没有新 commit
- `update-gitops.yml` workflow 失败或未运行

**诊断步骤**:

```bash
# 1. 检查 update-gitops workflow 状态
# 访问: https://github.com/TomXiaoYZ/HermesFlow/actions/workflows/update-gitops.yml

# 2. 检查 CI 是否成功完成
# update-gitops 只在 CI workflow 成功后触发

# 3. 检查 artifacts
# CI 应该上传 built-modules artifact
# 访问 CI workflow run → Artifacts 部分

# 4. 检查 GITOPS_PAT secret
# 访问: https://github.com/TomXiaoYZ/HermesFlow/settings/secrets/actions
# 确认 GITOPS_PAT 存在且有效
```

**解决方案**:

```bash
# 重新生成 GitHub PAT
# 1. 访问: https://github.com/settings/tokens
# 2. Generate new token (classic)
# 3. 权限: repo (full control)
# 4. 复制 token
# 5. 更新 GITOPS_PAT secret

# 手动触发 update-gitops（用于测试）
# 访问: https://github.com/TomXiaoYZ/HermesFlow/actions/workflows/update-gitops.yml
# 点击 "Run workflow"
```

---

### 问题 5: GitOps Push 冲突

**症状**:
- `update-gitops.yml` 报错: `rejected - non-fast-forward`
- 多个 CI 同时运行导致冲突

**解决方案**:

update-gitops workflow 已包含自动重试逻辑（最多 5 次），通常会自动解决。

如果仍然失败，手动解决：

```bash
# 1. Clone GitOps 仓库
cd /tmp
git clone https://github.com/TomXiaoYZ/HermesFlow-GitOps.git
cd HermesFlow-GitOps

# 2. Pull 最新变更
git pull origin main

# 3. 手动更新镜像标签
cd apps/dev/<module>/
# 编辑 values.yaml
# 修改 image.tag 为最新的标签

# 4. Commit 并推送
git add values.yaml
git commit -m "chore(dev): update <module> to <tag>"
git push origin main
```

---

## ArgoCD 同步问题

### 问题 6: ArgoCD Application 显示 Unknown 状态

**症状**:
- `kubectl get application` 显示 `SYNC STATUS: Unknown`
- ArgoCD UI 显示同步错误

**诊断步骤**:

```bash
# 1. 查看详细错误信息
kubectl get application <app-name> -n argocd -o yaml | grep -A 20 "message:"

# 2. 常见错误类型
```

**错误类型 A: Authentication Required**

```bash
# 错误: rpc error: code = Unknown desc = authentication required
# 原因: ArgoCD 无法访问 Git 仓库

# 解决方案:
# 1. 检查 ArgoCD Repository 配置
kubectl get secret -n argocd -l argocd.argoproj.io/secret-type=repository

# 2. 重新配置 Repository (通过 ArgoCD UI)
# - Settings → Repositories → + CONNECT REPO
# - URL: https://github.com/TomXiaoYZ/HermesFlow-GitOps
# - Username: TomXiaoYZ
# - Password: GitHub PAT

# 3. 或通过命令行添加
argocd repo add https://github.com/TomXiaoYZ/HermesFlow-GitOps \
  --username TomXiaoYZ \
  --password <github-pat>
```

**错误类型 B: Helm Dependency Error**

```bash
# 错误: directory ../../base-charts/microservice not found
# 原因: Helm dependency 路径错误

# 解决方案:
# 1. 检查 Chart.yaml
cat HermesFlow-GitOps/apps/dev/<module>/Chart.yaml

# 2. 确保路径正确
dependencies:
  - name: hermesflow-microservice
    version: 1.0.0
    repository: "file://../../../base-charts/microservice"  # 3个 ../
```

**错误类型 C: ConfigMap Not Found**

```bash
# 错误: configmap "xxx-config" not found
# 原因: Deployment 引用了不存在的 ConfigMap

# 解决方案:
# 1. 检查 base chart 模板
cat HermesFlow-GitOps/base-charts/microservice/templates/configmap.yaml

# 2. 确保模板存在且条件正确
{{- if .Values.configMap.enabled }}
...
{{- end }}

# 3. 检查 values.yaml 配置
configMap:
  enabled: false  # 如果不需要，设置为 false
```

---

### 问题 7: ArgoCD 自动同步未触发

**症状**:
- GitOps 仓库已更新
- ArgoCD 应用状态仍然 `OutOfSync`
- 等待超过 3 分钟

**解决方案**:

```bash
# 1. 检查 auto-sync 配置
kubectl get application <app-name> -n argocd -o yaml | grep -A 5 "syncPolicy:"

# 应该看到:
# syncPolicy:
#   automated:
#     prune: true
#     selfHeal: true

# 2. 手动触发同步
kubectl patch application <app-name> -n argocd \
  --type merge \
  -p '{"operation":{"sync":{"revision":"HEAD"}}}'

# 3. 强制刷新
kubectl patch application <app-name> -n argocd \
  --type merge \
  -p '{"metadata":{"annotations":{"argocd.argoproj.io/refresh":"hard"}}}'

# 4. 重启 ArgoCD repo-server（清除缓存）
kubectl rollout restart deployment argocd-repo-server -n argocd
```

---

### 问题 8: ArgoCD Sync 成功但 Pod 未更新

**症状**:
- ArgoCD 显示 `Synced` / `Healthy`
- Pod 仍然运行旧镜像

**诊断步骤**:

```bash
# 1. 检查 Deployment 镜像标签
kubectl get deployment <deployment-name> -n hermesflow-dev \
  -o jsonpath='{.spec.template.spec.containers[0].image}'

# 2. 检查 ReplicaSet 镜像
kubectl get rs -n hermesflow-dev -l app.kubernetes.io/name=<app>

# 3. 检查 ArgoCD 应用的 revision
kubectl get application <app-name> -n argocd -o yaml | grep revision:
```

**可能原因**:
1. 镜像标签相同但内容不同（不推荐使用 `latest` tag）
2. Deployment 镜像拉取策略问题

**解决方案**:

```bash
# 1. 确保使用唯一的镜像标签
# CI 生成的标签格式: {branch}-{short_sha}
# 每次构建都应该不同

# 2. 删除 Pod 强制重建
kubectl delete pod -n hermesflow-dev -l app.kubernetes.io/instance=<app>-dev

# 3. 或删除整个 Deployment 让 ArgoCD 重建
kubectl delete deployment <deployment-name> -n hermesflow-dev
# 等待 ArgoCD 自动重建（self-heal）
```

---

## Kubernetes 部署问题

### 问题 9: Pod 状态为 ImagePullBackOff

**症状**:
- `kubectl get pods` 显示 `ImagePullBackOff` 或 `ErrImagePull`

**诊断步骤**:

```bash
# 1. 查看 Pod 详情
kubectl describe pod <pod-name> -n hermesflow-dev | tail -20

# 2. 检查错误信息
# 常见错误:
# - "image not found"
# - "pull access denied"
# - "connection timeout"
```

**解决方案 A: 镜像不存在**

```bash
# 1. 检查 ACR 中是否有该镜像
az acr repository show-tags \
  --name hermesflowdevacr \
  --repository data-engine \
  --orderby time_desc \
  --top 10

# 2. 如果镜像不存在，重新触发 CI
git commit --amend --no-edit
git push -f origin develop

# 3. 或手动构建并推送
cd modules/data-engine
docker build -t hermesflowdevacr.azurecr.io/data-engine:develop-xxx .
az acr login --name hermesflowdevacr
docker push hermesflowdevacr.azurecr.io/data-engine:develop-xxx
```

**解决方案 B: ACR 认证失败**

```bash
# 1. 检查 AKS 到 ACR 的连接
az aks check-acr \
  --name hermesflow-dev-aks \
  --resource-group hermesflow-dev-rg \
  --acr hermesflowdevacr.azurecr.io

# 2. 重新附加 ACR（如果需要）
az aks update \
  --name hermesflow-dev-aks \
  --resource-group hermesflow-dev-rg \
  --attach-acr hermesflowdevacr

# 3. 或创建 imagePullSecret
kubectl create secret docker-registry acr-secret \
  --docker-server=hermesflowdevacr.azurecr.io \
  --docker-username=<sp-id> \
  --docker-password=<sp-password> \
  -n hermesflow-dev
```

---

### 问题 10: Pod CrashLoopBackOff

**症状**:
- Pod 启动后立即崩溃
- 状态显示 `CrashLoopBackOff`

**诊断步骤**:

```bash
# 1. 查看 Pod 日志
kubectl logs <pod-name> -n hermesflow-dev --previous

# 2. 查看 Pod 事件
kubectl describe pod <pod-name> -n hermesflow-dev | tail -30

# 3. 检查容器退出代码
kubectl get pod <pod-name> -n hermesflow-dev -o yaml | grep -A 5 "containerStatuses:"
```

**常见原因和解决方案**:

```bash
# 1. 配置错误（环境变量、ConfigMap）
# 检查环境变量
kubectl get deployment <deployment> -n hermesflow-dev -o yaml | grep -A 20 "env:"

# 2. 健康检查失败
# 临时禁用健康检查进行调试
# 编辑 values.yaml:
healthCheck:
  enabled: false

# 3. 应用程序错误
# 查看应用程序日志找到具体错误
kubectl logs <pod-name> -n hermesflow-dev | tail -100

# 4. 权限问题
# 检查 SecurityContext
kubectl get pod <pod-name> -n hermesflow-dev -o yaml | grep -A 10 "securityContext:"

# 5. 资源不足
# 检查节点资源
kubectl top nodes
kubectl describe node <node-name> | grep -A 10 "Allocated resources:"
```

---

### 问题 11: Service 无法访问

**症状**:
- Pod 运行正常
- 但无法通过 Service 访问

**诊断步骤**:

```bash
# 1. 检查 Service 配置
kubectl get svc -n hermesflow-dev
kubectl describe svc <service-name> -n hermesflow-dev

# 2. 检查 Endpoints
kubectl get endpoints <service-name> -n hermesflow-dev

# 3. 测试 Pod 内部网络
kubectl exec -it <pod-name> -n hermesflow-dev -- curl http://localhost:8080/health
```

**解决方案**:

```bash
# 1. Endpoints 为空 → Label selector 不匹配
kubectl get pods -n hermesflow-dev --show-labels
kubectl get svc <service-name> -n hermesflow-dev -o yaml | grep -A 5 "selector:"

# 2. 端口不匹配
# 检查 Service port 和 targetPort
kubectl get svc <service-name> -n hermesflow-dev -o yaml | grep -A 3 "ports:"

# 3. 健康检查失败导致 Pod 不在 Endpoints
kubectl describe pod <pod-name> -n hermesflow-dev | grep -A 5 "Readiness:"
```

---

## 网络和认证问题

### 问题 12: GitHub Actions 网络超时

**症状**:
- Git push/pull 超时
- Docker push 超时
- `Operation timed out`

**解决方案**:

```bash
# 1. 检查 GitHub 状态
# 访问: https://www.githubstatus.com/

# 2. 使用重试逻辑
# update-gitops.yml 已包含重试（最多 5 次）

# 3. 如果持续失败，等待网络恢复后手动重试
# 访问 GitHub Actions 失败的 workflow
# 点击 "Re-run failed jobs"
```

---

### 问题 13: kubectl 命令报错 Unauthorized

**症状**:
- `error: You must be logged in to the server (Unauthorized)`

**解决方案**:

```bash
# 1. 检查当前 context
kubectl config current-context

# 2. 重新登录 AKS
az aks get-credentials \
  --resource-group hermesflow-dev-rg \
  --name hermesflow-dev-aks \
  --overwrite-existing

# 3. 如果使用 Azure AD，可能需要重新认证
kubelogin convert-kubeconfig -l azurecli

# 4. 验证连接
kubectl get nodes
```

---

## 回滚流程

### 标准回滚流程

```bash
# 步骤 1: 确定要回滚到的版本
cd /path/to/HermesFlow-GitOps
git log --oneline apps/dev/<module>/values.yaml | head -10

# 步骤 2: 查看某个版本的镜像标签
git show <commit-sha>:apps/dev/<module>/values.yaml | grep tag:

# 步骤 3: 方法 A - 使用 git revert
git revert <bad-commit-sha>
git push origin main

# 或方法 B - 手动修改
# 编辑 apps/dev/<module>/values.yaml
# 修改 tag 为之前的版本
git add apps/dev/<module>/values.yaml
git commit -m "chore(dev): rollback <module> to <previous-tag>"
git push origin main

# 步骤 4: 等待 ArgoCD 自动同步（1-3 分钟）
# 或手动触发同步
kubectl patch application <module>-dev -n argocd \
  --type merge \
  -p '{"operation":{"sync":{"revision":"HEAD"}}}'

# 步骤 5: 验证回滚
kubectl get pods -n hermesflow-dev -l app.kubernetes.io/instance=<module>-dev
kubectl get deployment <module>-dev-hermesflow-microservice -n hermesflow-dev \
  -o jsonpath='{.spec.template.spec.containers[0].image}'
```

### 紧急回滚（绕过 GitOps）

**仅在 GitOps 流程不可用时使用**

```bash
# 1. 直接修改 Deployment 镜像
kubectl set image deployment/<deployment-name> \
  <container-name>=hermesflowdevacr.azurecr.io/<module>:<old-tag> \
  -n hermesflow-dev

# 2. 验证
kubectl rollout status deployment/<deployment-name> -n hermesflow-dev

# 3. 之后务必更新 GitOps 仓库，否则 ArgoCD 会回滚你的更改
```

---

## 紧急修复步骤

### 场景: 生产环境故障，需要立即修复

**步骤 1: 评估影响**

```bash
# 检查受影响的服务
kubectl get pods -n hermesflow-main --field-selector=status.phase!=Running

# 查看错误日志
kubectl logs -n hermesflow-main -l app=<affected-app> --tail=50

# 检查监控指标
# 访问 Grafana Dashboard
```

**步骤 2: 快速回滚（如果最近部署导致）**

```bash
# 使用上述回滚流程
# 或直接 kubectl set image
```

**步骤 3: 热修复（如果需要代码修复）**

```bash
# 1. 从 main 创建 hotfix 分支
git checkout main
git pull origin main
git checkout -b hotfix/critical-bug

# 2. 快速修复代码
# ... 修改代码 ...

# 3. 提交并推送
git add .
git commit -m "[module: <module>] hotfix: 修复关键Bug"
git push origin hotfix/critical-bug

# 4. 合并到 main 并触发部署
git checkout main
git merge hotfix/critical-bug
git push origin main

# 5. 同时合并回 develop
git checkout develop
git merge hotfix/critical-bug
git push origin develop

# 6. 清理 hotfix 分支
git branch -d hotfix/critical-bug
git push origin --delete hotfix/critical-bug
```

**步骤 4: 通知和跟踪**

```bash
# 1. 记录事件
# 创建 incident report

# 2. 通知相关人员
# Slack/Email 通知

# 3. 监控修复效果
# 持续观察日志和指标
```

---

## 查看日志

### ArgoCD 日志

```bash
# Application Controller (负责同步)
kubectl logs -n argocd deployment/argocd-application-controller --tail=100

# Repo Server (负责渲染 manifests)
kubectl logs -n argocd deployment/argocd-repo-server --tail=100

# Server (API 和 UI)
kubectl logs -n argocd deployment/argocd-server --tail=100
```

### GitHub Actions 日志

```bash
# 通过浏览器访问
# https://github.com/TomXiaoYZ/HermesFlow/actions

# 或使用 GitHub CLI
gh run list --workflow=ci-rust.yml --limit=5
gh run view <run-id> --log
```

### Pod 应用日志

```bash
# 当前 Pod 日志
kubectl logs -n hermesflow-dev <pod-name>

# 之前崩溃的 Pod 日志
kubectl logs -n hermesflow-dev <pod-name> --previous

# 实时跟踪
kubectl logs -n hermesflow-dev <pod-name> -f

# 所有匹配的 Pod 日志
kubectl logs -n hermesflow-dev -l app.kubernetes.io/instance=<app>-dev --tail=50
```

---

## 联系支持

如果以上方法无法解决问题：

1. **检查文档**: [CI/CD Workflow Guide](../development/cicd-workflow.md)
2. **查看 User Story**: [DEVOPS-003](../stories/sprint-01/DEVOPS-003-argocd-gitops.md)
3. **提交 Issue**: [GitHub Issues](https://github.com/TomXiaoYZ/HermesFlow/issues)

---

## 参考资料

- [ArgoCD Documentation](https://argo-cd.readthedocs.io/)
- [Kubernetes Troubleshooting](https://kubernetes.io/docs/tasks/debug/)
- [GitHub Actions Documentation](https://docs.github.com/en/actions)
- [Azure AKS Troubleshooting](https://docs.microsoft.com/en-us/azure/aks/troubleshooting)

---

**维护者**: DevOps Team  
**最后更新**: 2025-10-21

