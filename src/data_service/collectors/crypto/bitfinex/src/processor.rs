use crate::collector::{BitfinexCollector, BitfinexCollectorConfig, CollectorEvent};
use crate::error::{Result, BitfinexError};
use crate::models::{Kline, Orderbook, Ticker, Trade};

use async_trait::async_trait;
use futures::StreamExt;
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};
use tokio::time::{Duration, Instant};
use std::time::{SystemTime};
use tokio::task::JoinHandle;
use thiserror::Error;

/// 数据处理器配置
#[derive(Debug, Clone)]
pub struct BitfinexProcessorConfig {
    /// 收集器配置
    pub collector_config: BitfinexCollectorConfig,
    /// Ticker缓存过期时间(秒)
    pub ticker_cache_ttl: Duration,
    /// 订单簿缓存过期时间(秒)
    pub orderbook_cache_ttl: Duration,
    /// 成交缓存过期时间(秒)
    pub trades_cache_ttl: Duration,
    /// K线缓存过期时间(秒)
    pub klines_cache_ttl: Duration,
    /// 成交缓存数量限制
    pub trades_cache_size: usize,
    /// K线缓存数量限制
    pub klines_cache_size: usize,
    /// 重连配置
    pub reconnect_config: ReconnectConfig,
}

impl Default for BitfinexProcessorConfig {
    fn default() -> Self {
        Self {
            collector_config: BitfinexCollectorConfig::default(),
            ticker_cache_ttl: Duration::from_secs(60),
            orderbook_cache_ttl: Duration::from_secs(60),
            trades_cache_ttl: Duration::from_secs(3600),
            klines_cache_ttl: Duration::from_secs(3600),
            trades_cache_size: 1000,
            klines_cache_size: 1000,
            reconnect_config: ReconnectConfig::default(),
        }
    }
}

/// 重连配置
#[derive(Debug, Clone)]
pub struct ReconnectConfig {
    /// 最大重试次数，0表示无限重试
    pub max_retries: u32,
    /// 初始重试间隔（秒）
    pub initial_interval: Duration,
    /// 最大重试间隔（秒）
    pub max_interval: Duration,
    /// 重试间隔增长因子
    pub backoff_factor: f64,
}

impl Default for ReconnectConfig {
    fn default() -> Self {
        Self {
            max_retries: 0,
            initial_interval: Duration::from_secs(1),
            max_interval: Duration::from_secs(300),
            backoff_factor: 2.0,
        }
    }
}

/// 缓存数据项
#[derive(Debug, Clone)]
struct CacheItem<T> {
    /// 数据
    data: T,
    /// 更新时间
    updated_at: Instant,
}

/// 处理器错误类型
#[derive(Debug, thiserror::Error)]
pub enum ProcessorError {
    #[error("缓存已满: {symbol}")]
    CacheFull {
        symbol: String,
    },
    
    #[error("缓存过期: {symbol}")]
    CacheExpired {
        symbol: String,
    },
    
    #[error("订阅失败: {symbol}, 原因: {reason}")]
    SubscriptionFailed {
        symbol: String,
        reason: String,
    },
    
    #[error("数据处理失败: {reason}")]
    ProcessingFailed {
        reason: String,
    },
}

impl From<ProcessorError> for BitfinexError {
    fn from(err: ProcessorError) -> Self {
        BitfinexError::Internal(err.to_string())
    }
}

