# Story 2: Azure Infrastructure as Code with Terraform

**Story ID**: DEVOPS-002  
**Epic**: DevOps Foundation  
**Priority**: P0 (Critical)  
**Estimate**: 13 Story Points (26 hours)  
**Sprint**: Sprint 1 (2025-01-10 ~ 2025-01-24)  
**Status**: Approved  
**Created**: 2025-01-13  
**Created By**: @sm.mdc  
**Validated By**: @po.mdc

---

## 📖 User Story

**作为** DevOps工程师  
**我想要** 使用Terraform管理Azure基础设施  
**以便** 实现基础设施即代码(IaC)，可复现、可版本控制的云资源管理

---

## 🎯 验收标准 (Acceptance Criteria)

### 1. 核心Azure资源创建

```gherkin
Scenario: 创建Dev环境基础设施
  Given Terraform配置文件已准备
  When 执行 terraform apply
  Then 系统应该创建:
    - Resource Group (hermesflow-dev-rg)
    - Virtual Network (10.0.0.0/16)
    - 3个子网 (AKS, Database, AppGateway)
    - Azure Kubernetes Service (3节点)
    - Azure Container Registry
    - Azure Database for PostgreSQL Flexible Server
    - Azure Key Vault
    - Log Analytics Workspace
  And 所有资源应使用统一的标签
  And 网络安全组规则正确配置
```

### 2. AKS集群配置

- [ ] Kubernetes版本: 1.28+
- [ ] 节点池配置: 
  - System节点池: 2节点 (Standard_D4s_v3)
  - User节点池: 1节点 (Standard_D8s_v3, 自动扩展1-5)
- [ ] 启用Azure CNI网络
- [ ] 启用Azure Monitor Container Insights
- [ ] 配置Pod Identity
- [ ] 配置RBAC
- [ ] 启用HTTP application routing (dev环境)

### 3. 网络架构

```
Virtual Network (10.0.0.0/16)
├── AKS Subnet (10.0.1.0/24)
│   └── AKS Nodes + Pods
├── Database Subnet (10.0.2.0/24)
│   └── PostgreSQL, Redis (Private Endpoint)
└── AppGateway Subnet (10.0.3.0/24)
    └── Application Gateway (未来)
```

- [ ] 子网正确创建
- [ ] NSG规则允许必要流量
- [ ] Private Endpoint配置(Database)
- [ ] Service Endpoints配置

### 4. ACR集成

- [ ] ACR创建 (Standard SKU)
- [ ] AKS配置ACR pull权限(通过Managed Identity)
- [ ] Geo-replication配置(未来生产环境)
- [ ] Webhook配置(镜像推送通知)

### 5. 安全与密钥管理

- [ ] Azure Key Vault创建
- [ ] 存储敏感配置(DB密码, API Keys)
- [ ] CSI Secret Store Driver集成AKS
- [ ] RBAC正确配置(最小权限原则)

### 6. 监控与日志

- [ ] Log Analytics Workspace创建
- [ ] Container Insights启用
- [ ] 诊断设置配置(NSG, AKS, ACR日志)
- [ ] Azure Monitor Alerts配置

### 7. 多环境支持

- [ ] dev环境: 小规格,成本优化
- [ ] main(production)环境: 高可用,冗余配置
- [ ] 环境隔离(独立Resource Group + VNet)
- [ ] Terraform Workspace或独立State管理

---

## 🔧 技术任务分解 (Technical Tasks)

### Task 2.1: 设计Terraform模块结构 (3h)

**负责人**: DevOps Lead

**目录结构**:
```
infrastructure/terraform/
├── modules/
│   ├── networking/
│   │   ├── main.tf
│   │   ├── variables.tf
│   │   ├── outputs.tf
│   │   └── README.md
│   ├── aks/
│   │   ├── main.tf
│   │   ├── variables.tf
│   │   ├── outputs.tf
│   │   └── README.md
│   ├── acr/
│   │   └── ...
│   ├── database/
│   │   └── ...
│   └── monitoring/
│       └── ...
│
├── environments/
│   ├── dev/
│   │   ├── main.tf
│   │   ├── variables.tf
│   │   ├── terraform.tfvars
│   │   ├── backend.tf
│   │   └── outputs.tf
│   └── main/
│       └── ...
│
├── .terraform.lock.hcl
└── README.md
```

