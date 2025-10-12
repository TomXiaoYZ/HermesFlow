# 风控模块详细需求文档

**模块名称**: 风控模块 (Risk Management Module)  
**技术栈**: Java 21 + Spring Boot 3.x  
**版本**: v2.0.0  
**最后更新**: 2024-12-20

---

## 1. 模块概述

### 1.1 模块职责

风控模块是HermesFlow平台的**风险控制核心**，负责：

1. **实时风险监控**: 计算和监控各类风险指标
2. **风控规则引擎**: 基于规则的自动化风控
3. **链上清算保护**: DeFi借贷清算风险防护
4. **风险事件告警**: 实时风险告警和通知
5. **风险报表**: 风险分析和报告生成

### 1.2 性能目标

| 指标 | 目标值 |
|------|--------|
| 风险计算延迟 | < 10ms |
| 熔断响应时间 | < 100ms |
| 告警延迟 | < 1s |
| 规则执行延迟 | < 5ms |

---

## 2. Epic详述

### Epic 1: 实时风险监控 [P0]

#### 功能描述

实时计算账户级和策略级风险指标。

#### 子功能

1. **VaR计算** [P0]
   - 历史模拟法
   - 参数法（正态分布）
   - 蒙特卡洛模拟

2. **敞口监控** [P0]
   - 总敞口
   - 单币种敞口
   - 单策略敞口

3. **波动率监控** [P1]
   - 实现波动率
   - 隐含波动率（期权）

#### 用户故事

```gherkin
Feature: 实时风险监控
  作为一个交易者
  我想要实时查看我的风险指标
  以便及时控制风险

Scenario: 计算VaR
  Given 我的账户持有多个币种
  When 系统计算VaR（95%置信度，1天期）
  Then VaR应该 < 账户总值的5%
  And VaR应该每分钟更新
  And 如果VaR超过阈值应该发送告警

Scenario: 监控持仓敞口
  Given 我持有BTC、ETH、SOL
  When 我查看风险仪表盘
  Then 我应该看到总敞口金额
  And 我应该看到各币种占比
  And 如果单币种占比 > 50%应该有警告
```

#### 技术实现

```java
@Service
public class RiskCalculator {
    
    /**
     * 计算VaR（历史模拟法）
     */
    public BigDecimal calculateVaR(UUID tenantId, double confidence, int days) {
        // 1. 获取持仓
        List<Position> positions = positionRepository.findByTenantId(tenantId);
        
        // 2. 获取历史收益率
        LocalDate endDate = LocalDate.now();
        LocalDate startDate = endDate.minusDays(252); // 1年数据
        
        List<BigDecimal> returns = calculatePortfolioReturns(positions, startDate, endDate);
        
        // 3. 排序并找到分位数
        returns.sort(Comparator.naturalOrder());
        int index = (int) ((1 - confidence) * returns.size());
        BigDecimal var = returns.get(index).abs();
        
        // 4. 换算为days天的VaR
        BigDecimal scaledVar = var.multiply(BigDecimal.valueOf(Math.sqrt(days)));
        
        return scaledVar;
    }
    
    /**
     * 计算敞口
     */
    public Map<String, BigDecimal> calculateExposure(UUID tenantId) {
        List<Position> positions = positionRepository.findByTenantId(tenantId);
        
        Map<String, BigDecimal> exposure = new HashMap<>();
        BigDecimal totalValue = BigDecimal.ZERO;
        
        for (Position pos : positions) {
            BigDecimal value = pos.getMarketValue();
            exposure.put(pos.getSymbol(), value);
            totalValue = totalValue.add(value);
        }
        
        exposure.put("TOTAL", totalValue);
        return exposure;
    }
}
```

#### 验收标准

- [ ] VaR计算准确（vs手工计算误差<1%）
- [ ] 风险指标更新频率 > 1次/分钟
- [ ] 计算延迟 < 10ms

---

### Epic 2: 风控规则引擎 [P0]

#### 功能描述

基于规则的自动风控，支持多层次风控策略。

#### 子功能

1. **订单前风控** [P0]
   - 仓位限制
   - 单笔订单限额
   - 杠杆限制

2. **持仓风控** [P0]
   - 最大亏损止损
   - 浮盈止盈
   - 单币种仓位限制

