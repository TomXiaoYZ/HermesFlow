# DevOps 工程师指南

> **HermesFlow 部署和运维指南**

---

## 🎯 DevOps 职责

1. ✅ 管理 CI/CD 流水线（GitHub Actions）
2. ✅ 管理 Kubernetes 集群（Azure AKS）
3. ✅ 管理 Helm Charts 和 GitOps（ArgoCD）
4. ✅ 监控和告警（Prometheus + Grafana）
5. ✅ 日志管理（ELK Stack）
6. ✅ 故障处理和应急响应

---

## 📚 必读文档

- [CI/CD 架构](../architecture/system-architecture.md#第11章-cicd架构)
- [Docker 部署指南](../deployment/docker-guide.md)
- [GitOps 最佳实践](../deployment/gitops-best-practices.md)
- [CI/CD 流程图](../architecture/diagrams/cicd-flow.md)
- [监控方案](./monitoring.md)
- [故障排查手册](./troubleshooting.md)

---

## 🚀 快速开始

### 1. 环境准备

```bash
# 安装 kubectl
brew install kubectl  # macOS

# 安装 Helm
brew install helm

# 安装 Azure CLI
brew install azure-cli

# 登录 Azure
az login

# 配置 kubectl
az aks get-credentials --resource-group hermesflow-rg --name hermesflow-aks
```

### 2. 验证访问

```bash
# 检查集群
kubectl cluster-info
kubectl get nodes

# 检查 ArgoCD
kubectl get pods -n argocd

# 访问 ArgoCD UI
kubectl port-forward svc/argocd-server -n argocd 8080:443
# 打开 https://localhost:8080
```

---

## 🔄 CI/CD 流程

### 完整流程

```
开发者推送代码 → GitHub
           ↓
   GitHub Actions 构建
           ↓
   Docker镜像 → Azure ACR
           ↓
   触发 GitOps 仓库更新
           ↓
   ArgoCD 检测到变更
           ↓
   自动同步到 Kubernetes
```

**详细文档**: [CI/CD 架构](../architecture/system-architecture.md#第11章-cicd架构)

---

## 🐳 Docker 管理

### 构建镜像

```bash
# Rust 服务（多阶段构建）
docker build -t hermesflowacr.azurecr.io/data-engine:v1.2.3 \
  -f scripts/data-engine/Dockerfile .

# Java 服务
docker build -t hermesflowacr.azurecr.io/trading-engine:v1.2.3 \
  -f scripts/trading-engine/Dockerfile .
```

### 推送到 ACR

```bash
# 登录 ACR
az acr login --name hermesflowacr

# 推送镜像
docker push hermesflowacr.azurecr.io/data-engine:v1.2.3
```

**详细文档**: [Docker 部署指南](../deployment/docker-guide.md)

---

## ☸️ Kubernetes 管理

### 常用命令

```bash
# 查看 Pods
kubectl get pods -n hermesflow-dev
kubectl get pods -n hermesflow-main

# 查看日志
kubectl logs -f deployment/data-engine -n hermesflow-dev

# 查看服务
kubectl get svc -n hermesflow-dev

# 查看部署
kubectl get deployments -n hermesflow-dev

# 扩缩容
kubectl scale deployment/data-engine --replicas=3 -n hermesflow-dev
```

### 部署新版本

```bash
# 方式 1: GitOps（推荐）
# 更新 GitOps 仓库的 values.yaml
cd HermesFlow-GitOps/apps/dev/data-engine
vim values.yaml  # 修改 image.tag
git commit -am "Update data-engine to v1.2.3"
git push
# ArgoCD 自动同步

# 方式 2: 直接 kubectl
kubectl set image deployment/data-engine \
  data-engine=hermesflowacr.azurecr.io/data-engine:v1.2.3 \
  -n hermesflow-dev
```

### 回滚

```bash
# 查看部署历史
kubectl rollout history deployment/data-engine -n hermesflow-dev

# 回滚到上一个版本
kubectl rollout undo deployment/data-engine -n hermesflow-dev
```

---

## 📊 监控和告警

### Prometheus

```bash
# 访问 Prometheus
kubectl port-forward svc/prometheus-server -n monitoring 9090:80

# 打开 http://localhost:9090
```

**常用查询**:
```promql
# API 请求速率
rate(http_requests_total[5m])

# P95 延迟
histogram_quantile(0.95, rate(http_request_duration_seconds_bucket[5m]))

# CPU 使用率
rate(container_cpu_usage_seconds_total[5m])
```

### Grafana

```bash
# 访问 Grafana
kubectl port-forward svc/grafana -n monitoring 3000:80

# 打开 http://localhost:3000
```

**仪表盘**: HermesFlow Dashboard

**详细文档**: [监控方案](./monitoring.md)

---

## 🔥 故障处理

### 常见问题

#### 1. Pod 启动失败

```bash
# 查看 Pod 状态
kubectl describe pod <pod-name> -n hermesflow-dev

# 查看日志
kubectl logs <pod-name> -n hermesflow-dev

# 常见原因
# - 镜像拉取失败
# - 环境变量配置错误
# - 健康检查失败
```

#### 2. 服务不可访问

```bash
# 检查 Service
kubectl get svc -n hermesflow-dev

# 检查 Endpoints
kubectl get endpoints -n hermesflow-dev

# 测试连接
kubectl run -it --rm debug --image=busybox --restart=Never -- \
  wget -O- http://data-engine.hermesflow-dev.svc.cluster.local:8081/health
```

#### 3. 数据库连接失败

```bash
# 检查 PostgreSQL Pod
kubectl get pods -n hermesflow-dev | grep postgres

# 测试数据库连接
kubectl run -it --rm psql --image=postgres:15 --restart=Never -- \
  psql -h postgres.hermesflow-dev.svc.cluster.local -U hermesflow -d hermesflow
```

**详细文档**: [故障排查手册](./troubleshooting.md)

---

## 📋 最佳实践

### 1. GitOps 工作流

- ✅ 所有配置变更通过 Git
- ✅ 使用 PR 进行配置审查
- ✅ ArgoCD 自动同步
- ✅ 不直接 `kubectl apply`

### 2. 安全

- ✅ 使用 Secret 管理敏感信息
- ✅ 定期轮换密钥
- ✅ 使用 RBAC 限制访问
- ✅ 扫描镜像漏洞（Trivy）

### 3. 性能

- ✅ 设置资源限制（CPU/Memory）
- ✅ 使用 HPA 自动扩缩容
- ✅ 优化镜像大小（多阶段构建）

**详细文档**: [GitOps 最佳实践](../deployment/gitops-best-practices.md)

---

## 🔗 相关资源

- [Kubernetes 文档](https://kubernetes.io/docs/)
- [Helm 文档](https://helm.sh/docs/)
- [ArgoCD 文档](https://argo-cd.readthedocs.io/)
- [Azure AKS 文档](https://docs.microsoft.com/azure/aks/)

---

## 📞 获取帮助

- **DevOps Team**: Slack `#devops`
- **紧急问题**: [故障排查手册](./troubleshooting.md)

---

**最后更新**: 2025-01-13  
**维护者**: @architect.mdc  
**版本**: v1.0

