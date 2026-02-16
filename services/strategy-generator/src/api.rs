use crate::backtest::Backtester;
use axum::{
    extract::{Json, Path, Query, State},
    routing::{get, post},
    Router,
};
use backtest_engine::config::FactorConfig;
use serde::Deserialize;
use serde_json::{json, Value};
use sqlx::postgres::PgPool;
use sqlx::Row;
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

#[derive(Deserialize)]
pub struct GenerationsQuery {
    pub limit: Option<i64>,
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
        .route("/:exchange/symbols", get(list_symbols))
        .route("/:exchange/overview", get(get_overview))
        .route("/:exchange/generations", get(list_generations))
        .route("/:exchange/generations/:gen", get(get_generation))
        .route(
            "/:exchange/:symbol/generations",
            get(list_symbol_generations),
        )
        .route(
            "/:exchange/:symbol/generations/:gen",
            get(get_symbol_generation),
        )
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

async fn list_generations(
    State(state): State<ApiState>,
    Path(exchange): Path<String>,
    Query(params): Query<GenerationsQuery>,
) -> Json<Value> {
    let key = exchange.to_lowercase();
    let exchange_name = match state.exchanges.get(&key) {
        Some(cfg) => cfg.exchange.clone(),
        None => return Json(json!({"error": format!("Unknown exchange: {}", exchange)})),
    };

    let limit = params.limit.unwrap_or(100).min(500);

    let rows = sqlx::query(
        r#"SELECT
            sg.generation,
            sg.fitness,
            sg.best_genome,
            sg.strategy_id,
            sg.timestamp,
            sg.metadata,
            br.pnl_percent,
            br.sharpe_ratio,
            br.max_drawdown,
            br.win_rate,
            br.total_trades
        FROM strategy_generations sg
        LEFT JOIN backtest_results br ON br.strategy_id = sg.strategy_id
        WHERE sg.exchange = $1
        ORDER BY sg.generation DESC
        LIMIT $2"#,
    )
    .bind(&exchange_name)
    .bind(limit)
    .fetch_all(&state.pool)
    .await;

    match rows {
        Ok(rows) => {
            let generations: Vec<Value> = rows
                .iter()
                .map(|row| row_to_generation_json(row, false))
                .collect();

            Json(json!({
                "exchange": exchange_name,
                "generations": generations,
            }))
        }
        Err(e) => {
            tracing::error!("Failed to query generations: {}", e);
            Json(json!({"error": "Failed to fetch generations"}))
        }
    }
}

async fn get_generation(
    State(state): State<ApiState>,
    Path((exchange, gen)): Path<(String, i32)>,
) -> Json<Value> {
    let key = exchange.to_lowercase();
    let exchange_name = match state.exchanges.get(&key) {
        Some(cfg) => cfg.exchange.clone(),
        None => return Json(json!({"error": format!("Unknown exchange: {}", exchange)})),
    };

    let row = sqlx::query(
        r#"SELECT
            sg.generation,
            sg.fitness,
            sg.best_genome,
            sg.strategy_id,
            sg.timestamp,
            sg.metadata,
            br.pnl_percent,
            br.sharpe_ratio,
            br.max_drawdown,
            br.win_rate,
            br.total_trades,
            br.equity_curve
        FROM strategy_generations sg
        LEFT JOIN backtest_results br ON br.strategy_id = sg.strategy_id
        WHERE sg.exchange = $1 AND sg.generation = $2"#,
    )
    .bind(&exchange_name)
    .bind(gen)
    .fetch_optional(&state.pool)
    .await;

    match row {
        Ok(Some(row)) => Json(row_to_generation_json(&row, true)),
        Ok(None) => Json(json!({"error": "Generation not found"})),
        Err(e) => {
            tracing::error!("Failed to query generation {}: {}", gen, e);
            Json(json!({"error": "Failed to fetch generation"}))
        }
    }
}

/// List symbols being evolved for an exchange.
async fn list_symbols(State(state): State<ApiState>, Path(exchange): Path<String>) -> Json<Value> {
    let key = exchange.to_lowercase();
    let exchange_name = match state.exchanges.get(&key) {
        Some(cfg) => cfg.exchange.clone(),
        None => return Json(json!({"error": format!("Unknown exchange: {}", exchange)})),
    };

    let rows = sqlx::query(
        "SELECT DISTINCT symbol FROM strategy_generations WHERE exchange = $1 ORDER BY symbol",
    )
    .bind(&exchange_name)
    .fetch_all(&state.pool)
    .await;

    match rows {
        Ok(rows) => {
            let symbols: Vec<String> = rows.iter().map(|r| r.get("symbol")).collect();
            Json(json!({ "exchange": exchange_name, "symbols": symbols }))
        }
        Err(e) => {
            tracing::error!("Failed to query symbols: {}", e);
            Json(json!({"error": "Failed to fetch symbols"}))
        }
    }
}

/// Overview of all symbols' evolution status for an exchange.
async fn get_overview(State(state): State<ApiState>, Path(exchange): Path<String>) -> Json<Value> {
    let key = exchange.to_lowercase();
    let exchange_name = match state.exchanges.get(&key) {
        Some(cfg) => cfg.exchange.clone(),
        None => return Json(json!({"error": format!("Unknown exchange: {}", exchange)})),
    };

    let rows = sqlx::query(
        r#"SELECT
            sg.symbol,
            sg.generation AS latest_gen,
            sg.fitness AS best_fitness,
            sg.timestamp AS last_updated,
            sg.metadata,
            br.pnl_percent,
            br.sharpe_ratio,
            br.max_drawdown,
            br.win_rate
        FROM strategy_generations sg
        LEFT JOIN backtest_results br ON br.strategy_id = sg.strategy_id
        WHERE sg.exchange = $1
        AND sg.generation = (
            SELECT MAX(sg2.generation) FROM strategy_generations sg2
            WHERE sg2.exchange = sg.exchange AND sg2.symbol = sg.symbol
        )
        ORDER BY sg.symbol"#,
    )
    .bind(&exchange_name)
    .fetch_all(&state.pool)
    .await;

    match rows {
        Ok(rows) => {
            let symbols: Vec<Value> = rows
                .iter()
                .map(|row| {
                    let symbol: String = row.get("symbol");
                    let latest_gen: i32 = row.get("latest_gen");
                    let best_fitness: Option<f64> = row.get("best_fitness");
                    let last_updated: Option<chrono::DateTime<chrono::Utc>> =
                        row.get("last_updated");
                    let metadata: Option<Value> = row.get("metadata");
                    let pnl_percent: Option<f64> = row.get("pnl_percent");
                    let sharpe_ratio: Option<f64> = row.get("sharpe_ratio");
                    let max_drawdown: Option<f64> = row.get("max_drawdown");
                    let win_rate: Option<f64> = row.get("win_rate");

                    let oos_ic = metadata
                        .as_ref()
                        .and_then(|m| m.get("oos_ic"))
                        .and_then(|v| v.as_f64());

                    json!({
                        "symbol": symbol,
                        "latest_gen": latest_gen,
                        "best_fitness": best_fitness,
                        "best_oos_ic": oos_ic,
                        "best_pnl": pnl_percent,
                        "sharpe_ratio": sharpe_ratio,
                        "max_drawdown": max_drawdown,
                        "win_rate": win_rate,
                        "last_updated": last_updated.map(|t| t.to_rfc3339()),
                    })
                })
                .collect();

            Json(json!({ "exchange": exchange_name, "symbols": symbols }))
        }
        Err(e) => {
            tracing::error!("Failed to query overview: {}", e);
            Json(json!({"error": "Failed to fetch overview"}))
        }
    }
}

/// Per-symbol generation history.
async fn list_symbol_generations(
    State(state): State<ApiState>,
    Path((exchange, symbol)): Path<(String, String)>,
    Query(params): Query<GenerationsQuery>,
) -> Json<Value> {
    let key = exchange.to_lowercase();
    let exchange_name = match state.exchanges.get(&key) {
        Some(cfg) => cfg.exchange.clone(),
        None => return Json(json!({"error": format!("Unknown exchange: {}", exchange)})),
    };

    let limit = params.limit.unwrap_or(100).min(500);

    let rows = sqlx::query(
        r#"SELECT
            sg.generation,
            sg.fitness,
            sg.best_genome,
            sg.strategy_id,
            sg.timestamp,
            sg.metadata,
            br.pnl_percent,
            br.sharpe_ratio,
            br.max_drawdown,
            br.win_rate,
            br.total_trades
        FROM strategy_generations sg
        LEFT JOIN backtest_results br ON br.strategy_id = sg.strategy_id
        WHERE sg.exchange = $1 AND sg.symbol = $2
        ORDER BY sg.generation DESC
        LIMIT $3"#,
    )
    .bind(&exchange_name)
    .bind(&symbol)
    .bind(limit)
    .fetch_all(&state.pool)
    .await;

    match rows {
        Ok(rows) => {
            let generations: Vec<Value> = rows
                .iter()
                .map(|row| row_to_generation_json(row, false))
                .collect();

            Json(json!({
                "exchange": exchange_name,
                "symbol": symbol,
                "generations": generations,
            }))
        }
        Err(e) => {
            tracing::error!("Failed to query generations for {}: {}", symbol, e);
            Json(json!({"error": "Failed to fetch generations"}))
        }
    }
}

/// Per-symbol generation detail with equity curve.
async fn get_symbol_generation(
    State(state): State<ApiState>,
    Path((exchange, symbol, gen)): Path<(String, String, i32)>,
) -> Json<Value> {
    let key = exchange.to_lowercase();
    let exchange_name = match state.exchanges.get(&key) {
        Some(cfg) => cfg.exchange.clone(),
        None => return Json(json!({"error": format!("Unknown exchange: {}", exchange)})),
    };

    let row = sqlx::query(
        r#"SELECT
            sg.generation,
            sg.fitness,
            sg.best_genome,
            sg.strategy_id,
            sg.timestamp,
            sg.metadata,
            br.pnl_percent,
            br.sharpe_ratio,
            br.max_drawdown,
            br.win_rate,
            br.total_trades,
            br.equity_curve
        FROM strategy_generations sg
        LEFT JOIN backtest_results br ON br.strategy_id = sg.strategy_id
        WHERE sg.exchange = $1 AND sg.symbol = $2 AND sg.generation = $3"#,
    )
    .bind(&exchange_name)
    .bind(&symbol)
    .bind(gen)
    .fetch_optional(&state.pool)
    .await;

    match row {
        Ok(Some(row)) => Json(row_to_generation_json(&row, true)),
        Ok(None) => Json(json!({"error": "Generation not found"})),
        Err(e) => {
            tracing::error!("Failed to query generation {} for {}: {}", gen, symbol, e);
            Json(json!({"error": "Failed to fetch generation"}))
        }
    }
}

/// Helper: Convert a DB row to generation JSON.
fn row_to_generation_json(row: &sqlx::postgres::PgRow, include_equity: bool) -> Value {
    let generation: i32 = row.get("generation");
    let fitness: Option<f64> = row.get("fitness");
    let best_genome: Option<Vec<i32>> = row.get("best_genome");
    let strategy_id: Option<String> = row.get("strategy_id");
    let timestamp: Option<chrono::DateTime<chrono::Utc>> = row.get("timestamp");
    let metadata: Option<Value> = row.get("metadata");
    let pnl_percent: Option<f64> = row.get("pnl_percent");
    let sharpe_ratio: Option<f64> = row.get("sharpe_ratio");
    let max_drawdown: Option<f64> = row.get("max_drawdown");
    let win_rate: Option<f64> = row.get("win_rate");
    let total_trades: Option<i32> = row.get("total_trades");

    let oos_ic = metadata
        .as_ref()
        .and_then(|m| m.get("oos_ic"))
        .and_then(|v| v.as_f64());

    let backtest = if pnl_percent.is_some() {
        let mut bt = json!({
            "pnl_percent": pnl_percent,
            "sharpe_ratio": sharpe_ratio,
            "max_drawdown": max_drawdown,
            "win_rate": win_rate,
            "total_trades": total_trades,
        });
        if include_equity {
            let equity_curve: Option<Value> = row.get("equity_curve");
            bt["equity_curve"] = equity_curve.unwrap_or(json!(null));
        }
        Some(bt)
    } else {
        None
    };

    json!({
        "generation": generation,
        "fitness": fitness,
        "best_genome": best_genome,
        "strategy_id": strategy_id,
        "timestamp": timestamp.map(|t| t.to_rfc3339()),
        "oos_ic": oos_ic,
        "backtest": backtest,
    })
}
