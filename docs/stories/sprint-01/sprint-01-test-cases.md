# Sprint 1 Test Cases - DevOps Foundation

**Sprint**: Sprint 1 (2025-01-10 ~ 2025-01-24)  
**Document Type**: Test Cases  
**Created By**: @qa.mdc  
**Created Date**: 2025-01-13  
**Last Updated**: 2025-01-13  
**Status**: Ready for Execution

---

## 📊 Test Cases Summary

| Category | P0 Cases | P1 Cases | P2 Cases | Total | Auto% |
|----------|----------|----------|----------|-------|-------|
| **Unit Tests (UT)** | 10 | 5 | 0 | 15 | 100% |
| **Integration Tests (IT)** | 15 | 10 | 0 | 25 | 80% |
| **Infrastructure Tests (INF)** | 20 | 10 | 0 | 30 | 70% |
| **Security Tests (SEC)** | 8 | 4 | 0 | 12 | 90% |
| **Performance Tests (PERF)** | 5 | 5 | 0 | 10 | 100% |
| **Disaster Recovery (DR)** | 4 | 4 | 0 | 8 | 60% |
| **Total** | **62** | **38** | **0** | **100** | **80%** |

---

## 🧪 I. Unit Tests (UT-001 ~ UT-015)

### UT-001: GitHub Actions Workflow Syntax Validation

**Priority**: P0  
**Type**: Automated  
**Story**: DEVOPS-001

**Objective**: Verify all GitHub Actions workflow files have valid YAML syntax.

**Pre-conditions**:
- actionlint tool installed
- Workflow files exist in `.github/workflows/`

**Test Steps**:
```bash
# Step 1: Install actionlint (if not installed)
go install github.com/rhysd/actionlint/cmd/actionlint@latest

# Step 2: Run actionlint on all workflow files
actionlint .github/workflows/*.yml

# Step 3: Check exit code
echo $?
```

**Expected Result**:
- ✅ Exit code = 0 (no errors)
- ✅ No syntax errors reported
- ✅ All workflow files validated

**Actual Result**: _[To be filled during execution]_  
**Status**: ⏳ Pending  
**Executed By**: _[Tester name]_  
**Executed Date**: _[Date]_

---

### UT-002: Terraform Formatting Check

**Priority**: P0  
**Type**: Automated  
**Story**: DEVOPS-002

**Objective**: Verify all Terraform files follow standard formatting.

**Pre-conditions**:
- Terraform CLI installed
- Terraform files exist in `infrastructure/terraform/`

**Test Steps**:
```bash
# Step 1: Navigate to terraform directory
cd infrastructure/terraform

# Step 2: Run format check
terraform fmt -check -recursive

# Step 3: Check exit code
echo $?
```

**Expected Result**:
- ✅ Exit code = 0
- ✅ No formatting changes needed
- ✅ Output: "No changes detected"

**Actual Result**: _[To be filled]_  
**Status**: ⏳ Pending

---

### UT-003: Terraform Module Validation

**Priority**: P0  
**Type**: Automated  
**Story**: DEVOPS-002

**Objective**: Validate Terraform syntax for all modules.

**Pre-conditions**:
- Terraform CLI installed (version >= 1.5)

**Test Steps**:
```bash
# For each module: networking, aks, acr, database, keyvault, monitoring
for module in networking aks acr database keyvault monitoring; do
  echo "Validating module: $module"
  cd infrastructure/terraform/modules/$module
  terraform init
  terraform validate
  cd -
done
```

**Expected Result**:
- ✅ All modules pass validation
- ✅ Output: "Success! The configuration is valid."

**Actual Result**: _[To be filled]_  
**Status**: ⏳ Pending

---

### UT-004: Docker Multi-stage Build Validation

**Priority**: P0  
**Type**: Manual + Automated  
**Story**: DEVOPS-001

**Objective**: Verify Dockerfiles use multi-stage builds correctly.

**Pre-conditions**:
- Docker installed
- Dockerfiles exist for each module

**Test Steps**:
```bash
# Step 1: Check Dockerfile structure
for dockerfile in modules/*/Dockerfile; do
  echo "Checking $dockerfile"
  grep -q "FROM.*as builder" $dockerfile && \
  grep -q "FROM.*slim\|alpine" $dockerfile || \
  echo "ERROR: $dockerfile doesn't use multi-stage build"
done

# Step 2: Verify build stages
docker build -t test-image modules/data-engine/

# Step 3: Check image size
docker images test-image --format "{{.Size}}"
```

**Expected Result**:
- ✅ All Dockerfiles use multi-stage builds
- ✅ Final image uses slim/alpine base
- ✅ Image size < 500MB (Rust), < 300MB (Java), < 200MB (Python)

