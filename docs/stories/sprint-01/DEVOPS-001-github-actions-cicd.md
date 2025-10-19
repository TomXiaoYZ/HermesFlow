# Story 1: GitHub Actions CI/CD Pipeline Setup

**Story ID**: DEVOPS-001  
**Epic**: DevOps Foundation  
**Priority**: P0 (Critical)  
**Estimate**: 8 Story Points (16 hours)  
**Sprint**: Sprint 1 (2025-01-10 ~ 2025-01-24)  
**Status**: Approved  
**Created**: 2025-01-13  
**Created By**: @sm.mdc  
**Validated By**: @po.mdc

---

## 📖 User Story

**作为** DevOps工程师  
**我想要** 建立GitHub Actions CI/CD自动化流水线  
**以便** 自动化构建、测试、发布Rust/Java/Python微服务，并推送Docker镜像到Azure Container Registry

---

## 🎯 验收标准 (Acceptance Criteria)

### 1. 多语言构建支持

```gherkin
Scenario: Rust服务自动构建
  Given 代码推送到feature分支包含Rust模块变更
  When GitHub Actions工作流被触发
  Then 系统应该:
    - 检测到Rust代码变更
    - 使用Cargo编译代码
    - 运行cargo test (覆盖率 ≥ 85%)
    - 运行cargo clippy检查
    - 构建Docker镜像
    - 推送镜像到ACR (tag: branch-${sha})
```

```gherkin
Scenario: Java服务自动构建
  Given 代码推送到feature分支包含Java模块变更  
  When GitHub Actions工作流被触发
  Then 系统应该:
    - 检测到Java代码变更
    - 使用Maven/Gradle构建
    - 运行测试 (覆盖率 ≥ 80%)
    - 运行Checkstyle检查
    - 构建Docker镜像
    - 推送镜像到ACR (tag: branch-${sha})
```

```gherkin
Scenario: Python服务自动构建
  Given 代码推送到feature分支包含Python模块变更
  When GitHub Actions工作流被触发  
  Then 系统应该:
    - 检测到Python代码变更
    - 安装依赖(requirements.txt)
    - 运行pytest (覆盖率 ≥ 75%)
    - 运行pylint检查
    - 构建Docker镜像
    - 推送镜像到ACR (tag: branch-${sha})
```

### 2. 智能路径检测

- [ ] 工作流能检测变更的模块路径
- [ ] 只构建受影响的模块，节省CI时间
- [ ] 共享库变更触发所有依赖模块构建

### 3. 安全扫描

- [ ] 使用Trivy扫描Docker镜像漏洞
- [ ] 发现HIGH/CRITICAL漏洞时构建失败
- [ ] 扫描结果上传到GitHub Security tab

### 4. 自动化Git标签和版本号

- [ ] main分支合并自动创建release tag
- [ ] 镜像标签包含: `{version}`, `{sha}`, `latest`
- [ ] PR合并后自动更新GitOps仓库的image tag

### 5. 构建缓存优化

- [ ] Rust使用sccache或cargo-cache
- [ ] Java使用Maven/Gradle缓存
- [ ] Python使用pip cache
- [ ] Docker layer缓存利用

### 6. 通知机制

- [ ] 构建失败发送Slack/Email通知
- [ ] PR状态检查显示在GitHub界面

---

## 🔧 技术任务分解 (Technical Tasks)

### Task 1.1: 创建GitHub Actions工作流文件结构 (2h)

**负责人**: DevOps Lead

**具体任务**:
```yaml
.github/workflows/
├── ci-rust.yml           # Rust模块CI
├── ci-java.yml           # Java模块CI  
├── ci-python.yml         # Python模块CI
├── ci-frontend.yml       # React前端CI
├── cd-main.yml           # 主分支自动部署
├── security-scan.yml     # 定期安全扫描
└── update-gitops.yml     # 更新GitOps仓库
```

**验收**:
- [ ] 工作流文件结构创建
- [ ] 基础触发器配置完成(push, pull_request, workflow_dispatch)

