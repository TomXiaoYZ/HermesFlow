use lazy_static::lazy_static;
use metrics::{Counter, Gauge, Histogram, Key, KeyName, Unit, describe_counter, describe_gauge, describe_histogram};
use std::time::Instant;

lazy_static! {
    // RPC 调用相关指标
    static ref RPC_REQUESTS_TOTAL: Counter = Counter::new("uniswap_v3_rpc_requests_total")
        .with_description("RPC请求总数")
        .with_unit(Unit::Count);
    
    static ref RPC_REQUESTS_FAILED: Counter = Counter::new("uniswap_v3_rpc_requests_failed")
        .with_description("RPC请求失败数")
        .with_unit(Unit::Count);
    
    static ref RPC_REQUEST_DURATION: Histogram = Histogram::new("uniswap_v3_rpc_request_duration_seconds")
        .with_description("RPC请求耗时(秒)")
        .with_unit(Unit::Seconds);

    // 数据采集相关指标
    static ref POOL_DATA_FETCH_TOTAL: Counter = Counter::new("uniswap_v3_pool_data_fetch_total")
        .with_description("池子数据获取总次数")
        .with_unit(Unit::Count);
    
    static ref POOL_DATA_FETCH_FAILED: Counter = Counter::new("uniswap_v3_pool_data_fetch_failed")
        .with_description("池子数据获取失败次数")
        .with_unit(Unit::Count);
    
    static ref POOL_DATA_FETCH_DURATION: Histogram = Histogram::new("uniswap_v3_pool_data_fetch_duration_seconds")
        .with_description("池子数据获取耗时(秒)")
        .with_unit(Unit::Seconds);

    // 流动性相关指标
    static ref POOL_LIQUIDITY: Gauge = Gauge::new("uniswap_v3_pool_liquidity")
        .with_description("池子当前流动性")
        .with_unit(Unit::Count);
    
    static ref POOL_TVL_USD: Gauge = Gauge::new("uniswap_v3_pool_tvl_usd")
        .with_description("池子总锁仓价值(USD)")
        .with_unit(Unit::Dollars);

    // 交易相关指标
    static ref SWAP_EVENTS_TOTAL: Counter = Counter::new("uniswap_v3_swap_events_total")
        .with_description("Swap事件总数")
        .with_unit(Unit::Count);
    
    static ref SWAP_VOLUME_USD: Counter = Counter::new("uniswap_v3_swap_volume_usd")
        .with_description("交易量(USD)")
        .with_unit(Unit::Dollars);

    // 错误相关指标
    static ref ERROR_TOTAL: Counter = Counter::new("uniswap_v3_error_total")
        .with_description("错误总数")
        .with_unit(Unit::Count);

    // 缓存相关指标
    static ref CACHE_HITS: Counter = Counter::new("uniswap_v3_cache_hits")
        .with_description("缓存命中次数")
        .with_unit(Unit::Count);
    
    static ref CACHE_MISSES: Counter = Counter::new("uniswap_v3_cache_misses")
        .with_description("缓存未命中次数")
        .with_unit(Unit::Count);
    
    static ref CACHE_SIZE: Gauge = Gauge::new("uniswap_v3_cache_size")
        .with_description("缓存大小")
        .with_unit(Unit::Count);
}

/// 监控指标收集器
pub struct MetricsCollector;

impl MetricsCollector {
    /// 记录 RPC 请求
    pub fn record_rpc_request(duration: f64, is_success: bool) {
        RPC_REQUESTS_TOTAL.increment(1);
        RPC_REQUEST_DURATION.record(duration);
        
        if !is_success {
            RPC_REQUESTS_FAILED.increment(1);
        }
    }

    /// 记录池子数据获取
    pub fn record_pool_data_fetch(duration: f64, is_success: bool) {
        POOL_DATA_FETCH_TOTAL.increment(1);
        POOL_DATA_FETCH_DURATION.record(duration);
        
        if !is_success {
            POOL_DATA_FETCH_FAILED.increment(1);
        }
    }