**Actual Result**: _[To be filled]_  
**Status**: ⏳ Pending

---

### UT-005: Rust Cargo.toml Dependency Check

**Priority**: P0  
**Type**: Automated  
**Story**: DEVOPS-001

**Objective**: Verify Rust dependencies are pinned and secure.

**Pre-conditions**:
- Rust toolchain installed
- Cargo.toml files exist

**Test Steps**:
```bash
# Step 1: Check dependency versions
cd modules/data-engine
cat Cargo.toml | grep "^\[dependencies\]" -A 50

# Step 2: Run cargo check
cargo check

# Step 3: Run cargo audit for security vulnerabilities
cargo audit
```

**Expected Result**:
- ✅ All dependencies have version constraints
- ✅ `cargo check` succeeds
- ✅ No HIGH/CRITICAL security vulnerabilities

**Actual Result**: _[To be filled]_  
**Status**: ⏳ Pending

---

### UT-006: Java POM Dependency Check

**Priority**: P0  
**Type**: Automated  
**Story**: DEVOPS-001

**Test Steps**:
```bash
cd modules/user-management
mvn dependency:tree
mvn dependency:analyze
mvn validate
```

**Expected Result**:
- ✅ No dependency conflicts
- ✅ No unused dependencies
- ✅ POM validation successful

---

### UT-007: Python Requirements Check

**Priority**: P0  
**Type**: Automated  
**Story**: DEVOPS-001

**Test Steps**:
```bash
cd modules/strategy-engine
pip install -r requirements.txt --dry-run
safety check -r requirements.txt
```

**Expected Result**:
- ✅ All packages available
- ✅ No known security vulnerabilities

---

### UT-008: GitHub Actions Path Filter Configuration

**Priority**: P0  
**Type**: Automated  
**Story**: DEVOPS-001

**Test Steps**:
```bash
# Verify path filter syntax
yq eval '.jobs.detect-changes.steps[].with.filters' .github/workflows/ci-rust.yml
```

**Expected Result**:
- ✅ Filters match module directory structure
- ✅ YAML syntax valid

---

### UT-009: Docker Cache Key Generation Logic

**Priority**: P1  
**Type**: Automated  
**Story**: DEVOPS-001

**Test Steps**:
```bash
# Extract cache key from workflow
CACHE_KEY=$(yq eval '.jobs.build-rust.steps[] | select(.name == "Cache cargo registry").with.key' .github/workflows/ci-rust.yml)
echo "Cache key: $CACHE_KEY"

# Verify it includes Cargo.lock hash
echo $CACHE_KEY | grep -q "hashFiles.*Cargo.lock"
```

**Expected Result**:
- ✅ Cache key includes file hash
- ✅ Unique per module

---

### UT-010: Secrets Reference Validation

**Priority**: P0  
**Type**: Manual  
**Story**: DEVOPS-001

**Test Steps**:
1. Review all workflow files
2. List all `${{ secrets.* }}` references
3. Verify each secret is documented in README

**Expected Result**:
- ✅ All secrets documented
- ✅ No hardcoded secrets
- ✅ Secret names follow naming convention

---

### UT-011 ~ UT-015: Additional Unit Tests

- **UT-011**: Terraform Variable Type Validation (P1)
- **UT-012**: Terraform Output Validation (P1)
- **UT-013**: NSG Rule Configuration Check (P1)
- **UT-014**: RBAC Role Definition Validation (P1)
- **UT-015**: Monitoring Alert Threshold Check (P1)

_(Detailed steps available in test execution sheet)_

---

## 🔗 II. Integration Tests (IT-001 ~ IT-025)

### IT-001: End-to-End CI/CD Flow Test

**Priority**: P0  
**Type**: Manual + Automated  
**Story**: DEVOPS-001 + DEVOPS-002

**Objective**: Validate complete CI/CD pipeline from code push to GitOps update.

**Pre-conditions**:
- Azure infrastructure deployed
- GitHub Actions configured
- GitOps repository exists

