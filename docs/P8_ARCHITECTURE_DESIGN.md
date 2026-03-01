# P8 架构设计：语义先验 MCTS、主动降维与多样性闭环

**日期**: 2026-03-01
**状态**: DESIGN
**前置条件**: P7 (Statistical Barriers + MCTS Integration) 已部署并验证

---

## 0. 摘要

P8 基于 Gemini 顾问对 P7 交付物的审查意见，聚焦五个核心改进方向。经过代码验证和事实核查，我们确认其中四项建议有效（一项存在事实错误），并据此设计了五阶段实施方案。

**2026-03-01 更新**: 整合 Gemini 第二轮评审（P7阶段架构分析与评审）的有效建议。经逐项核查，采纳 4 项建议（0B 扩展因子注入、0C canonical RPN hash、2B 精英替换、4C 移除限流门控），拒绝 2 项建议（Phase 1 空间隔离、Phase 3 unsafe uget）。五阶段结构不变，总工期从 11 天调整为 12 天。

**核心目标**：将盲目的 MCTS 搜索升级为语义引导搜索，激活沉睡的降维能力，并构建从被动监控到主动干预的多样性闭环。

### 交付物概览

| 阶段 | 名称 | 优先级 | 核心改进 |
|------|------|--------|----------|
| Phase 0 | LLM-Guided MCTS Policy Prior | HIGHEST | 盲目搜索 → 语义引导搜索 |
| Phase 1 | CCIPCA Active Token Remapping | HIGH | 75维特征空间降维去相关 |
| Phase 2 | ALPS Diversity-Triggered Injection | HIGH | 被动日志 → 主动干预闭环 |
| Phase 3 | VM Hot Path Optimization | MEDIUM | 形状守卫 + 条件清洗优化性能 |
| Phase 4 | sqlx 0.8 迁移 + 金融精度强化 | MEDIUM | 安全修复 + Decimal 精度 |

---

## 1. Gemini 审查意见事实核查

Gemini 顾问在审查 P7 交付物后提出了 5 项建议。我们逐一核实了代码库现状：

| # | Gemini 建议 | 核查结果 | 依据 |
|---|------------|---------|------|
| 1 | LLM Prior for MCTS (替换 UniformPolicy) | **有效** | `main.rs:1055` 确认使用 UniformPolicy。LlmCachedPolicy 已实现但未接入。 |
| 2 | CCIPCA Active Token Remapping | **有效** | `incremental_pca.rs` 仅用于诊断。`FOLDER_INDEX.md:48` 明确推迟至 P8。 |
| 3 | VM Shape Guards & unsafe uget() | **部分有效** | VM 已有 `token < feat_offset` 守卫。未发现运行时 panic 证据。性能优化角度有价值，但风险被高估。 |
| 4 | ALPS Diversity Trigger | **有效** | `main.rs:1146` 每 50 代计算多样性但仅记录日志，无任何反馈动作。 |
| 5 | Actix→Axum 统一 + f64→Decimal | **Actix 部分事实错误** | 代码库中零 Actix 引用（全部使用 Axum）。f64→Decimal 仅在执行路径部分有效。 |

### 1.1 Actix-web 事实错误澄清

Gemini 报告中声称 "strategy-generator 服务异常挂载于 Actix-web 框架"。**这与事实不符。**

**代码证据** — `services/strategy-generator/src/api.rs`:
```rust
let app = Router::new()
    .route("/exchanges", get(list_exchanges))
    .route("/:exchange/config/factors", get(get_factor_config))
    // ... more routes
    .with_state(state);

let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
axum::serve(listener, app).await.unwrap();
```

**验证**：
- `grep -r "actix" services/strategy-generator/` → 零匹配
- `Cargo.toml` 无 actix 依赖
- 所有 HermesFlow 服务统一使用 Axum 0.7，无需框架统一迁移

### 1.2 第二轮评审核查（P7阶段架构分析与评审）

Gemini 在审阅 P7 交付物后提交了第二轮深度评审报告，对 P8 五个 Phase 提出了具体实施建议。逐项核查结果如下：

| Gemini 建议 | 采纳状态 | 理由 |
|------------|---------|------|
| 0C: AST 规范化哈希 | **部分采纳** | 无 AST 基础设施（~200-300 LOC 新增），改用轻量 canonical RPN 排序；仅排序 ADD/MUL/TS_CORR 操作数 |
| 0B: Top-10/Bottom-10 因子 | **采纳** | 增强 LLM prompt 质量，提供正反两面因子信息 |
| 1: 禁止 PC 追加，双轨变异 | **拒绝** | GA 不计算协方差矩阵，VM 栈模型将特征视为不透明数组。双轨变异破坏均匀抽象，增加热路径分支，违反"变异盲目"原则。P3 多时间框架拼接已证明 append 模式可行。详见 4.1.1 |
| 2B: 精英替换注入 | **采纳** | 比纯随机注入更有效打破僵局，抹杀 L0 最低 10% 后注入 MCTS/Oracle 种子 |
| 3B: unsafe uget() | **拒绝** | 每公式仅 2 次 `index_axis()`，非每 time-step。LTO+opt-level=3 已启用。违反 CLAUDE.md "No unsafe without documented justification"。详见 6.0 |
| 4B: 移除 16MiB 限流 | **采纳** | sqlx 0.8 从根本修复 RUSTSEC-2024-0363，P7 临时门控可安全移除。新增 P8-4C |

---

## 2. 当前架构基线 (P7)

### 2.1 演化循环数据流

