use anyhow::{Context, Result};
use reqwest::Client;
use serde::Deserialize;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;
use tracing::{debug, warn};

#[derive(Clone)]
pub struct MassiveClient {
    client: Client,
    api_key: String,
    // Rate limiter state: last request time
    last_request: std::sync::Arc<Mutex<Instant>>,
    // Minimum interval between requests (e.g. 12s for 5/min)
    min_interval: Duration,
    base_url: String,
}

#[allow(non_snake_case)]
#[derive(Debug, Deserialize)]
pub struct AggregatesResponse {
    pub ticker: String,
    pub status: String,
    pub queryCount: Option<i64>,
    pub resultsCount: Option<i64>,
    pub adjusted: bool,
    #[serde(default)]
    pub results: Vec<AggregateResult>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct AggregateResult {
    pub v: f64,          // Volume
    pub vw: Option<f64>, // VWAP
    pub o: f64,          // Open
    pub c: f64,          // Close
    pub h: f64,          // High
    pub l: f64,          // Low
    pub t: i64,          // Timestamp (Unix Msec)
    pub n: Option<i64>,  // Number of transactions
}

impl MassiveClient {
    pub fn new(api_key: String, rate_limit_per_min: u64, base_url: String) -> Self {
        let interval_secs = if rate_limit_per_min > 0 {
            60.0 / rate_limit_per_min as f64
        } else {
            0.0
        };

        Self {
            client: Client::new(),
            api_key,
            last_request: std::sync::Arc::new(Mutex::new(Instant::now() - Duration::from_secs(60))),
            min_interval: Duration::from_secs_f64(interval_secs),
            base_url,
        }
    }

    async fn rate_limit(&self) {
        let mut last = self.last_request.lock().await;
        let now = Instant::now();
        let elapsed = now.duration_since(*last);

        if elapsed < self.min_interval {
            let wait = self.min_interval - elapsed;
            debug!("Rate limiting: Waiting for {:?}", wait);
            tokio::time::sleep(wait).await;
            *last = Instant::now();
        } else {
            *last = now;
        }
    }

    pub fn api_key(&self) -> &str {
        &self.api_key
    }

    pub async fn get_aggregates(
        &self,
        ticker: &str,
        multiplier: i32,
        timespan: &str, // minute, hour, day
        from: &str,     // YYYY-MM-DD
        to: &str,       // YYYY-MM-DD
    ) -> Result<Vec<AggregateResult>> {
        self.rate_limit().await;

        let url = format!(
            "{}/v2/aggs/ticker/{}/range/{}/{}/{}/{}",
            self.base_url, ticker, multiplier, timespan, from, to
        );

        debug!("Fetching aggregates: {}", url);

        let resp = self
            .client
            .get(&url)
            .query(&[
                ("apiKey", self.api_key.as_str()),
                ("adjusted", "true"),
                ("sort", "asc"),
                ("limit", "50000"),
            ])
            .send()
            .await
            .context("Failed to send request to Massive/Polygon")?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            warn!("Massive API Error: {} - {}", status, text);
            return Err(anyhow::anyhow!("Massive API returned error: {}", status));
        }

        let data: AggregatesResponse = resp
            .json()
            .await
            .context("Failed to parse Massive aggregates")?;

        if data.status != "OK" {
            // Sometimes status is "DELAYED" which is fine, but "ERROR" is bad.
            if data.status == "ERROR" {
                return Err(anyhow::anyhow!(
                    "Massive API returned status: {}",
                    data.status
                ));
            }
        }

        Ok(data.results)
    }
}
