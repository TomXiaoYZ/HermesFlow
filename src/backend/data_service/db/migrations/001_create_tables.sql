-- 创建市场行情表
CREATE TABLE IF NOT EXISTS market_tickers (
    id SERIAL PRIMARY KEY,
    exchange VARCHAR(50) NOT NULL,
    market VARCHAR(20) NOT NULL,
    symbol VARCHAR(20) NOT NULL,
    price DECIMAL(20, 8) NOT NULL,
    volume DECIMAL(20, 8) NOT NULL,
    amount DECIMAL(20, 8) NOT NULL,
    timestamp TIMESTAMP NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    UNIQUE (exchange, market, symbol, timestamp)
);

-- 创建订单簿表
CREATE TABLE IF NOT EXISTS order_books (
    id SERIAL PRIMARY KEY,
    exchange VARCHAR(50) NOT NULL,
    market VARCHAR(20) NOT NULL,
    symbol VARCHAR(20) NOT NULL,
    side VARCHAR(10) NOT NULL,
    price DECIMAL(20, 8) NOT NULL,
    quantity DECIMAL(20, 8) NOT NULL,
    timestamp TIMESTAMP NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    UNIQUE (exchange, market, symbol, side, price, timestamp)
);

-- 创建成交记录表
CREATE TABLE IF NOT EXISTS trades (
    id SERIAL PRIMARY KEY,
    exchange VARCHAR(50) NOT NULL,
    market VARCHAR(20) NOT NULL,
    symbol VARCHAR(20) NOT NULL,
    trade_id VARCHAR(50) NOT NULL,
    price DECIMAL(20, 8) NOT NULL,
    quantity DECIMAL(20, 8) NOT NULL,
    amount DECIMAL(20, 8) NOT NULL,
    side VARCHAR(10) NOT NULL,
    timestamp TIMESTAMP NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    UNIQUE (exchange, market, symbol, trade_id)
);

-- 创建K线数据表
CREATE TABLE IF NOT EXISTS klines (
    id SERIAL PRIMARY KEY,
    exchange VARCHAR(50) NOT NULL,
    market VARCHAR(20) NOT NULL,
    symbol VARCHAR(20) NOT NULL,
    interval VARCHAR(10) NOT NULL,
    open_time TIMESTAMP NOT NULL,
    close_time TIMESTAMP NOT NULL,
    open_price DECIMAL(20, 8) NOT NULL,
    high_price DECIMAL(20, 8) NOT NULL,
    low_price DECIMAL(20, 8) NOT NULL,
    close_price DECIMAL(20, 8) NOT NULL,
    volume DECIMAL(20, 8) NOT NULL,
    amount DECIMAL(20, 8) NOT NULL,
    trades_count INTEGER NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    UNIQUE (exchange, market, symbol, interval, open_time)
);

-- 创建订单表
CREATE TABLE IF NOT EXISTS orders (
    id SERIAL PRIMARY KEY,
    exchange VARCHAR(50) NOT NULL,
    symbol VARCHAR(20) NOT NULL,
    order_id VARCHAR(50) NOT NULL,
    client_order_id VARCHAR(50),
    price DECIMAL(20, 8) NOT NULL,
    quantity DECIMAL(20, 8) NOT NULL,
    executed_qty DECIMAL(20, 8) NOT NULL,
    executed_price DECIMAL(20, 8),
    side VARCHAR(10) NOT NULL,
    position_side VARCHAR(10) NOT NULL,
    type VARCHAR(20) NOT NULL,
    status VARCHAR(20) NOT NULL,
    time_in_force VARCHAR(10) NOT NULL,
    margin_type VARCHAR(10) NOT NULL,
    leverage INTEGER NOT NULL,
    stop_price DECIMAL(20, 8),
    timestamp TIMESTAMP NOT NULL,
    update_time TIMESTAMP NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    UNIQUE (exchange, symbol, order_id)
);

-- 创建持仓信息表
CREATE TABLE IF NOT EXISTS positions (
    id SERIAL PRIMARY KEY,
    exchange VARCHAR(50) NOT NULL,
    symbol VARCHAR(20) NOT NULL,
    position_side VARCHAR(10) NOT NULL,
    margin_type VARCHAR(10) NOT NULL,
    leverage INTEGER NOT NULL,
    quantity DECIMAL(20, 8) NOT NULL,
    entry_price DECIMAL(20, 8) NOT NULL,
    mark_price DECIMAL(20, 8) NOT NULL,
    unrealized_pnl DECIMAL(20, 8) NOT NULL,
    margin DECIMAL(20, 8) NOT NULL,
    maintenance_margin DECIMAL(20, 8) NOT NULL,
    timestamp TIMESTAMP NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    UNIQUE (exchange, symbol, position_side, timestamp)
); 