```
mkt_equity_candles (1h/4h/1d)
  │
  └─ FeatureEngineer::compute_features_from_config()
       │
       └─ Array3<f64> (1, 75, T)
            │                                         ┌─ CCIPCA: 诊断日志 (被动)
            ├─ 每代: parallel K-fold backtest ─────── ┤
            │                                         └─ Diversity: 日志 (被动)
            │
            ├─ 每 50 代: MCTS 种子注入 (UniformPolicy) ──→ ALPS L0
            │
            └─ 触发时: LLM Oracle ──→ ALPS L0
```

### 2.2 关键组件现状

| 组件 | 文件 | 当前状态 | P8 目标 |
|------|------|---------|---------|
| UniformPolicy | `mcts/policy.rs:20-31` | 生产使用中 | 替换为 LlmCachedPolicy |
| LlmCachedPolicy | `mcts/policy.rs:88-170` | `#[allow(dead_code)]` | 激活并接入 |
| HeuristicPolicy | `mcts/policy.rs:35-78` | `#[allow(dead_code)]` | 作为 LLM 失败回退 |
| FactorImportance | `backtest/factor_importance.rs` | `#[allow(dead_code)]` | 接入演化循环 |
| CcipcaState | `backtest/incremental_pca.rs` | 仅诊断日志 | 主动特征投影 |
| layer_diversity() | `main.rs:1144-1156` | 仅 `info!()` 日志 | 触发 MCTS/Oracle |

### 2.3 Token 布局 (不变)

```
Token:    0────24  25────49  50────74  │  75  76  77  ...  97
含义:     ─1h──── ──4h───── ──1d───── │ ── 23 operators (14 active) ──
          feat_offset = 75
```

P8 在 Phase 1 中将扩展至 feat_offset = 80（新增 5 个 PC 特征）。

---

## 3. Phase 0: LLM-Guided MCTS Policy Prior [HIGHEST]

### 3.0 动机

当前 MCTS 使用 `UniformPolicy`（对所有合法 token 分配等概率），这意味着搜索是盲目的——搜索引擎不知道哪些因子组合更有可能产生有意义的金融信号。

P7 已实现 `LlmCachedPolicy`（带 HashMap 缓存的 LLM 先验策略）和 `FactorImportance`（排列重要性计算），但两者均标记为 dead_code。Phase 0 将它们串联起来，形成完整的语义先验管道：

```
Factor Importance ──→ LLM Prompt ──→ Token Weights ──→ LlmCachedPolicy ──→ MCTS
   (每 500 代)         (带上下文)      (非均匀先验)         (缓存查找)       (语义搜索)
```

### 3.1 目标架构

```
┌──────────────────────────────────────────────────────────────────────┐
│                        Evolution Loop                                │
│                                                                      │
│  ┌─────────────────────────────┐                                    │
│  │  Factor Importance Cache     │   HashMap<String, Vec<FI>>         │
│  │  每 500 代 per symbol 重算   │   key = symbol                     │
│  └──────────────┬──────────────┘                                    │
│                 │                                                     │
│                 ▼                                                     │
│  ┌─────────────────────────────┐                                    │
│  │  build_llm_prior_weights()   │                                    │
│  │  1. Boost 重要因子 token     │   importance * boost_scale         │
│  │  2. Boost 精英公式中的算子   │   出现次数 * op_boost              │
│  │  3. 输出: Vec<f64> (vocab)  │                                    │
│  └──────────────┬──────────────┘                                    │
│                 │                                                     │
│                 ▼                                                     │
│  ┌─────────────────────────────┐                                    │
│  │  LlmCachedPolicy            │                                    │
│  │  cache: HashMap<u64, Vec>    │   key = FNV1a(tokens, stack_depth) │
│  │  命中 → 非均匀先验           │   未命中 → 退化为均匀分布          │
│  └──────────────┬──────────────┘                                    │
│                 │                                                     │
│                 ▼                                                     │
│  ┌─────────────────────────────┐                                    │
│  │  run_mcts_round()            │                                    │
│  │  policy.prior() 指导扩展     │   重要因子更可能被选择             │
│  │  seeds → inject ALPS L0     │                                    │
│  └─────────────────────────────┘                                    │
│                                                                      │
└──────────────────────────────────────────────────────────────────────┘
```

### 3.2 实施细节

#### P8-0A: Factor Importance 生产接入

**文件**: `strategy-generator/src/backtest/factor_importance.rs`, `strategy-generator/src/main.rs`

**变更**:
1. 移除 `compute_permutation_importance()` 和 `FactorImportance` 的 `#[allow(dead_code)]` 注解
2. 在演化循环中添加重要性计算（每 500 代，per symbol）
3. 结果缓存至 `HashMap<String, Vec<FactorImportance>>`，key 为 symbol

**代码位置** — `main.rs` 演化循环内，`gen.is_multiple_of(importance_interval)` 分支：

```rust
// P8-0A: Recompute factor importance every N gens
if gen > 0 && gen.is_multiple_of(importance_interval) {
    if let Some(best) = ga.best_genome.clone() {
        let fi = backtest::factor_importance::compute_permutation_importance(
            &backtester, &best, &symbol, k, mode, &factor_names,
        );
        let summary = backtest::factor_importance::top_n_summary(&fi, 10);
        info!(
            "[{}:{}:{}] Gen {} factor importance: {:?}",
            exchange, symbol, mode_str, gen, summary
        );
        importance_cache.insert(symbol.clone(), fi);
    }
}
```

**性能开销**: N+1 次 backtest（N = feat_offset = 75），但仅每 500 代执行一次。以每代 ~2ms 的评估速度计算：76 × 2ms = ~152ms / 500 代 = 0.3ms 均摊开销，可忽略。

#### P8-0B: 增强 LLM Prompt 添加因子上下文

**文件**: `strategy-generator/src/llm_oracle.rs`

**变更**:
1. 扩展 `OracleContext` 添加 `factor_importance: Option<Vec<FactorImportance>>` 字段
2. 在 `build_prompt()` 中新增 "Top Important Factors" 段落