**Test Steps**:
```bash
# Step 1: Create test branch
git checkout -b feature/e2e-test-$(date +%s)

# Step 2: Make trivial code change
echo "// Test comment" >> modules/data-engine/src/main.rs
git add .
git commit -m "[module:data-engine] test: e2e test"
git push origin feature/e2e-test-*

# Step 3: Monitor GitHub Actions
gh run list --branch feature/e2e-test-* --limit 1

# Step 4: Wait for completion
gh run watch $(gh run list --branch feature/e2e-test-* --limit 1 --json databaseId --jq '.[0].databaseId')

# Step 5: Verify image pushed to ACR
IMAGE_TAG=$(git rev-parse HEAD)
az acr repository show-tags \
  --name hermesflowdevacr \
  --repository data-engine \
  --output table | grep $IMAGE_TAG

# Step 6: Create and merge PR
gh pr create --title "Test E2E" --body "E2E test" --base main
PR_NUMBER=$(gh pr list --state open --limit 1 --json number --jq '.[0].number')
gh pr merge $PR_NUMBER --squash --delete-branch

# Step 7: Verify GitOps update
cd ../HermesFlow-GitOps
git pull
git log -1 --pretty=format:'%s' | grep "chore: update data-engine image to"
```

**Expected Result**:
- ✅ GitHub Actions workflow triggered automatically
- ✅ All tests passed
- ✅ Docker image built and pushed
- ✅ Trivy scan passed
- ✅ GitOps repository updated
- ✅ Total time < 20 minutes

**Actual Result**: _[To be filled]_  
**Status**: ⏳ Pending  
**Execution Time**: _[Duration]_

---

### IT-002: ACR Push and Pull Integration Test

**Priority**: P0  
**Type**: Automated  
**Story**: DEVOPS-001 + DEVOPS-002

**Objective**: Verify AKS can pull images from ACR using Managed Identity.

**Pre-conditions**:
- AKS cluster deployed
- ACR deployed with AcrPull role assigned

**Test Steps**:
```bash
# Step 1: Build and push test image
docker build -t hermesflow-dev-acr.azurecr.io/test:$(date +%s) \
  -f - . <<EOF
FROM alpine:latest
CMD ["echo", "Integration test successful"]
EOF

# Step 2: Login to ACR
az acr login --name hermesflowdevacr

# Step 3: Push image
IMAGE_TAG="test:$(date +%s)"
docker push hermesflow-dev-acr.azurecr.io/$IMAGE_TAG

# Step 4: Get AKS credentials
az aks get-credentials \
  --name hermesflow-dev-aks \
  --resource-group hermesflow-dev-rg \
  --overwrite-existing

# Step 5: Create Pod using ACR image
kubectl run acr-test \
  --image=hermesflow-dev-acr.azurecr.io/$IMAGE_TAG \
  --restart=Never

# Step 6: Wait for Pod completion
kubectl wait --for=condition=Completed pod/acr-test --timeout=120s

# Step 7: Check logs
kubectl logs acr-test

# Step 8: Cleanup
kubectl delete pod acr-test
```

**Expected Result**:
- ✅ Image pushed to ACR successfully
- ✅ AKS pulled image without authentication errors
- ✅ Pod ran successfully
- ✅ Log output: "Integration test successful"

**Actual Result**: _[To be filled]_  
**Status**: ⏳ Pending

---

### IT-003: GitOps Auto-Update Integration Test

**Priority**: P0  
**Type**: Automated  
**Story**: DEVOPS-001

**Objective**: Verify GitOps repository is updated automatically after main branch build.

**Pre-conditions**:
- GITOPS_PAT secret configured
- update-gitops.yml workflow exists

**Test Steps**:
```bash
# Step 1: Get current GitOps repo state
cd ../HermesFlow-GitOps
git pull
BEFORE_SHA=$(git rev-parse HEAD)

# Step 2: Trigger main branch build
cd ../HermesFlow
git checkout main
git pull
echo "// GitOps test" >> modules/data-engine/src/main.rs
git add .
git commit -m "[module:data-engine] test: gitops integration"
git push origin main

# Step 3: Wait for CI/CD completion
sleep 600  # Wait 10 minutes

# Step 4: Check GitOps repo update
cd ../HermesFlow-GitOps
git pull
AFTER_SHA=$(git rev-parse HEAD)

# Step 5: Verify new commit
[[ "$BEFORE_SHA" != "$AFTER_SHA" ]] || exit 1

# Step 6: Verify commit message format
COMMIT_MSG=$(git log -1 --pretty=format:'%s')
echo $COMMIT_MSG | grep -E "^chore: update data-engine image to [a-f0-9]{40}$"

# Step 7: Verify image tag in values.yaml
IMAGE_TAG=$(yq eval '.image.tag' apps/dev/data-engine/values.yaml)
echo "Image tag: $IMAGE_TAG"
```

**Expected Result**:
- ✅ GitOps repo has new commit
- ✅ Commit message matches expected format
- ✅ values.yaml updated with correct image tag
- ✅ Update completed < 15 minutes after CI success

**Actual Result**: _[To be filled]_  
**Status**: ⏳ Pending

---

### IT-004: Terraform Module Dependency Chain Test

