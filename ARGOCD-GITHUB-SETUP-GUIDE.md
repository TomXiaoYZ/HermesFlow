# ArgoCD GitHub 访问配置详细指南

**目标**: 配置 ArgoCD 访问私有 GitOps 仓库，实现自动同步部署

**当前问题**: `Failed to load target state: authentication required`

---

## 📋 前置条件检查

在开始之前，请确认：

```bash
# 1. 检查 kubectl 连接
kubectl config use-context hermesflow-dev-aks-admin
kubectl get pods -n argocd

# 2. 确认 ArgoCD 运行正常
kubectl get application data-engine-dev -n argocd

# 3. 检查当前错误
kubectl get application data-engine-dev -n argocd -o yaml | grep -A 5 "message:"
```

**预期看到**: `authentication required` 错误

---

## 🔧 配置步骤

### 步骤 1: 创建 GitHub Personal Access Token

#### 1.1 访问 GitHub Token 页面

打开浏览器，访问：
```
https://github.com/settings/tokens
```

或者：
1. 点击右上角头像
2. Settings
3. 左侧菜单最底部：Developer settings
4. Personal access tokens → Tokens (classic)

#### 1.2 创建新 Token

点击 **"Generate new token (classic)"**

**配置项**:

| 字段 | 值 |
|------|-----|
| Note | `ArgoCD-HermesFlow-GitOps` (方便识别) |
| Expiration | 建议：90 days 或 No expiration |
| 权限 | ✅ **repo** (完整仓库访问) |

**重要**: 只需要勾选 `repo`，其他权限不需要！

```
✅ repo
   ✅ repo:status
   ✅ repo_deployment
   ✅ public_repo
   ✅ repo:invite
   ✅ security_events
```

#### 1.3 保存 Token

点击 **"Generate token"**

⚠️ **非常重要**: 
- Token 只显示一次！
- 立即复制并保存到安全的地方
- 格式类似：`ghp_xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx`

---

### 步骤 2: 在 ArgoCD 中配置 GitHub 凭据

#### 方法 A: 使用 ArgoCD UI (推荐，更直观)

##### A.1 访问 ArgoCD UI

```bash
# 开启端口转发
kubectl port-forward svc/argocd-server -n argocd 8443:443
```

在浏览器中访问：`https://localhost:8443`

⚠️ 浏览器会警告证书不安全，点击"继续访问"即可

##### A.2 登录 ArgoCD

**获取初始密码**:
```bash
kubectl get secret argocd-initial-admin-secret -n argocd \
  -o jsonpath="{.data.password}" | base64 -d && echo
```

**登录信息**:
- Username: `admin`
- Password: 上面命令的输出

##### A.3 添加 Repository

1. 点击左侧菜单 **"Settings"** (齿轮图标)
2. 点击 **"Repositories"**
3. 点击右上角 **"+ CONNECT REPO"**

**填写表单**:

| 字段 | 值 |
|------|-----|
| Choose your connection method | **HTTPS** |
| Type | **git** |
| Project | **default** (或 hermesflow) |
| Repository URL | `https://github.com/TomXiaoYZ/HermesFlow-GitOps` |
| Username | `TomXiaoYZ` (您的 GitHub 用户名) |
| Password | `ghp_xxx...` (刚才创建的 Token) |

4. 点击 **"CONNECT"**

**验证成功标志**:
- 状态显示 "Successful"
- 可以看到 GitOps 仓库的内容

---

#### 方法 B: 使用 kubectl (命令行方式)

##### B.1 创建 Secret

```bash
# 替换 <YOUR_GITHUB_TOKEN> 为实际的 token
kubectl create secret generic hermesflow-gitops-repo \
  -n argocd \
  --from-literal=type=git \
  --from-literal=url=https://github.com/TomXiaoYZ/HermesFlow-GitOps \
  --from-literal=password=<YOUR_GITHUB_TOKEN> \
  --from-literal=username=TomXiaoYZ
```