**Prompt 新增段落**（第二轮评审后扩展为 Top-10 + Bottom-10）:
```
## Top Important Factors (by PSR impact)
The following 10 factors have the highest impact on strategy fitness:
1. return_1h (index 0): 0.45 PSR drop when shuffled — most impactful
2. atr_pct_4h (index 30): 0.38 PSR drop — volatility regime signal
3. momentum_1d (index 50): 0.31 PSR drop — daily trend strength
4. volume_ratio_1h (index 3): 0.22 PSR drop — liquidity signal
5. bb_width_4h (index 34): 0.18 PSR drop — Bollinger squeeze
6-10: [Next 5 factors with PSR drop values]

## Low-Impact Factors (avoid these)
The following 10 factors add noise without meaningful signal:
1-10: [Bottom-10 factors with PSR drop < 0.05]

Generate formulas that primarily combine top factors.
Explicitly avoid bottom factors as they add noise.
```

**效果**: LLM 同时获得正面（高重要性）和反面（噪音）因子信息，生成的公式质量更高。原 Top-5 扩展为 Top-10 重要因子 + Bottom-10 噪音因子，基于 Gemini 第二轮评审建议。

#### P8-0C: Canonical RPN Hash（第二轮评审新增）

**文件**: `strategy-generator/src/mcts/policy.rs`（`LlmCachedPolicy::hash_state()`）

**背景**: Gemini 建议使用 AST 规范化哈希（解析 RPN→AST，交换律子节点排序）来提升缓存命中率。经评估，代码库无 AST 基础设施（需新增 ~200-300 LOC），且在 MCTS 紧密循环中 AST 构建的 O(n) 开销可能影响延迟。采用轻量替代方案：在哈希前对 RPN token 序列中交换律算子的操作数按 token 值排序，形成 canonical 形式。

**核心思路**: 对交换律二元算子（ADD=0, MUL=2, TS_CORR=16），确保 `left_operand <= right_operand`，使数学等价表达式产生相同 hash。

```rust
/// Normalize commutative binary ops in RPN before hashing.
/// For ADD/MUL/TS_CORR: ensure left_operand <= right_operand.
fn canonicalize_tokens(tokens: &[u32], feat_offset: u32) -> Vec<u32> {
    let mut result = tokens.to_vec();
    let commutative = [0u32, 2, 16]; // ADD, MUL, TS_CORR
    for i in 0..result.len() {
        let op = result[i];
        if op >= feat_offset && commutative.contains(&(op - feat_offset)) {
            // Find the two operand sub-expressions and sort
            if let Some((left_start, right_start)) = find_binary_operands(&result, i) {
                if result[left_start] > result[right_start] {
                    // Swap operand sub-trees
                    swap_subtrees(&mut result, left_start, right_start, i);
                }
            }
        }
    }
    result
}
```

**集成**: 修改 `hash_state()` 在 FNV1a 哈希前先调用 `canonicalize_tokens()`：
```rust
fn hash_state(&self, tokens: &[u32], stack_depth: u32) -> u64 {
    let canonical = canonicalize_tokens(tokens, self.feat_offset);
    let mut hasher = FnvHasher::default();
    for &t in &canonical {
        hasher.write_u32(t);
    }
    hasher.write_u32(stack_depth);
    hasher.finish()
}
```

**效果**: 数学等价表达式（如 `A B ADD` 和 `B A ADD`）产生相同 hash，提升缓存命中率。

**不建 AST 的原因**: 代码库无 AST 基础设施（~200-300 LOC 新增），且在 MCTS 紧密循环中 AST 构建的 O(n) 开销可能影响延迟。轻量 canonical RPN 排序在 O(n) 内完成且无额外内存分配（原地排序）。

#### P8-0D: LlmCachedPolicy 缓存填充

**文件**: `strategy-generator/src/mcts/policy.rs` (新增辅助函数), `strategy-generator/src/main.rs`

**新增函数** — `build_llm_prior_weights()`:

```rust
/// Build prior weights for LlmCachedPolicy from factor importance and elite tokens.
///
/// Weight construction:
/// 1. Base weight = 1.0 for all tokens
/// 2. Important factors: weight += importance * boost_scale
/// 3. Elite operators: weight *= 1.0 + op_boost_scale (per occurrence)
/// 4. Factors below min_threshold: weight unchanged (stays at 1.0)
fn build_llm_prior_weights(
    importance: &[FactorImportance],
    elite_tokens: &[Vec<usize>],
    vocab_size: usize,
    feat_offset: usize,
    config: &LlmMctsPriorConfig,
) -> Vec<f64> {
    let mut weights = vec![1.0; vocab_size];

    // Boost important factors
    for fi in importance {
        if fi.importance >= config.min_importance_threshold && fi.factor_index < feat_offset {
            weights[fi.factor_index] += fi.importance * config.importance_boost_scale;
        }
    }

    // Boost operators from elite formulas
    let mut op_counts = vec![0usize; vocab_size];
    for tokens in elite_tokens {
        for &t in tokens {
            if t >= feat_offset && t < vocab_size {
                op_counts[t] += 1;
            }
        }
    }
    for (i, &count) in op_counts.iter().enumerate() {
        if count > 0 {
            weights[i] *= 1.0 + config.operator_boost_scale * (count as f64).min(5.0);
        }
    }

    weights
}
```

**缓存填充策略**:
- 对精英基因组的每个前缀（prefix）生成缓存条目
- 例如精英序列 `[0, 25, 75]`（return_1h, return_4h, ADD）:
  - prefix `[]` + stack_depth 0 → 完整权重向量
  - prefix `[0]` + stack_depth 1 → 权重向量
  - prefix `[0, 25]` + stack_depth 2 → 权重向量（偏向二元算子）
