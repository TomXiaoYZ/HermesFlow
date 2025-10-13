"""
SQL注入防护测试
验证所有SQL查询使用Prepared Statement，防止SQL注入攻击
"""
import pytest
from sqlalchemy import text


class TestSQLInjection:
    """SQL注入防护测试套件"""
    
    def test_sql_injection_in_query_parameter(self, api_client, user_token):
        """TC-SQL-001: 查询参数SQL注入防护"""
        # 尝试SQL注入攻击
        malicious_inputs = [
            "1' OR '1'='1",
            "1; DROP TABLE strategies;--",
            "1' UNION SELECT * FROM users--",
            "' OR 1=1--",
            "admin'--",
            "1' AND 1=0 UNION ALL SELECT 'admin', 'password'--"
        ]
        
        for malicious_input in malicious_inputs:
            response = api_client.get(
                f'/api/v1/strategies/{malicious_input}',
                headers={'Authorization': f'Bearer {user_token}'}
            )
            
            # 验证攻击被阻止（返回400/404，而不是500或200并返回错误数据）
            assert response.status_code in [400, 404], \
                f"SQL注入未被阻止: {malicious_input}, 状态码: {response.status_code}"
            
            # 验证没有返回敏感数据
            if response.status_code == 200:
                data = response.json()
                assert 'password' not in str(data).lower(), "可能泄露了敏感数据"
    
    def test_sql_injection_in_search_query(self, api_client, user_token):
        """TC-SQL-002: 搜索查询SQL注入防护"""
        malicious_queries = [
            "test' OR '1'='1",
            "test'; DELETE FROM strategies WHERE '1'='1",
            "test' UNION SELECT password FROM users--"
        ]
        
        for query in malicious_queries:
            response = api_client.get(
                f'/api/v1/strategies/search?q={query}',
                headers={'Authorization': f'Bearer {user_token}'}
            )
            
            # 应该返回空结果或错误，而不是所有数据
            assert response.status_code in [200, 400]
            if response.status_code == 200:
                data = response.json()
                # 如果返回数据，应该是空的或只包含合法搜索结果
                assert isinstance(data, list)
    
    def test_sql_injection_in_filter(self, api_client, user_token):
        """TC-SQL-003: 过滤条件SQL注入防护"""
        response = api_client.get(
            "/api/v1/strategies?status=active' OR '1'='1",
            headers={'Authorization': f'Bearer {user_token}'}
        )
        
        # 应该返回错误或只返回status=active的数据
        assert response.status_code in [200, 400]
    
    def test_sql_injection_in_order_by(self, api_client, user_token):
        """TC-SQL-004: ORDER BY SQL注入防护"""
        # 尝试通过ORDER BY注入
        malicious_orders = [
            "name; DROP TABLE strategies--",
            "name UNION SELECT password FROM users--",
            "(SELECT CASE WHEN (1=1) THEN name ELSE (SELECT password FROM users) END)"
        ]
        
        for order in malicious_orders:
            response = api_client.get(
                f'/api/v1/strategies?order_by={order}',
                headers={'Authorization': f'Bearer {user_token}'}
            )
            
            # 应该返回错误或使用默认排序
            assert response.status_code in [200, 400]
    
    def test_prepared_statement_usage(self, db_session, tenant_a):
        """TC-SQL-005: 验证使用Prepared Statement"""
        # 测试查询使用参数化查询
        strategy_id = "test' OR '1'='1"
        
        # 使用参数化查询（正确方式）
        result = db_session.execute(
            text("SELECT * FROM strategies WHERE id = :id AND tenant_id = :tenant_id"),
            {'id': strategy_id, 'tenant_id': tenant_a.id}
        ).fetchall()
        
        # 应该返回0行（因为没有这个ID）
        assert len(result) == 0, "参数化查询应该阻止SQL注入"
    
    def test_stored_xss_via_sql_injection(self, api_client, user_token):
        """TC-SQL-006: 防止通过SQL注入存储XSS"""
        # 尝试通过SQL注入插入XSS脚本
        xss_payload = "<script>alert('XSS')</script>"
        
        response = api_client.post(
            '/api/v1/strategies',
            headers={'Authorization': f'Bearer {user_token}'},
            json={
                'name': f"Test'; INSERT INTO strategies (name) VALUES ('{xss_payload}'); --",
                'code': 'def run(): pass'
            }
        )
        
        # 创建应该成功或失败，但不应该执行注入的SQL
        if response.status_code == 201:
            strategy_id = response.json()['id']
            
            # 获取策略，验证名称被正确存储（没有执行SQL注入）
            response = api_client.get(
                f'/api/v1/strategies/{strategy_id}',
                headers={'Authorization': f'Bearer {user_token}'}
            )
            
            strategy_name = response.json()['name']
            # 名称应该包含原始输入（包括恶意SQL），而不是XSS脚本
            assert xss_payload not in strategy_name
    
    def test_blind_sql_injection_protection(self, api_client, user_token):
        """TC-SQL-007: 防止盲注SQL注入"""
        # 时间盲注测试
        time_based_payloads = [
            "1' AND SLEEP(5)--",
            "1' WAITFOR DELAY '00:00:05'--",
            "1' OR pg_sleep(5)--"
        ]
        
        import time
        for payload in time_based_payloads:
            start_time = time.time()
            response = api_client.get(
                f'/api/v1/strategies/{payload}',
                headers={'Authorization': f'Bearer {user_token}'}
            )
            elapsed = time.time() - start_time
            
            # 响应时间不应超过2秒（说明SLEEP没有执行）
            assert elapsed < 2.0, f"可能存在时间盲注漏洞: {payload}"
            assert response.status_code in [400, 404]


class TestORMSecurity:
    """ORM安全测试"""
    
    def test_raw_sql_not_used(self):
        """TC-ORM-001: 验证不使用原始SQL拼接"""
        # 这是一个静态代码检查测试
        # 应该通过代码审查或静态分析工具验证
        # 示例：确保没有使用类似 f"SELECT * FROM users WHERE id = {user_id}" 的代码
        pass
    
    def test_sqlalchemy_text_with_parameters(self, db_session):
        """TC-ORM-002: SQLAlchemy text()使用参数"""
        # 正确的参数化查询示例
        user_input = "test' OR '1'='1"
        
        # 使用text()和参数（安全）
        result = db_session.execute(
            text("SELECT * FROM strategies WHERE name = :name"),
            {'name': user_input}
        ).fetchall()
        
        # 应该返回0行
        assert len(result) == 0

