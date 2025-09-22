# HermesFlow Main环境 AKS集群部署
terraform {
  required_version = ">= 1.0"
  
  required_providers {
    azurerm = {
      source  = "hashicorp/azurerm"
      version = "~> 3.0"
    }
    kubernetes = {
      source  = "hashicorp/kubernetes"
      version = "~> 2.0"
    }
  }

  # 远程状态管理
  backend "azurerm" {
    resource_group_name  = "hermesflow-terraform-rg"
    storage_account_name = "hermesflowterraformsa"
    container_name       = "tfstate"
    key                  = "main-aks.terraform.tfstate"
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

# Kubernetes Provider配置 (用于ArgoCD部署)
provider "kubernetes" {
  host                   = module.aks_main.cluster_endpoint
  cluster_ca_certificate = base64decode(module.aks_main.cluster_ca_certificate)
  client_certificate     = base64decode(module.aks_main.cluster_client_certificate)
  client_key             = base64decode(module.aks_main.cluster_client_key)
}


# 本地变量
locals {
  environment = "main"
  project     = "hermesflow"
  
  # Main环境配置 - 最小化成本，用于CI/CD验证
  main_config = {
    vm_size           = "Standard_B2s"    # 2vCPU, 4GB RAM - AKS系统节点池最小要求
    node_count        = 1                 # 最小单节点配置
    min_node_count    = 1                 # 始终保持1个节点
    max_node_count    = 2                 # 最多扩展到2个节点
    os_disk_size_gb   = 30               # 标准OS磁盘
        kubernetes_version = "1.30.14"       # 当前支持的稳定版本
  }

  # 通用标签
  common_tags = {
    Environment = local.environment
    Project     = local.project
    ManagedBy   = "Terraform"
    CostCenter  = "Production"
    Owner       = "HermesFlow-Team"
    Backup      = "Required"
  }
}

# 获取现有的ACR (如果存在)
data "azurerm_container_registry" "main" {
  name                = "hermesflowmainacr"
  resource_group_name = "hermesflow-main-rg"
}

# AKS集群模块调用
module "aks_main" {
  source = "../../modules/aks"

  # 基础配置
  cluster_name        = "${local.project}-${local.environment}-aks"
  resource_group_name = "${local.project}-${local.environment}-rg"
  location           = var.location
  dns_prefix         = "${local.project}-${local.environment}"

  # 生产环境配置
  vm_size            = local.main_config.vm_size
  node_count         = local.main_config.node_count
  kubernetes_version = local.main_config.kubernetes_version
  os_disk_size_gb    = local.main_config.os_disk_size_gb

  # 自动扩缩容
  enable_auto_scaling = true
  min_node_count      = local.main_config.min_node_count
  max_node_count      = local.main_config.max_node_count

  # 监控配置 (最小保留天数为30天)
  log_retention_days = 30

  # ACR集成
  acr_id = data.azurerm_container_registry.main.id

  # 工作负载节点池 (最小化环境不启用)
  enable_workload_node_pool = false

  tags = local.common_tags
}

# ArgoCD模块调用 (Main环境 - 最小化配置)
module "argocd_main" {
  source = "../../modules/argocd"

  # 基础配置
  admin_password = "admin123"
  namespace      = "argocd"
  
  # Main环境使用ClusterIP + 端口转发 (节省成本)
  service_type = "ClusterIP"
  
  # Main环境单副本部署
  server_replicas     = 1
  repo_server_replicas = 1
  controller_replicas  = 1
  
  # 资源配置 (Main环境最小配置)
  resource_requests = {
    server = {
      cpu    = "50m"
      memory = "64Mi"
    }
    repo_server = {
      cpu    = "50m"
      memory = "64Mi"
    }
    controller = {
      cpu    = "100m"
      memory = "128Mi"
    }
  }
  
  resource_limits = {
    server = {
      cpu    = "250m"
      memory = "256Mi"
    }
    repo_server = {
      cpu    = "250m"
      memory = "256Mi"
    }
    controller = {
      cpu    = "500m"
      memory = "512Mi"
    }
  }

  tags = local.common_tags

  # 确保ArgoCD在AKS集群创建后部署
  depends_on = [module.aks_main]
}
