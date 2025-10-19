# Sprint 1 Test Strategy - DevOps Foundation

**Sprint**: Sprint 1 (2025-01-10 ~ 2025-01-24)  
**Document Type**: Test Strategy  
**Created By**: @qa.mdc  
**Created Date**: 2025-01-13  
**Last Updated**: 2025-01-13  
**Status**: Active

---

## 📊 Executive Summary

本文档定义Sprint 1 (DevOps Foundation)的全面测试策略，覆盖CI/CD流水线和Azure基础设施的6大测试类型，包含~100个详细测试用例。

### 测试概览

| 测试类型 | 用例数量 | 优先级分布 | 自动化率 |
|---------|---------|-----------|---------|
| **单元测试** | 15 | P0: 10, P1: 5 | 100% |
| **集成测试** | 25 | P0: 15, P1: 10 | 80% |
| **基础设施测试** | 30 | P0: 20, P1: 10 | 70% |
| **安全测试** | 12 | P0: 8, P1: 4 | 90% |
| **性能测试** | 10 | P0: 5, P1: 5 | 100% |
| **灾难恢复测试** | 8 | P0: 4, P1: 4 | 60% |
| **总计** | **100** | **P0: 62, P1: 38** | **80%** |

### 质量目标

| 指标 | 目标值 | 实际值 | 状态 |
|------|--------|--------|------|
| **P0用例通过率** | 100% | _待执行_ | ⏳ |
| **P1用例通过率** | ≥95% | _待执行_ | ⏳ |
| **自动化覆盖率** | ≥80% | 80% | ✅ |
| **缺陷逃逸率** | <5% | _待评估_ | ⏳ |
| **测试执行时间** | <4h | _待测量_ | ⏳ |

---

## 🎯 I. 测试范围与目标

### 1.1 测试范围

#### 包含范围 (In Scope)

**DEVOPS-001: GitHub Actions CI/CD Pipeline**
- ✅ 多语言工作流(Rust/Java/Python/React)
- ✅ 路径检测逻辑
- ✅ 构建缓存机制
- ✅ Docker镜像构建和推送
- ✅ 安全扫描(Trivy)
- ✅ GitOps自动更新
- ✅ 通知机制

**DEVOPS-002: Azure Infrastructure as Code**
- ✅ Terraform模块(Networking, AKS, ACR, Database, KeyVault, Monitoring)
- ✅ 资源创建和配置
- ✅ 网络连通性
- ✅ RBAC和权限
- ✅ 监控和告警
- ✅ State管理

**Sprint整体**
- ✅ 端到端CI/CD流程
- ✅ 基础设施和应用集成
- ✅ 文档完整性

#### 排除范围 (Out of Scope)

- ❌ 应用业务逻辑(数据采集、策略引擎等 - 将在后续Sprint测试)
- ❌ 生产环境部署(仅测试dev环境)
- ❌ 性能压力测试(仅基准测试)
- ❌ 用户验收测试(UAT - 将在Sprint Review进行)

### 1.2 测试目标

**主要目标**:
1. ✅ 验证CI/CD流水线能成功构建和部署所有模块
2. ✅ 验证Azure基础设施按规格正确创建
3. ✅ 验证网络连通性和安全配置
4. ✅ 识别并修复所有P0和≥95% P1缺陷

**次要目标**:
1. ✅ 建立自动化测试框架
2. ✅ 收集性能基准数据
3. ✅ 验证灾难恢复流程
4. ✅ 创建测试文档和知识库

---

## 🧪 II. 测试类型详解

### 2.1 单元测试 (Unit Tests)

**目标**: 验证GitHub Actions工作流和Terraform模块的独立功能。

**测试数量**: 15个用例 (P0: 10, P1: 5)

#### 关键测试场景

**UT-001: GitHub Actions工作流语法验证**
```bash
# 使用actionlint验证工作流语法
actionlint .github/workflows/*.yml
```

**预期结果**: 所有工作流文件无语法错误

---

**UT-002: Terraform模块语法验证**
```bash
cd infrastructure/terraform/modules/networking
terraform fmt -check -recursive
terraform validate
```

**预期结果**: 所有Terraform文件格式正确且有效

---

**UT-003: Terraform模块单元测试 (Terratest)**
```go
func TestNetworkingModule(t *testing.T) {
    terraformOptions := &terraform.Options{
        TerraformDir: "../modules/networking",
        Vars: map[string]interface{}{
            "resource_group_name": "test-rg",
            "location": "East US",
            "prefix": "test",
        },
    }
    
    defer terraform.Destroy(t, terraformOptions)
    terraform.InitAndPlan(t, terraformOptions)
}
```

