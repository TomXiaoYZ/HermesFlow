"""
RBAC权限详细测试
验证基于角色的访问控制
"""
import pytest


class TestRBACPermissions:
    """RBAC权限矩阵测试"""
    
    # 权限矩阵定义
    PERMISSION_MATRIX = {
        'admin': {
            'strategies': ['create', 'read', 'update', 'delete', 'list'],
            'users': ['create', 'read', 'update', 'delete', 'list'],
            'orders': ['create', 'read', 'update', 'delete', 'list'],
            'reports': ['read', 'export'],
            'settings': ['read', 'update']
        },
        'trader': {
            'strategies': ['create', 'read', 'update', 'delete', 'list'],
            'orders': ['create', 'read', 'list'],
            'reports': ['read'],
            'settings': ['read']
        },
        'analyst': {
            'strategies': ['read', 'list'],
            'orders': ['read', 'list'],
            'reports': ['read', 'export'],
            'settings': ['read']
        },
        'viewer': {
            'strategies': ['read', 'list'],
            'orders': ['read', 'list'],
            'reports': ['read'],
            'settings': []
        }
    }
    
    def test_admin_full_access(self, api_client, admin_token):
        """TC-RBAC-001: 管理员拥有完全访问权限"""
        # 创建策略
        response = api_client.post(
            '/api/v1/strategies',
            headers={'Authorization': f'Bearer {admin_token}'},
            json={'name': 'Admin Strategy', 'code': 'def run(): pass'}
        )
        assert response.status_code == 201
        strategy_id = response.json()['id']
        
        # 读取策略
        response = api_client.get(
            f'/api/v1/strategies/{strategy_id}',
            headers={'Authorization': f'Bearer {admin_token}'}
        )
        assert response.status_code == 200
        
        # 更新策略
        response = api_client.put(
            f'/api/v1/strategies/{strategy_id}',
            headers={'Authorization': f'Bearer {admin_token}'},
            json={'name': 'Updated Strategy'}
        )
        assert response.status_code == 200
        
        # 删除策略
        response = api_client.delete(
            f'/api/v1/strategies/{strategy_id}',
            headers={'Authorization': f'Bearer {admin_token}'}
        )
        assert response.status_code == 204
    
    def test_trader_cannot_manage_users(self, api_client, trader_token):
        """TC-RBAC-002: 交易员无法管理用户"""
        # 尝试创建用户
        response = api_client.post(
            '/api/v1/admin/users',
            headers={'Authorization': f'Bearer {trader_token}'},
            json={'email': 'newuser@example.com', 'role': 'viewer'}
        )
        assert response.status_code == 403
        
        # 尝试删除用户
        response = api_client.delete(
            '/api/v1/admin/users/some-user-id',
            headers={'Authorization': f'Bearer {trader_token}'}
        )
        assert response.status_code == 403
    
    def test_analyst_read_only_strategies(self, api_client, analyst_token):
        """TC-RBAC-003: 分析师只能读取策略"""
        # 可以读取策略列表
        response = api_client.get(
            '/api/v1/strategies',
            headers={'Authorization': f'Bearer {analyst_token}'}
        )
        assert response.status_code == 200
        
        # 不能创建策略
        response = api_client.post(
            '/api/v1/strategies',
            headers={'Authorization': f'Bearer {analyst_token}'},
            json={'name': 'Analyst Strategy', 'code': 'def run(): pass'}
        )
        assert response.status_code == 403
        
        # 不能更新策略
        response = api_client.put(
            '/api/v1/strategies/some-id',
            headers={'Authorization': f'Bearer {analyst_token}'},
            json={'name': 'Updated'}
        )
        assert response.status_code == 403
        
        # 不能删除策略
        response = api_client.delete(
            '/api/v1/strategies/some-id',
            headers={'Authorization': f'Bearer {analyst_token}'}
        )
        assert response.status_code == 403
    
    def test_viewer_minimal_access(self, api_client, viewer_token):
        """TC-RBAC-004: 查看者只能查看"""
        # 可以读取策略
        response = api_client.get(
            '/api/v1/strategies',
            headers={'Authorization': f'Bearer {viewer_token}'}
        )
        assert response.status_code == 200
        
        # 不能创建策略
        response = api_client.post(
            '/api/v1/strategies',
            headers={'Authorization': f'Bearer {viewer_token}'},
            json={'name': 'Viewer Strategy', 'code': 'def run(): pass'}
        )
        assert response.status_code == 403
        
        # 不能访问设置
        response = api_client.get(
            '/api/v1/settings',
            headers={'Authorization': f'Bearer {viewer_token}'}
        )
        assert response.status_code == 403
    
    def test_trader_can_create_orders(self, api_client, trader_token):
        """TC-RBAC-005: 交易员可以创建订单"""
        response = api_client.post(
            '/api/v1/orders',
            headers={'Authorization': f'Bearer {trader_token}'},
            json={
                'symbol': 'BTCUSDT',
                'side': 'buy',
                'quantity': 0.1,
                'price': 50000
            }
        )
        assert response.status_code == 201
    
    def test_analyst_cannot_create_orders(self, api_client, analyst_token):
        """TC-RBAC-006: 分析师不能创建订单"""
        response = api_client.post(
            '/api/v1/orders',
            headers={'Authorization': f'Bearer {analyst_token}'},
            json={
                'symbol': 'BTCUSDT',
                'side': 'buy',
                'quantity': 0.1,
                'price': 50000
            }
        )
        assert response.status_code == 403
    
    def test_analyst_can_export_reports(self, api_client, analyst_token):
        """TC-RBAC-007: 分析师可以导出报告"""
        response = api_client.get(
            '/api/v1/reports/export',
            headers={'Authorization': f'Bearer {analyst_token}'}
        )
        assert response.status_code == 200
    
    def test_viewer_cannot_export_reports(self, api_client, viewer_token):
        """TC-RBAC-008: 查看者不能导出报告"""
        response = api_client.get(
            '/api/v1/reports/export',
            headers={'Authorization': f'Bearer {viewer_token}'}
        )
        assert response.status_code == 403
    
    def test_role_escalation_prevention(self, api_client, trader_token):
        """TC-RBAC-009: 防止角色提升"""
        # 尝试将自己提升为管理员
        response = api_client.put(
            '/api/v1/users/me',
            headers={'Authorization': f'Bearer {trader_token}'},
            json={'role': 'admin'}
        )
        
        # 应该被拒绝或忽略角色字段
        assert response.status_code in [403, 400] or \
               response.json().get('role') != 'admin'
    
    def test_cross_tenant_access_denied(self, api_client, user_token_tenant_a):
        """TC-RBAC-010: 拒绝跨租户访问"""
        # 用户A尝试访问租户B的资源
        response = api_client.get(
            '/api/v1/strategies/tenant-b-strategy-id',
            headers={'Authorization': f'Bearer {user_token_tenant_a}'}
        )
        assert response.status_code in [403, 404]


