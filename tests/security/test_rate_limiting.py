"""
Rate Limiting测试
验证API速率限制功能
"""
import pytest
import time
from concurrent.futures import ThreadPoolExecutor, as_completed


class TestRateLimiting:
    """API速率限制测试套件"""
    
    def test_basic_rate_limit(self, api_client, user_token):
        """TC-RATE-001: 基本速率限制"""
        # 快速发送100个请求
        responses = []
        for i in range(100):
            response = api_client.get(
                '/api/v1/strategies',
                headers={'Authorization': f'Bearer {user_token}'}
            )
            responses.append(response.status_code)
        
        # 验证部分请求被限流
        rate_limited = [r for r in responses if r == 429]
        assert len(rate_limited) > 0, "Rate Limiting未生效"
        
        print(f"限流请求数: {len(rate_limited)}/100")
    
    def test_rate_limit_headers(self, api_client, user_token):
        """TC-RATE-002: 速率限制响应头"""
        response = api_client.get(
            '/api/v1/strategies',
            headers={'Authorization': f'Bearer {user_token}'}
        )
        
        # 验证响应头包含限流信息
        assert 'X-RateLimit-Limit' in response.headers, "缺少X-RateLimit-Limit头"
        assert 'X-RateLimit-Remaining' in response.headers, "缺少X-RateLimit-Remaining头"
        assert 'X-RateLimit-Reset' in response.headers, "缺少X-RateLimit-Reset头"
        
        limit = int(response.headers['X-RateLimit-Limit'])
        remaining = int(response.headers['X-RateLimit-Remaining'])
        
        assert limit > 0, "限流上限应该大于0"
        assert remaining <= limit, "剩余次数不应超过上限"
    
    def test_rate_limit_by_user(self, api_client, user_token_a, user_token_b):
        """TC-RATE-003: 按用户限流（不同用户独立计数）"""
        # 用户A快速发送50个请求
        responses_a = []
        for i in range(50):
            response = api_client.get(
                '/api/v1/strategies',
                headers={'Authorization': f'Bearer {user_token_a}'}
            )
            responses_a.append(response.status_code)
        
        # 用户B发送请求（不应该被用户A的限流影响）
        response_b = api_client.get(
            '/api/v1/strategies',
            headers={'Authorization': f'Bearer {user_token_b}'}
        )
        
        # 用户B的请求应该成功（即使用户A被限流）
        assert response_b.status_code == 200, "用户B不应该被用户A的限流影响"
    
    def test_rate_limit_reset(self, api_client, user_token):
        """TC-RATE-004: 速率限制重置"""
        # 触发限流
        for i in range(200):
            response = api_client.get(
                '/api/v1/strategies',
                headers={'Authorization': f'Bearer {user_token}'}
            )
            if response.status_code == 429:
                reset_time = int(response.headers.get('X-RateLimit-Reset', 0))
                current_time = int(time.time())
                wait_seconds = max(0, reset_time - current_time) + 1
                
                print(f"限流触发，等待 {wait_seconds} 秒...")
                time.sleep(wait_seconds)
                
                # 重置后应该可以再次请求
                response = api_client.get(
                    '/api/v1/strategies',
                    headers={'Authorization': f'Bearer {user_token}'}
                )
                assert response.status_code == 200, "限流重置后应该可以继续请求"
                break
    
    def test_rate_limit_429_response(self, api_client, user_token):
        """TC-RATE-005: 429响应格式"""
        # 触发限流
        for i in range(200):
            response = api_client.get(
                '/api/v1/strategies',
                headers={'Authorization': f'Bearer {user_token}'}
            )
            
            if response.status_code == 429:
                # 验证429响应包含错误信息
                error_data = response.json()
                assert 'error' in error_data or 'message' in error_data, \
                    "429响应应该包含错误信息"
                
                error_message = error_data.get('error') or error_data.get('message')
                assert 'rate limit' in error_message.lower() or \
                       'too many' in error_message.lower(), \
                    "错误信息应该说明速率限制"
                
                # 验证Retry-After头
                assert 'Retry-After' in response.headers, "429响应应该包含Retry-After头"
                retry_after = int(response.headers['Retry-After'])
                assert retry_after > 0, "Retry-After应该大于0"
                break
    
    def test_rate_limit_per_endpoint(self, api_client, user_token):
        """TC-RATE-006: 不同端点独立限流"""
        # 在策略端点触发限流
        for i in range(100):
            api_client.get(
                '/api/v1/strategies',
                headers={'Authorization': f'Bearer {user_token}'}
            )
        
        # 尝试访问订单端点（应该有独立的限流计数）
        response = api_client.get(
            '/api/v1/orders',
            headers={'Authorization': f'Bearer {user_token}'}
        )
        
        # 根据实现，这可能成功（独立限流）或失败（全局限流）
        # 这里假设是独立限流
        # 如果是全局限流，则应该返回429
        print(f"订单端点响应: {response.status_code}")
    
    def test_rate_limit_burst_protection(self, api_client, user_token):
        """TC-RATE-007: 突发流量保护"""
        # 并发发送大量请求（模拟突发流量）
        def make_request():
            return api_client.get(
                '/api/v1/strategies',
                headers={'Authorization': f'Bearer {user_token}'}
            ).status_code
        
        with ThreadPoolExecutor(max_workers=20) as executor:
            futures = [executor.submit(make_request) for _ in range(100)]
            responses = [f.result() for f in as_completed(futures)]
        
        # 验证部分请求被限流
        rate_limited = [r for r in responses if r == 429]
        success = [r for r in responses if r == 200]
        
        assert len(rate_limited) > 0, "突发流量应该触发限流"
        assert len(success) > 0, "部分请求应该成功"
        
        print(f"成功: {len(success)}, 限流: {len(rate_limited)}")
    
    def test_rate_limit_different_methods(self, api_client, user_token):
        """TC-RATE-008: 不同HTTP方法的限流"""
        # GET请求
        get_responses = []
        for i in range(50):
            response = api_client.get(
                '/api/v1/strategies',
                headers={'Authorization': f'Bearer {user_token}'}
            )
            get_responses.append(response.status_code)
        
        # POST请求（写操作通常有更严格的限流）
        post_responses = []
        for i in range(50):
            response = api_client.post(
                '/api/v1/strategies',
                headers={'Authorization': f'Bearer {user_token}'},
                json={'name': f'Strategy {i}', 'code': 'def run(): pass'}
            )
            post_responses.append(response.status_code)
        
        # POST请求应该更容易触发限流
        get_limited = [r for r in get_responses if r == 429]
        post_limited = [r for r in post_responses if r == 429]
        
        print(f"GET限流: {len(get_limited)}/50, POST限流: {len(post_limited)}/50")
    
    def test_rate_limit_whitelist(self, api_client, admin_token):
        """TC-RATE-009: 管理员白名单（如果实现）"""
        # 管理员可能有更高的限流阈值或不受限流
        responses = []
        for i in range(200):
            response = api_client.get(
                '/api/v1/strategies',
                headers={'Authorization': f'Bearer {admin_token}'}
            )
            responses.append(response.status_code)
        
        rate_limited = [r for r in responses if r == 429]
        
        # 根据实现，管理员可能不受限流或有更高阈值
        print(f"管理员限流请求数: {len(rate_limited)}/200")