/// 数据处理器
pub struct BitfinexProcessor {
    /// 配置信息
    config: BitfinexProcessorConfig,
    /// 数据收集器
    collector: Arc<BitfinexCollector>,
    /// Ticker缓存
    tickers: Arc<RwLock<HashMap<String, CacheItem<Ticker>>>>,
    /// 订单簿缓存
    orderbooks: Arc<RwLock<HashMap<String, CacheItem<Orderbook>>>>,
    /// 成交缓存
    trades: Arc<RwLock<HashMap<String, VecDeque<CacheItem<Trade>>>>>
    /// K线缓存
    klines: Arc<RwLock<HashMap<(String, String), VecDeque<CacheItem<Kline>>>>>
    /// 数据广播通道
    event_tx: broadcast::Sender<ProcessorEvent>,
    /// 数据广播通道
    event_rx: broadcast::Receiver<ProcessorEvent>,
    /// 数据收集器任务句柄
    collector_task: Arc<RwLock<Option<JoinHandle<()>>>>,
    /// 清理任务句柄
    cleanup_task: Arc<RwLock<Option<JoinHandle<()>>>>,
}

/// 处理器事件
#[derive(Debug, Clone)]
pub enum ProcessorEvent {
    /// Ticker更新
    TickerUpdate {
        symbol: String,
        data: Ticker,
    },
    /// 订单簿更新
    OrderbookUpdate {
        symbol: String,
        data: Orderbook,
    },
    /// 成交更新
    TradeUpdate {
        symbol: String,
        data: Trade,
    },
    /// K线更新
    KlineUpdate {
        symbol: String,
        interval: String,
        data: Kline,
    },
    /// 错误
    Error(String),
    /// 重连事件
    Reconnect {
        /// 重试次数
        attempt: u32,
        /// 下次重试时间
        next_attempt: SystemTime,
        /// 错误原因
        reason: String,
    },
    /// 重连成功
    ReconnectSuccess {
        /// 重试次数
        attempt: u32,
    },
}

#[async_trait]
impl BitfinexProcessor {
    /// 创建新的数据处理器
    pub fn new(config: BitfinexProcessorConfig) -> Result<Self> {
        let collector = Arc::new(BitfinexCollector::new(config.collector_config.clone())?);
        let (event_tx, event_rx) = broadcast::channel(1000);

        Ok(Self {
            config,
            collector,
            tickers: Arc::new(RwLock::new(HashMap::new())),
            orderbooks: Arc::new(RwLock::new(HashMap::new())),
            trades: Arc::new(RwLock::new(HashMap::new())),
            klines: Arc::new(RwLock::new(HashMap::new())),
            event_tx,
            event_rx,
            collector_task: Arc::new(RwLock::new(None)),
            cleanup_task: Arc::new(RwLock::new(None)),
        })
    }

