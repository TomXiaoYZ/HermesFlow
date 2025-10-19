# HermesFlow Dev Environment Setup

This guide walks you through setting up the development environment infrastructure on Azure.

## Prerequisites

1. **Azure CLI** (version >= 2.50)
   ```bash
   az --version
   az login
   ```

2. **Terraform** (version >= 1.5)
   ```bash
   terraform --version
   ```

3. **Azure Subscription** with appropriate permissions
   - Owner or Contributor role
   - Ability to create Service Principals

## Step 1: Create Service Principal

Create a service principal for Terraform authentication:

```bash
# Create service principal
az ad sp create-for-rbac --name "hermesflow-terraform-sp" \
  --role="Contributor" \
  --scopes="/subscriptions/YOUR_SUBSCRIPTION_ID"
```

Save the output:
```json
{
  "appId": "YOUR_APP_ID",
  "displayName": "hermesflow-terraform-sp",
  "password": "YOUR_PASSWORD",
  "tenant": "YOUR_TENANT_ID"
}
```

## Step 2: Create Terraform State Backend

The Terraform state must be stored in Azure Storage for team collaboration.

```bash
# Set variables
RESOURCE_GROUP="hermesflow-tfstate-rg"
STORAGE_ACCOUNT="hermesflowdevtfstate"
CONTAINER_NAME="tfstate"
LOCATION="eastus"

# Create resource group
az group create \
  --name $RESOURCE_GROUP \
  --location $LOCATION

# Create storage account
az storage account create \
  --name $STORAGE_ACCOUNT \
  --resource-group $RESOURCE_GROUP \
  --location $LOCATION \
  --sku Standard_LRS \
  --encryption-services blob \
  --min-tls-version TLS1_2

# Get storage account key
ACCOUNT_KEY=$(az storage account keys list \
  --resource-group $RESOURCE_GROUP \
  --account-name $STORAGE_ACCOUNT \
  --query '[0].value' -o tsv)

# Create blob container
az storage container create \
  --name $CONTAINER_NAME \
  --account-name $STORAGE_ACCOUNT \
  --account-key $ACCOUNT_KEY

# Enable blob versioning
az storage account blob-service-properties update \
  --account-name $STORAGE_ACCOUNT \
  --resource-group $RESOURCE_GROUP \
  --enable-versioning true
```

## Step 3: Set Environment Variables

Set these environment variables for Terraform authentication:

```bash
# Azure Authentication
export ARM_CLIENT_ID="YOUR_APP_ID"
export ARM_CLIENT_SECRET="YOUR_PASSWORD"
export ARM_SUBSCRIPTION_ID="YOUR_SUBSCRIPTION_ID"
export ARM_TENANT_ID="YOUR_TENANT_ID"

# Terraform Variables
export TF_VAR_postgres_admin_password="YOUR_SECURE_PASSWORD"
export TF_VAR_slack_webhook_url="YOUR_SLACK_WEBHOOK"
export TF_VAR_alert_email="devops@hermesflow.io"
```

Add to `~/.zshrc` or `~/.bashrc` for persistence:
```bash
# HermesFlow Terraform
export ARM_CLIENT_ID="YOUR_APP_ID"
export ARM_CLIENT_SECRET="YOUR_PASSWORD"
export ARM_SUBSCRIPTION_ID="YOUR_SUBSCRIPTION_ID"
export ARM_TENANT_ID="YOUR_TENANT_ID"
```

## Step 4: Check Azure Quota

Ensure your subscription has enough quota:

```bash
# Check vCPU quota in East US
az vm list-usage --location "East US" --query "[?name.value=='cores'].{Name:name.localizedValue, Current:currentValue, Limit:limit}" -o table

# Check Public IP quota
az network list-usages --location "East US" --query "[?name.value=='PublicIPAddresses'].{Name:name.localizedValue, Current:currentValue, Limit:limit}" -o table
```

**Required for dev environment:**
- vCPU cores: at least 20
- Public IPs: at least 5
- Load Balancers: at least 2