**验收**:
- [ ] 模块化结构清晰
- [ ] 每个模块有独立的README
- [ ] 变量和输出定义完整

---

### Task 2.2: 实现Networking模块 (3h)

**负责人**: DevOps Engineer

**模块功能**:
```hcl
# modules/networking/main.tf
resource "azurerm_resource_group" "main" {
  name     = var.resource_group_name
  location = var.location
  tags     = var.tags
}

resource "azurerm_virtual_network" "main" {
  name                = "${var.prefix}-vnet"
  address_space       = ["10.0.0.0/16"]
  location            = azurerm_resource_group.main.location
  resource_group_name = azurerm_resource_group.main.name
  tags                = var.tags
}

resource "azurerm_subnet" "aks" {
  name                 = "aks-subnet"
  resource_group_name  = azurerm_resource_group.main.name
  virtual_network_name = azurerm_virtual_network.main.name
  address_prefixes     = ["10.0.1.0/24"]
}

resource "azurerm_subnet" "database" {
  name                 = "database-subnet"
  resource_group_name  = azurerm_resource_group.main.name
  virtual_network_name = azurerm_virtual_network.main.name
  address_prefixes     = ["10.0.2.0/24"]
  
  delegation {
    name = "postgresql-delegation"
    service_delegation {
      name = "Microsoft.DBforPostgreSQL/flexibleServers"
      actions = [
        "Microsoft.Network/virtualNetworks/subnets/join/action",
      ]
    }
  }
}

resource "azurerm_network_security_group" "aks" {
  name                = "${var.prefix}-aks-nsg"
  location            = azurerm_resource_group.main.location
  resource_group_name = azurerm_resource_group.main.name

  security_rule {
    name                       = "AllowHTTPS"
    priority                   = 100
    direction                  = "Inbound"
    access                     = "Allow"
    protocol                   = "Tcp"
    source_port_range          = "*"
    destination_port_range     = "443"
    source_address_prefix      = "*"
    destination_address_prefix = "*"
  }
  
  tags = var.tags
}

resource "azurerm_subnet_network_security_group_association" "aks" {
  subnet_id                 = azurerm_subnet.aks.id
  network_security_group_id = azurerm_network_security_group.aks.id
}
```

**验收**:
- [ ] VNet和子网创建成功
- [ ] NSG规则正确配置
- [ ] 子网关联正确

---

### Task 2.3: 实现AKS模块 (5h)

**负责人**: DevOps Engineer

**模块功能**:
```hcl
# modules/aks/main.tf
resource "azurerm_kubernetes_cluster" "main" {
  name                = "${var.prefix}-aks"
  location            = var.location
  resource_group_name = var.resource_group_name
  dns_prefix          = "${var.prefix}-aks"
  kubernetes_version  = var.kubernetes_version

  default_node_pool {
    name                = "system"
    node_count          = var.system_node_count
    vm_size             = var.system_node_size
    vnet_subnet_id      = var.subnet_id
    enable_auto_scaling = true
    min_count           = 2
    max_count           = 4
    os_disk_size_gb     = 128
    
    tags = var.tags
  }

  identity {
    type = "SystemAssigned"
  }

  network_profile {
    network_plugin     = "azure"
    network_policy     = "calico"
    dns_service_ip     = "10.0.0.10"
    service_cidr       = "10.0.0.0/24"
    docker_bridge_cidr = "172.17.0.1/16"
    load_balancer_sku  = "standard"
  }

  oms_agent {
    log_analytics_workspace_id = var.log_analytics_workspace_id
  }

  azure_active_directory_role_based_access_control {
    managed                = true
    azure_rbac_enabled     = true
  }

  role_based_access_control_enabled = true

  tags = var.tags
}

resource "azurerm_kubernetes_cluster_node_pool" "user" {
  name                  = "user"
  kubernetes_cluster_id = azurerm_kubernetes_cluster.main.id
  vm_size               = var.user_node_size
  node_count            = var.user_node_count
  enable_auto_scaling   = true
  min_count             = 1
  max_count             = 5
  vnet_subnet_id        = var.subnet_id
  
  tags = var.tags
}

# 授予AKS访问ACR的权限
resource "azurerm_role_assignment" "aks_acr" {
  principal_id         = azurerm_kubernetes_cluster.main.kubelet_identity[0].object_id
  role_definition_name = "AcrPull"
  scope                = var.acr_id
  skip_service_principal_aad_check = true
}
```

