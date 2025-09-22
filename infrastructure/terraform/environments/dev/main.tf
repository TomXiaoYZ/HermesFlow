# HermesFlow Dev环境 AKS集群部署
terraform {
  required_version = ">= 1.0"
  
  required_providers {
    azurerm = {
      source  = "hashicorp/azurerm"
      version = "~> 3.0"
    }
  }

  # 远程状态管理
  backend "azurerm" {
    resource_group_name  = "hermesflow-terraform-rg"
    storage_account_name = "hermesflowterraformsa"
    container_name       = "tfstate"
    key                  = "dev-aks.terraform.tfstate"
  }
}

# Azure Provider配置
provider "azurerm" {
  features {
    resource_group {
      prevent_deletion_if_contains_resources = false
    }
  }
}



# 本地变量
locals {
  environment = "dev"
  project     = "hermesflow"
  
  # Dev环境配置 - 适合开发调试使用
  dev_config = {
    vm_size           = "Standard_B2s"    # 2vCPU, 4GB RAM - 适合开发调试
    node_count        = 1                 # 当前1个节点，符合成本优化
    min_node_count    = 1                 # 最小可缩减到1个节点
    max_node_count    = 5                 # 支持开发期间的负载测试
    os_disk_size_gb   = 30               # 标准OS磁盘
        kubernetes_version = "1.30.14"       # 当前支持的稳定版本
  }

  # 通用标签
  common_tags = {
    Environment = local.environment
    Project     = local.project
    ManagedBy   = "Terraform"
    CostCenter  = "Development"
    Owner       = "HermesFlow-Team"
  }
}

# AKS集群模块调用
module "aks_dev" {
  source = "../../modules/aks"

  # 基础配置
  cluster_name        = "${local.project}-${local.environment}-aks"
  resource_group_name = "${local.project}-${local.environment}-rg"
  location           = var.location
  dns_prefix         = "${local.project}-${local.environment}"

  # 成本优化配置
  vm_size            = local.dev_config.vm_size
  node_count         = local.dev_config.node_count
  kubernetes_version = local.dev_config.kubernetes_version
  os_disk_size_gb    = local.dev_config.os_disk_size_gb

  # 自动扩缩容
  enable_auto_scaling = true
  min_node_count      = local.dev_config.min_node_count
  max_node_count      = local.dev_config.max_node_count

  # 监控优化 (Dev环境较短保留期)
  log_retention_days = 30  # 最小保留天数为30天

  # 暂时不启用ACR集成，避免循环依赖
  # acr_id = azurerm_container_registry.dev.id

  # 工作负载节点池 (Dev环境不启用)
  enable_workload_node_pool = false

  tags = local.common_tags
}

# ArgoCD模块调用 (使用Helm部署)
module "argocd_dev" {
  source = "../../modules/argocd"

  # 基础配置
  admin_password = var.argocd_admin_password
  namespace      = "argocd"
  
  # Dev环境使用LoadBalancer服务类型
  service_type = "LoadBalancer"
  
  # Dev环境单副本部署
  server_replicas     = 1
  repo_server_replicas = 1
  controller_replicas  = 1
  
  # 资源配置 (Dev环境适中配置)
  resource_requests = {
    server = {
      cpu    = "100m"
      memory = "128Mi"
    }
    repo_server = {
      cpu    = "100m"
      memory = "128Mi"
    }
    controller = {
      cpu    = "200m"
      memory = "256Mi"
    }
  }
  
  resource_limits = {
    server = {
      cpu    = "500m"
      memory = "512Mi"
    }
    repo_server = {
      cpu    = "500m"
      memory = "512Mi"
    }
    controller = {
      cpu    = "1000m"
      memory = "1Gi"
    }
  }

  # 部署配置
  enable_insecure = true
  wait_for_ready  = true

  tags = local.common_tags

  # 确保ArgoCD在AKS集群创建后部署
  depends_on = [module.aks_dev]
}

# 创建容器注册表
resource "azurerm_container_registry" "dev" {
  name                = "hermesflowdevacr"
  resource_group_name = module.aks_dev.resource_group_name
  location            = var.location
  sku                 = "Basic"
  admin_enabled       = false

  tags = local.common_tags

  depends_on = [
    module.aks_dev
  ]
}
