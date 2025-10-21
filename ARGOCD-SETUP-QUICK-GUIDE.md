# ArgoCD GitHub 访问配置 - 快速指南

**目标**: 配置 ArgoCD 访问私有 GitOps 仓库，实现自动同步部署

---

## 🚀 快速配置（5分钟）

### 步骤 1: 创建 GitHub Token

1. 访问: https://github.com/settings/tokens
2. 点击 **"Generate new token (classic)"**
3. 配置:
   - **Note**: `ArgoCD-HermesFlow-GitOps`
   - **Expiration**: `90 days` 或 `No expiration`
   - **权限**: ✅ **repo** (只需勾选这一个！)
4. 点击 **"Generate token"**
5. ⚠️ **立即复制 Token**（只显示一次！格式：`ghp_xxx...`）

---

### 步骤 2: 配置 ArgoCD（推荐使用 UI 方式）

#### 2.1 获取 ArgoCD 密码

```bash
# macOS 系统（注意是大写 -D）
kubectl get secret argocd-initial-admin-secret -n argocd \
  -o jsonpath="{.data.password}" | base64 -D && echo

# Linux 系统（小写 -d）
kubectl get secret argocd-initial-admin-secret -n argocd \
  -o jsonpath="{.data.password}" | base64 -d && echo
```

**您的密码**: `zt9mQigLG025oy0t`

#### 2.2 开启端口转发

```bash
kubectl port-forward svc/argocd-server -n argocd 8443:443
```

保持这个终端窗口打开！

#### 2.3 访问 ArgoCD UI

在浏览器中打开: https://localhost:8443

⚠️ **证书警告是正常的**，点击"继续访问"或"高级" → "继续"

#### 2.4 登录

- **Username**: `admin`
- **Password**: `zt9mQigLG025oy0t` (上面获取的密码)

#### 2.5 添加 GitOps 仓库

1. 点击左侧菜单 **"Settings"** (⚙️ 齿轮图标)
2. 点击 **"Repositories"**
3. 点击右上角 **"+ CONNECT REPO"**
4. 填写表单:

| 字段 | 值 |
|------|-----|
| **Choose your connection method** | `HTTPS` |
| **Type** | `git` |
| **Project** | `default` |
| **Repository URL** | `https://github.com/TomXiaoYZ/HermesFlow-GitOps` |
| **Username** | `TomXiaoYZ` |
| **Password** | 您在步骤1创建的 Token (`ghp_xxx...`) |

5. 点击 **"CONNECT"**

**成功标志**: 
- ✅ 连接状态显示 "**Successful**"
- ✅ 可以看到仓库内容

---

### 步骤 3: 验证配置

#### 3.1 检查 Application 状态

```bash
kubectl get application data-engine-dev -n argocd
```

**预期输出**:
```
NAME               SYNC STATUS   HEALTH STATUS
data-engine-dev    Synced        Healthy
```

如果仍然显示 `Unknown`，继续下一步。

#### 3.2 手动触发同步

```bash
kubectl patch application data-engine-dev -n argocd \
  --type merge \
  -p '{"operation":{"sync":{"revision":"HEAD"}}}'
```

#### 3.3 等待并重新检查

```bash
# 等待 10-20 秒
sleep 15

# 重新检查
kubectl get application data-engine-dev -n argocd
```

---

## 🎯 备选方案：使用命令行配置

如果您更喜欢命令行方式：

```bash
# 1. 创建 Secret（替换 <YOUR_GITHUB_TOKEN> 为实际 token）
kubectl create secret generic hermesflow-gitops-repo \
  -n argocd \
  --from-literal=type=git \
  --from-literal=url=https://github.com/TomXiaoYZ/HermesFlow-GitOps \
  --from-literal=password=<YOUR_GITHUB_TOKEN> \
  --from-literal=username=TomXiaoYZ

# 2. 添加 Label
kubectl label secret hermesflow-gitops-repo \
  -n argocd \
  argocd.argoproj.io/secret-type=repository

# 3. 验证
kubectl get secret hermesflow-gitops-repo -n argocd
```

---

## ✅ 成功标志

配置成功后，您应该看到：

1. **ArgoCD UI**:
   - ✅ Repositories 页面显示 GitOps 仓库
   - ✅ 连接状态: "Successful"

2. **Application 状态**:
   ```bash
   kubectl get application data-engine-dev -n argocd
   ```
   - ✅ SYNC STATUS: `Synced` (不再是 `Unknown`)
   - ✅ HEALTH STATUS: `Healthy`

3. **Pod 运行正常**:
   ```bash
   kubectl get pods -n hermesflow-dev -l app.kubernetes.io/name=data-engine
   ```

---

## 🧪 测试完整 CI/CD 流程

配置完成后，测试完整流程：

```bash
cd /Users/tomxiao/Desktop/Git/personal/HermesFlow

# 1. 创建测试 commit
git commit --allow-empty -m "[module: data-engine] 测试 ArgoCD 自动同步"
git push origin develop

# 2. 等待 CI 完成（3-4 分钟）
# 可以访问 https://github.com/TomXiaoYZ/HermesFlow/actions 查看进度

# 3. 等待 1-2 分钟后检查 pod
kubectl get pods -n hermesflow-dev -l app.kubernetes.io/name=data-engine

# 4. 检查镜像标签是否更新
kubectl get deployment data-engine -n hermesflow-dev \
  -o jsonpath='{.spec.template.spec.containers[0].image}'
```

**预期**: 镜像标签自动更新为最新的 `develop-xxxxx` 格式！

---

## ⚠️ 常见问题

### 问题 1: 无法获取密码

**错误**: `base64: invalid option -- d`

**原因**: macOS 使用大写 `-D`，Linux 使用小写 `-d`

**解决**:
```bash
# macOS
base64 -D

# Linux
base64 -d
```

### 问题 2: 连接 GitOps 仓库失败

**错误**: "authentication failed"

**检查**:
1. Token 是否正确复制（完整的 `ghp_xxx...`）
2. Token 是否有 `repo` 权限
3. 仓库 URL 是否正确

### 问题 3: Application 仍显示 Unknown

**解决**:
```bash
# 重启 ArgoCD repo server
kubectl rollout restart deployment argocd-repo-server -n argocd

# 等待 30 秒
sleep 30

# 重新检查
kubectl get application data-engine-dev -n argocd
```

---

## 📚 相关资源

- **详细文档**: 查看仓库根目录（如果需要更多细节）
- **GitHub Actions**: https://github.com/TomXiaoYZ/HermesFlow/actions
- **ArgoCD 文档**: https://argo-cd.readthedocs.io/

---

## 🎊 配置完成！

配置成功后，您的 CI/CD 流程将是：

```
git push → CI 构建 → 推送镜像 → 更新 GitOps → ArgoCD 自动同步 → 部署完成
```

**完全自动化！只需一个 `git push` 命令！** 🚀

---

**当前 Admin 密码**: `zt9mQigLG025oy0t`  
**ArgoCD UI**: https://localhost:8443 (需先启动 port-forward)

