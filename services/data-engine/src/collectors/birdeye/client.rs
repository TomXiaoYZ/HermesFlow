use crate::collectors::birdeye::config::BirdeyeConfig;
use crate::monitoring::metrics::BIRDEYE_API_REQUESTS_TOTAL;
use reqwest::header::{HeaderMap, HeaderValue, ACCEPT, CONTENT_TYPE};
use serde::{Deserialize, Serialize};
use std::error::Error;
use tracing::warn;

#[derive(Clone)]
pub struct BirdeyeClient {
    client: reqwest::Client,
    pub config: BirdeyeConfig,
}

#[derive(Debug, Deserialize)]
pub struct BirdeyeResponse<T> {
    pub success: bool,
    pub message: Option<String>,
    pub data: Option<T>,
}

#[derive(Debug, Deserialize)]
pub struct TokenOverview {
    pub address: String,
    pub decimals: Option<u8>,
    pub symbol: Option<String>,
    pub name: Option<String>,
    pub liquidity: Option<f64>,
    pub price: Option<f64>,
    #[serde(rename = "v24hUSD")]
    pub volume_24h: Option<f64>,
    #[serde(rename = "mc")]
    pub market_cap: Option<f64>, // Using MC as proxy for FDV if FDV missing, but usually FDV is better.
                                 // Birdeye API has 'fdv' field? Let's check docs or fallback.
                                 // "stats" endpoint has detailed info. "token_overview" has liquidity.
}

#[derive(Debug, Deserialize)]
pub struct OhlcvItem {
    pub address: String, // Added manually if not in response
    #[serde(rename = "o")]
    pub open: f64,
    #[serde(rename = "h")]
    pub high: f64,
    #[serde(rename = "l")]
    pub low: f64,
    #[serde(rename = "c")]
    pub close: f64,
    #[serde(rename = "v")]
    pub volume: f64,
    #[serde(rename = "unixTime")]
    pub unix_time: i64,
    // Birdeye history API usually doesn't return liquidity per candle.
    // We might need to approximate or fetch separately if needed.
}

#[derive(Debug, Deserialize)]
struct HistoryData {
    items: Vec<OhlcvItem>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct TrendingToken {
    pub address: String,
    pub symbol: Option<String>,
    pub name: Option<String>,
    pub decimals: Option<u8>,
    pub liquidity: Option<f64>,
    pub fdv: Option<f64>,
    #[serde(rename = "v24hChangePercent")]
    pub price_change_24h: Option<f64>,
    #[serde(rename = "mc")]
    pub market_cap: Option<f64>,
    #[serde(rename = "v24hUSD")]
    pub volume_24h: Option<f64>,
}

#[derive(Debug, Deserialize)]
struct TrendingData {
    tokens: Vec<TrendingToken>,
}

impl BirdeyeClient {
    pub fn new(config: BirdeyeConfig) -> Self {
        tracing::info!("Initializing BirdeyeClient with config: {:?}", config);
        let mut headers = HeaderMap::new();
        headers.insert(ACCEPT, HeaderValue::from_static("application/json"));
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        if !config.api_key.is_empty() {
            headers.insert("X-API-KEY", HeaderValue::from_str(&config.api_key).unwrap());
        }

        let client = reqwest::Client::builder()
            .default_headers(headers)
            .build()
            .unwrap();

        Self { client, config }
    }

    pub async fn get_token_overview(
        &self,
        address: &str,
    ) -> Result<TokenOverview, Box<dyn Error + Send + Sync>> {
        let url = format!(
            "{}/defi/token_overview?address={}",
            self.config.base_url, address
        );
        BIRDEYE_API_REQUESTS_TOTAL.inc();
        let resp = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| Box::new(e) as Box<dyn Error + Send + Sync>)?;

        // Handle rate limits?
        if resp.status() == 429 {
            return Err("Rate limited".into());
        }

        let text = resp
            .text()
            .await
            .map_err(|e| Box::new(e) as Box<dyn Error + Send + Sync>)?;
        let result: BirdeyeResponse<TokenOverview> =
            serde_json::from_str(&text).map_err(|e| Box::new(e) as Box<dyn Error + Send + Sync>)?;