**Priority**: P0  
**Type**: Automated  
**Story**: DEVOPS-002

**Objective**: Verify Terraform modules are applied in correct dependency order.

**Pre-conditions**:
- Terraform code complete
- Test environment available

**Test Steps**:
```bash
cd infrastructure/terraform/environments/dev

# Step 1: Initialize Terraform
terraform init

# Step 2: Apply only Networking module
terraform apply -target=module.networking -auto-approve

# Step 3: Verify VNet created
az network vnet show \
  --name hermesflow-dev-vnet \
  --resource-group hermesflow-dev-rg

# Step 4: Apply Monitoring module (depends on RG)
terraform apply -target=module.monitoring -auto-approve

# Step 5: Apply ACR module
terraform apply -target=module.acr -auto-approve

# Step 6: Apply AKS module (depends on Networking + ACR)
terraform apply -target=module.aks -auto-approve

# Step 7: Verify AKS can access ACR
az aks check-acr \
  --name hermesflow-dev-aks \
  --resource-group hermesflow-dev-rg \
  --acr hermesflowdevacr.azurecr.io

# Step 8: Apply Database module
terraform apply -target=module.database -auto-approve

# Step 9: Apply KeyVault module (depends on AKS)
terraform apply -target=module.keyvault -auto-approve

# Step 10: Verify complete state
terraform plan | grep "No changes"
```

**Expected Result**:
- ✅ Each module applied successfully in order
- ✅ No dependency errors
- ✅ Final `terraform plan` shows no changes
- ✅ All resources in "Succeeded" state

**Actual Result**: _[To be filled]_  
**Status**: ⏳ Pending

---

### IT-005 ~ IT-025: Additional Integration Tests

**CI/CD Integration Tests**:
- IT-005: Parallel Multi-Module Build Test (P0)
- IT-006: Build Cache Hit Rate Test (P1)
- IT-007: Trivy Scan Integration Test (P0)
- IT-008: Codecov Upload Test (P1)
- IT-009: Slack Notification Test (P1)
- IT-010: Workflow Failure Notification Test (P1)

**Infrastructure Integration Tests**:
- IT-011: AKS + ACR Integration (P0) - Covered in IT-002
- IT-012: AKS + Database Connection Test (P0)
- IT-013: AKS + KeyVault CSI Driver Test (P0)
- IT-014: Database + Private DNS Resolution (P0)
- IT-015: NSG + Subnet Association Test (P0)
- IT-016: Log Analytics + Container Insights (P1)
- IT-017: Alert Rule Trigger Test (P1)
- IT-018: Terraform State Lock Test (P0)
- IT-019: Cross-Module Variable Passing (P1)
- IT-020: RBAC Permission Chain Test (P0)
- IT-021: Service Principal + ACR Auth (P0)
- IT-022: Managed Identity + KeyVault (P0)
- IT-023: VNet Service Endpoint Test (P1)
- IT-024: Private Endpoint Connectivity (P0)
- IT-025: Multi-Resource Data Flow Test (P1)

_(Detailed test steps in separate execution sheet)_

---

## 🏗️ III. Infrastructure Tests (INF-001 ~ INF-030)

### INF-001: Resource Group Validation

**Priority**: P0  
**Type**: Automated  
**Story**: DEVOPS-002

**Objective**: Verify Resource Group exists with correct configuration.

**Test Script**:
```bash
#!/bin/bash
set -e

RG_NAME="hermesflow-dev-rg"
EXPECTED_LOCATION="eastus"

# Test 1: RG exists
echo "Test 1: Checking if Resource Group exists..."
az group exists --name $RG_NAME | grep -q "true" || exit 1
echo "✅ PASS"

# Test 2: Location correct
echo "Test 2: Verifying location..."
ACTUAL_LOCATION=$(az group show --name $RG_NAME --query location -o tsv)
[[ "$ACTUAL_LOCATION" == "$EXPECTED_LOCATION" ]] || { echo "❌ FAIL: Expected $EXPECTED_LOCATION, got $ACTUAL_LOCATION"; exit 1; }
echo "✅ PASS"

# Test 3: Tags present
echo "Test 3: Checking tags..."
az group show --name $RG_NAME --query 'tags.Environment' -o tsv | grep -q "Development" || exit 1
az group show --name $RG_NAME --query 'tags.Project' -o tsv | grep -q "HermesFlow" || exit 1
az group show --name $RG_NAME --query 'tags.ManagedBy' -o tsv | grep -q "Terraform" || exit 1
echo "✅ PASS"

echo "All tests passed for Resource Group validation!"
```

