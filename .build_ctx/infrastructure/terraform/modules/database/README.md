# Database Module (PostgreSQL Flexible Server)

This module creates and configures an Azure PostgreSQL Flexible Server for HermesFlow application data storage.

## Resources Created

- **PostgreSQL Flexible Server**: PostgreSQL 15
- **Private DNS Zone**: For VNet integration
- **VNet Link**: Connects DNS zone to VNet
- **Database**: hermesflow database
- **Firewall Rules**: Allow AKS access
- **Performance Configs**: Optimized settings

## Usage

```hcl
module "database" {
  source = "../../modules/database"
  
  resource_group_name    = "hermesflow-dev-rg"
  location               = "East US"
  prefix                 = "hermesflow-dev"
  subnet_id              = module.networking.database_subnet_id
  vnet_id                = module.networking.vnet_id
  admin_username         = "hermesadmin"
  admin_password         = var.postgres_admin_password
  sku_name               = "B_Standard_B1ms"
  storage_mb             = 32768
  backup_retention_days  = 7
  environment            = "dev"
  aks_outbound_ip        = module.aks.outbound_ip
  
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
| subnet_id | Database subnet ID | string | yes | - |
| vnet_id | Virtual network ID | string | yes | - |
| admin_username | PostgreSQL admin user | string | no | hermesadmin |
| admin_password | PostgreSQL admin password | string | yes | - |
| sku_name | PostgreSQL SKU | string | no | B_Standard_B1ms |
| storage_mb | Storage size in MB | number | no | 32768 |
| backup_retention_days | Backup retention days | number | no | 7 |
| environment | Environment name | string | yes | - |
| aks_outbound_ip | AKS outbound IP | string | yes | - |

## Outputs

| Name | Description |
|------|-------------|
| server_id | PostgreSQL server ID |
| server_name | Server name |
| server_fqdn | Server FQDN |
| database_name | Database name |
| connection_string | Full connection string (sensitive) |
| admin_username | Admin username (sensitive) |

## SKU Options

### Development
- **B_Standard_B1ms**: 1 vCore, 2 GB RAM (~$15/month)
- **B_Standard_B2s**: 2 vCores, 4 GB RAM (~$30/month)

### Production
- **GP_Standard_D2s_v3**: 2 vCores, 8 GB RAM (~$150/month)
- **GP_Standard_D4s_v3**: 4 vCores, 16 GB RAM (~$300/month)

## High Availability

- **Dev**: Disabled
- **Production**: Zone-redundant (automatic failover)

## Backup Configuration

- **Retention**: 7 days (dev), 35 days (production)
- **Type**: Automated daily backups
- **Recovery**: Point-in-time restore

## Security Features

- **VNet Integration**: Private endpoint, no public access
- **Firewall Rules**: Only AKS cluster can connect
- **SSL/TLS**: Enforced for all connections
- **Private DNS**: Internal name resolution

## Connection from AKS

```yaml
# Example Kubernetes secret
apiVersion: v1
kind: Secret
metadata:
  name: postgres-connection
type: Opaque
stringData:
  host: hermesflow-dev-postgres.postgres.database.azure.com
  port: "5432"
  database: hermesflow
  username: hermesadmin
  password: <from-keyvault>
  sslmode: require
```

## Maintenance Window

- **Day**: Sunday
- **Time**: 03:00 UTC
- **Duration**: Up to 1 hour

