# 监控方案

**版本**: v2.0.0  
**最后更新**: 2024-12-20

---

## 目录

1. [监控架构](#1-监控架构)
2. [关键指标定义](#2-关键指标定义)
3. [告警规则](#3-告警规则)
4. [仪表盘](#4-仪表盘)

---

## 1. 监控架构

```
Services (Rust/Java/Python)
      │
      ├─► Prometheus (指标收集)
      │         │
      │         ├─► Grafana (可视化)
      │         │
      │         └─► AlertManager (告警)
      │                  │
      │                  ├─► Slack
      │                  └─► Email
      │
      └─► ELK Stack (日志)
```

---

## 2. 关键指标定义

### 2.1 Rust服务指标 ⭐

**数据采集服务 (data-engine)**

```rust
use prometheus::{Counter, Histogram, Gauge, Registry};

lazy_static! {
    // HTTP请求延迟
    static ref HTTP_REQUEST_DURATION: Histogram = Histogram::with_opts(
        HistogramOpts::new("http_request_duration_seconds", "HTTP request duration")
            .namespace("data_engine")
            .buckets(vec![0.001, 0.005, 0.01, 0.05, 0.1, 0.5, 1.0])
    ).unwrap();
    
    // WebSocket连接数
    static ref WS_CONNECTIONS: Gauge = Gauge::new(
        "websocket_connections", 
        "Active WebSocket connections"
    ).unwrap();
    
    // 消息吞吐量
    static ref MESSAGES_PROCESSED: Counter = Counter::new(
        "messages_processed_total",
        "Total messages processed"
    ).unwrap();
    
    // 错误计数
    static ref ERRORS: Counter = Counter::with_opts(
        Opts::new("errors_total", "Total errors")
            .namespace("data_engine")
    ).unwrap();
}

// 使用示例
async fn handle_request(req: Request) -> Response {
    let timer = HTTP_REQUEST_DURATION.start_timer();
    
    // 处理请求...
    MESSAGES_PROCESSED.inc();
    
    timer.observe_duration();
    response
}
```

**暴露指标端点**

```rust
use actix_web::{web, App, HttpResponse, HttpServer};
use prometheus::{Encoder, TextEncoder};

async fn metrics() -> HttpResponse {
    let encoder = TextEncoder::new();
    let metric_families = prometheus::gather();
    let mut buffer = vec![];
    encoder.encode(&metric_families, &mut buffer).unwrap();
    
    HttpResponse::Ok()
        .content_type("text/plain; version=0.0.4")
        .body(buffer)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .route("/metrics", web::get().to(metrics))
    })
    .bind("0.0.0.0:18001")?
    .run()
    .await
}
```

### 2.2 Java服务指标

**使用Micrometer**

```java
@Configuration
public class MetricsConfig {
    
    @Bean
    public MeterRegistry meterRegistry() {
        return new PrometheusMeterRegistry(PrometheusConfig.DEFAULT);
    }
}

@Service
public class OrderService {
    
    private final Counter orderCounter;
    private final Timer orderTimer;
    
    public OrderService(MeterRegistry registry) {
        this.orderCounter = Counter.builder("orders_created_total")
            .tag("service", "trading-engine")
            .register(registry);
            
        this.orderTimer = Timer.builder("order_processing_duration")
            .tag("service", "trading-engine")
            .register(registry);
    }
    
    public Order createOrder(CreateOrderRequest request) {
        return orderTimer.record(() -> {
            Order order = // ... 处理订单
            orderCounter.increment();
            return order;
        });
    }
}
```

### 2.3 指标类别

| 类别 | 指标示例 | 类型 | 说明 |
|------|---------|------|------|
| **业务指标** | orders_total | Counter | 订单总数 |
| | active_strategies | Gauge | 活跃策略数 |
| | trade_volume_usd | Counter | 交易金额 |
| **性能指标** | request_duration_seconds | Histogram | 请求延迟 |
| | websocket_latency | Histogram | WS延迟 ⭐ |
| | message_throughput | Counter | 消息吞吐 ⭐ |
| **资源指标** | cpu_usage_percent | Gauge | CPU使用率 |
| | memory_usage_bytes | Gauge | 内存使用 |
| | goroutines_count | Gauge | 协程数 |
| **错误指标** | errors_total | Counter | 错误总数 |
| | http_5xx_total | Counter | 5xx错误 |
| | timeout_total | Counter | 超时次数 |

---

## 3. 告警规则

### 3.1 Prometheus告警规则

**prometheus/alert-rules.yml**

```yaml
groups:
  - name: data-engine-alerts
    interval: 30s
    rules:
      # 数据采集服务高错误率
      - alert: DataEngineHighErrorRate
        expr: |
          rate(data_engine_errors_total[5m]) > 0.01
        for: 5m
        labels:
          severity: warning
          service: data-engine
        annotations:
          summary: "Data Engine高错误率"
          description: "错误率: {{ $value | humanize }}/s"

      # WebSocket连接数异常
      - alert: WebSocketConnectionsLow
        expr: |
          websocket_connections < 1
        for: 2m
        labels:
          severity: critical
          service: data-engine
        annotations:
          summary: "WebSocket连接中断"
          description: "当前连接数: {{ $value }}"

      # 消息延迟过高
      - alert: MessageLatencyHigh
        expr: |
          histogram_quantile(0.99, 
            rate(message_processing_duration_bucket[5m])
          ) > 0.01
        for: 5m
        labels:
          severity: warning
        annotations:
          summary: "消息处理延迟过高"
          description: "P99延迟: {{ $value | humanizeD uration }}"

  - name: trading-engine-alerts
    interval: 30s
    rules:
      # 订单失败率过高
      - alert: HighOrderFailureRate
        expr: |
          rate(orders_failed_total[5m]) / rate(orders_total[5m]) > 0.05
        for: 5m
        labels:
          severity: critical
          service: trading-engine
        annotations:
          summary: "订单失败率过高"
          description: "失败率: {{ $value | humanizePercentage }}"

      # 订单处理延迟
      - alert: OrderProcessingSlowdown
        expr: |
          histogram_quantile(0.99,
            rate(order_processing_duration_bucket[5m])
          ) > 1.0
        for: 10m
        labels:
          severity: warning
        annotations:
          summary: "订单处理变慢"

  - name: system-alerts
    interval: 30s
    rules:
      # CPU使用率过高
      - alert: HighCPUUsage
        expr: |
          process_cpu_seconds_total > 0.8
        for: 5m
        labels:
          severity: warning
        annotations:
          summary: "CPU使用率过高"

      # 内存使用率过高
      - alert: HighMemoryUsage
        expr: |
          process_resident_memory_bytes / 1024 / 1024 / 1024 > 0.8
        for: 5m
        labels:
          severity: warning
        annotations:
          summary: "内存使用率过高"
```

### 3.2 AlertManager配置

**alertmanager/alertmanager.yml**

```yaml
global:
  resolve_timeout: 5m
  slack_api_url: 'https://hooks.slack.com/services/YOUR/SLACK/WEBHOOK'

route:
  group_by: ['alertname', 'cluster', 'service']
  group_wait: 10s
  group_interval: 10s
  repeat_interval: 12h
  receiver: 'slack-notifications'
  
  routes:
    # 关键告警立即通知
    - match:
        severity: critical
      receiver: 'slack-critical'
      continue: true
    
    # 非关键告警
    - match:
        severity: warning
      receiver: 'slack-notifications'

receivers:
  - name: 'slack-notifications'
    slack_configs:
      - channel: '#hermesflow-alerts'
        title: '{{ .GroupLabels.alertname }}'
        text: '{{ range .Alerts }}{{ .Annotations.summary }}\n{{ .Annotations.description }}\n{{ end }}'

  - name: 'slack-critical'
    slack_configs:
      - channel: '#hermesflow-critical'
        title: '🚨 CRITICAL: {{ .GroupLabels.alertname }}'
        text: '{{ range .Alerts }}{{ .Annotations.summary }}\n{{ .Annotations.description }}\n{{ end }}'
```

---

## 4. 仪表盘

### 4.1 Grafana仪表盘

**数据引擎仪表盘**

```json
{
  "dashboard": {
    "title": "Data Engine - Rust Service",
    "panels": [
      {
        "title": "WebSocket连接数",
        "targets": [
          {
            "expr": "websocket_connections"
          }
        ],
        "type": "graph"
      },
      {
        "title": "消息处理延迟 (P50/P95/P99)",
        "targets": [
          {
            "expr": "histogram_quantile(0.50, rate(message_processing_duration_bucket[5m]))",
            "legendFormat": "P50"
          },
          {
            "expr": "histogram_quantile(0.95, rate(message_processing_duration_bucket[5m]))",
            "legendFormat": "P95"
          },
          {
            "expr": "histogram_quantile(0.99, rate(message_processing_duration_bucket[5m]))",
            "legendFormat": "P99"
          }
        ]
      },
      {
        "title": "消息吞吐量",
        "targets": [
          {
            "expr": "rate(messages_processed_total[1m])"
          }
        ]
      },
      {
        "title": "错误率",
        "targets": [
          {
            "expr": "rate(errors_total[5m])"
          }
        ]
      }
    ]
  }
}
```

### 4.2 关键仪表盘列表

| 仪表盘 | 服务 | 关键指标 |
|-------|------|---------|
| Data Engine Overview | Rust | WebSocket连接、消息延迟、吞吐量 |
| Trading Engine | Java | 订单状态、执行延迟、成功率 |
| Strategy Engine | Python | 活跃策略、信号生成、回测性能 |
| Risk Engine | Java | VaR、敞口、风控触发 |
| System Overview | All | CPU、内存、网络、磁盘 |

---

**文档维护者**: DevOps Team  
**最后更新**: 2024-12-20

