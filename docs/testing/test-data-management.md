# 测试数据管理指南

**版本**: v1.0.0  
**最后更新**: 2024-12-20

---

## 1. 测试数据策略

### 1.1 测试金字塔与数据策略

```
         /\
        /  \  E2E测试
       /----\  数据：生产级真实数据（完整场景）
      /      \  
     /--------\  
    /          \ 集成测试
   /------------\  数据：真实数据子集（核心场景）
  /              \  
 /----------------\  
--------------------
    单元测试
    数据：最小化Mock数据
```

| 测试层级 | 数据量 | 数据来源 | 清理策略 |
|---------|-------|---------|---------|
| **单元测试** | 最小化 | 硬编码/Mock | 无需清理 |
| **集成测试** | 中等 | Fixtures/TestContainers | 自动清理 |
| **E2E测试** | 大量 | 脱敏生产数据 | 定期重置 |

### 1.2 数据隔离原则

- ✅ 每个测试独立（无共享状态）
- ✅ 使用事务回滚（数据库测试）
- ✅ 使用唯一ID（避免冲突）
- ✅ 测试后自动清理
- ❌ 禁止使用生产数据库

---

## 2. 测试数据Fixtures

### 2.1 用户与租户Fixtures

**文件**: `tests/fixtures/users.py`

```python
import pytest
from models import User, Tenant
from datetime import datetime
import uuid

@pytest.fixture
def tenant_a(db_session):
    """租户A - 高级版"""
    tenant = Tenant(
        id=str(uuid.uuid4()),
        name='Test Tenant A',
        plan='premium',
        created_at=datetime.utcnow()
    )
    db_session.add(tenant)
    db_session.commit()
    yield tenant
    # 清理
    db_session.delete(tenant)
    db_session.commit()

@pytest.fixture
def tenant_b(db_session):
    """租户B - 基础版"""
    tenant = Tenant(
        id=str(uuid.uuid4()),
        name='Test Tenant B',
        plan='basic',
        created_at=datetime.utcnow()
    )
    db_session.add(tenant)
    db_session.commit()
    yield tenant
    db_session.delete(tenant)
    db_session.commit()

@pytest.fixture
def user_tenant_a(db_session, tenant_a):
    """租户A的普通用户"""
    user = User(
        id=str(uuid.uuid4()),
        email=f'user-{uuid.uuid4().hex[:8]}@example.com',
        tenant_id=tenant_a.id,
        role='user',
        created_at=datetime.utcnow()
    )
    db_session.add(user)
    db_session.commit()
    yield user
    db_session.delete(user)
    db_session.commit()

@pytest.fixture
def admin_tenant_a(db_session, tenant_a):
    """租户A的管理员"""
    user = User(
        id=str(uuid.uuid4()),
        email=f'admin-{uuid.uuid4().hex[:8]}@example.com',
        tenant_id=tenant_a.id,
        role='admin',
        created_at=datetime.utcnow()
    )
    db_session.add(user)
    db_session.commit()
    yield user
    db_session.delete(user)
    db_session.commit()
```

### 2.2 市场数据Fixtures

**文件**: `tests/fixtures/market_data.py`

```python
import pytest
import pandas as pd
from datetime import datetime, timedelta

@pytest.fixture
def btc_1h_data():
    """BTC 1小时K线数据（1000条）"""
    start_time = datetime(2024, 1, 1)
    timestamps = [start_time + timedelta(hours=i) for i in range(1000)]
    
    return pd.DataFrame({
        'timestamp': timestamps,
        'open': [50000 + i * 10 for i in range(1000)],
        'high': [50100 + i * 10 for i in range(1000)],
        'low': [49900 + i * 10 for i in range(1000)],
        'close': [50050 + i * 10 for i in range(1000)],
        'volume': [1000 + i for i in range(1000)]
    })

@pytest.fixture
def multi_symbol_data():
    """多币种数据（模拟真实场景）"""
    symbols = ['BTCUSDT', 'ETHUSDT', 'BNBUSDT']
    data = {}
    start_time = datetime(2024, 1, 1)
    
    for symbol in symbols:
        timestamps = [start_time + timedelta(minutes=i*5) for i in range(500)]
        base_price = {'BTCUSDT': 50000, 'ETHUSDT': 3000, 'BNBUSDT': 300}[symbol]
        
        data[symbol] = pd.DataFrame({
            'timestamp': timestamps,
            'price': [base_price + i for i in range(500)],
            'volume': [100 + i for i in range(500)]
        })
    return data

@pytest.fixture
def tick_data():
    """逐笔成交数据"""
    return pd.DataFrame({
        'timestamp': pd.date_range('2024-01-01', periods=10000, freq='100ms'),
        'price': [50000 + (i % 100) for i in range(10000)],
        'quantity': [0.01 + (i % 10) * 0.001 for i in range(10000)],
        'side': ['buy' if i % 2 == 0 else 'sell' for i in range(10000)]
    })
```