3. **账户风控** [P0]
   - 日内亏损限制
   - 总资产回撤限制
   - 保证金率监控

4. **熔断机制** [P0]
   - 单策略熔断
   - 全账户熔断
   - 市场异常熔断

#### 用户故事

```gherkin
Feature: 风控规则
  作为一个交易者
  我想要设置风控规则
  以便自动控制风险

Scenario: 设置止损规则
  Given 我创建一个风控规则
  When 我设置"单策略最大亏损 -5%"
  And 某个策略亏损达到 -5%
  Then 系统应该自动停止该策略
  And 系统应该平掉该策略的所有持仓
  And 我应该收到止损告警通知

Scenario: 仓位限制
  Given 我设置"单币种最大仓位 30%"
  When 我尝试买入BTC，会导致BTC仓位占比 35%
  Then 系统应该拒绝该订单
  And 我应该收到仓位限制告警
```

#### 技术实现

```java
@Component
public class RiskRuleEngine {
    
    /**
     * 订单前风控检查
     */
    public RiskCheckResult checkOrder(Order order) {
        RiskCheckResult result = new RiskCheckResult();
        
        // 1. 仓位限制检查
        Position currentPos = positionService.getPosition(order.getSymbol());
        BigDecimal newSize = calculateNewSize(currentPos, order);
        
        RiskRule positionLimit = ruleRepository.findByType("POSITION_LIMIT");
        if (newSize.compareTo(positionLimit.getThreshold()) > 0) {
            result.setAllowed(false);
            result.setReason("超过仓位限制");
            return result;
        }
        
        // 2. 订单金额限制
        BigDecimal orderValue = order.getQuantity().multiply(order.getPrice());
        RiskRule orderLimit = ruleRepository.findByType("ORDER_LIMIT");
        
        if (orderValue.compareTo(orderLimit.getThreshold()) > 0) {
            result.setAllowed(false);
            result.setReason("超过单笔订单限额");
            return result;
        }
        
        // 3. 杠杆检查
        BigDecimal leverage = calculateLeverage(order);
        if (leverage.compareTo(BigDecimal.valueOf(3)) > 0) {
            result.setAllowed(false);
            result.setReason("超过最大杠杆倍数");
            return result;
        }
        
        result.setAllowed(true);
        return result;
    }
    
    /**
     * 持仓风控检查
     */
    @Scheduled(fixedDelay = 60000) // 每分钟检查
    public void checkPositions() {
        List<Position> positions = positionService.getAllPositions();
        
        for (Position pos : positions) {
            // 检查止损
            if (pos.getUnrealizedPnlPct().compareTo(BigDecimal.valueOf(-0.05)) < 0) {
                // 触发止损
                executionService.closePosition(pos.getId());
                alertService.sendAlert(AlertType.STOP_LOSS, pos);
            }
            
            // 检查止盈
            if (pos.getUnrealizedPnlPct().compareTo(BigDecimal.valueOf(0.20)) > 0) {
                // 触发止盈
                executionService.closePosition(pos.getId());
                alertService.sendAlert(AlertType.TAKE_PROFIT, pos);
            }
        }
    }
}
```

#### 验收标准

- [ ] 规则执行准确率 100%
- [ ] 规则执行延迟 < 5ms
- [ ] 支持至少10种规则类型
- [ ] 支持规则优先级

---

### Epic 3: 链上清算保护 [P1]

#### 功能描述

监控DeFi借贷平台的抵押率，防止清算。

#### 子功能

1. **抵押率监控** [P1]
2. **自动补仓** [P1]
3. **清算告警** [P1]

---

## 附录

### 风控规则类型

| 规则类型 | 说明 | 优先级 |
|---------|------|--------|
| POSITION_LIMIT | 仓位限制 | P0 |
| ORDER_LIMIT | 单笔限额 | P0 |
| LEVERAGE_LIMIT | 杠杆限制 | P0 |
| STOP_LOSS | 止损 | P0 |
| TAKE_PROFIT | 止盈 | P1 |
| DAILY_LOSS_LIMIT | 日内亏损 | P0 |
| DRAWDOWN_LIMIT | 最大回撤 | P0 |

---

**文档维护者**: Risk Team  
**最后更新**: 2024-12-20

