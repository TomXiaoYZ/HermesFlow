# HermesFlow AKS 模块输出

output "cluster_id" {
  description = "AKS集群ID"
  value       = azurerm_kubernetes_cluster.aks.id
}

output "cluster_name" {
  description = "AKS集群名称"
  value       = azurerm_kubernetes_cluster.aks.name
}

output "cluster_fqdn" {
  description = "AKS集群FQDN"
  value       = azurerm_kubernetes_cluster.aks.fqdn
}

output "cluster_endpoint" {
  description = "AKS集群API端点"
  value       = azurerm_kubernetes_cluster.aks.kube_config.0.host
  sensitive   = true
}

output "cluster_ca_certificate" {
  description = "AKS集群CA证书"
  value       = azurerm_kubernetes_cluster.aks.kube_config.0.cluster_ca_certificate
  sensitive   = true
}

output "cluster_client_certificate" {
  description = "AKS集群客户端证书"
  value       = azurerm_kubernetes_cluster.aks.kube_config.0.client_certificate
  sensitive   = true
}

output "cluster_client_key" {
  description = "AKS集群客户端密钥"
  value       = azurerm_kubernetes_cluster.aks.kube_config.0.client_key
  sensitive   = true
}

output "kube_config" {
  description = "Kubernetes配置"
  value       = azurerm_kubernetes_cluster.aks.kube_config_raw
  sensitive   = true
}

output "kubelet_identity_object_id" {
  description = "Kubelet身份对象ID"
  value       = azurerm_kubernetes_cluster.aks.kubelet_identity[0].object_id
}

output "kubelet_identity_client_id" {
  description = "Kubelet身份客户端ID"
  value       = azurerm_kubernetes_cluster.aks.kubelet_identity[0].client_id
}

output "resource_group_name" {
  description = "资源组名称"
  value       = azurerm_resource_group.aks.name
}

output "log_analytics_workspace_id" {
  description = "Log Analytics工作区ID"
  value       = azurerm_log_analytics_workspace.aks.id
}

# 连接信息 (用于ArgoCD配置)
output "argocd_cluster_config" {
  description = "ArgoCD集群配置信息"
  value = {
    name   = var.cluster_name
    server = azurerm_kubernetes_cluster.aks.kube_config.0.host
    config = {
      tlsClientConfig = {
        caData = azurerm_kubernetes_cluster.aks.kube_config.0.cluster_ca_certificate
      }
    }
  }
  sensitive = true
}
