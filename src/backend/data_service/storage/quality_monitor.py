"""
数据质量监控模块

该模块负责监控和检查数据质量，包括：
1. 数据完整性检查
2. 数据准确性验证
3. 数据延迟监控
4. 异常数据检测
"""

import logging
from typing import Dict, List, Optional, Union
from datetime import datetime, timedelta
import numpy as np
from ..common.singleton import Singleton

logger = logging.getLogger(__name__)

class QualityMonitor(metaclass=Singleton):
    """数据质量监控类，使用单例模式确保监控的一致性"""
    
    def __init__(self):
        """初始化数据质量监控系统"""
        self.data_stats = {}
        self.alerts = []
        logger.info("数据质量监控系统初始化完成")
    
    def check_market_data_quality(self, exchange: str, symbol: str,
                                data: Dict) -> Dict[str, bool]:
        """检查市场数据质量
        
        Args:
            exchange: 交易所名称
            symbol: 交易对
            data: 市场数据
            
        Returns:
            Dict[str, bool]: 检查结果
        """
        results = {
            'completeness': True,  # 数据完整性
            'accuracy': True,      # 数据准确性
            'timeliness': True,    # 数据时效性
            'consistency': True    # 数据一致性
        }
        
        # 检查数据完整性
        required_fields = ['price', 'volume', 'timestamp']
        for field in required_fields:
            if field not in data:
                results['completeness'] = False
                self._record_alert(
                    'market_data', exchange, symbol,
                    f"缺失必要字段: {field}"
                )
        
        # 检查数据准确性
        if results['completeness']:
            if not self._is_valid_price(data['price']):
                results['accuracy'] = False
                self._record_alert(
                    'market_data', exchange, symbol,
                    f"价格异常: {data['price']}"
                )
            if not self._is_valid_volume(data['volume']):
                results['accuracy'] = False
                self._record_alert(
                    'market_data', exchange, symbol,
                    f"成交量异常: {data['volume']}"
                )
        
        # 检查数据时效性
        if results['completeness']:
            delay = datetime.now() - data['timestamp']
            if delay > timedelta(seconds=5):
                results['timeliness'] = False
                self._record_alert(
                    'market_data', exchange, symbol,
                    f"数据延迟: {delay.total_seconds()}秒"
                )
        
        # 检查数据一致性
        if results['completeness'] and 'bid' in data and 'ask' in data:
            if not self._is_valid_spread(data['bid'], data['ask']):
                results['consistency'] = False
                self._record_alert(
                    'market_data', exchange, symbol,
                    f"买卖价差异常: bid={data['bid']}, ask={data['ask']}"
                )
        
        return results
    
    def check_trade_data_quality(self, exchange: str, symbol: str,
                               trades: List[Dict]) -> Dict[str, bool]:
        """检查交易数据质量
        
        Args:
            exchange: 交易所名称
            symbol: 交易对
            trades: 交易记录列表
            
        Returns:
            Dict[str, bool]: 检查结果
        """
        results = {
            'completeness': True,
            'accuracy': True,
            'sequence': True
        }
        
        if not trades:
            return results
        
        # 检查数据完整性
        required_fields = ['trade_id', 'price', 'amount', 'timestamp']
        for trade in trades:
            for field in required_fields:
                if field not in trade:
                    results['completeness'] = False
                    self._record_alert(
                        'trade_data', exchange, symbol,
                        f"交易记录缺失必要字段: {field}"
                    )
        
        # 检查数据准确性
        if results['completeness']:
            for trade in trades:
                if not self._is_valid_price(trade['price']):
                    results['accuracy'] = False
                    self._record_alert(
                        'trade_data', exchange, symbol,
                        f"交易价格异常: {trade['price']}"
                    )
                if not self._is_valid_volume(trade['amount']):
                    results['accuracy'] = False
                    self._record_alert(
                        'trade_data', exchange, symbol,
                        f"交易数量异常: {trade['amount']}"
                    )
        
        # 检查交易序列
        if len(trades) > 1:
            timestamps = [t['timestamp'] for t in trades]
            if not all(t1 <= t2 for t1, t2 in zip(timestamps[:-1], timestamps[1:])):
                results['sequence'] = False
                self._record_alert(
                    'trade_data', exchange, symbol,
                    "交易时间序列异常"
                )
        
        return results
    
    def check_order_data_quality(self, exchange: str, user_id: str,
                               orders: List[Dict]) -> Dict[str, bool]:
        """检查订单数据质量
        
        Args:
            exchange: 交易所名称
            user_id: 用户ID
            orders: 订单记录列表
            
        Returns:
            Dict[str, bool]: 检查结果
        """
        results = {
            'completeness': True,
            'accuracy': True,
            'consistency': True
        }
        
        if not orders:
            return results
        
        # 检查数据完整性
        required_fields = ['order_id', 'symbol', 'type', 'side',
                         'price', 'amount', 'status']
        for order in orders:
            for field in required_fields:
                if field not in order:
                    results['completeness'] = False
                    self._record_alert(
                        'order_data', exchange, user_id,
                        f"订单记录缺失必要字段: {field}"
                    )
        
        # 检查数据准确性
        if results['completeness']:
            for order in orders:
                if not self._is_valid_price(order['price']):
                    results['accuracy'] = False
                    self._record_alert(
                        'order_data', exchange, user_id,
                        f"订单价格异常: {order['price']}"
                    )
                if not self._is_valid_volume(order['amount']):
                    results['accuracy'] = False
                    self._record_alert(
                        'order_data', exchange, user_id,
                        f"订单数量异常: {order['amount']}"
                    )
        
        # 检查数据一致性
        if results['completeness']:
            for order in orders:
                if 'filled' in order and order['filled'] > order['amount']:
                    results['consistency'] = False
                    self._record_alert(
                        'order_data', exchange, user_id,
                        f"订单成交量超过委托量: filled={order['filled']}, "
                        f"amount={order['amount']}"
                    )
        
        return results
    
    def get_data_stats(self, data_type: str, exchange: str,
                      symbol: Optional[str] = None) -> Dict:
        """获取数据统计信息
        
        Args:
            data_type: 数据类型
            exchange: 交易所名称
            symbol: 交易对，可选
            
        Returns:
            Dict: 统计信息
        """
        key = f"{data_type}:{exchange}"
        if symbol:
            key = f"{key}:{symbol}"
        return self.data_stats.get(key, {})
    
    def get_alerts(self, start_time: Optional[datetime] = None,
                  end_time: Optional[datetime] = None) -> List[Dict]:
        """获取警报记录
        
        Args:
            start_time: 开始时间，可选
            end_time: 结束时间，可选
            
        Returns:
            List[Dict]: 警报记录列表
        """
        if not start_time and not end_time:
            return self.alerts
        
        filtered_alerts = []
        for alert in self.alerts:
            if start_time and alert['timestamp'] < start_time:
                continue
            if end_time and alert['timestamp'] > end_time:
                continue
            filtered_alerts.append(alert)
        return filtered_alerts
    
    def _record_alert(self, data_type: str, exchange: str,
                     identifier: str, message: str):
        """记录警报
        
        Args:
            data_type: 数据类型
            exchange: 交易所名称
            identifier: 标识符（symbol或user_id）
            message: 警报消息
        """
        alert = {
            'timestamp': datetime.now(),
            'data_type': data_type,
            'exchange': exchange,
            'identifier': identifier,
            'message': message
        }
        self.alerts.append(alert)
        logger.warning(f"数据质量警报: {message}")
    
    def _is_valid_price(self, price: float) -> bool:
        """检查价格是否有效
        
        Args:
            price: 价格
            
        Returns:
            bool: 是否有效
        """
        return (isinstance(price, (int, float)) and
                price > 0 and
                not np.isnan(price) and
                not np.isinf(price))
    
    def _is_valid_volume(self, volume: float) -> bool:
        """检查数量是否有效
        
        Args:
            volume: 数量
            
        Returns:
            bool: 是否有效
        """
        return (isinstance(volume, (int, float)) and
                volume >= 0 and
                not np.isnan(volume) and
                not np.isinf(volume))
    
    def _is_valid_spread(self, bid: float, ask: float) -> bool:
        """检查买卖价差是否有效
        
        Args:
            bid: 买价
            ask: 卖价
            
        Returns:
            bool: 是否有效
        """
        if not (self._is_valid_price(bid) and self._is_valid_price(ask)):
            return False
        return bid <= ask 