---

### Task 1.2: 实现Rust服务CI工作流 (3h)

**负责人**: Rust Developer

**工作流示例**:
```yaml
name: CI - Rust Services

on:
  push:
    branches: [main, develop, 'feature/**']
    paths:
      - 'modules/data-engine/**'
      - 'modules/gateway/**'
  pull_request:
    paths:
      - 'modules/data-engine/**'
      - 'modules/gateway/**'

env:
  RUST_VERSION: 1.75
  CARGO_TERM_COLOR: always

jobs:
  detect-changes:
    runs-on: ubuntu-latest
    outputs:
      data-engine: ${{ steps.filter.outputs.data-engine }}
      gateway: ${{ steps.filter.outputs.gateway }}
    steps:
      - uses: actions/checkout@v4
      - uses: dorny/paths-filter@v2
        id: filter
        with:
          filters: |
            data-engine:
              - 'modules/data-engine/**'
            gateway:
              - 'modules/gateway/**'

  build-rust:
    needs: detect-changes
    runs-on: ubuntu-latest
    strategy:
      matrix:
        module: 
          - { name: data-engine, build: ${{ needs.detect-changes.outputs.data-engine }} }
          - { name: gateway, build: ${{ needs.detect-changes.outputs.gateway }} }
    if: matrix.module.build == 'true'
    
    steps:
      - name: Checkout code
        uses: actions/checkout@v4

      - name: Setup Rust toolchain
        uses: actions-rs/toolchain@v1
        with:
          toolchain: ${{ env.RUST_VERSION }}
          override: true
          components: rustfmt, clippy

      - name: Cache cargo registry
        uses: actions/cache@v3
        with:
          path: ~/.cargo/registry
          key: ${{ runner.os }}-cargo-registry-${{ hashFiles('**/Cargo.lock') }}

      - name: Cache cargo build
        uses: actions/cache@v3
        with:
          path: target
          key: ${{ runner.os }}-cargo-build-${{ hashFiles('**/Cargo.lock') }}

      - name: Run cargo fmt check
        working-directory: modules/${{ matrix.module.name }}
        run: cargo fmt -- --check

      - name: Run cargo clippy
        working-directory: modules/${{ matrix.module.name }}
        run: cargo clippy -- -D warnings

      - name: Run tests
        working-directory: modules/${{ matrix.module.name }}
        run: cargo test --verbose

      - name: Generate coverage report
        working-directory: modules/${{ matrix.module.name }}
        run: |
          cargo install cargo-tarpaulin
          cargo tarpaulin --out Xml --output-dir coverage

      - name: Upload coverage to Codecov
        uses: codecov/codecov-action@v3
        with:
          file: modules/${{ matrix.module.name }}/coverage/cobertura.xml
          flags: rust-${{ matrix.module.name }}

      - name: Build release binary
        working-directory: modules/${{ matrix.module.name }}
        run: cargo build --release

      - name: Login to Azure Container Registry
        uses: azure/docker-login@v1
        with:
          login-server: ${{ secrets.ACR_LOGIN_SERVER }}
          username: ${{ secrets.ACR_USERNAME }}
          password: ${{ secrets.ACR_PASSWORD }}

      - name: Build and push Docker image
        working-directory: modules/${{ matrix.module.name }}
        run: |
          IMAGE_TAG=${{ secrets.ACR_LOGIN_SERVER }}/${{ matrix.module.name }}:${{ github.sha }}
          docker build -t $IMAGE_TAG .
          docker push $IMAGE_TAG
          
          if [[ "${{ github.ref }}" == "refs/heads/main" ]]; then
            docker tag $IMAGE_TAG ${{ secrets.ACR_LOGIN_SERVER }}/${{ matrix.module.name }}:latest
            docker push ${{ secrets.ACR_LOGIN_SERVER }}/${{ matrix.module.name }}:latest
          fi

      - name: Run Trivy security scan
        uses: aquasecurity/trivy-action@master
        with:
          image-ref: ${{ secrets.ACR_LOGIN_SERVER }}/${{ matrix.module.name }}:${{ github.sha }}
          format: 'sarif'
          output: 'trivy-results.sarif'

      - name: Upload Trivy results to GitHub Security
        uses: github/codeql-action/upload-sarif@v2
        with:
          sarif_file: 'trivy-results.sarif'
```

