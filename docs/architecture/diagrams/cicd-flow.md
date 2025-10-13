# CI/CD 流程图与时序图

本文档包含 HermesFlow 项目的 CI/CD 流程的详细可视化图表。

## 1. 整体流程图

### 1.1 完整部署流水线

```mermaid
graph TB
    A[开发者提交代码] --> B{Commit Message<br/>包含 [module:xxx]?}
    B -->|是| C[GitHub Actions 触发]
    B -->|否| Z[跳过构建]
    
    C --> D[解析模块标签]
    D --> E[设置环境变量<br/>dev/main]
    E --> F[构建源代码]
    
    F --> G{模块类型?}
    G -->|Java| H1[Maven 构建]
    G -->|Rust| H2[Cargo 构建]
    G -->|Node.js| H3[npm 构建]
    
    H1 --> I[构建 Docker 镜像]
    H2 --> I
    H3 --> I
    
    I --> J[推送到 Azure Container Registry]
    J --> K[触发 GitOps 仓库更新]
    
    K --> L[GitOps Workflow 执行]
    L --> M[更新 Helm values.yaml]
    M --> N[Git Commit & Push]
    
    N --> O[ArgoCD 检测变化]
    O --> P[ArgoCD 同步]
    P --> Q[Kubernetes 滚动更新]
    
    Q --> R[新 Pod 启动]
    R --> S{健康检查通过?}
    S -->|是| T[旧 Pod 终止]
    S -->|否| U[回滚]
    T --> V[部署完成]
    
    style C fill:#e1f5ff
    style I fill:#fff3cd
    style M fill:#d4edda
    style Q fill:#cce5ff
    style V fill:#d1e7dd
```

### 1.2 分支策略流程

```mermaid
graph LR
    A[feature/*] -->|Pull Request| B[dev 分支]
    C[hotfix/*] -->|Pull Request| B
    B -->|自动部署| D[dev 环境<br/>hermesflow-dev]
    
    B -->|Pull Request<br/>需要审核| E[main 分支]
    E -->|自动部署| F[main 环境<br/>hermesflow-prod]
    
    style B fill:#17a2b8
    style D fill:#28a745
    style E fill:#dc3545
    style F fill:#ffc107
```

## 2. 时序图

### 2.1 标准部署时序

```mermaid
sequenceDiagram
    autonumber
    participant Dev as 👨‍💻 开发者
    participant Git as GitHub
    participant GA as GitHub Actions
    participant ACR as Azure Container Registry
    participant GitOps as HermesFlow-GitOps
    participant ArgoCD as ArgoCD
    participant K8s as Kubernetes Cluster
    
    Dev->>Git: git push (commit: [module:data-engine] fix: ...)
    Git->>GA: Webhook 触发 workflow
    
    rect rgb(230, 245, 255)
        Note over GA: CI阶段 (~5分钟)
        GA->>GA: 1. 解析模块标签
        GA->>GA: 2. 设置环境变量
        GA->>GA: 3. 构建 Rust 代码 (cargo build)
        GA->>GA: 4. 构建 Docker 镜像
        GA->>ACR: 5. 推送镜像<br/>tag: abc123def456
        ACR-->>GA: 推送成功
    end
    
    GA->>GitOps: 6. 触发 workflow_dispatch<br/>(module, tag, env, registry)
    
    rect rgb(230, 255, 230)
        Note over GitOps: GitOps更新阶段 (~1分钟)
        GitOps->>GitOps: 7. Checkout 仓库
        GitOps->>GitOps: 8. 使用 yq 更新 values.yaml
        GitOps->>GitOps: 9. Git commit & push
    end
    
    rect rgb(255, 245, 230)
        Note over ArgoCD,K8s: CD阶段 (~3分钟)
        ArgoCD->>GitOps: 10. 轮询检测变化 (每3分钟)
        GitOps-->>ArgoCD: values.yaml 已更新
        ArgoCD->>ArgoCD: 11. 渲染 Helm Chart
        ArgoCD->>ArgoCD: 12. 计算资源差异
        ArgoCD->>K8s: 13. 应用 Deployment
        
        K8s->>K8s: 14. 创建新 Pod (abc123def)
        K8s->>K8s: 15. 健康检查 (readiness probe)
        K8s->>K8s: 16. 新 Pod Ready
        K8s->>K8s: 17. 终止旧 Pod (xyz789old)
        K8s-->>ArgoCD: 同步完成
    end
    
    ArgoCD-->>Dev: 18. 部署完成通知
```

