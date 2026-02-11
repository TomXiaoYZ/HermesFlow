use crate::config::IbkrConfig;
use crate::error::{DataError, Result};
use crate::models::{AssetType, DataSourceType, MarketDataType, StandardMarketData};
use crate::repository::MarketDataRepository;
use crate::traits::{ConnectorStats, DataSourceConnector};
// use testcontainers::core::mounts::AccessMode::ReadWrite; // Removed unused import
use async_trait::async_trait;
use chrono::{TimeZone, Utc};
use ibapi::market_data::historical::{BarSize, Duration as IbDuration};
use ibapi::prelude::*;
use rust_decimal::Decimal;
use std::sync::Arc;
use time::OffsetDateTime;
use tokio::sync::mpsc::{self, Receiver};
use tracing::{error, info, warn};

pub struct IBKRCollector {
    config: IbkrConfig,
    client: Option<Arc<Client>>,
    repository: Arc<dyn MarketDataRepository>,
    stats: ConnectorStats,
    running: bool,
}

impl IBKRCollector {
    pub fn new(config: IbkrConfig, repository: Arc<dyn MarketDataRepository>) -> Self {
        Self {
            config,
            client: None,
            repository,
            stats: ConnectorStats::default(),
            running: false,
        }
    }
}

#[async_trait]
impl DataSourceConnector for IBKRCollector {
    fn source_type(&self) -> DataSourceType {
        DataSourceType::IbkrStock
    }

    fn supported_assets(&self) -> Vec<AssetType> {
        vec![AssetType::Stock, AssetType::Option]
    }

