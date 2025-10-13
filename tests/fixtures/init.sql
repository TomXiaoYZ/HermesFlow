-- PostgreSQL 测试数据库初始化脚本
-- 创建表结构和RLS策略

-- ============================================================================
-- 租户表
-- ============================================================================
CREATE TABLE IF NOT EXISTS tenants (
    id VARCHAR(50) PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    plan VARCHAR(50) NOT NULL DEFAULT 'basic',
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW()
);

-- ============================================================================
-- 用户表
-- ============================================================================
CREATE TABLE IF NOT EXISTS users (
    id VARCHAR(50) PRIMARY KEY,
    email VARCHAR(255) UNIQUE NOT NULL,
    password_hash VARCHAR(255) NOT NULL,
    tenant_id VARCHAR(50) NOT NULL REFERENCES tenants(id),
    role VARCHAR(50) NOT NULL DEFAULT 'user',
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_users_tenant_id ON users(tenant_id);
CREATE INDEX IF NOT EXISTS idx_users_email ON users(email);

-- ============================================================================
-- 策略表
-- ============================================================================
CREATE TABLE IF NOT EXISTS strategies (
    id VARCHAR(50) PRIMARY KEY,
    tenant_id VARCHAR(50) NOT NULL REFERENCES tenants(id),
    user_id VARCHAR(50) NOT NULL REFERENCES users(id),
    name VARCHAR(255) NOT NULL,
    code TEXT NOT NULL,
    status VARCHAR(50) NOT NULL DEFAULT 'draft',
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_strategies_tenant_id ON strategies(tenant_id);
CREATE INDEX IF NOT EXISTS idx_strategies_user_id ON strategies(user_id);

-- ============================================================================
-- 订单表
-- ============================================================================
CREATE TABLE IF NOT EXISTS orders (
    id VARCHAR(50) PRIMARY KEY,
    tenant_id VARCHAR(50) NOT NULL REFERENCES tenants(id),
    user_id VARCHAR(50) NOT NULL REFERENCES users(id),
    strategy_id VARCHAR(50) REFERENCES strategies(id),
    symbol VARCHAR(50) NOT NULL,
    side VARCHAR(10) NOT NULL,
    quantity DECIMAL(20, 8) NOT NULL,
    price DECIMAL(20, 8),
    status VARCHAR(50) NOT NULL DEFAULT 'pending',
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_orders_tenant_id ON orders(tenant_id);
CREATE INDEX IF NOT EXISTS idx_orders_user_id ON orders(user_id);
CREATE INDEX IF NOT EXISTS idx_orders_strategy_id ON orders(strategy_id);

-- ============================================================================
-- 启用Row-Level Security (RLS)
-- ============================================================================

-- 为strategies表启用RLS
ALTER TABLE strategies ENABLE ROW LEVEL SECURITY;

-- RLS策略：只能访问自己租户的数据
CREATE POLICY tenant_isolation_strategies ON strategies
    FOR ALL
    USING (tenant_id = current_setting('app.current_tenant', true));

-- 为orders表启用RLS
ALTER TABLE orders ENABLE ROW LEVEL SECURITY;

CREATE POLICY tenant_isolation_orders ON orders
    FOR ALL
    USING (tenant_id = current_setting('app.current_tenant', true));

-- ============================================================================
-- 测试数据
-- ============================================================================

-- 创建测试租户
INSERT INTO tenants (id, name, plan) VALUES
    ('tenant-a', 'Test Tenant A', 'premium'),
    ('tenant-b', 'Test Tenant B', 'basic')
ON CONFLICT (id) DO NOTHING;

-- 创建测试用户
INSERT INTO users (id, email, password_hash, tenant_id, role) VALUES
    ('user-a-1', 'user-a@example.com', 'hash1', 'tenant-a', 'user'),
    ('admin-a-1', 'admin-a@example.com', 'hash2', 'tenant-a', 'admin'),
    ('trader-a-1', 'trader-a@example.com', 'hash3', 'tenant-a', 'trader'),
    ('analyst-a-1', 'analyst-a@example.com', 'hash4', 'tenant-a', 'analyst'),
    ('viewer-a-1', 'viewer-a@example.com', 'hash5', 'tenant-a', 'viewer'),
    ('user-b-1', 'user-b@example.com', 'hash6', 'tenant-b', 'user')
ON CONFLICT (id) DO NOTHING;

-- 创建测试策略
INSERT INTO strategies (id, tenant_id, user_id, name, code, status) VALUES
    ('strategy-a-1', 'tenant-a', 'user-a-1', 'Strategy A1', 'def run(): pass', 'active'),
    ('strategy-a-2', 'tenant-a', 'trader-a-1', 'Strategy A2', 'def run(): pass', 'draft'),
    ('strategy-b-1', 'tenant-b', 'user-b-1', 'Strategy B1', 'def run(): pass', 'active')
ON CONFLICT (id) DO NOTHING;

-- 创建测试订单
INSERT INTO orders (id, tenant_id, user_id, strategy_id, symbol, side, quantity, price, status) VALUES
    ('order-a-1', 'tenant-a', 'trader-a-1', 'strategy-a-1', 'BTCUSDT', 'buy', 0.1, 50000, 'filled'),
    ('order-a-2', 'tenant-a', 'trader-a-1', 'strategy-a-2', 'ETHUSDT', 'sell', 1.0, 3000, 'pending'),
    ('order-b-1', 'tenant-b', 'user-b-1', 'strategy-b-1', 'BTCUSDT', 'buy', 0.05, 50000, 'filled')
ON CONFLICT (id) DO NOTHING;

-- ============================================================================
-- 验证RLS配置
-- ============================================================================

-- 验证RLS已启用
DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM pg_class c
        JOIN pg_namespace n ON n.oid = c.relnamespace
        WHERE c.relname = 'strategies' AND c.relrowsecurity = true
    ) THEN
        RAISE EXCEPTION 'RLS未在strategies表上启用';
    END IF;
    
    IF NOT EXISTS (
        SELECT 1 FROM pg_class c
        JOIN pg_namespace n ON n.oid = c.relnamespace
        WHERE c.relname = 'orders' AND c.relrowsecurity = true
    ) THEN
        RAISE EXCEPTION 'RLS未在orders表上启用';
    END IF;
END $$;

-- 打印初始化信息
DO $$
BEGIN
    RAISE NOTICE '✅ 数据库初始化完成';
    RAISE NOTICE '✅ RLS策略已配置';
    RAISE NOTICE '✅ 测试数据已加载';
    RAISE NOTICE '   - 租户数: %', (SELECT COUNT(*) FROM tenants);
    RAISE NOTICE '   - 用户数: %', (SELECT COUNT(*) FROM users);
    RAISE NOTICE '   - 策略数: %', (SELECT COUNT(*) FROM strategies);
    RAISE NOTICE '   - 订单数: %', (SELECT COUNT(*) FROM orders);
END $$;