    /// 启动数据处理
    pub async fn start(&self) -> Result<()> {
        // 启动数据收集器
        self.collector.start().await.map_err(|e| ProcessorError::ProcessingFailed {
            reason: format!("启动收集器失败: {}", e)
        })?;

        // 启动数据处理任务
        let mut collector_rx = self.collector.subscribe_updates();
        let event_tx = self.event_tx.clone();
        let tickers = self.tickers.clone();
        let orderbooks = self.orderbooks.clone();
        let trades = self.trades.clone();
        let klines = self.klines.clone();
        let config = self.config.clone();

        let collector_task = tokio::spawn(async move {
            while let Ok(event) = collector_rx.recv().await {
                let result = match event {
                    CollectorEvent::TickerUpdate(ticker) => {
                        let mut tickers = tickers.write().await;
                        if tickers.len() >= config.trades_cache_size {
                            let err = ProcessorError::CacheFull {
                                symbol: ticker.symbol.clone(),
                            };
                            let _ = event_tx.send(ProcessorEvent::Error(err.to_string()));
                            continue;
                        }
                        
                        tickers.insert(ticker.symbol.clone(), CacheItem {
                            data: ticker.clone(),
                            updated_at: Instant::now(),
                        });
                        
                        if let Err(e) = event_tx.send(ProcessorEvent::TickerUpdate {
                            symbol: ticker.symbol.clone(),
                            data: ticker,
                        }) {
                            let err = ProcessorError::ProcessingFailed {
                                reason: format!("发送Ticker更新事件失败: {}", e)
                            };
                            let _ = event_tx.send(ProcessorEvent::Error(err.to_string()));
                        }
                    }
                    CollectorEvent::OrderbookUpdate(orderbook) => {
                        let mut orderbooks = orderbooks.write().await;
                        if orderbooks.len() >= config.trades_cache_size {
                            let err = ProcessorError::CacheFull {
                                symbol: orderbook.symbol.clone(),
                            };
                            let _ = event_tx.send(ProcessorEvent::Error(err.to_string()));
                            continue;
                        }
                        
                        orderbooks.insert(orderbook.symbol.clone(), CacheItem {
                            data: orderbook.clone(),
                            updated_at: Instant::now(),
                        });
                        
                        if let Err(e) = event_tx.send(ProcessorEvent::OrderbookUpdate {
                            symbol: orderbook.symbol.clone(),
                            data: orderbook,
                        }) {
                            let err = ProcessorError::ProcessingFailed {
                                reason: format!("发送订单簿更新事件失败: {}", e)
                            };
                            let _ = event_tx.send(ProcessorEvent::Error(err.to_string()));
                        }
                    }
                    CollectorEvent::TradeUpdate(trade) => {
                        let mut trades = trades.write().await;
                        let trades_queue = trades
                            .entry(trade.symbol.clone())
                            .or_insert_with(VecDeque::new);
                        
                        trades_queue.push_back(CacheItem {
                            data: trade.clone(),
                            updated_at: Instant::now(),
                        });

                        while trades_queue.len() > config.trades_cache_size {
                            trades_queue.pop_front();
                        }
                        
                        if let Err(e) = event_tx.send(ProcessorEvent::TradeUpdate {
                            symbol: trade.symbol.clone(),
                            data: trade,
                        }) {
                            let err = ProcessorError::ProcessingFailed {
                                reason: format!("发送成交更新事件失败: {}", e)
                            };
                            let _ = event_tx.send(ProcessorEvent::Error(err.to_string()));
                        }
                    }
                    CollectorEvent::KlineUpdate(kline) => {
                        let mut klines = klines.write().await;
                        let key = (kline.symbol.clone(), kline.interval.clone());
                        let klines_queue = klines
                            .entry(key)
                            .or_insert_with(VecDeque::new);
                        
                        klines_queue.push_back(CacheItem {
                            data: kline.clone(),
                            updated_at: Instant::now(),
                        });

                        while klines_queue.len() > config.klines_cache_size {
                            klines_queue.pop_front();
                        }
                        
                        if let Err(e) = event_tx.send(ProcessorEvent::KlineUpdate {
                            symbol: kline.symbol.clone(),
                            interval: kline.interval.clone(),
                            data: kline,
                        }) {
                            let err = ProcessorError::ProcessingFailed {
                                reason: format!("发送K线更新事件失败: {}", e)
                            };
                            let _ = event_tx.send(ProcessorEvent::Error(err.to_string()));
                        }
                    }
                    CollectorEvent::Error(error) => {
                        let _ = event_tx.send(ProcessorEvent::Error(error));
                    }
                };
            }
        });

        // 启动缓存清理任务
        let cleanup_task = self.start_cleanup_task().await;

        // 保存任务句柄
        *self.collector_task.write().await = Some(collector_task);
        *self.cleanup_task.write().await = Some(cleanup_task);

        Ok(())
    }

    /// 停止数据处理
    pub async fn stop(&self) -> Result<()> {
        self.collector.stop().await?;

        // 停止处理任务
        if let Some(task) = self.collector_task.write().await.take() {
            task.abort();
        }

        // 停止清理任务
        if let Some(task) = self.cleanup_task.write().await.take() {
            task.abort();
        }

        // 清理缓存
        self.tickers.write().await.clear();
        self.orderbooks.write().await.clear();
        self.trades.write().await.clear();
        self.klines.write().await.clear();

        Ok(())
    }

    /// 订阅交易对数据
    pub async fn subscribe_ticker(&self, symbol: &str) -> Result<()> {
        self.collector.subscribe_ticker(symbol).await
    }

