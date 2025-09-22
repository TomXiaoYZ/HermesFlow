#!/usr/bin/env python3
"""
数据处理器模块 (Data Processor Module)

负责实时数据的处理、验证和标准化，包括：
- 数据格式标准化和转换
- 数据质量验证和过滤
- 异常数据检测和处理
- 数据完整性校验
- 性能优化的批量处理

支持多种数据源格式，输出统一的标准化数据
"""

import asyncio
import time
import logging
from typing import Dict, List, Optional, Any, Callable, Tuple
from decimal import Decimal, InvalidOperation
from dataclasses import dataclass
from enum import Enum
import json
import statistics
from collections import deque, defaultdict

from .models import (
    StreamData, StreamDataType, DataStatus, QualityLevel,
    MarketData, OrderBookData, TradeData, DataQuality,
    normalize_decimal_precision, normalize_timestamp,
    validate_symbol_format, create_stream_data,
    DATA_VALIDATION_RULES
)
from .config import SubscriptionConfig

# 设置日志
logger = logging.getLogger(__name__)

class ValidationResult(Enum):
    """验证结果枚举"""
    VALID = "valid"                     # 有效
    INVALID = "invalid"                 # 无效
    WARNING = "warning"                 # 警告
    CORRECTED = "corrected"             # 已修正

@dataclass
class ProcessingResult:
    """处理结果类"""
    success: bool
    stream_data: Optional[StreamData] = None
    validation_result: ValidationResult = ValidationResult.VALID
    warnings: List[str] = None
    errors: List[str] = None
    processing_time_ms: float = 0.0
    
    def __post_init__(self):
        if self.warnings is None:
            self.warnings = []
        if self.errors is None:
            self.errors = []

