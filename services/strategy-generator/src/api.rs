use crate::backtest::Backtester;
use axum::{
    extract::{Json, State},
    routing::{get, post},
    Router,
};
use backtest_engine::config::FactorConfig;
use serde::Deserialize;
use serde_json::{json, Value};
use sqlx::postgres::PgPool;

#[derive(Clone)]
pub struct ApiState {
    pub pool: PgPool,
    pub factor_config: FactorConfig,
    pub exchange: String,
    pub resolution: String,
}

#[derive(Deserialize)]
pub struct BacktestRequest {
    pub genome: Vec<i32>,
    pub token_address: String,
    pub days: Option<i64>,
}

pub async fn start_api_server(
    pool: PgPool,
    factor_config: FactorConfig,
    exchange: String,
    resolution: String,
    port: u16,
) {
    let state = ApiState {
        pool,
        factor_config,
        exchange,
        resolution,
    };

    let app = Router::new()
        .route("/backtest", post(handle_backtest))
        .route("/config/factors", get(get_factor_config))
        .with_state(state);

    let addr = std::net::SocketAddr::from(([0, 0, 0, 0], port));
    tracing::info!("Generator API listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn get_factor_config(State(state): State<ApiState>) -> Json<FactorConfig> {
    Json(state.factor_config)
}

async fn handle_backtest(
    State(state): State<ApiState>,
    Json(payload): Json<BacktestRequest>,
) -> Json<Value> {
    let mut backtester = Backtester::new(
        state.pool.clone(),
        state.factor_config.clone(),
        state.exchange.clone(),
        state.resolution.clone(),
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
            let pnl = metrics
                .get("total_return")
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0);
            let win = metrics
                .get("win_rate")
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0);
            let sharpe = metrics
                .get("sharpe_ratio")
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0);
            let drawdown = metrics
                .get("max_drawdown")
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0);
            let trades = metrics
                .get("total_trades")
                .and_then(|v| v.as_i64())
                .unwrap_or(0);

            let _ = sqlx::query(
                r#"
                INSERT INTO backtest_results
                (genome, token_address, metrics, equity_curve, pnl_percent, win_rate, total_trades, sharpe_ratio, max_drawdown)
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
                "#
            )
            .bind(&payload.genome)
            .bind(&payload.token_address)
            .bind(&metrics)
            .bind(&equity)
            .bind(pnl)
            .bind(win)
            .bind(trades as i32)
            .bind(sharpe)
            .bind(drawdown)
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