### 2.3 策略与订单Fixtures

**文件**: `tests/fixtures/strategies.py`

```python
import pytest
from models import Strategy, Order, Position

@pytest.fixture
def test_strategy(db_session, user_tenant_a):
    """测试策略"""
    strategy = Strategy(
        id=str(uuid.uuid4()),
        tenant_id=user_tenant_a.tenant_id,
        user_id=user_tenant_a.id,
        name='MA Crossover Test',
        code='''
def initialize(context):
    context.short_period = 10
    context.long_period = 20

def handle_data(context, data):
    short_ma = data['close'].rolling(context.short_period).mean()
    long_ma = data['close'].rolling(context.long_period).mean()
    
    if short_ma[-1] > long_ma[-1]:
        context.order('BTCUSDT', 0.01)
''',
        parameters={'short_period': 10, 'long_period': 20},
        status='active'
    )
    db_session.add(strategy)
    db_session.commit()
    yield strategy
    db_session.delete(strategy)
    db_session.commit()

@pytest.fixture
def test_order(db_session, test_strategy):
    """测试订单"""
    order = Order(
        id=str(uuid.uuid4()),
        tenant_id=test_strategy.tenant_id,
        strategy_id=test_strategy.id,
        symbol='BTCUSDT',
        side='BUY',
        type='MARKET',
        quantity=0.01,
        status='PENDING',
        created_at=datetime.utcnow()
    )
    db_session.add(order)
    db_session.commit()
    yield order
    db_session.delete(order)
    db_session.commit()
```

---

## 3. 数据库测试数据

### 3.1 初始化脚本

**文件**: `tests/fixtures/init.sql`

```sql
-- 创建测试数据库
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- 创建租户表
CREATE TABLE IF NOT EXISTS tenants (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name VARCHAR(255) NOT NULL,
    plan VARCHAR(50) NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- 创建用户表
CREATE TABLE IF NOT EXISTS users (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    email VARCHAR(255) UNIQUE NOT NULL,
    tenant_id UUID REFERENCES tenants(id),
    role VARCHAR(50) NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- 创建策略表
CREATE TABLE IF NOT EXISTS strategies (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    tenant_id UUID REFERENCES tenants(id),
    user_id UUID REFERENCES users(id),
    name VARCHAR(255) NOT NULL,
    code TEXT NOT NULL,
    parameters JSONB,
    status VARCHAR(50) DEFAULT 'draft',
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- 启用RLS
ALTER TABLE strategies ENABLE ROW LEVEL SECURITY;

-- 创建RLS策略
CREATE POLICY tenant_isolation_policy ON strategies
    USING (tenant_id = current_setting('app.current_tenant')::uuid);

-- 插入测试租户
INSERT INTO tenants (id, name, plan) VALUES
    ('00000000-0000-0000-0000-000000000001', 'Test Tenant A', 'premium'),
    ('00000000-0000-0000-0000-000000000002', 'Test Tenant B', 'basic');

-- 插入测试用户
INSERT INTO users (id, email, tenant_id, role) VALUES
    ('10000000-0000-0000-0000-000000000001', 'user-a@example.com', '00000000-0000-0000-0000-000000000001', 'user'),
    ('10000000-0000-0000-0000-000000000002', 'admin-a@example.com', '00000000-0000-0000-0000-000000000001', 'admin'),
    ('20000000-0000-0000-0000-000000000001', 'user-b@example.com', '00000000-0000-0000-0000-000000000002', 'user');
```

### 3.2 ClickHouse测试数据

**文件**: `tests/fixtures/clickhouse_init.sql`

