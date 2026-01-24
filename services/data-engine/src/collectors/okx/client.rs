use super::config::OkxConfig;
use crate::error::{DataError, Result};
use chrono::Utc;
use hmac::{Hmac, Mac};
use reqwest::{Client, Method, RequestBuilder};
use serde::de::DeserializeOwned;
use sha2::Sha256;
use std::collections::BTreeMap;

type HmacSha256 = Hmac<Sha256>;

#[derive(Clone)]
pub struct OkxClient {
    client: Client,
    config: OkxConfig,
}

impl OkxClient {
    pub fn new(config: OkxConfig) -> Self {
        Self {
            client: Client::new(),
            config,
        }
    }

    /// Sign request for OKX V5
    /// Signature = HMAC-SHA256(timestamp + method + requestPath + body, secret_key)
    /// Base64 encoded
    fn sign_request(
        &self,
        method: &str,
        path: &str,
        body: &str,
        timestamp: &str,
    ) -> Result<String> {
        let payload = format!("{}{}{}{}", timestamp, method, path, body);

        let mut mac = HmacSha256::new_from_slice(self.config.secret_key.as_bytes())
            .map_err(|e| DataError::ConfigurationError(format!("Invalid secret key: {}", e)))?;
        mac.update(payload.as_bytes());
        let result = mac.finalize();
        Ok(base64::encode(result.into_bytes()))
    }

    pub async fn request<T: DeserializeOwned>(
        &self,
        method: Method,
        endpoint: &str,
        query: Option<BTreeMap<String, String>>,
        body: Option<serde_json::Value>,
    ) -> Result<T> {
        let mut url = format!("{}{}", self.config.base_url, endpoint);

        // Append query to URL if GET, strictly following OKX signature rules which include query params in path
        if let Some(mut q) = query {
            // If query exists, append to url
            let query_str = q
                .iter()
                .map(|(k, v)| format!("{}={}", k, v))
                .collect::<Vec<String>>()
                .join("&");
            if !query_str.is_empty() {
                url = format!("{}?{}", url, query_str);
            }
        }

        // Just the path part for signature
        let path = if let Some(idx) = url.find(self.config.base_url.as_str()) {
            // This is hacky, better parse URL
            // Assuming base_url is strictly the prefix.
            // Or just recreate path+query manually
            // Let's use url crate to be safe? Or simple string manipulation since we constructed it.
            // We appended endpoint to base_url. So path is endpoint + ?query
            let mut p = endpoint.to_string();
            if url.contains('?') {
                let parts: Vec<&str> = url.split('?').collect();
                if parts.len() > 1 {
                    p = format!("{}?{}", endpoint, parts[1]);
                }
            }
            p
        } else {
            endpoint.to_string()
        };

        let timestamp = Utc::now().format("%Y-%m-%dT%H:%M:%S.000Z").to_string();
        let body_str = body.as_ref().map(|b| b.to_string()).unwrap_or_default();

        let signature = self.sign_request(method.as_str(), &path, &body_str, &timestamp)?;

        let mut builder = self.client.request(method, &url);

        builder = builder
            .header("OK-ACCESS-KEY", &self.config.api_key)
            .header("OK-ACCESS-SIGN", signature)
            .header("OK-ACCESS-TIMESTAMP", timestamp)
            .header("OK-ACCESS-PASSPHRASE", &self.config.passphrase)
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
                "OKX Error: {}",
                error_text
            )));
        }

        // OKX wraps response in { "code": "0", "data": [...], "msg": "" }
        // We should parse that wrapper
        #[derive(serde::Deserialize)]
        struct OkxResponse<D> {
            code: String,
            msg: String,
            data: D,
        }

        let wrapper: OkxResponse<T> = response
            .json()
            .await
            .map_err(|e| DataError::SerializationError(e.to_string()))?;

        if wrapper.code != "0" {
            return Err(DataError::ExchangeError(format!(
                "OKX API Error {}: {}",
                wrapper.code, wrapper.msg
            )));
        }

        Ok(wrapper.data)
    }
}
