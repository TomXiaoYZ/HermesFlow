use chrono::{DateTime, TimeZone, Utc};
use rust_decimal::Decimal;
use serde::Deserialize;
use std::sync::Arc;
use tokio::sync::broadcast::Receiver;
use tracing::{error, info, warn};

use crate::config::PolymarketConfig;
use crate::error::Result;
use crate::models::{MarketOutcome, PredictionMarket};
use crate::repository::postgres::PostgresPredictionRepository;
use crate::repository::PredictionRepository;

/// A single market from the Gamma API
#[derive(Debug, Deserialize)]
struct GammaMarket {
    /// Condition ID (unique market identifier)
    #[serde(rename = "conditionId", alias = "condition_id")]
    condition_id: Option<String>,
    /// Question ID (alternative identifier)
    #[serde(rename = "questionId", alias = "question_id")]
    question_id: Option<String>,
    /// Market question/title
    question: Option<String>,
    /// Market description
    description: Option<String>,
    /// Category/tag
    #[serde(rename = "groupItemTitle")]
    group_item_title: Option<String>,
    /// Market end date
    #[serde(rename = "endDate", alias = "end_date_iso")]
    end_date: Option<String>,
    /// Whether the market is active
    active: Option<bool>,
    /// Whether the market is closed
    closed: Option<bool>,
    /// Market outcomes (e.g., ["Yes", "No"])
    outcomes: Option<String>,
    /// Outcome prices as JSON string (e.g., "[\"0.65\", \"0.35\"]")
    #[serde(rename = "outcomePrices", alias = "outcome_prices")]
    outcome_prices: Option<String>,
    /// Total volume traded
    volume: Option<String>,
    /// Liquidity available
    liquidity: Option<String>,
    /// Market creation timestamp
    #[serde(rename = "createdAt")]
    created_at: Option<String>,
    /// Market update timestamp
    #[serde(rename = "updatedAt")]
    updated_at: Option<String>,
    /// Tags for categorization
    tags: Option<Vec<GammaTag>>,
    /// Spread
    spread: Option<f64>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct GammaTag {
    label: Option<String>,
    slug: Option<String>,
}

pub struct PolymarketCollector {
    config: PolymarketConfig,
    http_client: reqwest::Client,
    prediction_repo: Arc<PostgresPredictionRepository>,
}

impl PolymarketCollector {
    pub fn new(
        config: PolymarketConfig,
        prediction_repo: Arc<PostgresPredictionRepository>,
    ) -> Self {
        let http_client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .expect("Failed to create HTTP client");

        Self {
            config,
            http_client,
            prediction_repo,
        }
    }

    pub async fn start(&self, mut shutdown: Receiver<()>) -> Result<()> {
        info!(
            "Polymarket collector starting (poll_interval={}s, api={})",
            self.config.poll_interval_secs, self.config.api_base_url
        );

        // Initial full discovery
        if let Err(e) = self.discover_markets().await {
            error!("Polymarket initial discovery failed: {}", e);
        }

        let poll_interval =
            tokio::time::Duration::from_secs(self.config.poll_interval_secs);
        let mut interval = tokio::time::interval(poll_interval);

        loop {
            tokio::select! {
                _ = interval.tick() => {
                    if let Err(e) = self.refresh_prices().await {
                        error!("Polymarket price refresh failed: {}", e);
                    }
                }
                _ = shutdown.recv() => {
                    info!("Polymarket collector shutting down");
                    break;
                }
            }
        }

        Ok(())
    }

    /// Full market discovery — fetches all active markets with pagination
    pub async fn discover_markets(&self) -> std::result::Result<usize, String> {
        info!("Polymarket: Starting full market discovery...");
        let mut offset = 0;
        let limit = 100;
        let mut total_upserted = 0;

        loop {
            let url = format!(
                "{}/markets?active=true&closed=false&limit={}&offset={}",
                self.config.api_base_url, limit, offset
            );

            let markets = match self.fetch_markets_page(&url).await {
                Ok(m) => m,
                Err(e) => {
                    error!(
                        "Polymarket: Failed to fetch page at offset {}: {}",
                        offset, e
                    );
                    break;
                }
            };

            let page_count = markets.len();
            if page_count == 0 {
                break;
            }

            for gamma_market in &markets {
                match self.upsert_gamma_market(gamma_market).await {
                    Ok(_) => total_upserted += 1,
                    Err(e) => {
                        let id = gamma_market
                            .condition_id
                            .as_deref()
                            .unwrap_or("unknown");
                        warn!("Polymarket: Failed to upsert market {}: {}", id, e);
                    }
                }
            }

            info!(
                "Polymarket: Processed page offset={}, markets={}",
                offset, page_count
            );

            if page_count < limit {
                break;
            }
            offset += limit;

            // Small delay between pages to be polite
            tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;
        }

        info!(
            "Polymarket: Discovery complete. Upserted {} markets.",
            total_upserted
        );
        Ok(total_upserted)
    }

