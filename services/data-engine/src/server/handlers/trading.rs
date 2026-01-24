use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use rust_decimal::Decimal;
use serde_json::json;
use tracing::{error, info};

use crate::models::{trading::OrderRequest, Order};
use crate::repository::TradingRepository;
use crate::server::AppState;
use chrono::Utc;
use uuid::Uuid;

/// Place a new order
pub async fn place_order(
    State(state): State<AppState>,
    Json(payload): Json<OrderRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let trader = state.ibkr_trader.as_ref().ok_or((
        StatusCode::SERVICE_UNAVAILABLE,
        "IBKR Trading not enabled".to_string(),
    ))?;

    // 1. Place order via IBKR API
    let ib_order_id = trader.place_order(payload.clone()).await.map_err(|e| {
        error!("Failed to place IBKR order: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to place order: {}", e),
        )
    })?;

    // 2. Persist order in Postgres
    let quantity = Decimal::from_f64_retain(payload.quantity).unwrap_or_default();
    let status = "Pending".to_string();
    let order_id = Uuid::new_v4();

    let order = Order {
        id: order_id,
        ib_order_id,
        symbol: payload.symbol.clone(),
        action: payload.action.clone(),
        quantity,
        order_type: payload.order_type.clone(),
        status: status.clone(),
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };

    let saved_id = state
        .postgres
        .trading
        .insert_order(&order)
        .await
        .map_err(|e| {
            error!("Failed to persist order: {}", e);
            // Note: Order was placed in IBKR but DB failed. This is a consistency issue.
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Order placed but storage failed".to_string(),
            )
        })?;

    info!(
        "Order placed successfully: {} (IB: {})",
        saved_id, ib_order_id
    );

    Ok(Json(json!({
        "order_id": saved_id,
        "ib_order_id": ib_order_id,
        "status": status,
        "symbol": payload.symbol,
        "action": payload.action,
        "quantity": payload.quantity
    })))
}

/// Cancel an existing order
pub async fn cancel_order(
    State(state): State<AppState>,
    Path(ib_order_id): Path<i32>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let trader = state.ibkr_trader.as_ref().ok_or((
        StatusCode::SERVICE_UNAVAILABLE,
        "IBKR Trading not enabled".to_string(),
    ))?;

    trader.cancel_order(ib_order_id).await.map_err(|e| {
        error!("Failed to cancel IBKR order: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to cancel order: {}", e),
        )
    })?;

    // TODO: Update order status in DB (requires implementing update_order in PostgresWriter)

    info!("Order cancelled: {}", ib_order_id);

    Ok(Json(json!({
        "ib_order_id": ib_order_id,
        "status": "Cancelled"
    })))
}

/// Get current positions
pub async fn get_positions(
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let trader = state.ibkr_trader.as_ref().ok_or((
        StatusCode::SERVICE_UNAVAILABLE,
        "IBKR Trading not enabled".to_string(),
    ))?;

    let positions = trader.get_positions().await.map_err(|e| {
        error!("Failed to get positions: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to get positions: {}", e),
        )
    })?;

    Ok(Json(json!(positions)))
}

/// Get account summary
pub async fn get_account_summary(
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let trader = state.ibkr_trader.as_ref().ok_or((
        StatusCode::SERVICE_UNAVAILABLE,
        "IBKR Trading not enabled".to_string(),
    ))?;

    let summary = trader.get_account_summary().await.map_err(|e| {
        error!("Failed to get account summary: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to get account summary: {}", e),
        )
    })?;

    Ok(Json(json!(summary)))
}
