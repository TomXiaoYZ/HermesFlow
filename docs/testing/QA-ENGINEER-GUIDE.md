# QA 工程师完整指南

> **HermesFlow 质量保障指南** | **目标**: 确保产品质量

---

## 🎯 QA 职责

作为 HermesFlow 的 QA 工程师，您负责：

1. ✅ 编写和执行测试用例
2. ✅ 自动化测试（单元、集成、E2E）
3. ✅ 性能测试（k6）
4. ✅ 安全测试（SQL 注入、XSS、RBAC、多租户隔离）
5. ✅ 验收测试
6. ✅ 测试数据管理
7. ✅ 质量门禁监控

---

## 📚 必读文档

- [测试策略](./test-strategy.md) - 整体测试方法
- [早期测试策略](./early-test-strategy.md) - Alpha 阶段测试计划
- [高风险访问测试](./high-risk-access-testing.md) - 安全关键路径
- [测试数据管理](./test-data-management.md) - 测试数据准备
- [CI/CD 测试集成](./ci-cd-integration.md) - 自动化测试
- [验收测试清单](./ACCEPTANCE-CHECKLIST.md) - 验收标准

---

## 🚀 快速开始

### 1. 环境搭建

```bash
# 克隆代码
git clone <repo-url>/HermesFlow.git
cd HermesFlow

# 启动测试环境
docker-compose -f docker-compose.test.yml up -d

# 安装 Python 测试依赖
pip install -r requirements-test.txt

# 运行所有测试
pytest

# 安装 k6（性能测试）
brew install k6  # macOS
# 或参考 https://k6.io/docs/getting-started/installation/
```

### 2. 运行测试

```bash
# 单元测试
pytest tests/unit/

# 集成测试
pytest tests/integration/

# 安全测试
pytest tests/security/

# 性能测试
k6 run tests/performance/load_test.js
```

---

## 🧪 测试金字塔

```
       E2E 测试 (10%)
      /            \
     集成测试 (30%)
    /                \
   单元测试 (60%)
```

**原则**: 多写单元测试，适量集成测试，少量 E2E 测试

**覆盖率目标**:
- Rust: ≥ 85%
- Java: ≥ 80%
- Python: ≥ 75%

---

## 📋 测试类型

### 1. 功能测试

**目标**: 验证功能符合 PRD 要求

**方法**:
- 手动测试（探索性测试）
- 自动化测试（pytest, JUnit, Rust `cargo test`）

**示例**:
```python
# tests/test_strategy_service.py

def test_create_strategy(api_client, auth_headers):
    """测试创建策略"""
    payload = {
        "name": "My Strategy",
        "code": "def on_bar(bar): pass",
    }
    
    response = api_client.post(
        "/api/v1/strategies",
        json=payload,
        headers=auth_headers
    )
    
    assert response.status_code == 201
    assert response.json()["name"] == "My Strategy"
```

---

### 2. 安全测试

**目标**: 确保系统安全

**关键测试点**:
- ✅ SQL 注入防护
- ✅ XSS 防护
- ✅ JWT Token 验证
- ✅ RBAC 权限
- ✅ 多租户隔离

**示例**: [SQL 注入测试](../../tests/security/test_sql_injection.py)

```python
# tests/security/test_sql_injection.py

def test_sql_injection_in_query_params(api_client):
    """测试查询参数中的 SQL 注入"""
    malicious_params = [
        "1' OR '1'='1",
        "1; DROP TABLE users--",
        "1' UNION SELECT * FROM users--",
    ]
    
    for param in malicious_params:
        response = api_client.get(f"/api/v1/strategies?id={param}")
        
        # 应该返回 400 或空结果，而不是执行 SQL
        assert response.status_code in [400, 404]
```

**参考**: [高风险访问测试](./high-risk-access-testing.md)

---

### 3. 性能测试

**目标**: 验证性能基线

**工具**: k6

