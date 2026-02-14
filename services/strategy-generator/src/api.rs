use crate::backtest::Backtester;
use axum::{
    extract::{Json, Path, State},
    routing::{get, post},
    Router,
};
use backtest_engine::config::FactorConfig;
use serde::Deserialize;
use serde_json::{json, Value};
use sqlx::postgres::PgPool;
use std::collections::HashMap;

#[derive(Clone)]
pub struct ExchangeApiConfig {
    pub factor_config: FactorConfig,
    pub exchange: String,
    pub resolution: String,
}

#[derive(Clone)]
pub struct ApiState {
    pub pool: PgPool,
    pub exchanges: HashMap<String, ExchangeApiConfig>,
}

#[derive(Deserialize)]
pub struct BacktestRequest {
    pub genome: Vec<i32>,
    pub token_address: String,
    pub days: Option<i64>,
}

pub async fn start_api_server(
    pool: PgPool,
    exchanges: HashMap<String, ExchangeApiConfig>,
    port: u16,
) {
    let state = ApiState { pool, exchanges };

    let app = Router::new()
        .route("/exchanges", get(list_exchanges))
        .route("/:exchange/config/factors", get(get_factor_config))
        .route("/:exchange/backtest", post(handle_backtest))
        .with_state(state);

    let addr = std::net::SocketAddr::from(([0, 0, 0, 0], port));
    tracing::info!("Generator API listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn list_exchanges(State(state): State<ApiState>) -> Json<Value> {
    let exchanges: Vec<Value> = state
        .exchanges
        .iter()
        .map(|(key, cfg)| {
            json!({
                "key": key,
                "exchange": cfg.exchange,
                "resolution": cfg.resolution,
                "factor_count": cfg.factor_config.active_factors.len(),
            })
        })
        .collect();
    Json(json!({ "exchanges": exchanges }))
}

async fn get_factor_config(
    State(state): State<ApiState>,
    Path(exchange): Path<String>,
) -> Json<Value> {
    let key = exchange.to_lowercase();
    match state.exchanges.get(&key) {
        Some(cfg) => Json(json!(cfg.factor_config)),
        None => Json(json!({"error": format!("Unknown exchange: {}", exchange)})),
    }
}

async fn handle_backtest(
    State(state): State<ApiState>,
    Path(exchange): Path<String>,
    Json(payload): Json<BacktestRequest>,
) -> Json<Value> {
    let key = exchange.to_lowercase();
    let cfg = match state.exchanges.get(&key) {
        Some(c) => c,
        None => return Json(json!({"error": format!("Unknown exchange: {}", exchange)})),
    };

    let mut backtester = Backtester::new(
        state.pool.clone(),
        cfg.factor_config.clone(),
        cfg.exchange.clone(),
        cfg.resolution.clone(),
    );

    let days = payload.days.unwrap_or(7);

    let result = if payload.token_address == "ALL" || payload.token_address == "UNIVERSAL" {
        backtester
            .run_portfolio_simulation(&payload.genome, days)
            .await
    } else {
        backtester
            .run_detailed_simulation(&payload.genome, &payload.token_address, days)
            .await
    };

    match result {
        Ok(result) => {
            let metrics = result.get("metrics").cloned().unwrap_or(json!({}));
            let equity = result.get("equity_curve").cloned().unwrap_or(json!([]));

            let _ = sqlx::query(
                r#"INSERT INTO backtest_results
                   (genome, token_address, metrics, equity_curve, pnl_percent, win_rate, total_trades, sharpe_ratio, max_drawdown)
                   VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)"#,
            )
            .bind(&payload.genome)
            .bind(&payload.token_address)
            .bind(&metrics)
            .bind(&equity)
            .bind(metrics.get("total_return").and_then(|v| v.as_f64()).unwrap_or(0.0))
            .bind(metrics.get("win_rate").and_then(|v| v.as_f64()).unwrap_or(0.0))
            .bind(metrics.get("total_trades").and_then(|v| v.as_i64()).unwrap_or(0) as i32)
            .bind(metrics.get("sharpe_ratio").and_then(|v| v.as_f64()).unwrap_or(0.0))
            .bind(metrics.get("max_drawdown").and_then(|v| v.as_f64()).unwrap_or(0.0))
            .execute(&state.pool)
            .await;

            Json(result)
        }
        Err(e) => {
            tracing::error!("Backtest failed: {}", e);
            Json(json!({"error": e.to_string()}))
        }
    }
}
