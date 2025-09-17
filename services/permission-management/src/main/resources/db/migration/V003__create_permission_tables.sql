-- HermesFlow 权限管理系统数据库表结构
-- 版本: V003
-- 创建时间: 2024-12-30

-- 1. 权限表 (permissions)
-- 存储系统中所有可用的权限
CREATE TABLE permissions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(100) NOT NULL,                    -- 权限名称
    code VARCHAR(100) NOT NULL UNIQUE,             -- 权限代码 (如: user:create)
    resource VARCHAR(100) NOT NULL,                -- 资源类型 (如: user, role, strategy)
    action VARCHAR(50) NOT NULL,                   -- 操作类型 (如: create, read, update, delete)
    description TEXT,                              -- 权限描述
    permission_type VARCHAR(20) NOT NULL DEFAULT 'FUNCTIONAL', -- 权限类型: FUNCTIONAL, DATA, SYSTEM
    is_system BOOLEAN DEFAULT FALSE,               -- 是否为系统权限
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- 权限表索引
CREATE INDEX idx_permissions_resource ON permissions(resource);
CREATE INDEX idx_permissions_action ON permissions(action);
CREATE INDEX idx_permissions_type ON permissions(permission_type);
CREATE INDEX idx_permissions_system ON permissions(is_system);

-- 2. 角色表 (roles)
-- 存储系统中的角色信息，支持多租户和角色层级
CREATE TABLE roles (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    name VARCHAR(100) NOT NULL,                    -- 角色名称
    code VARCHAR(50) NOT NULL,                     -- 角色代码
    description TEXT,                              -- 角色描述
    role_type VARCHAR(20) NOT NULL DEFAULT 'CUSTOM', -- 角色类型: SYSTEM, PREDEFINED, CUSTOM
    parent_role_id UUID REFERENCES roles(id),     -- 父角色ID (支持角色继承)
    is_active BOOLEAN DEFAULT TRUE,               -- 是否激活
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    
    -- 确保同一租户内角色代码唯一
    UNIQUE(tenant_id, code)
);

-- 角色表索引
CREATE INDEX idx_roles_tenant ON roles(tenant_id);
CREATE INDEX idx_roles_code ON roles(code);
CREATE INDEX idx_roles_type ON roles(role_type);
CREATE INDEX idx_roles_parent ON roles(parent_role_id);
CREATE INDEX idx_roles_active ON roles(is_active);

-- 3. 用户角色关联表 (user_roles)
-- 存储用户与角色的多对多关系
CREATE TABLE user_roles (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    role_id UUID NOT NULL REFERENCES roles(id) ON DELETE CASCADE,
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    assigned_by UUID REFERENCES users(id),        -- 分配者
    assigned_at TIMESTAMPTZ DEFAULT NOW(),        -- 分配时间
    expires_at TIMESTAMPTZ,                       -- 过期时间 (可选)
    is_active BOOLEAN DEFAULT TRUE,               -- 是否激活
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    
    -- 确保同一用户在同一租户内不能重复分配同一角色
    UNIQUE(user_id, role_id, tenant_id)
);

-- 用户角色关联表索引
CREATE INDEX idx_user_roles_user ON user_roles(user_id);
CREATE INDEX idx_user_roles_role ON user_roles(role_id);
CREATE INDEX idx_user_roles_tenant ON user_roles(tenant_id);
CREATE INDEX idx_user_roles_active ON user_roles(is_active);
CREATE INDEX idx_user_roles_expires ON user_roles(expires_at);

-- 4. 角色权限关联表 (role_permissions)
-- 存储角色与权限的多对多关系
CREATE TABLE role_permissions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    role_id UUID NOT NULL REFERENCES roles(id) ON DELETE CASCADE,
    permission_id UUID NOT NULL REFERENCES permissions(id) ON DELETE CASCADE,
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    granted_by UUID REFERENCES users(id),         -- 授权者
    granted_at TIMESTAMPTZ DEFAULT NOW(),         -- 授权时间
    is_active BOOLEAN DEFAULT TRUE,               -- 是否激活
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    
    -- 确保同一角色在同一租户内不能重复分配同一权限
    UNIQUE(role_id, permission_id, tenant_id)
);

-- 角色权限关联表索引
CREATE INDEX idx_role_permissions_role ON role_permissions(role_id);
CREATE INDEX idx_role_permissions_permission ON role_permissions(permission_id);
CREATE INDEX idx_role_permissions_tenant ON role_permissions(tenant_id);
CREATE INDEX idx_role_permissions_active ON role_permissions(is_active);