class TestResourceOwnership:
    """资源所有权测试"""
    
    def test_user_can_access_own_resources(self, api_client, user_token):
        """TC-OWN-001: 用户可以访问自己的资源"""
        # 创建策略
        response = api_client.post(
            '/api/v1/strategies',
            headers={'Authorization': f'Bearer {user_token}'},
            json={'name': 'My Strategy', 'code': 'def run(): pass'}
        )
        assert response.status_code == 201
        strategy_id = response.json()['id']
        
        # 访问自己的策略
        response = api_client.get(
            f'/api/v1/strategies/{strategy_id}',
            headers={'Authorization': f'Bearer {user_token}'}
        )
        assert response.status_code == 200
    
    def test_user_cannot_access_others_resources(self, api_client, user_token_a, user_token_b):
        """TC-OWN-002: 用户不能访问他人的资源"""
        # 用户A创建策略
        response = api_client.post(
            '/api/v1/strategies',
            headers={'Authorization': f'Bearer {user_token_a}'},
            json={'name': 'User A Strategy', 'code': 'def run(): pass'}
        )
        strategy_id = response.json()['id']
        
        # 用户B尝试访问用户A的策略
        response = api_client.get(
            f'/api/v1/strategies/{strategy_id}',
            headers={'Authorization': f'Bearer {user_token_b}'}
        )
        assert response.status_code in [403, 404]
    
    def test_admin_can_access_all_resources(self, api_client, admin_token, user_token):
        """TC-OWN-003: 管理员可以访问所有资源"""
        # 普通用户创建策略
        response = api_client.post(
            '/api/v1/strategies',
            headers={'Authorization': f'Bearer {user_token}'},
            json={'name': 'User Strategy', 'code': 'def run(): pass'}
        )
        strategy_id = response.json()['id']
        
        # 管理员访问该策略
        response = api_client.get(
            f'/api/v1/strategies/{strategy_id}',
            headers={'Authorization': f'Bearer {admin_token}'}
        )
        assert response.status_code == 200


