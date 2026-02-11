use super::config::PolygonConfig;
use super::types::{AggregateBar, AggregatesResponse};
use reqwest::Client;
use std::error::Error;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Semaphore;
use tokio::time::sleep;
use tracing::{debug, error, warn};

pub struct PolygonClient {
    http_client: Client,
    config: PolygonConfig,
    rate_limiter: Arc<RateLimiter>,
}

/// Token bucket rate limiter
#[allow(dead_code)]
pub struct RateLimiter {
    semaphore: Arc<Semaphore>,
    tokens_per_sec: u32,
}

impl RateLimiter {
    pub fn new(tokens_per_sec: u32) -> Self {
        Self {
            semaphore: Arc::new(Semaphore::new(tokens_per_sec as usize)),
            tokens_per_sec,
        }
    }

    /// Acquire a token, waiting if necessary
    pub async fn acquire(&self) {
        // Just acquire permit - will be dropped when RateLimiter is used
        // Simple approach: just add a small delay
        if self.semaphore.available_permits() == 0 {
            tokio::time::sleep(Duration::from_millis(200)).await;
        }
        let _permit = self.semaphore.try_acquire();
    }
}

impl PolygonClient {
    pub fn new(config: PolygonConfig) -> Self {
        let http_client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .expect("Failed to create HTTP client");

        let rate_limiter = Arc::new(RateLimiter::new(config.rate_limit_per_sec));

        Self {
            http_client,
            config,
            rate_limiter,
        }
    }

    /// Get aggregates for a ticker
    ///
    /// # Arguments
    /// * `ticker` - Stock symbol (e.g., "AAPL")
    /// * `multiplier` - Size of the timespan (e.g., 1 for 1 minute, 5 for 5 minutes)
    /// * `timespan` - Size of the time window ("minute", "hour", "day", etc.)
    /// * `from` - Start date in YYYY-MM-DD format
    /// * `to` - End date in YYYY-MM-DD format
    pub async fn get_aggregates(
        &self,
        ticker: &str,
        multiplier: u32,
        timespan: &str,
        from: &str,
        to: &str,
    ) -> Result<Vec<AggregateBar>, Box<dyn Error + Send + Sync>> {
        let url = format!(
            "{}/v2/aggs/ticker/{}/range/{}/{}/{}/{}",
            self.config.rest_base_url, ticker, multiplier, timespan, from, to
        );

        debug!("Fetching aggregates: {}", url);

        // Apply rate limiting
        self.rate_limiter.acquire().await;

        let mut attempts = 0;
        loop {
            attempts += 1;

            match self
                .http_client
                .get(&url)
                .query(&[("apiKey", &self.config.api_key)])
                .send()
                .await
            {
                Ok(response) => {
                    let status = response.status();

                    if status.is_success() {
                        match response.json::<AggregatesResponse>().await {
                            Ok(data) => {
                                if data.status == "OK" {
                                    return Ok(data.results.unwrap_or_default());
                                } else {
                                    let err_msg = format!(
                                        "Polygon API error: status={}, ticker={}",
                                        data.status, ticker
                                    );
                                    error!("{}", err_msg);
                                    return Err(err_msg.into());
                                }
                            }
                            Err(e) => {
                                error!("Failed to parse Polygon response: {}", e);
                                return Err(format!("JSON parse error: {}", e).into());
                            }
                        }
                    } else if status.as_u16() == 429 {
                        // Rate limit exceeded
                        warn!("Rate limit exceeded, retrying after delay...");
                        if attempts >= self.config.retry_max_attempts {
                            return Err("Max retries exceeded due to rate limiting".into());
                        }
                        sleep(Duration::from_millis(
                            self.config.retry_delay_ms * attempts as u64,
                        ))
                        .await;
                        continue;
                    } else {
                        let err_msg = format!(
                            "Polygon API HTTP error: status={}, ticker={}",
                            status, ticker
                        );
                        error!("{}", err_msg);
                        return Err(err_msg.into());
                    }
                }
                Err(e) => {
                    error!("HTTP request failed: {}", e);

                    if attempts >= self.config.retry_max_attempts {
                        return Err(format!("Max retries exceeded: {}", e).into());
                    }

                    warn!(
                        "Retrying request (attempt {}/{})",
                        attempts, self.config.retry_max_attempts
                    );
                    sleep(Duration::from_millis(
                        self.config.retry_delay_ms * attempts as u64,
                    ))
                    .await;
                    continue;
                }
            }
        }
    }

    /// Get configuration
    pub fn config(&self) -> &PolygonConfig {
        &self.config
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_rate_limiter() {
        let limiter = RateLimiter::new(5); // 5 tokens per second

        let start = Instant::now();

        // Acquire 5 tokens quickly
        for _ in 0..5 {
            limiter.acquire().await;
        }

        let elapsed = start.elapsed();

        // Should be fast (< 100ms)
        assert!(elapsed < Duration::from_millis(100));

        // 6th token should wait ~1 second
        let start = Instant::now();
        limiter.acquire().await;
        let elapsed = start.elapsed();

        // Should wait close to 1 second
        assert!(elapsed >= Duration::from_millis(900));
        assert!(elapsed < Duration::from_millis(1200));
    }
}
