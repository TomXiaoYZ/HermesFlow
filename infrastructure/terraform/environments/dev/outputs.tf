# HermesFlow Dev环境输出

# 本地变量
locals {
  kubectl_config_command = "az aks get-credentials --resource-group ${module.aks_dev.resource_group_name} --name ${module.aks_dev.cluster_name}"
}

output "cluster_name" {
  description = "AKS集群名称"
  value       = module.aks_dev.cluster_name
}

output "cluster_endpoint" {
  description = "AKS集群API端点"
  value       = module.aks_dev.cluster_endpoint
  sensitive   = true
}

output "cluster_fqdn" {
  description = "AKS集群FQDN"
  value       = module.aks_dev.cluster_fqdn
}

output "resource_group_name" {
  description = "资源组名称"
  value       = module.aks_dev.resource_group_name
}

# ArgoCD部署信息
output "argocd_namespace" {
  description = "ArgoCD命名空间"
  value       = module.argocd_dev.namespace
}

output "argocd_admin_password" {
  description = "ArgoCD管理员密码"
  value       = module.argocd_dev.admin_password
  sensitive   = true
}

output "argocd_access_info" {
  description = "ArgoCD访问信息"
  value       = module.argocd_dev.access_instructions
  sensitive   = true
}

output "argocd_deployment_id" {
  description = "ArgoCD部署ID"
  value       = module.argocd_dev.deployment_id
}

# kubectl配置命令
output "kubectl_config_command" {
  description = "获取kubectl配置的命令"
  value       = "az aks get-credentials --resource-group ${module.aks_dev.resource_group_name} --name ${module.aks_dev.cluster_name}"
}

# 部署完成指南
output "post_deployment_instructions" {
  description = "部署完成后的操作指南"
  sensitive   = true
  value = <<-EOT
  
  🎉 HermesFlow Dev环境部署完成！ (AKS + ArgoCD)
  
  📋 集群信息:
  - 集群名称: ${module.aks_dev.cluster_name}
  - 资源组: ${module.aks_dev.resource_group_name}
  - 端点: ${module.aks_dev.cluster_fqdn}
  
  🔧 连接到集群:
  1. 获取kubectl凭据:
     ${local.kubectl_config_command}
  
  2. 验证连接:
     kubectl get nodes
  
  🚀 ArgoCD访问:
  1. 获取ArgoCD外部IP:
     kubectl get svc argocd-server -n argocd
  
  2. 访问ArgoCD Web界面:
     http://<EXTERNAL-IP>
     
  3. 或使用端口转发:
     kubectl port-forward svc/argocd-server -n argocd 8080:80
     然后访问: http://localhost:8080
  
  4. 登录信息:
     - 用户名: admin
     - 密码: ${var.argocd_admin_password}
  
  💰 成本优化:
  - 当前配置: 开发环境 (约$80-120/月，包含ArgoCD)
  - 节点规格: Standard_B2s (2vCPU, 4GB RAM)
  - 自动扩缩: 1-5个节点
  
  ⚠️  重要提醒:
  - ArgoCD已自动部署并配置完成
  - 请及时配置Git仓库连接
  - 建议配置HTTPS访问和证书
  - 定期检查资源使用情况
  
  EOT
}