-- 5. 权限审计日志表 (permission_audit_logs)
-- 记录权限相关的操作审计
CREATE TABLE permission_audit_logs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    user_id UUID REFERENCES users(id),            -- 操作用户
    target_user_id UUID REFERENCES users(id),     -- 目标用户 (如果是用户权限操作)
    role_id UUID REFERENCES roles(id),            -- 相关角色
    permission_id UUID REFERENCES permissions(id), -- 相关权限
    operation VARCHAR(50) NOT NULL,               -- 操作类型: GRANT, REVOKE, CHECK
    resource_type VARCHAR(50) NOT NULL,           -- 资源类型
    resource_id VARCHAR(100),                     -- 资源ID
    result VARCHAR(20) NOT NULL,                  -- 操作结果: SUCCESS, FAILED, DENIED
    ip_address INET,                              -- 操作IP
    user_agent TEXT,                              -- 用户代理
    details JSONB,                                -- 详细信息
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- 权限审计日志表索引
CREATE INDEX idx_audit_logs_tenant ON permission_audit_logs(tenant_id);
CREATE INDEX idx_audit_logs_user ON permission_audit_logs(user_id);
CREATE INDEX idx_audit_logs_operation ON permission_audit_logs(operation);
CREATE INDEX idx_audit_logs_result ON permission_audit_logs(result);
CREATE INDEX idx_audit_logs_created ON permission_audit_logs(created_at);

-- 6. 创建权限管理相关的RLS策略
-- 确保多租户数据隔离

-- 角色表RLS策略
ALTER TABLE roles ENABLE ROW LEVEL SECURITY;

CREATE POLICY roles_tenant_isolation ON roles
    FOR ALL
    TO PUBLIC
    USING (tenant_id = current_setting('app.current_tenant_id', true)::UUID);

-- 用户角色关联表RLS策略
ALTER TABLE user_roles ENABLE ROW LEVEL SECURITY;

CREATE POLICY user_roles_tenant_isolation ON user_roles
    FOR ALL
    TO PUBLIC
    USING (tenant_id = current_setting('app.current_tenant_id', true)::UUID);

-- 角色权限关联表RLS策略
ALTER TABLE role_permissions ENABLE ROW LEVEL SECURITY;

CREATE POLICY role_permissions_tenant_isolation ON role_permissions
    FOR ALL
    TO PUBLIC
    USING (tenant_id = current_setting('app.current_tenant_id', true)::UUID);

-- 权限审计日志表RLS策略
ALTER TABLE permission_audit_logs ENABLE ROW LEVEL SECURITY;

CREATE POLICY audit_logs_tenant_isolation ON permission_audit_logs
    FOR ALL
    TO PUBLIC
    USING (tenant_id = current_setting('app.current_tenant_id', true)::UUID);

-- 7. 创建权限管理相关的函数

-- 检查用户是否具有指定权限
CREATE OR REPLACE FUNCTION check_user_permission(
    p_user_id UUID,
    p_permission_code VARCHAR,
    p_tenant_id UUID DEFAULT NULL
) RETURNS BOOLEAN AS $$
DECLARE
    v_tenant_id UUID;
    v_has_permission BOOLEAN := FALSE;
BEGIN
    -- 如果没有指定租户ID，从当前设置中获取
    v_tenant_id := COALESCE(p_tenant_id, current_setting('app.current_tenant_id', true)::UUID);
    
    -- 检查用户是否通过角色拥有该权限
    SELECT EXISTS(
        SELECT 1
        FROM user_roles ur
        JOIN role_permissions rp ON ur.role_id = rp.role_id
        JOIN permissions p ON rp.permission_id = p.id
        WHERE ur.user_id = p_user_id
          AND ur.tenant_id = v_tenant_id
          AND p.code = p_permission_code
          AND ur.is_active = TRUE
          AND rp.is_active = TRUE
          AND (ur.expires_at IS NULL OR ur.expires_at > NOW())
    ) INTO v_has_permission;
    
    RETURN v_has_permission;
END;
$$ LANGUAGE plpgsql SECURITY DEFINER;

-- 获取用户的所有权限
CREATE OR REPLACE FUNCTION get_user_permissions(
    p_user_id UUID,
    p_tenant_id UUID DEFAULT NULL
) RETURNS TABLE(permission_code VARCHAR, permission_name VARCHAR, resource VARCHAR, action VARCHAR) AS $$
DECLARE
    v_tenant_id UUID;