**示例**:
```bash
kubectl create secret generic hermesflow-gitops-repo \
  -n argocd \
  --from-literal=type=git \
  --from-literal=url=https://github.com/TomXiaoYZ/HermesFlow-GitOps \
  --from-literal=password=ghp_AbCdEfGhIjKlMnOpQrStUvWxYz1234567890 \
  --from-literal=username=TomXiaoYZ
```

##### B.2 添加 Label

```bash
kubectl label secret hermesflow-gitops-repo \
  -n argocd \
  argocd.argoproj.io/secret-type=repository
```

##### B.3 验证 Secret

```bash
kubectl get secret hermesflow-gitops-repo -n argocd -o yaml
```

**预期输出**:
```yaml
apiVersion: v1
kind: Secret
metadata:
  labels:
    argocd.argoproj.io/secret-type: repository
  name: hermesflow-gitops-repo
  namespace: argocd
type: Opaque
data:
  password: Z2hwX...  # Base64 编码
  type: Z2l0
  url: aHR0cHM6...
  username: VG9t...
```

---

### 步骤 3: 验证配置

#### 3.1 检查 Repository 连接

```bash
# 使用 ArgoCD CLI (如果已安装)
argocd repo list

# 或使用 kubectl
kubectl get secret -n argocd -l argocd.argoproj.io/secret-type=repository
```

#### 3.2 强制同步 Application

```bash
# 方法 1: 使用 kubectl
kubectl patch application data-engine-dev \
  -n argocd \
  --type merge \
  -p '{"operation":{"initiatedBy":{"username":"admin"},"sync":{"revision":"HEAD"}}}'

# 方法 2: 使用 ArgoCD UI
# 在 Applications 页面，点击 data-engine-dev → SYNC
```

#### 3.3 检查同步状态

```bash
# 等待 10-20 秒后检查
kubectl get application data-engine-dev -n argocd -o jsonpath='{.status.sync.status}'
```

**预期输出**: `Synced` 或 `OutOfSync` (不再是 `Unknown`)

#### 3.4 查看详细状态

```bash
kubectl get application data-engine-dev -n argocd -o yaml | grep -A 20 "status:"
```

**成功标志**:
```yaml
status:
  sync:
    status: Synced
    revision: 03d36c2...
  health:
    status: Healthy
```

---

### 步骤 4: 验证完整流程

#### 4.1 检查 Pod 状态

```bash
# 查看 data-engine pod
kubectl get pods -n hermesflow-dev -l app.kubernetes.io/name=data-engine

# 查看 Pod 详情
kubectl describe pod -n hermesflow-dev -l app.kubernetes.io/name=data-engine
```

#### 4.2 检查镜像标签

```bash
kubectl get deployment data-engine -n hermesflow-dev \
  -o jsonpath='{.spec.template.spec.containers[0].image}'
```

**预期输出**: `hermesflowdevacr.azurecr.io/data-engine:develop-fb88d7e`

#### 4.3 测试端到端流程

```bash
# 1. 提交新的测试 commit
cd /Users/tomxiao/Desktop/Git/personal/HermesFlow
git commit --allow-empty -m "[module: data-engine] 测试 ArgoCD 自动同步"
git push origin develop

# 2. 等待 CI 完成 (3-4分钟)
sleep 240

# 3. 检查 GitOps 仓库更新
cd /Users/tomxiao/Desktop/Git/personal/HermesFlow-GitOps
git pull origin main
git log -1 --oneline

# 4. 等待 ArgoCD 同步 (1-2分钟，如果配置了 auto-sync)
sleep 60

# 5. 验证 pod 更新
kubectl get pods -n hermesflow-dev -l app.kubernetes.io/name=data-engine
```

---

## 🔍 故障排查

### 问题 1: Token 无效

**症状**: `authentication failed` 或 `bad credentials`

**解决**:
```bash
# 1. 检查 token 是否正确
kubectl get secret hermesflow-gitops-repo -n argocd \
  -o jsonpath='{.data.password}' | base64 -d && echo

# 2. 更新 secret
kubectl delete secret hermesflow-gitops-repo -n argocd
# 重新创建 (使用新 token)
```

