use crate::backtest::{Backtester, StrategyMode};
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
    pub mode: Option<String>,
}

#[derive(Deserialize)]
pub struct GenerationsQuery {
    pub limit: Option<i64>,
    pub mode: Option<String>,
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
        // P5: Ensemble endpoints
        .route("/:exchange/ensemble", get(get_ensemble))
        .route("/:exchange/ensemble/history", get(get_ensemble_history))
        .route("/:exchange/ensemble/equity", get(get_ensemble_equity))
        .route("/:exchange/ensemble/rebalance", post(trigger_rebalance))
        .route(
            "/:exchange/ensemble/backtest",
            get(get_backtest_results).post(trigger_backtest),
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
    let mode = payload
        .mode
        .as_deref()
        .and_then(|s| s.parse::<StrategyMode>().ok())
        .unwrap_or(StrategyMode::LongOnly);
    let mode_str = mode.as_str();

    let result = if payload.token_address == "ALL" || payload.token_address == "UNIVERSAL" {
        backtester
            .run_portfolio_simulation(&payload.genome, days)
            .await
    } else {
        backtester
            .run_detailed_simulation(&payload.genome, &payload.token_address, days, mode)
            .await
    };

    match result {
        Ok(result) => {
            let metrics = result.get("metrics").cloned().unwrap_or(json!({}));
            let equity = result.get("equity_curve").cloned().unwrap_or(json!([]));

            // Keep only the latest backtest per (token_address, mode)
            let _ =
                sqlx::query("DELETE FROM backtest_results WHERE token_address = $1 AND mode = $2")
                    .bind(&payload.token_address)
                    .bind(mode_str)
                    .execute(&state.pool)
                    .await;

            let _ = sqlx::query(
                r#"INSERT INTO backtest_results
                   (genome, token_address, mode, metrics, equity_curve, pnl_percent, win_rate, total_trades, sharpe_ratio, max_drawdown)
                   VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)"#,
            )
            .bind(&payload.genome)
            .bind(&payload.token_address)
            .bind(mode_str)
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
    let mode_filter = params.mode.as_deref();

    let rows = if let Some(mode_val) = mode_filter {
        sqlx::query(
            r#"SELECT
                sg.generation,
                sg.fitness,
                sg.best_genome,
                sg.strategy_id,
                sg.timestamp,
                sg.metadata,
                sg.mode,
                br.pnl_percent,
                br.sharpe_ratio,
                br.max_drawdown,
                br.win_rate,
                br.total_trades
            FROM strategy_generations sg
            LEFT JOIN backtest_results br ON br.strategy_id = sg.strategy_id
            WHERE sg.exchange = $1 AND sg.mode = $2
            ORDER BY sg.timestamp DESC
            LIMIT $3"#,
        )
        .bind(&exchange_name)
        .bind(mode_val)
        .bind(limit)
        .fetch_all(&state.pool)
        .await
    } else {
        sqlx::query(
            r#"SELECT
                sg.generation,
                sg.fitness,
                sg.best_genome,
                sg.strategy_id,
                sg.timestamp,
                sg.metadata,
                sg.mode,
                br.pnl_percent,
                br.sharpe_ratio,
                br.max_drawdown,
                br.win_rate,
                br.total_trades
            FROM strategy_generations sg
            LEFT JOIN backtest_results br ON br.strategy_id = sg.strategy_id
            WHERE sg.exchange = $1
            ORDER BY sg.timestamp DESC
            LIMIT $2"#,
        )
        .bind(&exchange_name)
        .bind(limit)
        .fetch_all(&state.pool)
        .await
    };

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
    Query(params): Query<GenerationsQuery>,
) -> Json<Value> {
    let key = exchange.to_lowercase();
    let exchange_name = match state.exchanges.get(&key) {
        Some(cfg) => cfg.exchange.clone(),
        None => return Json(json!({"error": format!("Unknown exchange: {}", exchange)})),
    };

    let mode_filter = params.mode.as_deref();

    let row = if let Some(mode_val) = mode_filter {
        sqlx::query(
            r#"SELECT
                sg.generation,
                sg.fitness,
                sg.best_genome,
                sg.strategy_id,
                sg.timestamp,
                sg.metadata,
                sg.mode,
                br.pnl_percent,
                br.sharpe_ratio,
                br.max_drawdown,
                br.win_rate,
                br.total_trades,
                br.equity_curve,
                br.trades,
                br.metrics
            FROM strategy_generations sg
            LEFT JOIN backtest_results br ON br.strategy_id = sg.strategy_id
            WHERE sg.exchange = $1 AND sg.generation = $2 AND sg.mode = $3"#,
        )
        .bind(&exchange_name)
        .bind(gen)
        .bind(mode_val)
        .fetch_optional(&state.pool)
        .await
    } else {
        sqlx::query(
            r#"SELECT
                sg.generation,
                sg.fitness,
                sg.best_genome,
                sg.strategy_id,
                sg.timestamp,
                sg.metadata,
                sg.mode,
                br.pnl_percent,
                br.sharpe_ratio,
                br.max_drawdown,
                br.win_rate,
                br.total_trades,
                br.equity_curve,
                br.trades,
                br.metrics
            FROM strategy_generations sg
            LEFT JOIN backtest_results br ON br.strategy_id = sg.strategy_id
            WHERE sg.exchange = $1 AND sg.generation = $2"#,
        )
        .bind(&exchange_name)
        .bind(gen)
        .fetch_optional(&state.pool)
        .await
    };

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
async fn get_overview(
    State(state): State<ApiState>,
    Path(exchange): Path<String>,
    Query(params): Query<GenerationsQuery>,
) -> Json<Value> {
    let key = exchange.to_lowercase();
    let exchange_name = match state.exchanges.get(&key) {
        Some(cfg) => cfg.exchange.clone(),
        None => return Json(json!({"error": format!("Unknown exchange: {}", exchange)})),
    };

    let mode_filter = params.mode.as_deref();

    let rows = if let Some(mode_val) = mode_filter {
        sqlx::query(
            r#"SELECT
                sg.symbol,
                sg.mode,
                sg.generation AS latest_gen,
                sg.fitness AS best_fitness,
                sg.timestamp AS last_updated,
                sg.metadata,
                best_bt.pnl_percent,
                best_bt.sharpe_ratio,
                best_bt.max_drawdown,
                best_bt.win_rate
            FROM strategy_generations sg
            LEFT JOIN LATERAL (
                SELECT br.pnl_percent, br.sharpe_ratio, br.max_drawdown, br.win_rate
                FROM backtest_results br
                WHERE br.token_address = sg.symbol AND br.mode = sg.mode
                ORDER BY br.created_at DESC
                LIMIT 1
            ) best_bt ON true
            WHERE sg.exchange = $1 AND sg.mode = $2
            AND sg.generation = (
                SELECT sg2.generation FROM strategy_generations sg2
                WHERE sg2.exchange = sg.exchange AND sg2.symbol = sg.symbol AND sg2.mode = sg.mode
                ORDER BY sg2.timestamp DESC
                LIMIT 1
            )
            ORDER BY sg.symbol"#,
        )
        .bind(&exchange_name)
        .bind(mode_val)
        .fetch_all(&state.pool)
        .await
    } else {
        sqlx::query(
            r#"SELECT
                sg.symbol,
                sg.mode,
                sg.generation AS latest_gen,
                sg.fitness AS best_fitness,
                sg.timestamp AS last_updated,
                sg.metadata,
                best_bt.pnl_percent,
                best_bt.sharpe_ratio,
                best_bt.max_drawdown,
                best_bt.win_rate
            FROM strategy_generations sg
            LEFT JOIN LATERAL (
                SELECT br.pnl_percent, br.sharpe_ratio, br.max_drawdown, br.win_rate
                FROM backtest_results br
                WHERE br.token_address = sg.symbol AND br.mode = sg.mode
                ORDER BY br.created_at DESC
                LIMIT 1
            ) best_bt ON true
            WHERE sg.exchange = $1
            AND sg.generation = (
                SELECT sg2.generation FROM strategy_generations sg2
                WHERE sg2.exchange = sg.exchange AND sg2.symbol = sg.symbol AND sg2.mode = sg.mode
                ORDER BY sg2.timestamp DESC
                LIMIT 1
            )
            ORDER BY sg.symbol"#,
        )
        .bind(&exchange_name)
        .fetch_all(&state.pool)
        .await
    };

    match rows {
        Ok(rows) => {
            let symbols: Vec<Value> = rows
                .iter()
                .map(|row| {
                    let symbol: String = row.get("symbol");
                    let mode: String = row.get("mode");
                    let latest_gen: i32 = row.get("latest_gen");
                    let best_fitness: Option<f64> = row.get("best_fitness");
                    let last_updated: Option<chrono::DateTime<chrono::Utc>> =
                        row.get("last_updated");
                    let metadata: Option<Value> = row.get("metadata");
                    let pnl_percent: Option<f64> = row.get("pnl_percent");
                    let sharpe_ratio: Option<f64> = row.get("sharpe_ratio");
                    let max_drawdown: Option<f64> = row.get("max_drawdown");
                    let win_rate: Option<f64> = row.get("win_rate");

                    // Read PSR fields (new) with fallback to PnL fields (historical)
                    let oos_psr = metadata
                        .as_ref()
                        .and_then(|m| m.get("oos_psr").or_else(|| m.get("oos_ic")))
                        .and_then(|v| v.as_f64());
                    let stagnation = metadata
                        .as_ref()
                        .and_then(|m| m.get("stagnation"))
                        .and_then(|v| v.as_u64());
                    let fold_psrs = metadata
                        .as_ref()
                        .and_then(|m| m.get("fold_psrs").or_else(|| m.get("fold_pnls")))
                        .cloned();

                    json!({
                        "symbol": symbol,
                        "mode": mode,
                        "latest_gen": latest_gen,
                        "best_fitness": best_fitness,
                        "best_oos_psr": oos_psr,
                        "best_pnl": pnl_percent,
                        "sharpe_ratio": sharpe_ratio,
                        "max_drawdown": max_drawdown,
                        "win_rate": win_rate,
                        "stagnation": stagnation,
                        "fold_psrs": fold_psrs,
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
    let mode_filter = params.mode.as_deref();

    let rows = if let Some(mode_val) = mode_filter {
        sqlx::query(
            r#"SELECT
                sg.generation,
                sg.fitness,
                sg.best_genome,
                sg.strategy_id,
                sg.timestamp,
                sg.metadata,
                sg.mode,
                br.pnl_percent,
                br.sharpe_ratio,
                br.max_drawdown,
                br.win_rate,
                br.total_trades
            FROM strategy_generations sg
            LEFT JOIN backtest_results br ON br.strategy_id = sg.strategy_id
            WHERE sg.exchange = $1 AND sg.symbol = $2 AND sg.mode = $3
            ORDER BY sg.timestamp DESC
            LIMIT $4"#,
        )
        .bind(&exchange_name)
        .bind(&symbol)
        .bind(mode_val)
        .bind(limit)
        .fetch_all(&state.pool)
        .await
    } else {
        sqlx::query(
            r#"SELECT
                sg.generation,
                sg.fitness,
                sg.best_genome,
                sg.strategy_id,
                sg.timestamp,
                sg.metadata,
                sg.mode,
                br.pnl_percent,
                br.sharpe_ratio,
                br.max_drawdown,
                br.win_rate,
                br.total_trades
            FROM strategy_generations sg
            LEFT JOIN backtest_results br ON br.strategy_id = sg.strategy_id
            WHERE sg.exchange = $1 AND sg.symbol = $2
            ORDER BY sg.timestamp DESC
            LIMIT $3"#,
        )
        .bind(&exchange_name)
        .bind(&symbol)
        .bind(limit)
        .fetch_all(&state.pool)
        .await
    };

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
    Query(params): Query<GenerationsQuery>,
) -> Json<Value> {
    let key = exchange.to_lowercase();
    let exchange_name = match state.exchanges.get(&key) {
        Some(cfg) => cfg.exchange.clone(),
        None => return Json(json!({"error": format!("Unknown exchange: {}", exchange)})),
    };

    let mode_filter = params.mode.as_deref();

    let row = if let Some(mode_val) = mode_filter {
        sqlx::query(
            r#"SELECT
                sg.generation,
                sg.fitness,
                sg.best_genome,
                sg.strategy_id,
                sg.timestamp,
                sg.metadata,
                sg.mode,
                br.pnl_percent,
                br.sharpe_ratio,
                br.max_drawdown,
                br.win_rate,
                br.total_trades,
                br.equity_curve,
                br.trades,
                br.metrics
            FROM strategy_generations sg
            LEFT JOIN backtest_results br ON br.strategy_id = sg.strategy_id
            WHERE sg.exchange = $1 AND sg.symbol = $2 AND sg.generation = $3 AND sg.mode = $4
            ORDER BY br.created_at DESC LIMIT 1"#,
        )
        .bind(&exchange_name)
        .bind(&symbol)
        .bind(gen)
        .bind(mode_val)
        .fetch_optional(&state.pool)
        .await
    } else {
        sqlx::query(
            r#"SELECT
                sg.generation,
                sg.fitness,
                sg.best_genome,
                sg.strategy_id,
                sg.timestamp,
                sg.metadata,
                sg.mode,
                br.pnl_percent,
                br.sharpe_ratio,
                br.max_drawdown,
                br.win_rate,
                br.total_trades,
                br.equity_curve,
                br.trades,
                br.metrics
            FROM strategy_generations sg
            LEFT JOIN backtest_results br ON br.strategy_id = sg.strategy_id
            WHERE sg.exchange = $1 AND sg.symbol = $2 AND sg.generation = $3
            ORDER BY br.created_at DESC LIMIT 1"#,
        )
        .bind(&exchange_name)
        .bind(&symbol)
        .bind(gen)
        .fetch_optional(&state.pool)
        .await
    };

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
    let mode: Option<String> = row.try_get("mode").unwrap_or(None);
    let pnl_percent: Option<f64> = row.get("pnl_percent");
    let sharpe_ratio: Option<f64> = row.get("sharpe_ratio");
    let max_drawdown: Option<f64> = row.get("max_drawdown");
    let win_rate: Option<f64> = row.get("win_rate");
    let total_trades: Option<i32> = row.get("total_trades");

    // Read PSR fields (new) with fallback to PnL fields (historical)
    let oos_psr = metadata
        .as_ref()
        .and_then(|m| m.get("oos_psr").or_else(|| m.get("oos_ic")))
        .and_then(|v| v.as_f64());
    let stagnation = metadata
        .as_ref()
        .and_then(|m| m.get("stagnation"))
        .and_then(|v| v.as_u64());
    let fold_psrs = metadata
        .as_ref()
        .and_then(|m| m.get("fold_psrs").or_else(|| m.get("fold_pnls")))
        .cloned();

    let llm_oracle = metadata.as_ref().and_then(|m| m.get("llm_oracle")).cloned();

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
            let trades: Option<Value> = row.try_get("trades").unwrap_or(None);
            let metrics: Option<Value> = row.try_get("metrics").unwrap_or(None);
            bt["equity_curve"] = equity_curve.unwrap_or(json!(null));
            if let Some(t) = trades {
                bt["trades"] = t;
            }
            if let Some(m) = metrics {
                bt["metrics"] = m;
            }
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
        "mode": mode,
        "oos_psr": oos_psr,
        "stagnation": stagnation,
        "fold_psrs": fold_psrs,
        "backtest": backtest,
        "llm_oracle": llm_oracle,
    })
}

// ── P5: Ensemble API Endpoints ──────────────────────────────────────────

/// GET /:exchange/ensemble — current (latest) ensemble allocation
async fn get_ensemble(State(state): State<ApiState>, Path(exchange): Path<String>) -> Json<Value> {
    let key = exchange.to_lowercase();
    if !state.exchanges.contains_key(&key) {
        return Json(json!({"error": format!("Unknown exchange: {}", exchange)}));
    }
    let exchange_name = state
        .exchanges
        .get(&key)
        .map(|c| c.exchange.as_str())
        .unwrap_or(&exchange);

    let row = sqlx::query(
        "SELECT pe.*, \
         (SELECT json_agg(row_to_json(pes.*)) \
          FROM portfolio_ensemble_strategies pes \
          WHERE pes.ensemble_id = pe.id) as strategies \
         FROM portfolio_ensembles pe \
         WHERE pe.exchange = $1 \
         ORDER BY pe.version DESC \
         LIMIT 1",
    )
    .bind(exchange_name)
    .fetch_optional(&state.pool)
    .await;

    match row {
        Ok(Some(r)) => {
            let id: uuid::Uuid = r.get("id");
            let version: i32 = r.get("version");
            let strategy_count: i32 = r.get("strategy_count");
            let portfolio_sharpe: Option<f64> = r.get("portfolio_sharpe");
            let portfolio_max_drawdown: Option<f64> = r.get("portfolio_max_drawdown");
            let avg_pairwise_correlation: Option<f64> = r.get("avg_pairwise_correlation");
            let crowded_pair_count: Option<i32> = r.get("crowded_pair_count");
            let weights: Value = r.get("weights");
            let hrp_diagnostics: Option<Value> = r.get("hrp_diagnostics");
            let strategies: Option<Value> = r.get("strategies");
            let created_at: Option<chrono::DateTime<chrono::Utc>> = r.get("created_at");

            Json(json!({
                "id": id.to_string(),
                "exchange": exchange_name,
                "version": version,
                "strategy_count": strategy_count,
                "portfolio_sharpe": portfolio_sharpe,
                "portfolio_max_drawdown": portfolio_max_drawdown,
                "avg_pairwise_correlation": avg_pairwise_correlation,
                "crowded_pair_count": crowded_pair_count,
                "weights": weights,
                "hrp_diagnostics": hrp_diagnostics,
                "strategies": strategies,
                "created_at": created_at.map(|t| t.to_rfc3339()),
            }))
        }
        Ok(None) => Json(json!({"error": "No ensemble found for this exchange"})),
        Err(e) => Json(json!({"error": format!("Database error: {}", e)})),
    }
}

#[derive(Deserialize)]
struct EnsembleHistoryQuery {
    limit: Option<i64>,
}

/// GET /:exchange/ensemble/history — past ensemble versions
async fn get_ensemble_history(
    State(state): State<ApiState>,
    Path(exchange): Path<String>,
    Query(params): Query<EnsembleHistoryQuery>,
) -> Json<Value> {
    let key = exchange.to_lowercase();
    if !state.exchanges.contains_key(&key) {
        return Json(json!({"error": format!("Unknown exchange: {}", exchange)}));
    }
    let exchange_name = state
        .exchanges
        .get(&key)
        .map(|c| c.exchange.as_str())
        .unwrap_or(&exchange);
    let limit = params.limit.unwrap_or(10).min(100);

    let rows = sqlx::query(
        "SELECT id, version, strategy_count, portfolio_sharpe, \
         portfolio_max_drawdown, avg_pairwise_correlation, \
         crowded_pair_count, weights, created_at \
         FROM portfolio_ensembles \
         WHERE exchange = $1 \
         ORDER BY version DESC \
         LIMIT $2",
    )
    .bind(exchange_name)
    .bind(limit)
    .fetch_all(&state.pool)
    .await;

    match rows {
        Ok(rows) => {
            let ensembles: Vec<Value> = rows
                .iter()
                .map(|r| {
                    let id: uuid::Uuid = r.get("id");
                    let version: i32 = r.get("version");
                    let strategy_count: i32 = r.get("strategy_count");
                    let portfolio_sharpe: Option<f64> = r.get("portfolio_sharpe");
                    let portfolio_max_drawdown: Option<f64> = r.get("portfolio_max_drawdown");
                    let avg_corr: Option<f64> = r.get("avg_pairwise_correlation");
                    let crowded: Option<i32> = r.get("crowded_pair_count");
                    let weights: Value = r.get("weights");
                    let created_at: Option<chrono::DateTime<chrono::Utc>> = r.get("created_at");
                    json!({
                        "id": id.to_string(),
                        "version": version,
                        "strategy_count": strategy_count,
                        "portfolio_sharpe": portfolio_sharpe,
                        "portfolio_max_drawdown": portfolio_max_drawdown,
                        "avg_pairwise_correlation": avg_corr,
                        "crowded_pair_count": crowded,
                        "weights": weights,
                        "created_at": created_at.map(|t| t.to_rfc3339()),
                    })
                })
                .collect();
            Json(json!({"ensembles": ensembles, "count": ensembles.len()}))
        }
        Err(e) => Json(json!({"error": format!("Database error: {}", e)})),
    }
}

#[derive(Deserialize)]
struct EnsembleEquityQuery {
    version: Option<i32>,
    limit: Option<i64>,
}

/// GET /:exchange/ensemble/equity — shadow portfolio equity curve
async fn get_ensemble_equity(
    State(state): State<ApiState>,
    Path(exchange): Path<String>,
    Query(params): Query<EnsembleEquityQuery>,
) -> Json<Value> {
    let key = exchange.to_lowercase();
    if !state.exchanges.contains_key(&key) {
        return Json(json!({"error": format!("Unknown exchange: {}", exchange)}));
    }
    let exchange_name = state
        .exchanges
        .get(&key)
        .map(|c| c.exchange.as_str())
        .unwrap_or(&exchange);
    let limit = params.limit.unwrap_or(500).min(5000);

    // If version not specified, use the latest
    let version = match params.version {
        Some(v) => v,
        None => {
            let row = sqlx::query(
                "SELECT COALESCE(MAX(version), 0) as v FROM portfolio_ensembles WHERE exchange = $1",
            )
            .bind(exchange_name)
            .fetch_one(&state.pool)
            .await;
            match row {
                Ok(r) => r.get("v"),
                Err(e) => return Json(json!({"error": format!("Database error: {}", e)})),
            }
        }
    };

    let rows = sqlx::query(
        "SELECT timestamp, equity, period_return \
         FROM portfolio_ensemble_equity \
         WHERE exchange = $1 AND ensemble_version = $2 \
         ORDER BY timestamp DESC \
         LIMIT $3",
    )
    .bind(exchange_name)
    .bind(version)
    .bind(limit)
    .fetch_all(&state.pool)
    .await;

    match rows {
        Ok(rows) => {
            let points: Vec<Value> = rows
                .iter()
                .map(|r| {
                    let ts: chrono::DateTime<chrono::Utc> = r.get("timestamp");
                    let equity: f64 = r.get("equity");
                    let period_return: Option<f64> = r.get("period_return");
                    json!({
                        "timestamp": ts.to_rfc3339(),
                        "equity": equity,
                        "period_return": period_return,
                    })
                })
                .collect();
            Json(json!({
                "exchange": exchange_name,
                "version": version,
                "equity_curve": points,
                "count": points.len(),
            }))
        }
        Err(e) => Json(json!({"error": format!("Database error: {}", e)})),
    }
}

#[derive(Deserialize)]
#[allow(dead_code)]
struct RebalanceRequest {
    #[serde(default)]
    force: bool,
}

/// POST /:exchange/ensemble/rebalance — manually trigger rebalance
async fn trigger_rebalance(
    State(state): State<ApiState>,
    Path(exchange): Path<String>,
    Json(_payload): Json<RebalanceRequest>,
) -> Json<Value> {
    let key = exchange.to_lowercase();
    if !state.exchanges.contains_key(&key) {
        return Json(json!({"error": format!("Unknown exchange: {}", exchange)}));
    }

    // Return the current ensemble status (actual rebalance is done by the background loop)
    // The API serves as a status check / manual trigger signal
    let row = sqlx::query(
        "SELECT version, strategy_count, portfolio_sharpe, created_at \
         FROM portfolio_ensembles \
         WHERE exchange = $1 \
         ORDER BY version DESC LIMIT 1",
    )
    .bind(
        state
            .exchanges
            .get(&key)
            .map(|c| c.exchange.as_str())
            .unwrap_or(&exchange),
    )
    .fetch_optional(&state.pool)
    .await;

    match row {
        Ok(Some(r)) => {
            let version: i32 = r.get("version");
            let strategy_count: i32 = r.get("strategy_count");
            let sharpe: Option<f64> = r.get("portfolio_sharpe");
            let created_at: Option<chrono::DateTime<chrono::Utc>> = r.get("created_at");
            Json(json!({
                "status": "rebalance_acknowledged",
                "current_version": version,
                "strategy_count": strategy_count,
                "portfolio_sharpe": sharpe,
                "last_rebalance": created_at.map(|t| t.to_rfc3339()),
            }))
        }
        Ok(None) => Json(json!({
            "status": "no_ensemble_yet",
            "message": "No ensemble has been created yet. The background loop will create one shortly.",
        })),
        Err(e) => Json(json!({"error": format!("Database error: {}", e)})),
    }
}

/// POST /:exchange/ensemble/backtest — trigger walk-forward backtest
async fn trigger_backtest(
    State(state): State<ApiState>,
    Path(exchange): Path<String>,
) -> Json<Value> {
    let key = exchange.to_lowercase();
    let exchange_name = match state.exchanges.get(&key) {
        Some(cfg) => cfg.exchange.clone(),
        None => return Json(json!({"error": format!("Unknown exchange: {}", exchange)})),
    };

    match crate::backtest::ensemble::run_ensemble_walk_forward(&state.pool, &exchange_name).await {
        Ok(Some(result)) => {
            let response = json!({
                "status": "completed",
                "exchange": result.exchange,
                "backtest_start": result.backtest_start.to_rfc3339(),
                "backtest_end": result.backtest_end.to_rfc3339(),
                "rebalance_count": result.rebalance_count,
                "cumulative_return": result.cumulative_return,
                "annualized_sharpe": result.annualized_sharpe,
                "max_drawdown": result.max_drawdown,
                "avg_turnover": result.avg_turnover,
                "total_turnover_cost": result.total_turnover_cost,
                "avg_strategy_count": result.avg_strategy_count,
            });
            // Persist result
            if let Err(e) =
                crate::backtest::ensemble::persist_backtest_result(&state.pool, &result).await
            {
                tracing::error!("[{}] Failed to persist backtest: {}", exchange_name, e);
            }
            Json(response)
        }
        Ok(None) => Json(json!({
            "status": "insufficient_data",
            "message": "Not enough ensemble history for backtest",
        })),
        Err(e) => Json(json!({"error": format!("Backtest failed: {}", e)})),
    }
}

/// GET /:exchange/ensemble/backtest — get latest backtest results
async fn get_backtest_results(
    State(state): State<ApiState>,
    Path(exchange): Path<String>,
) -> Json<Value> {
    let key = exchange.to_lowercase();
    let exchange_name = match state.exchanges.get(&key) {
        Some(cfg) => cfg.exchange.clone(),
        None => return Json(json!({"error": format!("Unknown exchange: {}", exchange)})),
    };

    let row = sqlx::query(
        "SELECT id, exchange, backtest_start, backtest_end, rebalance_count, \
                cumulative_return, annualized_sharpe, max_drawdown, \
                avg_turnover, total_turnover_cost, avg_strategy_count, \
                created_at \
         FROM ensemble_backtest_results \
         WHERE exchange = $1 \
         ORDER BY created_at DESC LIMIT 1",
    )
    .bind(&exchange_name)
    .fetch_optional(&state.pool)
    .await;

    match row {
        Ok(Some(r)) => {
            let id: uuid::Uuid = r.get("id");
            let start: chrono::DateTime<chrono::Utc> = r.get("backtest_start");
            let end: chrono::DateTime<chrono::Utc> = r.get("backtest_end");
            let count: i32 = r.get("rebalance_count");
            let cum_ret: Option<f64> = r.get("cumulative_return");
            let sharpe: Option<f64> = r.get("annualized_sharpe");
            let max_dd: Option<f64> = r.get("max_drawdown");
            let avg_turn: Option<f64> = r.get("avg_turnover");
            let total_cost: Option<f64> = r.get("total_turnover_cost");
            let avg_strat: Option<f64> = r.get("avg_strategy_count");
            let created: chrono::DateTime<chrono::Utc> = r.get("created_at");
            Json(json!({
                "id": id.to_string(),
                "exchange": exchange_name,
                "backtest_start": start.to_rfc3339(),
                "backtest_end": end.to_rfc3339(),
                "rebalance_count": count,
                "cumulative_return": cum_ret,
                "annualized_sharpe": sharpe,
                "max_drawdown": max_dd,
                "avg_turnover": avg_turn,
                "total_turnover_cost": total_cost,
                "avg_strategy_count": avg_strat,
                "computed_at": created.to_rfc3339(),
            }))
        }
        Ok(None) => Json(json!({
            "status": "no_backtest_yet",
            "message": "No backtest has been run yet. POST to trigger one.",
        })),
        Err(e) => Json(json!({"error": format!("Database error: {}", e)})),
    }
}
