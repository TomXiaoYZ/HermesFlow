# AKS (Azure Kubernetes Service) Module

This module creates and configures an Azure Kubernetes Service cluster for running HermesFlow microservices.

## Resources Created

- **AKS Cluster**: Managed Kubernetes cluster
- **System Node Pool**: 2-4 nodes (auto-scaling) for system workloads
- **User Node Pool**: 1-5 nodes (auto-scaling) for application workloads
- **Role Assignment**: AcrPull permission for ACR integration

## Usage

```hcl
module "aks" {
  source = "../../modules/aks"
  
  resource_group_name           = "hermesflow-dev-rg"
  location                      = "East US"
  prefix                        = "hermesflow-dev"
  kubernetes_version            = "1.28"
  subnet_id                     = module.networking.aks_subnet_id
  system_node_count             = 2
  system_node_size              = "Standard_D4s_v3"
  user_node_count               = 1
  user_node_size                = "Standard_D8s_v3"
  log_analytics_workspace_id    = module.monitoring.workspace_id
  acr_id                        = module.acr.id
  
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
| kubernetes_version | Kubernetes version | string | no | 1.28 |
| subnet_id | AKS subnet ID | string | yes | - |
| system_node_count | System pool node count | number | no | 2 |
| system_node_size | System pool VM size | string | no | Standard_D4s_v3 |
| user_node_count | User pool node count | number | no | 1 |
| user_node_size | User pool VM size | string | no | Standard_D8s_v3 |
| log_analytics_workspace_id | Log Analytics ID | string | yes | - |
| acr_id | ACR ID | string | yes | - |

## Outputs

| Name | Description |
|------|-------------|
| cluster_id | AKS cluster ID |
| cluster_name | AKS cluster name |
| kube_config | Kubernetes config (sensitive) |
| kubelet_identity_object_id | Kubelet identity object ID |
| cluster_fqdn | Cluster FQDN |
| outbound_ip | Cluster outbound IP |

## Node Pools

### System Node Pool
- **Purpose**: Kubernetes system components (CoreDNS, metrics-server, etc.)
- **VM Size**: Standard_D4s_v3 (4 vCPU, 16 GB RAM)
- **Scaling**: 2-4 nodes
- **Cost**: ~$280/month

### User Node Pool
- **Purpose**: Application workloads
- **VM Size**: Standard_D8s_v3 (8 vCPU, 32 GB RAM)
- **Scaling**: 1-5 nodes
- **Cost**: ~$280/month (1 node)

## Network Configuration

- **Plugin**: Azure CNI (advanced networking)
- **Policy**: Calico (network policy enforcement)
- **Service CIDR**: 10.0.0.0/24 (for Kubernetes services)
- **DNS Service IP**: 10.0.0.10

## Security Features

- **Azure AD Integration**: Managed AAD with RBAC
- **Managed Identity**: System-assigned identity for Azure resources
- **Network Policy**: Calico for pod-to-pod security
- **Container Insights**: Monitoring integration

## Access the Cluster

```bash
# Get credentials
az aks get-credentials --resource-group hermesflow-dev-rg --name hermesflow-dev-aks

# Verify access
kubectl get nodes
kubectl get pods --all-namespaces
```

## Cost Optimization

- **Auto-scaling**: Scales down during low usage
- **Spot Instances**: Consider for non-production (future)
- **Node Pool Schedules**: Start/stop pools on schedule

