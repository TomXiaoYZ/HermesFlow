-- HermesFlow User Management Service - Database Schema
-- Version: 1.0.0
-- Description: 创建用户管理服务的核心数据表

-- 创建租户表
CREATE TABLE IF NOT EXISTS tenants (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL,
    display_name VARCHAR(255) NOT NULL,
    description TEXT,
    status VARCHAR(50) NOT NULL DEFAULT 'ACTIVE',
    plan VARCHAR(50) NOT NULL DEFAULT 'BASIC',
    max_users INTEGER NOT NULL DEFAULT 100,
    contact_email VARCHAR(255),
    contact_phone VARCHAR(50),
    address TEXT,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    created_by VARCHAR(255),
    updated_by VARCHAR(255),
    CONSTRAINT uk_tenants_name UNIQUE (name)
);

-- 创建租户配置表
CREATE TABLE IF NOT EXISTS tenant_configs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL,
    config_key VARCHAR(255) NOT NULL,
    config_value TEXT,
    description TEXT,
    is_encrypted BOOLEAN DEFAULT FALSE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    created_by VARCHAR(255),
    updated_by VARCHAR(255),
    CONSTRAINT fk_tenant_configs_tenant FOREIGN KEY (tenant_id) REFERENCES tenants(id) ON DELETE CASCADE,
    CONSTRAINT uk_tenant_configs_key UNIQUE (tenant_id, config_key)
);

-- 创建用户表
CREATE TABLE IF NOT EXISTS users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL,
    username VARCHAR(255) NOT NULL,
    email VARCHAR(255) NOT NULL,
    password_hash VARCHAR(255) NOT NULL,
    first_name VARCHAR(255),
    last_name VARCHAR(255),
    phone VARCHAR(50),
    status VARCHAR(50) NOT NULL DEFAULT 'ACTIVE',
    email_verified BOOLEAN DEFAULT FALSE,
    phone_verified BOOLEAN DEFAULT FALSE,
    failed_login_attempts INTEGER DEFAULT 0,
    locked_until TIMESTAMP WITH TIME ZONE,
    last_login_at TIMESTAMP WITH TIME ZONE,
    password_changed_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    created_by VARCHAR(255),
    updated_by VARCHAR(255),
    CONSTRAINT fk_users_tenant FOREIGN KEY (tenant_id) REFERENCES tenants(id) ON DELETE CASCADE,
    CONSTRAINT uk_users_username_tenant UNIQUE (tenant_id, username),
    CONSTRAINT uk_users_email_tenant UNIQUE (tenant_id, email)
);

-- 创建用户会话表
CREATE TABLE IF NOT EXISTS user_sessions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL,
    tenant_id UUID NOT NULL,
    session_token VARCHAR(500) NOT NULL,
    refresh_token VARCHAR(500),
    device_info TEXT,
    ip_address VARCHAR(45),
    user_agent TEXT,
    status VARCHAR(50) NOT NULL DEFAULT 'ACTIVE',
    expires_at TIMESTAMP WITH TIME ZONE NOT NULL,
    last_accessed_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    CONSTRAINT fk_user_sessions_user FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    CONSTRAINT fk_user_sessions_tenant FOREIGN KEY (tenant_id) REFERENCES tenants(id) ON DELETE CASCADE,
    CONSTRAINT uk_user_sessions_token UNIQUE (session_token)
);

-- 创建索引以提高查询性能
CREATE INDEX IF NOT EXISTS idx_tenants_status ON tenants(status);
CREATE INDEX IF NOT EXISTS idx_tenants_plan ON tenants(plan);

CREATE INDEX IF NOT EXISTS idx_tenant_configs_tenant_id ON tenant_configs(tenant_id);
CREATE INDEX IF NOT EXISTS idx_tenant_configs_key ON tenant_configs(config_key);

CREATE INDEX IF NOT EXISTS idx_users_tenant_id ON users(tenant_id);
CREATE INDEX IF NOT EXISTS idx_users_status ON users(status);
CREATE INDEX IF NOT EXISTS idx_users_email ON users(email);
CREATE INDEX IF NOT EXISTS idx_users_username ON users(username);
CREATE INDEX IF NOT EXISTS idx_users_last_login ON users(last_login_at);

CREATE INDEX IF NOT EXISTS idx_user_sessions_user_id ON user_sessions(user_id);
CREATE INDEX IF NOT EXISTS idx_user_sessions_tenant_id ON user_sessions(tenant_id);
CREATE INDEX IF NOT EXISTS idx_user_sessions_status ON user_sessions(status);
CREATE INDEX IF NOT EXISTS idx_user_sessions_expires_at ON user_sessions(expires_at);
CREATE INDEX IF NOT EXISTS idx_user_sessions_token ON user_sessions(session_token);

-- 创建更新时间触发器函数
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = CURRENT_TIMESTAMP;
    RETURN NEW;
END;
$$ language 'plpgsql';

-- 为所有表创建更新时间触发器
CREATE TRIGGER update_tenants_updated_at BEFORE UPDATE ON tenants
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_tenant_configs_updated_at BEFORE UPDATE ON tenant_configs
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_users_updated_at BEFORE UPDATE ON users
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_user_sessions_updated_at BEFORE UPDATE ON user_sessions
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

-- 插入默认租户数据
INSERT INTO tenants (id, name, display_name, description, status, plan, max_users, contact_email)
VALUES 
    ('00000000-0000-0000-0000-000000000001', 'system', 'System Tenant', 'System default tenant', 'ACTIVE', 'ENTERPRISE', 1000, 'admin@hermesflow.com'),
    ('00000000-0000-0000-0000-000000000002', 'demo', 'Demo Tenant', 'Demo tenant for testing', 'ACTIVE', 'BASIC', 100, 'demo@hermesflow.com')
ON CONFLICT (name) DO NOTHING;

-- 插入默认管理员用户 (密码: admin123)
INSERT INTO users (id, tenant_id, username, email, password_hash, first_name, last_name, status, email_verified)
VALUES 
    ('00000000-0000-0000-0000-000000000001', '00000000-0000-0000-0000-000000000001', 'admin', 'admin@hermesflow.com', '$2a$10$rqQjQjQjQjQjQjQjQjQjQeJ8K8K8K8K8K8K8K8K8K8K8K8K8K8K8K8', 'System', 'Administrator', 'ACTIVE', true),
    ('00000000-0000-0000-0000-000000000002', '00000000-0000-0000-0000-000000000002', 'demo', 'demo@hermesflow.com', '$2a$10$rqQjQjQjQjQjQjQjQjQjQeJ8K8K8K8K8K8K8K8K8K8K8K8K8K8K8K8', 'Demo', 'User', 'ACTIVE', true)
ON CONFLICT (tenant_id, username) DO NOTHING;

-- 插入默认租户配置
INSERT INTO tenant_configs (tenant_id, config_key, config_value, description)
VALUES 
    ('00000000-0000-0000-0000-000000000001', 'max_api_calls_per_hour', '10000', 'Maximum API calls per hour'),
    ('00000000-0000-0000-0000-000000000001', 'data_retention_days', '365', 'Data retention period in days'),
    ('00000000-0000-0000-0000-000000000002', 'max_api_calls_per_hour', '1000', 'Maximum API calls per hour'),
    ('00000000-0000-0000-0000-000000000002', 'data_retention_days', '90', 'Data retention period in days')
ON CONFLICT (tenant_id, config_key) DO NOTHING; 