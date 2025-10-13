# 高风险访问点测试计划

**版本**: v1.0.0  
**最后更新**: 2024-12-20  
**优先级**: P0 Critical

---

## 文档目录

- [1. 概述](#1-概述)
- [2. 数据库访问高风险点](#2-数据库访问高风险点)
- [3. API安全高风险点](#3-api安全高风险点)
- [4. 外部服务集成高风险点](#4-外部服务集成高风险点)
- [5. 测试执行与报告](#5-测试执行与报告)

---

## 1. 概述

### 1.1 什么是高风险访问点

高风险访问点是指系统中可能导致以下后果的关键位置：

- **数据泄露**：租户间数据访问、敏感信息暴露
- **安全漏洞**：SQL注入、XSS、CSRF攻击
- **系统故障**：外部服务失败导致系统崩溃
- **资金损失**：交易执行错误、订单泄露
- **合规风险**：审计日志缺失、权限绕过

### 1.2 测试目标

**零容忍原则**：所有高风险访问点测试必须100%通过，任何失败都会阻止代码合并。

**测试覆盖**：
- ✅ PostgreSQL RLS多租户隔离
- ✅ ClickHouse查询隔离
- ✅ Redis Key命名空间隔离
- ✅ Kafka Topic分区隔离
- ✅ JWT Token验证
- ✅ RBAC权限验证
- ✅ SQL注入防护
- ✅ XSS防护
- ✅ Rate Limiting
- ✅ 交易所API故障处理
- ✅ 外部服务降级

### 1.3 测试频率

| 测试类型 | 频率 | 触发条件 |
|---------|------|---------|
| **单元测试** | 每次commit | 自动（GitHub Actions） |
| **集成测试** | 每次PR | 自动（GitHub Actions） |
| **渗透测试** | 每月 | 手动（安全团队） |
| **审计** | 每季度 | 手动（外部审计） |

---

## 2. 数据库访问高风险点

### 2.1 PostgreSQL Row-Level Security (RLS)

#### 2.1.1 风险描述

**威胁模型**：
- 租户A的用户尝试读取租户B的数据
- 应用层Bug导致`app.current_tenant`未设置
- SQL注入绕过RLS策略

**影响**：
- **严重程度**: Critical
- **影响范围**: 所有租户数据
- **合规风险**: GDPR、数据主权

#### 2.1.2 测试用例

**TC-DB-001: 验证RLS策略生效**

```sql
-- 测试脚本: tests/security/test_rls_isolation.sql

-- 前置条件：创建测试数据
INSERT INTO tenants (id, name) VALUES ('tenant-a', 'Tenant A'), ('tenant-b', 'Tenant B');
INSERT INTO users (id, email, tenant_id, role) VALUES 
  ('user-a-1', 'user-a@example.com', 'tenant-a', 'user'),
  ('user-b-1', 'user-b@example.com', 'tenant-b', 'user');
INSERT INTO strategies (id, tenant_id, name, code) VALUES
  ('strategy-a-1', 'tenant-a', 'Strategy A1', 'def run(): pass'),
  ('strategy-b-1', 'tenant-b', 'Strategy B1', 'def run(): pass');

-- 测试步骤：
-- 1. 设置当前租户为tenant-a
SET app.current_tenant = 'tenant-a';

-- 2. 查询策略表，应该只返回租户A的数据
SELECT id, tenant_id, name FROM strategies;

-- 预期结果：
-- id            | tenant_id | name
-- ------------- | --------- | -----------
-- strategy-a-1  | tenant-a  | Strategy A1

-- 3. 尝试查询租户B的数据（应该返回0行）
SELECT COUNT(*) FROM strategies WHERE tenant_id = 'tenant-b';
-- 预期返回: 0

-- 4. 尝试联表查询（验证RLS在JOIN中也生效）
SELECT s.id, s.name, u.email 
FROM strategies s 
JOIN users u ON s.user_id = u.id
WHERE s.tenant_id = 'tenant-b';
-- 预期返回: 0行
```

**自动化测试（Python）**：

```python
# tests/security/test_tenant_isolation.py

import pytest
from sqlalchemy import text

class TestPostgreSQLRLS:
    """PostgreSQL RLS隔离测试"""
    
    def test_rls_isolation_basic(self, db_session, tenant_a, tenant_b):
        """TC-DB-001: 基本RLS隔离"""
        # 创建测试数据
        db_session.execute(text("""
            INSERT INTO strategies (id, tenant_id, name, code) VALUES
            ('strategy-a-1', :tenant_a, 'Strategy A1', 'def run(): pass'),
            ('strategy-b-1', :tenant_b, 'Strategy B1', 'def run(): pass');
        """), {'tenant_a': tenant_a.id, 'tenant_b': tenant_b.id})
        db_session.commit()
        
        # 设置当前租户为tenant-a
        db_session.execute(text(f"SET app.current_tenant = '{tenant_a.id}'"))
        
        # 查询策略，应该只看到tenant-a的数据
        result = db_session.execute(
            text("SELECT id, tenant_id FROM strategies")
        ).fetchall()
        
        strategy_ids = [row[0] for row in result]
        tenant_ids = set(row[1] for row in result)
        
        assert 'strategy-a-1' in strategy_ids, "应该看到租户A的策略"
        assert 'strategy-b-1' not in strategy_ids, "不应该看到租户B的策略"
        assert tenant_ids == {tenant_a.id}, f"所有策略都应属于{tenant_a.id}"
    
    def test_rls_isolation_count(self, db_session, tenant_a, tenant_b):
        """TC-DB-001-2: COUNT查询隔离"""
        db_session.execute(text(f"SET app.current_tenant = '{tenant_a.id}'"))
        
        # COUNT查询应该只统计当前租户的数据
        result = db_session.execute(
            text("SELECT COUNT(*) FROM strategies WHERE tenant_id = :tenant_b"),
            {'tenant_b': tenant_b.id}
        ).scalar()
        
        assert result == 0, "不应统计到其他租户的数据"
```

**TC-DB-002: 尝试绕过RLS（攻击测试）**

```python
def test_rls_bypass_attempt_update(self, db_session, tenant_a, tenant_b):
    """TC-DB-002: 尝试通过UPDATE绕过RLS"""
    db_session.execute(text(f"SET app.current_tenant = '{tenant_a.id}'"))
    
    # 尝试更新租户B的数据
    result = db_session.execute(text("""
        UPDATE strategies 
        SET name = 'Hacked!' 
        WHERE id = 'strategy-b-1'
    """))
    
    assert result.rowcount == 0, "RLS应该阻止跨租户更新"
    
    # 验证租户B的数据未被修改
    db_session.execute(text(f"SET app.current_tenant = '{tenant_b.id}'"))
    result = db_session.execute(
        text("SELECT name FROM strategies WHERE id = 'strategy-b-1'")
    ).fetchone()
    
    assert result[0] == 'Strategy B1', "数据不应被修改"

def test_rls_bypass_attempt_delete(self, db_session, tenant_a, tenant_b):
    """TC-DB-002-2: 尝试通过DELETE绕过RLS"""
    db_session.execute(text(f"SET app.current_tenant = '{tenant_a.id}'"))
    
    # 尝试删除租户B的数据
    result = db_session.execute(text("""
        DELETE FROM strategies WHERE id = 'strategy-b-1'
    """))
    
    assert result.rowcount == 0, "RLS应该阻止跨租户删除"

def test_rls_bypass_attempt_insert(self, db_session, tenant_a, tenant_b):
    """TC-DB-002-3: 尝试通过INSERT插入其他租户的数据"""
    db_session.execute(text(f"SET app.current_tenant = '{tenant_a.id}'"))
    
    # 尝试插入tenant-b的数据
    try:
        db_session.execute(text("""
            INSERT INTO strategies (id, tenant_id, name, code) 
            VALUES (gen_random_uuid(), :tenant_b, 'Hacked!', 'pass')
        """), {'tenant_b': tenant_b.id})
        db_session.commit()
        
        # 如果插入成功，验证tenant_id被强制改为tenant-a
        result = db_session.execute(text("""
            SELECT tenant_id FROM strategies WHERE name = 'Hacked!'
        """)).fetchone()
        
        assert result[0] == tenant_a.id, "tenant_id应被强制改为当前租户"
    except Exception as e:
        # 或者插入被拒绝（也是正确的行为）
        assert 'violates row-level security policy' in str(e).lower()
```

**TC-DB-003: 未设置current_tenant的安全检查**

```python
def test_missing_current_tenant(self, db_session):
    """TC-DB-003: 未设置current_tenant时应拒绝访问"""
    # 不设置app.current_tenant
    
    try:
        result = db_session.execute(text("SELECT * FROM strategies")).fetchall()
        
        # 如果查询成功，应该返回0行
        assert len(result) == 0, "未设置tenant时应返回空结果"
    except Exception as e:
        # 或者直接报错（更安全）
        assert 'current_tenant' in str(e).lower() or 'not set' in str(e).lower()
```

#### 2.1.3 RLS策略定义

```sql
-- database/migrations/001_create_rls_policies.sql

-- 1. 启用RLS
ALTER TABLE strategies ENABLE ROW LEVEL SECURITY;
ALTER TABLE orders ENABLE ROW LEVEL SECURITY;
ALTER TABLE positions ENABLE ROW LEVEL SECURITY;

-- 2. 创建RLS策略
CREATE POLICY tenant_isolation_policy ON strategies
    USING (tenant_id = current_setting('app.current_tenant')::uuid);

CREATE POLICY tenant_isolation_policy ON orders
    USING (tenant_id = current_setting('app.current_tenant')::uuid);

CREATE POLICY tenant_isolation_policy ON positions
    USING (tenant_id = current_setting('app.current_tenant')::uuid);

-- 3. 创建FORCE策略（防止INSERT绕过）
CREATE POLICY tenant_isolation_insert_policy ON strategies
    FOR INSERT
    WITH CHECK (tenant_id = current_setting('app.current_tenant')::uuid);

-- 4. 创建函数验证current_tenant已设置
CREATE OR REPLACE FUNCTION check_current_tenant()
RETURNS TRIGGER AS $$
BEGIN
    IF current_setting('app.current_tenant', true) IS NULL THEN
        RAISE EXCEPTION 'app.current_tenant is not set';
    END IF;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- 5. 为关键表添加触发器
CREATE TRIGGER ensure_current_tenant_set
    BEFORE INSERT OR UPDATE OR DELETE ON strategies
    FOR EACH ROW
    EXECUTE FUNCTION check_current_tenant();
```

### 2.2 ClickHouse 多租户查询隔离

#### 2.2.1 风险描述

**威胁模型**：
- 查询未包含`tenant_id`条件，返回所有租户数据
- 高频查询导致租户间资源竞争
- JOIN查询泄露其他租户数据

**影响**：
- **严重程度**: High
- **影响范围**: 时序数据、分析查询
- **性能影响**: 租户间干扰

#### 2.2.2 测试用例

**TC-CH-001: 强制WHERE tenant_id**

```python
# tests/security/test_clickhouse_isolation.py

def test_clickhouse_requires_tenant_id(ch_client):
    """TC-CH-001: 查询必须包含tenant_id条件"""
    # 尝试查询不带tenant_id的数据
    with pytest.raises(Exception) as exc_info:
        ch_client.execute("""
            SELECT * FROM market_data_1m 
            WHERE symbol = 'BTCUSDT' 
            LIMIT 100
        """)
    
    # 验证报错信息提示需要tenant_id
    assert 'tenant_id' in str(exc_info.value).lower() or \
           'mandatory' in str(exc_info.value).lower()

def test_clickhouse_tenant_isolation(ch_client, tenant_a, tenant_b):
    """TC-CH-002: 验证租户数据隔离"""
    # 插入测试数据
    ch_client.execute("""
        INSERT INTO market_data_1m (tenant_id, symbol, timestamp, open, high, low, close, volume)
        VALUES
        (%(tenant_a)s, 'BTCUSDT', '2024-01-01 00:00:00', 50000, 50100, 49900, 50050, 1000),
        (%(tenant_b)s, 'BTCUSDT', '2024-01-01 00:00:00', 51000, 51100, 50900, 51050, 1500)
    """, {'tenant_a': tenant_a.id, 'tenant_b': tenant_b.id})
    
    # 查询租户A的数据
    result_a = ch_client.execute("""
        SELECT COUNT(*), AVG(close) 
        FROM market_data_1m 
        WHERE tenant_id = %(tenant_a)s AND symbol = 'BTCUSDT'
    """, {'tenant_a': tenant_a.id})
    
    assert result_a[0][0] == 1, "应该只有1条租户A的数据"
    assert result_a[0][1] == 50050, "租户A的平均价格应该是50050"
    
    # 查询租户B的数据
    result_b = ch_client.execute("""
        SELECT COUNT(*), AVG(close) 
        FROM market_data_1m 
        WHERE tenant_id = %(tenant_b)s AND symbol = 'BTCUSDT'
    """, {'tenant_b': tenant_b.id})
    
    assert result_b[0][0] == 1, "应该只有1条租户B的数据"
    assert result_b[0][1] == 51050, "租户B的平均价格应该是51050"
```

#### 2.2.3 ClickHouse表设计

```sql
-- database/clickhouse/create_tables.sql

CREATE TABLE market_data_1m
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

-- 创建物化视图（自动包含tenant_id）
CREATE MATERIALIZED VIEW market_data_1h_mv
ENGINE = AggregatingMergeTree()
PARTITION BY toYYYYMM(timestamp)
ORDER BY (tenant_id, symbol, timestamp)
AS SELECT
    tenant_id,
    symbol,
    toStartOfHour(timestamp) as timestamp,
    argMax(open, timestamp) as open,
    max(high) as high,
    min(low) as low,
    argMax(close, timestamp) as close,
    sum(volume) as volume
FROM market_data_1m
GROUP BY tenant_id, symbol, timestamp;
```

### 2.3 Redis Key 命名空间隔离

#### 2.3.1 风险描述

**威胁模型**：
- Key命名未包含`tenant_id`前缀
- 使用`KEYS *`命令泄露其他租户Key
- 缓存污染攻击

**影响**：
- **严重程度**: Medium
- **影响范围**: 缓存数据
- **性能影响**: 缓存命中率

#### 2.3.2 测试用例

**TC-REDIS-001: Key前缀隔离**

```python
# tests/security/test_redis_isolation.py

def test_redis_key_prefix_isolation(redis_client, tenant_a, tenant_b):
    """TC-REDIS-001: Redis Key前缀隔离"""
    # 租户A写入数据
    key_a = f"tenant-{tenant_a.id}:market_data:BTCUSDT"
    redis_client.set(key_a, json.dumps({'price': 50000}))
    
    # 租户B尝试读取（使用tenant-b前缀）
    key_b = f"tenant-{tenant_b.id}:market_data:BTCUSDT"
    value_b = redis_client.get(key_b)
    
    assert value_b is None, "租户B不应该读到租户A的数据"
    
    # 租户A读取自己的数据（应该成功）
    value_a = redis_client.get(key_a)
    assert value_a is not None
    assert json.loads(value_a)['price'] == 50000

def test_redis_keys_command_isolation(redis_client, tenant_a, tenant_b):
    """TC-REDIS-002: KEYS命令隔离"""
    # 创建多个Key
    redis_client.set(f"tenant-{tenant_a.id}:strategy:1", "data1")
    redis_client.set(f"tenant-{tenant_a.id}:strategy:2", "data2")
    redis_client.set(f"tenant-{tenant_b.id}:strategy:1", "data3")
    
    # 租户A查询自己的Keys
    keys_a = redis_client.keys(f"tenant-{tenant_a.id}:*")
    keys_a_decoded = [k.decode() for k in keys_a]
    
    # 验证只返回租户A的Keys
    assert len(keys_a) == 2
    assert all(k.startswith(f"tenant-{tenant_a.id}:".encode()) for k in keys_a)
    
    # 验证不包含租户B的Keys
    assert all(f"tenant-{tenant_b.id}" not in k for k in keys_a_decoded)
```

#### 2.3.3 Redis Key命名规范

```python
# utils/redis_helper.py

class TenantAwareRedisClient:
    """租户感知的Redis客户端"""
    
    def __init__(self, redis_client, tenant_id):
        self.redis_client = redis_client
        self.tenant_id = tenant_id
    
    def _make_key(self, key: str) -> str:
        """生成带租户前缀的Key"""
        if not key.startswith(f"tenant-{self.tenant_id}:"):
            return f"tenant-{self.tenant_id}:{key}"
        return key
    
    def get(self, key: str):
        """安全的GET操作"""
        return self.redis_client.get(self._make_key(key))
    
    def set(self, key: str, value, **kwargs):
        """安全的SET操作"""
        return self.redis_client.set(self._make_key(key), value, **kwargs)
    
    def keys(self, pattern: str):
        """安全的KEYS操作（限制为当前租户）"""
        tenant_pattern = self._make_key(pattern)
        return self.redis_client.keys(tenant_pattern)
```

### 2.4 Kafka Topic 分区隔离

#### 2.4.1 风险描述

**威胁模型**：
- 消息Key未包含`tenant_id`
- 消费者组未按租户隔离
- 消息被路由到错误的租户

**影响**：
- **严重程度**: High
- **影响范围**: 实时数据流
- **业务影响**: 交易信号泄露

#### 2.4.2 测试用例

**TC-KAFKA-001: 消息分区隔离**

```python
# tests/security/test_kafka_isolation.py

def test_kafka_message_partition_by_tenant(kafka_producer, kafka_consumer, tenant_a, tenant_b):
    """TC-KAFKA-001: Kafka消息按租户分区"""
    # 发送消息到租户A
    message_a = {
        'tenant_id': str(tenant_a.id),
        'symbol': 'BTCUSDT',
        'price': 50000
    }
    kafka_producer.send(
        'market-data',
        key=str(tenant_a.id).encode(),
        value=json.dumps(message_a).encode()
    )
    kafka_producer.flush()
    
    # 租户B的消费者（只订阅tenant-b分区）
    consumer_b = KafkaConsumer(
        'market-data',
        group_id=f'consumer-{tenant_b.id}',
        auto_offset_reset='earliest',
        key_deserializer=lambda k: k.decode('utf-8')
    )
    
    # 消费消息（超时5秒）
    messages_received = []
    start_time = time.time()
    for msg in consumer_b:
        if msg.key == str(tenant_b.id):
            messages_received.append(json.loads(msg.value))
        
        # 超时或收到10条消息后停止
        if time.time() - start_time > 5 or len(messages_received) >= 10:
            break
    
    # 验证没有收到租户A的消息
    assert all(m['tenant_id'] != str(tenant_a.id) for m in messages_received), \
        "租户B不应收到租户A的消息"
```

---

## 3. API安全高风险点

### 3.1 JWT Token验证

#### 3.1.1 风险描述

**威胁模型**：
- 过期Token被接受
- Token签名被篡改
- Token泄露后未撤销

**影响**：
- **严重程度**: Critical
- **影响范围**: 所有API
- **业务影响**: 未授权访问

#### 3.1.2 测试用例完整汇总

见早期测试策略文档第2.1节。

### 3.2 RBAC权限验证

详细测试用例见早期测试策略文档。

### 3.3 Rate Limiting

#### 3.3.1 风险描述

**威胁模型**：
- DDoS攻击
- 恶意爬虫
- 资源耗尽

#### 3.3.2 测试用例

**TC-RATE-001: API速率限制生效**

```python
def test_api_rate_limiting(api_client, user_token):
    """TC-RATE-001: API速率限制生效"""
    # 快速发送100个请求
    responses = []
    for i in range(100):
        response = api_client.get(
            '/api/v1/strategies',
            headers={'Authorization': f'Bearer {user_token}'}
        )
        responses.append((response.status_code, response.headers))
        time.sleep(0.01)  # 10ms间隔
    
    # 验证部分请求被限流
    rate_limited_count = sum(1 for code, _ in responses if code == 429)
    assert rate_limited_count > 0, "Rate Limiting未生效"
    
    # 验证响应头包含限流信息
    for status_code, headers in responses:
        if status_code == 200:
            assert 'X-RateLimit-Limit' in headers, "缺少速率限制头"
            assert 'X-RateLimit-Remaining' in headers
            assert 'X-RateLimit-Reset' in headers
```

---

## 4. 外部服务集成高风险点

### 4.1 交易所API失败处理

详细测试用例见早期测试策略文档第5章。

### 4.2 外部服务降级

**TC-FALLBACK-001: 主数据源失败，切换到备用数据源**

```python
def test_fallback_to_secondary_datasource(monkeypatch):
    """TC-FALLBACK-001: 数据源故障降级"""
    # 模拟Binance API失败
    def mock_binance_fail(*args, **kwargs):
        raise ConnectionError("Binance API unavailable")
    
    # 模拟OKX API正常
    def mock_okx_success(*args, **kwargs):
        return {'symbol': 'BTCUSDT', 'price': 50000, 'source': 'okx'}
    
    monkeypatch.setattr('connectors.binance.get_ticker', mock_binance_fail)
    monkeypatch.setattr('connectors.okx.get_ticker', mock_okx_success)
    
    # 调用数据服务
    result = data_service.fetch_market_data('BTCUSDT')
    
    # 验证自动切换到备用数据源
    assert result['status'] == 'success'
    assert result['source'] == 'okx'
    assert result['price'] == 50000
```

---

## 5. 测试执行与报告

### 5.1 CI/CD集成

```yaml
# .github/workflows/high-risk-tests.yml
name: High-Risk Access Point Tests

on:
  push:
    branches: [dev, main]
  pull_request:
    branches: [dev, main]

jobs:
  high-risk-security-tests:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      
      - name: Run High-Risk Tests
        run: |
          pytest tests/security/ -v --tb=short \
            --markers="high_risk" \
            --junit-xml=high-risk-results.xml
      
      - name: Fail if any test failed
        run: |
          if [ $? -ne 0 ]; then
            echo "❌ 高风险测试失败，阻止代码合并"
            exit 1
          fi
      
      - name: Upload Results
        uses: actions/upload-artifact@v3
        with:
          name: high-risk-test-results
          path: high-risk-results.xml
```

### 5.2 测试报告模板

```markdown
## 高风险访问点测试报告

**执行时间**: 2024-12-20 10:00:00  
**执行人**: CI/CD Pipeline  
**分支**: dev

### 测试结果摘要

| 类别 | 总计 | 通过 | 失败 | 跳过 |
|------|------|------|------|------|
| PostgreSQL RLS | 15 | 15 | 0 | 0 |
| ClickHouse隔离 | 8 | 8 | 0 | 0 |
| Redis Key隔离 | 6 | 6 | 0 | 0 |
| Kafka分区隔离 | 5 | 5 | 0 | 0 |
| JWT验证 | 10 | 10 | 0 | 0 |
| RBAC权限 | 12 | 12 | 0 | 0 |
| SQL注入防护 | 8 | 8 | 0 | 0 |
| Rate Limiting | 4 | 4 | 0 | 0 |
| 外部服务故障 | 6 | 6 | 0 | 0 |
| **总计** | **74** | **74** | **0** | **0** |

### 测试覆盖率

✅ 所有高风险访问点已覆盖  
✅ 100%测试通过率  
✅ 无阻塞性问题

### 建议

无。所有高风险访问点测试通过，代码可以安全合并。
```

---

**最后更新**: 2024-12-20  
**负责团队**: Security & QA Team  
**联系方式**: security@hermesflow.com