- 典型缓存大小: 50-200 条目（受限于精英数量 × 公式长度）

#### P8-0E: 替换 UniformPolicy

**文件**: `strategy-generator/src/main.rs:1051-1117`

**当前代码**:
```rust
let policy = mcts::policy::UniformPolicy;
```

**替换为**:
```rust
let policy: Box<dyn mcts::policy::Policy> = if let Some(importance) = importance_cache.get(&symbol) {
    let mut llm_policy = mcts::policy::LlmCachedPolicy::new(vocab_size);
    let elite_tokens: Vec<Vec<usize>> = ga.top_n_genomes(5)
        .iter()
        .map(|g| g.tokens.clone())
        .collect();
    populate_policy_cache(
        &mut llm_policy, importance, &elite_tokens,
        vocab_size, feat_offset, &llm_prior_config,
    );
    info!(
        "[{}:{}:{}] Gen {} MCTS: using LLM prior (cache={} entries)",
        exchange, symbol, mode_str, gen, llm_policy.cache_size()
    );
    Box::new(llm_policy)
} else {
    Box::new(mcts::policy::UniformPolicy)
};
```

**兼容性**: 首 500 代（尚无 importance 数据时）自动退化为 UniformPolicy。

#### P8-0F: 配置

**文件**: `config/generator.yaml`

```yaml
# P8-0: LLM-guided MCTS prior
# Transforms blind MCTS exploration into semantically guided search
# by biasing action selection toward factors identified as important.
llm_mcts_prior:
  enabled: true
  importance_recompute_interval: 500  # Gens between importance recalculation
  importance_boost_scale: 0.5         # Weight boost per unit of PSR drop
  operator_boost_scale: 0.2           # Weight boost for elite-observed operators
  min_importance_threshold: 0.05      # Skip factors below this PSR drop
```

### 3.3 验证计划

1. **单元测试**: `build_llm_prior_weights()` 输出权重归一化正确
2. **集成测试**: importance_cache 在 500 代后非空
3. **A/B 对比**: 相同 symbol 运行 1000 代：UniformPolicy vs LlmCachedPolicy
4. **指标**: MCTS 种子的平均 PSR 应提升（盲目搜索 → 语义搜索）
5. **日志**: 确认 LLM 先验缓存命中率 > 50%

---

## 4. Phase 1: CCIPCA Active Token Remapping [HIGH]

### 4.0 动机

当前 75 维特征空间（25 因子 × 3 时间框架）存在显著共线性——同一因子在不同时间框架上高度相关（如 `return_1h` 和 `return_4h` 相关系数通常 > 0.7）。这导致：

1. **GA 冗余搜索**: 不同的 token 组合产生几乎相同的信号
2. **MCTS 效率低下**: 搜索树在等价分支上浪费预算
3. **过拟合风险**: 公式包含冗余因子时更易过拟合

P7 已部署 CCIPCA (Covariance-free Incremental PCA) 用于诊断监控。Phase 1 将其升级为主动特征投影——将 75 维数据投影到 5 个主成分轴上，作为新的一等特征提供给 GA 和 MCTS。

### 4.1 目标架构

```
原始特征 Array3 (1, 75, T)
  │
  ├─ token 0-74: 原始 75 个因子 (不变)
  │
  └─ CCIPCA components() → W: (5, 75)
       │
       └─ PC features = W × centered_features → (5, T)
            │
            └─ 拼接 → Array3 (1, 80, T)
                 │
                 token 75-79: PC0 ~ PC4 (新增)
                 feat_offset: 75 → 80
                 算子起始位: 80+
```

**设计决策**: 采用"增量扩展"而非"替换映射"，保持原始 75 个因子不变，新增 5 个 PC 特征。

**理由**:
- 零向后兼容问题（现有基因组 token 0-74 仍然有效）
- GA 可以自然发现 PC 特征的价值（如果有用，进化会选择它们）
- 如果 PC 特征无用，不会影响现有性能

#### 4.1.1 设计决策：为什么拒绝"空间隔离 + 双轨变异"

Gemini 第二轮评审建议禁止将 PC 追加到原始特征后，理由是"混用会使协方差矩阵陷入奇异状态，引发适应度景观欺骗"。经代码验证，该论点不成立：

1. **GA 不计算协方差矩阵** — 适应度由 PSR（Walk-Forward OOS 评估）决定，不涉及特征空间的协方差。
2. **VM 栈模型将特征视为不透明数组** — `index_axis(Axis(1), token)` 对所有 feature token 一视同仁，无"空间"概念。
3. **P3 先例** — 多时间框架拼接（25×3=75）已证明 append 模式对 VM/GA 完全兼容，且无共线性问题。
4. **双轨变异破坏核心抽象** — RPN 栈 VM 的优雅之处在于"features are features, operators are operators, mutation is blind"。添加空间标记违反此原则。
5. **热路径性能** — 双轨方案在 VM `execute()` 中添加额外分支判断，增加分支预测失败开销。

### 4.2 实施细节

#### P8-1A: CCIPCA Feature Projection

**文件**: `strategy-generator/src/backtest/incremental_pca.rs`, `strategy-generator/src/main.rs`

**新增方法** — `CcipcaState::project_features()`:

