# Development Environment Outputs

# Networking Outputs
output "resource_group_name" {
  description = "Name of the resource group"
  value       = module.networking.resource_group_name
}

output "vnet_name" {
  description = "Name of the virtual network"
  value       = module.networking.vnet_name
}

# ACR Outputs
output "acr_login_server" {
  description = "ACR login server URL"
  value       = module.acr.login_server
}

output "acr_name" {
  description = "ACR name"
  value       = module.acr.name
}

# AKS Outputs
output "aks_cluster_name" {
  description = "AKS cluster name"
  value       = module.aks.cluster_name
}

output "aks_cluster_fqdn" {
  description = "AKS cluster FQDN"
  value       = module.aks.cluster_fqdn
}

output "aks_get_credentials_command" {
  description = "Command to get AKS credentials"
  value       = "az aks get-credentials --resource-group ${module.networking.resource_group_name} --name ${module.aks.cluster_name}"
}

output "aks_kube_config" {
  description = "AKS Kubernetes config for ArgoCD deployment"
  value       = module.aks.kube_config_structured
  sensitive   = true
}

# Database Outputs
output "postgres_server_fqdn" {
  description = "PostgreSQL server FQDN"
  value       = module.database.server_fqdn
}

output "postgres_database_name" {
  description = "PostgreSQL database name"
  value       = module.database.database_name
}

output "postgres_connection_string" {
  description = "PostgreSQL connection string"
  value       = module.database.connection_string
  sensitive   = true
}

# Key Vault Outputs
output "keyvault_name" {
  description = "Key Vault name"
  value       = module.keyvault.name
}

output "keyvault_uri" {
  description = "Key Vault URI"
  value       = module.keyvault.vault_uri
}

# Monitoring Outputs
output "log_analytics_workspace_name" {
  description = "Log Analytics workspace name"
  value       = module.monitoring.workspace_name
}

# Quick Start Commands
output "quick_start_commands" {
  description = "Commands to get started"
  value       = <<-EOT
    # Get AKS credentials
    az aks get-credentials --resource-group ${module.networking.resource_group_name} --name ${module.aks.cluster_name}
    
    # Verify cluster access
    kubectl get nodes
    
    # Get ACR credentials
    az acr credential show --name ${module.acr.name}
    
    # View Key Vault secrets
    az keyvault secret list --vault-name ${module.keyvault.name}
    
    # View PostgreSQL connection info
    echo "Host: ${module.database.server_fqdn}"
    echo "Database: ${module.database.database_name}"
  EOT
}