### 2.2 回滚时序

```mermaid
sequenceDiagram
    autonumber
    participant Ops as 👨‍💻 运维人员
    participant K8s as Kubernetes
    participant ArgoCD as ArgoCD
    participant GitOps as HermesFlow-GitOps
    
    Note over Ops: 方法1: Kubernetes原生回滚 (~30秒)
    Ops->>K8s: kubectl rollout undo deployment/data-engine
    K8s->>K8s: 回滚到上一个 ReplicaSet
    K8s-->>Ops: 回滚完成
    
    Note over Ops: 方法2: ArgoCD回滚 (~2分钟)
    Ops->>ArgoCD: argocd app rollback hermesflow-dev 2
    ArgoCD->>K8s: 应用历史版本的资源清单
    K8s-->>ArgoCD: 应用成功
    ArgoCD-->>Ops: 回滚完成
    
    Note over Ops: 方法3: GitOps回滚 (~3分钟)
    Ops->>GitOps: git revert HEAD && git push
    GitOps-->>ArgoCD: 检测到 values.yaml 变化
    ArgoCD->>K8s: 自动同步到旧版本
    K8s-->>ArgoCD: 同步完成
    ArgoCD-->>Ops: 回滚完成
```

### 2.3 失败处理时序

```mermaid
sequenceDiagram
    participant GA as GitHub Actions
    participant ACR as Azure Container Registry
    participant K8s as Kubernetes
    participant Alert as 告警系统
    
    GA->>ACR: 推送镜像
    
    alt 推送失败
        ACR-->>GA: 认证失败 (401)
        GA->>Alert: 发送告警: ACR 认证失败
        GA->>GA: 任务失败，停止流程
    end
    
    alt 部署失败
        K8s->>K8s: 新 Pod 启动
        K8s->>K8s: 健康检查失败 (CrashLoopBackOff)
        K8s->>Alert: 发送告警: Pod 启动失败
        K8s->>K8s: 保留旧 Pod (maxUnavailable: 0)
        K8s->>K8s: 自动回滚到上一个版本
    end
```

## 3. 组件交互图

### 3.1 CI/CD 系统组件

```mermaid
graph TB
    subgraph "代码仓库"
        A1[HermesFlow<br/>源代码]
        A2[HermesFlow-GitOps<br/>声明式配置]
    end
    
    subgraph "CI系统"
        B1[GitHub Actions]
        B2[build-module.sh]
        B3[Docker Build]
    end
    
    subgraph "镜像仓库"
        C1[Azure Container Registry<br/>dev环境]
        C2[Azure Container Registry<br/>main环境]
    end
    
    subgraph "CD系统"
        D1[ArgoCD Server]
        D2[ArgoCD Application<br/>hermesflow-dev]
        D3[ArgoCD Application<br/>hermesflow-main]
    end
    
    subgraph "运行环境"
        E1[AKS Dev Cluster]
        E2[AKS Main Cluster]
    end
    
    A1 -->|git push| B1
    B1 --> B2
    B2 --> B3
    B3 --> C1
    B3 --> C2
    
    B1 -->|workflow_dispatch| A2
    A2 -->|git poll| D1
    
    D1 --> D2
    D1 --> D3
    
    D2 -->|kubectl apply| E1
    D3 -->|kubectl apply| E2
    
    E1 -.->|pull image| C1
    E2 -.->|pull image| C2
    
    style B1 fill:#e1f5ff
    style D1 fill:#d4edda
    style C1 fill:#fff3cd
    style C2 fill:#fff3cd
```

### 3.2 环境隔离架构

```mermaid
graph TB
    subgraph "dev 分支"
        A1[modules/data-engine]
    end
    
    subgraph "main 分支"
        A2[modules/data-engine]
    end
    
    subgraph "GitHub Actions"
        B[module-cicd.yml]
    end
    
    subgraph "Dev 环境"
        C1[hermesflow-dev-acr.azurecr.io]
        D1[HermesFlow-GitOps<br/>apps/dev/]
        E1[AKS hermesflow-dev namespace]
    end
    
    subgraph "Main 环境"
        C2[hermesflow-prod-acr.azurecr.io]
        D2[HermesFlow-GitOps<br/>apps/main/]
        E2[AKS hermesflow-prod namespace]
    end
    
    A1 -->|push| B
    A2 -->|push| B
    
    B -->|dev 分支| C1
    B -->|main 分支| C2
    
    C1 --> D1
    C2 --> D2
    
    D1 --> E1
    D2 --> E2
    
    style C1 fill:#28a745
    style C2 fill:#dc3545
    style E1 fill:#28a745
    style E2 fill:#dc3545
```