**验收**:
- [ ] AKS集群创建成功
- [ ] 双节点池配置正确
- [ ] 网络插件和策略配置
- [ ] Container Insights启用
- [ ] RBAC配置完成

---

### Task 2.4: 实现ACR模块 (2h)

**负责人**: DevOps Engineer

**模块功能**:
```hcl
# modules/acr/main.tf
resource "azurerm_container_registry" "main" {
  name                = replace("${var.prefix}acr", "-", "")
  resource_group_name = var.resource_group_name
  location            = var.location
  sku                 = var.sku
  admin_enabled       = false

  dynamic "georeplications" {
    for_each = var.georeplications
    content {
      location                = georeplications.value.location
      zone_redundancy_enabled = georeplications.value.zone_redundancy_enabled
      tags                    = var.tags
    }
  }

  network_rule_set {
    default_action = var.environment == "production" ? "Deny" : "Allow"
    
    ip_rule {
      action   = "Allow"
      ip_range = var.allowed_ip_ranges
    }
  }

  tags = var.tags
}

resource "azurerm_monitor_diagnostic_setting" "acr" {
  name                       = "${var.prefix}-acr-diag"
  target_resource_id         = azurerm_container_registry.main.id
  log_analytics_workspace_id = var.log_analytics_workspace_id

  log {
    category = "ContainerRegistryRepositoryEvents"
    enabled  = true
  }

  log {
    category = "ContainerRegistryLoginEvents"
    enabled  = true
  }

  metric {
    category = "AllMetrics"
    enabled  = true
  }
}
```

**验收**:
- [ ] ACR创建成功
- [ ] 网络规则配置(dev环境允许所有,main环境限制)
- [ ] 诊断日志启用

---

### Task 2.5: 实现Database模块 (4h)

**负责人**: DevOps Engineer

**模块功能**:
```hcl
# modules/database/main.tf
resource "azurerm_postgresql_flexible_server" "main" {
  name                   = "${var.prefix}-postgres"
  resource_group_name    = var.resource_group_name
  location               = var.location
  version                = "15"
  delegated_subnet_id    = var.subnet_id
  private_dns_zone_id    = azurerm_private_dns_zone.postgres.id
  administrator_login    = var.admin_username
  administrator_password = var.admin_password
  zone                   = "1"
  storage_mb             = var.storage_mb
  sku_name               = var.sku_name
  backup_retention_days  = var.backup_retention_days

  high_availability {
    mode                      = var.environment == "production" ? "ZoneRedundant" : "Disabled"
    standby_availability_zone = var.environment == "production" ? "2" : null
  }

  maintenance_window {
    day_of_week  = 0  # Sunday
    start_hour   = 3
    start_minute = 0
  }

  tags = var.tags
  
  depends_on = [azurerm_private_dns_zone_virtual_network_link.postgres]
}

resource "azurerm_private_dns_zone" "postgres" {
  name                = "${var.prefix}.private.postgres.database.azure.com"
  resource_group_name = var.resource_group_name
  tags                = var.tags
}

resource "azurerm_private_dns_zone_virtual_network_link" "postgres" {
  name                  = "${var.prefix}-postgres-vnet-link"
  private_dns_zone_name = azurerm_private_dns_zone.postgres.name
  resource_group_name   = var.resource_group_name
  virtual_network_id    = var.vnet_id
  tags                  = var.tags
}

resource "azurerm_postgresql_flexible_server_database" "hermesflow" {
  name      = "hermesflow"
  server_id = azurerm_postgresql_flexible_server.main.id
  collation = "en_US.utf8"
  charset   = "utf8"
}

resource "azurerm_postgresql_flexible_server_firewall_rule" "aks" {
  name             = "AllowAKS"
  server_id        = azurerm_postgresql_flexible_server.main.id
  start_ip_address = var.aks_outbound_ip
  end_ip_address   = var.aks_outbound_ip
}
```

