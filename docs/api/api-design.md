# HermesFlow API 设计文档

**版本**: v2.0.0  
**最后更新**: 2024-12-20  
**状态**: 设计中

---

## 目录

1. [API设计原则](#1-api设计原则)
2. [认证与授权](#2-认证与授权)
3. [统一响应格式](#3-统一响应格式)
4. [错误处理](#4-错误处理)
5. [限流策略](#5-限流策略)
6. [服务API概览](#6-服务api概览)
7. [REST API详细设计](#7-rest-api详细设计)
8. [gRPC API设计](#8-grpc-api设计)

---

## 1. API设计原则

### 1.1 RESTful原则

1. **资源导向**: 使用名词复数形式表示资源（`/api/v1/strategies`）
2. **HTTP方法语义**:
   - GET: 查询资源
   - POST: 创建资源
   - PUT: 完整更新资源
   - PATCH: 部分更新资源
   - DELETE: 删除资源

3. **无状态**: 每个请求包含完整的认证信息，服务器不保存会话状态
4. **统一接口**: 所有API遵循统一的URL结构、响应格式、错误格式

### 1.2 版本控制

- URL版本控制: `/api/v1/`, `/api/v2/`
- 主版本号变更：不兼容的API修改
- 保持至少两个主版本并行支持

### 1.3 命名规范

- **URL**: 小写字母 + 连字符（kebab-case）
  - ✅ `/api/v1/market-data`
  - ❌ `/api/v1/marketData` 或 `/api/v1/market_data`

- **JSON字段**: 小写字母 + 下划线（snake_case）
  - ✅ `{ "created_at": "..." }`
  - ❌ `{ "createdAt": "..." }`

- **查询参数**: 小写字母 + 下划线
  - ✅ `?start_time=xxx&end_time=yyy`

### 1.4 分页规范

```http
GET /api/v1/strategies?page=1&page_size=20&sort=-created_at
```

**响应**:
```json
{
  "success": true,
  "data": {
    "items": [...],
    "pagination": {
      "current_page": 1,
      "page_size": 20,
      "total_items": 156,
      "total_pages": 8
    }
  }
}
```

### 1.5 字段过滤

```http
GET /api/v1/strategies?fields=id,name,status
```

只返回指定字段，减少网络传输。

---

## 2. 认证与授权

### 2.1 JWT认证

**登录流程**:
```
客户端 -> POST /api/v1/auth/login { username, password }
服务器 <- 返回 { access_token, refresh_token, expires_in }
客户端 -> 携带 Header: Authorization: Bearer <access_token>
```

**Token结构**:
```json
{
  "sub": "user_id_uuid",
  "tenant_id": "tenant_id_uuid",
  "username": "user@example.com",
  "roles": ["ADMIN", "TRADER"],
  "exp": 1703001600,
  "iat": 1702915200
}
```

### 2.2 Token刷新

```http
POST /api/v1/auth/refresh
Authorization: Bearer <refresh_token>

Response:
{
  "access_token": "new_access_token",
  "expires_in": 86400
}
```

### 2.3 权限检查

基于角色的访问控制(RBAC)：

| 角色 | 权限 |
|------|------|
| ADMIN | 所有权限 |
| DEVELOPER | 策略开发、回测、查看数据 |
| TRADER | 交易执行、查看持仓、查看数据 |
| VIEWER | 只读权限 |

**API权限标注**:
```java
@GetMapping("/api/v1/strategies")
@PreAuthorize("hasAnyRole('ADMIN', 'DEVELOPER', 'TRADER')")
public ResponseEntity<List<Strategy>> getStrategies() { ... }
```

---

## 3. 统一响应格式

### 3.1 成功响应

```json
{
  "success": true,
  "data": {
    // 实际数据
  },
  "error": null,
  "timestamp": "2024-12-20T10:30:00Z",
  "request_id": "550e8400-e29b-41d4-a716-446655440000"
}
```

### 3.2 错误响应

```json
{
  "success": false,
  "data": null,
  "error": {
    "code": "INVALID_PARAMETER",
    "message": "参数 'symbol' 不能为空",
    "details": {
      "field": "symbol",
      "constraint": "required"
    }
  },
  "timestamp": "2024-12-20T10:30:00Z",
  "request_id": "550e8400-e29b-41d4-a716-446655440000"
}
```

### 3.3 列表响应（带分页）

```json
{
  "success": true,
  "data": {
    "items": [
      { "id": 1, "name": "Strategy 1" },
      { "id": 2, "name": "Strategy 2" }
    ],
    "pagination": {
      "current_page": 1,
      "page_size": 20,
      "total_items": 156,
      "total_pages": 8,
      "has_next": true,
      "has_previous": false
    }
  },
  "error": null,
  "timestamp": "2024-12-20T10:30:00Z",
  "request_id": "550e8400-e29b-41d4-a716-446655440000"
}
```

---

## 4. 错误处理

### 4.1 HTTP状态码

| 状态码 | 说明 | 使用场景 |
|--------|------|---------|
| 200 | OK | 成功 |
| 201 | Created | 资源创建成功 |
| 204 | No Content | 删除成功 |
| 400 | Bad Request | 客户端参数错误 |
| 401 | Unauthorized | 未认证或Token过期 |
| 403 | Forbidden | 无权限 |
| 404 | Not Found | 资源不存在 |
| 409 | Conflict | 资源冲突（如重复创建） |
| 422 | Unprocessable Entity | 业务逻辑验证失败 |
| 429 | Too Many Requests | 超过限流 |
| 500 | Internal Server Error | 服务器内部错误 |
| 503 | Service Unavailable | 服务暂时不可用 |

### 4.2 错误码体系

| 错误码 | HTTP状态码 | 说明 |
|--------|-----------|------|
| INVALID_PARAMETER | 400 | 参数无效 |
| MISSING_PARAMETER | 400 | 缺少必需参数 |
| UNAUTHORIZED | 401 | 未认证 |
| TOKEN_EXPIRED | 401 | Token过期 |
| FORBIDDEN | 403 | 无权限 |
| RESOURCE_NOT_FOUND | 404 | 资源不存在 |
| RESOURCE_ALREADY_EXISTS | 409 | 资源已存在 |
| BUSINESS_LOGIC_ERROR | 422 | 业务逻辑错误 |
| RATE_LIMIT_EXCEEDED | 429 | 超过限流 |
| INTERNAL_ERROR | 500 | 内部错误 |
| SERVICE_UNAVAILABLE | 503 | 服务不可用 |
| EXTERNAL_SERVICE_ERROR | 502 | 外部服务错误 |

### 4.3 错误响应示例

```json
{
  "success": false,
  "data": null,
  "error": {
    "code": "BUSINESS_LOGIC_ERROR",
    "message": "策略必须先停止才能删除",
    "details": {
      "strategy_id": "abc-123",
      "current_status": "RUNNING"
    },
    "trace_id": "a1b2c3d4e5f6",
    "documentation_url": "https://docs.hermesflow.com/errors/BUSINESS_LOGIC_ERROR"
  },
  "timestamp": "2024-12-20T10:30:00Z",
  "request_id": "550e8400-e29b-41d4-a716-446655440000"
}
```

---

## 5. 限流策略

### 5.1 限流级别

1. **全局限流**: 所有API总限制
2. **用户级限流**: 每个用户的限制
3. **接口级限流**: 特定接口的限制

### 5.2 限流配置

| 限流级别 | 限制 |
|---------|------|
| 全局 | 10,000 req/min |
| 用户 | 1,000 req/min |
| 登录接口 | 10 req/min |
| 数据查询接口 | 100 req/min |
| 交易接口 | 50 req/min |

### 5.3 限流响应

**Headers**:
```
X-RateLimit-Limit: 1000
X-RateLimit-Remaining: 998
X-RateLimit-Reset: 1703001600
```

**响应 (429)**:
```json
{
  "success": false,
  "error": {
    "code": "RATE_LIMIT_EXCEEDED",
    "message": "请求过于频繁，请稍后再试",
    "details": {
      "limit": 1000,
      "remaining": 0,
      "reset_at": "2024-12-20T11:00:00Z"
    }
  }
}
```

---

## 6. 服务API概览

### 6.1 服务端口映射

| 服务 | 端口 | 协议 | 说明 |
|------|------|------|------|
| API Gateway | 18000 | HTTP/WS | 统一入口 |
| 数据采集服务 (Rust) | 18001 | HTTP/gRPC | 实时数据采集 |
| 数据处理服务 (Rust) | 18002 | HTTP/gRPC | 历史数据处理 |
| 用户管理服务 (Java) | 18010 | HTTP | 用户认证授权 |
| 策略引擎服务 (Python) | 18020 | HTTP | 策略开发执行 |
| 回测引擎 (Python) | 18021 | HTTP | 策略回测 |
| 交易执行服务 (Java) | 18030 | HTTP | 订单执行 |
| 风控服务 (Java) | 18040 | HTTP | 风险监控 |

### 6.2 API分类

#### 认证与用户管理 (18010)

```
POST   /api/v1/auth/login          # 登录
POST   /api/v1/auth/logout         # 登出
POST   /api/v1/auth/refresh        # 刷新Token
POST   /api/v1/auth/register       # 注册

GET    /api/v1/users               # 用户列表
GET    /api/v1/users/:id           # 用户详情
PUT    /api/v1/users/:id           # 更新用户
DELETE /api/v1/users/:id           # 删除用户
```

#### 数据采集 (18001 - Rust)

```
GET    /api/v1/market/realtime/:exchange/:symbol     # 实时行情
GET    /api/v1/market/history/:exchange/:symbol      # 历史数据
POST   /api/v1/market/subscribe                      # 订阅数据
DELETE /api/v1/market/subscribe/:subscription_id     # 取消订阅
GET    /api/v1/market/orderbook/:exchange/:symbol    # 订单簿
GET    /api/v1/market/trades/:exchange/:symbol       # 最近成交
GET    /health                                        # 健康检查
GET    /metrics                                       # Prometheus指标
```

#### 数据处理与分析 (18002 - Rust)

```
POST   /api/v1/analyze/indicators                    # 计算技术指标
GET    /api/v1/analyze/stats/:symbol                 # 统计分析
POST   /api/v1/analyze/correlation                   # 相关性分析
GET    /api/v1/sentiment/:keyword                    # 情绪数据
```

#### 策略管理 (18020 - Python)

```
GET    /api/v1/strategies                            # 策略列表
POST   /api/v1/strategies                            # 创建策略
GET    /api/v1/strategies/:id                        # 策略详情
PUT    /api/v1/strategies/:id                        # 更新策略
DELETE /api/v1/strategies/:id                        # 删除策略
POST   /api/v1/strategies/:id/start                  # 启动策略
POST   /api/v1/strategies/:id/stop                   # 停止策略
GET    /api/v1/strategies/:id/performance            # 策略性能
```

#### 回测 (18021 - Python)

```
POST   /api/v1/backtest                              # 创建回测任务
GET    /api/v1/backtest/:id                          # 回测结果
GET    /api/v1/backtest/:id/report                   # 回测报告
DELETE /api/v1/backtest/:id                          # 删除回测
```

#### 交易执行 (18030 - Java)

```
GET    /api/v1/orders                                # 订单列表
POST   /api/v1/orders                                # 创建订单
GET    /api/v1/orders/:id                            # 订单详情
DELETE /api/v1/orders/:id                            # 撤销订单
GET    /api/v1/positions                             # 持仓列表
GET    /api/v1/accounts/balance                      # 账户余额
```

#### 风控 (18040 - Java)

```
GET    /api/v1/risk/metrics                          # 风险指标
GET    /api/v1/risk/rules                            # 风控规则
POST   /api/v1/risk/rules                            # 创建风控规则
PUT    /api/v1/risk/rules/:id                        # 更新风控规则
GET    /api/v1/risk/alerts                           # 风险告警
```

---

## 7. REST API详细设计

### 7.1 数据采集服务API (Rust - 18001)

#### 获取实时行情

```http
GET /api/v1/market/realtime/:exchange/:symbol
```

**请求参数**:
- `exchange`: 交易所名称（binance/okx/ibkr）
- `symbol`: 交易对（BTCUSDT/AAPL）

**查询参数**:
- `fields`: 可选，返回字段过滤

**响应示例**:
```json
{
  "success": true,
  "data": {
    "exchange": "binance",
    "symbol": "BTCUSDT",
    "bid": 46000.00,
    "ask": 46001.50,
    "last": 46000.75,
    "volume": 12345.67,
    "timestamp": 1703001600000000,
    "quality_score": 100
  },
  "timestamp": "2024-12-20T10:30:00Z",
  "request_id": "uuid"
}
```

**错误响应**:
- 404: 交易对不存在
- 503: 数据源暂时不可用

#### 订阅实时数据

```http
POST /api/v1/market/subscribe
Content-Type: application/json

{
  "exchange": "binance",
  "symbols": ["BTCUSDT", "ETHUSDT"],
  "data_types": ["trade", "ticker"],
  "callback_url": "https://your-server.com/webhook"
}
```

**响应**:
```json
{
  "success": true,
  "data": {
    "subscription_id": "sub_abc123",
    "status": "active",
    "expires_at": "2024-12-21T10:30:00Z"
  }
}
```

#### 获取历史数据

```http
GET /api/v1/market/history/:exchange/:symbol?start_time=xxx&end_time=yyy&interval=1m
```

**查询参数**:
- `start_time`: 开始时间（Unix微秒）
- `end_time`: 结束时间（Unix微秒）
- `interval`: 聚合间隔（raw/1m/5m/15m/1h/1d）
- `limit`: 返回条数限制（默认1000，最大10000）

**响应**:
```json
{
  "success": true,
  "data": {
    "exchange": "binance",
    "symbol": "BTCUSDT",
    "interval": "1m",
    "klines": [
      {
        "timestamp": 1703001600000000,
        "open": 46000.00,
        "high": 46050.00,
        "low": 45980.00,
        "close": 46020.00,
        "volume": 123.45
      }
    ]
  }
}
```

#### 获取订单簿

```http
GET /api/v1/market/orderbook/:exchange/:symbol?depth=20
```

**响应**:
```json
{
  "success": true,
  "data": {
    "exchange": "binance",
    "symbol": "BTCUSDT",
    "timestamp": 1703001600000000,
    "bids": [
      [46000.00, 1.5],
      [45999.00, 2.0]
    ],
    "asks": [
      [46001.00, 1.2],
      [46002.00, 1.8]
    ]
  }
}
```

---

### 7.2 策略引擎API (Python - 18020)

#### 创建策略

```http
POST /api/v1/strategies
Content-Type: application/json

{
  "name": "MA Crossover",
  "description": "移动平均线交叉策略",
  "language": "python",
  "code": "class Strategy(BaseStrategy): ...",
  "parameters": {
    "fast_period": 10,
    "slow_period": 30
  }
}
```

**响应**:
```json
{
  "success": true,
  "data": {
    "id": "strat_abc123",
    "name": "MA Crossover",
    "status": "DRAFT",
    "created_at": "2024-12-20T10:30:00Z"
  }
}
```

#### 启动策略

```http
POST /api/v1/strategies/:id/start
Content-Type: application/json

{
  "mode": "live",  // 或 "paper"
  "exchanges": ["binance"],
  "symbols": ["BTCUSDT", "ETHUSDT"],
  "capital": 10000
}
```

**响应**:
```json
{
  "success": true,
  "data": {
    "execution_id": "exec_xyz789",
    "status": "STARTING",
    "started_at": "2024-12-20T10:30:00Z"
  }
}
```

#### 获取策略性能

```http
GET /api/v1/strategies/:id/performance?period=7d
```

**响应**:
```json
{
  "success": true,
  "data": {
    "strategy_id": "strat_abc123",
    "period": "7d",
    "metrics": {
      "total_return": 0.125,
      "sharpe_ratio": 1.85,
      "max_drawdown": 0.08,
      "win_rate": 0.62,
      "total_trades": 45,
      "profit_factor": 2.1
    },
    "equity_curve": [...]
  }
}
```

---

### 7.3 交易执行API (Java - 18030)

#### 创建订单

```http
POST /api/v1/orders
Content-Type: application/json

{
  "exchange": "binance",
  "symbol": "BTCUSDT",
  "side": "BUY",
  "type": "LIMIT",
  "quantity": 0.1,
  "price": 46000.00,
  "time_in_force": "GTC"
}
```

**响应**:
```json
{
  "success": true,
  "data": {
    "order_id": "order_abc123",
    "exchange_order_id": "binance_123456",
    "status": "SUBMITTED",
    "submitted_at": "2024-12-20T10:30:00.123Z"
  }
}
```

#### 撤销订单

```http
DELETE /api/v1/orders/:id
```

**响应**:
```json
{
  "success": true,
  "data": {
    "order_id": "order_abc123",
    "status": "CANCELLED",
    "cancelled_at": "2024-12-20T10:31:00Z"
  }
}
```

#### 查询持仓

```http
GET /api/v1/positions?exchange=binance
```

**响应**:
```json
{
  "success": true,
  "data": {
    "positions": [
      {
        "exchange": "binance",
        "symbol": "BTCUSDT",
        "side": "LONG",
        "quantity": 0.5,
        "avg_entry_price": 45000.00,
        "current_price": 46000.00,
        "unrealized_pnl": 500.00,
        "unrealized_pnl_pct": 0.0222
      }
    ],
    "total_value": 23000.00
  }
}
```

---

## 8. gRPC API设计

### 8.1 实时数据流 (market_data.proto)

```protobuf
syntax = "proto3";

package hermesflow.data;

// 市场数据服务
service MarketDataService {
  // 订阅实时数据流（服务端流）
  rpc StreamMarketData(StreamRequest) returns (stream MarketDataEvent);
  
  // 获取最新价格（一元RPC）
  rpc GetLatestPrice(PriceRequest) returns (PriceResponse);
  
  // 获取订单簿（一元RPC）
  rpc GetOrderBook(OrderBookRequest) returns (OrderBookResponse);
  
  // 批量查询（客户端流）
  rpc BatchQuery(stream QueryRequest) returns (stream QueryResponse);
}

// 订阅请求
message StreamRequest {
  repeated string exchanges = 1;     // 交易所列表
  repeated string symbols = 2;       // 交易对列表
  repeated DataType data_types = 3;  // 数据类型
  string client_id = 4;              // 客户端ID
}

// 市场数据事件
message MarketDataEvent {
  string exchange = 1;
  string symbol = 2;
  DataType data_type = 3;
  
  // 价格数据（可选）
  optional double bid = 4;
  optional double ask = 5;
  optional double last = 6;
  optional double volume = 7;
  
  int64 timestamp = 8;    // 微秒时间戳
  uint32 quality_score = 9;
}

// 数据类型枚举
enum DataType {
  TICK = 0;
  TRADE = 1;
  ORDERBOOK = 2;
  KLINE_1M = 3;
  KLINE_5M = 4;
  KLINE_15M = 5;
  KLINE_1H = 6;
}

// 价格查询请求
message PriceRequest {
  string exchange = 1;
  string symbol = 2;
}

// 价格查询响应
message PriceResponse {
  string exchange = 1;
  string symbol = 2;
  double price = 3;
  int64 timestamp = 4;
}

// 订单簿请求
message OrderBookRequest {
  string exchange = 1;
  string symbol = 2;
  uint32 depth = 3;  // 深度（默认20）
}

// 订单簿响应
message OrderBookResponse {
  string exchange = 1;
  string symbol = 2;
  repeated OrderBookLevel bids = 3;
  repeated OrderBookLevel asks = 4;
  int64 timestamp = 5;
}

message OrderBookLevel {
  double price = 1;
  double quantity = 2;
}
```

### 8.2 Rust服务端实现

```rust
use tonic::{transport::Server, Request, Response, Status};
use tokio_stream::wrappers::ReceiverStream;

pub struct MarketDataServer {
    // ... fields
}

#[tonic::async_trait]
impl MarketDataService for MarketDataServer {
    type StreamMarketDataStream = ReceiverStream<Result<MarketDataEvent, Status>>;
    
    async fn stream_market_data(
        &self,
        request: Request<StreamRequest>,
    ) -> Result<Response<Self::StreamMarketDataStream>, Status> {
        let req = request.into_inner();
        let (tx, rx) = mpsc::channel(1000);
        
        // 启动数据流任务
        tokio::spawn(async move {
            // 实现数据流逻辑...
        });
        
        Ok(Response::new(ReceiverStream::new(rx)))
    }
}
```

### 8.3 客户端使用示例

**Python客户端**:
```python
import grpc
from hermesflow.proto import market_data_pb2, market_data_pb2_grpc

async def stream_market_data():
    async with grpc.aio.insecure_channel('localhost:18001') as channel:
        stub = market_data_pb2_grpc.MarketDataServiceStub(channel)
        
        request = market_data_pb2.StreamRequest(
            exchanges=['binance'],
            symbols=['BTCUSDT', 'ETHUSDT'],
            data_types=[market_data_pb2.TRADE]
        )
        
        async for event in stub.StreamMarketData(request):
            print(f"{event.symbol}: {event.last}")
```

**Java客户端**:
```java
ManagedChannel channel = ManagedChannelBuilder
    .forAddress("localhost", 18001)
    .usePlaintext()
    .build();

MarketDataServiceGrpc.MarketDataServiceStub stub = 
    MarketDataServiceGrpc.newStub(channel);

StreamRequest request = StreamRequest.newBuilder()
    .addExchanges("binance")
    .addSymbols("BTCUSDT")
    .addDataTypes(DataType.TRADE)
    .build();

stub.streamMarketData(request, new StreamObserver<MarketDataEvent>() {
    @Override
    public void onNext(MarketDataEvent event) {
        System.out.println(event.getSymbol() + ": " + event.getLast());
    }
    
    @Override
    public void onError(Throwable t) {
        t.printStackTrace();
    }
    
    @Override
    public void onCompleted() {
        System.out.println("Stream completed");
    }
});
```

---

## 附录

### A. API测试工具

- **Postman Collection**: `docs/api/HermesFlow.postman_collection.json`
- **Swagger UI**: http://localhost:18000/swagger-ui
- **gRPC UI**: `grpcui -plaintext localhost:18001`

### B. API变更日志

| 版本 | 日期 | 变更内容 |
|------|------|---------|
| v2.0.0 | 2024-12-20 | 初始API设计 |

### C. 相关文档

- [OpenAPI规范](./rest-api-spec.yaml)
- [gRPC协议定义](./grpc-proto/)
- [API使用示例](./api-examples.md)

---

**文档维护者**: API Team  
**最后更新**: 2024-12-20

