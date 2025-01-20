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

# 确认销毁操作
read -p "确定要销毁 $ENV 环境的所有资源吗？(y/N) " -n 1 -r
echo
if [[ ! $REPLY =~ ^[Yy]$ ]]
then
    exit 1
fi

# 删除 Kubernetes 资源
echo "正在删除 Kubernetes 资源..."
cd "$PROJECT_ROOT/infrastructure/k8s/$ENV"
kubectl delete -k . || true

# 销毁 Terraform 基础设施
echo "正在销毁 Terraform 基础设施..."
cd "$PROJECT_ROOT/infrastructure/terraform/$ENV"
terraform destroy -auto-approve

echo "销毁完成！" 