**Expected Result**: All 3 tests pass  
**Actual Result**: _[To be filled]_  
**Status**: ⏳ Pending

---

### INF-002: Virtual Network Configuration Test

**Priority**: P0  
**Type**: Automated  
**Story**: DEVOPS-002

**Test Script**:
```bash
#!/bin/bash
set -e

VNET_NAME="hermesflow-dev-vnet"
RG_NAME="hermesflow-dev-rg"

echo "Testing Virtual Network configuration..."

# Test 1: VNet exists
az network vnet show --name $VNET_NAME --resource-group $RG_NAME > /dev/null
echo "✅ VNet exists"

# Test 2: Address space correct
ADDRESS_SPACE=$(az network vnet show --name $VNET_NAME --resource-group $RG_NAME \
  --query 'addressSpace.addressPrefixes[0]' -o tsv)
[[ "$ADDRESS_SPACE" == "10.0.0.0/16" ]] || exit 1
echo "✅ Address space correct: $ADDRESS_SPACE"

# Test 3: Subnet count
SUBNETS=$(az network vnet subnet list --vnet-name $VNET_NAME --resource-group $RG_NAME \
  --query '[].name' -o tsv)
SUBNET_COUNT=$(echo "$SUBNETS" | wc -l)
[[ $SUBNET_COUNT -eq 3 ]] || exit 1
echo "✅ Subnet count correct: $SUBNET_COUNT"

# Test 4: AKS subnet configuration
AKS_SUBNET=$(az network vnet subnet show \
  --vnet-name $VNET_NAME \
  --resource-group $RG_NAME \
  --name aks-subnet \
  --query 'addressPrefix' -o tsv)
[[ "$AKS_SUBNET" == "10.0.1.0/24" ]] || exit 1
echo "✅ AKS subnet correct: $AKS_SUBNET"

# Test 5: Database subnet delegation
DB_DELEGATION=$(az network vnet subnet show \
  --vnet-name $VNET_NAME \
  --resource-group $RG_NAME \
  --name database-subnet \
  --query 'delegations[0].serviceName' -o tsv)
[[ "$DB_DELEGATION" == "Microsoft.DBforPostgreSQL/flexibleServers" ]] || exit 1
echo "✅ Database subnet delegation correct"

echo "All VNet tests passed!"
```

**Expected Result**: All 5 tests pass  
**Actual Result**: _[To be filled]_  
**Status**: ⏳ Pending

---

### INF-003: AKS Cluster Comprehensive Test

**Priority**: P0  
**Type**: Automated  
**Story**: DEVOPS-002

**Test Script**:
```bash
#!/bin/bash
set -e

AKS_NAME="hermesflow-dev-aks"
RG_NAME="hermesflow-dev-rg"

echo "Testing AKS cluster configuration..."

# Test 1: Cluster exists and is running
PROVISIONING_STATE=$(az aks show --name $AKS_NAME --resource-group $RG_NAME \
  --query 'provisioningState' -o tsv)
[[ "$PROVISIONING_STATE" == "Succeeded" ]] || exit 1
echo "✅ Cluster provisioned successfully"

# Test 2: Kubernetes version
K8S_VERSION=$(az aks show --name $AKS_NAME --resource-group $RG_NAME \
  --query 'kubernetesVersion' -o tsv)
[[ "$K8S_VERSION" =~ ^1\.28 ]] || exit 1
echo "✅ K8s version: $K8S_VERSION"

# Test 3: Node pools
NODE_POOLS=$(az aks nodepool list --cluster-name $AKS_NAME --resource-group $RG_NAME \
  --query '[].name' -o tsv)
echo "$NODE_POOLS" | grep -q "system" || exit 1
echo "$NODE_POOLS" | grep -q "user" || exit 1
echo "✅ Both node pools present"

# Test 4: System node pool configuration
SYSTEM_COUNT=$(az aks nodepool show \
  --cluster-name $AKS_NAME \
  --resource-group $RG_NAME \
  --name system \
  --query 'count' -o tsv)
[[ $SYSTEM_COUNT -eq 2 ]] || exit 1
echo "✅ System node pool: $SYSTEM_COUNT nodes"

# Test 5: Network plugin
NETWORK_PLUGIN=$(az aks show --name $AKS_NAME --resource-group $RG_NAME \
  --query 'networkProfile.networkPlugin' -o tsv)
[[ "$NETWORK_PLUGIN" == "azure" ]] || exit 1
echo "✅ Network plugin: $NETWORK_PLUGIN"

# Test 6: RBAC enabled
RBAC_ENABLED=$(az aks show --name $AKS_NAME --resource-group $RG_NAME \
  --query 'enableRbac' -o tsv)
[[ "$RBAC_ENABLED" == "true" ]] || exit 1
echo "✅ RBAC enabled"

# Test 7: Get credentials and check connectivity
az aks get-credentials --name $AKS_NAME --resource-group $RG_NAME --overwrite-existing
kubectl cluster-info
kubectl get nodes
echo "✅ kubectl connectivity successful"

echo "All AKS tests passed!"
```

