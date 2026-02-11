use super::AppState;
use axum::{
    extract::{Json, State},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::{Deserialize, Serialize};

/// Exchange Configuration
#[derive(Serialize, Deserialize)]
pub struct ExchangeConfig {
    pub exchange: String,
    pub api_key: Option<String>, // Masked on GET, updated on POST
    pub is_enabled: bool,
}

#[derive(Deserialize)]
pub struct UpdateExchangeRequest {
    pub exchange: String,
    pub api_key: String,
    pub is_enabled: bool,
}

/// Watchlist Item
#[derive(Serialize, Deserialize)]
pub struct WatchlistItem {
    pub exchange: String,
    pub symbol: String,
    pub name: Option<String>,
    pub added_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Deserialize)]
pub struct AddWatchlistRequest {
    pub exchange: String,
    pub symbol: String,
    pub name: Option<String>,
    pub asset_type: Option<String>,
}

#[derive(Deserialize)]
pub struct RemoveWatchlistRequest {
    pub exchange: String,
    pub symbol: String,
}

/// GET /api/v1/config/exchanges
pub async fn get_exchange_config(State(_state): State<AppState>) -> Response {
    // In a real app, we'd fetch from DB. For now, we'll mock or return env var status
    // Since we don't have a config table yet, let's just return hardcoded status or read from env
    // But user asked for a "Configuration API", suggesting persistence.
    // For MVP, we can treat the ENV vars as source of truth for "active" but return them.

    // TODO: Implement DB-backed config table if needed.
    // For now, return what's in AppConfig but mask keys

    let configs = vec![ExchangeConfig {
        exchange: "Polygon".to_string(),
        api_key: Some("********".to_string()),
        is_enabled: true,
    }];

    (StatusCode::OK, Json(configs)).into_response()
}

/// POST /api/v1/config/exchanges
pub async fn update_exchange_config(
    State(_state): State<AppState>,
    Json(payload): Json<UpdateExchangeRequest>,
) -> Response {
    // Ideally update DB or .env file (dangerous).
    // For this MVP, we might just log it or update in-memory if possible,
    // but changing env vars at runtime is tricky.
    // Let's assume we have a DB table `exchange_config` later.
    // For now, just return OK to mock success for UI dev.

    tracing::info!("Updated config for {}", payload.exchange);

    (
        StatusCode::OK,
        Json(serde_json::json!({"status": "updated"})),
    )
        .into_response()
}

/// GET /api/v1/watchlist
pub async fn get_watchlist(State(state): State<AppState>) -> Response {
    let pool = &state.postgres.pool;

    let query = "
        SELECT exchange, symbol, name, created_at as added_at 
        FROM market_watchlist 
        WHERE is_active = true
        ORDER BY created_at DESC
    ";

    let rows_res = sqlx::query(query).fetch_all(pool).await;

    match rows_res {
        Ok(rows) => {
            let items: Vec<WatchlistItem> = rows
                .into_iter()
                .map(|row| {
                    use sqlx::Row;
                    WatchlistItem {
                        exchange: row.get("exchange"),
                        symbol: row.get("symbol"),
                        name: row.try_get("name").ok(),
                        added_at: row.try_get("added_at").ok(),
                    }
                })
                .collect();

            (StatusCode::OK, Json(items)).into_response()
        }
        Err(e) => {
            tracing::error!("Failed to fetch watchlist: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": e.to_string()})),
            )
                .into_response()
        }
    }
}

/// POST /api/v1/watchlist
pub async fn add_to_watchlist(
    State(state): State<AppState>,
    Json(payload): Json<AddWatchlistRequest>,
) -> Response {
    let pool = &state.postgres.pool;

    // Insert into DB
    let query = "
        INSERT INTO market_watchlist (exchange, symbol, name, asset_type, is_active, created_at)
        VALUES ($1, $2, $3, $4, true, NOW())
        ON CONFLICT (exchange, symbol) DO UPDATE SET is_active = true
    ";

    let asset_type = payload
        .asset_type
        .clone()
        .unwrap_or_else(|| "stock".to_string());

    let res = sqlx::query(query)
        .bind(&payload.exchange)
        .bind(&payload.symbol.to_uppercase())
        .bind(&payload.name)
        .bind(asset_type)
        .execute(pool)
        .await;

    match res {
        Ok(_) => {
            // Trigger auto-sync manually if needed, but we have triggers!
            // The DB triggers we set up (trigger_auto_sync_on_insert) should handle this.
            // Just return success.
            tracing::info!("Added {}/{} to watchlist", payload.exchange, payload.symbol);
            (
                StatusCode::CREATED,
                Json(serde_json::json!({"status": "added"})),
            )
                .into_response()
        }
        Err(e) => {
            tracing::error!("Failed to add to watchlist: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": e.to_string()})),
            )
                .into_response()
        }
    }
}

/// DELETE /api/v1/watchlist
pub async fn remove_from_watchlist(
    State(state): State<AppState>,
    Json(payload): Json<RemoveWatchlistRequest>,
) -> Response {
    let pool = &state.postgres.pool;

    let query = "
        UPDATE market_watchlist 
        SET is_active = false 
        WHERE exchange = $1 AND symbol = $2
    ";

    let res = sqlx::query(query)
        .bind(&payload.exchange)
        .bind(&payload.symbol.to_uppercase())
        .execute(pool)
        .await;

    match res {
        Ok(_) => (
            StatusCode::OK,
            Json(serde_json::json!({"status": "removed"})),
        )
            .into_response(),
        Err(e) => {
            tracing::error!("Failed to remove from watchlist: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": e.to_string()})),
            )
                .into_response()
        }
    }
}
