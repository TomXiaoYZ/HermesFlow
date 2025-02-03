use crate::{
    error::UniswapV3Error,
    models::{Pool, SwapEvent, TickData},
    metrics::MetricsCollector,
};

use async_trait::async_trait;
use chrono::{DateTime, Duration, Utc};
use lru::LruCache;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    num::NonZeroUsize,
    sync::{Arc, RwLock},
};
use tokio::sync::RwLock as AsyncRwLock;
use tracing::{debug, error, info};

/// 缓存条目
#[derive(Clone, Debug, Serialize, Deserialize)]
struct CacheEntry<T> {
    data: T,
    timestamp: DateTime<Utc>,
    ttl: Duration,
}

impl<T> CacheEntry<T> {
    fn new(data: T, ttl: Duration) -> Self {
        Self {
            data,
            timestamp: Utc::now(),
            ttl,
        }
    }

    fn is_expired(&self) -> bool {
        Utc::now() - self.timestamp > self.ttl
    }
}

/// 缓存配置
#[derive(Clone, Debug)]
pub struct CacheConfig {
    /// 池子数据缓存时间 (秒)
    pub pool_ttl: i64,
    /// 流动性数据缓存时间 (秒)
    pub liquidity_ttl: i64,
    /// 交易数据缓存时间 (秒)
    pub swap_ttl: i64,
    /// 内存缓存容量
    pub memory_cache_size: usize,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            pool_ttl: 60,         // 1分钟
            liquidity_ttl: 30,    // 30秒
            swap_ttl: 15,         // 15秒
            memory_cache_size: 1000,
        }
    }
}

/// 缓存接口
#[async_trait]
pub trait Cache: Send + Sync {
    /// 获取池子数据
    async fn get_pool(&self, address: &str) -> Option<Pool>;
    /// 设置池子数据
    async fn set_pool(&self, address: &str, pool: Pool);
    /// 获取流动性分布
    async fn get_liquidity(&self, address: &str) -> Option<Vec<TickData>>;
    /// 设置流动性分布
    async fn set_liquidity(&self, address: &str, ticks: Vec<TickData>);
    /// 获取最近交易
    async fn get_swaps(&self, address: &str) -> Option<Vec<SwapEvent>>;
    /// 设置最近交易
    async fn set_swaps(&self, address: &str, swaps: Vec<SwapEvent>);
    /// 清理过期数据
    async fn cleanup(&self);
}

/// 内存缓存实现
pub struct MemoryCache {
    config: CacheConfig,
    pools: AsyncRwLock<LruCache<String, CacheEntry<Pool>>>,
    liquidity: AsyncRwLock<LruCache<String, CacheEntry<Vec<TickData>>>>,
    swaps: AsyncRwLock<LruCache<String, CacheEntry<Vec<SwapEvent>>>>,
}

impl MemoryCache {
    pub fn new(config: CacheConfig) -> Self {
        Self {
            pools: AsyncRwLock::new(LruCache::new(NonZeroUsize::new(config.memory_cache_size).unwrap())),
            liquidity: AsyncRwLock::new(LruCache::new(NonZeroUsize::new(config.memory_cache_size).unwrap())),
            swaps: AsyncRwLock::new(LruCache::new(NonZeroUsize::new(config.memory_cache_size).unwrap())),
            config,
        }
    }
}

#[async_trait]
impl Cache for MemoryCache {
    async fn get_pool(&self, address: &str) -> Option<Pool> {
        let mut cache = self.pools.write().await;
        if let Some(entry) = cache.get(address) {
            if !entry.is_expired() {
                debug!("命中池子缓存: {}", address);
                MetricsCollector::record_cache_hit("pool");
                return Some(entry.data.clone());
            }
            debug!("池子缓存已过期: {}", address);
            cache.pop(address);
        }
        MetricsCollector::record_cache_miss("pool");
        None
    }

