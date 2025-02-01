use async_trait::async_trait;
use crate::{KucoinCollector, KucoinConfig, Result};
use crate::models::*;
use crate::rest::KucoinRestClient;
use crate::websocket::KucoinWebsocketClient;
use crate::types::*;

/// Kucoin数据采集器的实现
pub struct KucoinCollectorImpl {
    /// REST API客户端
    rest_client: KucoinRestClient,
    /// WebSocket客户端
    ws_client: KucoinWebsocketClient,
}

impl KucoinCollectorImpl {
    /// 创建新的数据采集器
    pub fn new(config: KucoinConfig) -> Self {
        Self {
            rest_client: KucoinRestClient::new(config.clone()),
            ws_client: KucoinWebsocketClient::new(config),
        }
    }

    /// 构建频道名称
    fn build_channel(symbol: &str, channel_type: &str) -> String {
        match channel_type {
            "ticker" => format!("/market/ticker:{}", symbol.to_uppercase()),
            "depth" => format!("/market/level2:{}", symbol.to_uppercase()),
            "trade" => format!("/market/match:{}", symbol.to_uppercase()),
            _ => format!("/market/{}:{}", channel_type, symbol.to_uppercase()),
        }
    }
}

#[async_trait]
impl KucoinCollector for KucoinCollectorImpl {
    async fn get_symbols(&self) -> Result<Vec<Symbol>> {
        let symbols = self.rest_client.get_symbols().await?;
        Ok(symbols.into_iter().map(Symbol::from).collect())
    }

    async fn get_ticker(&self, symbol: &str) -> Result<Ticker> {
        let ticker = self.rest_client.get_ticker(symbol).await?;
        Ok(ticker.into())
    }

    async fn get_orderbook(&self, symbol: &str, limit: Option<u32>) -> Result<Orderbook> {
        let orderbook = self.rest_client.get_orderbook(symbol, limit).await?;
        Ok((symbol.to_string(), orderbook).into())
    }

    async fn get_trades(&self, symbol: &str, limit: Option<u32>) -> Result<Vec<Trade>> {
        let trades = self.rest_client.get_trades(symbol, limit).await?;
        Ok(trades.into_iter().map(|trade| (symbol.to_string(), trade).into()).collect())
    }

    async fn subscribe_market_data(&mut self, symbols: Vec<String>, channels: Vec<String>) -> Result<()> {
        // 确保WebSocket已连接
        if self.ws_client.connect().await.is_err() {
            return Err(crate::KucoinError::NetworkError("Failed to connect to WebSocket".to_string()));
        }

        // 构建订阅频道
        let mut subscription_channels = Vec::new();
        for symbol in &symbols {
            for channel in &channels {
                subscription_channels.push(Self::build_channel(symbol, channel));
            }
        }

        // 发送订阅请求
        self.ws_client.subscribe(subscription_channels).await
    }

    async fn unsubscribe_market_data(&mut self, symbols: Vec<String>, channels: Vec<String>) -> Result<()> {
        // 构建取消订阅的频道
        let mut unsubscription_channels = Vec::new();
        for symbol in &symbols {
            for channel in &channels {
                unsubscription_channels.push(Self::build_channel(symbol, channel));
            }
        }

        // 发送取消订阅请求
        self.ws_client.unsubscribe(unsubscription_channels).await
    }
}