    /// 订阅订单簿数据
    pub async fn subscribe_orderbook(&self, symbol: &str) -> Result<()> {
        self.collector.subscribe_orderbook(symbol).await
    }

    /// 订阅成交数据
    pub async fn subscribe_trades(&self, symbol: &str) -> Result<()> {
        self.collector.subscribe_trades(symbol).await
    }

    /// 验证缓存是否有效
    async fn validate_cache<T>(&self, symbol: &str, cache_item: &CacheItem<T>) -> Result<()> {
        let age = SystemTime::now()
            .duration_since(cache_item.updated_at.into())
            .map_err(|e| ProcessorError::ProcessingFailed {
                reason: format!("时间计算错误: {}", e)
            })?;
            
        if age > self.config.ticker_cache_ttl {
            return Err(ProcessorError::CacheExpired {
                symbol: symbol.to_string(),
            }.into());
        }
        
        Ok(())
    }

    /// 获取Ticker数据
    pub async fn get_ticker(&self, symbol: &str) -> Result<Option<Ticker>> {
        let tickers = self.tickers.read().await;
        if let Some(cache_item) = tickers.get(symbol) {
            self.validate_cache(symbol, cache_item).await?;
            Ok(Some(cache_item.data.clone()))
        } else {
            Ok(None)
        }
    }

    /// 获取订单簿数据
    pub async fn get_orderbook(&self, symbol: &str) -> Result<Option<Orderbook>> {
        let orderbooks = self.orderbooks.read().await;
        if let Some(cache_item) = orderbooks.get(symbol) {
            self.validate_cache(symbol, cache_item).await?;
            Ok(Some(cache_item.data.clone()))
        } else {
            Ok(None)
        }
    }

    /// 获取最新成交数据
    pub async fn get_trades(&self, symbol: &str, limit: Option<usize>) -> Result<Vec<Trade>> {
        let trades = self.trades.read().await;
        if let Some(trades_queue) = trades.get(symbol) {
            let mut result = Vec::new();
            for entry in trades_queue.iter().rev().take(limit.unwrap_or(trades_queue.len())) {
                self.validate_cache(symbol, entry).await?;
                result.push(entry.data.clone());
            }
            Ok(result)
        } else {
            Ok(vec![])
        }
    }

    /// 获取K线数据
    pub async fn get_klines(&self, symbol: &str, interval: &str, limit: Option<usize>) -> Result<Vec<Kline>> {
        let klines = self.klines.read().await;
        if let Some(klines_queue) = klines.get(&(symbol.to_string(), interval.to_string())) {
            let mut result = Vec::new();
            for entry in klines_queue.iter().rev().take(limit.unwrap_or(klines_queue.len())) {
                self.validate_cache(symbol, entry).await?;
                result.push(entry.data.clone());
            }
            Ok(result)
        } else {
            Ok(vec![])
        }
    }

    /// 订阅数据更新
    pub fn subscribe_events(&self) -> broadcast::Receiver<ProcessorEvent> {
        self.event_rx.subscribe()
    }

