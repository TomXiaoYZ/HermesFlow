# CI/CD 使用指南

## 🚀 快速开始

### 基本用法

在提交代码时，在 commit message 中添加 `[module: xxx]` 来指定要构建的模块：

```bash
# 只构建 data-engine
git commit -m "[module: data-engine] Add new feature"
git push origin develop

# 只构建 frontend
git commit -m "[module: frontend] Update UI components"
git push origin develop

# 构建所有模块
git commit -m "[module: all] Update dependencies"
git push origin develop

# 不指定模块（默认构建所有）
git commit -m "chore: update configuration"
git push origin develop
```

---

## 📦 支持的模块

### Rust Services
- `data-engine` - 数据引擎服务
- `gateway` - API 网关服务

### Java Services
- `user-management` - 用户管理服务
- `api-gateway` - API 网关服务
- `trading-engine` - 交易引擎服务

### Python Services
- `strategy-engine` - 策略引擎服务
- `backtest-engine` - 回测引擎服务
- `risk-engine` - 风险引擎服务

### Frontend
- `frontend` - 前端应用

---

## 🔄 工作流程

```
提交代码 → CI 触发 → 构建镜像 → 推送到 ACR → 更新 GitOps → ArgoCD 部署
```

### 详细步骤

1. **开发阶段**
   ```bash
   # 在 feature 分支开发
   git checkout -b feature/add-new-api
   
   # 提交代码（feature 分支不触发 CI）
   git commit -m "feat: add new API endpoint"
   git push origin feature/add-new-api
   ```

2. **测试阶段**
   ```bash
   # 合并到 develop 分支（触发 Dev 环境 CI/CD）
   git checkout develop
   git merge feature/add-new-api
   
   # 添加模块标记
   git commit --amend -m "[module: data-engine] feat: add new API endpoint"
   git push origin develop
   ```

3. **生产部署**
   ```bash
   # 合并到 main 分支（触发 Prod 环境 CI/CD）
   git checkout main
   git merge develop
   git push origin main
   ```

---

## 🧪 测试 CI/CD 流程

### 使用测试脚本（推荐）

```bash
# 运行交互式测试脚本
./test-cicd-flow.sh

# 脚本会引导你:
# 1. 选择要测试的模块
# 2. 自动创建测试提交
# 3. 推送到远程仓库
# 4. 显示验证步骤
```

### 手动测试

```bash
# 测试单个模块
git commit --allow-empty -m "[module: data-engine] 测试 CI/CD"
git push origin develop

# 测试所有模块
git commit --allow-empty -m "[module: all] 测试所有模块"
git push origin develop
```

---

## 📊 查看构建状态

### GitHub Actions

访问 [GitHub Actions](https://github.com/TomXiaoYZ/HermesFlow/actions) 查看：
- ✅ CI workflows 运行状态
- 📦 构建的模块
- 🐳 Docker 镜像 tags
- ⏱️ 构建时间

### GitOps 仓库更新

```bash
# 检查 GitOps 仓库更新
cd ../HermesFlow-GitOps
git pull origin main
git log --oneline -5

# 查看具体的镜像更新
git show HEAD:apps/dev/data-engine/values.yaml
```

### ArgoCD 部署状态

```bash
# 查看 ArgoCD 应用状态
kubectl get applications -n argocd

# 查看具体服务的 Pods
kubectl get pods -n hermesflow-dev

# 查看 Pod 详情（包括镜像版本）
kubectl describe pod <pod-name> -n hermesflow-dev
```

---

## ⚠️ 常见问题

### Q1: 为什么我的 commit 没有触发 CI？

**A**: 检查以下几点：
1. 是否在 `main` 或 `develop` 分支？（feature 分支不触发 CI）
2. Commit message 格式是否正确？
3. GitHub Actions 是否启用？

### Q2: 如何同时构建多个模块？

**A**: 有两种方式：
1. 使用 `[module: all]` 构建所有模块
2. 拆分成多个 commits，每个指定一个模块

### Q3: GitOps 更新失败怎么办？

**A**: 
1. 检查 `GITOPS_PAT` secret 是否配置
2. 查看 update-gitops workflow 日志
3. 网络重试机制会自动重试 5 次

### Q4: 如何查看构建的 Docker 镜像？

**A**:
```bash
# 登录 Azure Container Registry
az acr login --name hermesflowdevacr

# 列出镜像
az acr repository list --name hermesflowdevacr --output table

# 查看特定镜像的 tags
az acr repository show-tags --name hermesflowdevacr --repository data-engine --output table
```

### Q5: 如何回滚到之前的版本？

**A**:
```bash
# 方法 1: 通过 GitOps 仓库回滚
cd ../HermesFlow-GitOps
git revert HEAD
git push origin main

# 方法 2: 手动修改 values.yaml
# 编辑 apps/dev/<module>/values.yaml
# 修改 image.tag 为之前的版本
git commit -m "chore: rollback <module> to previous version"
git push origin main
```

---

## 🔧 高级用法

### 自定义构建行为

如果需要更复杂的构建逻辑，可以：

1. **修改 parse-commit job**
   - 支持更复杂的 commit message 格式
   - 添加额外的构建参数

2. **添加环境变量**
   - 在 CI workflows 中添加环境变量
   - 控制构建行为

3. **自定义 Docker 构建参数**
   - 修改 `docker build` 命令
   - 添加构建参数

### 调试 CI/CD

```bash
# 启用调试日志
# 在 GitHub Actions secrets 中添加:
# ACTIONS_STEP_DEBUG = true
# ACTIONS_RUNNER_DEBUG = true

# 查看详细的构建日志
# 访问 GitHub Actions → 选择 workflow → 查看步骤详情
```

---

## 📚 相关文档

- [完整实施报告](docs/qa/commit-based-cicd-implementation-report.md)
- [ArgoCD 部署指南](docs/stories/sprint-01/DEVOPS-003-argocd-gitops.md)
- [开发者快速指南](docs/development/developer-quickstart.md)

---

## 💡 最佳实践

### Commit Message

1. **使用语义化提交**
   ```bash
   [module: data-engine] feat: add new API endpoint
   [module: frontend] fix: resolve login issue
   [module: all] chore: update dependencies
   ```

2. **明确的模块标记**
   ```bash
   # ✅ 好的示例
   [module: data-engine] feat: add caching layer
   
   # ❌ 不好的示例
   [module:data-engine] add feature  # 缺少空格
   [Module: Data-Engine] add feature # 大小写错误
   ```

### 分支策略

1. **Feature 分支**: 
   - 命名: `feature/description`
   - 不触发 CI
   - 用于开发和本地测试

2. **Develop 分支**:
   - 触发 Dev 环境 CI/CD
   - 自动部署到开发环境
   - 用于集成测试

3. **Main 分支**:
   - 触发 Prod 环境 CI/CD
   - 自动部署到生产环境
   - 只合并经过测试的代码

### 部署策略

1. **小步迭代**
   - 每次只部署一个模块
   - 便于问题定位和回滚

2. **充分测试**
   - 在 Dev 环境充分测试
   - 确认无误后再部署到 Prod

3. **监控部署**
   - 部署后检查 Pod 状态
   - 查看应用日志
   - 验证健康检查

---

## 🆘 获取帮助

如有问题：
1. 查看 [FAQ](docs/faq.md)
2. 创建 [GitHub Issue](https://github.com/TomXiaoYZ/HermesFlow/issues)
3. 查看 [Troubleshooting Guide](docs/operations/troubleshooting.md)

---

**最后更新**: 2025-10-20  
**版本**: 1.0.0

