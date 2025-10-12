# 执行模块详细需求文档

**模块名称**: 执行模块 (Execution Module)  
**技术栈**: Java 21 + Spring Boot 3.x + WebFlux  
**版本**: v2.0.0  
**最后更新**: 2024-12-20

---

## 目录

1. [模块概述](#1-模块概述)
2. [架构设计](#2-架构设计)
3. [Epic详述](#3-epic详述)
4. [API规范](#4-api规范)
5. [性能基线与测试](#5-性能基线与测试)

---

## 1. 模块概述

### 1.1 模块职责

执行模块是HermesFlow平台的**交易执行核心**，负责：

1. **订单管理**: 订单全生命周期管理（创建、提交、跟踪、撤销）
2. **交易所集成**: 多交易所API集成与统一封装
3. **智能路由**: 最优价格选择和流动性分析
4. **订单执行优化**: 拆单、算法交易、滑点控制
5. **持仓管理**: 实时持仓同步和PnL计算

### 1.2 核心价值

- **可靠性**: 订单状态准确跟踪，不丢单不重单
- **性能**: 低延迟订单执行（<50ms P99）
- **智能化**: 智能路由选择最优执行路径
- **统一接口**: 屏蔽不同交易所API差异
- **容错性**: 交易所故障自动切换

### 1.3 性能目标

| 指标 | 目标值 | 测量方法 |
|------|--------|---------|
| 订单提交延迟 | P99 < 50ms | Prometheus监控 |
| 订单吞吐量 | > 1000 orders/s | 压力测试 |
| 订单准确率 | > 99.99% | 对账检查 |
| API可用性 | > 99.9% | Uptime监控 |

---

## 2. 架构设计

### 2.1 整体架构

```
┌─────────────────────────────────────────────────────────────┐
│            交易执行服务 (Port 18030)                          │
│         Java 21 + Spring Boot 3.x + WebFlux                 │
├─────────────────────────────────────────────────────────────┤
│                                                               │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐      │
│  │   Order      │  │   Exchange   │  │   Smart      │      │
│  │  Management  │  │  Connectors  │  │   Router     │      │
│  │              │  │              │  │              │      │
│  │ • Create     │──│ • Binance    │──│ • Price      │      │
│  │ • Track      │  │ • OKX        │  │ • Liquidity  │      │
│  │ • Cancel     │  │ • Bitget     │  │ • Split      │      │
│  │ • Sync       │  │ • Unified    │  │              │      │
│  └──────────────┘  └──────────────┘  └──────────────┘      │
│                                                               │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐      │
│  │  Position    │  │  Account     │  │  Execution   │      │
│  │  Manager     │  │  Sync        │  │  Reporter    │      │
│  │              │  │              │  │              │      │
│  │ • Track      │  │ • Balance    │  │ • Fills      │      │
│  │ • PnL        │  │ • Assets     │  │ • Stats      │      │
│  │ • Close      │  │ • Fees       │  │ • Alerts     │      │
│  └──────────────┘  └──────────────┘  └──────────────┘      │
│                                                               │
└─────────────────────────────────────────────────────────────┘
```

### 2.2 订单状态机

```
        CREATE
          │
          ▼
      PENDING ────────────────┐
          │                   │
       SUBMIT                 │
          │                   │ REJECT
          ▼                   │
     SUBMITTED ───────────────┤
          │                   │
      PARTIAL                 │
          │                   │
          ▼                   ▼
       FILLED              REJECTED
          │                   │
          │                   │
        DONE                DONE
```

---

## 3. Epic详述

### Epic 1: 订单管理 [P0]

#### 功能描述

提供完整的订单生命周期管理，包括创建、提交、跟踪、撤销。

#### 子功能

1. **订单创建** [P0]
   - 市价单（MARKET）
   - 限价单（LIMIT）
   - 条件单（STOP_LIMIT, STOP_MARKET）
   - 参数验证

2. **订单提交** [P0]
   - 异步提交到交易所
   - 重试机制
   - 幂等性保证

3. **订单跟踪** [P0]
   - 实时状态同步
   - WebSocket推送
   - 数据库持久化

4. **订单撤销** [P0]
   - 单个撤销
   - 批量撤销
   - 撤销所有

#### 用户故事

```gherkin
Feature: 下单交易
  作为一个交易者
  我想要在交易所下单
  以便执行交易策略

Scenario: 创建限价买单
  Given 我有足够的USDT余额
  When 我创建一个限价买单：
    | 交易所 | binance  |
    | 交易对 | BTCUSDT  |
    | 方向   | BUY      |
    | 类型   | LIMIT    |
    | 数量   | 0.1      |
    | 价格   | 46000.00 |
  Then 订单应该创建成功
  And 订单状态应该是 PENDING
  And 订单应该保存到数据库

Scenario: 提交订单到交易所
  Given 我有一个PENDING状态的订单
  When 系统提交订单到Binance
  Then 订单应该在50ms内提交成功
  And 订单状态应该更新为 SUBMITTED
  And 我应该收到Binance的订单ID
  And 我应该收到订单确认通知

Scenario: 取消未成交订单
  Given 我有一个SUBMITTED状态的订单
  And 订单尚未成交
  When 我点击"取消订单"按钮
  Then 系统应该向Binance发送撤单请求
  And 订单状态应该更新为 CANCELLED
  And 我应该收到撤单成功通知
```

#### 技术实现

```java
@Service
@Slf4j
public class OrderService {
    
    @Autowired
    private OrderRepository orderRepository;
    
    @Autowired
    private ExchangeConnectorFactory connectorFactory;
    
    @Autowired
    private KafkaTemplate<String, OrderEvent> kafkaTemplate;
    
    /**
     * 创建订单
     */
    @Transactional
    public Mono<Order> createOrder(CreateOrderRequest request) {
        return Mono.fromCallable(() -> {
            // 1. 参数验证
            validateOrder(request);
            
            // 2. 风控检查
            riskService.checkOrder(request);
            
            // 3. 创建订单实体
            Order order = Order.builder()
                .tenantId(SecurityContext.getCurrentTenant())
                .userId(SecurityContext.getCurrentUser())
                .strategyId(request.getStrategyId())
                .exchange(request.getExchange())
                .symbol(request.getSymbol())
                .side(request.getSide())
                .type(request.getType())
                .quantity(request.getQuantity())
                .price(request.getPrice())
                .status(OrderStatus.PENDING)
                .build();
            
            // 4. 保存到数据库
            order = orderRepository.save(order);
            
            // 5. 发布订单创建事件
            publishOrderEvent(order, OrderEventType.CREATED);
            
            log.info("订单创建成功: {}", order.getId());
            return order;
        })
        .subscribeOn(Schedulers.boundedElastic());
    }
    
    /**
     * 提交订单到交易所
     */
    public Mono<Order> submitOrder(UUID orderId) {
        return orderRepository.findById(orderId)
            .flatMap(order -> {
                // 1. 获取交易所连接器
                ExchangeConnector connector = connectorFactory.getConnector(order.getExchange());
                
                // 2. 提交订单
                return connector.submitOrder(order)
                    .flatMap(exchangeOrderId -> {
                        // 3. 更新订单状态
                        order.setExchangeOrderId(exchangeOrderId);
                        order.setStatus(OrderStatus.SUBMITTED);
                        order.setSubmittedAt(Instant.now());
                        
                        return orderRepository.save(order);
                    })
                    .doOnSuccess(o -> {
                        // 4. 发布订单提交事件
                        publishOrderEvent(o, OrderEventType.SUBMITTED);
                        log.info("订单提交成功: {} -> {}", o.getId(), o.getExchangeOrderId());
                    })
                    .doOnError(e -> {
                        // 5. 提交失败处理
                        order.setStatus(OrderStatus.REJECTED);
                        order.setErrorMessage(e.getMessage());
                        orderRepository.save(order);
                        
                        log.error("订单提交失败: {}", orderId, e);
                    });
            });
    }
    
    /**
     * 取消订单
     */
    public Mono<Order> cancelOrder(UUID orderId) {
        return orderRepository.findById(orderId)
            .flatMap(order -> {
                // 1. 检查订单状态
                if (!order.isCancellable()) {
                    return Mono.error(new BusinessException("订单不可取消"));
                }
                
                // 2. 调用交易所API取消
                ExchangeConnector connector = connectorFactory.getConnector(order.getExchange());
                
                return connector.cancelOrder(order.getExchangeOrderId())
                    .flatMap(success -> {
                        if (success) {
                            // 3. 更新订单状态
                            order.setStatus(OrderStatus.CANCELLED);
                            order.setCancelledAt(Instant.now());
                            
                            return orderRepository.save(order);
                        } else {
                            return Mono.error(new BusinessException("取消订单失败"));
                        }
                    })
                    .doOnSuccess(o -> {
                        publishOrderEvent(o, OrderEventType.CANCELLED);
                        log.info("订单取消成功: {}", orderId);
                    });
            });
    }
}
```

#### 验收标准

- [ ] 订单创建成功率 > 99.9%
- [ ] 订单提交延迟 P99 < 50ms
- [ ] 订单状态同步延迟 < 100ms
- [ ] 支持幂等提交（防止重复下单）
- [ ] 订单对账准确率 100%

---

### Epic 2: 交易所集成 [P0]

#### 功能描述

集成多个交易所API，提供统一的交易接口。

#### 子功能

1. **Binance集成** [P0]
   - Spot API
   - Futures API
   - WebSocket订阅

2. **OKX集成** [P0]
   - 现货/合约API
   - 签名认证
   - 错误处理

3. **Bitget集成** [P1]
   - API封装
   - 限流管理

4. **统一适配器** [P0]
   - 接口标准化
   - 数据格式转换
   - 错误码映射

#### 技术实现

```java
/**
 * 交易所连接器接口
 */
public interface ExchangeConnector {
    
    /**
     * 提交订单
     */
    Mono<String> submitOrder(Order order);
    
    /**
     * 取消订单
     */
    Mono<Boolean> cancelOrder(String exchangeOrderId);
    
    /**
     * 查询订单状态
     */
    Mono<ExchangeOrder> getOrderStatus(String exchangeOrderId);
    
    /**
     * 查询账户余额
     */
    Mono<Map<String, BigDecimal>> getBalance();
    
    /**
     * 查询持仓
     */
    Mono<List<Position>> getPositions();
}

/**
 * Binance连接器实现
 */
@Component
public class BinanceConnector implements ExchangeConnector {
    
    @Autowired
    private WebClient webClient;
    
    @Autowired
    private ApiKeyService apiKeyService;
    
    @Override
    public Mono<String> submitOrder(Order order) {
        return apiKeyService.getApiKey(order.getTenantId(), "binance")
            .flatMap(apiKey -> {
                // 1. 构建请求参数
                Map<String, String> params = new HashMap<>();
                params.put("symbol", order.getSymbol());
                params.put("side", order.getSide().name());
                params.put("type", order.getType().name());
                params.put("quantity", order.getQuantity().toPlainString());
                
                if (order.getPrice() != null) {
                    params.put("price", order.getPrice().toPlainString());
                }
                
                params.put("timestamp", String.valueOf(System.currentTimeMillis()));
                
                // 2. 签名
                String signature = signRequest(params, apiKey.getSecret());
                params.put("signature", signature);
                
                // 3. 发送HTTP请求
                return webClient.post()
                    .uri("https://api.binance.com/api/v3/order")
                    .header("X-MBX-APIKEY", apiKey.getKey())
                    .bodyValue(params)
                    .retrieve()
                    .bodyToMono(BinanceOrderResponse.class)
                    .map(response -> String.valueOf(response.getOrderId()))
                    .timeout(Duration.ofSeconds(5))
                    .retry(3)
                    .doOnError(e -> log.error("Binance下单失败", e));
            });
    }
    
    private String signRequest(Map<String, String> params, String secret) {
        // HMAC-SHA256签名
        String queryString = params.entrySet().stream()
            .sorted(Map.Entry.comparingByKey())
            .map(e -> e.getKey() + "=" + e.getValue())
            .collect(Collectors.joining("&"));
            
        return HmacUtils.hmacSha256Hex(secret, queryString);
    }
}
```

#### 验收标准

- [ ] 支持Binance/OKX/Bitget三个交易所
- [ ] API调用成功率 > 99%
- [ ] 自动限流（不超过交易所限制）
- [ ] 错误自动重试（最多3次）
- [ ] API密钥安全存储（AES-256加密）

---

### Epic 3: 智能路由 [P1]

#### 功能描述

分析多个交易所的价格和流动性，选择最优执行路径。

#### 子功能

1. **价格对比** [P1]
   - 实时价格获取
   - 手续费计算
   - 净价比较

2. **流动性分析** [P1]
   - 订单簿深度
   - 滑点估算
   - 市场冲击

3. **订单拆分** [P2]
   - 大单拆分
   - TWAP/VWAP算法
   - 跨交易所分配

#### 用户故事

```gherkin
Feature: 智能路由
  作为一个交易者
  我想要系统自动选择最优交易所
  以便获得最优价格

Scenario: 选择最优交易所
  Given 我要买入1 BTC
  And Binance的卖一价是 46000 USDT，手续费 0.1%
  And OKX的卖一价是 45950 USDT，手续费 0.08%
  When 系统执行智能路由
  Then 系统应该选择 OKX
  And 系统应该提交订单到 OKX
  And 我应该节省约 50 USDT + 手续费差
```

#### 验收标准

- [ ] 路由决策延迟 < 10ms
- [ ] 价格改善率 > 60%（vs单一交易所）
- [ ] 支持最多3个交易所比价

---

### Epic 4: 持仓管理 [P0]

#### 功能描述

实时追踪持仓状态，计算盈亏。

#### 子功能

1. **持仓追踪** [P0]
   - 实时同步
   - 多交易所聚合
   - 可用/冻结区分

2. **PnL计算** [P0]
   - 已实现盈亏
   - 未实现盈亏
   - 盈亏百分比

3. **持仓平仓** [P1]
   - 单个平仓
   - 批量平仓
   - 紧急平仓

#### 验收标准

- [ ] 持仓同步延迟 < 1秒
- [ ] PnL计算准确率 > 99.99%
- [ ] 支持最多100个持仓并发追踪

---

### Epic 5: 账户同步 [P0]

#### 功能描述

同步交易所账户余额、资产、手续费等信息。

#### 子功能

1. **余额同步** [P0]
2. **资产查询** [P0]
3. **手续费统计** [P1]
4. **资金划拨** [P2]

#### 验收标准

- [ ] 余额同步延迟 < 5秒
- [ ] 支持定时自动同步（每分钟）
- [ ] 支持手动刷新

---

## 4. API规范

### 4.1 REST API

```java
// 订单管理
POST   /api/v1/orders              // 创建订单
GET    /api/v1/orders              // 订单列表
GET    /api/v1/orders/:id          // 订单详情
DELETE /api/v1/orders/:id          // 取消订单
POST   /api/v1/orders/batch-cancel // 批量取消

// 持仓管理
GET    /api/v1/positions           // 持仓列表
GET    /api/v1/positions/:id       // 持仓详情
POST   /api/v1/positions/:id/close // 平仓

// 账户管理
GET    /api/v1/accounts/balance    // 账户余额
GET    /api/v1/accounts/assets     // 资产列表
POST   /api/v1/accounts/sync       // 同步账户
```

---

## 5. 性能基线与测试

### 5.1 性能基线

| 指标 | 目标 | 实际 |
|------|------|------|
| 订单提交延迟 | P99 < 50ms | TBD |
| 订单吞吐量 | > 1000/s | TBD |
| 持仓同步延迟 | < 1s | TBD |
| API可用性 | > 99.9% | TBD |

### 5.2 测试用例

```java
@SpringBootTest
@AutoConfigureWebTestClient
class OrderServiceTest {
    
    @Autowired
    private WebTestClient webClient;
    
    @Test
    void testCreateOrder() {
        // 测试创建订单
        CreateOrderRequest request = CreateOrderRequest.builder()
            .exchange("binance")
            .symbol("BTCUSDT")
            .side(OrderSide.BUY)
            .type(OrderType.LIMIT)
            .quantity(new BigDecimal("0.1"))
            .price(new BigDecimal("46000"))
            .build();
        
        webClient.post()
            .uri("/api/v1/orders")
            .bodyValue(request)
            .exchange()
            .expectStatus().isCreated()
            .expectBody()
            .jsonPath("$.id").isNotEmpty()
            .jsonPath("$.status").isEqualTo("PENDING");
    }
    
    @Test
    void testSubmitOrder() {
        // 测试提交订单
        // ...
    }
    
    @Test
    void testCancelOrder() {
        // 测试取消订单
        // ...
    }
}
```

---

**文档维护者**: Execution Team  
**最后更新**: 2024-12-20

