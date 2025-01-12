"""
PostgreSQL 配置数据存储服务
"""
import json
from datetime import datetime
from typing import Dict, List, Optional, Union

import asyncpg
from asyncpg import Connection, Pool

from app.core.config import settings
from app.core.logging import logger
from app.core.metrics import DATA_PROCESSING_LATENCY
from app.models.config_data import (
    ApiKey,
    ExchangeConfig,
    StrategyConfig,
    SystemConfig,
    TradingPairConfig,
)


class PostgresqlService:
    """PostgreSQL 服务类"""

    def __init__(self) -> None:
        """初始化服务"""
        self.pool: Optional[Pool] = None

    async def initialize(self) -> None:
        """初始化数据库连接池"""
        try:
            self.pool = await asyncpg.create_pool(
                host=settings.POSTGRESQL_HOST,
                port=settings.POSTGRESQL_PORT,
                user=settings.POSTGRESQL_USER,
                password=settings.POSTGRESQL_PASSWORD,
                database=settings.POSTGRESQL_DB,
                min_size=5,
                max_size=20
            )
            # 创建数据表
            async with self.pool.acquire() as conn:
                await self._create_tables(conn)
            logger.info("postgresql_service_initialized")
        except Exception as e:
            logger.error(
                "postgresql_service_initialization_failed",
                error=str(e)
            )
            raise

    async def close(self) -> None:
        """关闭连接池"""
        if self.pool:
            await self.pool.close()
        logger.info("postgresql_service_closed")

    async def _create_tables(self, conn: Connection) -> None:
        """创建数据表"""
        # API密钥表
        await conn.execute("""
            CREATE TABLE IF NOT EXISTS api_keys (
                id SERIAL PRIMARY KEY,
                exchange VARCHAR(50) NOT NULL,
                name VARCHAR(100) NOT NULL,
                api_key VARCHAR(200) NOT NULL,
                api_secret VARCHAR(200) NOT NULL,
                passphrase VARCHAR(200),
                is_test BOOLEAN DEFAULT false,
                created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
                updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
                UNIQUE(exchange, name)
            )
        """)

        # 交易所配置表
        await conn.execute("""
            CREATE TABLE IF NOT EXISTS exchange_configs (
                id SERIAL PRIMARY KEY,
                exchange VARCHAR(50) NOT NULL,
                ws_url VARCHAR(200),
                rest_url VARCHAR(200),
                rate_limit_per_second INTEGER DEFAULT 10,
                created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
                updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
                UNIQUE(exchange)
            )
        """)

        # 交易对配置表
        await conn.execute("""
            CREATE TABLE IF NOT EXISTS trading_pair_configs (
                id SERIAL PRIMARY KEY,
                exchange VARCHAR(50) NOT NULL,
                symbol VARCHAR(50) NOT NULL,
                base_asset VARCHAR(20) NOT NULL,
                quote_asset VARCHAR(20) NOT NULL,
                price_precision INTEGER NOT NULL,
                volume_precision INTEGER NOT NULL,
                min_price DECIMAL(30, 8),
                max_price DECIMAL(30, 8),
                min_volume DECIMAL(30, 8),
                max_volume DECIMAL(30, 8),
                min_notional DECIMAL(30, 8),
                created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
                updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
                UNIQUE(exchange, symbol)
            )
        """)

        # 策略配置表
        await conn.execute("""
            CREATE TABLE IF NOT EXISTS strategy_configs (
                id SERIAL PRIMARY KEY,
                name VARCHAR(100) NOT NULL,
                description TEXT,
                parameters JSONB NOT NULL,
                is_active BOOLEAN DEFAULT true,
                created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
                updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
                UNIQUE(name)
            )
        """)

        # 系统配置表
        await conn.execute("""
            CREATE TABLE IF NOT EXISTS system_configs (
                id SERIAL PRIMARY KEY,
                key VARCHAR(100) NOT NULL,
                value JSONB NOT NULL,
                description TEXT,
                created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
                updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
                UNIQUE(key)
            )
        """)

    async def save_api_key(self, api_key: ApiKey) -> None:
        """保存API密钥"""
        if not self.pool:
            raise Exception("Service not initialized")

        async with self.pool.acquire() as conn:
            await conn.execute("""
                INSERT INTO api_keys (
                    exchange, name, api_key, api_secret, passphrase, is_test
                ) VALUES ($1, $2, $3, $4, $5, $6)
                ON CONFLICT (exchange, name) DO UPDATE SET
                    api_key = EXCLUDED.api_key,
                    api_secret = EXCLUDED.api_secret,
                    passphrase = EXCLUDED.passphrase,
                    is_test = EXCLUDED.is_test,
                    updated_at = CURRENT_TIMESTAMP
            """, api_key.exchange.value, api_key.name, api_key.api_key,
                api_key.api_secret, api_key.passphrase, api_key.is_test)

    async def get_api_key(
        self,
        exchange: str,
        name: str
    ) -> Optional[ApiKey]:
        """获取API密钥"""
        if not self.pool:
            raise Exception("Service not initialized")

        async with self.pool.acquire() as conn:
            row = await conn.fetchrow("""
                SELECT * FROM api_keys
                WHERE exchange = $1 AND name = $2
            """, exchange, name)

            if not row:
                return None

            return ApiKey(
                exchange=row["exchange"],
                name=row["name"],
                api_key=row["api_key"],
                api_secret=row["api_secret"],
                passphrase=row["passphrase"],
                is_test=row["is_test"]
            )

    async def list_api_keys(
        self,
        exchange: Optional[str] = None
    ) -> List[ApiKey]:
        """获取API密钥列表"""
        if not self.pool:
            raise Exception("Service not initialized")

        async with self.pool.acquire() as conn:
            if exchange:
                rows = await conn.fetch("""
                    SELECT * FROM api_keys
                    WHERE exchange = $1
                    ORDER BY created_at DESC
                """, exchange)
            else:
                rows = await conn.fetch("""
                    SELECT * FROM api_keys
                    ORDER BY created_at DESC
                """)

            return [
                ApiKey(
                    exchange=row["exchange"],
                    name=row["name"],
                    api_key=row["api_key"],
                    api_secret=row["api_secret"],
                    passphrase=row["passphrase"],
                    is_test=row["is_test"]
                )
                for row in rows
            ]

    async def delete_api_key(
        self,
        exchange: str,
        name: str
    ) -> bool:
        """删除API密钥"""
        if not self.pool:
            raise Exception("Service not initialized")

        async with self.pool.acquire() as conn:
            result = await conn.execute("""
                DELETE FROM api_keys
                WHERE exchange = $1 AND name = $2
            """, exchange, name)
            return result == "DELETE 1"

    async def save_exchange_config(
        self,
        config: ExchangeConfig
    ) -> None:
        """保存交易所配置"""
        if not self.pool:
            raise Exception("Service not initialized")

        async with self.pool.acquire() as conn:
            await conn.execute("""
                INSERT INTO exchange_configs (
                    exchange, ws_url, rest_url, rate_limit_per_second
                ) VALUES ($1, $2, $3, $4)
                ON CONFLICT (exchange) DO UPDATE SET
                    ws_url = EXCLUDED.ws_url,
                    rest_url = EXCLUDED.rest_url,
                    rate_limit_per_second = EXCLUDED.rate_limit_per_second,
                    updated_at = CURRENT_TIMESTAMP
            """, config.exchange.value, config.ws_url, config.rest_url,
                config.rate_limit_per_second)

    async def get_exchange_config(
        self,
        exchange: str
    ) -> Optional[ExchangeConfig]:
        """获取交易所配置"""
        if not self.pool:
            raise Exception("Service not initialized")

        async with self.pool.acquire() as conn:
            row = await conn.fetchrow("""
                SELECT * FROM exchange_configs
                WHERE exchange = $1
            """, exchange)

            if not row:
                return None

            return ExchangeConfig(
                exchange=row["exchange"],
                ws_url=row["ws_url"],
                rest_url=row["rest_url"],
                rate_limit_per_second=row["rate_limit_per_second"]
            )

    async def save_trading_pair_config(
        self,
        config: TradingPairConfig
    ) -> None:
        """保存交易对配置"""
        if not self.pool:
            raise Exception("Service not initialized")

        async with self.pool.acquire() as conn:
            await conn.execute("""
                INSERT INTO trading_pair_configs (
                    exchange, symbol, base_asset, quote_asset,
                    price_precision, volume_precision,
                    min_price, max_price, min_volume, max_volume,
                    min_notional
                ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
                ON CONFLICT (exchange, symbol) DO UPDATE SET
                    base_asset = EXCLUDED.base_asset,
                    quote_asset = EXCLUDED.quote_asset,
                    price_precision = EXCLUDED.price_precision,
                    volume_precision = EXCLUDED.volume_precision,
                    min_price = EXCLUDED.min_price,
                    max_price = EXCLUDED.max_price,
                    min_volume = EXCLUDED.min_volume,
                    max_volume = EXCLUDED.max_volume,
                    min_notional = EXCLUDED.min_notional,
                    updated_at = CURRENT_TIMESTAMP
            """, config.exchange.value, config.symbol, config.base_asset,
                config.quote_asset, config.price_precision,
                config.volume_precision, config.min_price, config.max_price,
                config.min_volume, config.max_volume, config.min_notional)

    async def get_trading_pair_config(
        self,
        exchange: str,
        symbol: str
    ) -> Optional[TradingPairConfig]:
        """获取交易对配置"""
        if not self.pool:
            raise Exception("Service not initialized")

        async with self.pool.acquire() as conn:
            row = await conn.fetchrow("""
                SELECT * FROM trading_pair_configs
                WHERE exchange = $1 AND symbol = $2
            """, exchange, symbol)

            if not row:
                return None

            return TradingPairConfig(
                exchange=row["exchange"],
                symbol=row["symbol"],
                base_asset=row["base_asset"],
                quote_asset=row["quote_asset"],
                price_precision=row["price_precision"],
                volume_precision=row["volume_precision"],
                min_price=row["min_price"],
                max_price=row["max_price"],
                min_volume=row["min_volume"],
                max_volume=row["max_volume"],
                min_notional=row["min_notional"]
            )

    async def list_trading_pair_configs(
        self,
        exchange: Optional[str] = None
    ) -> List[TradingPairConfig]:
        """获取交易对配置列表"""
        if not self.pool:
            raise Exception("Service not initialized")

        async with self.pool.acquire() as conn:
            if exchange:
                rows = await conn.fetch("""
                    SELECT * FROM trading_pair_configs
                    WHERE exchange = $1
                    ORDER BY created_at DESC
                """, exchange)
            else:
                rows = await conn.fetch("""
                    SELECT * FROM trading_pair_configs
                    ORDER BY created_at DESC
                """)

            return [
                TradingPairConfig(
                    exchange=row["exchange"],
                    symbol=row["symbol"],
                    base_asset=row["base_asset"],
                    quote_asset=row["quote_asset"],
                    price_precision=row["price_precision"],
                    volume_precision=row["volume_precision"],
                    min_price=row["min_price"],
                    max_price=row["max_price"],
                    min_volume=row["min_volume"],
                    max_volume=row["max_volume"],
                    min_notional=row["min_notional"]
                )
                for row in rows
            ]

    async def save_strategy_config(
        self,
        config: StrategyConfig
    ) -> None:
        """保存策略配置"""
        if not self.pool:
            raise Exception("Service not initialized")

        async with self.pool.acquire() as conn:
            await conn.execute("""
                INSERT INTO strategy_configs (
                    name, description, parameters, is_active
                ) VALUES ($1, $2, $3, $4)
                ON CONFLICT (name) DO UPDATE SET
                    description = EXCLUDED.description,
                    parameters = EXCLUDED.parameters,
                    is_active = EXCLUDED.is_active,
                    updated_at = CURRENT_TIMESTAMP
            """, config.name, config.description,
                json.dumps(config.parameters), config.is_active)

    async def get_strategy_config(
        self,
        name: str
    ) -> Optional[StrategyConfig]:
        """获取策略配置"""
        if not self.pool:
            raise Exception("Service not initialized")

        async with self.pool.acquire() as conn:
            row = await conn.fetchrow("""
                SELECT * FROM strategy_configs
                WHERE name = $1
            """, name)

            if not row:
                return None

            return StrategyConfig(
                name=row["name"],
                description=row["description"],
                parameters=json.loads(row["parameters"]),
                is_active=row["is_active"]
            )

    async def list_strategy_configs(
        self,
        active_only: bool = False
    ) -> List[StrategyConfig]:
        """获取策略配置列表"""
        if not self.pool:
            raise Exception("Service not initialized")

        async with self.pool.acquire() as conn:
            if active_only:
                rows = await conn.fetch("""
                    SELECT * FROM strategy_configs
                    WHERE is_active = true
                    ORDER BY created_at DESC
                """)
            else:
                rows = await conn.fetch("""
                    SELECT * FROM strategy_configs
                    ORDER BY created_at DESC
                """)

            return [
                StrategyConfig(
                    name=row["name"],
                    description=row["description"],
                    parameters=json.loads(row["parameters"]),
                    is_active=row["is_active"]
                )
                for row in rows
            ]

    async def save_system_config(
        self,
        key: str,
        value: Union[str, int, float, bool, dict, list],
        description: Optional[str] = None
    ) -> None:
        """保存系统配置"""
        if not self.pool:
            raise Exception("Service not initialized")

        async with self.pool.acquire() as conn:
            await conn.execute("""
                INSERT INTO system_configs (
                    key, value, description
                ) VALUES ($1, $2, $3)
                ON CONFLICT (key) DO UPDATE SET
                    value = EXCLUDED.value,
                    description = EXCLUDED.description,
                    updated_at = CURRENT_TIMESTAMP
            """, key, json.dumps(value), description)

    async def get_system_config(
        self,
        key: str
    ) -> Optional[SystemConfig]:
        """获取系统配置"""
        if not self.pool:
            raise Exception("Service not initialized")

        async with self.pool.acquire() as conn:
            row = await conn.fetchrow("""
                SELECT * FROM system_configs
                WHERE key = $1
            """, key)

            if not row:
                return None

            return SystemConfig(
                key=row["key"],
                value=json.loads(row["value"]),
                description=row["description"]
            )

    async def list_system_configs(self) -> List[SystemConfig]:
        """获取系统配置列表"""
        if not self.pool:
            raise Exception("Service not initialized")

        async with self.pool.acquire() as conn:
            rows = await conn.fetch("""
                SELECT * FROM system_configs
                ORDER BY created_at DESC
            """)

            return [
                SystemConfig(
                    key=row["key"],
                    value=json.loads(row["value"]),
                    description=row["description"]
                )
                for row in rows
            ] 