**验收**:
- [ ] 工作流能成功编译Rust代码
- [ ] 测试覆盖率报告生成并上传
- [ ] Docker镜像构建并推送到ACR
- [ ] 安全扫描完成

---

### Task 1.3: 实现Java服务CI工作流 (3h)

**负责人**: Java Developer

**关键步骤**:
- Setup JDK 21 (with virtual threads support)
- Maven/Gradle构建和测试
- Checkstyle和SpotBugs代码质量检查
- Jacoco覆盖率报告
- Docker镜像构建(多阶段构建)
- 推送到ACR

**验收**:
- [ ] 工作流能成功构建Java服务
- [ ] 测试覆盖率 ≥ 80%
- [ ] 代码质量检查通过

---

### Task 1.4: 实现Python服务CI工作流 (2h)

**负责人**: Python Developer

**关键步骤**:
- Setup Python 3.12
- 安装依赖(requirements.txt + requirements-dev.txt)
- Pytest + pytest-cov
- Pylint/Flake8代码检查
- Docker镜像构建
- 推送到ACR

**验收**:
- [ ] 工作流能成功运行Python测试
- [ ] 覆盖率 ≥ 75%
- [ ] Pylint评分 ≥ 8.0

---

### Task 1.5: 实现前端React CI工作流 (2h)

**负责人**: Frontend Developer

**关键步骤**:
- Setup Node.js 20
- npm ci (使用package-lock.json)
- ESLint + Prettier检查
- Jest单元测试
- 构建生产版本(npm run build)
- Nginx + React静态文件Docker镜像

**验收**:
- [ ] 前端构建成功
- [ ] ESLint无错误
- [ ] 单元测试通过

---

### Task 1.6: 实现GitOps自动更新工作流 (3h)

**负责人**: DevOps Lead

**功能描述**:
当main分支构建成功后，自动更新HermesFlow-GitOps仓库的镜像标签。

**工作流示例**:
```yaml
name: Update GitOps Repository

on:
  workflow_run:
    workflows: ["CI - Rust Services", "CI - Java Services", "CI - Python Services"]
    types: [completed]
    branches: [main]

jobs:
  update-gitops:
    if: ${{ github.event.workflow_run.conclusion == 'success' }}
    runs-on: ubuntu-latest
    steps:
      - name: Checkout GitOps repo
        uses: actions/checkout@v4
        with:
          repository: hermesflow/HermesFlow-GitOps
          token: ${{ secrets.GITOPS_PAT }}
          path: gitops

      - name: Update image tags
        run: |
          cd gitops
          # 更新dev环境的values.yaml
          MODULE_NAME="${{ github.event.workflow_run.name }}"
          NEW_TAG="${{ github.sha }}"
          
          yq eval ".image.tag = \"$NEW_TAG\"" -i apps/dev/${MODULE_NAME}/values.yaml
          
          git config user.name "GitHub Actions"
          git config user.email "actions@github.com"
          git add apps/dev/${MODULE_NAME}/values.yaml
          git commit -m "chore: update ${MODULE_NAME} image to ${NEW_TAG}"
          git push
```

**验收**:
- [ ] main分支CI成功后自动触发
- [ ] GitOps仓库values.yaml正确更新
- [ ] ArgoCD检测到变更并自动同步

---

### Task 1.7: 配置GitHub Secrets (1h)

**负责人**: DevOps Lead

**需要配置的Secrets**:
```
ACR_LOGIN_SERVER = hermesflow-dev-acr.azurecr.io
ACR_USERNAME = <service-principal-id>
ACR_PASSWORD = <service-principal-password>
GITOPS_PAT = <personal-access-token>
SLACK_WEBHOOK_URL = <slack-webhook-url>
CODECOV_TOKEN = <codecov-token>
```

