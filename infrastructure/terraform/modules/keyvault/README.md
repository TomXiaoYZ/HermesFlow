# Key Vault Module

This module creates and configures Azure Key Vault for secure storage of secrets, keys, and certificates used by HermesFlow.

## Resources Created

- **Azure Key Vault**: Standard SKU
- **Access Policies**: For Terraform and AKS
- **Secrets**:
  - PostgreSQL admin password
  - Redis password (auto-generated)
  - JWT secret (auto-generated)
  - Encryption key (auto-generated)

## Usage

```hcl
module "keyvault" {
  source = "../../modules/keyvault"
  
  resource_group_name             = "hermesflow-dev-rg"
  location                        = "East US"
  prefix                          = "hermesflow-dev"
  environment                     = "dev"
  aks_subnet_id                   = module.networking.aks_subnet_id
  aks_kubelet_identity_object_id  = module.aks.kubelet_identity_object_id
  postgres_admin_password         = var.postgres_admin_password
  allowed_ip_ranges               = []
  
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
| environment | Environment name | string | yes | - |
| aks_subnet_id | AKS subnet ID | string | yes | - |
| aks_kubelet_identity_object_id | AKS kubelet identity | string | yes | - |
| postgres_admin_password | PostgreSQL password | string | yes | - |
| allowed_ip_ranges | Allowed IP ranges | list(string) | no | [] |

## Outputs

| Name | Description |
|------|-------------|
| id | Key Vault ID |
| name | Key Vault name |
| vault_uri | Key Vault URI |
| redis_password | Redis password (sensitive) |
| jwt_secret | JWT secret (sensitive) |
| encryption_key | Encryption key (sensitive) |

## Stored Secrets

| Secret Name | Description | Source |
|-------------|-------------|--------|
| postgres-admin-password | PostgreSQL admin password | Provided via variable |
| redis-password | Redis password | Auto-generated (32 chars) |
| jwt-secret | JWT signing secret | Auto-generated (64 chars) |
| encryption-key | Data encryption key | Auto-generated (32 chars) |

## Access Policies

### Terraform Service Principal
- **Permissions**: Full management (Get, List, Set, Delete, Purge)
- **Purpose**: Manage secrets during deployment

### AKS Kubelet Identity
- **Permissions**: Read-only (Get, List)
- **Purpose**: Applications read secrets at runtime

## Network Security

- **Dev Environment**: Allow all (for development ease)
- **Production**: 
  - Deny all by default
  - Allow AKS subnet
  - Allow specific IP ranges

## Integration with AKS

Use Azure Key Vault Provider for Secrets Store CSI Driver:

```yaml
apiVersion: secrets-store.csi.x-k8s.io/v1
kind: SecretProviderClass
metadata:
  name: hermesflow-secrets
spec:
  provider: azure
  parameters:
    keyvaultName: hermesflow-dev-kv
    objects: |
      array:
        - |
          objectName: postgres-admin-password
          objectType: secret
        - |
          objectName: redis-password
          objectType: secret
```

## Soft Delete and Purge Protection

- **Soft Delete**: 7 days retention (always enabled)
- **Purge Protection**: Enabled in production only
- **Recovery**: Deleted secrets can be recovered within retention period

## Cost

- **Key Vault**: ~$0.03 per 10,000 operations
- **Storage**: First 10,000 secrets free
- **Typical Cost**: < $5/month

