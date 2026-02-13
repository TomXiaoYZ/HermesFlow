use clickhouse::Client;
use rust_decimal::Decimal;
use serde::Serialize;
use time::OffsetDateTime;
use tokio::sync::mpsc;

use crate::models::StandardMarketData;
use crate::monitoring::metrics::DEAD_LETTER_TOTAL;

/// ClickHouse Decimal(32, 8) scale factor.
const CH_DECIMAL_SCALE: u32 = 8;

fn decimal_to_ch(d: Decimal) -> i128 {
    let mut d = d;
    d.rescale(CH_DECIMAL_SCALE);
    d.mantissa()
}

/// Row struct matching ClickHouse dead_letters table schema.
#[derive(Debug, clickhouse::Row, Serialize)]
struct DeadLetterRow {
    source: String,
    exchange: String,
    symbol: String,
    price: i128,
    quantity: i128,
    #[serde(with = "clickhouse::serde::time::datetime64::millis")]
    timestamp: OffsetDateTime,
    storage_target: String,
    error: String,
    raw_data: String,
}

impl DeadLetterRow {
    fn from_market_data(data: &StandardMarketData, error: &str, target: &str) -> Self {
        let ts_millis = data.timestamp;
        let ts_secs = ts_millis / 1000;
        let ts_nanos = ((ts_millis % 1000) * 1_000_000) as u32;
        let timestamp = OffsetDateTime::from_unix_timestamp(ts_secs)
            .map(|t| t.replace_nanosecond(ts_nanos).unwrap_or(t))
            .unwrap_or_else(|_| OffsetDateTime::now_utc());

        Self {
            source: data.source.as_str().to_string(),
            exchange: data.exchange.clone(),
            symbol: data.symbol.clone(),
            price: decimal_to_ch(data.price),
            quantity: decimal_to_ch(data.quantity),
            timestamp,
            storage_target: target.to_string(),
            error: error.to_string(),
            raw_data: data.raw_data.clone(),
        }
    }
}

/// Message sent through the dead letter channel.
struct DeadLetterMsg {
    data: StandardMarketData,
    error: String,
    target: String,
}

/// Writer that persists dead letters to ClickHouse via a background task.
///
/// Uses a bounded mpsc channel so callers never block on ClickHouse I/O.
/// If the channel is full, the dead letter is still logged via tracing.
pub struct DeadLetterWriter {
    tx: mpsc::Sender<DeadLetterMsg>,
}

impl DeadLetterWriter {
    /// Create a new dead letter writer with a background ClickHouse persistence task.
    ///
    /// `ch_url` / `ch_database` / `ch_user` / `ch_password` configure the
    /// independent ClickHouse connection (separate from the main data pipeline).
    pub fn new(ch_url: &str, ch_database: &str, ch_user: &str, ch_password: &str) -> Self {
        let (tx, rx) = mpsc::channel::<DeadLetterMsg>(256);

        let mut client = Client::default()
            .with_url(ch_url)
            .with_database(ch_database)
            .with_user(ch_user);
        if !ch_password.is_empty() {
            client = client.with_password(ch_password);
        }

        tokio::spawn(Self::background_writer(client, rx));

        Self { tx }
    }

    /// Record a dead letter. Sends to background writer; falls back to log-only
    /// if the channel is full.
    pub fn record(&self, data: &StandardMarketData, error: &str, target: &str) {
        DEAD_LETTER_TOTAL.inc();

        // Always log for structured audit trail
        tracing::error!(
            target: "dead_letter",
            symbol = %data.symbol,
            source = %data.source,
            price = %data.price,
            timestamp = data.timestamp,
            storage_target = target,
            error = error,
            "DEAD LETTER: data permanently dropped after retry exhaustion"
        );

        // Best-effort send to ClickHouse writer
        let msg = DeadLetterMsg {
            data: data.clone(),
            error: error.to_string(),
            target: target.to_string(),
        };
        if self.tx.try_send(msg).is_err() {
            tracing::warn!("Dead letter channel full, ClickHouse persistence skipped");
        }
    }

    /// Background task that drains the channel and batch-inserts into ClickHouse.
    async fn background_writer(client: Client, mut rx: mpsc::Receiver<DeadLetterMsg>) {
        // Create schema on startup
        let schema_sql = include_str!(
            "../../../../infrastructure/database/clickhouse/migrations/006_dead_letter.sql"
        );
        if let Err(e) = client.query(schema_sql).execute().await {
            tracing::error!("Failed to create dead_letters schema: {}", e);
        }

        while let Some(msg) = rx.recv().await {
            let row = DeadLetterRow::from_market_data(&msg.data, &msg.error, &msg.target);

            if let Err(e) = Self::insert_row(&client, &row).await {
                tracing::error!(
                    symbol = %row.symbol,
                    source = %row.source,
                    "Failed to persist dead letter to ClickHouse: {}",
                    e
                );
            }
        }
    }

    async fn insert_row(client: &Client, row: &DeadLetterRow) -> Result<(), String> {
        let mut insert = client
            .insert("dead_letters")
            .map_err(|e| format!("Insert init: {}", e))?;
        insert
            .write(row)
            .await
            .map_err(|e| format!("Row write: {}", e))?;
        insert
            .end()
            .await
            .map_err(|e| format!("Insert end: {}", e))?;
        Ok(())
    }
}

/// Log a permanently dropped record (legacy compatibility).
///
/// Prefer using `DeadLetterWriter::record()` when a writer instance is available.
pub fn log_dead_letter(data: &StandardMarketData, error: &str, target: &str) {
    DEAD_LETTER_TOTAL.inc();
    tracing::error!(
        target: "dead_letter",
        symbol = %data.symbol,
        source = %data.source,
        price = %data.price,
        timestamp = data.timestamp,
        storage_target = target,
        error = error,
        "DEAD LETTER: data permanently dropped after retry exhaustion"
    );
}