**验收**:
- [ ] 所有Secrets配置完成
- [ ] 工作流能成功使用Secrets
- [ ] Secrets权限最小化(least privilege)

---

## 📊 测试策略

### 1. 工作流测试
- [ ] 手动触发(workflow_dispatch)测试每个工作流
- [ ] 创建测试PR验证路径检测
- [ ] 验证缓存机制生效

### 2. 安全测试
- [ ] Trivy扫描发现已知漏洞
- [ ] Secrets不泄露在日志中
- [ ] 仅授权人员可访问CI结果

### 3. 性能测试
- [ ] 首次构建时间基准: Rust < 15min, Java < 10min, Python < 5min
- [ ] 缓存命中后构建时间: 减少50%+
- [ ] 并行构建多模块

---

## 🔗 依赖关系

**前置依赖**:
- [ ] Azure Container Registry已创建 (来自DEVOPS-002或手动创建)
- [ ] GitHub仓库基本结构已建立
- [ ] Service Principal创建(用于ACR推送)

**后续依赖**:
- DEVOPS-003 (ArgoCD安装) 需要CI/CD推送的镜像

**与其他Story的关系**:
- 建议在DEVOPS-002完成后执行，以确保ACR可用
- 可以与DEVOPS-002并行开发，但需手动创建临时ACR

---

## 📚 相关文档

- [Docker部署指南](../../deployment/docker-guide.md)
- [GitOps最佳实践](../../deployment/gitops-best-practices.md)
- [系统架构文档](../../architecture/system-architecture.md)
- [编码规范](../../development/coding-standards.md)

---

## 🎓 学习资源

**GitHub Actions**:
- [GitHub Actions官方文档](https://docs.github.com/en/actions)
- [Actions Marketplace](https://github.com/marketplace?type=actions)

**Docker多阶段构建**:
- [Docker Build Best Practices](https://docs.docker.com/develop/dev-best-practices/)

**Trivy安全扫描**:
- [Trivy官方文档](https://aquasecurity.github.io/trivy/)

---

## ✅ Definition of Done

**代码层面**:
- [ ] 所有工作流文件已创建并通过语法检查
- [ ] 工作流在实际PR中成功运行
- [ ] 代码已Review并合并到main分支
- [ ] 所有路径检测逻辑测试通过

**测试层面**:
- [ ] 每个语言的CI工作流测试通过
- [ ] 安全扫描集成并运行
- [ ] 覆盖率报告正确生成
- [ ] 缓存机制验证有效

**文档层面**:
- [ ] README更新,包含CI badge
- [ ] 添加工作流使用说明文档
- [ ] Secrets配置文档完成
- [ ] 故障排查文档创建

**部署层面**:
- [ ] Docker镜像成功推送到ACR
- [ ] GitOps仓库自动更新机制验证
- [ ] 至少一次完整的CI/CD流程成功执行

---

## 📝 开发笔记 (Dev/QA Notes)

**待开发团队填写**:

### 实现进度
- [ ] Task 1.1 完成
- [ ] Task 1.2 完成
- [ ] Task 1.3 完成
- [ ] Task 1.4 完成
- [ ] Task 1.5 完成
- [ ] Task 1.6 完成
- [ ] Task 1.7 完成

### 技术决策记录
_开发过程中的重要技术决策将记录在此_

### 遇到的问题
_开发过程中遇到的问题和解决方案_

### 性能基准
_实际测量的构建时间和性能数据_

---

## 🔄 Story History

| 日期 | 事件 | 操作人 |
|------|------|--------|
| 2025-01-13 | Story创建 | @sm.mdc |
| 2025-01-13 | Story验证通过 | @po.mdc |
| 2025-01-13 | Story批准进入Sprint Backlog | @po.mdc |

---

**Last Updated**: 2025-01-13  
**Next Review**: Sprint 1 Planning Meeting

