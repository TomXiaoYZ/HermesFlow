use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;
use tokio::time::interval;
use tracing::{error, info, warn};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceHealth {
    pub service: String,
    pub status: String,
    pub timestamp: String,
    pub details: Option<serde_json::Value>,
}

#[derive(Clone)]
pub struct HealthChecker {
    client: reqwest::Client,
    services: Vec<ServiceEndpoint>,
}

#[derive(Clone)]
struct ServiceEndpoint {
    name: String,
    url: String,
}

impl HealthChecker {
    pub fn new() -> Self {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(3))
            .build()
            .unwrap();

        let services = vec![
            ServiceEndpoint {
                name: "data-engine".to_string(),
                url: "http://data-engine:8080/health".to_string(),
            },
            ServiceEndpoint {
                name: "strategy-engine".to_string(),
                url: "http://strategy-engine:8082/health".to_string(),
            },
            ServiceEndpoint {
                name: "execution-engine".to_string(),
                url: "http://execution-engine:8083/health".to_string(),
            },
            ServiceEndpoint {
                name: "strategy-generator".to_string(),
                url: "http://strategy-generator:8084/health".to_string(),
            },
        ];

        Self { client, services }
    }

    pub async fn start_monitoring(self, redis_client: redis::Client) {
        let mut interval = interval(Duration::from_secs(5));
        
        info!("Health checker started, polling every 5s");

        loop {
            interval.tick().await;

            for service in &self.services {
                match self.check_health(&service).await {
                    Ok(health) => {
                        // Store in Redis
                        if let Ok(mut conn) = redis_client.get_connection() {
                            let key = format!("service:health:{}", service.name);
                            let value = serde_json::to_string(&health).unwrap();
                            let _: redis::RedisResult<()> = redis::Commands::set_ex(&mut conn, key, value, 30);
                        }

                        // Publish heartbeat (for backward compatibility)
                        if let Ok(mut conn) = redis_client.get_connection() {
                            let hb = serde_json::json!({
                                "service": service.name,
                                "status": if health.status == "healthy" { "online" } else { "degraded" },
                                "timestamp": chrono::Utc::now().timestamp_millis()
                            });
                            let _: redis::RedisResult<()> = redis::Commands::publish(&mut conn, "system_heartbeat", hb.to_string());
                        }
                    }
                    Err(e) => {
                        warn!("Health check failed for {}: {}", service.name, e);
                        
                        // Publish offline status
                        if let Ok(mut conn) = redis_client.get_connection() {
                            let hb = serde_json::json!({
                                "service": service.name,
                                "status": "offline",
                                "timestamp": chrono::Utc::now().timestamp_millis()
                            });
                            let _: redis::RedisResult<()> = redis::Commands::publish(&mut conn, "system_heartbeat", hb.to_string());
                        }
                    }
                }
            }
        }
    }

    async fn check_health(&self, service: &ServiceEndpoint) -> Result<ServiceHealth, reqwest::Error> {
        let response = self.client.get(&service.url).send().await?;
        
        if response.status().is_success() {
            let health: ServiceHealth = response.json().await?;
            Ok(health)
        } else {
            Err(reqwest::Error::from(response.error_for_status().unwrap_err()))
        }
    }
}