If quota is insufficient:
```bash
# Request quota increase
az support quota create --quota-params '[{"name":"cores","limit":50}]' --region "East US"
```

## Step 5: Initialize Terraform

```bash
cd infrastructure/terraform/environments/dev

# Initialize Terraform (downloads providers and configures backend)
terraform init

# Validate configuration
terraform validate

# Format code
terraform fmt -recursive
```

## Step 6: Plan Infrastructure

```bash
# Generate execution plan
terraform plan -out=tfplan

# Review the plan carefully
# Should show creation of ~30-40 resources
```

Expected resources:
- 1 Resource Group
- 1 VNet with 3 Subnets
- 2 Network Security Groups
- 1 AKS Cluster (with 2 node pools)
- 1 Azure Container Registry
- 1 PostgreSQL Flexible Server
- 1 Key Vault
- 1 Log Analytics Workspace
- Multiple monitoring alerts

## Step 7: Apply Infrastructure

```bash
# Apply the plan (creates resources)
terraform apply tfplan

# This will take 15-20 minutes (AKS cluster creation is slow)
```

## Step 8: Verify Deployment

```bash
# Get all outputs
terraform output

# Get AKS credentials
az aks get-credentials \
  --resource-group hermesflow-dev-rg \
  --name hermesflow-dev-aks

# Verify cluster
kubectl get nodes
kubectl get pods --all-namespaces

# Verify ACR
az acr list --resource-group hermesflow-dev-rg --output table

# Verify PostgreSQL
az postgres flexible-server list --resource-group hermesflow-dev-rg --output table
```

## Step 9: Configure GitHub Secrets

Get credentials for GitHub Actions:

```bash
# ACR credentials
az acr credential show --name $(terraform output -raw acr_name)

# AKS info
terraform output aks_cluster_name
terraform output aks_get_credentials_command

# Key Vault name
terraform output keyvault_name

# PostgreSQL connection string (sensitive)
terraform output -raw postgres_connection_string
```

Configure these in GitHub:
- Repository → Settings → Secrets and variables → Actions
- Add secrets listed in `docs/deployment/github-secrets-setup.md`

## Troubleshooting

### Error: Insufficient quota

```
Error: creating Kubernetes Cluster: Code="QuotaExceeded"
```

**Solution**: Request quota increase (see Step 4)

### Error: Backend initialization failed

```
Error: Failed to get existing workspaces: containers.Client#ListBlobs
```

**Solution**: Verify storage account exists and credentials are correct

### Error: Key Vault access denied

```
Error: checking for presence of existing Secret: keyvault.BaseClient#GetSecret
```

**Solution**: Ensure service principal has Key Vault access policy

### Error: PostgreSQL subnet delegation

```
Error: subnet must have Microsoft.DBforPostgreSQL/flexibleServers delegation
```

**Solution**: Already configured in networking module, ensure clean apply

## Cost Monitoring

After deployment, monitor costs:

```bash
# View current month's cost
az consumption usage list \
  --start-date $(date -u -d "month start" +%Y-%m-%d) \
  --end-date $(date -u +%Y-%m-%d) \
  --query "[].{Date:usageStart, Cost:pretaxCost}" -o table

# Set up budget alert
az consumption budget create \
  --budget-name "hermesflow-dev-monthly" \
  --amount 700 \
  --time-grain Monthly \
  --category Cost
```

## Cleanup (Destroy Infrastructure)

**WARNING**: This will delete all resources and data!

```bash
# Preview what will be destroyed
terraform plan -destroy

# Destroy all resources
terraform destroy

# Clean up state backend (optional)
az group delete --name hermesflow-tfstate-rg --yes
```

## Next Steps

1. ✅ Infrastructure deployed
2. Configure GitHub Actions (see `docs/deployment/github-secrets-setup.md`)
3. Deploy HermesFlow services via GitOps
4. Configure monitoring dashboards
5. Run integration tests

## Support

For issues:
- Check Terraform state: `terraform show`
- View logs: Azure Portal → Resource Group → Activity log
- Slack: `#hermesflow-devops`
- Email: devops@hermesflow.io

