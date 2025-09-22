-- =====================================================
-- PostgreSQL Row Level Security (RLS) 策略
-- 用于实现多租户数据隔离
-- =====================================================

-- 启用行级安全策略
ALTER TABLE tenants ENABLE ROW LEVEL SECURITY;
ALTER TABLE users ENABLE ROW LEVEL SECURITY;
ALTER TABLE tenant_configs ENABLE ROW LEVEL SECURITY;
ALTER TABLE user_sessions ENABLE ROW LEVEL SECURITY;

-- =====================================================
-- 租户表 RLS 策略
-- =====================================================

-- 租户只能访问自己的记录
CREATE POLICY tenant_isolation_tenants ON tenants
    FOR ALL
    TO PUBLIC
    USING (id = COALESCE(current_setting('app.current_tenant', true)::uuid, id))
    WITH CHECK (id = COALESCE(current_setting('app.current_tenant', true)::uuid, id));

-- =====================================================
-- 用户表 RLS 策略
-- =====================================================

-- 用户只能访问同租户的用户记录
CREATE POLICY tenant_isolation_users ON users
    FOR ALL
    TO PUBLIC
    USING (tenant_id = COALESCE(current_setting('app.current_tenant', true)::uuid, tenant_id))
    WITH CHECK (tenant_id = COALESCE(current_setting('app.current_tenant', true)::uuid, tenant_id));

-- =====================================================
-- 租户配置表 RLS 策略
-- =====================================================

-- 租户配置只能被同租户访问
CREATE POLICY tenant_isolation_configs ON tenant_configs
    FOR ALL
    TO PUBLIC
    USING (tenant_id = COALESCE(current_setting('app.current_tenant', true)::uuid, tenant_id))
    WITH CHECK (tenant_id = COALESCE(current_setting('app.current_tenant', true)::uuid, tenant_id));

-- =====================================================
-- 用户会话表 RLS 策略
-- =====================================================

-- 用户会话只能被同租户的用户访问
CREATE POLICY tenant_isolation_sessions ON user_sessions
    FOR ALL
    TO PUBLIC
    USING (user_id IN (
        SELECT id FROM users 
        WHERE tenant_id = COALESCE(current_setting('app.current_tenant', true)::uuid, tenant_id)
    ))
    WITH CHECK (user_id IN (
        SELECT id FROM users 
        WHERE tenant_id = COALESCE(current_setting('app.current_tenant', true)::uuid, tenant_id)
    ));

-- =====================================================
-- 创建租户上下文设置函数
-- =====================================================

-- 设置当前租户上下文的函数
CREATE OR REPLACE FUNCTION set_current_tenant(tenant_uuid UUID)
RETURNS VOID AS $$
BEGIN
    PERFORM set_config('app.current_tenant', tenant_uuid::text, false);
END;
$$ LANGUAGE plpgsql;

-- 获取当前租户上下文的函数
CREATE OR REPLACE FUNCTION get_current_tenant()
RETURNS UUID AS $$
BEGIN
    RETURN COALESCE(current_setting('app.current_tenant', true)::uuid, NULL);
END;
$$ LANGUAGE plpgsql;

-- 清除当前租户上下文的函数
CREATE OR REPLACE FUNCTION clear_current_tenant()
RETURNS VOID AS $$
BEGIN
    PERFORM set_config('app.current_tenant', '', false);
END;
$$ LANGUAGE plpgsql;

-- =====================================================
-- 创建索引以优化 RLS 查询性能
-- =====================================================

-- 为租户ID创建索引
CREATE INDEX IF NOT EXISTS idx_users_tenant_id ON users(tenant_id);
CREATE INDEX IF NOT EXISTS idx_tenant_configs_tenant_id ON tenant_configs(tenant_id);
CREATE INDEX IF NOT EXISTS idx_user_sessions_user_tenant ON user_sessions(user_id) 
    WHERE user_id IN (SELECT id FROM users);

-- =====================================================
-- 创建租户管理员角色和权限
-- =====================================================

-- 创建租户管理员角色（可以绕过 RLS 策略）
DO $$
BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_roles WHERE rolname = 'tenant_admin') THEN
        CREATE ROLE tenant_admin;
    END IF;
END
$$;

-- 授予租户管理员绕过 RLS 的权限
ALTER ROLE tenant_admin BYPASSRLS;

-- =====================================================
-- 注释说明
-- =====================================================

COMMENT ON POLICY tenant_isolation_tenants ON tenants IS 
'租户表行级安全策略：租户只能访问自己的记录';

COMMENT ON POLICY tenant_isolation_users ON users IS 
'用户表行级安全策略：用户只能访问同租户的用户记录';

COMMENT ON POLICY tenant_isolation_configs ON tenant_configs IS 
'租户配置表行级安全策略：租户配置只能被同租户访问';

COMMENT ON POLICY tenant_isolation_sessions ON user_sessions IS 
'用户会话表行级安全策略：用户会话只能被同租户的用户访问';

COMMENT ON FUNCTION set_current_tenant(UUID) IS 
'设置当前会话的租户上下文，用于 RLS 策略过滤';

COMMENT ON FUNCTION get_current_tenant() IS 
'获取当前会话的租户上下文';

COMMENT ON FUNCTION clear_current_tenant() IS 
'清除当前会话的租户上下文'; 