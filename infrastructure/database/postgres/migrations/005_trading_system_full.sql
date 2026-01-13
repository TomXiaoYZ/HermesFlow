-- ============================================================
-- Trading System Schema for Options Spread Trading
-- Version: 1.0.0
-- Auto-created on container startup if not exists
-- ============================================================

-- ============================================================
-- 参考数据 (Reference Data)
-- ============================================================

-- 标的基础信息
CREATE TABLE IF NOT EXISTS ref_underlyings (
    id SERIAL PRIMARY KEY,
    symbol VARCHAR(20) NOT NULL UNIQUE,
    name VARCHAR(100),
    asset_class VARCHAR(20) NOT NULL,  -- 'equity', 'etf', 'index'
    exchange VARCHAR(20),
    currency VARCHAR(3) DEFAULT 'USD',
    tick_size DECIMAL(10,4),
    lot_size INTEGER DEFAULT 100,
    is_active BOOLEAN DEFAULT true,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- 期权合约静态信息
CREATE TABLE IF NOT EXISTS ref_option_contracts (
    id SERIAL PRIMARY KEY,
    underlying_id INTEGER REFERENCES ref_underlyings(id),
    symbol VARCHAR(30) NOT NULL UNIQUE,  -- OCC格式: SPY260117C00450000
    underlying_symbol VARCHAR(20) NOT NULL,
    expiration DATE NOT NULL,
    strike DECIMAL(18,2) NOT NULL,
    option_type CHAR(1) NOT NULL,  -- 'C' or 'P'
    exercise_style CHAR(1) DEFAULT 'A',  -- 'A'merican, 'E'uropean
    multiplier INTEGER DEFAULT 100,
    is_active BOOLEAN DEFAULT true,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_ref_option_contracts_underlying 
    ON ref_option_contracts(underlying_symbol, expiration, strike);

-- ============================================================
-- 市场数据 (Market Data)
-- ============================================================

-- 股票/ETF 实时快照
CREATE TABLE IF NOT EXISTS mkt_equity_snapshots (
    id BIGSERIAL PRIMARY KEY,
    symbol VARCHAR(20) NOT NULL,
    price DECIMAL(18,4) NOT NULL,
    bid DECIMAL(18,4),
    ask DECIMAL(18,4),
    bid_size INTEGER,
    ask_size INTEGER,
    volume BIGINT,
    vwap DECIMAL(18,4),
    high DECIMAL(18,4),
    low DECIMAL(18,4),
    open DECIMAL(18,4),
    prev_close DECIMAL(18,4),
    timestamp TIMESTAMPTZ NOT NULL,
    received_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_mkt_equity_snapshots_symbol_time 
    ON mkt_equity_snapshots(symbol, timestamp DESC);

-- 期权实时快照
CREATE TABLE IF NOT EXISTS mkt_option_snapshots (
    id BIGSERIAL PRIMARY KEY,
    contract_symbol VARCHAR(30) NOT NULL,
    underlying_symbol VARCHAR(20) NOT NULL,
    expiration DATE NOT NULL,
    strike DECIMAL(18,2) NOT NULL,
    option_type CHAR(1) NOT NULL,
    
    -- 价格数据
    bid DECIMAL(18,4),
    ask DECIMAL(18,4),
    last DECIMAL(18,4),
    bid_size INTEGER,
    ask_size INTEGER,
    volume INTEGER,
    open_interest INTEGER,
    
    -- Greeks
    implied_volatility DECIMAL(8,4),
    delta DECIMAL(8,4),
    gamma DECIMAL(8,6),
    theta DECIMAL(8,4),
    vega DECIMAL(8,4),
    rho DECIMAL(8,4),
    
    -- 标的价格 (快照时)
    underlying_price DECIMAL(18,4),
    
    timestamp TIMESTAMPTZ NOT NULL,
    received_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_mkt_option_snapshots_underlying 
    ON mkt_option_snapshots(underlying_symbol, expiration, timestamp DESC);
CREATE INDEX IF NOT EXISTS idx_mkt_option_snapshots_contract 
    ON mkt_option_snapshots(contract_symbol, timestamp DESC);

-- ============================================================
-- 期权专用 (Options)
-- ============================================================

-- 期权链快照
CREATE TABLE IF NOT EXISTS opt_chain_snapshots (
    id BIGSERIAL PRIMARY KEY,
    underlying_symbol VARCHAR(20) NOT NULL,
    expiration DATE NOT NULL,
    snapshot_time TIMESTAMPTZ NOT NULL,
    chain_data JSONB NOT NULL,
    atm_strike DECIMAL(18,2),
    atm_iv DECIMAL(8,4),
    created_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_opt_chain_snapshots_lookup 
    ON opt_chain_snapshots(underlying_symbol, expiration, snapshot_time DESC);

-- IV曲面数据
CREATE TABLE IF NOT EXISTS opt_iv_surface (
    id BIGSERIAL PRIMARY KEY,
    underlying_symbol VARCHAR(20) NOT NULL,
    expiration DATE NOT NULL,
    strike DECIMAL(18,2) NOT NULL,
    option_type CHAR(1) NOT NULL,
    implied_volatility DECIMAL(8,4) NOT NULL,
    underlying_price DECIMAL(18,4),
    moneyness DECIMAL(8,4),
    days_to_expiry INTEGER,
    timestamp TIMESTAMPTZ NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_opt_iv_surface_lookup 
    ON opt_iv_surface(underlying_symbol, timestamp DESC);

-- ============================================================
-- 交易 (Trading)
-- ============================================================

-- 订单主表
CREATE TABLE IF NOT EXISTS trd_orders (
    id BIGSERIAL PRIMARY KEY,
    order_id VARCHAR(50) UNIQUE,
    parent_order_id BIGINT REFERENCES trd_orders(id),
    
    order_type VARCHAR(20) NOT NULL,
    strategy_type VARCHAR(30),
    
    underlying_symbol VARCHAR(20) NOT NULL,
    
    side VARCHAR(10) NOT NULL,
    quantity INTEGER NOT NULL,
    limit_price DECIMAL(18,4),
    stop_price DECIMAL(18,4),
    
    status VARCHAR(20) NOT NULL DEFAULT 'pending',
    
    filled_quantity INTEGER DEFAULT 0,
    avg_fill_price DECIMAL(18,4),
    commission DECIMAL(18,4),
    
    created_at TIMESTAMPTZ DEFAULT NOW(),
    submitted_at TIMESTAMPTZ,
    filled_at TIMESTAMPTZ,
    cancelled_at TIMESTAMPTZ,
    
    notes TEXT,
    metadata JSONB
);

CREATE INDEX IF NOT EXISTS idx_trd_orders_status 
    ON trd_orders(status, created_at DESC);
CREATE INDEX IF NOT EXISTS idx_trd_orders_underlying 
    ON trd_orders(underlying_symbol, created_at DESC);

-- 订单腿
CREATE TABLE IF NOT EXISTS trd_order_legs (
    id BIGSERIAL PRIMARY KEY,
    order_id BIGINT NOT NULL REFERENCES trd_orders(id) ON DELETE CASCADE,
    leg_index INTEGER NOT NULL,
    
    contract_symbol VARCHAR(30) NOT NULL,
    underlying_symbol VARCHAR(20) NOT NULL,
    expiration DATE,
    strike DECIMAL(18,2),
    option_type CHAR(1),
    
    side VARCHAR(10) NOT NULL,
    quantity INTEGER NOT NULL,
    ratio INTEGER DEFAULT 1,
    
    fill_price DECIMAL(18,4),
    filled_quantity INTEGER DEFAULT 0,
    
    UNIQUE(order_id, leg_index)
);

-- 成交记录
CREATE TABLE IF NOT EXISTS trd_executions (
    id BIGSERIAL PRIMARY KEY,
    execution_id VARCHAR(50) UNIQUE,
    order_id BIGINT REFERENCES trd_orders(id),
    order_leg_id BIGINT REFERENCES trd_order_legs(id),
    
    contract_symbol VARCHAR(30) NOT NULL,
    side VARCHAR(10) NOT NULL,
    quantity INTEGER NOT NULL,
    price DECIMAL(18,4) NOT NULL,
    commission DECIMAL(18,4),
    
    executed_at TIMESTAMPTZ NOT NULL,
    received_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_trd_executions_time 
    ON trd_executions(executed_at DESC);

-- ============================================================
-- 持仓 (Positions)
-- ============================================================

-- 当前持仓
CREATE TABLE IF NOT EXISTS pos_current (
    id BIGSERIAL PRIMARY KEY,
    
    contract_symbol VARCHAR(30) NOT NULL UNIQUE,
    underlying_symbol VARCHAR(20) NOT NULL,
    expiration DATE,
    strike DECIMAL(18,2),
    option_type CHAR(1),
    
    quantity INTEGER NOT NULL,
    avg_cost DECIMAL(18,4) NOT NULL,
    
    market_price DECIMAL(18,4),
    market_value DECIMAL(18,4),
    unrealized_pnl DECIMAL(18,4),
    unrealized_pnl_pct DECIMAL(8,4),
    
    delta DECIMAL(12,4),
    gamma DECIMAL(12,6),
    theta DECIMAL(12,4),
    vega DECIMAL(12,4),
    
    opened_at TIMESTAMPTZ NOT NULL,
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_pos_current_underlying 
    ON pos_current(underlying_symbol);

-- Spread组合持仓
CREATE TABLE IF NOT EXISTS pos_spreads (
    id BIGSERIAL PRIMARY KEY,
    spread_id VARCHAR(50) UNIQUE,
    
    strategy_type VARCHAR(30) NOT NULL,
    
    underlying_symbol VARCHAR(20) NOT NULL,
    expiration DATE NOT NULL,
    
    legs JSONB NOT NULL,
    
    net_cost DECIMAL(18,4) NOT NULL,
    current_value DECIMAL(18,4),
    max_profit DECIMAL(18,4),
    max_loss DECIMAL(18,4),
    breakeven_price DECIMAL(18,4),
    
    unrealized_pnl DECIMAL(18,4),
    realized_pnl DECIMAL(18,4) DEFAULT 0,
    
    net_delta DECIMAL(12,4),
    net_gamma DECIMAL(12,6),
    net_theta DECIMAL(12,4),
    net_vega DECIMAL(12,4),
    
    status VARCHAR(20) DEFAULT 'open',
    
    opened_at TIMESTAMPTZ DEFAULT NOW(),
    closed_at TIMESTAMPTZ,
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_pos_spreads_underlying 
    ON pos_spreads(underlying_symbol, status);
CREATE INDEX IF NOT EXISTS idx_pos_spreads_expiration 
    ON pos_spreads(expiration) WHERE status = 'open';

-- 持仓历史快照
CREATE TABLE IF NOT EXISTS pos_daily_snapshots (
    id BIGSERIAL PRIMARY KEY,
    snapshot_date DATE NOT NULL UNIQUE,
    
    total_market_value DECIMAL(18,4),
    total_cost_basis DECIMAL(18,4),
    total_unrealized_pnl DECIMAL(18,4),
    daily_realized_pnl DECIMAL(18,4),
    
    portfolio_delta DECIMAL(12,4),
    portfolio_gamma DECIMAL(12,6),
    portfolio_theta DECIMAL(12,4),
    portfolio_vega DECIMAL(12,4),
    
    positions_snapshot JSONB,
    spreads_snapshot JSONB,
    
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- ============================================================
-- 账户 (Account)
-- ============================================================

-- 账户余额
CREATE TABLE IF NOT EXISTS acct_balances (
    id BIGSERIAL PRIMARY KEY,
    
    cash_balance DECIMAL(18,4),
    buying_power DECIMAL(18,4),
    margin_used DECIMAL(18,4),
    margin_available DECIMAL(18,4),
    
    portfolio_value DECIMAL(18,4),
    daily_pnl DECIMAL(18,4),
    
    timestamp TIMESTAMPTZ DEFAULT NOW()
);

-- 每日账户快照
CREATE TABLE IF NOT EXISTS acct_daily_summary (
    id BIGSERIAL PRIMARY KEY,
    summary_date DATE NOT NULL UNIQUE,
    
    starting_balance DECIMAL(18,4),
    ending_balance DECIMAL(18,4),
    
    realized_pnl DECIMAL(18,4),
    unrealized_pnl DECIMAL(18,4),
    commissions DECIMAL(18,4),
    
    trades_count INTEGER,
    win_count INTEGER,
    loss_count INTEGER,
    
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- ============================================================
-- 策略 (Strategy)
-- ============================================================

-- 策略配置
CREATE TABLE IF NOT EXISTS strat_configs (
    id SERIAL PRIMARY KEY,
    strategy_name VARCHAR(50) NOT NULL UNIQUE,
    strategy_type VARCHAR(30) NOT NULL,
    
    entry_rules JSONB,
    exit_rules JSONB,
    position_sizing JSONB,
    
    is_active BOOLEAN DEFAULT true,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- 策略信号
CREATE TABLE IF NOT EXISTS strat_signals (
    id BIGSERIAL PRIMARY KEY,
    strategy_id INTEGER REFERENCES strat_configs(id),
    
    signal_type VARCHAR(20) NOT NULL,
    underlying_symbol VARCHAR(20) NOT NULL,
    
    signal_data JSONB,
    confidence DECIMAL(5,4),
    
    status VARCHAR(20) DEFAULT 'pending',
    
    order_id BIGINT REFERENCES trd_orders(id),
    
    generated_at TIMESTAMPTZ DEFAULT NOW(),
    executed_at TIMESTAMPTZ
);

CREATE INDEX IF NOT EXISTS idx_strat_signals_status 
    ON strat_signals(status, generated_at DESC);

-- ============================================================
-- 回测 (Backtest)
-- ============================================================

-- 回测运行记录
CREATE TABLE IF NOT EXISTS bt_runs (
    id BIGSERIAL PRIMARY KEY,
    run_name VARCHAR(100),
    strategy_id INTEGER REFERENCES strat_configs(id),
    
    start_date DATE NOT NULL,
    end_date DATE NOT NULL,
    initial_capital DECIMAL(18,4),
    parameters JSONB,
    
    final_capital DECIMAL(18,4),
    total_return DECIMAL(10,4),
    sharpe_ratio DECIMAL(8,4),
    max_drawdown DECIMAL(8,4),
    win_rate DECIMAL(5,4),
    profit_factor DECIMAL(8,4),
    total_trades INTEGER,
    
    equity_curve JSONB,
    trade_log JSONB,
    
    status VARCHAR(20) DEFAULT 'running',
    started_at TIMESTAMPTZ DEFAULT NOW(),
    completed_at TIMESTAMPTZ
);

-- 回测交易记录
CREATE TABLE IF NOT EXISTS bt_trades (
    id BIGSERIAL PRIMARY KEY,
    run_id BIGINT REFERENCES bt_runs(id) ON DELETE CASCADE,
    
    strategy_type VARCHAR(30),
    underlying_symbol VARCHAR(20),
    
    entry_date DATE,
    exit_date DATE,
    
    entry_price DECIMAL(18,4),
    exit_price DECIMAL(18,4),
    
    pnl DECIMAL(18,4),
    pnl_pct DECIMAL(8,4),
    
    trade_details JSONB
);

CREATE INDEX IF NOT EXISTS idx_bt_trades_run ON bt_trades(run_id);

-- ============================================================
-- 风控 (Risk)
-- ============================================================

-- 风控规则
CREATE TABLE IF NOT EXISTS risk_rules (
    id SERIAL PRIMARY KEY,
    rule_name VARCHAR(50) NOT NULL,
    rule_type VARCHAR(30) NOT NULL,
    
    conditions JSONB NOT NULL,
    action VARCHAR(20) NOT NULL,
    
    is_active BOOLEAN DEFAULT true,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- 风控告警
CREATE TABLE IF NOT EXISTS risk_alerts (
    id BIGSERIAL PRIMARY KEY,
    rule_id INTEGER REFERENCES risk_rules(id),
    
    alert_type VARCHAR(30) NOT NULL,
    severity VARCHAR(10) NOT NULL,
    
    message TEXT NOT NULL,
    details JSONB,
    
    is_acknowledged BOOLEAN DEFAULT false,
    acknowledged_at TIMESTAMPTZ,
    
    created_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_risk_alerts_unacked 
    ON risk_alerts(created_at DESC) WHERE NOT is_acknowledged;

-- ============================================================
-- Schema版本管理
-- ============================================================

CREATE TABLE IF NOT EXISTS schema_migrations (
    version VARCHAR(50) PRIMARY KEY,
    description TEXT,
    applied_at TIMESTAMPTZ DEFAULT NOW()
);

-- 记录本次迁移
INSERT INTO schema_migrations (version, description)
VALUES ('003_trading_system', 'Trading system schema for options spread trading')
ON CONFLICT (version) DO NOTHING;
