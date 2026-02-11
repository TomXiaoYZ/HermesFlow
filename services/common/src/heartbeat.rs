use redis::Commands;
use tracing::{info, warn};

/// Spawn a background task that publishes heartbeats to Redis every 5 seconds.
///
/// Fails silently if Redis is unreachable (logs a warning and skips).
pub fn spawn_heartbeat(service_name: &str, redis_url: &str) {
    let name = service_name.to_owned();
    let url = redis_url.to_owned();

    tokio::spawn(async move {
        let client = match redis::Client::open(url.as_str()) {
            Ok(c) => c,
            Err(e) => {
                warn!("Heartbeat: cannot open Redis client: {}", e);
                return;
            }
        };

        let mut con = match client.get_connection() {
            Ok(c) => c,
            Err(e) => {
                warn!("Heartbeat: cannot connect to Redis: {}", e);
                return;
            }
        };

        info!("{} heartbeat started", name);
        loop {
            let hb = serde_json::json!({
                "service": name,
                "status": "online",
                "timestamp": chrono::Utc::now().timestamp_millis()
            });
            let _: redis::RedisResult<()> = con.publish("system_heartbeat", hb.to_string());
            tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
        }
    });
}
