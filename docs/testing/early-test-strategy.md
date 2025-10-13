# HermesFlow 早期测试策略

**版本**: v1.0.0  
**最后更新**: 2024-12-20  
**负责团队**: QA Team

---

## 文档目录

- [1. 测试策略概述](#1-测试策略概述)
- [2. 安全测试策略](#2-安全测试策略)
- [3. 功能测试策略](#3-功能测试策略)
- [4. 性能测试策略](#4-性能测试策略)
- [5. 集成测试策略](#5-集成测试策略)
- [6. 测试执行计划](#6-测试执行计划)
- [7. 质量门禁](#7-质量门禁)

---

## 1. 测试策略概述

### 1.1 测试目标

HermesFlow 作为多租户量化交易平台，测试目标是确保：

1. **安全性**：多租户数据隔离、认证授权、防注入攻击
2. **功能性**：核心业务流程正确、API接口可靠
3. **性能**：高吞吐量数据处理、低延迟API响应
4. **可靠性**：外部服务故障降级、系统稳定性
5. **可维护性**：测试代码质量、自动化覆盖率

### 1.2 测试范围

| 模块 | 技术栈 | 测试重点 |
|------|--------|---------|
| **数据引擎** | Rust | 高并发、内存安全、数据隔离 |
| **策略引擎** | Python | 算法正确性、回测准确性、因子计算 |
| **交易服务** | Java | 订单处理、风控拦截、状态机 |
| **用户服务** | Java | 认证授权、多租户隔离、RBAC |
| **风控服务** | Java | 规则引擎、实时监控、告警 |
| **API网关** | Java | 路由、限流、认证代理 |
| **前端应用** | React | UI交互、数据展示、响应式 |

### 1.3 测试金字塔模型

```
         /\
        /  \  E2E测试 (10%)
       /----\  - 关键业务流程端到端验证
      /      \  - 用户场景模拟
     /--------\  
    /          \ 集成测试 (20%)
   /------------\  - 微服务间集成
  /              \  - 外部服务集成
 /----------------\  - 数据库集成
/                  \ 
--------------------
    单元测试 (70%)
    - 函数级测试
    - 类/模块测试
    - Mock外部依赖
```

**测试分布原则**：
- **70% 单元测试**：快速、稳定、易维护
- **20% 集成测试**：验证模块间协作
- **10% E2E测试**：验证关键业务流程

### 1.4 测试覆盖率目标

| 语言 | 覆盖率目标 | 强制性 | 衡量维度 |
|------|-----------|--------|---------|
| **Rust** | ≥85% | 是 | 行覆盖率 + 分支覆盖率 |
| **Java** | ≥80% | 是 | 行覆盖率 + 分支覆盖率 |
| **Python** | ≥75% | 是 | 行覆盖率 + 分支覆盖率 |
| **TypeScript** | ≥70% | 否 | 行覆盖率 |

**测量工具**：
- Rust: `cargo-tarpaulin`
- Java: JaCoCo
- Python: `pytest-cov`
- TypeScript: Jest

### 1.5 测试环境配置

#### 本地开发环境

```yaml
# docker-compose.test.yml
services:
  postgres:
    image: postgres:15
    environment:
      POSTGRES_DB: hermesflow_test
      POSTGRES_USER: testuser
      POSTGRES_PASSWORD: testpassword
  
  clickhouse:
    image: clickhouse/clickhouse-server:23.8
  
  redis:
    image: redis:7
  
  kafka:
    image: confluentinc/cp-kafka:7.5.0
```

**启动测试环境**：

```bash
docker-compose -f docker-compose.test.yml up -d
```

#### CI/CD测试环境

- **GitHub Actions**: 每次 commit/PR 自动触发
- **并行执行**: 多模块并行测试
- **缓存优化**: 依赖缓存、构建缓存

---

## 2. 安全测试策略

### 2.1 认证与授权测试

#### 2.1.1 JWT Token 验证

**测试目标**：确保Token生命周期管理正确，防止未授权访问。

**测试用例**：

| 用例ID | 场景 | 预期结果 | 优先级 |
|--------|------|---------|--------|
| TC-AUTH-001 | 过期Token访问API | 返回401 Unauthorized | P0 |
| TC-AUTH-002 | 篡改Token访问API | 返回401 Unauthorized | P0 |
| TC-AUTH-003 | 无Token访问受保护API | 返回401 Unauthorized | P0 |
| TC-AUTH-004 | 有效Token正常访问 | 返回200 OK | P0 |
| TC-AUTH-005 | Token刷新机制 | 返回新Token | P1 |
| TC-AUTH-006 | Token撤销后访问 | 返回401 Unauthorized | P1 |

**实现示例**：

```python
def test_expired_token_rejected(api_client, expired_token):
    """TC-AUTH-001: 过期Token被拒绝"""
    response = api_client.get(
        '/api/v1/strategies',
        headers={'Authorization': f'Bearer {expired_token}'}
    )
    assert response.status_code == 401
    assert 'expired' in response.json()['error'].lower()
```

#### 2.1.2 RBAC 权限测试

**测试目标**：验证基于角色的访问控制正确实施。

**角色权限矩阵**：

| 角色 | 查看策略 | 创建策略 | 删除策略 | 管理用户 | 查看所有用户数据 |
|------|---------|---------|---------|---------|----------------|
| **User** | ✅ (自己) | ✅ (自己) | ✅ (自己) | ❌ | ❌ |
| **Admin** | ✅ (租户内) | ✅ (租户内) | ✅ (租户内) | ✅ (租户内) | ✅ (租户内) |
| **SuperAdmin** | ✅ (全部) | ✅ (全部) | ✅ (全部) | ✅ (全部) | ✅ (全部) |

**测试用例**：

```python
def test_user_cannot_access_admin_api(api_client, user_token):
    """TC-RBAC-001: 普通用户无法访问管理员API"""
    response = api_client.post(
        '/api/v1/admin/users',
        headers={'Authorization': f'Bearer {user_token}'},
        json={'email': 'newuser@example.com', 'role': 'admin'}
    )
    assert response.status_code == 403
    assert 'permission denied' in response.json()['error'].lower()
```

#### 2.1.3 Session 管理测试

**测试场景**：
- 并发登录限制（可选）
- Session超时机制
- 注销后Session失效
- 跨设备Session管理

### 2.2 多租户隔离测试

#### 2.2.1 PostgreSQL RLS 测试

**测试目标**：验证Row-Level Security策略生效，租户数据完全隔离。

**测试用例**：

```sql
-- TC-DB-001: 验证RLS策略生效
SET app.current_tenant = 'tenant-a';
SELECT COUNT(*) FROM strategies WHERE tenant_id = 'tenant-b';
-- 预期返回: 0

-- TC-DB-002: 尝试绕过RLS（安全测试）
SET app.current_tenant = 'tenant-a';
UPDATE strategies SET tenant_id = 'tenant-a' WHERE id = '<tenant-b-strategy-id>';
-- 预期结果: 更新0行（RLS阻止跨租户操作）

-- TC-DB-003: 验证INSERT隔离
SET app.current_tenant = 'tenant-a';
INSERT INTO strategies (id, tenant_id, name) VALUES (gen_random_uuid(), 'tenant-b', 'Hacked!');
-- 预期结果: 插入失败或tenant_id被强制改为'tenant-a'
```

#### 2.2.2 Application 层隔离测试

**测试目标**：验证Service层权限检查。

```python
def test_service_layer_tenant_isolation():
    """TC-APP-001: Service层租户隔离"""
    # 租户A的用户尝试访问租户B的策略
    user_a = login_as_user('user-a@tenant-a.com', 'password')
    strategy_b_id = create_strategy_for_tenant_b()
    
    response = api_client.get(
        f'/api/v1/strategies/{strategy_b_id}',
        headers={'Authorization': f'Bearer {user_a.token}'}
    )
    
    # 应该返回404（而不是403，避免信息泄露）
    assert response.status_code == 404
```

#### 2.2.3 Redis Key 隔离测试

**测试目标**：验证Redis Key命名空间隔离。

```python
def test_redis_key_isolation(redis_client):
    """TC-REDIS-001: Redis Key命名空间隔离"""
    # 租户A写入数据
    redis_client.set("tenant-a:market_data:BTCUSDT", '{"price": 50000}')
    
    # 租户B尝试读取（使用tenant-b前缀）
    value = redis_client.get("tenant-b:market_data:BTCUSDT")
    assert value is None, "跨租户访问未被阻止！"
    
    # 验证KEYS命令隔离
    keys_a = redis_client.keys("tenant-a:*")
    assert all(k.startswith(b"tenant-a:") for k in keys_a)
```

#### 2.2.4 Kafka Topic 分区测试

**测试目标**：验证消息按租户分区，不跨租户消费。

```python
def test_kafka_topic_partition(kafka_producer, kafka_consumer):
    """TC-KAFKA-001: Kafka消息租户隔离"""
    # 发送消息到租户A的分区
    kafka_producer.send(
        'market-data',
        key=b'tenant-a',
        value=b'{"symbol": "BTCUSDT", "price": 50000}'
    )
    kafka_producer.flush()
    
    # 租户B的消费者（订阅tenant-b分区）
    consumer_b = KafkaConsumer(
        'market-data',
        group_id='tenant-b-consumer',
        key_deserializer=lambda k: k.decode('utf-8')
    )
    
    # 验证消费不到租户A的消息
    messages = []
    for msg in consumer_b:
        if msg.key == 'tenant-b':
            messages.append(msg.value)
        if len(messages) >= 10:
            break
    
    assert all(msg.key != 'tenant-a' for msg in messages)
```

### 2.3 输入验证与注入防护

#### 2.3.1 SQL 注入测试

**测试目标**：验证所有SQL查询使用Prepared Statement，防止SQL注入。

**测试用例**：

```python
def test_sql_injection_prevention(api_client, user_token):
    """TC-SQL-001: SQL注入攻击被阻止"""
    # 尝试SQL注入
    malicious_inputs = [
        "1' OR '1'='1",
        "1; DROP TABLE strategies;--",
        "1' UNION SELECT * FROM users--"
    ]
    
    for malicious_input in malicious_inputs:
        response = api_client.get(
            f'/api/v1/strategies/{malicious_input}',
            headers={'Authorization': f'Bearer {user_token}'}
        )
        
        # 验证攻击被阻止（返回404或400，而不是500）
        assert response.status_code in [400, 404], \
            f"SQL注入未被阻止: {malicious_input}"
```

**代码审查检查点**：
- ✅ 所有Java代码使用JPA或MyBatis参数化查询
- ✅ 所有Python代码使用SQLAlchemy或参数化查询
- ✅ 禁止字符串拼接SQL
- ✅ ORM使用安全配置

#### 2.3.2 XSS 防护测试

**测试目标**：验证前端输出转义，防止XSS攻击。

```python
def test_xss_prevention(api_client, user_token):
    """TC-XSS-001: XSS攻击被阻止"""
    # 尝试插入XSS脚本
    xss_payloads = [
        "<script>alert('XSS')</script>",
        "<img src=x onerror=alert('XSS')>",
        "javascript:alert('XSS')"
    ]
    
    for payload in xss_payloads:
        # 创建包含XSS的策略
        response = api_client.post(
            '/api/v1/strategies',
            headers={'Authorization': f'Bearer {user_token}'},
            json={'name': payload, 'code': 'def run(): pass'}
        )
        
        strategy_id = response.json()['id']
        
        # 获取策略，验证输出已转义
        response = api_client.get(
            f'/api/v1/strategies/{strategy_id}',
            headers={'Authorization': f'Bearer {user_token}'}
        )
        
        strategy_name = response.json()['name']
        assert '<script>' not in strategy_name, "XSS未被转义"
```

#### 2.3.3 CSRF 防护测试

**测试目标**：验证关键操作需要CSRF Token。

```python
def test_csrf_protection(api_client, user_token):
    """TC-CSRF-001: CSRF防护生效"""
    # 不带CSRF Token的POST请求
    response = api_client.post(
        '/api/v1/strategies',
        headers={'Authorization': f'Bearer {user_token}'},
        # 缺少 'X-CSRF-Token'
        json={'name': 'Test Strategy', 'code': 'def run(): pass'}
    )
    
    assert response.status_code == 403
    assert 'csrf' in response.json()['error'].lower()
```

#### 2.3.4 API 参数验证测试

**测试目标**：验证API参数类型、范围、格式校验。

```python
def test_api_parameter_validation(api_client, user_token):
    """TC-PARAM-001: API参数验证"""
    test_cases = [
        # (参数, 预期状态码, 描述)
        ({'name': '', 'code': 'def run(): pass'}, 400, "空名称"),
        ({'name': 'A' * 256, 'code': 'def run(): pass'}, 400, "名称过长"),
        ({'name': 'Test', 'code': ''}, 400, "空代码"),
        ({'name': 123, 'code': 'def run(): pass'}, 400, "名称类型错误"),
    ]
    
    for params, expected_status, desc in test_cases:
        response = api_client.post(
            '/api/v1/strategies',
            headers={'Authorization': f'Bearer {user_token}'},
            json=params
        )
        assert response.status_code == expected_status, f"参数验证失败: {desc}"
```

### 2.4 密钥与敏感数据保护

#### 2.4.1 API 密钥加密存储测试

**测试目标**：验证API密钥存储加密。

```python
def test_api_key_encryption(db_session):
    """TC-SECRET-001: API密钥加密存储"""
    # 创建API密钥
    api_key = create_api_key(user_id='user-123', exchange='binance', 
                             api_key='plain_api_key', secret='plain_secret')
    
    # 直接查询数据库
    result = db_session.execute(
        "SELECT api_key, secret FROM api_keys WHERE id = %s",
        (api_key.id,)
    ).fetchone()
    
    # 验证存储的是加密值
    assert result['api_key'] != 'plain_api_key', "API Key未加密"
    assert result['secret'] != 'plain_secret', "Secret未加密"
    
    # 验证可以正确解密
    decrypted_key = api_key.decrypt_api_key()
    assert decrypted_key == 'plain_api_key', "解密失败"
```

#### 2.4.2 日志脱敏测试

**测试目标**：验证敏感信息不出现在日志中。

```python
def test_log_masking(caplog):
    """TC-LOG-001: 日志脱敏"""
    # 模拟包含敏感信息的操作
    user = login_user('user@example.com', 'password123')
    
    # 检查日志
    sensitive_patterns = ['password123', 'Bearer eyJ', 'api_key=', 'secret=']
    for record in caplog.records:
        for pattern in sensitive_patterns:
            assert pattern not in record.message, \
                f"敏感信息出现在日志中: {pattern}"
```

---

## 3. 功能测试策略

### 3.1 核心业务流程测试

#### 3.1.1 用户注册与登录流程

**测试场景**：

```python
def test_user_registration_flow():
    """TC-FLOW-001: 完整用户注册流程"""
    # 1. 注册新用户
    response = api_client.post('/api/v1/auth/register', json={
        'email': 'newuser@example.com',
        'password': 'SecurePass123!',
        'tenant_name': 'My Tenant'
    })
    assert response.status_code == 201
    
    # 2. 验证邮箱（Mock）
    verification_token = mock_email_service.get_verification_token()
    response = api_client.post('/api/v1/auth/verify-email', json={
        'token': verification_token
    })
    assert response.status_code == 200
    
    # 3. 登录
    response = api_client.post('/api/v1/auth/login', json={
        'email': 'newuser@example.com',
        'password': 'SecurePass123!'
    })
    assert response.status_code == 200
    assert 'access_token' in response.json()
    
    # 4. 使用Token访问受保护资源
    token = response.json()['access_token']
    response = api_client.get('/api/v1/user/profile', 
                               headers={'Authorization': f'Bearer {token}'})
    assert response.status_code == 200
```

#### 3.1.2 策略创建、回测、部署流程

**测试场景**：

```python
def test_strategy_lifecycle():
    """TC-FLOW-002: 策略完整生命周期"""
    # 1. 创建策略
    response = api_client.post('/api/v1/strategies', 
        headers={'Authorization': f'Bearer {user_token}'},
        json={
            'name': 'MA Crossover',
            'code': STRATEGY_CODE,
            'parameters': {'short_period': 10, 'long_period': 20}
        })
    strategy_id = response.json()['id']
    
    # 2. 运行回测
    response = api_client.post(f'/api/v1/strategies/{strategy_id}/backtest',
        headers={'Authorization': f'Bearer {user_token}'},
        json={
            'start_date': '2024-01-01',
            'end_date': '2024-06-01',
            'initial_capital': 10000
        })
    backtest_id = response.json()['id']
    
    # 3. 等待回测完成
    wait_for_backtest_completion(backtest_id, timeout=60)
    
    # 4. 查看回测结果
    response = api_client.get(f'/api/v1/backtests/{backtest_id}/results',
                               headers={'Authorization': f'Bearer {user_token}'})
    assert response.status_code == 200
    assert response.json()['sharpe_ratio'] > 0
    
    # 5. 部署到模拟交易
    response = api_client.post(f'/api/v1/strategies/{strategy_id}/deploy',
        headers={'Authorization': f'Bearer {user_token}'},
        json={'mode': 'paper', 'capital': 5000})
    assert response.status_code == 200
```

#### 3.1.3 订单下单、撮合、成交流程

**测试场景**：

```python
def test_order_execution_flow():
    """TC-FLOW-003: 订单执行流程"""
    # 1. 下市价单
    response = api_client.post('/api/v1/orders',
        headers={'Authorization': f'Bearer {user_token}'},
        json={
            'strategy_id': strategy_id,
            'symbol': 'BTCUSDT',
            'side': 'BUY',
            'type': 'MARKET',
            'quantity': 0.01
        })
    order_id = response.json()['id']
    assert response.json()['status'] == 'PENDING'
    
    # 2. 等待订单成交
    time.sleep(2)
    response = api_client.get(f'/api/v1/orders/{order_id}',
                               headers={'Authorization': f'Bearer {user_token}'})
    assert response.json()['status'] == 'FILLED'
    
    # 3. 验证持仓更新
    response = api_client.get('/api/v1/positions',
                               headers={'Authorization': f'Bearer {user_token}'})
    positions = response.json()
    btc_position = next(p for p in positions if p['symbol'] == 'BTCUSDT')
    assert btc_position['quantity'] == 0.01
```

### 3.2 API 功能测试

#### 3.2.1 RESTful API CRUD 测试

**测试模板**：

```python
class TestStrategyAPI:
    """策略API CRUD测试"""
    
    def test_create_strategy(self, api_client, user_token):
        """TC-API-001: 创建策略"""
        response = api_client.post('/api/v1/strategies',
            headers={'Authorization': f'Bearer {user_token}'},
            json={'name': 'Test Strategy', 'code': 'def run(): pass'})
        assert response.status_code == 201
        assert 'id' in response.json()
    
    def test_read_strategy(self, api_client, user_token, test_strategy):
        """TC-API-002: 读取策略"""
        response = api_client.get(f'/api/v1/strategies/{test_strategy.id}',
                                   headers={'Authorization': f'Bearer {user_token}'})
        assert response.status_code == 200
        assert response.json()['name'] == test_strategy.name
    
    def test_update_strategy(self, api_client, user_token, test_strategy):
        """TC-API-003: 更新策略"""
        response = api_client.put(f'/api/v1/strategies/{test_strategy.id}',
            headers={'Authorization': f'Bearer {user_token}'},
            json={'name': 'Updated Name'})
        assert response.status_code == 200
        assert response.json()['name'] == 'Updated Name'
    
    def test_delete_strategy(self, api_client, user_token, test_strategy):
        """TC-API-004: 删除策略"""
        response = api_client.delete(f'/api/v1/strategies/{test_strategy.id}',
                                      headers={'Authorization': f'Bearer {user_token}'})
        assert response.status_code == 204
        
        # 验证已删除
        response = api_client.get(f'/api/v1/strategies/{test_strategy.id}',
                                   headers={'Authorization': f'Bearer {user_token}'})
        assert response.status_code == 404
```

#### 3.2.2 gRPC 接口测试

**测试示例**：

```python
def test_grpc_market_data_stream(grpc_channel):
    """TC-GRPC-001: 市场数据流式接口"""
    stub = market_data_pb2_grpc.MarketDataServiceStub(grpc_channel)
    
    # 订阅市场数据流
    request = market_data_pb2.SubscribeRequest(
        symbols=['BTCUSDT', 'ETHUSDT'],
        interval='1m'
    )
    
    # 接收流式数据
    received_count = 0
    for response in stub.SubscribeMarketData(request):
        assert response.symbol in ['BTCUSDT', 'ETHUSDT']
        assert response.price > 0
        received_count += 1
        if received_count >= 10:
            break
    
    assert received_count == 10, "未收到足够的流式数据"
```

#### 3.2.3 API 版本兼容性测试

**测试场景**：

```python
def test_api_version_compatibility():
    """TC-VER-001: API版本兼容性"""
    # V1 API
    response_v1 = api_client.get('/api/v1/strategies',
                                  headers={'Authorization': f'Bearer {token}'})
    assert response_v1.status_code == 200
    
    # V2 API（如果存在）
    response_v2 = api_client.get('/api/v2/strategies',
                                  headers={'Authorization': f'Bearer {token}'})
    
    # 验证向后兼容
    if response_v2.status_code == 200:
        assert len(response_v1.json()) == len(response_v2.json())
```

### 3.3 数据一致性测试

#### 3.3.1 PostgreSQL 与 ClickHouse 数据同步测试

**测试场景**：

```python
def test_postgres_clickhouse_sync():
    """TC-SYNC-001: PostgreSQL到ClickHouse数据同步"""
    # 1. 在PostgreSQL中创建策略
    strategy = create_strategy(name='Test Strategy', user_id='user-123')
    
    # 2. 执行回测，生成交易数据
    backtest = run_backtest(strategy_id=strategy.id)
    
    # 3. 等待数据同步到ClickHouse
    time.sleep(5)
    
    # 4. 验证ClickHouse中存在相同数据
    ch_client = clickhouse.Client()
    result = ch_client.execute("""
        SELECT COUNT(*) FROM trades 
        WHERE backtest_id = %(backtest_id)s
    """, {'backtest_id': backtest.id})
    
    pg_count = db_session.query(Trade).filter_by(backtest_id=backtest.id).count()
    ch_count = result[0][0]
    
    assert pg_count == ch_count, f"数据不一致: PG={pg_count}, CH={ch_count}"
```

#### 3.3.2 Kafka 消息可靠性测试

**测试场景**：

```python
def test_kafka_at_least_once_delivery():
    """TC-KAFKA-002: Kafka至少一次语义"""
    # 发送10条消息
    messages_sent = []
    for i in range(10):
        msg = {'id': i, 'symbol': 'BTCUSDT', 'price': 50000 + i}
        producer.send('market-data', value=json.dumps(msg).encode())
        messages_sent.append(msg)
    producer.flush()
    
    # 消费消息
    consumer = KafkaConsumer('market-data', auto_offset_reset='earliest')
    messages_received = []
    for msg in consumer:
        messages_received.append(json.loads(msg.value))
        if len(messages_received) >= 10:
            break
    
    # 验证所有消息都收到了
    sent_ids = set(m['id'] for m in messages_sent)
    received_ids = set(m['id'] for m in messages_received)
    assert sent_ids == received_ids, "消息丢失"
```

#### 3.3.3 Redis 缓存一致性测试

**测试场景**：

```python
def test_redis_cache_consistency():
    """TC-CACHE-001: Redis缓存一致性（Cache Aside模式）"""
    # 1. 首次查询（缓存未命中，从DB加载）
    strategy = get_strategy(strategy_id='strategy-123')
    assert strategy is not None
    
    # 2. 验证缓存已填充
    cached_value = redis_client.get('strategy:strategy-123')
    assert cached_value is not None
    
    # 3. 更新数据库
    db_session.query(Strategy).filter_by(id='strategy-123').update({'name': 'Updated'})
    db_session.commit()
    
    # 4. 验证缓存已失效
    cached_value = redis_client.get('strategy:strategy-123')
    assert cached_value is None, "缓存未失效"
    
    # 5. 再次查询，应该加载新数据
    strategy = get_strategy(strategy_id='strategy-123')
    assert strategy.name == 'Updated'
```

---

## 4. 性能测试策略

### 4.1 负载测试

#### 4.1.1 数据服务吞吐量测试

**测试目标**：验证数据服务达到 10,000 msg/s 的目标吞吐量。

**测试工具**：自定义Rust性能测试

```rust
// tests/performance/data_throughput_test.rs
#[tokio::test]
async fn test_data_service_throughput() {
    let data_service = DataService::new();
    let start_time = Instant::now();
    let total_messages = 100_000;
    
    // 并发发送消息
    let handles: Vec<_> = (0..total_messages)
        .map(|i| {
            let service = data_service.clone();
            tokio::spawn(async move {
                service.process_market_data(MarketData {
                    symbol: "BTCUSDT".to_string(),
                    price: 50000.0 + i as f64,
                    timestamp: Utc::now(),
                }).await
            })
        })
        .collect();
    
    // 等待所有任务完成
    for handle in handles {
        handle.await.unwrap();
    }
    
    let duration = start_time.elapsed();
    let throughput = total_messages as f64 / duration.as_secs_f64();
    
    println!("吞吐量: {:.2} msg/s", throughput);
    assert!(throughput >= 10_000.0, "吞吐量未达标");
}
```

#### 4.1.2 API 并发测试

**测试目标**：验证API支持 1,000 req/s 并发。

**测试工具**：k6

```javascript
// tests/performance/api_load_test.js
import http from 'k6/http';
import { check } from 'k6';

export let options = {
  stages: [
    { duration: '2m', target: 500 },
    { duration: '5m', target: 1000 },
    { duration: '2m', target: 0 },
  ],
  thresholds: {
    'http_req_duration': ['p(95)<500'],  // 95%请求<500ms
    'http_req_failed': ['rate<0.01'],    // 错误率<1%
  },
};

export default function () {
  let response = http.get('http://api.hermesflow.com/api/v1/strategies', {
    headers: { 'Authorization': `Bearer ${__ENV.API_TOKEN}` },
  });
  
  check(response, {
    'status is 200': (r) => r.status === 200,
  });
}
```

#### 4.1.3 数据库连接池压力测试

**测试场景**：

```python
def test_database_connection_pool_stress():
    """TC-PERF-003: 数据库连接池压力测试"""
    from concurrent.futures import ThreadPoolExecutor
    
    def query_database():
        with db_engine.connect() as conn:
            result = conn.execute("SELECT COUNT(*) FROM strategies")
            return result.fetchone()[0]
    
    # 1000个并发查询
    with ThreadPoolExecutor(max_workers=1000) as executor:
        futures = [executor.submit(query_database) for _ in range(1000)]
        results = [f.result() for f in futures]
    
    # 验证所有查询都成功
    assert len(results) == 1000
    assert all(r >= 0 for r in results)
```

### 4.2 压力测试

#### 4.2.1 系统极限容量测试

**测试目标**：找到系统崩溃点。

**测试工具**：k6 + Prometheus

```javascript
export let options = {
  stages: [
    { duration: '5m', target: 100 },
    { duration: '5m', target: 500 },
    { duration: '5m', target: 1000 },
    { duration: '5m', target: 2000 },   // 逐步增加
    { duration: '5m', target: 5000 },   // 直到系统崩溃
  ],
};
```

**监控指标**：
- CPU使用率
- 内存使用率
- 响应时间P99
- 错误率

#### 4.2.2 峰值流量测试

**测试场景**：模拟市场突发行情导致的流量激增。

```javascript
export let options = {
  scenarios: {
    spike: {
      executor: 'ramping-arrival-rate',
      startRate: 100,
      timeUnit: '1s',
      preAllocatedVUs: 500,
      maxVUs: 5000,
      stages: [
        { duration: '2m', target: 100 },
        { duration: '1m', target: 2000 }, // 突增
        { duration: '5m', target: 2000 }, // 保持
        { duration: '2m', target: 100 },  // 恢复
      ],
    },
  },
};
```

### 4.3 性能基线与监控

#### 4.3.1 响应时间基线

| API端点 | P50 | P95 | P99 | 目标 |
|---------|-----|-----|-----|------|
| GET /api/v1/strategies | <50ms | <200ms | <500ms | 符合 |
| POST /api/v1/orders | <100ms | <300ms | <800ms | 符合 |
| GET /api/v1/market-data/{symbol} | <20ms | <100ms | <200ms | 符合 |
| POST /api/v1/strategies/{id}/backtest | <200ms | <500ms | <1000ms | 符合 |

#### 4.3.2 资源使用率基线

| 资源 | 空闲 | 正常负载 | 高负载 | 告警阈值 |
|------|------|---------|--------|---------|
| **CPU** | <10% | 30-50% | 60-80% | >85% |
| **Memory** | <20% | 40-60% | 70-85% | >90% |
| **Disk I/O** | <10% | 20-40% | 50-70% | >80% |
| **Network** | <5% | 10-30% | 40-60% | >70% |

---

## 5. 集成测试策略

### 5.1 外部服务集成测试

#### 5.1.1 交易所 API 集成测试

**测试场景**：

```python
def test_binance_api_integration():
    """TC-EXT-001: Binance API集成测试"""
    connector = BinanceConnector(api_key='test_key', secret='test_secret')
    
    # 测试获取市场数据
    ticker = connector.get_ticker('BTCUSDT')
    assert ticker is not None
    assert ticker['symbol'] == 'BTCUSDT'
    assert ticker['price'] > 0
    
    # 测试获取K线数据
    klines = connector.get_klines('BTCUSDT', interval='1h', limit=100)
    assert len(klines) == 100
```

#### 5.1.2 模拟外部服务故障

**测试场景**：

```python
def test_exchange_api_timeout_handling():
    """TC-EXT-002: 交易所API超时处理"""
    with patch('binance.client.Client.get_ticker') as mock:
        mock.side_effect = requests.exceptions.Timeout()
        
        # 调用数据采集服务
        response = data_service.fetch_market_data('BTCUSDT')
        
        # 验证超时被正确处理
        assert response['status'] == 'error'
        assert 'timeout' in response['message'].lower()

def test_exchange_api_rate_limit_handling():
    """TC-EXT-003: 交易所API限流处理"""
    with patch('binance.client.Client.get_ticker') as mock:
        mock.side_effect = BinanceAPIException(None, 429)
        
        # 验证自动重试机制
        response = data_service.fetch_market_data('BTCUSDT')
        
        # 验证指数退避重试
        assert mock.call_count > 1, "未实现重试机制"
```

### 5.2 微服务间集成测试

#### 5.2.1 数据服务 → 策略引擎集成

**测试场景**：

```python
def test_data_to_strategy_integration():
    """TC-INT-001: 数据服务到策略引擎集成"""
    # 1. 数据服务推送市场数据到Kafka
    data_service.push_market_data({
        'symbol': 'BTCUSDT',
        'price': 50000,
        'timestamp': '2024-01-01T00:00:00Z'
    })
    
    # 2. 等待策略引擎消费
    time.sleep(2)
    
    # 3. 验证策略引擎收到数据
    assert strategy_engine.get_latest_price('BTCUSDT') == 50000
```

#### 5.2.2 策略引擎 → 交易服务集成

**测试场景**：

```python
def test_strategy_to_trading_integration():
    """TC-INT-002: 策略引擎到交易服务集成"""
    # 1. 策略引擎生成交易信号
    signal = strategy_engine.generate_signal({
        'strategy_id': 'strategy-123',
        'symbol': 'BTCUSDT',
        'side': 'BUY',
        'quantity': 0.01
    })
    
    # 2. 交易服务接收信号并下单
    order_id = trading_service.place_order(signal)
    
    # 3. 验证订单已创建
    order = trading_service.get_order(order_id)
    assert order['status'] in ['PENDING', 'FILLED']
```

---

## 6. 测试执行计划

### 6.1 测试阶段

| 阶段 | 时间 | 测试类型 | 覆盖率目标 | 负责人 |
|------|------|---------|-----------|--------|
| **阶段1** | Week 1 | 单元测试 | 70% | 开发团队 |
| **阶段2** | Week 2 | 安全测试 | 100%（高风险点） | QA Team |
| **阶段3** | Week 3 | 集成测试 | 关键流程覆盖 | QA Team |
| **阶段4** | Week 4 | 性能测试 | 性能基线验证 | QA Team |
| **阶段5** | Week 5 | E2E测试 | 核心业务流程 | QA Team |

### 6.2 CI/CD 集成

**自动触发条件**：
- ✅ 每次 commit 到 dev/main 分支
- ✅ 每次创建 Pull Request
- ✅ 每天定时（夜间完整测试）

**测试流程**：

```
1. 代码提交
   ↓
2. 触发 GitHub Actions
   ↓
3. 并行执行单元测试（多模块）
   ↓
4. 执行安全测试
   ↓
5. 执行集成测试
   ↓
6. (main分支) 执行性能测试
   ↓
7. 生成测试报告
   ↓
8. 上传覆盖率到 Codecov
   ↓
9. 质量门禁检查
   ↓
10. 通过/失败通知
```

### 6.3 测试报告

**测试报告包含**：
- 测试覆盖率（按模块）
- 通过/失败用例统计
- 性能测试结果（响应时间、吞吐量）
- 安全测试结果
- 趋势图（覆盖率变化、性能变化）

**报告格式**：
- JUnit XML（CI/CD集成）
- HTML报告（人类可读）
- Codecov Dashboard（覆盖率可视化）

---

## 7. 质量门禁

### 7.1 代码合并门禁

**必须满足以下条件才能合并到 main 分支**：

| 检查项 | 要求 | 强制性 |
|--------|------|--------|
| 单元测试 | 100%通过 | ✅ 是 |
| 覆盖率 | Rust≥85%, Java≥80%, Python≥75% | ✅ 是 |
| 安全测试 | 100%通过 | ✅ 是 |
| 集成测试 | 100%通过 | ✅ 是 |
| 代码审查 | 至少2人批准 | ✅ 是 |
| 静态代码分析 | 无严重问题 | ✅ 是 |
| 性能测试 | 响应时间符合基线 | ❌ 否（main分支） |

### 7.2 发布门禁

**发布到生产环境前必须满足**：

| 检查项 | 要求 | 验证方式 |
|--------|------|---------|
| 全部测试通过 | 100% | CI/CD报告 |
| 性能测试通过 | 符合基线 | k6报告 |
| 安全扫描 | 无高危漏洞 | Trivy扫描 |
| 压力测试 | 系统稳定 | 手动验证 |
| 回归测试 | 核心功能正常 | 手动验证 |
| 文档更新 | Release Notes | Git commit |

### 7.3 监控与告警

**生产环境持续监控**：

```yaml
# Prometheus告警规则
groups:
- name: quality
  rules:
  - alert: HighErrorRate
    expr: rate(http_requests_total{status=~"5.."}[5m]) > 0.01
    for: 5m
    annotations:
      summary: "错误率过高 (>1%)"
  
  - alert: SlowResponseTime
    expr: histogram_quantile(0.95, http_request_duration_seconds_bucket) > 0.5
    for: 10m
    annotations:
      summary: "P95响应时间超过500ms"
```

---

## 附录

### A. 测试工具清单

| 工具 | 用途 | 语言 |
|------|------|------|
| pytest | 单元测试 | Python |
| cargo test | 单元测试 | Rust |
| JUnit 5 | 单元测试 | Java |
| k6 | 性能测试 | JavaScript |
| Postman/Newman | API测试 | - |
| Testcontainers | 集成测试 | Java/Python |
| MockServer | Mock外部服务 | - |
| Codecov | 覆盖率报告 | - |

### B. 参考文档

- [测试策略详细版](./test-strategy.md)
- [高风险访问点测试](./high-risk-access-testing.md)
- [CI/CD集成指南](./ci-cd-integration.md)
- [测试数据管理](./test-data-management.md)

---

**最后更新**: 2024-12-20  
**下次审查**: 2025-01-20  
**维护团队**: QA Team  
**联系方式**: qa@hermesflow.com