```rust
/// Project a full feature tensor onto PC space, appending PC columns.
///
/// Input: Array3<f64> shape (1, n_feat, T) — original features
/// Output: Array3<f64> shape (1, n_feat + k, T) — augmented with k PC features
///
/// Each PC value at time t: pc_i(t) = W_i · centered_features(:, t)
/// where W_i is the i-th normalized eigenvector from CCIPCA.
pub fn project_features(&self, features: &Array3<f64>) -> Array3<f64> {
    let n_feat = features.shape()[1];
    let n_bars = features.shape()[2];
    let k = self.config.n_components;

    let mut augmented = Array3::zeros((1, n_feat + k, n_bars));

    // Copy original features
    for f in 0..n_feat {
        for t in 0..n_bars {
            augmented[[0, f, t]] = features[[0, f, t]];
        }
    }

    // Compute and append PC features
    let components = self.components(); // (k, n_feat) normalized
    for t in 0..n_bars {
        let obs: Array1<f64> = Array1::from_iter(
            (0..n_feat).map(|f| features[[0, f, t]])
        );
        let centered = &obs - &self.mean;
        let pc_values = components.dot(&centered); // (k,)
        for i in 0..k {
            augmented[[0, n_feat + i, t]] = pc_values[i];
        }
    }

    augmented
}
```

#### P8-1B: Feature Tensor Augmentation

**文件**: `strategy-generator/src/main.rs`, `strategy-generator/src/backtest/mod.rs`

**演化循环集成**:

```rust
// P8-1B: After CCIPCA warmup, augment feature tensor with PC columns
let (effective_features, effective_feat_offset) = if let Some(ref pca) = ccipca {
    if pca.is_valid() && pca.n_observations() > 100 {
        let augmented = pca.project_features(&cached_features);
        let new_offset = feat_offset + pca.config.n_components; // 75 + 5 = 80
        info!(
            "[{}:{}:{}] CCIPCA augmentation: {}→{} features",
            exchange, symbol, mode_str, feat_offset, new_offset
        );
        (augmented, new_offset)
    } else {
        (cached_features.clone(), feat_offset)
    }
} else {
    (cached_features.clone(), feat_offset)
};
```

**注意事项**:
- Backtester 缓存需要以 augmented features 重建
- GA 的 `vocab_size` 需要相应调整（从 75+23 到 80+23）
- MCTS 的 `ActionSpace` 需使用 `effective_feat_offset`

#### P8-1C: PC Factor Names for LLM Oracle

**文件**: `strategy-generator/src/llm_oracle.rs`

扩展因子词汇表以包含 PC 特征:

```rust
// Append PC feature names when CCIPCA is active
if let Some(ref pca) = ccipca {
    if pca.is_valid() {
        let ev = pca.explained_variance();
        let total: f64 = ev.iter().sum();
        for (i, &v) in ev.iter().enumerate() {
            let ratio = if total > 0.0 { v / total } else { 0.0 };
            factor_names.push(format!("PC{}_var_{:.2}", i, ratio));
        }
    }
}
```

LLM 将理解这些是正交特征组合，可以生成直接引用 PC 特征的公式：
```
PC0_var_0.35 PC2_var_0.12 MUL  → 结合两个主成分的交互信号
return_1h PC0_var_0.35 SUB     → 原始因子与主成分的偏差
```

### 4.3 验证计划

1. **单元测试**: `project_features()` 输出形状正确 `(1, 80, T)`
2. **数值测试**: PC 投影值与手动矩阵乘法一致
3. **回归测试**: 仅使用 token 0-74 的现有基因组适应度不变
4. **收敛测试**: 运行 2000 代后，观察 GA 是否自然选择 PC token (75-79)
5. **共线性测试**: 验证 PC 特征与原始因子的相关系数 < 0.3

---

## 5. Phase 2: ALPS Diversity-Triggered Injection [HIGH]

### 5.0 动机

P7 在 `main.rs:1144-1156` 实现了 Hamming 距离多样性监控：

```rust
// P7-5B: Log genome diversity every 50 gens
if gen.is_multiple_of(50) {
    let diversity = ga.layer_diversity();
    let div_strs: Vec<String> = diversity
        .iter()
        .map(|(i, n, d)| format!("L{}:{:.2}(n={})", i, d, n))
        .collect();
    info!(
        "[{}:{}:{}] Gen {} diversity (Hamming): [{}]",
        exchange, symbol, mode_str, gen, div_strs.join(", ")
    );
}
```

**问题**: 这是纯被动监控。当 L3/L4（高龄层，age 89/500）多样性跌破阈值时，意味着该层种群已趋同——进化陷入局部最优。当前系统对此无反应，错失了最佳干预时机。

### 5.1 目标架构

```
┌─────────────────────────────────────────────────────────────┐
│                   Diversity Feedback Loop                     │
│                                                               │
│  layer_diversity() ──→ 检测 L3/L4 低多样性                    │
│        │                                                      │
│        ▼                                                      │
│  ┌──────────────┐    YES    ┌──────────────────────────┐     │
│  │ 低于阈值?    │──────────→│ Emergency Injection:      │     │
│  │ L3 < 0.25    │           │  1. 强制 MCTS 轮次       │     │
│  │ L4 < 0.20    │           │  2. 强制 Oracle 调用     │     │
│  └──────┬───────┘           │  3. 注入新随机基因组      │     │
│         │ NO                 └──────────────────────────┘     │
│         ▼                                                     │
│  正常演化继续                                                  │
│                                                               │
│  冷却期: 100 代 (防止过度注入)                                 │
└─────────────────────────────────────────────────────────────┘
```

### 5.2 实施细节

#### P8-2A: Diversity Trigger 配置

**文件**: `config/generator.yaml`

```yaml
# P8-2: Diversity-triggered emergency injection
# Converts passive Hamming diversity logging into active trigger
# for MCTS and Oracle when high-age layers converge excessively.
diversity_trigger:
  enabled: true
  min_diversity_l3: 0.25       # L3 (age 89) threshold
  min_diversity_l4: 0.20       # L4 (age 500) threshold
  trigger_action: "mcts_and_oracle"  # "mcts_only" | "oracle_only" | "mcts_and_oracle"
  cooldown_gens: 100           # Min gens between emergency injections
  random_injection_count: 10   # Extra random genomes injected into L0 on trigger
  elitist_cull_ratio: 0.10     # Fraction of L0 weakest to cull on trigger (second review)
```