    /// Refresh prices for active markets (lighter operation)
    pub async fn refresh_prices(&self) -> std::result::Result<(), String> {
        let url = format!(
            "{}/markets?active=true&closed=false&limit=100",
            self.config.api_base_url
        );

        let markets = self.fetch_markets_page(&url).await?;
        let mut updated = 0;

        for gamma_market in &markets {
            match self.upsert_gamma_market(gamma_market).await {
                Ok(_) => updated += 1,
                Err(e) => {
                    let id = gamma_market
                        .condition_id
                        .as_deref()
                        .unwrap_or("unknown");
                    warn!("Polymarket: Price refresh failed for {}: {}", id, e);
                }
            }
        }

        info!("Polymarket: Price refresh complete. Updated {} markets.", updated);
        Ok(())
    }

    /// Fetch a single page of markets from the Gamma API
    async fn fetch_markets_page(
        &self,
        url: &str,
    ) -> std::result::Result<Vec<GammaMarket>, String> {
        let response = self
            .http_client
            .get(url)
            .header("Accept", "application/json")
            .send()
            .await
            .map_err(|e| format!("HTTP request failed: {}", e))?;

        if !response.status().is_success() {
            return Err(format!(
                "Gamma API returned status {}",
                response.status()
            ));
        }

        let markets: Vec<GammaMarket> = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse Gamma API response: {}", e))?;

        Ok(markets)
    }

    /// Convert a Gamma API market to our domain model and persist it
    async fn upsert_gamma_market(
        &self,
        gamma: &GammaMarket,
    ) -> std::result::Result<(), String> {
        let market_id = gamma
            .condition_id
            .as_deref()
            .or(gamma.question_id.as_deref())
            .ok_or_else(|| "Market has no ID".to_string())?;

        let title = gamma
            .question
            .as_deref()
            .unwrap_or("Untitled Market")
            .to_string();

        let category = gamma
            .tags
            .as_ref()
            .and_then(|tags| tags.first())
            .and_then(|t| t.label.clone())
            .or_else(|| gamma.group_item_title.clone());

        let end_date = gamma
            .end_date
            .as_deref()
            .and_then(parse_datetime);

        let created_at = gamma
            .created_at
            .as_deref()
            .and_then(parse_datetime)
            .unwrap_or_else(Utc::now);

        let updated_at = gamma
            .updated_at
            .as_deref()
            .and_then(parse_datetime)
            .unwrap_or_else(Utc::now);

        let active = gamma.active.unwrap_or(true) && !gamma.closed.unwrap_or(false);

        // Parse outcomes and prices
        let outcomes = parse_outcomes_and_prices(
            gamma.outcomes.as_deref(),
            gamma.outcome_prices.as_deref(),
            gamma.volume.as_deref(),
        );

        let metadata = serde_json::json!({
            "volume": gamma.volume,
            "liquidity": gamma.liquidity,
            "spread": gamma.spread,
            "tags": gamma.tags.as_ref().map(|tags|
                tags.iter().filter_map(|t| t.label.clone()).collect::<Vec<_>>()
            ),
        });

        let market = PredictionMarket {
            id: market_id.to_string(),
            source: "Polymarket".to_string(),
            title,
            description: gamma.description.clone(),
            category,
            end_date,
            created_at,
            updated_at,
            active,
            outcomes: outcomes.clone(),
            metadata,
        };

        // Upsert market
        self.prediction_repo
            .upsert_market(&market)
            .await
            .map_err(|e| format!("DB upsert market failed: {}", e))?;

        // Insert outcomes
        for outcome in &outcomes {
            self.prediction_repo
                .insert_outcome(market_id, outcome)
                .await
                .map_err(|e| format!("DB insert outcome failed: {}", e))?;
        }

        Ok(())
    }
}

/// Parse a datetime string in various formats
fn parse_datetime(s: &str) -> Option<DateTime<Utc>> {
    // Try RFC3339 first
    if let Ok(dt) = DateTime::parse_from_rfc3339(s) {
        return Some(dt.with_timezone(&Utc));
    }
    // Try ISO 8601 without timezone
    if let Ok(dt) = chrono::NaiveDateTime::parse_from_str(s, "%Y-%m-%dT%H:%M:%S%.f") {
        return Some(Utc.from_utc_datetime(&dt));
    }
    if let Ok(dt) = chrono::NaiveDateTime::parse_from_str(s, "%Y-%m-%dT%H:%M:%S") {
        return Some(Utc.from_utc_datetime(&dt));
    }
    // Try date only
    if let Ok(d) = chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d") {
        return d
            .and_hms_opt(0, 0, 0)
            .map(|dt| Utc.from_utc_datetime(&dt));
    }
    None
}

