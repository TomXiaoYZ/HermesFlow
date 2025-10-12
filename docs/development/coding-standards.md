# 编码规范

**版本**: v2.0.0  
**最后更新**: 2024-12-20

---

## 目录

1. [Rust编码规范](#1-rust编码规范)
2. [Java编码规范](#2-java编码规范)
3. [Python编码规范](#3-python编码规范)
4. [通用规范](#4-通用规范)

---

## 1. Rust编码规范

### 1.1 代码风格

**遵循官方指南**

- 严格遵循 [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- 使用 `rustfmt` 自动格式化代码
- 使用 `clippy` 进行代码检查

```bash
# 格式化代码
cargo fmt

# 检查代码
cargo clippy -- -D warnings
```

### 1.2 命名约定

```rust
// 类型、结构体、枚举：PascalCase
struct MarketData {
    timestamp: i64,
    symbol: String,
}

enum OrderSide {
    Buy,
    Sell,
}

// 函数、变量、模块：snake_case
fn process_market_data(data: &MarketData) -> Result<()> {
    let processed_data = transform_data(data)?;
    Ok(())
}

// 常量、静态变量：SCREAMING_SNAKE_CASE
const MAX_CONNECTIONS: usize = 1000;
static GLOBAL_CONFIG: OnceCell<Config> = OnceCell::new();

// 生命周期参数：小写单字母
fn parse<'a>(input: &'a str) -> &'a str {
    input
}

// 泛型参数：单个大写字母或PascalCase
fn process<T: Serialize>(data: T) -> String {
    serde_json::to_string(&data).unwrap()
}
```

### 1.3 错误处理

```rust
use anyhow::{Context, Result};
use thiserror::Error;

// 使用 thiserror 定义自定义错误
#[derive(Error, Debug)]
pub enum DataEngineError {
    #[error("WebSocket connection failed: {0}")]
    WebSocketError(String),
    
    #[error("Data parsing error: {0}")]
    ParseError(String),
    
    #[error("Database error")]
    DatabaseError(#[from] sqlx::Error),
}

// 使用 Result<T, E> 返回可能失败的操作
pub async fn connect_to_exchange(url: &str) -> Result<WebSocketStream> {
    let (ws_stream, _) = connect_async(url)
        .await
        .context("Failed to connect to exchange")?;
    
    Ok(ws_stream)
}

// 使用 ? 操作符传播错误
pub fn process_data(raw: &str) -> Result<MarketData> {
    let data: MarketData = serde_json::from_str(raw)?;
    validate_data(&data)?;
    Ok(data)
}
```

### 1.4 异步编程

```rust
use tokio;

// 使用 async/await
#[tokio::main]
async fn main() -> Result<()> {
    let result = fetch_data().await?;
    process_data(result).await?;
    Ok(())
}

// 并发执行多个任务
async fn fetch_multiple_sources() -> Result<Vec<MarketData>> {
    let (binance, okx, bitget) = tokio::join!(
        fetch_from_binance(),
        fetch_from_okx(),
        fetch_from_bitget()
    );
    
    Ok(vec![binance?, okx?, bitget?])
}

// 使用 select! 处理多个Future
use tokio::select;

async fn handle_events() {
    loop {
        select! {
            msg = ws_receiver.recv() => {
                handle_websocket_message(msg).await;
            }
            _ = shutdown_signal.recv() => {
                break;
            }
        }
    }
}
```

### 1.5 所有权与借用

```rust
// 优先使用引用，避免不必要的克隆
fn process_data(data: &MarketData) -> Result<()> {
    // 只读访问
    println!("Processing {}", data.symbol);
    Ok(())
}

// 需要修改时使用可变引用
fn update_price(data: &mut MarketData, new_price: f64) {
    data.price = new_price;
}

// 需要所有权时使用值传递
fn consume_data(data: MarketData) {
    // data 被移动，调用者不能再使用
}

// 使用 Clone trait 明确克隆
let data_copy = original_data.clone();
```

### 1.6 文档注释

```rust
/// 从交易所获取实时市场数据
///
/// # Arguments
///
/// * `exchange` - 交易所名称（binance/okx/bitget）
/// * `symbol` - 交易对符号（例如：BTCUSDT）
///
/// # Returns
///
/// 返回 `Result<MarketData>`，成功时包含市场数据，失败时包含错误信息
///
/// # Errors
///
/// 当网络连接失败或数据格式错误时返回错误
///
/// # Examples
///
/// ```
/// let data = fetch_market_data("binance", "BTCUSDT").await?;
/// println!("Price: {}", data.price);
/// ```
pub async fn fetch_market_data(
    exchange: &str,
    symbol: &str,
) -> Result<MarketData> {
    // 实现...
}

//! 模块级文档注释
//! 
//! 本模块提供交易所数据采集功能
```

### 1.7 测试

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_data_parsing() {
        let raw = r#"{"symbol":"BTCUSDT","price":46000.0}"#;
        let data = parse_market_data(raw).unwrap();
        assert_eq!(data.symbol, "BTCUSDT");
        assert_eq!(data.price, 46000.0);
    }

    #[tokio::test]
    async fn test_async_fetch() {
        let result = fetch_market_data("binance", "BTCUSDT").await;
        assert!(result.is_ok());
    }
}
```

### 1.8 性能优化

```rust
// 避免不必要的分配
fn process_symbols(symbols: &[String]) -> Vec<String> {
    // 好：预分配容量
    let mut results = Vec::with_capacity(symbols.len());
    for symbol in symbols {
        results.push(process_symbol(symbol));
    }
    results
}

// 使用 Cow 避免不必要的克隆
use std::borrow::Cow;

fn normalize_symbol(symbol: &str) -> Cow<str> {
    if symbol.is_ascii() {
        Cow::Borrowed(symbol)
    } else {
        Cow::Owned(symbol.to_uppercase())
    }
}

// 使用迭代器链代替中间集合
fn filter_high_volume(data: &[MarketData]) -> Vec<&MarketData> {
    data.iter()
        .filter(|d| d.volume > 1_000_000.0)
        .collect()
}
```

---

## 2. Java编码规范

### 2.1 代码风格

遵循 [Google Java Style Guide](https://google.github.io/styleguide/javaguide.html)

```java
// 类名：PascalCase
public class OrderService {
    
    // 常量：UPPER_SNAKE_CASE
    private static final int MAX_RETRIES = 3;
    
    // 变量和方法：camelCase
    private OrderRepository orderRepository;
    
    public Order createOrder(CreateOrderRequest request) {
        // 实现...
    }
}
```

### 2.2 Spring Boot最佳实践

```java
@Service
@Slf4j
public class OrderService {
    
    private final OrderRepository orderRepository;
    private final KafkaTemplate<String, OrderEvent> kafkaTemplate;
    
    // 构造器注入（推荐）
    public OrderService(
        OrderRepository orderRepository,
        KafkaTemplate<String, OrderEvent> kafkaTemplate
    ) {
        this.orderRepository = orderRepository;
        this.kafkaTemplate = kafkaTemplate;
    }
    
    @Transactional
    public Mono<Order> createOrder(CreateOrderRequest request) {
        return Mono.fromCallable(() -> {
            // 业务逻辑
            Order order = Order.builder()
                .symbol(request.getSymbol())
                .quantity(request.getQuantity())
                .build();
            
            return orderRepository.save(order);
        })
        .subscribeOn(Schedulers.boundedElastic())
        .doOnSuccess(order -> {
            kafkaTemplate.send("orders", order.getId().toString(), 
                OrderEvent.created(order));
            log.info("Order created: {}", order.getId());
        });
    }
}
```

### 2.3 异常处理

```java
// 自定义异常
public class BusinessException extends RuntimeException {
    private final String errorCode;
    
    public BusinessException(String errorCode, String message) {
        super(message);
        this.errorCode = errorCode;
    }
}

// 全局异常处理
@RestControllerAdvice
public class GlobalExceptionHandler {
    
    @ExceptionHandler(BusinessException.class)
    public ResponseEntity<ErrorResponse> handleBusinessException(
        BusinessException ex
    ) {
        return ResponseEntity
            .status(HttpStatus.BAD_REQUEST)
            .body(new ErrorResponse(ex.getErrorCode(), ex.getMessage()));
    }
}
```

---

## 3. Python编码规范

### 3.1 代码风格

遵循 [PEP 8](https://pep8.org/)

```python
# 类名：PascalCase
class StrategyEngine:
    
    # 常量：UPPER_SNAKE_CASE
    MAX_POSITIONS = 10
    
    # 函数和变量：snake_case
    def execute_strategy(self, strategy_id: str) -> bool:
        """执行策略
        
        Args:
            strategy_id: 策略ID
            
        Returns:
            bool: 执行是否成功
        """
        pass

# 私有方法：前缀下划线
def _internal_method(self):
    pass
```

### 3.2 类型注解

```python
from typing import List, Dict, Optional, Union
from decimal import Decimal

def calculate_returns(
    prices: List[Decimal],
    initial_capital: Decimal = Decimal("10000")
) -> Dict[str, Union[Decimal, float]]:
    """计算收益率
    
    Args:
        prices: 价格列表
        initial_capital: 初始资金
        
    Returns:
        包含收益指标的字典
    """
    return {
        'total_return': Decimal("0.15"),
        'sharpe_ratio': 1.85
    }
```

### 3.3 异步编程

```python
import asyncio
from typing import List

async def fetch_market_data(symbol: str) -> dict:
    """异步获取市场数据"""
    async with aiohttp.ClientSession() as session:
        async with session.get(f"/api/market/{symbol}") as resp:
            return await resp.json()

async def fetch_multiple_symbols(symbols: List[str]) -> List[dict]:
    """并发获取多个交易对数据"""
    tasks = [fetch_market_data(symbol) for symbol in symbols]
    return await asyncio.gather(*tasks)
```

---

## 4. 通用规范

### 4.1 注释规范

```rust
// Rust: 单行注释使用 //
// 多行说明也使用多个 //

/// 文档注释使用 ///
```

```java
// Java: 单行注释
/* 多行注释 */

/**
 * Javadoc注释
 * @param id 订单ID
 * @return 订单对象
 */
```

```python
# Python: 单行注释

"""
多行注释或文档字符串
"""
```

### 4.2 Git Commit规范

```bash
# 格式
[module:service-name] type: subject

# 类型
feat: 新功能
fix: 修复bug
docs: 文档更新
refactor: 代码重构
test: 测试相关
chore: 构建/工具相关

# 示例
[module:data-engine-rust] feat: 新增Binance WebSocket连接器
[module:strategy-engine] fix: 修复回测引擎滑点计算错误
[docs] docs: 更新API文档
```

### 4.3 代码审查清单

**Rust代码审查**
- [ ] 所有权和借用正确
- [ ] 没有数据竞争
- [ ] 错误处理完整
- [ ] 性能关键路径优化
- [ ] 异步代码正确使用
- [ ] 测试覆盖率 > 85%

**Java代码审查**
- [ ] Spring注解使用正确
- [ ] 事务边界合理
- [ ] 异常处理完整
- [ ] 日志级别恰当
- [ ] 测试覆盖率 > 80%

**Python代码审查**
- [ ] 类型注解完整
- [ ] 异步代码正确
- [ ] 资源正确释放
- [ ] 测试覆盖率 > 75%

---

**文档维护者**: Development Team  
**最后更新**: 2024-12-20

