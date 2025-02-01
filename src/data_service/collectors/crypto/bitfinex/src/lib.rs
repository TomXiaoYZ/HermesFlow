pub mod collector;
pub mod error;
pub mod models;
pub mod processor;
pub mod rest;
pub mod types;
pub mod websocket;

#[cfg(test)]
mod tests;

pub use collector::{BitfinexCollector, BitfinexCollectorConfig};
pub use error::{BitfinexError, Result};
pub use models::{ExchangeInfo, ExchangeStatus, Kline, Orderbook, Symbol, Ticker, Trade, TradeSide};
pub use processor::{BitfinexProcessor, BitfinexProcessorConfig, ProcessorEvent};
pub use rest::{BitfinexRestClient, BitfinexRestConfig};
pub use types::{ApiResponse, OrderbookInfo, SymbolInfo, TickerInfo, TradeInfo};
pub use websocket::{BitfinexWebsocketClient, BitfinexWebsocketConfig};
