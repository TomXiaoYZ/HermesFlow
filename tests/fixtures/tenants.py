"""
租户测试数据Fixtures
"""
from sqlalchemy import text


def create_tenant(session, tenant_id: str, name: str, plan: str = 'basic'):
    """创建测试租户"""
    session.execute(text("""
        INSERT INTO tenants (id, name, plan, created_at, updated_at)
        VALUES (:id, :name, :plan, NOW(), NOW())
        ON CONFLICT (id) DO NOTHING
    """), {'id': tenant_id, 'name': name, 'plan': plan})
    session.commit()
    
    # 返回租户对象
    class Tenant:
        def __init__(self, id, name, plan):
            self.id = id
            self.name = name
            self.plan = plan
    
    return Tenant(tenant_id, name, plan)