### 问题 2: 仓库 URL 错误

**症状**: `repository not found`

**解决**:
```bash
# 检查 URL
kubectl get secret hermesflow-gitops-repo -n argocd \
  -o jsonpath='{.data.url}' | base64 -d && echo

# 应该是: https://github.com/TomXiaoYZ/HermesFlow-GitOps
```

### 问题 3: Application 仍然 Unknown

**症状**: `sync status: Unknown`

**可能原因**:
1. repoURL 配置错误
2. ArgoCD 缓存问题

**解决**:
```bash
# 1. 检查 Application 配置
kubectl get application data-engine-dev -n argocd -o yaml | grep repoURL

# 2. 重启 ArgoCD repo server
kubectl rollout restart deployment argocd-repo-server -n argocd

# 3. 等待 30 秒后重新检查
sleep 30
kubectl get application data-engine-dev -n argocd
```

### 问题 4: 权限不足

**症状**: `permission denied`

**解决**:
- 确认 GitHub Token 有 `repo` 权限
- 确认用户对 GitOps 仓库有访问权限
- Token 未过期

---

## 📊 配置验证清单

完成所有步骤后，请验证：

- [ ] GitHub Personal Access Token 已创建
- [ ] Token 有 `repo` 权限
- [ ] Token 已保存到安全位置
- [ ] ArgoCD 中已添加 Repository
- [ ] Repository 连接状态为 "Successful"
- [ ] `data-engine-dev` Application 状态不再是 "Unknown"
- [ ] Application 能够成功同步
- [ ] Pod 使用了正确的镜像标签
- [ ] 端到端流程测试通过

---

## 🎯 快速命令参考

```bash
# 1. 获取 ArgoCD 密码
kubectl get secret argocd-initial-admin-secret -n argocd \
  -o jsonpath="{.data.password}" | base64 -d && echo

# 2. 端口转发 ArgoCD UI
kubectl port-forward svc/argocd-server -n argocd 8443:443

# 3. 创建 GitHub 凭据
kubectl create secret generic hermesflow-gitops-repo \
  -n argocd \
  --from-literal=type=git \
  --from-literal=url=https://github.com/TomXiaoYZ/HermesFlow-GitOps \
  --from-literal=password=<YOUR_TOKEN> \
  --from-literal=username=TomXiaoYZ

# 4. 添加 Label
kubectl label secret hermesflow-gitops-repo \
  -n argocd \
  argocd.argoproj.io/secret-type=repository

# 5. 检查状态
kubectl get application data-engine-dev -n argocd

# 6. 强制同步
kubectl patch application data-engine-dev -n argocd \
  --type merge \
  -p '{"operation":{"sync":{"revision":"HEAD"}}}'

# 7. 查看 Pod
kubectl get pods -n hermesflow-dev -l app.kubernetes.io/name=data-engine
```

---

## 💡 最佳实践

1. **Token 管理**
   - 使用专门的 Token 给 ArgoCD
   - 定期轮换 Token
   - 不要分享 Token

2. **权限最小化**
   - 只授予必要的 `repo` 权限
   - 考虑使用 GitHub App (更安全)

3. **监控**
   - 定期检查 ArgoCD 同步状态
   - 设置告警通知

4. **备份**
   - 保存 Token 到安全的密码管理器
   - 记录配置步骤

---

## 📞 需要帮助？

如果遇到问题，请检查：

1. ArgoCD 日志：
   ```bash
   kubectl logs -n argocd deployment/argocd-repo-server
   ```

2. Application 事件：
   ```bash
   kubectl get events -n argocd --field-selector involvedObject.name=data-engine-dev
   ```

3. ArgoCD 服务状态：
   ```bash
   kubectl get pods -n argocd
   ```

---

**配置完成后，您的 CI/CD 流程将完全自动化！** 🎉

从 `git push` 到 Kubernetes 部署，一切都将自动完成，无需人工干预！