---

**UT-004-UT-015**: 其他单元测试用例
- UT-004: Docker多阶段构建验证
- UT-005: Rust Cargo.toml依赖检查
- UT-006: Java POM依赖检查
- UT-007: Python requirements.txt检查
- UT-008: 路径检测filter配置测试
- UT-009: Cache key生成逻辑测试
- UT-010: Secret引用检查
- UT-011: Terraform变量类型验证
- UT-012: Terraform输出变量测试
- UT-013: NSG规则配置验证
- UT-014: RBAC角色定义验证
- UT-015: 监控告警阈值合理性检查

---

### 2.2 集成测试 (Integration Tests)

**目标**: 验证CI/CD流程和基础设施组件间的交互。

**测试数量**: 25个用例 (P0: 15, P1: 10)

#### 关键测试场景

**IT-001: 端到端CI/CD流程测试**

**测试步骤**:
1. 创建feature分支: `feature/test-cicd`
2. 修改Rust模块代码(添加注释)
3. 推送到GitHub: `git push origin feature/test-cicd`
4. 观察GitHub Actions执行
5. 验证Docker镜像推送到ACR
6. 合并PR到main分支
7. 验证GitOps仓库自动更新

**预期结果**:
- ✅ CI工作流成功执行
- ✅ 所有测试通过
- ✅ 镜像标签: `hermesflow-dev-acr.azurecr.io/data-engine:${SHA}`
- ✅ GitOps仓库commit: `chore: update data-engine image to ${SHA}`

**执行时间**: 预计15分钟

---

**IT-002: ACR推送和拉取测试**

**测试步骤**:
```bash
# 1. 手动构建测试镜像
docker build -t hermesflow-dev-acr.azurecr.io/test:latest .

# 2. 使用SP登录ACR
az acr login --name hermesflowdevacr \
  --username $ACR_USERNAME \
  --password $ACR_PASSWORD

# 3. 推送镜像
docker push hermesflow-dev-acr.azurecr.io/test:latest

# 4. 从AKS拉取镜像
kubectl run test-pod \
  --image=hermesflow-dev-acr.azurecr.io/test:latest \
  --restart=Never

# 5. 验证Pod状态
kubectl get pod test-pod -o jsonpath='{.status.phase}'
```

**预期结果**: Pod状态为 `Running` 或 `Completed`

---

**IT-003: GitOps自动更新集成测试**

**测试步骤**:
1. 触发main分支CI构建
2. 等待构建成功
3. 检查GitOps仓库是否有新commit
4. 验证values.yaml中的image.tag更新

**验证脚本**:
```bash
# 获取最新commit
cd HermesFlow-GitOps
git pull
LATEST_COMMIT=$(git log -1 --pretty=format:'%s')

# 验证commit message格式
echo $LATEST_COMMIT | grep -E "^chore: update .* image to [a-f0-9]{40}$"
```

---

**IT-004: Terraform模块间集成测试**

**测试场景**: 验证Networking → AKS → ACR → Database依赖链

```bash
# 1. 创建测试环境
cd infrastructure/terraform/environments/dev
terraform init

# 2. 仅应用Networking模块
terraform apply -target=module.networking -auto-approve

# 3. 验证VNet和Subnet创建
az network vnet show --name hermesflow-dev-vnet --resource-group hermesflow-dev-rg

# 4. 应用AKS模块(依赖Networking)
terraform apply -target=module.aks -auto-approve

# 5. 验证AKS使用正确的Subnet
az aks show --name hermesflow-dev-aks --resource-group hermesflow-dev-rg \
  --query "agentPoolProfiles[0].vnetSubnetId"
```

---

**IT-005-IT-025**: 其他集成测试用例

**CI/CD集成** (6个):
- IT-005: 多模块并行构建测试
- IT-006: 构建缓存命中测试
- IT-007: Trivy扫描集成测试
- IT-008: Codecov上传测试
- IT-009: Slack通知测试
- IT-010: 工作流失败通知测试

