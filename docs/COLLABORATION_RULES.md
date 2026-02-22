# HermesFlow 项目协作规则

**日期**: 2026-02-22  
**项目经理**: 贾维斯 (Jarvis) 🤖  
**项目路径**: `/Users/tomxiao/Desktop/Git/personal/HermesFlow`  
**前端**: http://localhost:3000 (admin/admin)

---

## 一、项目概览

HermesFlow 是一个 Rust 构建的多资产量化交易平台，支持加密货币 (Solana)、美股 (IBKR)、港股 (Futu)。

### 当前阶段
| 阶段 | 名称 | 状态 |
|------|------|------|
| P0 | Walk-Forward OOS Evaluation | ✅ 完成 |
| P1 | Factor Enrichment (13→25) | ✅ 完成 |
| P2 | LLM-Guided Mutation Oracle | ✅ 已部署运行 |
| **P3** | **Multi-Timeframe Factor Stacking** | 🚀 **当前阶段** |
| P4 | Adaptive Threshold Tuning | 📋 计划中 |
| P5 | Strategy Ensemble & Portfolio | 📋 计划中 |
| P6 | Live Paper Trading | 📋 计划中 |

### 核心服务架构
```
Web UI (Next.js:3000) → Gateway (8080) → Data Engine (8081)
                                   ↓
                    Strategy Generator (8082) → Strategy Engine → Execution Engine
```

---

## 二、协作角色定义

| 角色 | 工具 | 职责 |
|------|------|------|
| **产品经理/PM** | 贾维斯 (我) | 项目规划、进度跟踪、协调沟通、规则制定 |
| **主开发** | Claude Code (tmux) | 代码实现、调试、部署、功能验证 |
| **分析顾问** | Gemini | 架构评估、方案建议、代码审查、风险评估 |
| **决策人** | 你 (用户) | 方向确认、关键决策、最终验收 |

---

## 三、协作流程 (Workflow)

### 3.1 迭代开发循环

```
┌─────────────────────────────────────────────────────────────────────┐
│  Phase X 开发迭代                                                    │
├─────────────────────────────────────────────────────────────────────┤
│  1. 规划 → 贾维斯读取文档，确认 Phase X 目标，与 Claude Code 对齐     │
│            ↓                                                        │
│  2. 开发 → Claude Code 实现功能，贾维斯通过 tmux 监控进度             │
│            ↓                                                        │
│  3. 自测 → Claude Code 运行测试、构建 Docker、验证健康检查            │
│            ↓                                                        │
│  4. 部署 → 本地 Docker 部署，服务启动验证                            │
│            ↓                                                        │
│  5. 功能验证 → 前端验证、API 测试、数据库验证                         │
│            ↓                                                        │
│  6. 总结 → Claude Code 生成 Phase X 报告 (markdown)                   │
│            ↓                                                        │
│  7. Gemini 评审 → 贾维斯将报告同步给 Gemini 分析建议                  │
│            ↓                                                        │
│  8. 确认 → 你确认完成，或提出调整                                    │
└─────────────────────────────────────────────────────────────────────┘
```

### 3.2 每个 Phase 必须交付的文档

| 文档 | 位置 | 内容要求 |
|------|------|----------|
| `P{X}_ARCHITECTURE_DESIGN.md` | `docs/` | 技术方案设计、接口变更、依赖分析 |
| `P{X}_IMPLEMENTATION_REPORT.md` | `docs/` | 实现细节、文件变更、测试结果、验证数据 |
| Git Commit | Git History | 每个功能模块独立 commit，message 清晰 |

---

## 四、Claude Code 开发规则

### 4.1 代码提交规则 (来自 CLAUDE.md)
- ✅ **Commit 在完成后**: 每个任务/功能完成后立即 commit，不要留未完成的工作
- ✅ **Commit message 清晰**: 描述做了什么、为什么
- ✅ **本地部署验证**: 修改任何服务后必须：
  1. 构建 Docker 镜像
  2. 启动服务
  3. 验证健康端点
  4. 冒烟测试变更功能
  5. Commit → Push