**示例**:
```javascript
// tests/performance/load_test.js

import http from 'k6/http';
import { check, sleep } from 'k6';

export let options = {
    stages: [
        { duration: '30s', target: 50 },   // Ramp-up
        { duration: '1m', target: 100 },   // Stay at 100 users
        { duration: '30s', target: 0 },    // Ramp-down
    ],
    thresholds: {
        http_req_duration: ['p(95)<500'],  // P95 < 500ms
        http_req_failed: ['rate<0.01'],    // 错误率 < 1%
    },
};

export default function () {
    let response = http.get('http://localhost:8081/api/v1/market-data?symbol=BTC/USDT');
    
    check(response, {
        'status is 200': (r) => r.status === 200,
        'response time < 500ms': (r) => r.timings.duration < 500,
    });
    
    sleep(1);
}
```

**运行**:
```bash
k6 run tests/performance/load_test.js
```

---

### 4. 多租户隔离测试

**目标**: 确保租户数据隔离

**测试场景**:
- 租户 A 无法访问租户 B 的数据
- PostgreSQL RLS 策略生效
- Redis Key 使用租户前缀

**示例**:
```python
# tests/security/test_tenant_isolation.py

def test_tenant_cannot_access_other_tenant_data(api_client):
    """测试租户隔离"""
    # 租户 A 创建策略
    tenant_a_token = get_jwt_token(tenant_id="tenant-a")
    response_a = api_client.post(
        "/api/v1/strategies",
        json={"name": "Strategy A"},
        headers={"Authorization": f"Bearer {tenant_a_token}"}
    )
    strategy_id = response_a.json()["id"]
    
    # 租户 B 尝试访问租户 A 的策略
    tenant_b_token = get_jwt_token(tenant_id="tenant-b")
    response_b = api_client.get(
        f"/api/v1/strategies/{strategy_id}",
        headers={"Authorization": f"Bearer {tenant_b_token}"}
    )
    
    # 应该返回 404 或 403
    assert response_b.status_code in [403, 404]
```

**参考**: [高风险访问测试 - RLS 测试](./high-risk-access-testing.md#postgresql-rls-测试)

---

## 📊 测试报告

### 1. 生成测试报告

```bash
# pytest HTML 报告
pytest --html=report.html --self-contained-html

# pytest 覆盖率报告
pytest --cov=src --cov-report=html

# k6 性能报告
k6 run --out json=test_results.json tests/performance/load_test.js
```

### 2. CI/CD 集成

测试自动在 GitHub Actions 中运行（`.github/workflows/test.yml`）

**查看测试结果**: GitHub Actions → Workflow run → Test job

---

## 🛠️ 常用工具

### pytest fixtures

```python
# tests/conftest.py

import pytest
from app import create_app
from app.database import db

@pytest.fixture
def app():
    """创建测试应用"""
    app = create_app('testing')
    with app.app_context():
        db.create_all()
        yield app
        db.session.remove()
        db.drop_all()

@pytest.fixture
def client(app):
    """创建测试客户端"""
    return app.test_client()

@pytest.fixture
def auth_headers(client):
    """获取认证 Token"""
    response = client.post('/api/v1/auth/login', json={
        'username': 'testuser',
        'password': 'testpass'
    })
    token = response.json()['token']
    return {'Authorization': f'Bearer {token}'}
```

---

## ✅ 验收测试

使用 [验收测试清单](./ACCEPTANCE-CHECKLIST.md) 进行验收：

**关键检查项**:
- [ ] 所有功能验收标准满足
- [ ] 单元测试覆盖率达标
- [ ] 安全测试通过
- [ ] 性能测试通过
- [ ] 文档已更新

---

## 📚 学习资源

- [pytest 文档](https://docs.pytest.org/)
- [k6 文档](https://k6.io/docs/)
- [OWASP Top 10](https://owasp.org/www-project-top-ten/)

---

## 📞 获取帮助

- **QA Team**: Slack `#qa-team`
- **技术问题**: [FAQ](../FAQ.md)

---

**最后更新**: 2025-01-13  
**维护者**: @qa.mdc  
**版本**: v1.0