**Expected Result**: All 7 tests pass  
**Actual Result**: _[To be filled]_  
**Status**: ⏳ Pending

---

### INF-004 ~ INF-030: Additional Infrastructure Tests

**Core Resource Tests** (P0):
- INF-004: ACR Configuration Test
- INF-005: PostgreSQL Server Test
- INF-006: Key Vault Configuration Test
- INF-007: Log Analytics Workspace Test

**Network Connectivity Tests** (P0):
- INF-008: AKS to Internet Connectivity
- INF-009: AKS to ACR Connectivity
- INF-010: AKS to PostgreSQL Connectivity
- INF-011: NSG Rule Effectiveness Test
- INF-012: Subnet CIDR No-Conflict Test
- INF-013: Private DNS Resolution Test
- INF-014: Service Endpoint Test
- INF-015: Load Balancer Health Check

**Security Tests** (P0):
- INF-016: RBAC Role Assignment Verification
- INF-017: Managed Identity Test
- INF-018: Key Vault Access Policy Test
- INF-019: Pod Security Policy Test
- INF-020: Network Policy Test

**Monitoring Tests** (P1):
- INF-021: Log Analytics Data Ingestion
- INF-022: Container Insights Metrics
- INF-023: Alert Rule Trigger Test
- INF-024: Action Group Notification
- INF-025: Saved Query Execution

**Configuration Tests** (P1):
- INF-026: Terraform State Consistency
- INF-027: Resource Tag Consistency
- INF-028: Cost Tag Validation
- INF-029: Backup Configuration Check
- INF-030: High Availability Config

_(Detailed scripts available in automation repository)_

---

## 🔒 IV. Security Tests (SEC-001 ~ SEC-012)

### SEC-001: Trivy Container Image Scan

**Priority**: P0  
**Type**: Automated  
**Story**: DEVOPS-001

**Objective**: Verify Docker images have no CRITICAL vulnerabilities.

**Test Steps**:
```bash
#!/bin/bash

IMAGES=(
  "hermesflow-dev-acr.azurecr.io/data-engine:latest"
  "hermesflow-dev-acr.azurecr.io/user-management:latest"
  "hermesflow-dev-acr.azurecr.io/strategy-engine:latest"
)

for IMAGE in "${IMAGES[@]}"; do
  echo "Scanning $IMAGE..."
  
  # Scan for HIGH and CRITICAL vulnerabilities
  trivy image --severity HIGH,CRITICAL --exit-code 1 $IMAGE
  
  if [ $? -eq 0 ]; then
    echo "✅ PASS: No HIGH/CRITICAL vulnerabilities in $IMAGE"
  else
    echo "❌ FAIL: Vulnerabilities found in $IMAGE"
    # Generate detailed report
    trivy image --severity HIGH,CRITICAL --format json -o $(basename $IMAGE)-report.json $IMAGE
    exit 1
  fi
done

echo "All images passed security scan!"
```

**Expected Result**:
- ✅ No CRITICAL vulnerabilities
- ✅ HIGH vulnerabilities < 5
- ✅ Scan completes < 2 minutes per image

**Actual Result**: _[To be filled]_  
**Status**: ⏳ Pending

---

### SEC-002: tfsec Terraform Security Scan

**Priority**: P0  
**Type**: Automated  
**Story**: DEVOPS-002

**Test Steps**:
```bash
#!/bin/bash
cd infrastructure/terraform

# Run tfsec scan
tfsec . --format json -o tfsec-results.json

# Check for HIGH/CRITICAL issues
HIGH_COUNT=$(cat tfsec-results.json | jq '.results[] | select(.severity=="HIGH")' | jq -s 'length')
CRITICAL_COUNT=$(cat tfsec-results.json | jq '.results[] | select(.severity=="CRITICAL")' | jq -s 'length')

echo "HIGH issues: $HIGH_COUNT"
echo "CRITICAL issues: $CRITICAL_COUNT"

if [ $CRITICAL_COUNT -gt 0 ]; then
  echo "❌ FAIL: CRITICAL issues found"
  cat tfsec-results.json | jq '.results[] | select(.severity=="CRITICAL")'
  exit 1
fi

if [ $HIGH_COUNT -gt 0 ]; then
  echo "⚠️  WARNING: HIGH issues found (review required)"
  cat tfsec-results.json | jq '.results[] | select(.severity=="HIGH")'
fi

echo "✅ PASS: No CRITICAL issues"
```

