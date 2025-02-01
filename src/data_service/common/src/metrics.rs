use lazy_static::lazy_static;
use prometheus::{
    Counter, CounterVec, Gauge, GaugeVec, Histogram, HistogramVec, IntCounter,
    IntCounterVec, IntGauge, IntGaugeVec, Registry,
};
use std::time::Instant;

lazy_static! {
    pub static ref REGISTRY: Registry = Registry::new();

    // 连接状态指标
    pub static ref CONNECTION_STATUS: IntGaugeVec = IntGaugeVec::new(
        prometheus::opts!("connection_status", "Connection status (1 for connected, 0 for disconnected)"),
        &["exchange", "connection_type"]
    ).unwrap();

    // WebSocket延迟指标
    pub static ref WEBSOCKET_LATENCY: HistogramVec = HistogramVec::new(
        prometheus::histogram_opts!(
            "websocket_latency_seconds",
            "WebSocket message latency in seconds",
            vec![0.001, 0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0]
        ),
        &["exchange", "channel"]
    ).unwrap();

    // REST API延迟指标
    pub static ref REST_API_LATENCY: HistogramVec = HistogramVec::new(
        prometheus::histogram_opts!(
            "rest_api_latency_seconds",
            "REST API request latency in seconds",
            vec![0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0]
        ),
        &["exchange", "endpoint"]
    ).unwrap();

    // 错误计数器
    pub static ref ERROR_COUNTER: CounterVec = CounterVec::new(
        prometheus::opts!("errors_total", "Total number of errors"),
        &["exchange", "error_type"]
    ).unwrap();

    // 消息计数器
    pub static ref MESSAGE_COUNTER: CounterVec = CounterVec::new(
        prometheus::opts!("messages_total", "Total number of messages"),
        &["exchange", "message_type"]
    ).unwrap();

    // 数据质量指标
    pub static ref DATA_QUALITY: GaugeVec = GaugeVec::new(
        prometheus::opts!("data_quality", "Data quality score (0-100)"),
        &["exchange", "data_type"]
    ).unwrap();
}

/// 初始化监控系统
pub fn init() {
    // 注册所有指标
    REGISTRY.register(Box::new(CONNECTION_STATUS.clone())).unwrap();
    REGISTRY.register(Box::new(WEBSOCKET_LATENCY.clone())).unwrap();
    REGISTRY.register(Box::new(REST_API_LATENCY.clone())).unwrap();
    REGISTRY.register(Box::new(ERROR_COUNTER.clone())).unwrap();
    REGISTRY.register(Box::new(MESSAGE_COUNTER.clone())).unwrap();
    REGISTRY.register(Box::new(DATA_QUALITY.clone())).unwrap();
}

/// 记录WebSocket延迟
pub fn record_ws_latency(exchange: &str, channel: &str, start_time: Instant) {
    let duration = start_time.elapsed();
    WEBSOCKET_LATENCY
        .with_label_values(&[exchange, channel])
        .observe(duration.as_secs_f64());
}

/// 记录REST API延迟
pub fn record_rest_latency(exchange: &str, endpoint: &str, start_time: Instant) {
    let duration = start_time.elapsed();
    REST_API_LATENCY
        .with_label_values(&[exchange, endpoint])
        .observe(duration.as_secs_f64());
}

/// 更新连接状态
pub fn update_connection_status(exchange: &str, connection_type: &str, is_connected: bool) {
    CONNECTION_STATUS
        .with_label_values(&[exchange, connection_type])
        .set(if is_connected { 1 } else { 0 });
}

/// 记录错误
pub fn record_error(exchange: &str, error_type: &str) {
    ERROR_COUNTER
        .with_label_values(&[exchange, error_type])
        .inc();
}

/// 记录消息
pub fn record_message(exchange: &str, message_type: &str) {
    MESSAGE_COUNTER
        .with_label_values(&[exchange, message_type])
        .inc();
}

/// 更新数据质量分数
pub fn update_data_quality(exchange: &str, data_type: &str, score: f64) {
    DATA_QUALITY
        .with_label_values(&[exchange, data_type])
        .set(score);
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn test_metrics_recording() {
        init();

        // 测试连接状态
        update_connection_status("binance", "websocket", true);
        assert_eq!(
            CONNECTION_STATUS
                .with_label_values(&["binance", "websocket"])
                .get(),
            1
        );

        // 测试错误计数
        record_error("binance", "connection_error");
        assert_eq!(
            ERROR_COUNTER
                .with_label_values(&["binance", "connection_error"])
                .get(),
            1.0
        );

        // 测试消息计数
        record_message("binance", "trade");
        assert_eq!(
            MESSAGE_COUNTER
                .with_label_values(&["binance", "trade"])
                .get(),
            1.0
        );

        // 测试延迟记录
        let start = Instant::now();
        thread::sleep(Duration::from_millis(10));
        record_ws_latency("binance", "trades", start);

        // 测试数据质量
        update_data_quality("binance", "trade_data", 95.5);
        assert_eq!(
            DATA_QUALITY
                .with_label_values(&["binance", "trade_data"])
                .get(),
            95.5
        );
    }
} 