use crate::collectors::jupiter::config::JupiterConfig;
use reqwest::header::{HeaderMap, HeaderValue, ACCEPT, CONTENT_TYPE};
use serde::Deserialize;
use std::collections::HashMap;
use std::error::Error;
use tracing::info;

#[derive(Clone)]
pub struct JupiterClient {
    client: reqwest::Client,
    pub config: JupiterConfig,
}

#[derive(Debug, Deserialize)]
pub struct JupiterPriceItem {
    pub id: Option<String>, // V3 uses key as ID, but maybe id field exists in some cases? Sample didn't show it.
    #[serde(rename = "type")]
    pub price_type: Option<String>,
    #[serde(rename = "usdPrice")]
    pub price: f64, // V3 returns number
                    // extra fields ignored
}

// V3 returns HashMap<String, JupiterPriceItem> directly, no "data" wrapper
// But let's check if we can deserialize directly into HashMap

impl JupiterClient {
    pub fn new(config: JupiterConfig) -> Self {
        info!("Initializing JupiterClient with config: {:?}", config);
        let mut headers = HeaderMap::new();
        headers.insert(ACCEPT, HeaderValue::from_static("application/json"));
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

        if let Some(key) = &config.api_key {
            if let Ok(val) = HeaderValue::from_str(key) {
                headers.insert("x-api-key", val);
            }
        }

        let client = reqwest::Client::builder()
            .default_headers(headers)
            .build()
            .unwrap();

        Self { client, config }
    }

    /// Fetch prices for a list of mint addresses (comma separated)
    /// Max 100 per request recommended by Jupiter
    pub async fn get_prices(
        &self,
        ids: &[String],
    ) -> Result<HashMap<String, JupiterPriceItem>, Box<dyn Error + Send + Sync>> {
        if ids.is_empty() {
            return Ok(HashMap::new());
        }

        // Jupiter allows up to 100 IDs. We should batch them if sending more,
        // but for now let's assume the caller handles batching or we do it here.
        // Let's keep it simple: the caller (Connector) does batching logic.

        let ids_str = ids.join(",");
        let url = format!("{}?ids={}", self.config.api_url, ids_str);

        let resp = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| Box::new(e) as Box<dyn Error + Send + Sync>)?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            return Err(format!("Jupiter Price API Error {}: {}", status, text).into());
        }

        // V3 returns HashMap<String, Item> directly
        let result: HashMap<String, Option<JupiterPriceItem>> = resp
            .json()
            .await
            .map_err(|e| Box::new(e) as Box<dyn Error + Send + Sync>)?;

        // Filter out None values
        let mut valid_prices = HashMap::new();
        for (k, v) in result {
            if let Some(mut item) = v {
                // Ensure ID is set (from key)
                if item.id.is_none() {
                    item.id = Some(k.clone());
                }
                valid_prices.insert(k, item);
            }
        }

        Ok(valid_prices)
    }
}