**验收**:
- [ ] PostgreSQL Flexible Server创建
- [ ] Private DNS Zone配置
- [ ] VNet集成完成
- [ ] 数据库创建
- [ ] 防火墙规则配置

---

### Task 2.6: 实现Key Vault模块 (2h)

**负责人**: DevOps Engineer

**模块功能**:
```hcl
# modules/keyvault/main.tf
data "azurerm_client_config" "current" {}

resource "azurerm_key_vault" "main" {
  name                        = "${var.prefix}-kv"
  location                    = var.location
  resource_group_name         = var.resource_group_name
  enabled_for_disk_encryption = true
  tenant_id                   = data.azurerm_client_config.current.tenant_id
  soft_delete_retention_days  = 7
  purge_protection_enabled    = var.environment == "production"
  sku_name                    = "standard"

  network_acls {
    bypass                     = "AzureServices"
    default_action             = var.environment == "production" ? "Deny" : "Allow"
    ip_rules                   = var.allowed_ip_ranges
    virtual_network_subnet_ids = [var.aks_subnet_id]
  }

  tags = var.tags
}

# 授权AKS访问Key Vault
resource "azurerm_key_vault_access_policy" "aks" {
  key_vault_id = azurerm_key_vault.main.id
  tenant_id    = data.azurerm_client_config.current.tenant_id
  object_id    = var.aks_kubelet_identity_object_id

  secret_permissions = [
    "Get",
    "List",
  ]
}

# 存储数据库密码
resource "azurerm_key_vault_secret" "postgres_password" {
  name         = "postgres-admin-password"
  value        = var.postgres_admin_password
  key_vault_id = azurerm_key_vault.main.id
}

# 存储Redis密码
resource "azurerm_key_vault_secret" "redis_password" {
  name         = "redis-password"
  value        = random_password.redis.result
  key_vault_id = azurerm_key_vault.main.id
}

resource "random_password" "redis" {
  length  = 32
  special = true
}
```

**验收**:
- [ ] Key Vault创建成功
- [ ] 网络访问策略配置
- [ ] AKS访问权限授予
- [ ] 敏感信息存储

---

### Task 2.7: 实现Monitoring模块 (2h)

**负责人**: DevOps Engineer

**模块功能**:
```hcl
# modules/monitoring/main.tf
resource "azurerm_log_analytics_workspace" "main" {
  name                = "${var.prefix}-logs"
  location            = var.location
  resource_group_name = var.resource_group_name
  sku                 = "PerGB2018"
  retention_in_days   = var.log_retention_days
  tags                = var.tags
}

resource "azurerm_log_analytics_solution" "container_insights" {
  solution_name         = "ContainerInsights"
  location              = var.location
  resource_group_name   = var.resource_group_name
  workspace_resource_id = azurerm_log_analytics_workspace.main.id
  workspace_name        = azurerm_log_analytics_workspace.main.name

  plan {
    publisher = "Microsoft"
    product   = "OMSGallery/ContainerInsights"
  }

  tags = var.tags
}

# 创建常用查询
resource "azurerm_log_analytics_saved_search" "pod_errors" {
  name                       = "PodErrors"
  log_analytics_workspace_id = azurerm_log_analytics_workspace.main.id
  category                   = "Kubernetes"
  display_name               = "Pod Errors"
  query                      = <<-QUERY
    KubePodInventory
    | where ClusterName == "${var.cluster_name}"
    | where PodStatus == "Failed" or PodStatus == "CrashLoopBackOff"
    | project TimeGenerated, Namespace, Name, PodStatus, ContainerStatusReason
    | order by TimeGenerated desc
  QUERY
}

# 配置Alert规则
resource "azurerm_monitor_metric_alert" "aks_cpu" {
  name                = "${var.prefix}-aks-cpu-alert"
  resource_group_name = var.resource_group_name
  scopes              = [var.aks_id]
  description         = "Alert when AKS cluster CPU usage is high"
  severity            = 2
  frequency           = "PT5M"
  window_size         = "PT15M"

  criteria {
    metric_namespace = "Microsoft.ContainerService/managedClusters"
    metric_name      = "node_cpu_usage_percentage"
    aggregation      = "Average"
    operator         = "GreaterThan"
    threshold        = 80
  }

  action {
    action_group_id = azurerm_monitor_action_group.main.id
  }

  tags = var.tags
}

resource "azurerm_monitor_action_group" "main" {
  name                = "${var.prefix}-action-group"
  resource_group_name = var.resource_group_name
  short_name          = "HermesOps"

  email_receiver {
    name                    = "DevOps Team"
    email_address           = var.alert_email
    use_common_alert_schema = true
  }

  webhook_receiver {
    name        = "Slack"
    service_uri = var.slack_webhook_url
  }

  tags = var.tags
}
```