class DataValidator:
    """数据验证器"""
    
    def __init__(self, config: SubscriptionConfig):
        self.config = config
        self.validation_rules = DATA_VALIDATION_RULES
        
        # 验证统计
        self.validation_stats = {
            'total_validations': 0,
            'valid_count': 0,
            'invalid_count': 0,
            'warning_count': 0,
            'corrected_count': 0
        }
        
        # 价格历史记录用于异常检测
        self.price_history: Dict[str, deque] = defaultdict(lambda: deque(maxlen=100))
        self.last_prices: Dict[str, Decimal] = {}
        
        logger.info("数据验证器初始化完成")
    
    def validate_basic_fields(self, data: Dict[str, Any], data_type: StreamDataType) -> Tuple[bool, List[str]]:
        """验证基本字段"""
        errors = []
        
        try:
            # 获取验证规则
            rules = self.validation_rules.get(data_type, {})
            required_fields = rules.get('required_fields', [])
            numeric_fields = rules.get('numeric_fields', [])
            
            # 检查必需字段
            for field in required_fields:
                if field not in data or data[field] is None:
                    errors.append(f"缺少必需字段: {field}")
                elif str(data[field]).strip() == '':
                    errors.append(f"字段为空: {field}")
            
            # 检查数值字段
            for field in numeric_fields:
                if field in data and data[field] is not None:
                    try:
                        value = Decimal(str(data[field]))
                        if value < 0:
                            errors.append(f"数值字段不能为负数: {field} = {value}")
                    except (ValueError, InvalidOperation):
                        errors.append(f"数值字段格式错误: {field} = {data[field]}")
            
            return len(errors) == 0, errors
            
        except Exception as e:
            errors.append(f"基本字段验证异常: {e}")
            return False, errors
    
    def validate_market_data(self, data: Dict[str, Any], symbol: str) -> Tuple[ValidationResult, List[str], List[str]]:
        """验证市场数据"""
        warnings = []
        errors = []
        
        try:
            price = Decimal(str(data.get('price', 0)))
            volume = Decimal(str(data.get('volume', 0)))
            
            # 价格合理性检查
            if price <= 0:
                errors.append(f"价格必须大于0: {price}")
                return ValidationResult.INVALID, warnings, errors
            
            # 成交量检查
            if volume < 0:
                errors.append(f"成交量不能为负数: {volume}")
                return ValidationResult.INVALID, warnings, errors
            
            # 价格异常检测
            symbol_key = f"{data.get('source', 'unknown')}_{symbol}"
            if symbol_key in self.last_prices:
                last_price = self.last_prices[symbol_key]
                price_change_percent = abs((price - last_price) / last_price * 100)
                
                # 检查价格变化是否过大(默认50%)
                max_change = self.validation_rules[StreamDataType.MARKET_DATA]['max_price_change_percent']
                if price_change_percent > max_change:
                    warnings.append(f"价格变化异常: {price_change_percent:.2f}% (阈值: {max_change}%)")
            
            # 记录价格历史
            self.price_history[symbol_key].append(price)
            self.last_prices[symbol_key] = price
            
            # 检查24小时数据一致性
            high_24h = data.get('highPrice')
            low_24h = data.get('lowPrice')
            if high_24h and low_24h:
                try:
                    high_24h = Decimal(str(high_24h))
                    low_24h = Decimal(str(low_24h))
                    
                    if high_24h < low_24h:
                        errors.append(f"24h最高价不能低于最低价: high={high_24h}, low={low_24h}")
                    elif not (low_24h <= price <= high_24h):
                        warnings.append(f"当前价格超出24h价格区间: price={price}, range=[{low_24h}, {high_24h}]")
                        
                except (ValueError, InvalidOperation):
                    warnings.append("24小时价格数据格式错误")
            
            return ValidationResult.VALID if not warnings else ValidationResult.WARNING, warnings, errors
            
        except Exception as e:
            errors.append(f"市场数据验证异常: {e}")
            return ValidationResult.INVALID, warnings, errors
    
    def validate_orderbook_data(self, data: Dict[str, Any]) -> Tuple[ValidationResult, List[str], List[str]]:
        """验证订单簿数据"""
        warnings = []
        errors = []
        
        try:
            bids = data.get('bids', [])
            asks = data.get('asks', [])
            
            # 检查买卖盘是否存在
            if not bids and not asks:
                errors.append("订单簿数据为空")
                return ValidationResult.INVALID, warnings, errors
            
            # 验证买盘数据
            if bids:
                for i, bid in enumerate(bids):
                    if len(bid) != 2:
                        errors.append(f"买盘格式错误，索引 {i}: {bid}")
                        continue
                    
                    try:
                        price, qty = Decimal(str(bid[0])), Decimal(str(bid[1]))
                        if price <= 0 or qty <= 0:
                            errors.append(f"买盘价格或数量无效，索引 {i}: price={price}, qty={qty}")
                    except (ValueError, InvalidOperation):
                        errors.append(f"买盘数值格式错误，索引 {i}: {bid}")
                
                # 检查买盘价格排序(应该降序)
                for i in range(1, len(bids)):
                    try:
                        if Decimal(str(bids[i-1][0])) < Decimal(str(bids[i][0])):
                            warnings.append("买盘价格排序异常(应该降序)")
                            break
                    except (ValueError, InvalidOperation):
                        continue
            
            # 验证卖盘数据
            if asks:
                for i, ask in enumerate(asks):
                    if len(ask) != 2:
                        errors.append(f"卖盘格式错误，索引 {i}: {ask}")
                        continue
                    
                    try:
                        price, qty = Decimal(str(ask[0])), Decimal(str(ask[1]))
                        if price <= 0 or qty <= 0:
                            errors.append(f"卖盘价格或数量无效，索引 {i}: price={price}, qty={qty}")
                    except (ValueError, InvalidOperation):
                        errors.append(f"卖盘数值格式错误，索引 {i}: {ask}")
                
                # 检查卖盘价格排序(应该升序)
                for i in range(1, len(asks)):
                    try:
                        if Decimal(str(asks[i-1][0])) > Decimal(str(asks[i][0])):
                            warnings.append("卖盘价格排序异常(应该升序)")
                            break
                    except (ValueError, InvalidOperation):
                        continue
            
            # 检查买卖价差
            if bids and asks:
                try:
                    best_bid = Decimal(str(bids[0][0]))
                    best_ask = Decimal(str(asks[0][0]))
                    
                    if best_bid >= best_ask:
                        errors.append(f"买卖价格异常: 最佳买价({best_bid}) >= 最佳卖价({best_ask})")
                    else:
                        spread_ratio = (best_ask - best_bid) / best_ask
                        min_spread_ratio = self.validation_rules[StreamDataType.ORDER_BOOK]['min_spread_ratio']
                        if spread_ratio < min_spread_ratio:
                            warnings.append(f"价差过小: {spread_ratio:.6f} (最小: {min_spread_ratio})")
                            
                except (ValueError, InvalidOperation):
                    warnings.append("无法计算买卖价差")
            
            # 检查订单簿深度
            max_levels = self.validation_rules[StreamDataType.ORDER_BOOK]['max_levels']
            if len(bids) > max_levels or len(asks) > max_levels:
                warnings.append(f"订单簿层级过多: bids={len(bids)}, asks={len(asks)} (最大: {max_levels})")
            
            return ValidationResult.VALID if not warnings else ValidationResult.WARNING, warnings, errors
            
        except Exception as e:
            errors.append(f"订单簿数据验证异常: {e}")
            return ValidationResult.INVALID, warnings, errors
    
    def validate_trade_data(self, data: Dict[str, Any]) -> Tuple[ValidationResult, List[str], List[str]]:
        """验证成交数据"""
        warnings = []
        errors = []
        
        try:
            price = data.get('price')
            quantity = data.get('qty') or data.get('quantity')
            
            if price is None:
                errors.append("缺少成交价格")
            else:
                try:
                    price = Decimal(str(price))
                    if price <= 0:
                        errors.append(f"成交价格必须大于0: {price}")
                except (ValueError, InvalidOperation):
                    errors.append(f"成交价格格式错误: {price}")
            
            if quantity is None:
                errors.append("缺少成交数量")
            else:
                try:
                    quantity = Decimal(str(quantity))
                    if quantity <= 0:
                        errors.append(f"成交数量必须大于0: {quantity}")
                except (ValueError, InvalidOperation):
                    errors.append(f"成交数量格式错误: {quantity}")
            
            # 检查成交时间
            trade_time = data.get('time')
            if trade_time:
                try:
                    trade_timestamp = normalize_timestamp(trade_time)
                    current_time = time.time()
                    
                    # 检查时间是否合理(不能是未来时间，不能太久以前)
                    if trade_timestamp > current_time + 60:  # 允许1分钟误差
                        warnings.append(f"成交时间异常(未来时间): {trade_timestamp}")
                    elif current_time - trade_timestamp > 86400:  # 超过24小时
                        warnings.append(f"成交时间异常(过期数据): {trade_timestamp}")
                        
                except Exception:
                    warnings.append("成交时间格式错误")
            
            return ValidationResult.VALID if not warnings else ValidationResult.WARNING, warnings, errors
            
        except Exception as e:
            errors.append(f"成交数据验证异常: {e}")
            return ValidationResult.INVALID, warnings, errors
    
    def validate(self, data: Dict[str, Any], data_type: StreamDataType, 
                symbol: str, source: str) -> Tuple[ValidationResult, List[str], List[str]]:
        """主验证方法"""
        self.validation_stats['total_validations'] += 1
        
        # 基本字段验证
        basic_valid, basic_errors = self.validate_basic_fields(data, data_type)
        if not basic_valid:
            self.validation_stats['invalid_count'] += 1
            return ValidationResult.INVALID, [], basic_errors
        
        # 符号格式验证
        if not validate_symbol_format(symbol):
            self.validation_stats['invalid_count'] += 1
            return ValidationResult.INVALID, [], [f"交易对格式无效: {symbol}"]
        
        # 特定类型验证
        warnings = []
        errors = []
        
        if data_type == StreamDataType.MARKET_DATA:
            result, warnings, errors = self.validate_market_data(data, symbol)
        elif data_type == StreamDataType.ORDER_BOOK:
            result, warnings, errors = self.validate_orderbook_data(data)
        elif data_type == StreamDataType.TRADE_DATA:
            result, warnings, errors = self.validate_trade_data(data)
        else:
            result = ValidationResult.VALID
        
        # 更新统计
        if result == ValidationResult.VALID:
            self.validation_stats['valid_count'] += 1
        elif result == ValidationResult.WARNING:
            self.validation_stats['warning_count'] += 1
        elif result == ValidationResult.INVALID:
            self.validation_stats['invalid_count'] += 1
        elif result == ValidationResult.CORRECTED:
            self.validation_stats['corrected_count'] += 1
        
        return result, warnings, errors