### 4.2 禁止事项
- ❌ **禁止跳过验证**: 永远不要不验证就进入下一个模块
- ❌ **禁止硬编码密钥**: 使用 `.env` 本地配置，生产用 Secret Store
- ❌ **禁止 shell 脚本**: 所有构建逻辑放在 Makefile 或 GitHub Actions
- ❌ **禁止 node_modules/.venv 进 Git**

### 4.3 修改服务后的验证清单

```bash
# 1. 构建
make build  # 或 docker compose build <service>

# 2. 启动
docker compose up -d <service>

# 3. 健康检查
curl http://localhost:<port>/health

# 4. 功能冒烟测试
curl http://localhost:<port>/<endpoint> | jq

# 5. 前端验证 (如适用)
# 访问 http://localhost:3000 验证 UI

# 6. 提交
git add .
git commit -m "feat(scope): description"
```

---

## 五、Gemini 协作规则

### 5.1 何时请求 Gemini 分析
- 每个 Phase 完成后，生成报告提交给 Gemini
- 遇到架构抉择难题时
- 需要第三方视角评估代码质量时
- 需要进行风险评估时

### 5.2 提交给 Gemini 的内容
1. Phase 实现报告 (`P{X}_IMPLEMENTATION_REPORT.md`)
2. 关键代码片段（如有需要）
3. 具体问题或关注点

### 5.3 Gemini 回复后处理
- 贾维斯总结 Gemini 的建议
- 如有需要，将建议转化为具体任务给 Claude Code
- 关键决策提交给你确认

---

## 六、沟通机制

### 6.1 汇报频率
| 场景 | 方式 | 内容 |
|------|------|------|
| 每日/每阶段开始 | Slack | 贾维斯汇报 Phase X 启动、目标、计划 |
| 关键里程碑 | Slack | 完成重要功能、通过验证、提交 Gemini |
| Gemini 分析后 | Slack | Gemini 建议摘要、待确认事项 |
| 紧急问题 | Slack | 阻塞问题、需要你决策的事项 |

### 6.2 状态追踪
- 贾维斯维护当前 Phase 的进度状态
- 每个任务完成后更新状态
- 异常情况立即上报

---

## 七、Phase 3 目标确认

### 7.1 P3: Multi-Timeframe Factor Stacking

**目标**: 增加时间深度，通过计算多分辨率因子（1h + 4h + 1d）来增强策略表达能力。

**当前状态**: 25 因子 × 1 分辨率 = 25 特征  
**目标状态**: 25 因子 × 3 分辨率 = 75 特征

**关键任务**:
1. 蜡烛重采样: 1h → 4h → 1d
2. 多分辨率因子计算
3. 特征堆叠: `[1h_factors, 4h_factors, 1d_factors]`
4. Genome 兼容迁移 (feat_offset: 25 → 75)
5. 性能优化（缓存、计算优化）

**依赖**: P2 LLM Oracle 必须正常工作（75 维搜索空间需要引导）

---

## 八、附录：常用命令速查

```bash
# 项目目录
cd /Users/tomxiao/Desktop/Git/personal/HermesFlow

# 查看服务状态
docker compose ps

# 查看策略生成器日志
docker compose logs -f strategy-generator

# 查询数据库
docker compose exec timescaledb psql -U postgres -d hermesflow -c "SELECT ..."

# 前端开发
make web-dev

# 构建 & 测试
make lint
make test
make build

# Tmux 控制 (Claude Code 会话)
tmux attach -t 0          # 进入会话
tmux capture-pane -p      # 捕获输出
tmux send-keys -t 0 "..." Enter   # 发送命令
```

---

**规则版本**: v1.0  
**最后更新**: 2026-02-22  
**下次审查**: Phase 3 完成后
