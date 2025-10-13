"""
Pytest全局配置文件
定义通用Fixtures和测试配置
"""
import pytest
import os
from sqlalchemy import create_engine, text
from sqlalchemy.orm import sessionmaker, Session
from redis import Redis
from kafka import KafkaProducer, KafkaConsumer
import jwt
from datetime import datetime, timedelta


# ============================================================================
# 测试配置
# ============================================================================

@pytest.fixture(scope='session')
def test_config():
    """测试环境配置"""
    return {
        'database_url': os.getenv('DATABASE_URL', 'postgresql://testuser:testpassword@localhost:5432/hermesflow_test'),
        'redis_url': os.getenv('REDIS_URL', 'redis://localhost:6379'),
        'kafka_brokers': os.getenv('KAFKA_BROKERS', 'localhost:9092').split(','),
        'clickhouse_url': os.getenv('CLICKHOUSE_URL', 'http://localhost:8123'),
        'jwt_secret': os.getenv('JWT_SECRET', 'test-secret-key-do-not-use-in-production'),
        'jwt_algorithm': 'HS256'
    }


# ============================================================================
# 数据库Fixtures
# ============================================================================

@pytest.fixture(scope='session')
def db_engine(test_config):
    """数据库引擎（session级别，整个测试会话共享）"""
    engine = create_engine(test_config['database_url'])
    yield engine
    engine.dispose()


@pytest.fixture(scope='function')
def db_session(db_engine):
    """数据库会话（function级别，每个测试独立）"""
    connection = db_engine.connect()
    transaction = connection.begin()
    
    SessionLocal = sessionmaker(bind=connection)
    session = SessionLocal()
    
    yield session
    
    # 清理：回滚事务，确保测试间隔离
    session.close()
    transaction.rollback()
    connection.close()


# ============================================================================
# Redis Fixtures
# ============================================================================

@pytest.fixture(scope='function')
def redis_client(test_config):
    """Redis客户端（function级别）"""
    client = Redis.from_url(test_config['redis_url'], decode_responses=True)
    
    yield client
    
    # 清理：删除所有测试数据
    client.flushdb()
    client.close()


# ============================================================================
# Kafka Fixtures
# ============================================================================

@pytest.fixture(scope='session')
def kafka_producer(test_config):
    """Kafka生产者（session级别）"""
    producer = KafkaProducer(
        bootstrap_servers=test_config['kafka_brokers'],
        value_serializer=lambda v: v.encode('utf-8') if isinstance(v, str) else v
    )
    
    yield producer
    
    producer.close()


@pytest.fixture(scope='function')
def kafka_consumer(test_config):
    """Kafka消费者（function级别）"""
    consumer = KafkaConsumer(
        bootstrap_servers=test_config['kafka_brokers'],
        auto_offset_reset='earliest',
        consumer_timeout_ms=1000
    )
    
    yield consumer
    
    consumer.close()


# ============================================================================
# 租户Fixtures
# ============================================================================

@pytest.fixture(scope='function')
def tenant_a(db_session):
    """租户A"""
    from tests.fixtures.tenants import create_tenant
    tenant = create_tenant(db_session, 'tenant-a', 'Test Tenant A', 'premium')
    yield tenant
    # 清理由事务回滚处理


@pytest.fixture(scope='function')
def tenant_b(db_session):
    """租户B"""
    from tests.fixtures.tenants import create_tenant
    tenant = create_tenant(db_session, 'tenant-b', 'Test Tenant B', 'basic')
    yield tenant


# ============================================================================
# 用户Fixtures
# ============================================================================

@pytest.fixture(scope='function')
def user_tenant_a(db_session, tenant_a):
    """租户A的普通用户"""
    from tests.fixtures.users import create_user
    user = create_user(
        db_session,
        user_id='user-a-1',
        email='user-a@example.com',
        tenant_id=tenant_a.id,
        role='user'
    )
    yield user


@pytest.fixture(scope='function')
def admin_tenant_a(db_session, tenant_a):
    """租户A的管理员"""
    from tests.fixtures.users import create_user
    user = create_user(
        db_session,
        user_id='admin-a-1',
        email='admin-a@example.com',
        tenant_id=tenant_a.id,
        role='admin'
    )
    yield user