**验收**:
- [ ] Log Analytics Workspace创建
- [ ] Container Insights解决方案安装
- [ ] 常用查询保存
- [ ] Alert规则配置

---

### Task 2.8: 编写Dev环境主配置 (3h)

**负责人**: DevOps Lead

**文件**: `environments/dev/main.tf`

```hcl
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
      purge_soft_delete_on_destroy = true
    }
  }
}

locals {
  environment = "dev"
  prefix      = "hermesflow-dev"
  location    = "East US"
  
  tags = {
    Environment = "Development"
    Project     = "HermesFlow"
    ManagedBy   = "Terraform"
    CostCenter  = "Engineering"
  }
}

module "networking" {
  source = "../../modules/networking"
  
  resource_group_name = "${local.prefix}-rg"
  location            = local.location
  prefix              = local.prefix
  tags                = local.tags
}

module "monitoring" {
  source = "../../modules/monitoring"
  
  resource_group_name = module.networking.resource_group_name
  location            = local.location
  prefix              = local.prefix
  log_retention_days  = 30
  alert_email         = var.alert_email
  slack_webhook_url   = var.slack_webhook_url
  tags                = local.tags
}

module "acr" {
  source = "../../modules/acr"
  
  resource_group_name           = module.networking.resource_group_name
  location                      = local.location
  prefix                        = local.prefix
  sku                           = "Standard"
  environment                   = local.environment
  log_analytics_workspace_id    = module.monitoring.workspace_id
  allowed_ip_ranges             = var.allowed_ip_ranges
  georeplications               = []
  tags                          = local.tags
}

module "aks" {
  source = "../../modules/aks"
  
  resource_group_name           = module.networking.resource_group_name
  location                      = local.location
  prefix                        = local.prefix
  kubernetes_version            = "1.28"
  subnet_id                     = module.networking.aks_subnet_id
  system_node_count             = 2
  system_node_size              = "Standard_D4s_v3"
  user_node_count               = 1
  user_node_size                = "Standard_D8s_v3"
  log_analytics_workspace_id    = module.monitoring.workspace_id
  acr_id                        = module.acr.id
  tags                          = local.tags
  
  depends_on = [module.networking, module.acr]
}

module "database" {
  source = "../../modules/database"
  
  resource_group_name    = module.networking.resource_group_name
  location               = local.location
  prefix                 = local.prefix
  subnet_id              = module.networking.database_subnet_id
  vnet_id                = module.networking.vnet_id
  admin_username         = var.postgres_admin_username
  admin_password         = var.postgres_admin_password
  sku_name               = "B_Standard_B1ms"
  storage_mb             = 32768
  backup_retention_days  = 7
  environment            = local.environment
  aks_outbound_ip        = module.aks.outbound_ip
  tags                   = local.tags
  
  depends_on = [module.networking]
}

module "keyvault" {
  source = "../../modules/keyvault"
  
  resource_group_name             = module.networking.resource_group_name
  location                        = local.location
  prefix                          = local.prefix
  environment                     = local.environment
  aks_subnet_id                   = module.networking.aks_subnet_id
  aks_kubelet_identity_object_id  = module.aks.kubelet_identity_object_id
  postgres_admin_password         = var.postgres_admin_password
  allowed_ip_ranges               = var.allowed_ip_ranges
  tags                            = local.tags
  
  depends_on = [module.aks]
}
```