    async fn connect(&mut self) -> Result<Receiver<StandardMarketData>> {
        self.running = true;
        let addr = format!("{}:{}", self.config.host, self.config.port);
        info!("Connecting to IBKR for data collection at {}", addr);

        let (tx, rx) = mpsc::channel(100);

        // Connect to IB Gateway
        let client = Client::connect(&addr, self.config.client_id)
            .await
            .map_err(DataError::IbkrError)?;

        info!("IBKR Collector connected successfully");
        let client_arc = Arc::new(client);
        self.client = Some(client_arc.clone());

        let symbols = self.config.symbols.clone();
        let repository = self.repository.clone(); // Updated to use repository trait

        // Ensure we get live data
        // if let Err(e) = client_arc.switch_market_data_type(IbMarketDataType::Realtime).await {
        //     warn!("Failed to set market data type to Live: {}", e);
        // }

        // Spawn a task to manage subscriptions and potentially historical data
        tokio::spawn(async move {
            // 1. Fetch Historical Data for each symbol and resolution
            let timeframes = vec![
                ("1m", IbDuration::days(7), BarSize::Min),
                ("15m", IbDuration::days(30), BarSize::Min15),
                ("1h", IbDuration::months(6), BarSize::Hour),
                ("4h", IbDuration::years(1), BarSize::Hour4),
                ("1d", IbDuration::years(5), BarSize::Day),
            ];

            for symbol in &symbols {
                let contract = Contract::stock(symbol).build();

                for (resolution, duration, bar_size) in &timeframes {
                    info!(
                        "Fetching {} historical data for {} (Lookback: {})",
                        resolution, symbol, duration
                    );

                    // Explicit end date is required to avoid ibapi panic
                    let end_date_time =
                        OffsetDateTime::from_unix_timestamp(Utc::now().timestamp()).unwrap();

                    let historical_result = client_arc
                        .historical_data(
                            &contract,
                            Some(end_date_time),
                            *duration,
                            *bar_size,
                            Some(HistoricalWhatToShow::Trades),
                            TradingHours::Extended,
                        )
                        .await;

                    match historical_result {
                        Ok(historical_data) => {
                            let mut count = 0;
                            for bar in historical_data.bars {
                                // Convert bar.date (OffsetDateTime) to Utc timestamp
                                let timestamp =
                                    Utc.timestamp_opt(bar.date.unix_timestamp(), 0).unwrap();

                                // Create Candle DTO (Force Update)
                                let candle = crate::models::Candle::new(
                                    "IBKR".to_string(),
                                    symbol.clone(),
                                    resolution.to_string(),
                                    Decimal::from_f64_retain(bar.open).unwrap_or_default(),
                                    Decimal::from_f64_retain(bar.high).unwrap_or_default(),
                                    Decimal::from_f64_retain(bar.low).unwrap_or_default(),
                                    Decimal::from_f64_retain(bar.close).unwrap_or_default(),
                                    Decimal::from_f64_retain(bar.volume).unwrap_or_default(),
                                    timestamp,
                                );

                                if let Err(e) = repository.insert_candle(&candle).await {
                                    warn!(
                                        "Failed to insert {} candle for {}: {}",
                                        resolution, symbol, e
                                    );
                                }
                                count += 1;
                            }
                            info!(
                                "Fetched {} historical {} bars for {}",
                                count, resolution, symbol
                            );
                        }
                        Err(e) => error!(
                            "Failed to fetch {} historical data for {}: {}",
                            resolution, symbol, e
                        ),
                    }

                    // Small delay between requests to be polite to the API
                    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
                }
            }

            // 2. Subscribe to Real-time Bars
            for symbol in &symbols {
                let contract = Contract::stock(symbol).build();
                info!("Subscribing to realtime bars for {}", symbol);

                match client_arc
                    .realtime_bars(
                        &contract,
                        RealtimeBarSize::Sec5,
                        RealtimeWhatToShow::Trades,
                        TradingHours::Extended,
                    )
                    .await
                {
                    Ok(mut stream) => {
                        let tx_clone = tx.clone();
                        let sym = symbol.clone();

                        tokio::spawn(async move {
                            while let Some(bar_result) = stream.next().await {
                                match bar_result {
                                    Ok(bar) => {
                                        let price =
                                            Decimal::from_f64_retain(bar.close).unwrap_or_default();
                                        let md = StandardMarketData {
                                            symbol: sym.clone(),
                                            data_type: MarketDataType::Ticker,
                                            asset_type: AssetType::Stock,
                                            price,
                                            quantity: Decimal::from_f64_retain(bar.volume).unwrap_or_default(),
                                            timestamp: bar.date.unix_timestamp_nanos() as i64 / 1_000_000,
                                            source: DataSourceType::IbkrStock,
                                            exchange: "IBKR".to_string(),
                                            bid: None,
                                            ask: None,
                                            volume_24h: Some(Decimal::from_f64_retain(bar.volume).unwrap_or_default()),
                                            high_24h: Some(Decimal::from_f64_retain(bar.high).unwrap_or_default()),
                                            low_24h: Some(Decimal::from_f64_retain(bar.low).unwrap_or_default()),
                                            raw_data: format!(
                                                "{{\"open\":{},\"high\":{},\"low\":{},\"close\":{},\"volume\":{},\"wap\":{},\"count\":{}}}",
                                                bar.open, bar.high, bar.low, bar.close, bar.volume, bar.wap, bar.count
                                            ),
                                            received_at: Utc::now().timestamp_millis(),
                                            sequence_id: None,
                                            open_interest: None,
                                            funding_rate: None,
                                            liquidity: None,
                                            fdv: None,
                                        };

                                        if let Err(e) = tx_clone.send(md).await {
                                            warn!("Failed to send market data update: {}", e);
                                            break;
                                        }
                                    }
                                    Err(e) => {
                                        error!("Error receiving realtime bar for {}: {}", sym, e)
                                    }
                                }
                            }
                        });
                    }
                    Err(e) => error!("Failed to subscribe to realtime bars for {}: {}", symbol, e),
                }
            }
        });

        Ok(rx)
    }

    async fn disconnect(&mut self) -> Result<()> {
        info!("Disconnecting IBKR Collector");
        self.running = false;
        self.client = None;
        Ok(())
    }

    async fn is_healthy(&self) -> bool {
        self.running
    }

    fn stats(&self) -> ConnectorStats {
        self.stats.clone()
    }
}