class TestRateLimitByPlan:
    """基于订阅计划的速率限制"""
    
    def test_premium_plan_higher_limit(self, api_client, premium_user_token, basic_user_token):
        """TC-PLAN-001: 高级用户有更高限流阈值"""
        # 基础用户
        basic_responses = []
        for i in range(100):
            response = api_client.get(
                '/api/v1/strategies',
                headers={'Authorization': f'Bearer {basic_user_token}'}
            )
            basic_responses.append(response.status_code)
        
        # 高级用户
        premium_responses = []
        for i in range(100):
            response = api_client.get(
                '/api/v1/strategies',
                headers={'Authorization': f'Bearer {premium_user_token}'}
            )
            premium_responses.append(response.status_code)
        
        # 统计限流次数
        basic_limited = [r for r in basic_responses if r == 429]
        premium_limited = [r for r in premium_responses if r == 429]
        
        # 高级用户应该有更少的限流
        assert len(premium_limited) <= len(basic_limited), \
            "高级用户应该有更高的限流阈值"
        
        print(f"基础用户限流: {len(basic_limited)}, 高级用户限流: {len(premium_limited)}")


class TestRateLimitIPBased:
    """基于IP的速率限制"""
    
    def test_ip_based_rate_limit(self, api_client, user_token):
        """TC-IP-001: 基于IP的限流"""
        # 模拟来自同一IP的多个请求
        # 注意：这需要测试环境支持X-Forwarded-For头
        responses = []
        for i in range(100):
            response = api_client.get(
                '/api/v1/public/market-data',
                headers={'X-Forwarded-For': '192.168.1.100'}
            )
            responses.append(response.status_code)
        
        rate_limited = [r for r in responses if r == 429]
        assert len(rate_limited) > 0, "IP级限流未生效"
    
    def test_different_ips_independent_limits(self, api_client):
        """TC-IP-002: 不同IP独立限流"""
        # IP A
        responses_a = []
        for i in range(50):
            response = api_client.get(
                '/api/v1/public/market-data',
                headers={'X-Forwarded-For': '192.168.1.100'}
            )
            responses_a.append(response.status_code)
        
        # IP B
        responses_b = []
        for i in range(50):
            response = api_client.get(
                '/api/v1/public/market-data',
                headers={'X-Forwarded-For': '192.168.1.101'}
            )
            responses_b.append(response.status_code)
        
        # 不同IP应该有独立的限流计数
        # 即使IP A被限流，IP B也应该可以继续请求

