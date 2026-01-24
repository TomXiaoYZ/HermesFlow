# HermesFlow Development Environment
# Integrates all Terraform modules for dev infrastructure

terraform {
  required_version = ">= 1.5"

  required_providers {
    azurerm = {
      source  = "hashicorp/azurerm"
      version = "~> 3.85"
    }
    random = {
      source  = "hashicorp/random"
      version = "~> 3.5"
    }
  }

  backend "azurerm" {
    resource_group_name  = "hermesflow-tfstate-rg"
    storage_account_name = "hermesflowdevtfstate"
    container_name       = "tfstate"
    key                  = "dev.terraform.tfstate"
  }
}

provider "azurerm" {
  features {
    key_vault {
      purge_soft_delete_on_destroy    = true
      recover_soft_deleted_key_vaults = true
    }
    resource_group {
      prevent_deletion_if_contains_resources = false
    }
  }
}

locals {
  environment = "dev"
  prefix      = "hermesflow-dev"
  location    = "centralus"  # Final region - verified PostgreSQL support

  tags = {
    Environment = "Development"
    Project     = "HermesFlow"
    ManagedBy   = "Terraform"
    CostCenter  = "Engineering"
    Owner       = "DevOps Team"
  }
}

# Module 1: Networking (VNet, Subnets, NSGs)
module "networking" {
  source = "../../modules/networking"

  resource_group_name = "${local.prefix}-rg"
  location            = local.location
  prefix              = local.prefix
  tags                = local.tags
}

# Module 2: Monitoring (Log Analytics, Container Insights, Alerts)
module "monitoring" {
  source = "../../modules/monitoring"

  resource_group_name = module.networking.resource_group_name
  location            = local.location
  prefix              = local.prefix
  log_retention_days  = 30
  cluster_name        = "${local.prefix}-aks"
  aks_id              = "" # Will be updated after AKS creation
  alert_email         = var.alert_email
  slack_webhook_url   = var.slack_webhook_url
  tags                = local.tags
}

# Module 3: ACR (Azure Container Registry)
module "acr" {
  source = "../../modules/acr"

  resource_group_name        = module.networking.resource_group_name
  location                   = local.location
  prefix                     = local.prefix
  sku                        = "Standard"
  environment                = local.environment
  log_analytics_workspace_id = module.monitoring.workspace_id
  allowed_ip_ranges          = var.allowed_ip_ranges
  georeplications            = []
  tags                       = local.tags
}

# Module 4: AKS (Azure Kubernetes Service)
module "aks" {
  source = "../../modules/aks"

  resource_group_name        = module.networking.resource_group_name
  location                   = local.location
  prefix                     = local.prefix
  kubernetes_version         = var.kubernetes_version
  subnet_id                  = module.networking.aks_subnet_id
  system_node_count          = 2
  system_node_size           = "Standard_D4s_v3"
  user_node_count            = 1
  user_node_size             = "Standard_D8s_v3"
  log_analytics_workspace_id = module.monitoring.workspace_id
  acr_id                     = module.acr.id
  tags                       = local.tags

  depends_on = [module.networking, module.acr, module.monitoring]
}

# Module 5: Database (PostgreSQL Flexible Server)
module "database" {
  source = "../../modules/database"

  resource_group_name   = module.networking.resource_group_name
  location              = local.location
  prefix                = local.prefix
  subnet_id             = module.networking.database_subnet_id
  vnet_id               = module.networking.vnet_id
  admin_username        = var.postgres_admin_username
  admin_password        = var.postgres_admin_password
  sku_name              = "B_Standard_B1ms"
  storage_mb            = 32768
  backup_retention_days = 7
  environment           = local.environment
  aks_outbound_ip       = "" # Not needed with VNet integration
  tags                  = local.tags

  depends_on = [module.networking, module.aks]
}

# Module 6: Key Vault (Secrets Management)
module "keyvault" {
  source = "../../modules/keyvault"

  resource_group_name            = module.networking.resource_group_name
  location                       = local.location
  prefix                         = local.prefix
  environment                    = local.environment
  aks_subnet_id                  = module.networking.aks_subnet_id
  aks_kubelet_identity_object_id = module.aks.kubelet_identity_object_id
  postgres_admin_password        = var.postgres_admin_password
  allowed_ip_ranges              = var.allowed_ip_ranges
  tags                           = local.tags

  depends_on = [module.aks]
}

