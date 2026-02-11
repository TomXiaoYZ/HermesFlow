use serde::{Deserialize, Serialize};

/// Polygon API aggregate bar response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AggregateBar {
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

    #[serde(rename = "vw")]
    pub vwap: Option<f64>,

    #[serde(rename = "t")]
    pub timestamp: i64, // Unix timestamp in milliseconds

    #[serde(rename = "n")]
    pub transactions: Option<i64>,
}

/// Polygon API aggregates response wrapper
#[derive(Debug, Deserialize)]
pub struct AggregatesResponse {
    pub ticker: String,
    #[serde(rename = "queryCount")]
    pub query_count: i64,
    #[serde(rename = "resultsCount")]
    pub results_count: i64,
    pub adjusted: bool,
    pub results: Option<Vec<AggregateBar>>,
    pub status: String,
    #[serde(rename = "request_id")]
    pub request_id: Option<String>,
    pub count: Option<i64>,
}

/// WebSocket aggregate message (AM event)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebSocketAggregate {
    #[serde(rename = "ev")]
    pub event_type: String, // "AM" for aggregates per minute

    #[serde(rename = "sym")]
    pub symbol: String,

    #[serde(rename = "v")]
    pub volume: f64,

    #[serde(rename = "av")]
    pub accumulated_volume: Option<f64>,

    #[serde(rename = "op")]
    pub official_open: Option<f64>,

    #[serde(rename = "vw")]
    pub vwap: Option<f64>,

    #[serde(rename = "o")]
    pub open: f64,

    #[serde(rename = "c")]
    pub close: f64,

    #[serde(rename = "h")]
    pub high: f64,

    #[serde(rename = "l")]
    pub low: f64,

    #[serde(rename = "a")]
    pub average: Option<f64>,

    #[serde(rename = "z")]
    pub average_trade_size: Option<i64>,

    #[serde(rename = "s")]
    pub start_timestamp: i64, // Unix timestamp in milliseconds

    #[serde(rename = "e")]
    pub end_timestamp: i64, // Unix timestamp in milliseconds
}

/// WebSocket authentication message
#[derive(Debug, Serialize)]
pub struct AuthMessage {
    pub action: String, // "auth"
    pub params: String, // API key
}

/// WebSocket subscription message
#[derive(Debug, Serialize)]
pub struct SubscribeMessage {
    pub action: String, // "subscribe"
    pub params: String, // "AM.*" or "AM.AAPL,AM.MSFT"
}

/// Resolution mapping for Polygon API
pub enum PolygonTimespan {
    Minute,
    Hour,
    Day,
    Week,
    Month,
}

impl PolygonTimespan {
    pub fn as_str(&self) -> &str {
        match self {
            PolygonTimespan::Minute => "minute",
            PolygonTimespan::Hour => "hour",
            PolygonTimespan::Day => "day",
            PolygonTimespan::Week => "week",
            PolygonTimespan::Month => "month",
        }
    }
}

/// Convert HermesFlow resolution format to Polygon API format
pub fn resolution_to_polygon_params(resolution: &str) -> Result<(u32, PolygonTimespan), String> {
    match resolution {
        "1m" => Ok((1, PolygonTimespan::Minute)),
        "5m" => Ok((5, PolygonTimespan::Minute)),
        "15m" => Ok((15, PolygonTimespan::Minute)),
        "30m" => Ok((30, PolygonTimespan::Minute)),
        "1h" => Ok((1, PolygonTimespan::Hour)),
        "4h" => Ok((4, PolygonTimespan::Hour)),
        "1d" => Ok((1, PolygonTimespan::Day)),
        "1w" => Ok((1, PolygonTimespan::Week)),
        _ => Err(format!("Unsupported resolution: {}", resolution)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolution_mapping() {
        let (mult, ts) = resolution_to_polygon_params("1m").unwrap();
        assert_eq!(mult, 1);
        assert_eq!(ts.as_str(), "minute");

        let (mult, ts) = resolution_to_polygon_params("15m").unwrap();
        assert_eq!(mult, 15);
        assert_eq!(ts.as_str(), "minute");

        let (mult, ts) = resolution_to_polygon_params("1h").unwrap();
        assert_eq!(mult, 1);
        assert_eq!(ts.as_str(), "hour");

        let (mult, ts) = resolution_to_polygon_params("1d").unwrap();
        assert_eq!(mult, 1);
        assert_eq!(ts.as_str(), "day");

        assert!(resolution_to_polygon_params("invalid").is_err());
    }
}
