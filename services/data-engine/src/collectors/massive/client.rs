use reqwest::Client;
use rust_decimal::Decimal;
use serde::Deserialize;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;
use tracing::{debug, warn};

use crate::error::{retry_with_backoff, DataError, Result};

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
    #[serde(deserialize_with = "deserialize_decimal_from_number")]
    pub v: Decimal, // Volume
    #[serde(default, deserialize_with = "deserialize_option_decimal_from_number")]
    pub vw: Option<Decimal>, // VWAP
    #[serde(deserialize_with = "deserialize_decimal_from_number")]
    pub o: Decimal, // Open
    #[serde(deserialize_with = "deserialize_decimal_from_number")]
    pub c: Decimal, // Close
    #[serde(deserialize_with = "deserialize_decimal_from_number")]
    pub h: Decimal, // High
    #[serde(deserialize_with = "deserialize_decimal_from_number")]
    pub l: Decimal, // Low
    pub t: i64,         // Timestamp (Unix Msec)
    pub n: Option<i64>, // Number of transactions
}

/// Deserialize a JSON number (int or float) into Decimal
fn deserialize_decimal_from_number<'de, D>(
    deserializer: D,
) -> std::result::Result<Decimal, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::de;

    struct DecimalVisitor;

    impl<'de> de::Visitor<'de> for DecimalVisitor {
        type Value = Decimal;

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter.write_str("a number")
        }

        fn visit_f64<E: de::Error>(self, v: f64) -> std::result::Result<Decimal, E> {
            Decimal::from_f64_retain(v)
                .ok_or_else(|| E::custom(format!("cannot convert {v} to Decimal")))
        }

        fn visit_i64<E: de::Error>(self, v: i64) -> std::result::Result<Decimal, E> {
            Ok(Decimal::from(v))
        }

        fn visit_u64<E: de::Error>(self, v: u64) -> std::result::Result<Decimal, E> {
            Ok(Decimal::from(v))
        }
    }

    deserializer.deserialize_any(DecimalVisitor)
}

/// Deserialize an optional JSON number into Option<Decimal>
fn deserialize_option_decimal_from_number<'de, D>(
    deserializer: D,
) -> std::result::Result<Option<Decimal>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::de;

    struct OptionDecimalVisitor;

    impl<'de> de::Visitor<'de> for OptionDecimalVisitor {
        type Value = Option<Decimal>;

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter.write_str("a number or null")
        }

        fn visit_none<E: de::Error>(self) -> std::result::Result<Option<Decimal>, E> {
            Ok(None)
        }

        fn visit_some<D2: serde::Deserializer<'de>>(
            self,
            deserializer: D2,
        ) -> std::result::Result<Option<Decimal>, D2::Error> {
            deserialize_decimal_from_number(deserializer).map(Some)
        }

        fn visit_f64<E: de::Error>(self, v: f64) -> std::result::Result<Option<Decimal>, E> {
            Ok(Decimal::from_f64_retain(v))
        }

        fn visit_i64<E: de::Error>(self, v: i64) -> std::result::Result<Option<Decimal>, E> {
            Ok(Some(Decimal::from(v)))
        }

        fn visit_u64<E: de::Error>(self, v: u64) -> std::result::Result<Option<Decimal>, E> {
            Ok(Some(Decimal::from(v)))
        }

        fn visit_unit<E: de::Error>(self) -> std::result::Result<Option<Decimal>, E> {
            Ok(None)
        }
    }

    deserializer.deserialize_any(OptionDecimalVisitor)
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
        let url = format!(
            "{}/v2/aggs/ticker/{}/range/{}/{}/{}/{}",
            self.base_url, ticker, multiplier, timespan, from, to
        );
        let api_key = self.api_key.clone();
        let client = self.client.clone();

        self.rate_limit().await;

        retry_with_backoff(
            || {
                let url = url.clone();
                let api_key = api_key.clone();
                let client = client.clone();
                async move {
                    debug!("Fetching aggregates: {}", url);

                    let resp = client
                        .get(&url)
                        .query(&[
                            ("apiKey", api_key.as_str()),
                            ("adjusted", "true"),
                            ("sort", "asc"),
                            ("limit", "50000"),
                        ])
                        .send()
                        .await
                        .map_err(|e| DataError::ConnectionFailed {
                            data_source: "Polygon".to_string(),
                            reason: format!("HTTP request failed: {}", e),
                        })?;

                    if !resp.status().is_success() {
                        let status = resp.status();
                        let text = resp.text().await.unwrap_or_default();
                        warn!("Massive API Error: {} - {}", status, text);
                        return Err(DataError::ExchangeError(format!(
                            "Polygon API returned {}: {}",
                            status, text
                        )));
                    }

                    let data: AggregatesResponse =
                        resp.json().await.map_err(|e| DataError::ParseError {
                            data_source: "Polygon".to_string(),
                            message: format!("Failed to parse aggregates: {}", e),
                            raw_data: String::new(),
                        })?;

                    if data.status == "ERROR" {
                        return Err(DataError::ExchangeError(format!(
                            "Polygon API returned status: {}",
                            data.status
                        )));
                    }

                    Ok(data.results)
                }
            },
            3,
            1000,
        )
        .await
    }
}
