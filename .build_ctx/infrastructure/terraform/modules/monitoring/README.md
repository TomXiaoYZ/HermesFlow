# Monitoring Module

This module sets up comprehensive monitoring and alerting for HermesFlow infrastructure using Azure Monitor and Log Analytics.

## Resources Created

- **Log Analytics Workspace**: Centralized logging for all resources
- **Container Insights**: Kubernetes-specific monitoring solution
- **Saved Queries**: Pre-configured queries for common troubleshooting
- **Action Group**: Email and Slack notification channel
- **Metric Alerts**: CPU, memory, and pod count alerts for AKS

## Usage

```hcl
module "monitoring" {
  source = "../../modules/monitoring"
  
  resource_group_name = "hermesflow-dev-rg"
  location            = "East US"
  prefix              = "hermesflow-dev"
  log_retention_days  = 30
  cluster_name        = "hermesflow-dev-aks"
  aks_id              = module.aks.cluster_id
  alert_email         = "devops@hermesflow.io"
  slack_webhook_url   = var.slack_webhook_url
  
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
| log_retention_days | Days to retain logs | number | no | 30 |
| cluster_name | AKS cluster name | string | no | "" |
| aks_id | AKS cluster ID | string | no | "" |
| alert_email | Email for alerts | string | yes | - |
| slack_webhook_url | Slack webhook URL | string | yes | - |

## Outputs

| Name | Description |
|------|-------------|
| workspace_id | Log Analytics workspace ID |
| workspace_name | Workspace name |
| workspace_key | Workspace key (sensitive) |
| action_group_id | Action group ID |

## Configured Alerts

### High CPU Usage
- **Threshold**: >80% average CPU
- **Window**: 15 minutes
- **Severity**: Warning

### High Memory Usage
- **Threshold**: >85% working set
- **Window**: 15 minutes
- **Severity**: Warning

### Pod Health
- **Threshold**: <80% pods ready
- **Window**: 15 minutes
- **Severity**: Informational

## Saved Queries

1. **Pod Errors**: Failed or CrashLoopBackOff pods
2. **High CPU Usage**: Nodes with sustained high CPU

## Cost

- **Log Analytics**: ~$2.50/GB ingested (first 5GB free)
- **Typical Dev Usage**: ~5GB/day = ~$10/month
- **Alerts**: Free (first 1000 signals/month)