    /// 启动缓存清理任务
    async fn start_cleanup_task(&self) -> JoinHandle<()> {
        let tickers = self.tickers.clone();
        let orderbooks = self.orderbooks.clone();
        let trades = self.trades.clone();
        let klines = self.klines.clone();
        let config = self.config.clone();

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(60));
            loop {
                interval.tick().await;
                let now = SystemTime::now();

                // 清理过期的Ticker数据
                let mut tickers = tickers.write().await;
                tickers.retain(|_, entry| {
                    now.duration_since(entry.updated_at.into())
                        .map(|age| age < config.ticker_cache_ttl)
                        .unwrap_or(false)
                });

                // 清理过期的Orderbook数据
                let mut orderbooks = orderbooks.write().await;
                orderbooks.retain(|_, entry| {
                    now.duration_since(entry.updated_at.into())
                        .map(|age| age < config.orderbook_cache_ttl)
                        .unwrap_or(false)
                });

                // 清理过期的Trade数据
                let mut trades = trades.write().await;
                for trades_queue in trades.values_mut() {
                    trades_queue.retain(|entry| {
                        now.duration_since(entry.updated_at.into())
                            .map(|age| age < config.trades_cache_ttl)
                            .unwrap_or(false)
                    });
                }

                // 清理过期的Kline数据
                let mut klines = klines.write().await;
                for klines_queue in klines.values_mut() {
                    klines_queue.retain(|entry| {
                        now.duration_since(entry.updated_at.into())
                            .map(|age| age < config.klines_cache_ttl)
                            .unwrap_or(false)
                    });
                }
            }
        })
    }

    /// 处理重连
    async fn handle_reconnect(&self, error: &str) -> Result<()> {
        let mut attempt = 0;
        let config = &self.config.reconnect_config;
        
        loop {
            attempt += 1;
            if config.max_retries > 0 && attempt > config.max_retries {
                return Err(ProcessorError::ProcessingFailed {
                    reason: format!("重连失败，已达到最大重试次数: {}", config.max_retries)
                }.into());
            }
            
            // 计算下次重试间隔
            let interval = (config.initial_interval.as_secs_f64() 
                * config.backoff_factor.powi(attempt as i32 - 1))
                .min(config.max_interval.as_secs_f64());
            let interval = Duration::from_secs_f64(interval);
            
            // 发送重连事件
            let next_attempt = SystemTime::now() + interval;
            let _ = self.event_tx.send(ProcessorEvent::Reconnect {
                attempt,
                next_attempt,
                reason: error.to_string(),
            });
            
            // 等待重试间隔
            tokio::time::sleep(interval).await;
            
            // 尝试重连
            match self.reconnect().await {
                Ok(_) => {
                    let _ = self.event_tx.send(ProcessorEvent::ReconnectSuccess { attempt });
                    return Ok(());
                }
                Err(e) => {
                    let _ = self.event_tx.send(ProcessorEvent::Error(
                        format!("第{}次重连失败: {}", attempt, e)
                    ));
                    continue;
                }
            }
        }
    }
    
    /// 执行重连
    async fn reconnect(&self) -> Result<()> {
        // 停止现有连接
        self.collector.stop().await?;
        
        // 清理任务
        if let Some(task) = self.collector_task.write().await.take() {
            task.abort();
        }
        
        // 重新启动收集器
        self.collector.start().await?;
        
        // 重新订阅之前的交易对
        let tickers = self.tickers.read().await;
        for symbol in tickers.keys() {
            self.collector.subscribe_ticker(symbol).await?;
        }
        
        let orderbooks = self.orderbooks.read().await;
        for symbol in orderbooks.keys() {
            self.collector.subscribe_orderbook(symbol).await?;
        }
        
        let trades = self.trades.read().await;
        for symbol in trades.keys() {
            self.collector.subscribe_trades(symbol).await?;
        }
        
        // 重新启动处理任务
        let collector_task = self.start_collector_task().await;
        *self.collector_task.write().await = Some(collector_task);
        
        Ok(())
    }
    
    /// 启动收集器任务
    async fn start_collector_task(&self) -> JoinHandle<()> {
        let mut collector_rx = self.collector.subscribe_updates();
        let event_tx = self.event_tx.clone();
        let tickers = self.tickers.clone();
        let orderbooks = self.orderbooks.clone();
        let trades = self.trades.clone();
        let klines = self.klines.clone();
        let config = self.config.clone();
        let processor = self.clone();

        tokio::spawn(async move {
            while let Ok(event) = collector_rx.recv().await {
                match event {
                    CollectorEvent::Error(error) => {
                        let _ = event_tx.send(ProcessorEvent::Error(error.clone()));
                        if let Err(e) = processor.handle_reconnect(&error).await {
                            let _ = event_tx.send(ProcessorEvent::Error(e.to_string()));
                            break;
                        }
                    }
                    CollectorEvent::TickerUpdate(ticker) => {
                        let mut tickers = tickers.write().await;
                        if tickers.len() >= config.trades_cache_size {
                            let err = ProcessorError::CacheFull {
                                symbol: ticker.symbol.clone(),
                            };
                            let _ = event_tx.send(ProcessorEvent::Error(err.to_string()));
                            continue;
                        }
                        
                        tickers.insert(ticker.symbol.clone(), CacheItem {
                            data: ticker.clone(),
                            updated_at: Instant::now(),
                        });
                        
                        if let Err(e) = event_tx.send(ProcessorEvent::TickerUpdate {
                            symbol: ticker.symbol.clone(),
                            data: ticker,
                        }) {
                            let err = ProcessorError::ProcessingFailed {
                                reason: format!("发送Ticker更新事件失败: {}", e)
                            };
                            let _ = event_tx.send(ProcessorEvent::Error(err.to_string()));
                        }
                    }
                    CollectorEvent::OrderbookUpdate(orderbook) => {
                        let mut orderbooks = orderbooks.write().await;
                        if orderbooks.len() >= config.trades_cache_size {
                            let err = ProcessorError::CacheFull {
                                symbol: orderbook.symbol.clone(),
                            };
                            let _ = event_tx.send(ProcessorEvent::Error(err.to_string()));
                            continue;
                        }
                        
                        orderbooks.insert(orderbook.symbol.clone(), CacheItem {
                            data: orderbook.clone(),
                            updated_at: Instant::now(),
                        });
                        
                        if let Err(e) = event_tx.send(ProcessorEvent::OrderbookUpdate {
                            symbol: orderbook.symbol.clone(),
                            data: orderbook,
                        }) {
                            let err = ProcessorError::ProcessingFailed {
                                reason: format!("发送订单簿更新事件失败: {}", e)
                            };
                            let _ = event_tx.send(ProcessorEvent::Error(err.to_string()));
                        }
                    }
                    CollectorEvent::TradeUpdate(trade) => {
                        let mut trades = trades.write().await;
                        let trades_queue = trades
                            .entry(trade.symbol.clone())
                            .or_insert_with(VecDeque::new);
                        
                        trades_queue.push_back(CacheItem {
                            data: trade.clone(),
                            updated_at: Instant::now(),
                        });

                        while trades_queue.len() > config.trades_cache_size {
                            trades_queue.pop_front();
                        }
                        
                        if let Err(e) = event_tx.send(ProcessorEvent::TradeUpdate {
                            symbol: trade.symbol.clone(),
                            data: trade,
                        }) {
                            let err = ProcessorError::ProcessingFailed {
                                reason: format!("发送成交更新事件失败: {}", e)
                            };
                            let _ = event_tx.send(ProcessorEvent::Error(err.to_string()));
                        }
                    }
                    CollectorEvent::KlineUpdate(kline) => {
                        let mut klines = klines.write().await;
                        let key = (kline.symbol.clone(), kline.interval.clone());
                        let klines_queue = klines
                            .entry(key)
                            .or_insert_with(VecDeque::new);
                        
                        klines_queue.push_back(CacheItem {
                            data: kline.clone(),
                            updated_at: Instant::now(),
                        });

                        while klines_queue.len() > config.klines_cache_size {
                            klines_queue.pop_front();
                        }
                        
                        if let Err(e) = event_tx.send(ProcessorEvent::KlineUpdate {
                            symbol: kline.symbol.clone(),
                            interval: kline.interval.clone(),
                            data: kline,
                        }) {
                            let err = ProcessorError::ProcessingFailed {
                                reason: format!("发送K线更新事件失败: {}", e)
                            };
                            let _ = event_tx.send(ProcessorEvent::Error(err.to_string()));
                        }
                    }
                }
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio;

    #[tokio::test]
    async fn test_processor() -> Result<()> {
        let config = BitfinexProcessorConfig::default();
        let processor = BitfinexProcessor::new(config)?;
        
        // 启动处理器
        processor.start().await?;

        // 订阅事件
        let mut events = processor.subscribe_events();

        // 订阅BTCUSD的Ticker
        processor.subscribe_ticker("BTCUSD").await?;

        // 等待并处理一些事件
        let event_handle = tokio::spawn(async move {
            while let Ok(event) = events.recv().await {
                match event {
                    ProcessorEvent::TickerUpdate { symbol, data } => {
                        println!("收到处理后的Ticker数据: {} {:?}", symbol, data);
                        break;
                    }
                    ProcessorEvent::Error(err) => {
                        eprintln!("错误: {}", err);
                        break;
                    }
                    _ => {}
                }
            }
        });

        // 等待一段时间
        tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;

        // 获取缓存的数据
        if let Some(ticker) = processor.get_ticker("BTCUSD").await? {
            println!("缓存的Ticker数据: {} {:?}", "BTCUSD", ticker);
        }

        // 测试缓存验证
        {
            let mut tickers = processor.tickers.write().await;
            if let Some(entry) = tickers.get_mut("BTCUSD") {
                // 模拟过期的缓存
                entry.updated_at = Instant::now() - Duration::from_secs(3600);
            }
        }

        // 验证过期的缓存会返回错误
        match processor.get_ticker("BTCUSD").await {
            Ok(_) => panic!("应该返回缓存过期错误"),
            Err(e) => {
                assert!(matches!(e, BitfinexError::Internal(_)));
                println!("正确处理了缓存过期: {}", e);
            }
        }

        // 停止处理器
        processor.stop().await?;

        Ok(())
    }

    #[tokio::test]
    async fn test_cache_full() -> Result<()> {
        let mut config = BitfinexProcessorConfig::default();
        config.trades_cache_size = 1;
        let processor = BitfinexProcessor::new(config)?;
        
        // 启动处理器
        processor.start().await?;

        // 订阅多个交易对，触发缓存满的情况
        processor.subscribe_ticker("BTCUSD").await?;
        processor.subscribe_ticker("ETHUSD").await?;

        // 等待一段时间
        tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;

        // 停止处理器
        processor.stop().await?;

        Ok(())
    }

    #[tokio::test]
    async fn test_reconnect() -> Result<()> {
        let mut config = BitfinexProcessorConfig::default();
        config.reconnect_config.max_retries = 3;
        config.reconnect_config.initial_interval = Duration::from_millis(100);
        config.reconnect_config.max_interval = Duration::from_secs(1);
        
        let processor = BitfinexProcessor::new(config)?;
        
        // 启动处理器
        processor.start().await?;

        // 订阅事件
        let mut events = processor.subscribe_events();

        // 订阅BTCUSD的Ticker
        processor.subscribe_ticker("BTCUSD").await?;

        // 等待并处理事件
        let event_handle = tokio::spawn(async move {
            let mut reconnect_attempts = 0;
            
            while let Ok(event) = events.recv().await {
                match event {
                    ProcessorEvent::Reconnect { attempt, .. } => {
                        println!("收到重连事件: 第{}次尝试", attempt);
                        reconnect_attempts += 1;
                    }
                    ProcessorEvent::ReconnectSuccess { attempt } => {
                        println!("重连成功: 第{}次尝试", attempt);
                        break;
                    }
                    ProcessorEvent::Error(err) => {
                        println!("错误: {}", err);
                    }
                    _ => {}
                }
            }
            
            assert!(reconnect_attempts > 0, "应该有重连尝试");
        });

        // 等待事件处理完成
        tokio::time::sleep(Duration::from_secs(5)).await;
        
        // 停止处理器
        processor.stop().await?;

        Ok(())
    }
}