**Expected Result**: CRITICAL = 0, HIGH < 3  
**Actual Result**: _[To be filled]_  
**Status**: ⏳ Pending

---

### SEC-003 ~ SEC-012: Additional Security Tests

- **SEC-003**: Secrets Leak Detection (P0)
- **SEC-004**: RBAC Least Privilege Test (P0)
- **SEC-005**: NSG Rule Audit (P0)
- **SEC-006**: Key Vault Network ACL Test (P0)
- **SEC-007**: PostgreSQL SSL Enforcement (P0)
- **SEC-008**: ACR Admin Disabled Test (P0)
- **SEC-009**: Terraform State Encryption (P1)
- **SEC-010**: GitHub Secrets Audit Log (P1)
- **SEC-011**: Pod Security Standards (P1)
- **SEC-012**: CVE Scanning Integration (P1)

---

## ⚡ V. Performance Tests (PERF-001 ~ PERF-010)

### PERF-001: CI Build Time Baseline

**Priority**: P0  
**Type**: Automated  
**Story**: DEVOPS-001

**Objective**: Measure and validate CI build times.

**Test Steps**:
```bash
#!/bin/bash

# Trigger CI build and measure time
BUILD_START=$(date +%s)

# Trigger workflow
gh workflow run ci-rust.yml --ref main

# Wait for completion
RUN_ID=$(gh run list --workflow=ci-rust.yml --limit 1 --json databaseId --jq '.[0].databaseId')
gh run watch $RUN_ID

BUILD_END=$(date +%s)
DURATION=$((BUILD_END - BUILD_START))

echo "Build duration: ${DURATION}s ($(($DURATION / 60))m)"

# Validate against targets
if [ $DURATION -lt 900 ]; then
  echo "✅ PASS: Build time ${DURATION}s < 900s (15min)"
else
  echo "❌ FAIL: Build time ${DURATION}s >= 900s"
  exit 1
fi
```

**Expected Result**:
- Rust: < 900s (15min)
- Java: < 600s (10min)
- Python: < 300s (5min)

**Actual Result**: _[To be filled]_  
**Status**: ⏳ Pending

---

### PERF-002 ~ PERF-010: Additional Performance Tests

- **PERF-002**: Cache Efficiency Test (P0)
- **PERF-003**: Terraform Apply Time (P0)
- **PERF-004**: AKS Node Startup Time (P0)
- **PERF-005**: Docker Image Build Time (P0)
- **PERF-006**: ACR Push Speed Test (P1)
- **PERF-007**: Terraform Plan Time (P1)
- **PERF-008**: kubectl API Response Time (P1)
- **PERF-009**: Log Analytics Query Performance (P1)
- **PERF-010**: Parallel Build Efficiency (P1)

---

## 🔥 VI. Disaster Recovery Tests (DR-001 ~ DR-008)

### DR-001: Terraform State Recovery

**Priority**: P0  
**Type**: Manual  
**Story**: DEVOPS-002

**Objective**: Verify Terraform state can be recovered from backup.

**Test Steps**:
```bash
# Step 1: Backup current state
terraform state pull > backup-$(date +%Y%m%d).tfstate
echo "✅ State backed up"

# Step 2: Verify backup integrity
cat backup-*.tfstate | jq '.version'

# Step 3: Simulate state corruption (in test environment only!)
# [DANGEROUS - Only in test env]
az storage blob delete \
  --account-name hermesflowdevtfstate \
  --container-name tfstate \
  --name dev.terraform.tfstate

# Step 4: Attempt to run terraform plan (should fail)
terraform plan 2>&1 | grep -q "Error" && echo "✅ State corruption detected"

# Step 5: Restore from backup
terraform state push backup-*.tfstate
echo "✅ State restored"

# Step 6: Verify state recovered
terraform plan | grep -q "No changes" && echo "✅ State recovery successful"
```

**Expected Result**:
- ✅ Backup created successfully
- ✅ Corruption detected
- ✅ State restored
- ✅ No unexpected changes after restoration

**Actual Result**: _[To be filled]_  
**Status**: ⏳ Pending

---

### DR-002 ~ DR-008: Additional DR Tests

- **DR-002**: Full Infrastructure Destroy/Recreate (P0)
- **DR-003**: PostgreSQL Backup/Restore (P0)
- **DR-004**: GitHub Workflow Recovery (P0)
- **DR-005**: AKS Node Failure Recovery (P1)
- **DR-006**: ACR Image Backup Verification (P1)
- **DR-007**: Key Vault Soft Delete Recovery (P1)
- **DR-008**: Log Analytics Data Retention (P1)