class DataNormalizer:
    """数据标准化器"""
    
    def __init__(self):
        # 交易所特定的字段映射 - 按数据类型分组
        self.field_mappings = {
            'binance': {
                'ticker': {  # 24小时统计数据 (ticker stream)
                    'price': 'c',  # 当前价格 (close price)
                    'volume': 'v',  # 成交量 (base asset volume)
                    'high': 'h',    # 最高价
                    'low': 'l',     # 最低价
                    'change': 'p',  # 价格变化
                    'change_percent': 'P',  # 价格变化百分比
                    'timestamp': 'E',  # 事件时间
                    'open': 'o',    # 开盘价
                    'close': 'c',   # 收盘价
                    'quote_volume': 'q'  # 计价货币成交量
                },
                'trade': {  # 交易数据 (trade stream)
                    'price': 'p',  # 交易价格
                    'volume': 'q',  # 交易数量
                    'quantity': 'q',  # 交易数量 (别名)
                    'timestamp': 'E',  # 事件时间
                    'trade_time': 'T',  # 交易时间
                    'trade_id': 't',  # 交易ID
                    'is_buyer_maker': 'm'  # 是否买方挂单
                },
                'orderbook': {  # 订单簿数据 (depth stream)
                    'timestamp': 'E',  # 事件时间
                    'first_update_id': 'U',  # 首次更新ID
                    'final_update_id': 'u',  # 最终更新ID
                    'bids': 'b',  # 买盘
                    'asks': 'a'   # 卖盘
                }
            },
            'okx': {
                'ticker': {
                    'price': 'last',
                    'volume': 'vol24h',
                    'high': 'high24h',
                    'low': 'low24h',
                    'change': 'changeUtc',
                    'change_percent': 'changeUtcPct',
                    'timestamp': 'ts'
                },
                'trade': {
                    'price': 'px',
                    'volume': 'sz',
                    'quantity': 'sz',  # 交易数量 (别名)
                    'timestamp': 'ts',
                    'trade_id': 'tradeId',
                    'side': 'side'
                },
                'orderbook': {
                    'timestamp': 'ts',
                    'bids': 'bids',
                    'asks': 'asks'
                }
            },
            'bitget': {
                'ticker': {
                    'price': 'close',
                    'volume': 'baseVolume',
                    'high': 'high',
                    'low': 'low',
                    'change': 'change',
                    'change_percent': 'changeUtc',
                    'timestamp': 'ts'
                },
                'trade': {
                    'price': 'price',
                    'volume': 'size',
                    'quantity': 'size',  # 交易数量 (别名)
                    'timestamp': 'ts',
                    'trade_id': 'tradeId',
                    'side': 'side'
                },
                'orderbook': {
                    'timestamp': 'ts',
                    'bids': 'bids',
                    'asks': 'asks'
                }
            }
        }
        
        logger.info("数据标准化器初始化完成")
    
    def normalize_symbol(self, symbol: str, source: str) -> str:
        """标准化交易对格式"""
        if not symbol:
            return ""
        
        # 移除常见分隔符，统一为大写
        normalized = symbol.upper().replace('/', '').replace('-', '').replace('_', '')
        
        # 特定交易所的符号处理
        if source == 'okx':
            # OKX使用 -分隔
            if len(normalized) >= 6:
                # 简单处理：假设基础货币3-4字符，计价货币3-4字符
                if normalized.endswith('USDT'):
                    return f"{normalized[:-4]}-USDT"
                elif normalized.endswith('BTC'):
                    return f"{normalized[:-3]}-BTC"
                elif normalized.endswith('ETH'):
                    return f"{normalized[:-3]}-ETH"
        
        return normalized
    
    def normalize_decimal_fields(self, data: Dict[str, Any], 
                                 fields: List[str], precision: int = 8) -> Dict[str, Any]:
        """标准化小数字段"""
        normalized = data.copy()
        
        for field in fields:
            if field in normalized and normalized[field] is not None:
                try:
                    normalized[field] = normalize_decimal_precision(normalized[field], precision)
                except Exception as e:
                    logger.warning(f"标准化字段 {field} 失败: {e}")
                    normalized[field] = Decimal('0')
        
        return normalized
    
    def normalize_timestamp_fields(self, data: Dict[str, Any], 
                                  fields: List[str]) -> Dict[str, Any]:
        """标准化时间戳字段"""
        normalized = data.copy()
        
        for field in fields:
            if field in normalized and normalized[field] is not None:
                try:
                    normalized[field] = normalize_timestamp(normalized[field])
                except Exception as e:
                    logger.warning(f"标准化时间戳字段 {field} 失败: {e}")
                    normalized[field] = time.time()
        
        return normalized
    
    def map_fields(self, data: Dict[str, Any], source: str, data_type: str = 'ticker') -> Dict[str, Any]:
        """映射交易所特定字段到标准字段"""
        if source not in self.field_mappings:
            return data
        
        if data_type not in self.field_mappings[source]:
            # 如果没有找到特定数据类型的映射，尝试使用 ticker 映射作为默认
            data_type = 'ticker'
            if data_type not in self.field_mappings[source]:
                return data
        
        mapping = self.field_mappings[source][data_type]
        mapped_data = {}
        
        # 复制原始数据
        for key, value in data.items():
            mapped_data[key] = value
        
        # 添加标准字段映射
        for standard_field, source_field in mapping.items():
            if source_field in data:
                mapped_data[standard_field] = data[source_field]
        
        return mapped_data
    
    def normalize_market_data(self, data: Dict[str, Any], source: str) -> Dict[str, Any]:
        """标准化市场数据"""
        # 字段映射 - 使用 ticker 类型
        normalized = self.map_fields(data, source, 'ticker')
        
        # 标准化小数字段
        decimal_fields = ['price', 'volume', 'high', 'low', 'change', 'change_percent']
        normalized = self.normalize_decimal_fields(normalized, decimal_fields)
        
        # 标准化时间戳字段
        timestamp_fields = ['timestamp', 'closeTime', 'ts']
        normalized = self.normalize_timestamp_fields(normalized, timestamp_fields)
        
        return normalized
    
    def normalize_orderbook_data(self, data: Dict[str, Any]) -> Dict[str, Any]:
        """标准化订单簿数据"""
        normalized = data.copy()
        
        # 标准化买卖盘数据
        for side in ['bids', 'asks']:
            if side in normalized and isinstance(normalized[side], list):
                standardized_orders = []
                for order in normalized[side]:
                    if isinstance(order, (list, tuple)) and len(order) >= 2:
                        try:
                            price = normalize_decimal_precision(order[0])
                            quantity = normalize_decimal_precision(order[1])
                            standardized_orders.append([price, quantity])
                        except Exception as e:
                            logger.warning(f"标准化订单数据失败: {e}")
                            continue
                
                normalized[side] = standardized_orders
        
        # 标准化时间戳
        if 'lastUpdateId' in normalized:
            try:
                normalized['lastUpdateId'] = int(normalized['lastUpdateId'])
            except (ValueError, TypeError):
                pass
        
        return normalized
    
    def normalize_trade_data(self, data: Dict[str, Any], source: str = None) -> Dict[str, Any]:
        """标准化成交数据"""
        normalized = data.copy()
        
        # 如果提供了数据源，进行字段映射
        if source:
            normalized = self.map_fields(normalized, source, 'trade')
        
        # 标准化价格和数量
        decimal_fields = ['price', 'volume', 'quantity']
        normalized = self.normalize_decimal_fields(normalized, decimal_fields)
        
        # 标准化时间戳
        timestamp_fields = ['time', 'timestamp', 'T', 'trade_time']
        normalized = self.normalize_timestamp_fields(normalized, timestamp_fields)
        
        # 标准化布尔字段
        if 'isBuyerMaker' in normalized:
            normalized['isBuyerMaker'] = bool(normalized['isBuyerMaker'])
        if 'is_buyer_maker' in normalized:
            normalized['is_buyer_maker'] = bool(normalized['is_buyer_maker'])
        
        # 标准化ID字段
        for id_field in ['id', 'trade_id', 't']:
            if id_field in normalized:
                normalized[id_field] = str(normalized[id_field])
        
        return normalized

