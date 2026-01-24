use super::config::BinanceConfig;
use crate::error::{DataError, Result};
use chrono::Utc;
use hmac::{Hmac, Mac};
use reqwest::{Client, Method, RequestBuilder};
use serde::de::DeserializeOwned;
use sha2::Sha256;
use std::collections::BTreeMap;
use time::OffsetDateTime;

type HmacSha256 = Hmac<Sha256>;

#[derive(Clone)]
pub struct BinanceClient {
    client: Client,
    config: BinanceConfig,
}

impl BinanceClient {
    pub fn new(config: BinanceConfig) -> Self {
        Self {
            client: Client::new(),
            config,
        }
    }

    /// Sign query parameters with HMAC-SHA256
    fn sign_request(&self, params: &mut BTreeMap<String, String>) -> Result<()> {
        params.insert(
            "timestamp".to_string(),
            Utc::now().timestamp_millis().to_string(),
        );

        let query_string = params
            .iter()
            .map(|(k, v)| format!("{}={}", k, v))
            .collect::<Vec<String>>()
            .join("&");

        let mut mac = HmacSha256::new_from_slice(self.config.secret_key.as_bytes())
            .map_err(|e| DataError::ConfigurationError(format!("Invalid secret key: {}", e)))?;
        mac.update(query_string.as_bytes());
        let result = mac.finalize();
        let signature = hex::encode(result.into_bytes());

        params.insert("signature".to_string(), signature);
        Ok(())
    }

    /// Make a public request (no signing)
    pub async fn public_request<T: DeserializeOwned>(
        &self,
        endpoint: &str,
        params: &BTreeMap<String, String>,
    ) -> Result<T> {
        let url = format!("{}{}", self.config.base_url, endpoint);
        let response = self
            .client
            .get(&url)
            .query(params)
            .send()
            .await
            .map_err(|e| DataError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(DataError::ExchangeError(format!(
                "Binance Error: {}",
                error_text
            )));
        }

        response
            .json::<T>()
            .await
            .map_err(|e| DataError::SerializationError(e.to_string()))
    }

    /// Make an authenticated request (signed)
    pub async fn signed_request<T: DeserializeOwned>(
        &self,
        method: Method,
        endpoint: &str,
        params: Option<BTreeMap<String, String>>,
    ) -> Result<T> {
        let mut params = params.unwrap_or_default();
        self.sign_request(&mut params)?;

        let url = format!("{}{}", self.config.base_url, endpoint);

        let builder = match method {
            Method::GET => self.client.get(&url).query(&params),
            Method::POST => self.client.post(&url).form(&params),
            Method::DELETE => self.client.delete(&url).query(&params),
            _ => return Err(DataError::NetworkError("Unsupported method".to_string())),
        };

        let response = builder
            .header("X-MBX-APIKEY", &self.config.api_key)
            .send()
            .await
            .map_err(|e| DataError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(DataError::ExchangeError(format!(
                "Binance Error: {}",
                error_text
            )));
        }

        response
            .json::<T>()
            .await
            .map_err(|e| DataError::SerializationError(e.to_string()))
    }
}
