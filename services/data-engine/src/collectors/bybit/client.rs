use super::config::BybitConfig;
use crate::error::{DataError, Result};
use chrono::Utc;
use hmac::{Hmac, Mac};
use reqwest::{Client, Method};
use serde::de::DeserializeOwned;
use sha2::Sha256;
use std::collections::BTreeMap;

type HmacSha256 = Hmac<Sha256>;

#[derive(Clone)]
pub struct BybitClient {
    client: Client,
    config: BybitConfig,
}

impl BybitClient {
    pub fn new(config: BybitConfig) -> Self {
        Self {
            client: Client::new(),
            config,
        }
    }

    /// Sign request for Bybit V5
    /// param_str: sorted query string or json body string
    /// Signature = HMAC-SHA256(timestamp + key + recv_window + param_str, secret)
    fn sign_request(&self, timestamp: &str, params: &str) -> Result<String> {
        let recv_window = "5000";
        let payload = format!(
            "{}{}{}{}",
            timestamp, self.config.api_key, recv_window, params
        );

        let mut mac = HmacSha256::new_from_slice(self.config.secret_key.as_bytes())
            .map_err(|e| DataError::ConfigurationError(format!("Invalid secret key: {}", e)))?;
        mac.update(payload.as_bytes());
        let result = mac.finalize();
        Ok(hex::encode(result.into_bytes()))
    }

    pub async fn request<T: DeserializeOwned>(
        &self,
        method: Method,
        endpoint: &str,
        query: Option<BTreeMap<String, String>>,
        body: Option<serde_json::Value>,
    ) -> Result<T> {
        let mut url = format!("{}{}", self.config.base_url, endpoint);

        let timestamp = Utc::now().timestamp_millis().to_string();
        let mut params_str = String::new();

        if method == Method::GET {
            if let Some(q) = query {
                let query_str = q
                    .iter()
                    .map(|(k, v)| format!("{}={}", k, v))
                    .collect::<Vec<String>>()
                    .join("&");
                if !query_str.is_empty() {
                    url = format!("{}?{}", url, query_str);
                    params_str = query_str;
                }
            }
        } else {
            if let Some(b) = &body {
                params_str = b.to_string();
            }
        }

        let signature = self.sign_request(&timestamp, &params_str)?;

        let mut builder = self.client.request(method, &url);

        builder = builder
            .header("X-BAPI-API-KEY", &self.config.api_key)
            .header("X-BAPI-SIGN", signature)
            .header("X-BAPI-TIMESTAMP", timestamp)
            .header("X-BAPI-RECV-WINDOW", "5000")
            .header("Content-Type", "application/json");

        if let Some(b) = body {
            builder = builder.json(&b);
        }

        let response = builder
            .send()
            .await
            .map_err(|e| DataError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(DataError::ExchangeError(format!(
                "Bybit Error: {}",
                error_text
            )));
        }

        // Bybit V5 Wrapper: { "retCode": 0, "retMsg": "OK", "result": {...} }
        #[derive(serde::Deserialize)]
        struct BybitResponse<D> {
            #[serde(rename = "retCode")]
            ret_code: i32,
            #[serde(rename = "retMsg")]
            ret_msg: String,
            result: D,
        }

        let wrapper: BybitResponse<T> = response
            .json()
            .await
            .map_err(|e| DataError::SerializationError(e.to_string()))?;

        if wrapper.ret_code != 0 {
            return Err(DataError::ExchangeError(format!(
                "Bybit API Error {}: {}",
                wrapper.ret_code, wrapper.ret_msg
            )));
        }

        Ok(wrapper.result)
    }
}