**基础设施集成** (14个):
- IT-011: AKS + ACR集成测试
- IT-012: AKS + Database连接测试
- IT-013: AKS + KeyVault集成测试
- IT-014: Database + Private DNS测试
- IT-015: NSG + Subnet关联测试
- IT-016: Log Analytics + AKS集成测试
- IT-017: Monitoring + Alert测试
- IT-018: Terraform Backend + State Lock测试
- IT-019: Terraform Module依赖测试
- IT-020: RBAC权限链测试
- IT-021: Service Principal + ACR测试
- IT-022: Managed Identity + KeyVault测试
- IT-023: VNet Peering测试(如有)
- IT-024: Private Endpoint连接测试
- IT-025: 跨模块数据流测试

---

### 2.3 基础设施测试 (Infrastructure Tests)

**目标**: 验证Azure资源按照规格正确创建和配置。

**测试数量**: 30个用例 (P0: 20, P1: 10)

#### 关键测试场景

**INF-001: Resource Group验证**

**测试脚本**:
```bash
#!/bin/bash
# 验证Resource Group存在且配置正确

RG_NAME="hermesflow-dev-rg"
EXPECTED_LOCATION="eastus"

# 检查RG存在
az group exists --name $RG_NAME

# 验证Location
ACTUAL_LOCATION=$(az group show --name $RG_NAME --query location -o tsv)
[[ "$ACTUAL_LOCATION" == "$EXPECTED_LOCATION" ]] || exit 1

# 验证标签
az group show --name $RG_NAME --query 'tags.Environment' -o tsv | grep -q "Development"
az group show --name $RG_NAME --query 'tags.Project' -o tsv | grep -q "HermesFlow"
az group show --name $RG_NAME --query 'tags.ManagedBy' -o tsv | grep -q "Terraform"
```

**预期结果**: 所有检查通过

---

**INF-002: Virtual Network验证**

```bash
VNET_NAME="hermesflow-dev-vnet"
RG_NAME="hermesflow-dev-rg"

# 验证VNet存在
az network vnet show --name $VNET_NAME --resource-group $RG_NAME

# 验证地址空间
ADDRESS_SPACE=$(az network vnet show --name $VNET_NAME --resource-group $RG_NAME \
  --query 'addressSpace.addressPrefixes[0]' -o tsv)
[[ "$ADDRESS_SPACE" == "10.0.0.0/16" ]] || exit 1

# 验证子网数量
SUBNET_COUNT=$(az network vnet subnet list --vnet-name $VNET_NAME --resource-group $RG_NAME \
  --query 'length(@)' -o tsv)
[[ "$SUBNET_COUNT" -eq 3 ]] || exit 1
```

---

**INF-003: AKS集群验证**

```bash
AKS_NAME="hermesflow-dev-aks"
RG_NAME="hermesflow-dev-rg"

# 验证AKS存在且状态正常
PROVISIONING_STATE=$(az aks show --name $AKS_NAME --resource-group $RG_NAME \
  --query 'provisioningState' -o tsv)
[[ "$PROVISIONING_STATE" == "Succeeded" ]] || exit 1

# 验证K8s版本
K8S_VERSION=$(az aks show --name $AKS_NAME --resource-group $RG_NAME \
  --query 'kubernetesVersion' -o tsv)
[[ "$K8S_VERSION" == "1.28"* ]] || exit 1

# 验证节点池数量
NODE_POOL_COUNT=$(az aks nodepool list --cluster-name $AKS_NAME --resource-group $RG_NAME \
  --query 'length(@)' -o tsv)
[[ "$NODE_POOL_COUNT" -eq 2 ]] || exit 1

# 验证System节点池
az aks nodepool show --cluster-name $AKS_NAME --resource-group $RG_NAME --name system \
  --query 'count' -o tsv | grep -q "2"

# 验证网络插件
az aks show --name $AKS_NAME --resource-group $RG_NAME \
  --query 'networkProfile.networkPlugin' -o tsv | grep -q "azure"
```

---

**INF-004: ACR验证**

```bash
ACR_NAME="hermesflowdevacr"
RG_NAME="hermesflow-dev-rg"

# 验证ACR存在
az acr show --name $ACR_NAME --resource-group $RG_NAME

# 验证SKU
az acr show --name $ACR_NAME --resource-group $RG_NAME \
  --query 'sku.name' -o tsv | grep -q "Standard"

# 验证admin disabled
az acr show --name $ACR_NAME --resource-group $RG_NAME \
  --query 'adminUserEnabled' -o tsv | grep -q "false"

# 测试登录
az acr login --name $ACR_NAME
```

---

**INF-005: PostgreSQL验证**

