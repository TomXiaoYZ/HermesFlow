# HermesFlow 基础设施部署

## 📋 概述

HermesFlow基础设施采用Terraform管理，支持Dev和Main两套独立的AKS集群环境。

## 🏗️ 架构设计

```
HermesFlow Infrastructure
├── Dev Environment (AKS)           Main Environment (AKS)
│   ├── Standard_B2s (2vCPU, 4GB)   ├── Standard_D2s_v3 (2vCPU, 8GB)
│   ├── 1-3 节点自动扩缩             ├── 2-10 节点自动扩缩
│   ├── 开发工作负载                  ├── 生产工作负载
│   └── 成本优化配置                  └── 性能优化配置
│
└── ArgoCD (Container Instance)
    ├── 统一管理两套环境
    ├── GitOps自动部署
    └── 成本优化 ($30-40/月)
```

## 📁 项目结构

```
infrastructure/
├── terraform/
│   ├── modules/aks/                 # AKS集群模块
│   │   ├── main.tf                  # 主配置文件
│   │   ├── variables.tf             # 变量定义
│   │   └── outputs.tf               # 输出定义
│   ├── environments/                # 环境特定配置
│   │   ├── dev/                     # 开发环境
│   │   │   ├── main.tf
│   │   │   ├── variables.tf
│   │   │   └── outputs.tf
│   │   └── main/                    # 生产环境
│   │       ├── main.tf
│   │       ├── variables.tf
│   │       └── outputs.tf
│   └── deploy-aks.sh                # 🚀 统一部署脚本
└── README.md                        # 本文档
```

## 🚀 快速部署

### 先决条件
```bash
# 安装必要工具
az --version         # Azure CLI
terraform --version  # Terraform >= 1.0

# 登录Azure
az login
```

### 部署步骤

#### 1. 部署开发环境
```bash
cd infrastructure/terraform
./deploy-aks.sh dev
```

#### 2. 部署生产环境
```bash
./deploy-aks.sh main
```

#### 3. 部署所有环境
```bash
./deploy-aks.sh all
```

### 高级部署选项
```bash
# 仅查看执行计划
./deploy-aks.sh dev -p

# 自动批准部署
./deploy-aks.sh main -a

# 指定Azure区域
./deploy-aks.sh dev -l "West US 2"

# 销毁环境
./deploy-aks.sh dev -d
```

## 🔧 配置对比

### Dev环境配置
| 配置项 | 值 | 说明 |
|--------|----|----|
| VM规格 | Standard_B2s | 2vCPU, 4GB RAM |
| 节点数量 | 2个 | 最小配置 |
| 自动扩缩 | 1-3个节点 | 成本优化 |
| OS磁盘 | 30GB | 最小配置 |
| 日志保留 | 7天 | 短期保留 |
| 月费用 | $60-80 | 成本优化 |

### Main环境配置
| 配置项 | 值 | 说明 |
|--------|----|----|
| VM规格 | Standard_D2s_v3 | 2vCPU, 8GB RAM |
| 节点数量 | 3个 | 生产级配置 |
| 自动扩缩 | 2-10个节点 | 高可用 |
| OS磁盘 | 50GB | 更大存储 |
| 日志保留 | 90天 | 长期保留 |
| 月费用 | $200-300 | 生产级性能 |

## 🔗 集群连接

### 获取kubectl凭据
```bash
# Dev环境
az aks get-credentials --resource-group hermesflow-dev-rg --name hermesflow-dev-aks

# Main环境
az aks get-credentials --resource-group hermesflow-main-rg --name hermesflow-main-aks
```

### 验证连接
```bash
# 查看节点
kubectl get nodes

# 查看命名空间
kubectl get namespaces

# 切换上下文
kubectl config use-context hermesflow-dev-aks
kubectl config use-context hermesflow-main-aks
```

## 🚀 ArgoCD集成

### 添加集群到ArgoCD
```bash
# 安装ArgoCD CLI
curl -sSL -o argocd-linux-amd64 https://github.com/argoproj/argo-cd/releases/latest/download/argocd-linux-amd64
sudo install -m 555 argocd-linux-amd64 /usr/local/bin/argocd

# 登录ArgoCD
argocd login <ARGOCD_FQDN>:8443

# 添加Dev集群
argocd cluster add hermesflow-dev-aks --name hermesflow-dev

# 添加Main集群
argocd cluster add hermesflow-main-aks --name hermesflow-main

# 验证集群
argocd cluster list
```