BEGIN
    -- 如果没有指定租户ID，从当前设置中获取
    v_tenant_id := COALESCE(p_tenant_id, current_setting('app.current_tenant_id', true)::UUID);
    
    RETURN QUERY
    SELECT DISTINCT p.code, p.name, p.resource, p.action
    FROM user_roles ur
    JOIN role_permissions rp ON ur.role_id = rp.role_id
    JOIN permissions p ON rp.permission_id = p.id
    WHERE ur.user_id = p_user_id
      AND ur.tenant_id = v_tenant_id
      AND ur.is_active = TRUE
      AND rp.is_active = TRUE
      AND (ur.expires_at IS NULL OR ur.expires_at > NOW())
    ORDER BY p.resource, p.action;
END;
$$ LANGUAGE plpgsql SECURITY DEFINER;

-- 8. 插入系统预定义权限
INSERT INTO permissions (name, code, resource, action, description, permission_type, is_system) VALUES
-- 用户管理权限
('创建用户', 'user:create', 'user', 'create', '创建新用户账户', 'FUNCTIONAL', true),
('查看用户', 'user:read', 'user', 'read', '查看用户信息', 'FUNCTIONAL', true),
('更新用户', 'user:update', 'user', 'update', '更新用户信息', 'FUNCTIONAL', true),
('删除用户', 'user:delete', 'user', 'delete', '删除用户账户', 'FUNCTIONAL', true),
('用户列表', 'user:list', 'user', 'list', '查看用户列表', 'FUNCTIONAL', true),

-- 角色管理权限
('创建角色', 'role:create', 'role', 'create', '创建新角色', 'FUNCTIONAL', true),
('查看角色', 'role:read', 'role', 'read', '查看角色信息', 'FUNCTIONAL', true),
('更新角色', 'role:update', 'role', 'update', '更新角色信息', 'FUNCTIONAL', true),
('删除角色', 'role:delete', 'role', 'delete', '删除角色', 'FUNCTIONAL', true),
('分配角色', 'role:assign', 'role', 'assign', '为用户分配角色', 'FUNCTIONAL', true),

-- 权限管理权限
('查看权限', 'permission:read', 'permission', 'read', '查看权限信息', 'FUNCTIONAL', true),
('分配权限', 'permission:assign', 'permission', 'assign', '为角色分配权限', 'FUNCTIONAL', true),

-- 策略管理权限
('创建策略', 'strategy:create', 'strategy', 'create', '创建交易策略', 'FUNCTIONAL', true),
('查看策略', 'strategy:read', 'strategy', 'read', '查看策略信息', 'FUNCTIONAL', true),
('更新策略', 'strategy:update', 'strategy', 'update', '更新策略配置', 'FUNCTIONAL', true),
('删除策略', 'strategy:delete', 'strategy', 'delete', '删除策略', 'FUNCTIONAL', true),
('执行策略', 'strategy:execute', 'strategy', 'execute', '执行交易策略', 'FUNCTIONAL', true),
('策略回测', 'strategy:backtest', 'strategy', 'backtest', '进行策略回测', 'FUNCTIONAL', true),

-- 数据访问权限
('实时数据读取', 'data:realtime:read', 'data', 'realtime_read', '读取实时市场数据', 'DATA', true),
('历史数据读取', 'data:historical:read', 'data', 'historical_read', '读取历史市场数据', 'DATA', true),
('数据导出', 'data:export', 'data', 'export', '导出数据到文件', 'DATA', true),

-- 系统管理权限
('系统配置查看', 'system:config:read', 'system', 'config_read', '查看系统配置', 'SYSTEM', true),
('系统配置修改', 'system:config:write', 'system', 'config_write', '修改系统配置', 'SYSTEM', true),
('系统监控', 'system:monitor', 'system', 'monitor', '监控系统状态', 'SYSTEM', true),
('审计日志', 'system:audit', 'system', 'audit', '查看审计日志', 'SYSTEM', true);

-- 9. 创建性能优化的复合索引
CREATE INDEX idx_user_roles_user_tenant_active ON user_roles(user_id, tenant_id, is_active);
CREATE INDEX idx_role_permissions_role_tenant_active ON role_permissions(role_id, tenant_id, is_active);
CREATE INDEX idx_permissions_resource_action ON permissions(resource, action);

-- 10. 添加表注释
COMMENT ON TABLE permissions IS '系统权限表，存储所有可用的权限定义';
COMMENT ON TABLE roles IS '角色表，支持多租户和角色层级';
COMMENT ON TABLE user_roles IS '用户角色关联表，支持角色过期和激活状态';
COMMENT ON TABLE role_permissions IS '角色权限关联表，定义角色拥有的权限';
COMMENT ON TABLE permission_audit_logs IS '权限操作审计日志表，记录所有权限相关操作'; 