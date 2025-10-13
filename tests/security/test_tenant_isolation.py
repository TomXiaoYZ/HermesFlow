"""
多租户隔离测试
测试PostgreSQL RLS、Redis Key隔离、Kafka Topic分区
"""
import pytest
from sqlalchemy import text
from redis import Redis
import json
import time

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


class TestRedisIsolation:
    """Redis Key隔离测试"""
    
    def test_redis_key_prefix_isolation(self, redis_client, tenant_a, tenant_b):
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
    
    def test_redis_keys_command_isolation(self, redis_client, tenant_a, tenant_b):
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

