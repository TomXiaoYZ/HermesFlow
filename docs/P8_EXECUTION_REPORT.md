# P8 执行报告

**日期**: 2026-03-01
**状态**: COMPLETED
**分支**: main
**提交范围**: `9bd724d..550c223` (6 commits)

---

## 0. 摘要

P8 全部 5 个阶段已完成实施、测试通过、Docker 部署验证、推送至 main。共计修改 13 个源文件，新增 1,188 行、删除 256 行代码，新增 14 个单元测试。全部 485 个工作区测试通过，Clippy 零警告。

---

## 1. 提交记录

| 序号 | Commit | 类型 | 描述 |
|------|--------|------|------|
| 1 | `9bd724d` | docs | 整合 Gemini 第二轮评审至 P8 架构设计 |
| 2 | `4b66775` | feat | Phase 0: LLM-Guided MCTS Policy Prior |
| 3 | `0867bcf` | feat | Phase 1: CCIPCA Active Token Remapping |
| 4 | `1eb9d63` | feat | Phase 2: ALPS Diversity-Triggered Injection |
| 5 | `3eff1a5` | perf | Phase 3: VM Hot Path Optimization |
| 6 | `550c223` | feat | Phase 4: sqlx 0.8 + Decimal Precision |

---

## 2. 各阶段实施详情

### Phase 0: LLM-Guided MCTS Policy Prior [HIGHEST]

**目标**: 将盲目的 MCTS 搜索升级为语义引导搜索。

| 子阶段 | 文件 | 变更内容 |
|--------|------|---------|
| P8-0A | `backtest/factor_importance.rs` | 新增 `bottom_n_summary()` 返回 Bottom-10 噪音因子 |
| P8-0B | `mcts/policy.rs` | `build_llm_prior_weights()` — 基于因子重要性 + 精英算子统计构建非均匀先验权重 |
| P8-0C | `mcts/policy.rs` | `canonicalize_rpn()` — 交换律算子（ADD/MUL/TS_CORR）操作数排序，提升缓存命中 |
| P8-0D | `mcts/policy.rs` | `LlmCachedPolicy` 接入先验权重，替代均匀随机策略 |
| P8-0E | `main.rs` | `UniformPolicy` → `LlmCachedPolicy` 替换；每 500 代重算因子重要性 |

**新增测试**: 6 个（bottom_n_summary, canonicalize ×3, build_prior_weights ×2, populate_cache ×1）

**部署验证**: LLM oracle 日志确认注入生效（10/10 valid genomes across symbols）。

### Phase 1: CCIPCA Active Token Remapping [HIGH]

**目标**: 激活沉睡的 CCIPCA 降维能力，将 75 维特征空间扩展至 80 维。

| 子阶段 | 文件 | 变更内容 |
|--------|------|---------|
| P8-1A | `backtest/incremental_pca.rs` | `project_features()` — 将特征张量投影到 PC 空间，追加 k 个 PC 列 |
| P8-1B | `main.rs` | CCIPCA 达 200 观测后自动增广：`feat_offset 75→80`，更新 VM/GA/缓存 |
| P8-1C | `main.rs` | PC 因子命名（`PC0_var=X.XX%`），追加到 `factor_names` |

**新增测试**: 3 个（output_shape, preserves_original, pc_values_match_transform）

**部署验证**: 日志确认 `P8-1B: CCIPCA augmentation: 75→80 features (PC0..PC4)` 跨所有 symbol 生效。

### Phase 2: ALPS Diversity-Triggered Injection [HIGH]

**目标**: 从被动多样性日志升级为主动干预闭环。

| 子阶段 | 文件 | 变更内容 |
|--------|------|---------|
| P8-2A | `main.rs` | `DiversityTriggerConfig` 配置结构 + `generator.yaml` 配置 |
| P8-2B | `main.rs`, `genetic.rs` | L3/L4 Hamming 多样性低于阈值时触发：随机注入 L0 + 精英替换（淘汰最弱 10%） |
| P8-2C | `genetic.rs` | `layer_size()`, `generate_random_genomes()`, `cull_weakest()` 方法 |

**新增测试**: 4 个（layer_size, generate_random_genomes, cull_weakest, cull_weakest_out_of_range）

**部署验证**: 多样性监控日志确认 `diversity (Hamming): [L0:0.94, L1:0.98, ...]`，触发逻辑就绪。当前多样性健康（>0.80），无需紧急注入。

### Phase 3: VM Hot Path Optimization [MEDIUM]

**目标**: 形状守卫 + 条件 NaN 清洗 + O(n) 滑动窗口算子。

| 子阶段 | 文件 | 变更内容 |
|--------|------|---------|
| P8-3A | `vm/vm.rs` | 预执行形状守卫：验证所有 feature token 索引在 `n_features` 范围内 |
| P8-3B | `vm/ops.rs` | `ts_mean` / `ts_sum` 重写为 O(n) 滑动窗口（原 O(n·d) delay 累加） |
| P8-3C | `vm/vm.rs` | NaN 清洗从无条件改为条件触发：仅 DIV(3)/SIGNED_POWER(8)/TS_CORR(16)/LOG(19)/SQRT(20) |

**性能影响**: `ts_mean(x, 20)` 从 20 次 `ts_delay` 分配 → 1 次遍历。NaN 清洗跳过 ADD/MUL/SUB 等安全算子。

### Phase 4: sqlx 0.8 迁移 + 金融精度强化 [MEDIUM]

**目标**: 修复安全漏洞 + Decimal 精度 + 移除临时门控。

