use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use rust_decimal::prelude::ToPrimitive;
use serde::{Deserialize, Serialize};

use crate::repository::PredictionRepository;
use crate::server::routes::AppState;

#[derive(Deserialize)]
pub struct ListMarketsQuery {
    #[serde(default = "default_true")]
    pub active: bool,
    pub category: Option<String>,
    #[serde(default = "default_limit")]
    pub limit: i64,
    #[serde(default)]
    pub offset: i64,
}

fn default_true() -> bool {
    true
}

fn default_limit() -> i64 {
    50
}

#[derive(Serialize)]
pub struct MarketListResponse {
    pub markets: Vec<MarketSummary>,
    pub count: usize,
}

#[derive(Serialize)]
pub struct MarketSummary {
    pub id: String,
    pub title: String,
    pub category: Option<String>,
    pub active: bool,
    pub end_date: Option<String>,
    pub outcomes: Vec<OutcomeSummary>,
    pub volume: Option<String>,
    pub updated_at: String,
}

#[derive(Serialize)]
pub struct OutcomeSummary {
    pub outcome: String,
    pub price: f64,
    pub probability_pct: f64,
    pub volume: Option<f64>,
}

#[derive(Serialize)]
pub struct MarketDetailResponse {
    pub id: String,
    pub source: String,
    pub title: String,
    pub description: Option<String>,
    pub category: Option<String>,
    pub active: bool,
    pub end_date: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub outcomes: Vec<OutcomeSummary>,
    pub metadata: serde_json::Value,
}

#[derive(Deserialize)]
pub struct HistoryQuery {
    #[serde(default = "default_history_limit")]
    pub limit: i64,
}

fn default_history_limit() -> i64 {
    500
}

#[derive(Serialize)]
pub struct OutcomeHistoryResponse {
    pub market_id: String,
    pub history: Vec<OutcomeHistoryPoint>,
    pub count: usize,
}

#[derive(Serialize)]
pub struct OutcomeHistoryPoint {
    pub outcome: String,
    pub price: f64,
    pub volume: Option<f64>,
    pub timestamp: String,
}

/// GET /api/v1/prediction/markets
pub async fn list_prediction_markets(
    State(state): State<AppState>,
    Query(params): Query<ListMarketsQuery>,
) -> Response {
    let limit = params.limit.min(200);
    let offset = params.offset.max(0);

    match state
        .postgres
        .prediction
        .list_markets(params.active, params.category.as_deref(), limit, offset)
        .await
    {
        Ok(markets) => {
            let summaries: Vec<MarketSummary> = markets
                .iter()
                .map(|m| {
                    let volume = m
                        .metadata
                        .get("volume")
                        .and_then(|v| v.as_str())
                        .map(String::from);
                    MarketSummary {
                        id: m.id.clone(),
                        title: m.title.clone(),
                        category: m.category.clone(),
                        active: m.active,
                        end_date: m.end_date.map(|d| d.to_rfc3339()),
                        outcomes: m
                            .outcomes
                            .iter()
                            .map(|o| OutcomeSummary {
                                outcome: o.outcome.clone(),
                                price: o.price.to_f64().unwrap_or(0.0),
                                probability_pct: o.price.to_f64().unwrap_or(0.0) * 100.0,
                                volume: o.volume.and_then(|v| v.to_f64()),
                            })
                            .collect(),
                        volume,
                        updated_at: m.updated_at.to_rfc3339(),
                    }
                })
                .collect();

            (
                StatusCode::OK,
                Json(MarketListResponse {
                    count: summaries.len(),
                    markets: summaries,
                }),
            )
                .into_response()
        }
        Err(e) => {
            tracing::error!("Failed to list prediction markets: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": e.to_string()})),
            )
                .into_response()
        }
    }
}

/// GET /api/v1/prediction/markets/:id
pub async fn get_prediction_market(
    State(state): State<AppState>,
    Path(market_id): Path<String>,
) -> Response {
    match state.postgres.prediction.get_market(&market_id).await {
        Ok(Some(market)) => {
            let response = MarketDetailResponse {
                id: market.id,
                source: market.source,
                title: market.title,
                description: market.description,
                category: market.category,
                active: market.active,
                end_date: market.end_date.map(|d| d.to_rfc3339()),
                created_at: market.created_at.to_rfc3339(),
                updated_at: market.updated_at.to_rfc3339(),
                outcomes: market
                    .outcomes
                    .iter()
                    .map(|o| OutcomeSummary {
                        outcome: o.outcome.clone(),
                        price: o.price.to_f64().unwrap_or(0.0),
                        probability_pct: o.price.to_f64().unwrap_or(0.0) * 100.0,
                        volume: o.volume.and_then(|v| v.to_f64()),
                    })
                    .collect(),
                metadata: market.metadata,
            };

            (StatusCode::OK, Json(response)).into_response()
        }
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({
                "error": "Market not found",
                "market_id": market_id
            })),
        )
            .into_response(),
        Err(e) => {
            tracing::error!("Failed to get prediction market {}: {}", market_id, e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": e.to_string()})),
            )
                .into_response()
        }
    }
}

/// GET /api/v1/prediction/markets/:id/history
pub async fn get_prediction_market_history(
    State(state): State<AppState>,
    Path(market_id): Path<String>,
    Query(params): Query<HistoryQuery>,
) -> Response {
    let limit = params.limit.min(5000);

    match state
        .postgres
        .prediction
        .get_outcome_history(&market_id, limit)
        .await
    {
        Ok(outcomes) => {
            let history: Vec<OutcomeHistoryPoint> = outcomes
                .iter()
                .map(|o| OutcomeHistoryPoint {
                    outcome: o.outcome.clone(),
                    price: o.price.to_f64().unwrap_or(0.0),
                    volume: o.volume.and_then(|v| v.to_f64()),
                    timestamp: o.timestamp.to_rfc3339(),
                })
                .collect();

            (
                StatusCode::OK,
                Json(OutcomeHistoryResponse {
                    market_id,
                    count: history.len(),
                    history,
                }),
            )
                .into_response()
        }
        Err(e) => {
            tracing::error!(
                "Failed to get prediction market history {}: {}",
                market_id,
                e
            );
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": e.to_string()})),
            )
                .into_response()
        }
    }
}