    async fn set_pool(&self, address: &str, pool: Pool) {
        let mut cache = self.pools.write().await;
        let entry = CacheEntry::new(pool, Duration::seconds(self.config.pool_ttl));
        cache.put(address.to_string(), entry);
        debug!("更新池子缓存: {}", address);
    }

    async fn get_liquidity(&self, address: &str) -> Option<Vec<TickData>> {
        let mut cache = self.liquidity.write().await;
        if let Some(entry) = cache.get(address) {
            if !entry.is_expired() {
                debug!("命中流动性缓存: {}", address);
                MetricsCollector::record_cache_hit("liquidity");
                return Some(entry.data.clone());
            }
            debug!("流动性缓存已过期: {}", address);
            cache.pop(address);
        }
        MetricsCollector::record_cache_miss("liquidity");
        None
    }

    async fn set_liquidity(&self, address: &str, ticks: Vec<TickData>) {
        let mut cache = self.liquidity.write().await;
        let entry = CacheEntry::new(ticks, Duration::seconds(self.config.liquidity_ttl));
        cache.put(address.to_string(), entry);
        debug!("更新流动性缓存: {}", address);
    }

    async fn get_swaps(&self, address: &str) -> Option<Vec<SwapEvent>> {
        let mut cache = self.swaps.write().await;
        if let Some(entry) = cache.get(address) {
            if !entry.is_expired() {
                debug!("命中交易缓存: {}", address);
                MetricsCollector::record_cache_hit("swaps");
                return Some(entry.data.clone());
            }
            debug!("交易缓存已过期: {}", address);
            cache.pop(address);
        }
        MetricsCollector::record_cache_miss("swaps");
        None
    }

    async fn set_swaps(&self, address: &str, swaps: Vec<SwapEvent>) {
        let mut cache = self.swaps.write().await;
        let entry = CacheEntry::new(swaps, Duration::seconds(self.config.swap_ttl));
        cache.put(address.to_string(), entry);
        debug!("更新交易缓存: {}", address);
    }

    async fn cleanup(&self) {
        let mut pools = self.pools.write().await;
        let mut liquidity = self.liquidity.write().await;
        let mut swaps = self.swaps.write().await;

        // 清理过期的池子数据
        pools.iter().filter(|(_, entry)| entry.is_expired())
            .map(|(k, _)| k.clone())
            .collect::<Vec<_>>()
            .iter()
            .for_each(|k| {
                pools.pop(k);
                debug!("清理过期池子缓存: {}", k);
            });

        // 清理过期的流动性数据
        liquidity.iter().filter(|(_, entry)| entry.is_expired())
            .map(|(k, _)| k.clone())
            .collect::<Vec<_>>()
            .iter()
            .for_each(|k| {
                liquidity.pop(k);
                debug!("清理过期流动性缓存: {}", k);
            });

        // 清理过期的交易数据
        swaps.iter().filter(|(_, entry)| entry.is_expired())
            .map(|(k, _)| k.clone())
            .collect::<Vec<_>>()
            .iter()
            .for_each(|k| {
                swaps.pop(k);
                debug!("清理过期交易缓存: {}", k);
            });
    }
}

/// 持久化存储实现
pub struct PersistentCache {
    // TODO: 实现持久化存储
    // 可以使用 SQLite、RocksDB 等
}

/// 分层缓存
pub struct LayeredCache {
    memory: Arc<MemoryCache>,
    persistent: Option<Arc<PersistentCache>>,
}

impl LayeredCache {
    pub fn new(config: CacheConfig) -> Self {
        Self {
            memory: Arc::new(MemoryCache::new(config)),
            persistent: None, // TODO: 添加持久化存储
        }
    }

    /// 启动缓存清理任务
    pub async fn start_cleanup_task(cache: Arc<Self>) {
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(60));
            loop {
                interval.tick().await;
                cache.memory.cleanup().await;
                if let Some(persistent) = &cache.persistent {
                    persistent.cleanup().await;
                }
            }
        });
    }
} 