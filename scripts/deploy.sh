#!/bin/bash

set -e

# 检查环境参数
if [ -z "$1" ]; then
  echo "请指定环境 (dev/prod)"
  exit 1
fi

ENV=$1
SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

# 检查必要的工具
command -v terraform >/dev/null 2>&1 || { echo "需要安装 terraform"; exit 1; }
command -v kubectl >/dev/null 2>&1 || { echo "需要安装 kubectl"; exit 1; }
command -v aws >/dev/null 2>&1 || { echo "需要安装 aws cli"; exit 1; }

# 初始化和应用 Terraform 配置
echo "正在部署 Terraform 基础设施..."
cd "$PROJECT_ROOT/infrastructure/terraform/$ENV"
terraform init
terraform apply -auto-approve

# 获取 EKS 集群凭证
echo "正在获取 EKS 集群凭证..."
aws eks update-kubeconfig --name "hermesflow-$ENV" --region $(terraform output -raw aws_region)

# 应用 Kubernetes 配置
echo "正在部署 Kubernetes 资源..."
cd "$PROJECT_ROOT/infrastructure/k8s/$ENV"
kubectl apply -k .

echo "部署完成！"

# 输出重要信息
echo "集群端点: $(terraform output -raw cluster_endpoint)"
echo "Grafana 服务: $(kubectl get svc -n hermesflow-$ENV grafana -o jsonpath='{.status.loadBalancer.ingress[0].hostname}')"
echo "Kibana 服务: $(kubectl get svc -n hermesflow-$ENV kibana -o jsonpath='{.status.loadBalancer.ingress[0].hostname}')" 