    /// 更新池子流动性
    pub fn update_pool_liquidity(pool_address: &str, liquidity: u128) {
        POOL_LIQUIDITY.with_label("pool", pool_address).set(liquidity as f64);
    }

    /// 更新池子 TVL
    pub fn update_pool_tvl(pool_address: &str, tvl_usd: f64) {
        POOL_TVL_USD.with_label("pool", pool_address).set(tvl_usd);
    }

    /// 记录 Swap 事件
    pub fn record_swap_event(pool_address: &str, volume_usd: f64) {
        SWAP_EVENTS_TOTAL.with_label("pool", pool_address).increment(1);
        SWAP_VOLUME_USD.with_label("pool", pool_address).increment(volume_usd);
    }

    /// 记录错误
    pub fn record_error(error_type: &str) {
        ERROR_TOTAL.with_label("type", error_type).increment(1);
    }

    /// 记录缓存命中
    pub fn record_cache_hit(cache_type: &str) {
        CACHE_HITS.with_label("type", cache_type).increment(1);
    }

    /// 记录缓存未命中
    pub fn record_cache_miss(cache_type: &str) {
        CACHE_MISSES.with_label("type", cache_type).increment(1);
    }

    /// 更新缓存大小
    pub fn update_cache_size(cache_type: &str, size: usize) {
        CACHE_SIZE.with_label("type", cache_type).set(size as f64);
    }
}

/// 计时器，用于记录操作耗时
pub struct Timer {
    start: Instant,
    operation: String,
}

impl Timer {
    pub fn new(operation: &str) -> Self {
        Self {
            start: Instant::now(),
            operation: operation.to_string(),
        }
    }

    pub fn stop(self, is_success: bool) {
        let duration = self.start.elapsed().as_secs_f64();
        
        match self.operation.as_str() {
            "rpc_request" => MetricsCollector::record_rpc_request(duration, is_success),
            "pool_data_fetch" => MetricsCollector::record_pool_data_fetch(duration, is_success),
            _ => {}
        }
    }
}

/// 初始化指标描述
pub fn init_metrics() {
    describe_counter!(
        "uniswap_v3_rpc_requests_total",
        Unit::Count,
        "RPC请求总数"
    );
    describe_counter!(
        "uniswap_v3_rpc_requests_failed",
        Unit::Count,
        "RPC请求失败数"
    );
    describe_histogram!(
        "uniswap_v3_rpc_request_duration_seconds",
        Unit::Seconds,
        "RPC请求耗时(秒)"
    );
    describe_counter!(
        "uniswap_v3_pool_data_fetch_total",
        Unit::Count,
        "池子数据获取总次数"
    );
    describe_counter!(
        "uniswap_v3_pool_data_fetch_failed",
        Unit::Count,
        "池子数据获取失败次数"
    );
    describe_histogram!(
        "uniswap_v3_pool_data_fetch_duration_seconds",
        Unit::Seconds,
        "池子数据获取耗时(秒)"
    );
    describe_gauge!(
        "uniswap_v3_pool_liquidity",
        Unit::Count,
        "池子当前流动性"
    );
    describe_gauge!(
        "uniswap_v3_pool_tvl_usd",
        Unit::Dollars,
        "池子总锁仓价值(USD)"
    );
    describe_counter!(
        "uniswap_v3_swap_events_total",
        Unit::Count,
        "Swap事件总数"
    );
    describe_counter!(
        "uniswap_v3_swap_volume_usd",
        Unit::Dollars,
        "交易量(USD)"
    );
    describe_counter!(
        "uniswap_v3_error_total",
        Unit::Count,
        "错误总数"
    );
    describe_counter!(
        "uniswap_v3_cache_hits",
        Unit::Count,
        "缓存命中次数"
    );
    describe_counter!(
        "uniswap_v3_cache_misses",
        Unit::Count,
        "缓存未命中次数"
    );
    describe_gauge!(
        "uniswap_v3_cache_size",
        Unit::Count,
        "缓存大小"
    );
} 