---

## 📋 VII. Test Execution Tracking

### Execution Schedule

| Date | Test Type | Cases | Tester | Status |
|------|-----------|-------|--------|--------|
| 2025-01-11 | UT-001~015 | 15 | DevOps Team | ⏳ Scheduled |
| 2025-01-12 | INF-001~015 | 15 | QA | ⏳ Scheduled |
| 2025-01-13 | INF-016~030 | 15 | QA | ⏳ Scheduled |
| 2025-01-14 | SEC-001~012 | 12 | Security | ⏳ Scheduled |
| 2025-01-15 | PERF-001~010 | 10 | QA | ⏳ Scheduled |
| 2025-01-18 | IT-001~015 | 15 | QA | ⏳ Scheduled |
| 2025-01-19 | IT-016~025 | 10 | QA | ⏳ Scheduled |
| 2025-01-20 | DR-001~008 | 8 | DevOps | ⏳ Scheduled |
| 2025-01-21 | Regression | All P0 | QA | ⏳ Scheduled |
| 2025-01-22 | Acceptance | Selected | PO | ⏳ Scheduled |

### Daily Test Report Template

```markdown
# Test Execution Report - [Date]

## Summary
- Planned: XX cases
- Executed: XX cases
- Passed: XX cases
- Failed: XX cases
- Blocked: XX cases

## Pass Rate
- Overall: XX%
- P0: XX%
- P1: XX%

## Failed Cases
- [TC-ID]: [Reason]

## Blocked Cases
- [TC-ID]: [Blocker]

## New Defects
- [DEF-ID]: [Description]

## Notes
- [Important observations]
```

---

## 🐛 VIII. Defect Management

### Defect Reporting Template

```markdown
# Defect Report

**Defect ID**: DEF-XXX  
**Priority**: P0/P1/P2  
**Severity**: Critical/High/Medium/Low  
**Status**: Open/In Progress/Fixed/Closed

**Summary**: [One-line description]

**Test Case**: TC-XXX

**Steps to Reproduce**:
1. Step 1
2. Step 2
3. Step 3

**Expected Result**:
- [Expected behavior]

**Actual Result**:
- [Actual behavior]

**Environment**:
- Azure subscription: [ID]
- Terraform version: [version]
- GitHub Actions runner: [ubuntu-latest]

**Logs/Screenshots**:
[Attach relevant evidence]

**Suggested Fix**:
[If known]

**Reported By**: [Name]  
**Reported Date**: [Date]
```

### Defect Priority Definitions

| Priority | Response Time | Resolution Time | Description |
|----------|---------------|-----------------|-------------|
| **P0** | 1 hour | Sprint内 | 阻塞Sprint目标 |
| **P1** | 4 hours | Sprint内 | 影响主要功能 |
| **P2** | 24 hours | Next Sprint | 次要功能问题 |

---

## ✅ IX. Test Completion Criteria

### Sprint 1 Test Exit Criteria

- [ ] 100% P0 test cases executed
- [ ] 100% P0 test cases passed
- [ ] ≥95% P1 test cases passed
- [ ] All P0 defects fixed and verified
- [ ] ≥80% P1 defects fixed
- [ ] No open CRITICAL defects
- [ ] Test automation ≥80%
- [ ] Test documentation complete
- [ ] Sprint acceptance tests passed

### Quality Gates

**Gate 1: Unit Testing Complete** (Day 2)
- All UT cases pass
- Code merged to main

**Gate 2: Infrastructure Deployed** (Day 5)
- All INF P0 cases pass
- Azure resources operational

**Gate 3: Integration Verified** (Day 9)
- All IT P0 cases pass
- End-to-end flow works

**Gate 4: Sprint Ready for Review** (Day 13)
- All P0 + 95% P1 pass
- Demo prepared

---

## 📚 X. References

- [Sprint 1 Summary](./sprint-01-summary.md)
- [Sprint 1 Risk Profile](./sprint-01-risk-profile.md)
- [Sprint 1 Test Strategy](./sprint-01-test-strategy.md)
- [DEVOPS-001 Story](./DEVOPS-001-github-actions-cicd.md)
- [DEVOPS-002 Story](./DEVOPS-002-azure-terraform-iac.md)

---

**Document Version**: 1.0  
**Test Execution Start**: 2025-01-11  
**Test Execution End**: 2025-01-22  
**Approved By**: @qa.mdc

**Testing Mantra**: _"Test what you build, build what you test."_