## 4. 数据流图

### 4.1 镜像标签流转

```mermaid
graph LR
    A[Commit SHA<br/>abc123def456] --> B[GitHub Actions<br/>$GITHUB_SHA]
    
    B --> C[Docker Image Tag<br/>hermesflow-dev-acr.azurecr.io/data-engine:abc123def456]
    B --> D[Docker Image Tag<br/>hermesflow-dev-acr.azurecr.io/data-engine:dev-latest]
    
    C --> E[values.yaml<br/>image.tag: abc123def456]
    D --> F[快速引用<br/>最新版本]
    
    E --> G[Kubernetes Deployment<br/>image: xxx/data-engine:abc123def456]
    
    style A fill:#ffc107
    style C fill:#17a2b8
    style E fill:#28a745
    style G fill:#6c757d
```

### 4.2 配置传递链

```mermaid
graph TB
    A[GitHub Secrets] --> B[GitHub Actions 环境变量]
    
    B --> C1[AZURE_REGISTRY]
    B --> C2[AZURE_CLIENT_ID]
    B --> C3[AZURE_CLIENT_SECRET]
    B --> C4[GITOPS_TOKEN]
    
    C1 --> D[build-module.sh]
    C2 --> D
    C3 --> D
    
    C4 --> E[curl POST<br/>workflow_dispatch]
    
    E --> F[GitOps Repo<br/>update-values.yml]
    
    F --> G[values.yaml<br/>image.repository<br/>image.tag]
    
    G --> H[ArgoCD]
    
    H --> I[Kubernetes Secret<br/>imagePullSecrets]
    
    style A fill:#dc3545
    style D fill:#ffc107
    style F fill:#28a745
    style H fill:#17a2b8
```

## 5. 状态转换图

### 5.1 部署状态机

```mermaid
stateDiagram-v2
    [*] --> Idle: 代码提交
    
    Idle --> Parsing: GitHub Actions 触发
    Parsing --> Building: 解析模块标签成功
    Parsing --> Idle: 无模块标签
    
    Building --> DockerBuild: 源代码构建成功
    Building --> Failed: 构建失败
    
    DockerBuild --> Pushing: Docker 镜像构建成功
    DockerBuild --> Failed: 镜像构建失败
    
    Pushing --> GitOpsUpdate: 推送到 ACR 成功
    Pushing --> Failed: 推送失败
    
    GitOpsUpdate --> ArgoCDSync: values.yaml 更新成功
    GitOpsUpdate --> Failed: 更新失败
    
    ArgoCDSync --> K8sDeploying: ArgoCD 同步触发
    ArgoCDSync --> Failed: 同步失败
    
    K8sDeploying --> HealthCheck: 新 Pod 启动
    K8sDeploying --> Failed: Pod 创建失败
    
    HealthCheck --> Healthy: 健康检查通过
    HealthCheck --> Unhealthy: 健康检查失败
    
    Healthy --> Completed: 旧 Pod 终止
    Unhealthy --> Rollback: 自动回滚
    
    Rollback --> Idle: 回滚完成
    Completed --> [*]
    Failed --> [*]
```

### 5.2 Pod 生命周期

```mermaid
stateDiagram-v2
    [*] --> Pending: Deployment 创建
    
    Pending --> ImagePull: 调度到 Node
    ImagePull --> ContainerCreating: 镜像拉取成功
    ImagePull --> ImagePullBackOff: 镜像拉取失败
    
    ContainerCreating --> Running: 容器启动
    ContainerCreating --> CrashLoopBackOff: 启动失败
    
    Running --> Ready: Readiness Probe 通过
    Running --> NotReady: Readiness Probe 失败
    
    Ready --> Terminating: 部署新版本
    NotReady --> CrashLoopBackOff: 持续失败
    
    Terminating --> [*]: Pod 终止
    CrashLoopBackOff --> [*]: 放弃重试
    ImagePullBackOff --> [*]: 放弃重试
```