```bash
PG_SERVER="hermesflow-dev-postgres"
RG_NAME="hermesflow-dev-rg"

# 验证服务器存在
az postgres flexible-server show --name $PG_SERVER --resource-group $RG_NAME

# 验证版本
az postgres flexible-server show --name $PG_SERVER --resource-group $RG_NAME \
  --query 'version' -o tsv | grep -q "15"

# 验证SKU
az postgres flexible-server show --name $PG_SERVER --resource-group $RG_NAME \
  --query 'sku.name' -o tsv | grep -q "B_Standard_B1ms"

# 验证数据库存在
az postgres flexible-server db show --server-name $PG_SERVER --resource-group $RG_NAME \
  --database-name hermesflow
```

---

**INF-006: Key Vault验证**

```bash
KV_NAME="hermesflow-dev-kv"
RG_NAME="hermesflow-dev-rg"

# 验证Key Vault存在
az keyvault show --name $KV_NAME --resource-group $RG_NAME

# 验证Secrets存在
az keyvault secret list --vault-name $KV_NAME --query 'length(@)' -o tsv

# 验证特定Secret
az keyvault secret show --vault-name $KV_NAME --name postgres-admin-password
az keyvault secret show --vault-name $KV_NAME --name redis-password

# 验证网络规则
az keyvault show --name $KV_NAME --resource-group $RG_NAME \
  --query 'properties.networkAcls.defaultAction' -o tsv
```

---

**INF-007: Log Analytics验证**

```bash
WORKSPACE_NAME="hermesflow-dev-logs"
RG_NAME="hermesflow-dev-rg"

# 验证Workspace存在
az monitor log-analytics workspace show --workspace-name $WORKSPACE_NAME --resource-group $RG_NAME

# 验证Retention
az monitor log-analytics workspace show --workspace-name $WORKSPACE_NAME --resource-group $RG_NAME \
  --query 'retentionInDays' -o tsv | grep -q "30"

# 验证Container Insights已安装
az monitor log-analytics solution list --resource-group $RG_NAME \
  --query "[?name=='ContainerInsights($WORKSPACE_NAME)']"
```

---

**INF-008: 网络连通性测试 - AKS到Internet**

```bash
# 获取AKS凭证
az aks get-credentials --name hermesflow-dev-aks --resource-group hermesflow-dev-rg --overwrite

# 创建测试Pod
kubectl run network-test --image=busybox --restart=Never -- sleep 3600

# 等待Pod启动
kubectl wait --for=condition=Ready pod/network-test --timeout=60s

# 测试Internet连接
kubectl exec network-test -- nslookup google.com
kubectl exec network-test -- wget -O- https://www.google.com

# 清理
kubectl delete pod network-test
```

---

**INF-009: 网络连通性测试 - AKS到ACR**

```bash
# 从AKS拉取ACR镜像
kubectl run acr-test --image=hermesflow-dev-acr.azurecr.io/test:latest --restart=Never

# 检查Pod状态
kubectl get pod acr-test

# 检查Events
kubectl describe pod acr-test | grep -A 5 "Events:"

# 清理
kubectl delete pod acr-test
```

---

**INF-010: 网络连通性测试 - AKS到PostgreSQL**

```bash
# 创建PostgreSQL客户端Pod
kubectl run pg-test --image=postgres:15 --restart=Never -- sleep 3600

# 等待Pod启动
kubectl wait --for=condition=Ready pod/pg-test --timeout=60s

# 测试连接
PG_HOST="hermesflow-dev-postgres.postgres.database.azure.com"
kubectl exec pg-test -- psql \
  -h $PG_HOST \
  -U hermesadmin \
  -d hermesflow \
  -c "SELECT version();"

# 清理
kubectl delete pod pg-test
```

---

**INF-011-INF-030**: 其他基础设施测试用例

**网络测试** (5个):
- INF-011: NSG规则有效性测试
- INF-012: Subnet CIDR无冲突测试
- INF-013: Private DNS解析测试
- INF-014: Service Endpoint测试
- INF-015: Load Balancer健康检查测试

**安全测试** (5个):
- INF-016: RBAC角色分配验证
- INF-017: Managed Identity验证
- INF-018: Key Vault访问策略测试
- INF-019: Pod Security Policy测试
- INF-020: Network Policy测试

**监控测试** (5个):
- INF-021: Log Analytics数据流入测试
- INF-022: Container Insights指标测试
- INF-023: Alert规则触发测试
- INF-024: Action Group通知测试
- INF-025: Saved Query执行测试

