"""
用户测试数据Fixtures
"""
from sqlalchemy import text
import hashlib


def hash_password(password: str) -> str:
    """简单的密码哈希（测试用）"""
    return hashlib.sha256(password.encode()).hexdigest()


def create_user(session, user_id: str, email: str, tenant_id: str, role: str = 'user', password: str = 'testpass123'):
    """创建测试用户"""
    password_hash = hash_password(password)
    
    session.execute(text("""
        INSERT INTO users (id, email, password_hash, tenant_id, role, created_at, updated_at)
        VALUES (:id, :email, :password_hash, :tenant_id, :role, NOW(), NOW())
        ON CONFLICT (id) DO NOTHING
    """), {
        'id': user_id,
        'email': email,
        'password_hash': password_hash,
        'tenant_id': tenant_id,
        'role': role
    })
    session.commit()
    
    # 返回用户对象
    class User:
        def __init__(self, id, email, tenant_id, role):
            self.id = id
            self.email = email
            self.tenant_id = tenant_id
            self.role = role
    
    return User(user_id, email, tenant_id, role)