#### P8-2B: Trigger Logic

**文件**: `strategy-generator/src/main.rs:1144-1156`

**替换被动日志为主动触发**:

```rust
// P8-2B: Diversity monitoring with active trigger
if gen.is_multiple_of(50) {
    let diversity = ga.layer_diversity();
    let div_strs: Vec<String> = diversity
        .iter()
        .map(|(i, n, d)| format!("L{}:{:.2}(n={})", i, d, n))
        .collect();
    info!(
        "[{}:{}:{}] Gen {} diversity (Hamming): [{}]",
        exchange, symbol, mode_str, gen, div_strs.join(", ")
    );

    if diversity_trigger_config.enabled {
        let l3_div = diversity.iter()
            .find(|(i, _, _)| *i == 3)
            .map(|(_, _, d)| *d)
            .unwrap_or(1.0);
        let l4_div = diversity.iter()
            .find(|(i, _, _)| *i == 4)
            .map(|(_, _, d)| *d)
            .unwrap_or(1.0);

        let l3_stagnant = l3_div < diversity_trigger_config.min_diversity_l3;
        let l4_stagnant = l4_div < diversity_trigger_config.min_diversity_l4;

        if (l3_stagnant || l4_stagnant)
            && gens_since_diversity_trigger > diversity_trigger_config.cooldown_gens
        {
            warn!(
                "[{}:{}:{}] Gen {} DIVERSITY ALERT: L3={:.3} L4={:.3} — emergency injection",
                exchange, symbol, mode_str, gen, l3_div, l4_div
            );
            force_mcts_this_gen = true;
            force_oracle_this_gen = true;

            let random_genomes = ga.generate_random_genomes(
                diversity_trigger_config.random_injection_count,
                effective_feat_offset,
            );
            ga.inject_genomes(0, random_genomes);

            // P8-2B: Elitist replacement — kill lowest 10% in L0, inject MCTS seeds
            let cull_count = (ga.layer_size(0) as f64
                * diversity_trigger_config.elitist_cull_ratio).ceil() as usize;
            ga.cull_weakest(0, cull_count);
            info!(
                "[{}:{}:{}] Gen {} DIVERSITY: culled {} weakest from L0",
                exchange, symbol, mode_str, gen, cull_count
            );

            gens_since_diversity_trigger = 0;
        } else {
            gens_since_diversity_trigger += 50;
        }
    }
}
```

#### P8-2C: Metrics & Monitoring

**文件**: `strategy-generator/src/main.rs`

在代际元数据 JSONB 中记录触发事件:

```rust
metadata.insert("diversity_trigger_count", diversity_trigger_count.into());
metadata.insert("l3_diversity", l3_div.into());
metadata.insert("l4_diversity", l4_div.into());
```

**Prometheus 指标** (通过现有 `/metrics` 端点):
- `hermes_diversity_trigger_total{exchange, symbol, mode}` — 触发总次数
- `hermes_layer_diversity{exchange, symbol, mode, layer}` — 当前多样性值

### 5.3 验证计划

1. **单元测试**: 触发条件在阈值边界正确判断
2. **冷却测试**: 连续两次触发间隔 >= cooldown_gens
3. **集成测试**: 人工注入低多样性种群，验证触发和注入行为
4. **回归测试**: 正常多样性下不触发（避免误报）
5. **日志验证**: `warn!` 级别日志可被 Prometheus AlertManager 捕获

---

## 6. Phase 3: VM Hot Path Optimization [MEDIUM]

### 6.0 动机与事实核查

Gemini 建议添加静态形状守卫和 unsafe `uget()`。经核查：

**已有守卫**（`backtest-engine/src/vm/vm.rs:95`）:
```rust
if token < self.feat_offset {
    // Token is a feature index — zero-copy borrow from feature tensor
    let feature_view = features.index_axis(ndarray::Axis(1), token);
    stack.push(CowArray::from(feature_view));
}
```

`ndarray::index_axis()` 在 debug 模式下有 bounds check，release 模式下编译器可能优化掉。**未发现任何运行时 panic 证据**。

#### Gemini 第二轮建议：unsafe uget() — 再次拒绝

Gemini 在第二轮评审中再次建议使用 `unsafe { arr.uget() }` 消除边界检查。经量化分析：

- **热路径频率**: 每个公式执行仅 2 次 `index_axis()`（非每 time-step）
- **编译优化**: `Cargo.toml` 已配置 `lto = true, opt-level = 3, codegen-units = 1`
- **安全规则**: CLAUDE.md 明确规定 "No unsafe without documented justification"
- **无性能证据**: 未发现任何 panic 或性能瓶颈报告

如果未来性能分析（flamegraph）证实 bounds check 占比 >1%，可在 P9 中以 SAFETY 注释 + wrapper 函数形式引入。

**实际优化机会**:
1. 在进入热循环前一次性验证所有 token（避免每次迭代检查）
2. TS 操作符中使用 `ndarray::Zip` 替代手动索引
3. 条件性 NaN 清洗（仅对可能产生 NaN 的操作符执行）

### 6.1 实施细节

#### P8-3A: Pre-Execution Shape Guard

**文件**: `backtest-engine/src/vm/vm.rs:87-92`

在热循环前添加一次性验证:

```rust
pub fn execute<'f>(
    &self,
    formula_tokens: &[usize],
    features: &'f Array3<f64>,
) -> Option<Array2<f64>> {
    // P8-3A: Static shape guard — validate all feature tokens before hot loop
    let n_features = features.shape()[1];
    for &token in formula_tokens {
        if token < self.feat_offset && token >= n_features {
            return None;  // Feature index out of bounds
        }
    }

    let mut stack: Vec<CowArray<'f, f64, Ix2>> = Vec::new();
    // ... existing hot loop
```

**效果**: 将 N 次逐 token 的运行时检查替换为 1 次预扫描。

#### P8-3B: ndarray::Zip for TS Operators

**文件**: `backtest-engine/src/vm/ops.rs`

**当前**: TS 操作符（`ts_mean`, `ts_std` 等）使用手动行索引。

**优化**: 用 `ndarray::Zip` 重写，允许编译器进行 SIMD 自动向量化:

```rust
// Before: manual indexing
for row in 0..n_rows {
    for col in window..n_cols {
        let sum: f64 = (0..window).map(|w| input[[row, col - w]]).sum();
        output[[row, col]] = sum / window as f64;
    }
}

// After: ndarray::Zip (P8-3B)
Zip::from(output.rows_mut())
    .and(input.rows())
    .for_each(|mut out_row, in_row| {
        let mut running_sum = 0.0;
        for (i, &val) in in_row.iter().enumerate() {
            running_sum += val;
            if i >= window {
                running_sum -= in_row[i - window];
            }
            if i >= window - 1 {
                out_row[i] = running_sum / window as f64;
            }
        }
    });
```

#### P8-3C: Conditional NaN Sanitization

**文件**: `backtest-engine/src/vm/vm.rs:298-313`

**当前**: 每个操作符执行后都进行 NaN/Inf 清洗。

**优化**: 仅在可能产生 NaN/Inf 的操作符后清洗:

| 操作符 | 可能产生 NaN/Inf | 需要清洗 |
|--------|-----------------|---------|
| ADD, SUB, MUL | 仅 Inf (溢出) | 否 (float64 范围极大) |
| DIV | 除零 → NaN/Inf | **是** |
| SQRT | 负数 → NaN | **是** |
| LOG | ≤0 → NaN/Inf | **是** |
| SIGNED_POWER | 特殊情况 | **是** |
| ABS, SIGN, DELAY, NEGATE | 安全 | 否 |
| TS_MEAN, TS_STD, TS_RANK | 安全 | 否 |

**效果**: 减少约 60% 的清洗调用（23 个操作符中仅 4 个需要）。

### 6.2 验证计划

1. **基准测试**: `cargo bench` 对比优化前后的 VM 执行速度
2. **正确性测试**: 所有现有 VM 测试通过
3. **NaN 统计**: 优化后 `stats.protection_triggers` 计数不变
4. **回归测试**: 1000 个随机基因组的适应度与优化前完全一致

---

## 7. Phase 4: sqlx 0.8 迁移 + 金融精度强化 [MEDIUM]

### 7.0 动机

1. **安全**: sqlx 0.7.4 存在 RUSTSEC-2024-0363 安全公告
2. **精度**: 执行路径中的 f64 运算可能在大额交易中引入舍入误差

### 7.1 实施细节

#### P8-4A: sqlx 0.7.4 → 0.8.x 迁移

**文件**: `Cargo.toml:34`, 所有包含 sqlx 查询的 `*.rs` 文件

**当前依赖**:
```toml
sqlx = { version = "0.7.4", features = ["runtime-tokio", "tls-rustls", "postgres", "chrono", "json", "rust_decimal", "uuid"] }
```

**迁移步骤**:
1. 更新版本号: `0.7.4` → `0.8.x`
2. 检查 breaking changes（Pool API 变更、query macro 变更）
3. 按服务逐个验证: common → data-engine → strategy-generator → gateway
4. 运行 `cargo test --workspace` + Docker 部署验证

**迁移顺序** (依赖关系):
```
common (基础类型) → data-engine → strategy-generator → gateway
                                                      ↓
                                              execution-engine (独立构建)
```

#### P8-4B: f64 → Decimal in Execution Paths

**文件**: `strategy-generator/src/backtest/ensemble_weights.rs`, `execution-engine/src/shadow.rs`

**范围界定**: 仅转换涉及真实资金或执行信号的路径。

**需要转换**:
| 文件 | 当前类型 | 转换原因 |
|------|---------|---------|
| `ensemble_weights.rs`: HRP weights | `f64` | 权重精度影响资金分配 |
| `ensemble_weights.rs`: crowding penalty | `f64` | 惩罚计算影响最终权重 |
| `ensemble_weights.rs`: turnover cost | `f64` | 成本计算涉及真实资金 |
| `shadow.rs`: shadow equity | `f64` | 模拟权益追踪 |
| `shadow.rs`: PnL calculations | `f64` | 盈亏计算 |

**不转换**:
| 文件 | 保持 f64 原因 |
|------|--------------|
| `backtest-engine/src/vm/` | ndarray 需要 f64 进行向量化计算 |
| `backtest/mod.rs`: PSR 计算 | 统计量不涉及真实资金 |
| `mcts/search.rs`: UCB 分数 | 搜索启发式不需要金融精度 |

**代码示例** — `ensemble_weights.rs` 转换:

```rust
// Before (f64)
let psr_factor = 1.0 + config.psr_reward_scale
    * oos_psr.clamp(0.0, config.psr_max_reward);

// After (Decimal)
use rust_decimal::Decimal;
use rust_decimal_macros::dec;

let psr_factor = Decimal::ONE
    + Decimal::from_f64(config.psr_reward_scale).unwrap_or(dec!(0.2))
    * Decimal::from_f64(oos_psr)
        .unwrap_or(Decimal::ZERO)
        .max(Decimal::ZERO)
        .min(Decimal::from_f64(config.psr_max_reward).unwrap_or(dec!(3.0)));
```