**验收**:
- [ ] 主配置文件完整
- [ ] 变量定义清晰
- [ ] 模块依赖关系正确

---

### Task 2.9: 配置Terraform Backend (2h)

**负责人**: DevOps Lead

**步骤**:

1. 手动创建Backend资源(仅一次):
```bash
# 创建Resource Group
az group create \
  --name hermesflow-tfstate-rg \
  --location eastus

# 创建Storage Account
az storage account create \
  --name hermesflowdevtfstate \
  --resource-group hermesflow-tfstate-rg \
  --location eastus \
  --sku Standard_LRS \
  --encryption-services blob

# 创建Blob Container
az storage container create \
  --name tfstate \
  --account-name hermesflowdevtfstate

# 启用版本控制
az storage account blob-service-properties update \
  --account-name hermesflowdevtfstate \
  --enable-versioning true
```

2. 配置访问权限:
```bash
# 授予Service Principal访问权限
az role assignment create \
  --role "Storage Blob Data Contributor" \
  --assignee $SERVICE_PRINCIPAL_ID \
  --scope "/subscriptions/$SUBSCRIPTION_ID/resourceGroups/hermesflow-tfstate-rg/providers/Microsoft.Storage/storageAccounts/hermesflowdevtfstate"
```

**验收**:
- [ ] Backend Storage Account创建
- [ ] 版本控制启用
- [ ] State文件成功存储
- [ ] State Locking工作正常

---

### Task 2.10: 集成Terraform到GitHub Actions (2h)

**负责人**: DevOps Lead

**工作流**: `.github/workflows/terraform.yml`

```yaml
name: Terraform - Azure Infrastructure

on:
  push:
    branches: [main]
    paths:
      - 'infrastructure/terraform/**'
  pull_request:
    paths:
      - 'infrastructure/terraform/**'
  workflow_dispatch:
    inputs:
      environment:
        description: 'Environment to deploy'
        required: true
        type: choice
        options:
          - dev
          - main

env:
  ARM_CLIENT_ID: ${{ secrets.AZURE_CLIENT_ID }}
  ARM_CLIENT_SECRET: ${{ secrets.AZURE_CLIENT_SECRET }}
  ARM_SUBSCRIPTION_ID: ${{ secrets.AZURE_SUBSCRIPTION_ID }}
  ARM_TENANT_ID: ${{ secrets.AZURE_TENANT_ID }}

jobs:
  terraform-plan:
    runs-on: ubuntu-latest
    strategy:
      matrix:
        environment: [dev]
    
    steps:
      - name: Checkout code
        uses: actions/checkout@v4

      - name: Setup Terraform
        uses: hashicorp/setup-terraform@v3
        with:
          terraform_version: 1.6.0

      - name: Terraform Format Check
        working-directory: infrastructure/terraform/environments/${{ matrix.environment }}
        run: terraform fmt -check -recursive

      - name: Terraform Init
        working-directory: infrastructure/terraform/environments/${{ matrix.environment }}
        run: terraform init

      - name: Terraform Validate
        working-directory: infrastructure/terraform/environments/${{ matrix.environment }}
        run: terraform validate

      - name: Terraform Plan
        working-directory: infrastructure/terraform/environments/${{ matrix.environment }}
        run: |
          terraform plan \
            -var="postgres_admin_password=${{ secrets.POSTGRES_ADMIN_PASSWORD }}" \
            -var="alert_email=${{ secrets.ALERT_EMAIL }}" \
            -var="slack_webhook_url=${{ secrets.SLACK_WEBHOOK_URL }}" \
            -out=tfplan

      - name: Upload Plan
        uses: actions/upload-artifact@v3
        with:
          name: tfplan-${{ matrix.environment }}
          path: infrastructure/terraform/environments/${{ matrix.environment }}/tfplan

  terraform-apply:
    needs: terraform-plan
    if: github.ref == 'refs/heads/main' && github.event_name == 'push'
    runs-on: ubuntu-latest
    strategy:
      matrix:
        environment: [dev]
    environment:
      name: ${{ matrix.environment }}
    
    steps:
      - name: Checkout code
        uses: actions/checkout@v4

      - name: Setup Terraform
        uses: hashicorp/setup-terraform@v3
        with:
          terraform_version: 1.6.0

      - name: Terraform Init
        working-directory: infrastructure/terraform/environments/${{ matrix.environment }}
        run: terraform init

      - name: Download Plan
        uses: actions/download-artifact@v3
        with:
          name: tfplan-${{ matrix.environment }}
          path: infrastructure/terraform/environments/${{ matrix.environment }}

      - name: Terraform Apply
        working-directory: infrastructure/terraform/environments/${{ matrix.environment }}
        run: terraform apply -auto-approve tfplan

      - name: Notify Slack on Success
        if: success()
        run: |
          curl -X POST ${{ secrets.SLACK_WEBHOOK_URL }} \
            -H 'Content-Type: application/json' \
            -d '{
              "text": "✅ Terraform Apply Success - ${{ matrix.environment }} environment"
            }'
```

