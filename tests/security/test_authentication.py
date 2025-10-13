"""
认证与授权测试
测试JWT、RBAC、Session管理
"""
import pytest
import jwt
from datetime import datetime, timedelta

class TestAuthentication:
    """认证测试套件"""
    
    def test_expired_token_rejected(self, api_client, expired_token):
        """TC-AUTH-001: 过期Token被拒绝"""
        response = api_client.get(
            '/api/v1/strategies',
            headers={'Authorization': f'Bearer {expired_token}'}
        )
        assert response.status_code == 401
        assert 'expired' in response.json()['error'].lower()
    
    def test_tampered_token_rejected(self, api_client, valid_token):
        """TC-AUTH-002: 篡改Token被拒绝"""
        # 篡改Token（修改最后几个字符）
        tampered_token = valid_token[:-5] + 'xxxxx'
        
        response = api_client.get(
            '/api/v1/strategies',
            headers={'Authorization': f'Bearer {tampered_token}'}
        )
        assert response.status_code == 401
    
    def test_no_token_rejected(self, api_client):
        """TC-AUTH-003: 无Token请求被拒绝"""
        response = api_client.get('/api/v1/strategies')
        assert response.status_code == 401
    
    def test_valid_token_accepted(self, api_client, valid_token):
        """TC-AUTH-004: 有效Token正常访问"""
        response = api_client.get(
            '/api/v1/strategies',
            headers={'Authorization': f'Bearer {valid_token}'}
        )
        assert response.status_code == 200


class TestRBAC:
    """RBAC权限测试套件"""
    
    def test_user_cannot_access_admin_api(self, api_client, user_token):
        """TC-RBAC-001: 普通用户无法访问管理员API"""
        response = api_client.post(
            '/api/v1/admin/users',
            headers={'Authorization': f'Bearer {user_token}'},
            json={'email': 'newuser@example.com', 'role': 'admin'}
        )
        assert response.status_code == 403
    
    def test_admin_can_access_admin_api(self, api_client, admin_token):
        """TC-RBAC-002: 管理员可以访问管理员API"""
        response = api_client.post(
            '/api/v1/admin/users',
            headers={'Authorization': f'Bearer {admin_token}'},
            json={'email': 'newuser@example.com', 'role': 'user'}
        )
        assert response.status_code == 201

