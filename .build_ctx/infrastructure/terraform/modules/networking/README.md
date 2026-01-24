# Networking Module

This module creates the foundational network infrastructure for HermesFlow, including VNet, subnets, and network security groups.

## Resources Created

- **Resource Group**: Container for all Azure resources
- **Virtual Network**: 10.0.0.0/16 address space
- **Subnets**:
  - AKS Subnet (10.0.1.0/24): For Kubernetes nodes and pods
  - Database Subnet (10.0.2.0/24): For PostgreSQL with service delegation
  - AppGateway Subnet (10.0.3.0/24): Reserved for future Application Gateway
- **Network Security Groups**: Security rules for AKS and Database subnets

## Usage

```hcl
module "networking" {
  source = "../../modules/networking"
  
  resource_group_name = "hermesflow-dev-rg"
  location            = "East US"
  prefix              = "hermesflow-dev"
  
  tags = {
    Environment = "Development"
    Project     = "HermesFlow"
    ManagedBy   = "Terraform"
  }
}
```

## Inputs

| Name | Description | Type | Required |
|------|-------------|------|----------|
| resource_group_name | Name of the resource group | string | yes |
| location | Azure region | string | yes |
| prefix | Prefix for resource names | string | yes |
| tags | Tags to apply to resources | map(string) | no |

## Outputs

| Name | Description |
|------|-------------|
| resource_group_name | Name of the resource group |
| vnet_id | Virtual network ID |
| aks_subnet_id | AKS subnet ID |
| database_subnet_id | Database subnet ID |
| appgw_subnet_id | Application gateway subnet ID |

## Network Architecture

```
Virtual Network (10.0.0.0/16)
├── AKS Subnet (10.0.1.0/24)
│   └── AKS Nodes + Pods
├── Database Subnet (10.0.2.0/24)
│   └── PostgreSQL (Private Endpoint)
└── AppGateway Subnet (10.0.3.0/24)
    └── Reserved for future use
```

## Security

- Network Security Groups control traffic flow
- Database subnet only accepts connections from AKS subnet
- PostgreSQL delegation enables VNet integration