@pytest.fixture(scope='function')
def trader_tenant_a(db_session, tenant_a):
    """租户A的交易员"""
    from tests.fixtures.users import create_user
    user = create_user(
        db_session,
        user_id='trader-a-1',
        email='trader-a@example.com',
        tenant_id=tenant_a.id,
        role='trader'
    )
    yield user


@pytest.fixture(scope='function')
def analyst_tenant_a(db_session, tenant_a):
    """租户A的分析师"""
    from tests.fixtures.users import create_user
    user = create_user(
        db_session,
        user_id='analyst-a-1',
        email='analyst-a@example.com',
        tenant_id=tenant_a.id,
        role='analyst'
    )
    yield user


@pytest.fixture(scope='function')
def viewer_tenant_a(db_session, tenant_a):
    """租户A的查看者"""
    from tests.fixtures.users import create_user
    user = create_user(
        db_session,
        user_id='viewer-a-1',
        email='viewer-a@example.com',
        tenant_id=tenant_a.id,
        role='viewer'
    )
    yield user


# ============================================================================
# JWT Token Fixtures
# ============================================================================

def generate_token(user_id: str, tenant_id: str, role: str, config: dict, expired: bool = False) -> str:
    """生成JWT Token"""
    now = datetime.utcnow()
    exp_time = now - timedelta(hours=1) if expired else now + timedelta(hours=1)
    
    payload = {
        'user_id': user_id,
        'tenant_id': tenant_id,
        'role': role,
        'iat': now,
        'exp': exp_time
    }
    
    return jwt.encode(payload, config['jwt_secret'], algorithm=config['jwt_algorithm'])


@pytest.fixture
def user_token(user_tenant_a, test_config):
    """普通用户Token"""
    return generate_token(
        user_tenant_a.id,
        user_tenant_a.tenant_id,
        'user',
        test_config
    )


@pytest.fixture
def admin_token(admin_tenant_a, test_config):
    """管理员Token"""
    return generate_token(
        admin_tenant_a.id,
        admin_tenant_a.tenant_id,
        'admin',
        test_config
    )


@pytest.fixture
def trader_token(trader_tenant_a, test_config):
    """交易员Token"""
    return generate_token(
        trader_tenant_a.id,
        trader_tenant_a.tenant_id,
        'trader',
        test_config
    )


@pytest.fixture
def analyst_token(analyst_tenant_a, test_config):
    """分析师Token"""
    return generate_token(
        analyst_tenant_a.id,
        analyst_tenant_a.tenant_id,
        'analyst',
        test_config
    )


@pytest.fixture
def viewer_token(viewer_tenant_a, test_config):
    """查看者Token"""
    return generate_token(
        viewer_tenant_a.id,
        viewer_tenant_a.tenant_id,
        'viewer',
        test_config
    )


@pytest.fixture
def expired_token(user_tenant_a, test_config):
    """过期的Token"""
    return generate_token(
        user_tenant_a.id,
        user_tenant_a.tenant_id,
        'user',
        test_config,
        expired=True
    )


@pytest.fixture
def user_token_tenant_a(user_tenant_a, test_config):
    """租户A用户Token"""
    return generate_token(
        user_tenant_a.id,
        'tenant-a',
        'user',
        test_config
    )


@pytest.fixture
def user_token_a(db_session, tenant_a, test_config):
    """用户A Token"""
    from tests.fixtures.users import create_user
    user = create_user(db_session, 'user-a', 'usera@example.com', tenant_a.id, 'user')
    return generate_token(user.id, user.tenant_id, 'user', test_config)


@pytest.fixture
def user_token_b(db_session, tenant_a, test_config):
    """用户B Token（同租户不同用户）"""
    from tests.fixtures.users import create_user
    user = create_user(db_session, 'user-b', 'userb@example.com', tenant_a.id, 'user')
    return generate_token(user.id, user.tenant_id, 'user', test_config)