```sql
-- 创建市场数据表
CREATE TABLE IF NOT EXISTS market_data_1m
(
    tenant_id UUID,
    symbol String,
    timestamp DateTime,
    open Float64,
    high Float64,
    low Float64,
    close Float64,
    volume Float64
)
ENGINE = MergeTree()
PARTITION BY toYYYYMM(timestamp)
ORDER BY (tenant_id, symbol, timestamp);

-- 插入测试数据
INSERT INTO market_data_1m (tenant_id, symbol, timestamp, open, high, low, close, volume)
SELECT
    '00000000-0000-0000-0000-000000000001' as tenant_id,
    'BTCUSDT' as symbol,
    toDateTime('2024-01-01 00:00:00') + INTERVAL number HOUR as timestamp,
    50000 + number * 10 as open,
    50100 + number * 10 as high,
    49900 + number * 10 as low,
    50050 + number * 10 as close,
    1000 + number as volume
FROM numbers(1000);
```

---

## 4. Mock外部服务

### 4.1 Mock交易所API

**文件**: `tests/mocks/exchange_mock.py`

```python
from unittest.mock import Mock, patch

class BinanceMock:
    """Binance API Mock"""
    
    @staticmethod
    def get_ticker(symbol):
        """模拟获取Ticker"""
        return {
            'symbol': symbol,
            'price': 50000.0,
            'timestamp': '2024-01-01T00:00:00Z'
        }
    
    @staticmethod
    def get_klines(symbol, interval, limit):
        """模拟获取K线"""
        return [
            {
                'timestamp': f'2024-01-01 {i:02d}:00:00',
                'open': 50000 + i,
                'high': 50100 + i,
                'low': 49900 + i,
                'close': 50050 + i,
                'volume': 1000 + i
            }
            for i in range(limit)
        ]
    
    @staticmethod
    def place_order(symbol, side, type, quantity):
        """模拟下单"""
        return {
            'orderId': '123456789',
            'symbol': symbol,
            'side': side,
            'type': type,
            'quantity': quantity,
            'status': 'FILLED'
        }

@pytest.fixture
def mock_binance(monkeypatch):
    """Mock Binance Connector"""
    mock = BinanceMock()
    monkeypatch.setattr('connectors.binance.BinanceConnector.get_ticker', mock.get_ticker)
    monkeypatch.setattr('connectors.binance.BinanceConnector.get_klines', mock.get_klines)
    monkeypatch.setattr('connectors.binance.BinanceConnector.place_order', mock.place_order)
    return mock
```

### 4.2 Mock Kafka

**文件**: `tests/mocks/kafka_mock.py`

```python
from collections import defaultdict
from queue import Queue

class KafkaMock:
    """Kafka Mock"""
    
    def __init__(self):
        self.topics = defaultdict(Queue)
    
    def send(self, topic, key, value):
        """模拟发送消息"""
        self.topics[topic].put((key, value))
    
    def consume(self, topic, timeout=1):
        """模拟消费消息"""
        try:
            return self.topics[topic].get(timeout=timeout)
        except:
            return None
    
    def clear(self, topic=None):
        """清空消息"""
        if topic:
            self.topics[topic] = Queue()
        else:
            self.topics.clear()

@pytest.fixture
def mock_kafka():
    """Mock Kafka Producer/Consumer"""
    return KafkaMock()
```

---

## 5. 数据生成器

### 5.1 市场数据生成器

**文件**: `tests/generators/market_data_generator.py`

```python
import pandas as pd
import numpy as np
from datetime import datetime, timedelta

class MarketDataGenerator:
    """市场数据生成器"""
    
    @staticmethod
    def generate_klines(symbol, start_date, end_date, interval='1h', base_price=50000):
        """生成K线数据"""
        freq_map = {'1m': '1T', '5m': '5T', '1h': '1H', '1d': '1D'}
        freq = freq_map.get(interval, '1H')
        
        timestamps = pd.date_range(start_date, end_date, freq=freq)
        n = len(timestamps)
        
        # 生成价格走势（随机游走）
        returns = np.random.normal(0, 0.001, n).cumsum()
        close_prices = base_price * np.exp(returns)
        
        # 生成OHLC
        high_prices = close_prices * (1 + np.abs(np.random.normal(0, 0.002, n)))
        low_prices = close_prices * (1 - np.abs(np.random.normal(0, 0.002, n)))
        open_prices = np.roll(close_prices, 1)
        open_prices[0] = base_price
        
        # 生成成交量
        volumes = np.random.lognormal(7, 1, n)
        
        return pd.DataFrame({
            'symbol': symbol,
            'timestamp': timestamps,
            'open': open_prices,
            'high': high_prices,
            'low': low_prices,
            'close': close_prices,
            'volume': volumes
        })
    
    @staticmethod
    def generate_tick_data(symbol, start_time, duration_seconds=3600, base_price=50000):
        """生成逐笔成交数据"""
        n_ticks = duration_seconds * 10  # 平均每秒10笔
        
        timestamps = pd.date_range(
            start_time, 
            periods=n_ticks, 
            freq=f'{1000//10}ms'
        )
        
        # 生成价格（微小波动）
        price_changes = np.random.choice([-1, 0, 1], n_ticks, p=[0.45, 0.1, 0.45])
        prices = base_price + np.cumsum(price_changes)
        
        # 生成数量
        quantities = np.random.exponential(0.01, n_ticks)
        
        # 生成买卖方向
        sides = np.random.choice(['buy', 'sell'], n_ticks)
        
        return pd.DataFrame({
            'symbol': symbol,
            'timestamp': timestamps,
            'price': prices,
            'quantity': quantities,
            'side': sides
        })
```