        if result.success {
            if let Some(data) = result.data {
                Ok(data)
            } else {
                Err(result.message.unwrap_or("No data returned".into()).into())
            }
        } else {
            Err(result.message.unwrap_or("Unknown error".into()).into())
        }
    }

    pub async fn get_history(
        &self,
        address: &str,
        time_from: i64,
        time_to: i64,
        resolution: &str,
    ) -> Result<Vec<OhlcvItem>, Box<dyn Error + Send + Sync>> {
        // resolution: 1m, 1h, 1d
        let type_param = match resolution {
            "1d" => "1D",
            "1h" => "1H",
            "15m" => "15m",
            "1m" => "1m",
            _ => "1D",
        };

        let url = format!(
            "{}/defi/ohlcv?address={}&type={}&time_from={}&time_to={}",
            self.config.base_url, address, type_param, time_from, time_to
        );

        BIRDEYE_API_REQUESTS_TOTAL.inc();
        let resp = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| Box::new(e) as Box<dyn Error + Send + Sync>)?;
        let text = resp
            .text()
            .await
            .map_err(|e| Box::new(e) as Box<dyn Error + Send + Sync>)?;
        tracing::info!("[Birdeye Raw] {}", &text[0..std::cmp::min(200, text.len())]);
        // Note: Response format might be data: { items: [...] }
        let result: BirdeyeResponse<HistoryData> =
            serde_json::from_str(&text).map_err(|e| Box::new(e) as Box<dyn Error + Send + Sync>)?;

        if result.success {
            if let Some(data) = result.data {
                let mut items = data.items;
                for (i, item) in items.iter_mut().enumerate() {
                    if i < 3 {
                        tracing::info!(
                            "Parsed Item: Time={}, Vol={}, Close={}",
                            item.unix_time,
                            item.volume,
                            item.close
                        );
                    }
                    item.address = address.to_string();
                }
                Ok(items)
            } else {
                // Sometimes history is empty, check message
                Err(result.message.unwrap_or("No data returned".into()).into())
            }
        } else {
            Err(result.message.unwrap_or("Unknown error".into()).into())
        }
    }

    pub async fn get_trending_tokens(
        &self,
        limit: usize,
        offset: usize,
    ) -> Result<Vec<TrendingToken>, Box<dyn Error + Send + Sync>> {
        let url = format!(
            "{}/defi/token_trending?sort_by=rank&sort_type=asc&offset={}&limit={}",
            self.config.base_url, offset, limit
        );

        let mut attempts = 0;
        let max_attempts = 3;
        loop {
            attempts += 1;
            BIRDEYE_API_REQUESTS_TOTAL.inc();
            match self.client.get(&url).send().await {
                Ok(resp) => {
                    if resp.status() == 429 {
                        warn!(
                            "[Birdeye] Rate limited (Attempt {}/{})",
                            attempts, max_attempts
                        );
                        if attempts >= max_attempts {
                            return Err("Rate limited".into());
                        }
                        tokio::time::sleep(tokio::time::Duration::from_secs(2 * attempts as u64))
                            .await;
                        continue;
                    }

                    match resp.text().await {
                        Ok(text) => {
                            let result: BirdeyeResponse<TrendingData> = serde_json::from_str(&text)
                                .map_err(|e| Box::new(e) as Box<dyn Error + Send + Sync>)?;

                            if result.success {
                                if let Some(data) = result.data {
                                    return Ok(data.tokens);
                                } else {
                                    return Ok(Vec::new());
                                }
                            } else {
                                return Err(result
                                    .message
                                    .unwrap_or("Failed to fetch trending tokens".into())
                                    .into());
                            }
                        }
                        Err(e) => {
                            warn!("[Birdeye] Failed to read text: {}", e);
                        }
                    }
                }
                Err(e) => {
                    warn!(
                        "[Birdeye] Network error (Attempt {}/{}): {}",
                        attempts, max_attempts, e
                    );
                }
            }

            if attempts >= max_attempts {
                return Err("Failed after max retries".into());
            }
            tokio::time::sleep(tokio::time::Duration::from_millis(500 * attempts as u64)).await;
        }
    }
}