@pytest.fixture
def tokens_by_role(admin_token, trader_token, analyst_token, viewer_token):
    """按角色组织的Token字典"""
    return {
        'admin': admin_token,
        'trader': trader_token,
        'analyst': analyst_token,
        'viewer': viewer_token
    }


@pytest.fixture
def premium_user_token(db_session, test_config):
    """高级计划用户Token"""
    from tests.fixtures.tenants import create_tenant
    from tests.fixtures.users import create_user
    
    tenant = create_tenant(db_session, 'tenant-premium', 'Premium Tenant', 'premium')
    user = create_user(db_session, 'premium-user', 'premium@example.com', tenant.id, 'user')
    
    return generate_token(user.id, user.tenant_id, 'user', test_config)


@pytest.fixture
def basic_user_token(db_session, test_config):
    """基础计划用户Token"""
    from tests.fixtures.tenants import create_tenant
    from tests.fixtures.users import create_user
    
    tenant = create_tenant(db_session, 'tenant-basic', 'Basic Tenant', 'basic')
    user = create_user(db_session, 'basic-user', 'basic@example.com', tenant.id, 'user')
    
    return generate_token(user.id, user.tenant_id, 'user', test_config)


# ============================================================================
# API客户端Fixtures
# ============================================================================

@pytest.fixture
def api_client():
    """API测试客户端"""
    # 这里使用requests或httpx作为HTTP客户端
    # 根据实际API框架选择合适的测试客户端
    
    # 示例：使用requests
    import requests
    
    class APIClient:
        def __init__(self, base_url='http://localhost:8000'):
            self.base_url = base_url
            self.session = requests.Session()
        
        def get(self, path, **kwargs):
            return self.session.get(f'{self.base_url}{path}', **kwargs)
        
        def post(self, path, **kwargs):
            return self.session.post(f'{self.base_url}{path}', **kwargs)
        
        def put(self, path, **kwargs):
            return self.session.put(f'{self.base_url}{path}', **kwargs)
        
        def delete(self, path, **kwargs):
            return self.session.delete(f'{self.base_url}{path}', **kwargs)
    
    return APIClient()


# ============================================================================
# 市场数据Fixtures
# ============================================================================

@pytest.fixture
def btc_1h_data():
    """BTC 1小时K线数据"""
    import pandas as pd
    
    return pd.DataFrame({
        'timestamp': pd.date_range('2024-01-01', periods=1000, freq='1H'),
        'open': [50000 + i * 10 for i in range(1000)],
        'high': [50100 + i * 10 for i in range(1000)],
        'low': [49900 + i * 10 for i in range(1000)],
        'close': [50050 + i * 10 for i in range(1000)],
        'volume': [1000 + i for i in range(1000)]
    })


@pytest.fixture
def multi_symbol_data():
    """多币种数据"""
    import pandas as pd
    
    symbols = ['BTCUSDT', 'ETHUSDT', 'BNBUSDT']
    data = {}
    
    for symbol in symbols:
        data[symbol] = pd.DataFrame({
            'timestamp': pd.date_range('2024-01-01', periods=500, freq='5T'),
            'price': [50000 + i for i in range(500)],
            'volume': [100 + i for i in range(500)]
        })
    
    return data


# ============================================================================
# Pytest配置
# ============================================================================

def pytest_configure(config):
    """Pytest配置"""
    # 添加自定义标记
    config.addinivalue_line("markers", "slow: 标记慢速测试")
    config.addinivalue_line("markers", "integration: 标记集成测试")
    config.addinivalue_line("markers", "security: 标记安全测试")
    config.addinivalue_line("markers", "performance: 标记性能测试")


def pytest_collection_modifyitems(config, items):
    """修改测试收集"""
    for item in items:
        # 为security目录下的测试自动添加security标记
        if "security" in str(item.fspath):
            item.add_marker(pytest.mark.security)
        
        # 为integration目录下的测试自动添加integration标记
        if "integration" in str(item.fspath):
            item.add_marker(pytest.mark.integration)
        
        # 为performance目录下的测试自动添加performance标记
        if "performance" in str(item.fspath):
            item.add_marker(pytest.mark.performance)

