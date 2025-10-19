# GitHub Secrets Setup Guide

This guide explains how to configure GitHub Secrets for HermesFlow CI/CD pipelines.

## 📋 Required Secrets

### Azure Authentication

| Secret Name | Description | How to Get |
|-------------|-------------|------------|
| `AZURE_CLIENT_ID` | Service Principal Application ID | See [Create Service Principal](#create-service-principal) |
| `AZURE_CLIENT_SECRET` | Service Principal Password | See [Create Service Principal](#create-service-principal) |
| `AZURE_SUBSCRIPTION_ID` | Azure Subscription ID | `az account show --query id -o tsv` |
| `AZURE_TENANT_ID` | Azure AD Tenant ID | `az account show --query tenantId -o tsv` |

### Azure Container Registry

| Secret Name | Description | How to Get |
|-------------|-------------|------------|
| `ACR_LOGIN_SERVER` | ACR login server URL | `hermesflowdevacr.azurecr.io` or `az acr show -n hermesflowdevacr --query loginServer -o tsv` |
| `ACR_USERNAME` | ACR username (same as Service Principal ID) | Same as `AZURE_CLIENT_ID` |
| `ACR_PASSWORD` | ACR password (same as Service Principal Secret) | Same as `AZURE_CLIENT_SECRET` |

### GitOps

| Secret Name | Description | How to Get |
|-------------|-------------|------------|
| `GITOPS_PAT` | GitHub Personal Access Token for GitOps repo | See [Create Personal Access Token](#create-personal-access-token) |

### Database

| Secret Name | Description | How to Get |
|-------------|-------------|------------|
| `POSTGRES_ADMIN_PASSWORD` | PostgreSQL administrator password | Create a strong password (min 12 chars) |

### Monitoring & Notifications

| Secret Name | Description | How to Get |
|-------------|-------------|------------|
| `SLACK_WEBHOOK_URL` | Slack webhook for notifications | See [Create Slack Webhook](#create-slack-webhook) |
| `ALERT_EMAIL` | Email for alert notifications | Your team email (e.g., devops@hermesflow.io) |

### Code Coverage (Optional)

| Secret Name | Description | How to Get |
|-------------|-------------|------------|
| `CODECOV_TOKEN` | Codecov token for coverage reports | Sign up at [codecov.io](https://codecov.io) |

---

## 🔧 Setup Instructions

### Create Service Principal

Create an Azure Service Principal for Terraform and CI/CD authentication:

```bash
# Login to Azure
az login

# Set your subscription
az account set --subscription "YOUR_SUBSCRIPTION_NAME_OR_ID"

# Create service principal
az ad sp create-for-rbac \
  --name "hermesflow-terraform-sp" \
  --role="Contributor" \
  --scopes="/subscriptions/$(az account show --query id -o tsv)"
```

**Output**:
```json
{
  "appId": "12345678-1234-1234-1234-123456789abc",
  "displayName": "hermesflow-terraform-sp",
  "password": "your-secret-password-here",
  "tenant": "87654321-4321-4321-4321-cba987654321"
}
```

**Map to GitHub Secrets**:
- `appId` → `AZURE_CLIENT_ID` and `ACR_USERNAME`
- `password` → `AZURE_CLIENT_SECRET` and `ACR_PASSWORD`
- `tenant` → `AZURE_TENANT_ID`

**Grant Additional Permissions**:
```bash
# Grant User Access Administrator role (for role assignments)
az role assignment create \
  --assignee "YOUR_APP_ID" \
  --role "User Access Administrator" \
  --scope "/subscriptions/$(az account show --query id -o tsv)"

# Grant permission to manage ACR
az role assignment create \
  --assignee "YOUR_APP_ID" \
  --role "AcrPush" \
  --scope "/subscriptions/$(az account show --query id -o tsv)/resourceGroups/hermesflow-dev-rg/providers/Microsoft.ContainerRegistry/registries/hermesflowdevacr"
```

### Create Personal Access Token

Create a GitHub Personal Access Token for GitOps repository updates:

1. Go to GitHub → Settings → Developer settings → Personal access tokens → Tokens (classic)
2. Click "Generate new token (classic)"
3. Configure the token:
   - **Note**: `HermesFlow GitOps`
   - **Expiration**: 90 days (or custom)
   - **Select scopes**:
     - ✅ `repo` (Full control of private repositories)
     - ✅ `workflow` (Update GitHub Action workflows)
4. Click "Generate token"
5. **Copy the token immediately** (you won't see it again!)

**Save as**: `GITOPS_PAT`

### Create Slack Webhook

Set up Slack notifications for CI/CD events:

1. Go to your Slack workspace
2. Navigate to: https://api.slack.com/apps
3. Click "Create New App" → "From scratch"
4. Name: `HermesFlow CI/CD`
5. Select your workspace
6. Go to "Incoming Webhooks"
7. Activate Incoming Webhooks
8. Click "Add New Webhook to Workspace"
9. Select the channel (e.g., `#hermesflow-cicd`)
10. Copy the Webhook URL

**Example**: `https://hooks.slack.com/services/T00000000/B00000000/XXXXXXXXXXXXXXXXXXXXXXXX`

**Save as**: `SLACK_WEBHOOK_URL`

### Generate PostgreSQL Password

Generate a strong password for PostgreSQL:

```bash
# Generate a secure random password
openssl rand -base64 32

# Or use a password manager
# Minimum requirements:
# - 12+ characters
# - Mix of uppercase, lowercase, numbers, special chars
```

**Save as**: `POSTGRES_ADMIN_PASSWORD`

**Important**: 
- Do NOT commit this password to Git
- Store in a secure password manager
- Rotate every 90 days

### Get Azure Subscription Info

```bash
# Get Subscription ID
az account show --query id -o tsv

# Get Tenant ID
az account show --query tenantId -o tsv

# Get Subscription Name
az account show --query name -o tsv
```

---

## 📝 Configure GitHub Secrets

### Method 1: GitHub Web UI

1. Go to your repository on GitHub
2. Click **Settings** → **Secrets and variables** → **Actions**
3. Click **New repository secret**
4. Enter secret name and value
5. Click **Add secret**
6. Repeat for all secrets

### Method 2: GitHub CLI

```bash
# Install GitHub CLI
brew install gh

# Authenticate
gh auth login

# Add secrets
gh secret set AZURE_CLIENT_ID --body "YOUR_VALUE"
gh secret set AZURE_CLIENT_SECRET --body "YOUR_VALUE"
gh secret set AZURE_SUBSCRIPTION_ID --body "YOUR_VALUE"
gh secret set AZURE_TENANT_ID --body "YOUR_VALUE"
gh secret set ACR_LOGIN_SERVER --body "hermesflowdevacr.azurecr.io"
gh secret set ACR_USERNAME --body "YOUR_VALUE"
gh secret set ACR_PASSWORD --body "YOUR_VALUE"
gh secret set GITOPS_PAT --body "YOUR_VALUE"
gh secret set POSTGRES_ADMIN_PASSWORD --body "YOUR_VALUE"
gh secret set SLACK_WEBHOOK_URL --body "YOUR_VALUE"
gh secret set ALERT_EMAIL --body "devops@hermesflow.io"
gh secret set CODECOV_TOKEN --body "YOUR_VALUE"  # Optional
```

### Method 3: Script (Recommended)

Create a script `setup-secrets.sh`:

```bash
#!/bin/bash
set -e

# Check if gh is installed
if ! command -v gh &> /dev/null; then
    echo "GitHub CLI is not installed. Install it with: brew install gh"
    exit 1
fi

# Check if logged in
gh auth status || gh auth login

echo "Setting up GitHub Secrets for HermesFlow..."

# Azure
read -p "Azure Client ID: " AZURE_CLIENT_ID
gh secret set AZURE_CLIENT_ID --body "$AZURE_CLIENT_ID"

read -sp "Azure Client Secret: " AZURE_CLIENT_SECRET
echo
gh secret set AZURE_CLIENT_SECRET --body "$AZURE_CLIENT_SECRET"

read -p "Azure Subscription ID: " AZURE_SUBSCRIPTION_ID
gh secret set AZURE_SUBSCRIPTION_ID --body "$AZURE_SUBSCRIPTION_ID"

read -p "Azure Tenant ID: " AZURE_TENANT_ID
gh secret set AZURE_TENANT_ID --body "$AZURE_TENANT_ID"

# ACR
read -p "ACR Login Server [hermesflowdevacr.azurecr.io]: " ACR_LOGIN_SERVER
ACR_LOGIN_SERVER=${ACR_LOGIN_SERVER:-hermesflowdevacr.azurecr.io}
gh secret set ACR_LOGIN_SERVER --body "$ACR_LOGIN_SERVER"

gh secret set ACR_USERNAME --body "$AZURE_CLIENT_ID"
gh secret set ACR_PASSWORD --body "$AZURE_CLIENT_SECRET"

# GitOps
read -sp "GitOps PAT: " GITOPS_PAT
echo
gh secret set GITOPS_PAT --body "$GITOPS_PAT"

# Database
read -sp "PostgreSQL Admin Password: " POSTGRES_ADMIN_PASSWORD
echo
gh secret set POSTGRES_ADMIN_PASSWORD --body "$POSTGRES_ADMIN_PASSWORD"

# Notifications
read -p "Slack Webhook URL: " SLACK_WEBHOOK_URL
gh secret set SLACK_WEBHOOK_URL --body "$SLACK_WEBHOOK_URL"

read -p "Alert Email [devops@hermesflow.io]: " ALERT_EMAIL
ALERT_EMAIL=${ALERT_EMAIL:-devops@hermesflow.io}
gh secret set ALERT_EMAIL --body "$ALERT_EMAIL"

# Optional
read -p "Codecov Token (optional, press enter to skip): " CODECOV_TOKEN
if [ -n "$CODECOV_TOKEN" ]; then
    gh secret set CODECOV_TOKEN --body "$CODECOV_TOKEN"
fi

echo "✅ All secrets configured successfully!"
echo "Verify: gh secret list"
```

Run the script:
```bash
chmod +x setup-secrets.sh
./setup-secrets.sh
```

---

## ✅ Verify Configuration

### List all secrets

```bash
gh secret list
```

Expected output:
```
ALERT_EMAIL               Updated 2025-01-13
AZURE_CLIENT_ID           Updated 2025-01-13
AZURE_CLIENT_SECRET       Updated 2025-01-13
AZURE_SUBSCRIPTION_ID     Updated 2025-01-13
AZURE_TENANT_ID           Updated 2025-01-13
ACR_LOGIN_SERVER          Updated 2025-01-13
ACR_USERNAME              Updated 2025-01-13
ACR_PASSWORD              Updated 2025-01-13
CODECOV_TOKEN             Updated 2025-01-13
GITOPS_PAT                Updated 2025-01-13
POSTGRES_ADMIN_PASSWORD   Updated 2025-01-13
SLACK_WEBHOOK_URL         Updated 2025-01-13
```

### Test Terraform workflow

```bash
# Trigger Terraform validation
git checkout -b test/secrets-validation
git push origin test/secrets-validation

# Create a PR to test terraform plan
gh pr create --title "Test: Verify Secrets Configuration" --body "Testing GitHub Secrets setup"
```

Check if the Terraform workflow runs successfully.

---

## 🔒 Security Best Practices

### 1. Rotate Secrets Regularly

- Service Principal: Every 90 days
- Personal Access Token: Every 90 days
- PostgreSQL Password: Every 90 days
- Slack Webhook: When team members leave

### 2. Use Least Privilege

- Grant only necessary permissions to Service Principal
- Use separate Service Principals for dev and production
- Limit PAT scope to specific repositories

### 3. Monitor Secret Usage

```bash
# View workflow runs
gh run list

# View specific run
gh run view RUN_ID

# Check for failed authentications
gh run list --workflow=terraform.yml --status=failure
```

### 4. Audit Trail

- Enable Azure AD audit logs
- Monitor Service Principal sign-ins
- Set up alerts for failed authentication attempts

### 5. Emergency Response

If secrets are compromised:

```bash
# 1. Immediately revoke Service Principal
az ad sp delete --id YOUR_APP_ID

# 2. Delete Personal Access Token
# GitHub → Settings → Developer settings → PATs → Delete

# 3. Rotate PostgreSQL password
az postgres flexible-server update \
  --resource-group hermesflow-dev-rg \
  --name hermesflow-dev-postgres \
  --admin-password "NEW_PASSWORD"

# 4. Regenerate Slack webhook
# Slack → Apps → Incoming Webhooks → Regenerate

# 5. Update GitHub Secrets
gh secret set AZURE_CLIENT_SECRET --body "NEW_VALUE"
```

---

## 🆘 Troubleshooting

### Error: "Resource 'Microsoft.ContainerRegistry/registries' was not found"

**Solution**: ACR hasn't been created yet. Run Terraform first:
```bash
cd infrastructure/terraform/environments/dev
terraform apply
```

### Error: "authentication failed"

**Solution**: Check Service Principal permissions:
```bash
az role assignment list --assignee YOUR_APP_ID --output table
```

### Error: "refusing to allow an OAuth App to create or update workflow"

**Solution**: PAT needs `workflow` scope. Regenerate with correct permissions.

### Workflow doesn't trigger

**Solution**: 
1. Check if secrets are set: `gh secret list`
2. Verify branch protection rules allow Actions
3. Check `.github/workflows/` files are in `main` branch

---

## 📚 Additional Resources

- [GitHub Encrypted Secrets](https://docs.github.com/en/actions/security-guides/encrypted-secrets)
- [Azure Service Principals](https://learn.microsoft.com/azure/active-directory/develop/app-objects-and-service-principals)
- [Slack Incoming Webhooks](https://api.slack.com/messaging/webhooks)
- [GitHub CLI Reference](https://cli.github.com/manual/gh_secret)

---

## 📞 Support

- **Slack**: `#hermesflow-devops`
- **Email**: devops@hermesflow.io
- **Oncall**: Check PagerDuty

---

**Last Updated**: 2025-01-13  
**Maintained By**: DevOps Team