#### P8-4C: 移除 P7 临时 16MiB 序列化限流门控（第二轮评审新增）

**文件**: `strategy-generator/src/main.rs`（搜索 `16_000_000`）

**背景**: P7 为缓解 RUSTSEC-2024-0363 在持久化路径设立了 16MiB 负载门控。sqlx 0.8 从根本上修复了长度前缀截断溢出漏洞。

**变更**: 升级完成并验证后，移除该门控代码，释放数据库最大存取吞吐力。

**验证**: 发送 >16MiB 的测试负载，确认 sqlx 0.8 正确处理。

### 7.2 验证计划

1. **编译测试**: `cargo build --workspace` 无错误
2. **迁移测试**: `cargo test --workspace` 全部通过
3. **精度测试**: Decimal 路径与 f64 路径的差异 < 1e-10
4. **安全审计**: `cargo audit` 不再报告 RUSTSEC-2024-0363
5. **Docker**: 所有服务重新构建和健康检查通过

---

## 8. P8 目标数据流 (全局视图)

```
mkt_equity_candles (1h/4h/1d)
  │
  └─ FeatureEngineer → Array3 (1, 75, T)
       │
       ├─ CCIPCA (Phase 1)
       │    └─ project_features() → Array3 (1, 80, T)
       │         token 75-79 = PC0~PC4
       │
       ├─ Factor Importance (Phase 0)
       │    └─ compute_permutation_importance()
       │         → Vec<FactorImportance> (缓存, 每 500 代)
       │
       ├─ 每代: parallel K-fold backtest (Decimal 精度, Phase 4)
       │
       ├─ 每 50 代: Diversity Check (Phase 2)
       │    └─ L3/L4 低? → 强制 MCTS + Oracle
       │
       ├─ MCTS (Phase 0 + Phase 3)
       │    ├─ LlmCachedPolicy (语义先验, 非均匀概率)
       │    ├─ Pre-execution shape guard (Phase 3)
       │    └─ Seeds → ALPS L0
       │
       └─ LLM Oracle
            ├─ 增强 prompt (含因子重要性, Phase 0)
            ├─ PC factor names (Phase 1)
            └─ Genomes → ALPS L0
```

---

## 9. 风险评估

| 风险 | 概率 | 影响 | 缓解措施 |
|------|------|------|----------|
| Factor importance 计算开销过高 | 低 | 中 | 每 500 代一次，可调整间隔 |
| CCIPCA 投影引入数值不稳定 | 中 | 中 | min_observations 守卫 + 数值范围检查 |
| Diversity trigger 误报 (正常波动触发) | 中 | 低 | cooldown 100 代 + 阈值可调 |
| sqlx 0.8 迁移 breaking changes | 高 | 高 | 逐服务迁移，每步验证 |
| LlmCachedPolicy 缓存未命中率高 | 中 | 低 | 自动退化为 UniformPolicy |

---

## 10. 推迟至 P9 的项目

| 项目 | 推迟原因 |
|------|---------|
| unsafe `uget()` in VM | 风险大于收益，当前无 panic 证据。需 flamegraph 证实 bounds check 占比 >1% 后再考虑（以 SAFETY 注释 + wrapper 函数形式引入） |
| 完整 AST 规范化哈希 | P8-0C 采用轻量 canonical RPN 排序。当 LLM 缓存命中率证明不足时，再考虑建立 AST 基础设施（~200-300 LOC） |
| 全局 f64→Decimal (VM 内部) | ndarray 不支持 Decimal，需重写 VM |
| CCIPCA 自适应 k 选择 | 需要更多运行数据确定最优 k |
| 多 symbol 共享 PC 空间 | 需要跨 symbol 协方差分析 |
| Actix→Axum 迁移 | **不需要** — Gemini 事实错误，全部已是 Axum |

---

## 11. 实施排期

```
Phase 0 (LLM Prior):           ~3.5 天
  P8-0A Factor Importance 接入   1 天
  P8-0B LLM Prompt 增强 (Top-10/Bottom-10)  0.5 天
  P8-0C Canonical RPN Hash (新增) 0.5 天
  P8-0D Policy 缓存填充           1 天
  P8-0E UniformPolicy 替换        0.5 天

Phase 1 (CCIPCA Active):       ~2 天
  P8-1A Feature Projection       1 天
  P8-1B Tensor Augmentation      0.5 天
  P8-1C PC Factor Names          0.5 天

Phase 2 (Diversity Trigger):    ~1.25 天
  P8-2A 配置                      0.25 天
  P8-2B 触发逻辑 + 精英替换 (新增) 0.75 天
  P8-2C 监控指标                  0.25 天

Phase 3 (VM Optimization):      ~2 天
  P8-3A Shape Guard               0.5 天
  P8-3B Zip TS Operators          1 天
  P8-3C Conditional Sanitization  0.5 天

Phase 4 (sqlx + Decimal):       ~3.25 天
  P8-4A sqlx 迁移                 2 天
  P8-4B f64→Decimal               1 天
  P8-4C 移除 16MiB 限流 (新增)    0.25 天

总计: ~12 天
```

---

## 12. 成功标准

| 指标 | P7 基线 | P8 目标 |
|------|---------|---------|
| MCTS 种子平均 PSR | 盲搜基线 | > 1.5x 提升 (语义先验) |
| MCTS 缓存命中率 | 0% (无缓存) | > 50% |
| 高龄层多样性崩溃事件 | 无响应 | 100% 检测 + 干预 |
| VM 执行速度 | 基线 | > 15% 提升 |
| sqlx 安全漏洞 | RUSTSEC-2024-0363 | 清零 |
| 执行路径精度 | f64 (~15 位) | Decimal (28 位) |