## 💰 成本管理

### 成本预估
| 环境 | 基础成本 | 扩容成本 | 总预算 |
|-----|---------|---------|--------|
| Dev | $60/月 | +$30/节点 | $60-120/月 |
| Main | $200/月 | +$100/节点 | $200-800/月 |
| **合计** | **$260/月** | **变动** | **$260-920/月** |

### 成本优化建议
1. **开发环境优化**
   - 使用Spot节点池 (节省60-90%)
   - 非工作时间自动停机
   - 最小节点配置

2. **生产环境优化**
   - Reserved Instances (节省20-30%)
   - 合理的自动扩缩配置
   - 监控资源使用率

3. **统一优化**
   - 使用Azure Hybrid Benefit
   - 定期审查未使用资源
   - 设置预算告警

### 监控成本
```bash
# 查看当月费用
az consumption usage list --start-date $(date -d "$(date +%Y-%m-01)" +%Y-%m-%d)

# 设置预算告警
az consumption budget create \
  --budget-name "hermesflow-aks-budget" \
  --amount 500 \
  --time-grain Monthly
```

## 🛠️ 运维管理

### 集群升级
```bash
# 查看可用版本
az aks get-upgrades --resource-group hermesflow-dev-rg --name hermesflow-dev-aks

# 升级集群
az aks upgrade --resource-group hermesflow-dev-rg --name hermesflow-dev-aks --kubernetes-version 1.28.3
```

### 节点池管理
```bash
# 查看节点池
az aks nodepool list --resource-group hermesflow-dev-rg --cluster-name hermesflow-dev-aks

# 扩容节点池
az aks nodepool scale --resource-group hermesflow-dev-rg --cluster-name hermesflow-dev-aks --name default --node-count 5
```

### 监控和日志
```bash
# 查看集群状态
kubectl get nodes
kubectl top nodes

# 查看Pod状态
kubectl get pods --all-namespaces
kubectl top pods --all-namespaces
```

## 🔒 安全配置

### RBAC配置
```bash
# 创建集群角色绑定
kubectl create clusterrolebinding aks-cluster-admin --clusterrole=cluster-admin --user=<user-email>

# 创建命名空间角色
kubectl create rolebinding hermesflow-dev-admin --clusterrole=admin --user=<user-email> --namespace=hermesflow-dev
```

### 网络策略
```yaml
# 示例网络策略
apiVersion: networking.k8s.io/v1
kind: NetworkPolicy
metadata:
  name: hermesflow-network-policy
  namespace: hermesflow-dev
spec:
  podSelector: {}
  policyTypes:
  - Ingress
  - Egress
  ingress:
  - from:
    - namespaceSelector:
        matchLabels:
          name: hermesflow-dev
  egress:
  - to: []
```

## 🛠️ 故障排除

### 常见问题

#### 1. 节点无法启动
```bash
# 查看节点事件
kubectl describe node <node-name>

# 查看集群事件
kubectl get events --sort-by=.metadata.creationTimestamp
```

#### 2. Pod无法调度
```bash
# 查看Pod状态
kubectl describe pod <pod-name> -n <namespace>

# 检查资源配额
kubectl describe quota -n <namespace>
```

#### 3. 网络连接问题
```bash
# 测试DNS解析
kubectl run -it --rm debug --image=busybox --restart=Never -- nslookup kubernetes.default

# 测试网络连通性
kubectl run -it --rm debug --image=nicolaka/netshoot --restart=Never
```

## 📚 相关文档

- [ArgoCD部署指南](../../HermesFlow-GitOps/argocd/README.md)
- [Azure AKS官方文档](https://docs.microsoft.com/en-us/azure/aks/)
- [Terraform Azure Provider文档](https://registry.terraform.io/providers/hashicorp/azurerm/latest/docs)
- [Kubernetes官方文档](https://kubernetes.io/docs/)

---

**维护者**: HermesFlow DevOps Team  
**最后更新**: 2024年12月
