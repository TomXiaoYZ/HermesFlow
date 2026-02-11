use crate::collectors::polygon::PolygonConnector;
use sqlx::PgPool;
use std::error::Error;
use tracing::{error, info, warn};

/// Sync historical data for a list of tickers
pub async fn sync_polygon_history(
    pool: &PgPool,
    polygon: &PolygonConnector,
    tickers: Vec<String>,
    resolutions: Vec<String>,
    from_date: &str,
    to_date: &str,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    info!(
        "Starting Polygon historical sync for {} tickers, {} resolutions",
        tickers.len(),
        resolutions.len()
    );

    for ticker in &tickers {
        for resolution in &resolutions {
            info!("Syncing {} - {}", ticker, resolution);

            match polygon
                .fetch_history_candles(ticker, resolution, from_date, to_date)
                .await
            {
                Ok(candles) => {
                    if candles.is_empty() {
                        warn!("No candles returned for {} ({})", ticker, resolution);
                        continue;
                    }

                    info!(
                        "Fetched {} candles for {} ({}), inserting into database...",
                        candles.len(),
                        ticker,
                        resolution
                    );

                    match insert_candles_batch(pool, &candles).await {
                        Ok(inserted) => {
                            info!(
                                "Successfully inserted {} candles for {} ({})",
                                inserted, ticker, resolution
                            );
                        }
                        Err(e) => {
                            error!(
                                "Failed to insert candles for {} ({}): {}",
                                ticker, resolution, e
                            );
                        }
                    }
                }
                Err(e) => {
                    error!("Failed to fetch {} ({}): {}", ticker, resolution, e);
                    // Continue with next ticker/resolution instead of failing entirely
                    continue;
                }
            }

            // Small delay between tickers to avoid rate limiting
            tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
        }
    }

    info!("Polygon historical sync completed");
    Ok(())
}

/// Insert candles into database in batch
async fn insert_candles_batch(
    pool: &PgPool,
    candles: &[crate::models::Candle],
) -> Result<usize, Box<dyn Error + Send + Sync>> {
    if candles.is_empty() {
        return Ok(0);
    }

    let mut inserted = 0;

    // Insert in chunks of 1000 to avoid huge transactions
    for chunk in candles.chunks(1000) {
        let mut query_builder = sqlx::QueryBuilder::new(
            "INSERT INTO mkt_equity_candles (time, exchange, symbol, resolution, open, high, low, close, volume, metadata) "
        );

        query_builder.push_values(chunk, |mut b, candle| {
            b.push_bind(candle.time)
                .push_bind(&candle.exchange)
                .push_bind(&candle.symbol)
                .push_bind(&candle.resolution)
                .push_bind(candle.open)
                .push_bind(candle.high)
                .push_bind(candle.low)
                .push_bind(candle.close)
                .push_bind(candle.volume)
                .push_bind(&candle.metadata);
        });

        query_builder.push(" ON CONFLICT (exchange, symbol, resolution, time) DO NOTHING");

        let result = query_builder.build().execute(pool).await?;
        inserted += result.rows_affected() as usize;
    }

    Ok(inserted)
}

/// Get the last synced timestamp for a ticker/resolution
pub async fn get_last_synced_time(
    pool: &PgPool,
    exchange: &str,
    ticker: &str,
    resolution: &str,
) -> Result<Option<chrono::DateTime<chrono::Utc>>, Box<dyn Error + Send + Sync>> {
    let result = sqlx::query_scalar::<_, chrono::DateTime<chrono::Utc>>(
        "SELECT MAX(time) FROM mkt_equity_candles 
         WHERE exchange = $1 AND symbol = $2 AND resolution = $3",
    )
    .bind(exchange)
    .bind(ticker)
    .bind(resolution)
    .fetch_optional(pool)
    .await?;

    Ok(result)
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_date_range_calculation() {
        // Simple sanity check
        let from = chrono::NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
        let to = chrono::NaiveDate::from_ymd_opt(2024, 1, 31).unwrap();

        let days = (to - from).num_days();
        assert_eq!(days, 30);
    }
}
