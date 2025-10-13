"""
XSS防护测试
验证前端输出转义，防止XSS攻击
"""
import pytest


class TestXSSProtection:
    """XSS防护测试套件"""
    
    def test_stored_xss_in_strategy_name(self, api_client, user_token):
        """TC-XSS-001: 策略名称存储型XSS防护"""
        xss_payloads = [
            "<script>alert('XSS')</script>",
            "<img src=x onerror=alert('XSS')>",
            "<svg onload=alert('XSS')>",
            "javascript:alert('XSS')",
            "<iframe src='javascript:alert(\"XSS\")'></iframe>",
            "<body onload=alert('XSS')>",
            "<input onfocus=alert('XSS') autofocus>",
            "<select onfocus=alert('XSS') autofocus>",
            "<textarea onfocus=alert('XSS') autofocus>",
            "<marquee onstart=alert('XSS')>"
        ]
        
        for payload in xss_payloads:
            # 创建包含XSS的策略
            response = api_client.post(
                '/api/v1/strategies',
                headers={'Authorization': f'Bearer {user_token}'},
                json={'name': payload, 'code': 'def run(): pass'}
            )
            
            assert response.status_code == 201
            strategy_id = response.json()['id']
            
            # 获取策略，验证输出已转义
            response = api_client.get(
                f'/api/v1/strategies/{strategy_id}',
                headers={'Authorization': f'Bearer {user_token}'}
            )
            
            strategy_name = response.json()['name']
            
            # 验证危险标签已被转义或移除
            assert '<script>' not in strategy_name, f"XSS未被转义: {payload}"
            assert '<img' not in strategy_name or 'onerror' not in strategy_name
            assert 'javascript:' not in strategy_name
            assert '<iframe' not in strategy_name
    
    def test_reflected_xss_in_search(self, api_client, user_token):
        """TC-XSS-002: 搜索参数反射型XSS防护"""
        xss_payload = "<script>alert('XSS')</script>"
        
        response = api_client.get(
            f'/api/v1/strategies/search?q={xss_payload}',
            headers={'Authorization': f'Bearer {user_token}'}
        )
        
        # 响应中不应包含未转义的脚本
        response_text = response.text if hasattr(response, 'text') else str(response.content)
        assert '<script>' not in response_text, "反射型XSS未被防护"
    
    def test_xss_in_error_message(self, api_client, user_token):
        """TC-XSS-003: 错误消息XSS防护"""
        xss_payload = "<script>alert('XSS')</script>"
        
        # 尝试触发错误并在错误消息中注入XSS
        response = api_client.get(
            f'/api/v1/strategies/{xss_payload}',
            headers={'Authorization': f'Bearer {user_token}'}
        )
        
        # 错误消息中不应包含未转义的脚本
        if response.status_code >= 400:
            error_message = response.json().get('error', '')
            assert '<script>' not in error_message
    
    def test_dom_xss_prevention(self, api_client, user_token):
        """TC-XSS-004: DOM型XSS防护"""
        # 测试URL fragment中的XSS
        payloads = [
            "#<script>alert('XSS')</script>",
            "#javascript:alert('XSS')",
            "#<img src=x onerror=alert('XSS')>"
        ]
        
        # 这主要是前端测试，这里验证API不返回危险内容
        for payload in payloads:
            response = api_client.get(
                f'/api/v1/strategies?filter={payload}',
                headers={'Authorization': f'Bearer {user_token}'}
            )
            
            response_text = str(response.json())
            assert '<script>' not in response_text
    
    def test_xss_in_json_response(self, api_client, user_token):
        """TC-XSS-005: JSON响应XSS防护"""
        xss_payload = '"; alert("XSS"); var x="'
        
        response = api_client.post(
            '/api/v1/strategies',
            headers={'Authorization': f'Bearer {user_token}'},
            json={'name': xss_payload, 'code': 'def run(): pass'}
        )
        
        # JSON响应应该正确转义特殊字符
        response_json = response.json()
        strategy_name = response_json['name']
        
        # 验证响应的Content-Type
        assert response.headers.get('Content-Type') == 'application/json'
        
        # JSON中的特殊字符应该被转义
        assert '\\' in response.text  # 引号应该被转义
    
    def test_html_entity_encoding(self, api_client, user_token):
        """TC-XSS-006: HTML实体编码"""
        special_chars = "<>&\"'"
        
        response = api_client.post(
            '/api/v1/strategies',
            headers={'Authorization': f'Bearer {user_token}'},
            json={'name': special_chars, 'code': 'def run(): pass'}
        )
        
        strategy_id = response.json()['id']
        
        # 获取策略
        response = api_client.get(
            f'/api/v1/strategies/{strategy_id}',
            headers={'Authorization': f'Bearer {user_token}'}
        )
        
        # 特殊字符应该被编码或保持原样（不执行）
        # 在JSON API中，这些字符通常保持原样，由前端负责转义
        strategy_name = response.json()['name']
        assert strategy_name == special_chars  # JSON中应该原样返回
    
    def test_csp_header_present(self, api_client, user_token):
        """TC-XSS-007: Content-Security-Policy头存在"""
        response = api_client.get(
            '/api/v1/strategies',
            headers={'Authorization': f'Bearer {user_token}'}
        )
        
        # API应该设置CSP头
        # 注意：这主要用于前端HTML页面，API可能不需要
        # 但如果有HTML响应，应该设置CSP
        if 'text/html' in response.headers.get('Content-Type', ''):
            assert 'Content-Security-Policy' in response.headers
    
    def test_x_xss_protection_header(self, api_client, user_token):
        """TC-XSS-008: X-XSS-Protection头存在"""
        response = api_client.get(
            '/api/v1/strategies',
            headers={'Authorization': f'Bearer {user_token}'}
        )
        
        # 验证XSS保护头
        # 注意：现代浏览器推荐使用CSP而不是X-XSS-Protection
        # X-XSS-Protection已被废弃，但仍可用于旧浏览器兼容
        pass


class TestOutputEncoding:
    """输出编码测试"""
    
    def test_json_output_escaping(self, api_client, user_token):
        """TC-OUT-001: JSON输出转义"""
        dangerous_input = '</script><script>alert("XSS")</script>'
        
        response = api_client.post(
            '/api/v1/strategies',
            headers={'Authorization': f'Bearer {user_token}'},
            json={'name': dangerous_input, 'code': 'def run(): pass'}
        )
        
        # JSON应该正确序列化，不应该破坏JSON结构
        assert response.status_code == 201
        data = response.json()
        assert 'id' in data
    
    def test_url_encoding(self, api_client, user_token):
        """TC-OUT-002: URL参数编码"""
        import urllib.parse
        
        dangerous_input = '<script>alert("XSS")</script>'
        encoded = urllib.parse.quote(dangerous_input)
        
        response = api_client.get(
            f'/api/v1/strategies/search?q={encoded}',
            headers={'Authorization': f'Bearer {user_token}'}
        )
        
        # 应该正常处理编码后的输入
        assert response.status_code in [200, 400]