**配置测试** (5个):
- INF-026: Terraform State一致性检查
- INF-027: 资源标签一致性检查
- INF-028: 成本标签验证
- INF-029: 备份配置检查
- INF-030: 高可用性配置验证

---

### 2.4 安全测试 (Security Tests)

**目标**: 验证安全配置和漏洞扫描机制。

**测试数量**: 12个用例 (P0: 8, P1: 4)

#### 关键测试场景

**SEC-001: Trivy镜像扫描测试**

```bash
# 扫描已构建的镜像
trivy image hermesflow-dev-acr.azurecr.io/data-engine:latest \
  --severity HIGH,CRITICAL \
  --exit-code 1

# 验证扫描结果格式
trivy image hermesflow-dev-acr.azurecr.io/data-engine:latest \
  --format json -o trivy-results.json

# 检查无CRITICAL漏洞
cat trivy-results.json | jq '.Results[].Vulnerabilities[] | select(.Severity=="CRITICAL")' | wc -l
```

**预期结果**: 无CRITICAL漏洞,返回码0

---

**SEC-002: tfsec Terraform安全扫描**

```bash
# 扫描Terraform代码
cd infrastructure/terraform
tfsec . --format json -o tfsec-results.json

# 检查HIGH级别问题
cat tfsec-results.json | jq '.results[] | select(.severity=="HIGH")' | wc -l

# 生成报告
tfsec . --format markdown -o tfsec-report.md
```

**预期结果**: 无HIGH/CRITICAL问题

---

**SEC-003: Secrets泄露检测**

```bash
# 使用detect-secrets扫描
detect-secrets scan .github/workflows/*.yml

# 使用gitleaks扫描
gitleaks detect --source . --verbose --no-git

# 检查GitHub Actions日志
# 手动review最近的workflow run日志,确保无敏感信息打印
```

---

**SEC-004: RBAC权限最小化测试**

```bash
# 验证AKS RBAC配置
az aks show --name hermesflow-dev-aks --resource-group hermesflow-dev-rg \
  --query 'aadProfile.managed' -o tsv | grep -q "true"

# 验证Service Principal只有必要权限
az role assignment list --assignee $SP_ID --query '[].roleDefinitionName'

# 预期: ["Contributor", "User Access Administrator"]
```

---

**SEC-005: Network Security Group规则审计**

```bash
# 获取NSG规则
az network nsg rule list --nsg-name hermesflow-dev-aks-nsg --resource-group hermesflow-dev-rg

# 验证没有允许0.0.0.0/0的规则(除HTTPS)
az network nsg rule list --nsg-name hermesflow-dev-aks-nsg --resource-group hermesflow-dev-rg \
  --query "[?sourceAddressPrefix=='*'].{name:name, access:access, protocol:protocol, port:destinationPortRange}"
```

---

**SEC-006-SEC-012**: 其他安全测试用例
- SEC-006: Key Vault网络ACL测试
- SEC-007: PostgreSQL SSL强制测试
- SEC-008: ACR管理员账户禁用测试
- SEC-009: Terraform State加密验证
- SEC-010: GitHub Secrets访问日志审计
- SEC-011: Pod Security Standards测试
- SEC-012: CVE扫描集成测试

---

### 2.5 性能测试 (Performance Tests)

**目标**: 建立性能基准,验证性能指标。

**测试数量**: 10个用例 (P0: 5, P1: 5)

#### 关键测试场景

**PERF-001: CI构建时间基准测试**

**测试场景**: 测量首次构建和缓存构建时间

```bash
# 首次构建(无缓存)
TIME_START=$(date +%s)
# 触发GitHub Actions workflow
TIME_END=$(date +%s)
DURATION=$((TIME_END - TIME_START))

echo "首次构建时间: ${DURATION}秒"

# 目标:
# - Rust: < 900s (15min)
# - Java: < 600s (10min)
# - Python: < 300s (5min)
```

**预期结果**: 满足目标时间

---

**PERF-002: 构建缓存效率测试**

```bash
# 第二次构建(有缓存)
TIME_START=$(date +%s)
# 触发GitHub Actions workflow (无代码变更)
TIME_END=$(date +%s)
CACHED_DURATION=$((TIME_END - TIME_START))

# 计算缓存效率
IMPROVEMENT=$(echo "scale=2; (1 - $CACHED_DURATION / $DURATION) * 100" | bc)
echo "缓存提升: ${IMPROVEMENT}%"

# 目标: >50%提升
```

---

**PERF-003: Terraform Apply时间测试**

