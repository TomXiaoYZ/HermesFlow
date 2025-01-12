"""
配置数据模型
"""
from datetime import datetime
from decimal import Decimal
from typing import Dict, List, Optional, Union

from pydantic import BaseModel, Field

from app.models.market_data import Exchange


class ApiKey(BaseModel):
    """API密钥配置"""
    exchange: Exchange
    name: str = Field(..., description="密钥名称")
    api_key: str = Field(..., description="API Key")
    api_secret: str = Field(..., description="API Secret")
    passphrase: Optional[str] = Field(None, description="API密码(部分交易所需要)")
    is_test: bool = Field(False, description="是否为测试网络")


class ExchangeConfig(BaseModel):
    """交易所配置"""
    exchange: Exchange
    ws_url: Optional[str] = Field(None, description="WebSocket URL")
    rest_url: Optional[str] = Field(None, description="REST API URL")
    rate_limit_per_second: int = Field(10, description="每秒请求限制")


class TradingPairConfig(BaseModel):
    """交易对配置"""
    exchange: Exchange
    symbol: str = Field(..., description="交易对符号")
    base_asset: str = Field(..., description="基础资产")
    quote_asset: str = Field(..., description="计价资产")
    price_precision: int = Field(..., description="价格精度")
    volume_precision: int = Field(..., description="数量精度")
    min_price: Optional[Decimal] = Field(None, description="最小价格")
    max_price: Optional[Decimal] = Field(None, description="最大价格")
    min_volume: Optional[Decimal] = Field(None, description="最小数量")
    max_volume: Optional[Decimal] = Field(None, description="最大数量")
    min_notional: Optional[Decimal] = Field(None, description="最小交易额")


class StrategyConfig(BaseModel):
    """策略配置"""
    name: str = Field(..., description="策略名称")
    description: Optional[str] = Field(None, description="策略描述")
    parameters: Dict = Field(..., description="策略参数")
    is_active: bool = Field(True, description="是否激活")


class SystemConfig(BaseModel):
    """系统配置"""
    key: str = Field(..., description="配置键")
    value: Union[str, int, float, bool, Dict, List] = Field(..., description="配置值")
    description: Optional[str] = Field(None, description="配置描述") 