**验收**:
- [ ] PR触发terraform plan
- [ ] main分支推送触发terraform apply
- [ ] Plan结果评论到PR
- [ ] Apply成功/失败通知Slack

---

## 📊 测试策略

### 1. Terraform Validation
- [ ] `terraform fmt -check` 通过
- [ ] `terraform validate` 无错误
- [ ] `terraform plan` 无意外变更
- [ ] tfsec安全扫描通过

### 2. 安全测试
- [ ] tfsec扫描无HIGH/CRITICAL问题
- [ ] Checkov策略检查通过
- [ ] 敏感变量标记为sensitive
- [ ] 网络规则最小化原则

### 3. 成本估算
- [ ] 使用Infracost估算月度成本
- [ ] Dev环境成本 < $500/月
- [ ] Main环境成本 < $2000/月
- [ ] 资源规格符合预算

### 4. 集成测试
- [ ] 创建测试环境验证所有资源
- [ ] AKS能成功连接ACR
- [ ] 数据库可从AKS访问
- [ ] Key Vault集成工作
- [ ] 监控和告警配置正确

### 5. 灾难恢复测试
- [ ] Terraform State备份验证
- [ ] 资源销毁和重建测试
- [ ] 跨区域故障转移(生产环境)

---

## 🔗 依赖关系

**前置依赖**:
- [ ] Azure订阅已创建并激活
- [ ] Service Principal已创建(用于Terraform)
- [ ] 必要的Azure Provider已注册
- [ ] Terraform CLI已安装(版本 >= 1.5)

**阻塞依赖**:
- 无 (此Story是基础设施的起点)

**后续依赖**:
- DEVOPS-001 (GitHub Actions CI/CD) 需要ACR地址
- DEVOPS-003 (ArgoCD安装) 需要AKS集群
- 所有应用部署 需要完整的基础设施

**并行可能性**:
- 可以与DEVOPS-001并行开发(但DEVOPS-001需要等待ACR创建)

---

## 📚 相关文档