class DataProcessor:
    """数据处理器主类"""
    
    def __init__(self, config: SubscriptionConfig):
        self.config = config
        self.validator = DataValidator(config)
        self.normalizer = DataNormalizer()
        
        # 处理统计
        self.processing_stats = {
            'total_processed': 0,
            'successful_processed': 0,
            'failed_processed': 0,
            'total_processing_time_ms': 0.0
        }
        
        # 批处理缓冲区
        self.batch_buffer: List[Tuple[Dict[str, Any], str, str, StreamDataType]] = []
        self.batch_size = config.batch_size
        
        # 质量统计
        self.quality_stats = DataQuality()
        
        logger.info(f"数据处理器初始化完成，批处理大小: {self.batch_size}")
    
    async def process_single(self, raw_data: Dict[str, Any], source: str, 
                           symbol: str, data_type: StreamDataType) -> ProcessingResult:
        """处理单条数据"""
        start_time = time.time()
        
        try:
            # 先进行数据标准化
            if data_type == StreamDataType.MARKET_DATA:
                normalized_data = self.normalizer.normalize_market_data(raw_data, source)
            elif data_type == StreamDataType.ORDER_BOOK:
                normalized_data = self.normalizer.normalize_orderbook_data(raw_data)
            elif data_type == StreamDataType.TRADE_DATA:
                normalized_data = self.normalizer.normalize_trade_data(raw_data, source)
            else:
                normalized_data = raw_data
            
            # 对标准化后的数据进行验证
            validation_result, warnings, errors = self.validator.validate(
                normalized_data, data_type, symbol, source
            )
            
            if validation_result == ValidationResult.INVALID:
                self.processing_stats['failed_processed'] += 1
                return ProcessingResult(
                    success=False,
                    validation_result=validation_result,
                    warnings=warnings,
                    errors=errors,
                    processing_time_ms=(time.time() - start_time) * 1000
                )
            
            # 标准化交易对格式
            normalized_symbol = self.normalizer.normalize_symbol(symbol, source)
            
            # 创建StreamData对象
            stream_data = create_stream_data(data_type, source, normalized_symbol, raw_data)
            stream_data.normalized_data = normalized_data
            stream_data.mark_processed()
            
            # 设置质量等级
            if validation_result == ValidationResult.WARNING:
                stream_data.quality = QualityLevel.GOOD
            else:
                stream_data.quality = QualityLevel.EXCELLENT
            
            # 更新统计
            self.processing_stats['successful_processed'] += 1
            processing_time = (time.time() - start_time) * 1000
            self.processing_stats['total_processing_time_ms'] += processing_time
            
            return ProcessingResult(
                success=True,
                stream_data=stream_data,
                validation_result=validation_result,
                warnings=warnings,
                errors=errors,
                processing_time_ms=processing_time
            )
            
        except Exception as e:
            self.processing_stats['failed_processed'] += 1
            error_msg = f"数据处理异常: {e}"
            logger.error(error_msg)
            
            return ProcessingResult(
                success=False,
                validation_result=ValidationResult.INVALID,
                errors=[error_msg],
                processing_time_ms=(time.time() - start_time) * 1000
            )
        finally:
            self.processing_stats['total_processed'] += 1
    
    async def process_batch(self, data_batch: List[Tuple[Dict[str, Any], str, str, StreamDataType]]) -> List[ProcessingResult]:
        """批量处理数据"""
        if not data_batch:
            return []
        
        results = []
        
        # 并发处理批量数据
        tasks = []
        for raw_data, source, symbol, data_type in data_batch:
            task = self.process_single(raw_data, source, symbol, data_type)
            tasks.append(task)
        
        # 等待所有任务完成
        results = await asyncio.gather(*tasks, return_exceptions=True)
        
        # 处理异常结果
        processed_results = []
        for i, result in enumerate(results):
            if isinstance(result, Exception):
                logger.error(f"批处理任务 {i} 异常: {result}")
                processed_results.append(ProcessingResult(
                    success=False,
                    validation_result=ValidationResult.INVALID,
                    errors=[f"批处理异常: {result}"]
                ))
            else:
                processed_results.append(result)
        
        logger.debug(f"批量处理完成: {len(data_batch)} 条数据")
        return processed_results
    
    async def add_to_batch(self, raw_data: Dict[str, Any], source: str, 
                          symbol: str, data_type: StreamDataType) -> Optional[List[ProcessingResult]]:
        """添加数据到批处理缓冲区"""
        self.batch_buffer.append((raw_data, source, symbol, data_type))
        
        # 检查是否达到批处理大小
        if len(self.batch_buffer) >= self.batch_size:
            batch_to_process = self.batch_buffer.copy()
            self.batch_buffer.clear()
            return await self.process_batch(batch_to_process)
        
        return None
    
    async def flush_batch(self) -> List[ProcessingResult]:
        """强制处理缓冲区中的所有数据"""
        if not self.batch_buffer:
            return []
        
        batch_to_process = self.batch_buffer.copy()
        self.batch_buffer.clear()
        return await self.process_batch(batch_to_process)
    
    def get_processing_stats(self) -> Dict[str, Any]:
        """获取处理统计信息"""
        total = self.processing_stats['total_processed']
        if total > 0:
            avg_processing_time = self.processing_stats['total_processing_time_ms'] / total
            success_rate = self.processing_stats['successful_processed'] / total
        else:
            avg_processing_time = 0.0
            success_rate = 0.0
        
        return {
            **self.processing_stats,
            'avg_processing_time_ms': avg_processing_time,
            'success_rate': success_rate,
            'validation_stats': self.validator.validation_stats,
            'buffer_size': len(self.batch_buffer)
        }
    
    def reset_stats(self):
        """重置统计信息"""
        self.processing_stats = {
            'total_processed': 0,
            'successful_processed': 0,
            'failed_processed': 0,
            'total_processing_time_ms': 0.0
        }
        
        self.validator.validation_stats = {
            'total_validations': 0,
            'valid_count': 0,
            'invalid_count': 0,
            'warning_count': 0,
            'corrected_count': 0
        }
    
    async def process_data(self, stream_data: StreamData) -> bool:
        """处理StreamData对象"""
        try:
            # 从StreamData中提取信息
            raw_data = stream_data.data
            source = stream_data.source
            symbol = stream_data.symbol
            data_type = stream_data.data_type
            
            # 使用现有的process_single方法处理数据
            result = await self.process_single(raw_data, source, symbol, data_type)
            
            if result.success:
                # 更新StreamData对象
                if result.stream_data:
                    stream_data.normalized_data = result.stream_data.normalized_data
                    stream_data.quality = result.stream_data.quality
                    stream_data.status = result.stream_data.status
                    stream_data.processed_time = result.stream_data.processed_time
                    stream_data.latency_ms = result.stream_data.latency_ms
                
                logger.debug(f"数据处理成功: {stream_data.id}")
                return True
            else:
                # 处理失败，记录错误
                logger.warning(f"数据处理失败: {stream_data.id}, 错误: {result.errors}")
                stream_data.status = DataStatus.ERROR
                return False
                
        except Exception as e:
            logger.error(f"处理StreamData异常: {e}")
            stream_data.status = DataStatus.ERROR
            return False 