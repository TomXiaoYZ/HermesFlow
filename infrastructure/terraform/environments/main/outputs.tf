# HermesFlow Main环境输出

# 本地变量
locals {
  kubectl_config_command = "az aks get-credentials --resource-group ${module.aks_main.resource_group_name} --name ${module.aks_main.cluster_name}"
}

output "cluster_name" {
  description = "AKS集群名称"
  value       = module.aks_main.cluster_name
}

output "cluster_endpoint" {
  description = "AKS集群API端点"
  value       = module.aks_main.cluster_endpoint
  sensitive   = true
}

output "cluster_fqdn" {
  description = "AKS集群FQDN"
  value       = module.aks_main.cluster_fqdn
}

output "resource_group_name" {
  description = "资源组名称"
  value       = module.aks_main.resource_group_name
}

# ArgoCD部署信息
output "argocd_namespace" {
  description = "ArgoCD命名空间"
  value       = module.argocd_main.namespace
}

output "argocd_admin_password" {
  description = "ArgoCD管理员密码"
  value       = module.argocd_main.admin_password
  sensitive   = true
}

output "argocd_access_info" {
  description = "ArgoCD访问信息"
  value       = module.argocd_main.access_instructions
}

# kubectl配置命令
output "kubectl_config_command" {
  description = "获取kubectl配置的命令"
  value       = "az aks get-credentials --resource-group ${module.aks_main.resource_group_name} --name ${module.aks_main.cluster_name}"
}

# 部署完成指南
output "post_deployment_instructions" {
  description = "部署完成后的操作指南"
  value = <<-EOT
  
  🎉 HermesFlow Main环境部署完成！ (AKS + ArgoCD)
  
  📋 集群信息:
  - 集群名称: ${module.aks_main.cluster_name}
  - 资源组: ${module.aks_main.resource_group_name}
  - 端点: ${module.aks_main.cluster_fqdn}
  
  🔧 连接到集群:
  1. 获取kubectl凭据:
     ${local.kubectl_config_command}
  
  2. 验证连接:
     kubectl get nodes
  
  🚀 ArgoCD访问 (成本优化配置):
  1. 使用端口转发访问 (节省LoadBalancer成本):
     kubectl port-forward svc/argocd-server -n argocd 8080:80
     
  2. 访问ArgoCD Web界面:
     http://localhost:8080
  
  3. 登录信息:
     - 用户名: admin
     - 密码: ${module.argocd_main.admin_password}
  
  🔒 生产环境安全配置:
  1. 配置RBAC权限
  2. 启用Azure Policy
  3. 配置网络安全策略
  4. 设置备份和监控
  
  💰 成本优化:
  - 当前配置: 最小化环境 (约$60-80/月，包含ArgoCD)
  - 节点规格: Standard_B2s (2vCPU, 4GB RAM)
  - 自动扩缩: 1-2个节点
  - ArgoCD使用ClusterIP (避免LoadBalancer费用)
  
  🎯 环境目标:
  - ✅ GitOps自动化部署
  - ✅ 成本最小化
  - ✅ ArgoCD自动配置完成
  - ⚠️  生产环境建议配置HTTPS
  - ⚠️  请及时配置Git仓库连接
  - 💡 后续可根据需要扩展配置
  
  EOT
}