- [系统架构 - 部署架构](../../architecture/system-architecture.md#6-部署架构设计)
- [GitOps最佳实践](../../deployment/gitops-best-practices.md)
- [Azure最佳实践](https://learn.microsoft.com/azure/architecture/best-practices/)
- [Terraform Azure Provider文档](https://registry.terraform.io/providers/hashicorp/azurerm/latest/docs)

---

## 🎓 学习资源

**Terraform基础**:
- [Terraform官方文档](https://www.terraform.io/docs)
- [Terraform Best Practices](https://www.terraform-best-practices.com/)

**Azure AKS**:
- [AKS Documentation](https://learn.microsoft.com/azure/aks/)
- [AKS Baseline Architecture](https://learn.microsoft.com/azure/architecture/reference-architectures/containers/aks/baseline-aks)

**成本优化**:
- [Azure Pricing Calculator](https://azure.microsoft.com/pricing/calculator/)
- [Azure Cost Management](https://learn.microsoft.com/azure/cost-management-billing/)

---

## ⚠️ 风险与假设

### 风险识别

1. **高风险** - Azure配额限制
   - **影响**: 无法创建所需规格的资源
   - **缓解**: 提前检查订阅配额,必要时申请增加

2. **中风险** - Terraform状态冲突
   - **影响**: 多人同时操作导致状态不一致
   - **缓解**: 使用State Locking,明确操作流程

3. **中风险** - 成本超支
   - **影响**: 月度账单超出预算
   - **缓解**: 使用Infracost估算,设置Azure Budget Alert

4. **低风险** - 网络配置错误
   - **影响**: 服务间无法通信
   - **缓解**: 详细的网络测试,NSG规则审查

### 假设条件

- Azure订阅有足够权限创建资源
- 开发团队有基础的Terraform知识
- 网络规划(10.0.0.0/16)无冲突
- 选择的Azure Region支持所有所需服务

---

## 💰 成本估算 (Dev环境)

| 资源 | 规格 | 预估月度成本 |
|------|------|-------------|
| AKS (System Pool) | 2 x Standard_D4s_v3 | ~$280 |
| AKS (User Pool) | 1 x Standard_D8s_v3 | ~$280 |
| ACR | Standard SKU | ~$20 |
| PostgreSQL | B1ms | ~$15 |
| Key Vault | Standard | ~$3 |
| Log Analytics | 5GB/day | ~$10 |
| VNet & NSG | Standard | ~$5 |
| **总计** | | **~$613/月** |

**成本优化建议**:
- 非工作时间停止User节点池(节省~$200/月)
- 使用Azure Reserved Instances(节省~30%)
- 定期清理未使用资源

---

## ✅ Definition of Done

**代码层面**:
- [ ] 所有Terraform模块完成并测试
- [ ] 代码通过terraform validate
- [ ] terraform fmt格式化检查通过
- [ ] tfsec扫描无HIGH级别问题
- [ ] 代码已Review并合并

**测试层面**:
- [ ] 在隔离环境成功创建所有资源
- [ ] 资源间连接测试通过(AKS<->ACR, AKS<->DB, AKS<->KeyVault)
- [ ] 成本估算在预算内
- [ ] 安全扫描通过
- [ ] 灾难恢复流程验证

**文档层面**:
- [ ] 每个模块有完整README
- [ ] 变量和输出有清晰描述
- [ ] 运维手册更新(创建/销毁/更新流程)
- [ ] 架构图更新(如有变更)
- [ ] 故障排查指南创建

**部署层面**:
- [ ] Dev环境成功部署
- [ ] GitHub Actions工作流测试通过
- [ ] State文件正确存储在Azure Storage
- [ ] 所有Azure资源标签正确
- [ ] 监控和告警配置验证

---

## 📝 开发笔记 (Dev/QA Notes)

**待开发团队填写**:

### 实现进度
- [ ] Task 2.1 完成
- [ ] Task 2.2 完成
- [ ] Task 2.3 完成
- [ ] Task 2.4 完成
- [ ] Task 2.5 完成
- [ ] Task 2.6 完成
- [ ] Task 2.7 完成
- [ ] Task 2.8 完成
- [ ] Task 2.9 完成
- [ ] Task 2.10 完成

### 技术决策记录
_开发过程中的重要技术决策将记录在此_

### 遇到的问题
_开发过程中遇到的问题和解决方案_

### 实际成本数据
_部署后的实际月度成本_

### 性能基准
_AKS集群性能测试结果_

---

## 🔄 Story History

| 日期 | 事件 | 操作人 |
|------|------|--------|
| 2025-01-13 | Story创建 | @sm.mdc |
| 2025-01-13 | Story验证通过 | @po.mdc |
| 2025-01-13 | Story批准进入Sprint Backlog | @po.mdc |

---

**Last Updated**: 2025-01-13  
**Next Review**: Sprint 1 Planning Meeting