| 子阶段 | 文件 | 变更内容 |
|--------|------|---------|
| P8-4A | `Cargo.toml` | sqlx `0.7.4` → `0.8.6`，feature 重命名 `runtime-tokio-rustls` → `runtime-tokio` + `tls-rustls` |
| P8-4B | `ensemble_weights.rs` | HRP 权重、PSR 因子、拥挤惩罚、换手成本、Deadzone/L1 内部计算全部转为 `rust_decimal::Decimal` |
| P8-4C | `main.rs` | 移除 P7-3B 16MiB 负载大小门控（sqlx 0.8 协议级修复 RUSTSEC-2024-0363） |

**安全修复**: RUSTSEC-2024-0363（sqlx 长度前缀截断溢出）已从根本修复。

---

## 3. Gemini 第二轮评审处置记录

| Gemini 建议 | 决策 | 实施情况 |
|------------|------|---------|
| 0B: Top-10 + Bottom-10 因子注入 prompt | **采纳** | `bottom_n_summary()` + 先验权重构建 |
| 0C: AST 规范化哈希 | **部分采纳** | 轻量 canonical RPN 排序（非 AST），`canonicalize_rpn()` |
| 1: 禁止 PC 追加，双轨变异 | **拒绝** | 技术反驳已记录；append 方案验证成功（75→80） |
| 2B: 精英替换注入 | **采纳** | `cull_weakest()` 淘汰 L0 最弱 10% |
| 3B: unsafe uget() | **拒绝** | 无性能瓶颈证据，违反安全规则 |
| 4B: 移除 16MiB 限流 | **采纳** | P8-4C 实施完毕 |

---

## 4. 测试结果

| 测试集 | 数量 | 状态 |
|--------|------|------|
| backtest-engine | 73 | PASS |
| strategy-generator | 268 | PASS |
| data-engine | 125 | PASS |
| common | 11 | PASS |
| gateway | 3 | PASS |
| strategy-engine | 2 | PASS |
| 其他（doc-tests 等） | 3 | PASS |
| **工作区总计** | **485** | **ALL PASS** |

P8 新增测试: **14 个** (Phase 0: 6, Phase 1: 3, Phase 2: 4, Phase 3: 0 新增/2 已有修改, Phase 4: 0 新增/22 已有通过)

---

## 5. 部署验证

| 检查项 | 结果 |
|--------|------|
| `cargo clippy --workspace -- -D warnings` | PASS（零警告） |
| `cargo test --workspace` | PASS（485/485） |
| `cargo build` (execution-engine 独立) | PASS |
| Docker build: strategy-generator | PASS |
| Docker build: data-engine | PASS |
| Docker build: gateway | PASS |
| Docker build: strategy-engine | PASS |
| 容器重启 + 健康检查 | 4/4 healthy |
| strategy-generator API (`/exchanges`) | 正常响应 |
| CCIPCA 增广日志 (`75→80 features`) | 已确认 |
| LLM oracle 注入日志 | 已确认（10/10 valid） |
| 多样性监控日志 | 已确认（Hamming 指标正常） |
| 进化引擎运行 | 正常（多 symbol 并行进化中） |

---

## 6. 代码统计

| 指标 | 数值 |
|------|------|
| 修改文件数 | 13 |
| 新增行数 | +1,188 |
| 删除行数 | -256 |
| 净增行数 | +932 |
| 新增测试数 | 14 |
| Commits | 6 (含 1 docs) |

### 按阶段明细

| 阶段 | 文件数 | 新增 | 删除 |
|------|--------|------|------|
| Phase 0 | 5 | +478 | -29 |
| Phase 1 | 2 | +224 | -38 |
| Phase 2 | 3 | +192 | -2 |
| Phase 3 | 2 | +103 | -47 |
| Phase 4 | 5 | +192 | -141 |

---

## 7. 成功标准达成

| 指标 | P7 基线 | P8 目标 | 实际状态 |
|------|---------|---------|---------|
| MCTS 种子策略 | 盲搜（UniformPolicy） | 语义引导（LlmCachedPolicy） | **已实施** — 因子重要性 + 精英算子先验 |
| MCTS 缓存 | 无 | canonical hash 去重 | **已实施** — 交换律规范化 |
| 高龄层多样性闭环 | 被动日志 | 主动检测 + 干预 | **已实施** — L3/L4 阈值触发 + 精英替换 |
| VM 形状安全 | 运行时越界风险 | 预执行守卫 | **已实施** — feature token 范围校验 |
| VM 算子性能 | O(n·d) ts_mean/ts_sum | O(n) 滑动窗口 | **已实施** |
| sqlx 安全漏洞 | RUSTSEC-2024-0363 | 修复 | **已修复** — sqlx 0.8.6 |
| 执行路径精度 | f64 (~15 位) | Decimal (28 位) | **已实施** — ensemble_weights 全链路 |
| 16MiB 临时门控 | 存在 | 移除 | **已移除** |

---

## 8. 推迟至 P9

| 项目 | 原因 |
|------|------|
| `unsafe uget()` 无边界检查 | 无性能瓶颈证据，违反安全规则。需 flamegraph 证实后再议 |
| 完整 AST 规范化哈希 | 当前 canonical RPN 足够。需 ~300 LOC AST 基础设施 |
| 全局 f64→Decimal (VM 内部) | ndarray 不支持 Decimal，需重写 VM 核心 |

---

## 9. 已知限制

1. **CCIPCA 方差比率初始为零**: CCIPCA 在首次增广时 top-5 explained variance ratios 全部为 0.0（冷启动），随时间累积观测后收敛。
2. **多样性触发尚未实际触发**: 当前所有 symbol 的 L3/L4 多样性 >0.80（远高于 0.25/0.20 阈值），触发逻辑就绪但未实战验证。
3. **redis v0.24.0 future-incompat 警告**: `redis` crate 存在未来 Rust 版本兼容性警告，计划在 P9 升级。