```bash
# 测量完整基础设施创建时间
cd infrastructure/terraform/environments/dev

TIME_START=$(date +%s)
terraform apply -auto-approve
TIME_END=$(date +%s)
DURATION=$((TIME_END - TIME_START))

echo "Terraform Apply时间: ${DURATION}秒 ($(($DURATION / 60))分钟)"

# 目标: < 30分钟
```

---

**PERF-004: AKS节点启动时间测试**

```bash
# 扩展节点池并测量启动时间
az aks nodepool scale \
  --cluster-name hermesflow-dev-aks \
  --resource-group hermesflow-dev-rg \
  --name user \
  --node-count 3

# 监控节点Ready状态
kubectl get nodes -w

# 目标: < 5分钟
```

---

**PERF-005: Docker镜像构建时间测试**

```bash
# 测量各语言镜像构建时间
TIME_START=$(date +%s)
docker build -t rust-test -f modules/data-engine/Dockerfile modules/data-engine
TIME_END=$(date +%s)
echo "Rust镜像构建: $((TIME_END - TIME_START))秒"

# 目标:
# - Rust: < 600s
# - Java: < 300s
# - Python: < 120s
```

---

**PERF-006-PERF-010**: 其他性能测试用例
- PERF-006: ACR镜像推送速度测试
- PERF-007: Terraform Plan时间测试
- PERF-008: Kubectl API响应时间测试
- PERF-009: Log Analytics查询性能测试
- PERF-010: 并行构建效率测试

---

### 2.6 灾难恢复测试 (Disaster Recovery Tests)

**目标**: 验证备份和恢复机制。

**测试数量**: 8个用例 (P0: 4, P1: 4)

#### 关键测试场景

**DR-001: Terraform State恢复测试**

**测试步骤**:
1. 备份当前State:
   ```bash
   terraform state pull > backup.tfstate
   ```
2. 模拟State损坏:
   ```bash
   # 删除远程State(在测试环境)
   az storage blob delete --account-name hermesflowdevtfstate \
     --container-name tfstate --name dev.terraform.tfstate
   ```
3. 恢复State:
   ```bash
   terraform state push backup.tfstate
   ```
4. 验证恢复:
   ```bash
   terraform plan
   # 应该显示: No changes
   ```

**预期结果**: State成功恢复,plan显示无变更

---

**DR-002: 资源销毁和重建测试**

```bash
# 1. 记录当前资源状态
az resource list --resource-group hermesflow-dev-rg -o json > resources-before.json

# 2. 销毁所有资源
terraform destroy -auto-approve

# 3. 重建资源
terraform apply -auto-approve

# 4. 验证资源一致性
az resource list --resource-group hermesflow-dev-rg -o json > resources-after.json
diff resources-before.json resources-after.json
```

**预期结果**: 资源成功重建,配置一致

---

**DR-003: PostgreSQL备份恢复测试**

```bash
# 1. 创建测试数据
psql -h hermesflow-dev-postgres.postgres.database.azure.com \
  -U hermesadmin -d hermesflow \
  -c "CREATE TABLE test_table (id INT, data TEXT);"
psql -h ... -c "INSERT INTO test_table VALUES (1, 'test data');"

# 2. 触发手动备份
az postgres flexible-server backup create \
  --name hermesflow-dev-postgres \
  --resource-group hermesflow-dev-rg \
  --backup-name manual-backup-$(date +%Y%m%d)

# 3. 模拟数据丢失
psql -h ... -c "DROP TABLE test_table;"

# 4. 恢复备份
az postgres flexible-server restore \
  --source-server hermesflow-dev-postgres \
  --resource-group hermesflow-dev-rg \
  --name hermesflow-dev-postgres-restored \
  --restore-time "2025-01-15T10:00:00Z"

# 5. 验证数据恢复
psql -h hermesflow-dev-postgres-restored.postgres.database.azure.com \
  -U hermesadmin -d hermesflow \
  -c "SELECT * FROM test_table;"
```

---

**DR-004: GitHub Actions workflow恢复测试**

**测试场景**: 验证工作流文件从Git历史恢复

```bash
# 1. 删除workflow文件
git rm .github/workflows/ci-rust.yml
git commit -m "test: delete workflow"
git push

# 2. 恢复文件
git revert HEAD
git push

# 3. 验证workflow正常工作
# 触发workflow并检查执行
```

---

**DR-005-DR-008**: 其他灾难恢复测试用例
- DR-005: AKS节点故障恢复测试
- DR-006: ACR镜像备份验证
- DR-007: Key Vault软删除恢复测试
- DR-008: Log Analytics数据保留测试

