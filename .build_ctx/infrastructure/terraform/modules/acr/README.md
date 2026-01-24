# Azure Container Registry (ACR) Module

This module creates and configures an Azure Container Registry for storing Docker images used by HermesFlow microservices.

## Resources Created

- **Azure Container Registry**: Standard SKU for dev, Premium for production
- **Diagnostic Settings**: Logs for repository events and login events
- **Network Rules**: Configurable IP whitelist for enhanced security

## Usage

```hcl
module "acr" {
  source = "../../modules/acr"
  
  resource_group_name           = "hermesflow-dev-rg"
  location                      = "East US"
  prefix                        = "hermesflow-dev"
  sku                           = "Standard"
  environment                   = "dev"
  log_analytics_workspace_id    = module.monitoring.workspace_id
  allowed_ip_ranges             = []
  georeplications               = []
  
  tags = {
    Environment = "Development"
    Project     = "HermesFlow"
  }
}
```

## Inputs

| Name | Description | Type | Required | Default |
|------|-------------|------|----------|---------|
| resource_group_name | Resource group name | string | yes | - |
| location | Azure region | string | yes | - |
| prefix | Resource name prefix | string | yes | - |
| sku | ACR SKU (Basic/Standard/Premium) | string | no | Standard |
| environment | Environment name | string | yes | - |
| log_analytics_workspace_id | Log Analytics workspace ID | string | yes | - |
| allowed_ip_ranges | Allowed IP ranges | list(string) | no | [] |
| georeplications | Geo-replication configs | list(object) | no | [] |

## Outputs

| Name | Description |
|------|-------------|
| id | ACR resource ID |
| name | ACR name |
| login_server | ACR login server URL |
| admin_username | Admin username (sensitive) |
| admin_password | Admin password (sensitive) |

## Security Features

- **Managed Identity**: Admin account disabled by default, use Managed Identity
- **Network Rules**: Dev allows all, Production restricts by IP
- **Audit Logging**: All repository and login events logged
- **Geo-Replication**: Optional for high availability in production

## Integration with AKS

AKS clusters are granted `AcrPull` role via Managed Identity, configured in the AKS module.

## Cost Optimization

- **Dev Environment**: Standard SKU (~$20/month)
- **Production**: Premium SKU with geo-replication (~$100+/month)