## 6. 网络拓扑

### 6.1 CI/CD 网络架构

```mermaid
graph TB
    subgraph "Internet"
        A[GitHub]
    end
    
    subgraph "Azure Public Services"
        B[Azure Container Registry]
    end
    
    subgraph "AKS Cluster (Private)"
        C[ArgoCD Server]
        D[Data Engine Pods]
        E[Strategy Engine Pods]
    end
    
    subgraph "External Monitoring"
        F[Prometheus]
        G[Grafana]
    end
    
    A -->|HTTPS| B
    A -->|HTTPS| C
    
    C -->|HTTPS| B
    C -->|API| D
    C -->|API| E
    
    D -->|metrics| F
    E -->|metrics| F
    F -->|query| G
    
    style A fill:#24292e
    style B fill:#0078d4
    style C fill:#ef7b4d
    style F fill:#e6522c
    style G fill:#f46800
```

## 7. 时间线图

### 7.1 完整部署时间线（9分钟）

```
0:00 ────────────────────────────────────────────────────────── 开发者 git push
  │
  ├── 0:05  GitHub Actions 触发
  │
  ├── 0:10  解析模块标签 + 设置环境变量
  │
  ├── 0:15  开始构建 Rust 源代码
  │
  ├── 3:45  源代码构建完成
  │
  ├── 3:50  开始构建 Docker 镜像
  │
  ├── 4:30  Docker 镜像构建完成
  │
  ├── 4:35  开始推送到 ACR
  │
  ├── 4:50  推送完成
  │
  ├── 4:55  触发 GitOps workflow
  │
5:00 ────────────────────────────────────────────────────────── GitHub Actions 完成
  │
  ├── 5:05  GitOps workflow 触发
  │
  ├── 5:10  Checkout GitOps 仓库
  │
  ├── 5:15  使用 yq 更新 values.yaml
  │
  ├── 5:20  Git commit & push
  │
6:00 ────────────────────────────────────────────────────────── GitOps 更新完成
  │
  ├── 6:30  ArgoCD 检测到变化（轮询）
  │
  ├── 6:35  渲染 Helm Chart
  │
  ├── 6:40  计算资源差异
  │
  ├── 6:45  应用 Deployment 到 K8s
  │
  ├── 6:50  Kubernetes 创建新 Pod
  │
  ├── 7:10  新 Pod Running
  │
  ├── 7:20  Readiness Probe 开始 (initialDelaySeconds: 10s)
  │
  ├── 7:30  Readiness Probe 通过
  │
  ├── 7:35  新 Pod Ready
  │
  ├── 7:40  开始终止旧 Pod
  │
  ├── 8:00  旧 Pod 优雅关闭完成
  │
  ├── 8:30  ArgoCD 更新同步状态
  │
9:00 ────────────────────────────────────────────────────────── 部署完成 ✅
```

## 8. 容量规划图

### 8.1 并发构建容量

```mermaid
gantt
    title GitHub Actions 并发构建能力
    dateFormat HH:mm
    axisFormat %H:%M
    
    section Runner 1
    data-engine build    :done, r1-1, 10:00, 5m
    frontend build       :done, r1-2, 10:05, 3m
    
    section Runner 2
    strategy-engine build :done, r2-1, 10:00, 4m
    risk-engine build     :done, r2-2, 10:04, 5m
    
    section Runner 3
    trading-engine build  :done, r3-1, 10:00, 5m
    user-management build :done, r3-2, 10:05, 5m
    
    section Queue
    gateway build         :crit, q1, 10:03, 2m
```

---

## 参考资料

- [GitHub Actions 工作流语法](https://docs.github.com/en/actions/using-workflows/workflow-syntax-for-github-actions)
- [ArgoCD 架构文档](https://argo-cd.readthedocs.io/en/stable/operator-manual/architecture/)
- [Kubernetes Deployment 滚动更新策略](https://kubernetes.io/docs/concepts/workloads/controllers/deployment/#rolling-update-deployment)
- [Helm Chart 最佳实践](https://helm.sh/docs/chart_best_practices/)

---

**最后更新**: 2024-12-20  
**维护团队**: HermesFlow Architecture Team