---

## 🛠️ III. 测试环境配置

### 3.1 测试环境矩阵

| 环境 | 用途 | Azure订阅 | Terraform Workspace | 数据隔离 |
|------|------|-----------|---------------------|---------|
| **dev** | 集成测试、性能测试 | hermesflow-dev | dev | 独立RG |
| **test** | 破坏性测试、DR测试 | hermesflow-test | test | 独立RG |
| **local** | 单元测试 | N/A | N/A | Docker Compose |

### 3.2 测试工具栈

| 类别 | 工具 | 版本 | 用途 |
|------|------|------|------|
| **CI/CD测试** | actionlint | latest | GitHub Actions语法检查 |
| **IaC测试** | terraform-compliance | 1.3.x | Terraform策略测试 |
| **IaC测试** | terratest | 0.46.x | Terraform单元测试 |
| **安全扫描** | trivy | 0.48.x | 容器镜像扫描 |
| **安全扫描** | tfsec | 1.28.x | Terraform安全扫描 |
| **安全扫描** | detect-secrets | 1.4.x | Secrets泄露检测 |
| **性能测试** | time | built-in | 时间测量 |
| **网络测试** | kubectl | 1.28.x | K8s连通性测试 |
| **数据库测试** | psql | 15.x | PostgreSQL测试 |

### 3.3 测试数据管理

**测试数据原则**:
- ✅ 使用合成数据,避免真实敏感数据
- ✅ 测试后自动清理
- ✅ 可重复生成

**测试镜像**:
```bash
# 准备测试用Docker镜像
docker pull busybox:latest
docker pull postgres:15
docker pull nginx:alpine

docker tag busybox:latest hermesflow-dev-acr.azurecr.io/test:latest
docker push hermesflow-dev-acr.azurecr.io/test:latest
```

---

## 📅 IV. 测试执行计划

### 4.1 测试时间线

```
Sprint 1 Timeline:
┌─────────────────────────────────────────────────────────────┐
│ Week 1 (2025-01-10 ~ 2025-01-17)                           │
├─────────────────────────────────────────────────────────────┤
│ Day 1-2: DEVOPS-002开发 → 单元测试(UT)同步执行           │
│ Day 3-4: DEVOPS-002完成 → 基础设施测试(INF)开始          │
│ Day 5: 安全测试(SEC) + 性能基准测试(PERF)                 │
│                                                             │
│ Week 2 (2025-01-17 ~ 2025-01-24)                           │
├─────────────────────────────────────────────────────────────┤
│ Day 1-2: DEVOPS-001开发 → 单元测试(UT)同步执行           │
│ Day 3: DEVOPS-001完成 → 集成测试(IT)开始                  │
│ Day 4: 端到端测试 + 灾难恢复测试(DR)                      │
│ Day 5: 回归测试 + Sprint验收测试                          │
└─────────────────────────────────────────────────────────────┘
```

### 4.2 测试优先级排序

**Phase 1: 基础验证** (Day 1-5)
- Priority: P0
- 测试类型: UT + INF (核心资源)
- 通过率要求: 100%

**Phase 2: 功能验证** (Day 6-9)
- Priority: P0 + P1
- 测试类型: IT + SEC + PERF
- 通过率要求: P0=100%, P1≥90%

**Phase 3: 综合验证** (Day 10-14)
- Priority: 全部
- 测试类型: 全部
- 通过率要求: P0=100%, P1≥95%

### 4.3 每日测试检查清单

**开发者每日自测** (提交代码前):
- [ ] 本地单元测试通过
- [ ] 代码lint检查通过
- [ ] Terraform validate通过
- [ ] 本地Docker构建成功

**QA每日测试** (代码合并后):
- [ ] CI工作流全部成功
- [ ] 相关集成测试通过
- [ ] 无新增HIGH/CRITICAL安全漏洞
- [ ] 性能无明显退化

---

## ✅ V. 验收测试清单

### 5.1 Sprint验收标准

**DEVOPS-001验收**:
- [ ] 所有P0测试用例通过 (UT-001~015, IT-001~010, SEC-001~008)
- [ ] Rust/Java/Python/React模块能成功构建
- [ ] Docker镜像推送到ACR
- [ ] 安全扫描无CRITICAL漏洞
- [ ] GitOps自动更新验证通过
- [ ] CI构建时间满足性能目标

