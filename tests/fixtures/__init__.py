"""
测试Fixtures模块
"""

from .tenants import create_tenant
from .users import create_user

__all__ = ['create_tenant', 'create_user']

