# 报表模块详细需求文档

**模块名称**: 报表模块 (Report Module)  
**版本**: v2.0.0  
**最后更新**: 2024-12-20

---

## 1. 模块概述

### 1.1 模块职责

1. **交易报表**: 订单、成交、手续费统计
2. **风险报表**: VaR、敞口、回撤分析
3. **策略报表**: 策略收益、胜率、夏普比率
4. **土狗评分**: 链上代币评分系统
5. **数据导出**: CSV、Excel、PDF导出

---

## 2. Epic详述

### Epic 1: 交易报表 [P0]

#### 功能描述

统计和展示交易相关数据。

#### 子功能

1. **订单明细** [P0]
   - 所有订单列表
   - 过滤和搜索
   - 导出功能

2. **成交记录** [P0]
   - 成交明细
   - 成交汇总
   - 平均成交价

3. **手续费统计** [P1]
   - 日/月/年手续费
   - 交易所分布
   - 手续费占比

#### 用户故事

```gherkin
Feature: 查看交易报表
  作为一个交易者
  我想要查看我的交易统计
  以便分析交易表现

Scenario: 查看月度交易报表
  Given 我登录系统
  When 我进入报表页面
  And 我选择"2024年12月"
  Then 我应该看到本月交易次数
  And 我应该看到本月成交金额
  And 我应该看到本月手续费总额
  And 我应该看到盈利/亏损订单比例
  And 我应该看到前10大成交交易对
```

#### 验收标准

- [ ] 报表查询延迟 < 1秒
- [ ] 支持多维度筛选
- [ ] 支持CSV/Excel导出
- [ ] 数据准确率 100%

---

### Epic 2: 风险报表 [P0]

#### 功能描述

展示风险指标变化趋势。

#### 子功能

1. **VaR趋势** [P0]
2. **回撤曲线** [P0]
3. **敞口分布** [P0]

---

### Epic 3: 土狗评分 [P1]

#### 功能描述

基于链上数据对土狗项目评分。

#### 评分维度

1. **流动性**: 流动性池深度
2. **持币地址**: 地址数量和分布
3. **交易活跃度**: 24h交易量
4. **社区热度**: GMGN热度指标
5. **安全性**: 合约审计、LP锁仓

#### 评分算法

```python
def calculate_token_score(token_address: str) -> TokenScore:
    """计算代币评分"""
    
    # 1. 获取链上数据
    data = gmgn_client.get_token_data(token_address)
    
    # 2. 各维度评分（0-100分）
    liquidity_score = min(data.liquidity_usd / 100000 * 100, 100)
    holder_score = min(data.holder_count / 1000 * 100, 100)
    volume_score = min(data.volume_24h / 1000000 * 100, 100)
    hot_score = data.hot_level  # GMGN提供
    
    # 3. 安全性评分
    safety_score = 0
    if data.is_audited:
        safety_score += 30
    if data.lp_locked:
        safety_score += 40
    if data.owner_renounced:
        safety_score += 30
    
    # 4. 加权总分
    total_score = (
        liquidity_score * 0.25 +
        holder_score * 0.20 +
        volume_score * 0.15 +
        hot_score * 0.20 +
        safety_score * 0.20
    )
    
    return TokenScore(
        total=total_score,
        liquidity=liquidity_score,
        holders=holder_score,
        volume=volume_score,
        hot=hot_score,
        safety=safety_score,
        grade=calculate_grade(total_score)  # S/A/B/C/D
    )
```

#### 验收标准

- [ ] 评分计算准确
- [ ] 评分更新频率 > 1次/小时
- [ ] 支持历史评分趋势

---

**文档维护者**: Report Team  
**最后更新**: 2024-12-20