class TestPermissionInheritance:
    """权限继承测试"""
    
    def test_admin_inherits_all_permissions(self, api_client, admin_token):
        """TC-INH-001: 管理员继承所有权限"""
        # 管理员应该能执行所有角色的操作
        # 交易员的操作
        response = api_client.post(
            '/api/v1/orders',
            headers={'Authorization': f'Bearer {admin_token}'},
            json={'symbol': 'BTCUSDT', 'side': 'buy', 'quantity': 0.1}
        )
        assert response.status_code == 201
        
        # 分析师的操作
        response = api_client.get(
            '/api/v1/reports/export',
            headers={'Authorization': f'Bearer {admin_token}'}
        )
        assert response.status_code == 200
    
    def test_role_downgrade_restrictions(self, api_client, admin_token, user_id):
        """TC-INH-002: 角色降级限制"""
        # 管理员将用户从trader降级为viewer
        response = api_client.put(
            f'/api/v1/admin/users/{user_id}',
            headers={'Authorization': f'Bearer {admin_token}'},
            json={'role': 'viewer'}
        )
        assert response.status_code == 200
        
        # TODO: 验证用户的现有会话权限立即更新


class TestAPIEndpointPermissions:
    """API端点权限测试"""
    
    PROTECTED_ENDPOINTS = [
        ('POST', '/api/v1/admin/users', ['admin']),
        ('DELETE', '/api/v1/admin/users/{id}', ['admin']),
        ('POST', '/api/v1/orders', ['admin', 'trader']),
        ('PUT', '/api/v1/strategies/{id}', ['admin', 'trader']),
        ('DELETE', '/api/v1/strategies/{id}', ['admin', 'trader']),
        ('GET', '/api/v1/reports/export', ['admin', 'analyst']),
        ('PUT', '/api/v1/settings', ['admin']),
    ]
    
    def test_endpoint_permission_matrix(self, api_client, tokens_by_role):
        """TC-API-001: 验证端点权限矩阵"""
        for method, endpoint, allowed_roles in self.PROTECTED_ENDPOINTS:
            for role, token in tokens_by_role.items():
                # 替换端点中的占位符
                test_endpoint = endpoint.replace('{id}', 'test-id')
                
                # 发送请求
                if method == 'GET':
                    response = api_client.get(test_endpoint, headers={'Authorization': f'Bearer {token}'})
                elif method == 'POST':
                    response = api_client.post(test_endpoint, headers={'Authorization': f'Bearer {token}'}, json={})
                elif method == 'PUT':
                    response = api_client.put(test_endpoint, headers={'Authorization': f'Bearer {token}'}, json={})
                elif method == 'DELETE':
                    response = api_client.delete(test_endpoint, headers={'Authorization': f'Bearer {token}'})
                
                # 验证权限
                if role in allowed_roles:
                    assert response.status_code not in [403, 401], \
                        f"{role}应该可以访问{method} {endpoint}"
                else:
                    assert response.status_code == 403, \
                        f"{role}不应该可以访问{method} {endpoint}"

