"""
数据服务主应用
"""
import asyncio
from contextlib import asynccontextmanager
from typing import List

import prometheus_client
import uvicorn
from fastapi import FastAPI, HTTPException
from prometheus_client import start_http_server

from app.core.config import settings
from app.core.logging import logger, setup_logging
from app.models.market_data import Interval, Kline
from app.services.binance_service import BinanceService
from app.services.kafka_producer import KafkaProducerService


@asynccontextmanager
async def lifespan(app: FastAPI):
    """
    应用生命周期管理
    """
    # 设置日志
    setup_logging()
    logger.info("application_starting")

    # 启动 Prometheus 指标服务器
    if settings.ENABLE_METRICS:
        start_http_server(settings.METRICS_PORT)
        logger.info("metrics_server_started", port=settings.METRICS_PORT)

    # 初始化 Kafka 生产者
    kafka_producer = KafkaProducerService()
    kafka_producer.initialize()
    app.state.kafka_producer = kafka_producer

    # 初始化 Binance 服务
    binance_service = BinanceService(kafka_producer=kafka_producer)
    await binance_service.initialize()
    app.state.binance_service = binance_service

    # 启动市场数据流
    market_data_task = asyncio.create_task(
        binance_service.start_market_data_stream()
    )
    app.state.market_data_task = market_data_task

    logger.info("application_started")
    yield

    # 清理资源
    logger.info("application_stopping")
    market_data_task.cancel()
    await binance_service.close()
    kafka_producer.close()
    logger.info("application_stopped")


app = FastAPI(
    title="HermesFlow Data Service",
    description="数据服务 API",
    version="0.1.0",
    lifespan=lifespan,
)


@app.get("/health")
async def health_check():
    """健康检查接口"""
    return {"status": "ok"}


@app.get("/api/v1/klines/{symbol}")
async def get_klines(
    symbol: str,
    interval: Interval,
    limit: int = 500,
) -> List[Kline]:
    """获取K线数据"""
    try:
        klines = await app.state.binance_service.get_historical_klines(
            symbol=symbol,
            interval=interval,
            limit=limit,
        )
        return klines
    except Exception as e:
        logger.error(
            "get_klines_error",
            symbol=symbol,
            interval=interval,
            error=str(e),
        )
        raise HTTPException(
            status_code=500,
            detail=f"Failed to get klines: {str(e)}",
        )


if __name__ == "__main__":
    uvicorn.run(
        "app.main:app",
        host=settings.HOST,
        port=settings.PORT,
        reload=settings.DEBUG,
    ) 