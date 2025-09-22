# HermesFlow AKS 集群模块
terraform {
  required_providers {
    azurerm = {
      source  = "hashicorp/azurerm"
      version = "~> 3.0"
    }
  }
}

# 数据源
data "azurerm_client_config" "current" {}

# 资源组
resource "azurerm_resource_group" "aks" {
  name     = var.resource_group_name
  location = var.location
  tags     = var.tags
}

# Log Analytics Workspace (用于容器洞察)
resource "azurerm_log_analytics_workspace" "aks" {
  name                = "${var.cluster_name}-logs"
  location            = azurerm_resource_group.aks.location
  resource_group_name = azurerm_resource_group.aks.name
  sku                 = "PerGB2018"
  retention_in_days   = var.log_retention_days
  tags               = var.tags
}

# AKS 集群
resource "azurerm_kubernetes_cluster" "aks" {
  name                = var.cluster_name
  location            = azurerm_resource_group.aks.location
  resource_group_name = azurerm_resource_group.aks.name
  dns_prefix          = var.dns_prefix
  kubernetes_version  = var.kubernetes_version

  # 默认节点池
  default_node_pool {
    name       = "default"
    node_count = var.node_count
    vm_size    = var.vm_size
    
    # 成本优化配置
    enable_auto_scaling = var.enable_auto_scaling
    min_count          = var.enable_auto_scaling ? var.min_node_count : null
    max_count          = var.enable_auto_scaling ? var.max_node_count : null
    
    # 网络配置
    vnet_subnet_id = var.subnet_id
    
    # 节点配置
    os_disk_size_gb = var.os_disk_size_gb
    os_disk_type    = var.os_disk_type
    
    tags = var.tags
  }

  # 身份配置
  identity {
    type = "SystemAssigned"
  }

  # 网络配置
  network_profile {
    network_plugin    = "azure"
    network_policy    = "azure"
    dns_service_ip    = var.dns_service_ip
    service_cidr      = var.service_cidr
  }

  # 监控配置
  oms_agent {
    log_analytics_workspace_id = azurerm_log_analytics_workspace.aks.id
  }

  # 安全配置
  azure_policy_enabled = true
  
  # RBAC配置
  role_based_access_control_enabled = true
  
  # Azure AD集成 (AKS-managed Entra Integration)
  azure_active_directory_role_based_access_control {
    managed            = true
    azure_rbac_enabled = true
  }

  tags = var.tags
}

# 容器注册表集成 (可选)
resource "azurerm_role_assignment" "aks_acr" {
  count                = var.acr_id != null ? 1 : 0
  principal_id         = azurerm_kubernetes_cluster.aks.kubelet_identity[0].object_id
  role_definition_name = "AcrPull"
  scope                = var.acr_id
}

# 额外节点池 (用于生产环境的工作负载分离)
resource "azurerm_kubernetes_cluster_node_pool" "workload" {
  count                 = var.enable_workload_node_pool ? 1 : 0
  name                  = "workload"
  kubernetes_cluster_id = azurerm_kubernetes_cluster.aks.id
  vm_size              = var.workload_vm_size
  node_count           = var.workload_node_count
  
  enable_auto_scaling = true
  min_count          = var.workload_min_count
  max_count          = var.workload_max_count
  
  node_taints = var.workload_node_taints
  node_labels = var.workload_node_labels
  
  tags = var.tags
}