**DEVOPS-002验收**:
- [ ] 所有P0测试用例通过 (INF-001~020, DR-001~004)
- [ ] 所有Azure资源按规格创建
- [ ] 网络连通性测试全部通过
- [ ] RBAC和安全配置正确
- [ ] Terraform State管理正常
- [ ] 成本在预算内($700/月)

**Sprint整体验收**:
- [ ] 端到端CI/CD流程测试通过
- [ ] 所有P0缺陷已修复
- [ ] ≥95% P1缺陷已修复
- [ ] 文档完整且更新
- [ ] Sprint Review Demo准备完成

### 5.2 验收测试场景

**场景1: 完整CI/CD流程**
```gherkin
Given 开发者完成feature开发
When 推送代码到feature分支
Then GitHub Actions自动触发构建
And 所有测试通过
And Docker镜像推送到ACR
And 安全扫描通过

When 合并PR到main分支
Then 触发main分支构建
And GitOps仓库自动更新
And ArgoCD检测到变更

Then 整个流程<20分钟完成
```

**场景2: 基础设施验证**
```gherkin
Given Terraform配置已完成
When 执行terraform apply
Then 所有Azure资源创建成功
And 网络连通性正常
And AKS能访问ACR
And AKS能访问PostgreSQL
And 监控和告警配置正确

Then 基础设施创建<30分钟
```

---

## 📊 VI. 测试度量与报告

### 6.1 测试度量指标

| 指标 | 公式 | 目标值 |
|------|------|--------|
| **测试覆盖率** | (执行用例数 / 总用例数) × 100% | 100% |
| **测试通过率** | (通过用例数 / 执行用例数) × 100% | P0=100%, P1≥95% |
| **缺陷密度** | 缺陷总数 / Story Points | <2 |
| **缺陷逃逸率** | 生产缺陷数 / 总缺陷数 × 100% | <5% |
| **自动化率** | 自动化用例数 / 总用例数 × 100% | ≥80% |
| **平均修复时间** | Σ修复时间 / 缺陷数 | <4h (P0), <24h (P1) |

### 6.2 测试报告模板

**每日测试报告**:
```markdown
# 测试日报 - 2025-01-XX

## 测试执行情况
- 计划执行: XX个用例
- 实际执行: XX个用例
- 通过: XX个
- 失败: XX个
- 阻塞: XX个

## 新增缺陷
- DEF-001: [缺陷描述] (P0/P1/P2)

## 风险与问题
- [问题描述]

## 明日计划
- [测试计划]
```

**Sprint测试总结报告**:
```markdown
# Sprint 1 测试总结报告

## 测试统计
- 总用例数: 100
- 执行率: XX%
- 通过率: XX%
- 自动化率: 80%

## 缺陷统计
- P0缺陷: X个(100%修复)
- P1缺陷: X个(XX%修复)
- P2缺陷: X个(记录backlog)

## 质量评估
- Sprint目标达成度: XX%
- 质量等级: A/B/C

## 改进建议
- [建议内容]
```

---

## 🔗 VII. 附录

### A. 测试工具安装

```bash
# actionlint
brew install actionlint  # macOS
# or
go install github.com/rhysd/actionlint/cmd/actionlint@latest

# Terratest
go get github.com/gruntwork-io/terratest/modules/terraform

# Trivy
brew install aquasecurity/trivy/trivy

# tfsec
brew install tfsec

# detect-secrets
pip install detect-secrets
```

### B. 常用测试命令

```bash
# Terraform测试
terraform fmt -check -recursive
terraform validate
terraform plan

# Docker测试
docker build -t test:latest .
docker run --rm test:latest

# Kubernetes测试
kubectl get all
kubectl describe pod <pod-name>
kubectl logs <pod-name>

# Azure CLI测试
az group exists --name <rg-name>
az aks show --name <aks-name> --resource-group <rg-name>
```

### C. 测试文档引用

- [Sprint 1 Summary](./sprint-01-summary.md)
- [Sprint 1 Risk Profile](./sprint-01-risk-profile.md)
- [Sprint 1 Test Cases](./sprint-01-test-cases.md) ← 详细测试用例
- [DEVOPS-001 Story](./DEVOPS-001-github-actions-cicd.md)
- [DEVOPS-002 Story](./DEVOPS-002-azure-terraform-iac.md)

---

**Document Version**: 1.0  
**Next Review**: 2025-01-20 (Sprint中期)  
**Approved By**: @qa.mdc

**测试格言**: _"Test early, test often, automate relentlessly."_