/// Parse Gamma API outcomes and outcome_prices into MarketOutcome vec
fn parse_outcomes_and_prices(
    outcomes_str: Option<&str>,
    prices_str: Option<&str>,
    volume_str: Option<&str>,
) -> Vec<MarketOutcome> {
    let outcome_names: Vec<String> = outcomes_str
        .and_then(|s| serde_json::from_str::<Vec<String>>(s).ok())
        .or_else(|| {
            // Try comma-separated format
            outcomes_str.map(|s| {
                s.split(',')
                    .map(|x| x.trim().trim_matches('"').to_string())
                    .collect()
            })
        })
        .unwrap_or_default();

    let prices: Vec<Decimal> = prices_str
        .and_then(|s| serde_json::from_str::<Vec<String>>(s).ok())
        .map(|strs| {
            strs.iter()
                .filter_map(|p| p.parse::<Decimal>().ok())
                .collect()
        })
        .unwrap_or_default();

    let total_volume: Option<Decimal> = volume_str
        .and_then(|s| s.parse::<Decimal>().ok());

    outcome_names
        .into_iter()
        .enumerate()
        .map(|(i, name)| {
            let price = prices.get(i).copied().unwrap_or(Decimal::ZERO);
            MarketOutcome {
                outcome: name,
                price,
                volume_24h: total_volume,
                timestamp: Utc::now(),
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_parse_datetime_rfc3339() {
        let dt = parse_datetime("2024-11-05T12:00:00Z");
        assert!(dt.is_some());
    }

    #[test]
    fn test_parse_datetime_iso_no_tz() {
        let dt = parse_datetime("2024-11-05T12:00:00.000");
        assert!(dt.is_some());
    }

    #[test]
    fn test_parse_datetime_date_only() {
        let dt = parse_datetime("2024-11-05");
        assert!(dt.is_some());
    }

    #[test]
    fn test_parse_outcomes_and_prices() {
        let outcomes =
            parse_outcomes_and_prices(Some(r#"["Yes","No"]"#), Some(r#"["0.65","0.35"]"#), Some("1000000"));

        assert_eq!(outcomes.len(), 2);
        assert_eq!(outcomes[0].outcome, "Yes");
        assert_eq!(outcomes[0].price, dec!(0.65));
        assert_eq!(outcomes[1].outcome, "No");
        assert_eq!(outcomes[1].price, dec!(0.35));
        assert_eq!(outcomes[0].volume_24h, Some(dec!(1000000)));
    }

    #[test]
    fn test_parse_outcomes_empty() {
        let outcomes = parse_outcomes_and_prices(None, None, None);
        assert!(outcomes.is_empty());
    }

    #[test]
    fn test_parse_outcomes_mismatch_count() {
        let outcomes = parse_outcomes_and_prices(
            Some(r#"["Yes","No","Maybe"]"#),
            Some(r#"["0.5","0.3"]"#),
            None,
        );
        assert_eq!(outcomes.len(), 3);
        assert_eq!(outcomes[2].price, Decimal::ZERO);
    }
}