### 5.2 订单数据生成器

**文件**: `tests/generators/order_generator.py`

```python
class OrderGenerator:
    """订单数据生成器"""
    
    @staticmethod
    def generate_orders(n=100, symbols=['BTCUSDT', 'ETHUSDT'], tenant_id=None):
        """生成随机订单"""
        orders = []
        for i in range(n):
            order = {
                'id': str(uuid.uuid4()),
                'tenant_id': tenant_id or str(uuid.uuid4()),
                'symbol': np.random.choice(symbols),
                'side': np.random.choice(['BUY', 'SELL']),
                'type': np.random.choice(['MARKET', 'LIMIT']),
                'quantity': round(np.random.uniform(0.001, 1.0), 3),
                'price': round(np.random.uniform(45000, 55000), 2) if np.random.choice([True, False]) else None,
                'status': np.random.choice(['PENDING', 'FILLED', 'CANCELLED']),
                'created_at': datetime.utcnow() - timedelta(seconds=np.random.randint(0, 86400))
            }
            orders.append(order)
        return orders
```

---

## 6. 测试数据清理

### 6.1 自动清理策略

```python
# conftest.py

import pytest

@pytest.fixture(scope='function')
def db_session():
    """数据库Session（事务自动回滚）"""
    from database import SessionLocal
    
    session = SessionLocal()
    session.begin()  # 开始事务
    
    yield session
    
    # 测试结束后回滚事务
    session.rollback()
    session.close()

@pytest.fixture(scope='function')
def redis_client():
    """Redis客户端（测试后清空）"""
    import redis
    
    client = redis.Redis(host='localhost', port=6379, db=15)  # 使用测试DB
    
    yield client
    
    # 清空测试DB
    client.flushdb()

@pytest.fixture(scope='function')
def kafka_test_topic():
    """Kafka测试Topic（测试后删除）"""
    from kafka.admin import KafkaAdminClient, NewTopic
    
    admin = KafkaAdminClient(bootstrap_servers='localhost:9092')
    topic_name = f'test-topic-{uuid.uuid4().hex[:8]}'
    
    # 创建临时Topic
    topic = NewTopic(name=topic_name, num_partitions=1, replication_factor=1)
    admin.create_topics([topic])
    
    yield topic_name
    
    # 删除Topic
    admin.delete_topics([topic_name])
```

### 6.2 定期清理脚本

**文件**: `tests/cleanup.sh`

```bash
#!/bin/bash
# 测试环境清理脚本

echo "🧹 开始清理测试环境..."

# 停止测试容器
docker-compose -f docker-compose.test.yml down -v

# 删除测试数据卷
docker volume prune -f

# 清理测试结果
rm -rf test-results/
rm -rf htmlcov/
rm -rf .pytest_cache/
rm -f coverage.xml
rm -f .coverage

echo "✅ 测试环境清理完成"
```

---

## 7. 最佳实践

### 7.1 DO

- ✅ 使用UUID避免ID冲突
- ✅ 使用事务回滚清理数据库数据
- ✅ 使用独立的测试数据库（DB 15）
- ✅ 每个测试独立（无依赖）
- ✅ 使用有意义的测试数据（易于调试）
- ✅ Mock外部服务（快速、可靠）

### 7.2 DON'T

- ❌ 使用生产数据库
- ❌ 硬编码ID（容易冲突）
- ❌ 测试间共享状态
- ❌ 使用真实API（慢、不可靠）
- ❌ 忘记清理测试数据
- ❌ 使用过大的测试数据集

---

**最后更新**: 2024-12-20  
**维护团队**: QA Team

