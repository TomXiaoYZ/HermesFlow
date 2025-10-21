# GITOPS_PAT Secret 配置指南

## 🔍 问题诊断

**当前状态**:
- ✅ Rust CI 成功
- ✅ Artifact 下载成功
- ✅ 模块识别成功 (`data-engine`)
- ❌ **GitOps 仓库 checkout 失败**

**错误信息**:
```
Input required and not supplied: token
```

## 🎯 根本原因

移除 `environment` 后，workflow 无法访问 environment secrets。需要将 `GITOPS_PAT` 配置为 **Repository Secret**。

## ✅ 解决方案

### 步骤 1: 创建 Personal Access Token (如果还没有)

1. 访问 GitHub Settings → Developer settings → Personal access tokens → Tokens (classic)
2. 点击 "Generate new token (classic)"
3. Token 名称: `HermesFlow-GitOps`
4. 选择权限:
   - ✅ `repo` (完整仓库访问)
   - ✅ `workflow` (更新 workflow 文件)
5. 点击 "Generate token"
6. **复制 token 值** (只显示一次！)

### 步骤 2: 添加 Repository Secret

#### 在 HermesFlow 仓库中:
1. 访问 https://github.com/TomXiaoYZ/HermesFlow/settings/secrets/actions
2. 点击 "New repository secret"
3. Name: `GITOPS_PAT`
4. Value: 粘贴刚才复制的 token
5. 点击 "Add secret"

## 🧪 验证配置

运行以下命令测试 (无法直接检查 secret，但可以通过 workflow 测试):

```bash
# 触发一个新的测试
cd /Users/tomxiao/Desktop/Git/personal/HermesFlow
git commit --allow-empty -m "[module: data-engine] 测试 GITOPS_PAT 配置"
git push origin develop
```

然后监控 workflow：
```bash
# 等待并检查
sleep 60
gh run list --limit 5
```

## 📋 配置检查清单

- [ ] Personal Access Token 已创建
- [ ] Token 有 `repo` 和 `workflow` 权限
- [ ] `GITOPS_PAT` 已添加到 Repository Secrets
- [ ] Secret 名称完全匹配 (区分大小写)
- [ ] Token 未过期

## 🔄 当前流程状态

```
1. ✅ parse-commit 解析模块
2. ✅ 只构建 data-engine  
3. ✅ 上传 artifact
4. ✅ update-gitops 下载 artifact
5. ❌ Checkout GitOps 仓库 (需要 GITOPS_PAT)
6. ⏳ 更新 image tags
7. ⏳ Commit and push
8. ⏳ ArgoCD 自动同步
```

## 💡 备注

如果已经有 GITOPS_PAT 在 environment secrets 中，需要将它**复制**到 repository secrets。两者是独立的配置。

