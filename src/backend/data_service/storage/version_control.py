"""
版本控制模块

该模块负责管理数据的版本控制，包括：
1. 数据版本号管理
2. 数据变更记录
3. 数据回滚支持
"""

import logging
from typing import Dict, List, Optional, Union
from datetime import datetime
from ..common.singleton import Singleton

logger = logging.getLogger(__name__)

class VersionControl(metaclass=Singleton):
    """版本控制类，使用单例模式确保版本号的一致性"""
    
    def __init__(self):
        """初始化版本控制系统"""
        self.current_version = 1
        self.version_history = {}
        logger.info("版本控制系统初始化完成")
    
    def get_next_version(self) -> int:
        """获取下一个版本号
        
        Returns:
            int: 新的版本号
        """
        self.current_version += 1
        return self.current_version
    
    def record_change(self, data_type: str, operation: str,
                     details: Dict, timestamp: Optional[datetime] = None) -> int:
        """记录数据变更
        
        Args:
            data_type: 数据类型（market_data/trade/order/system_log）
            operation: 操作类型（insert/update/delete）
            details: 变更详情
            timestamp: 时间戳，默认为当前时间
            
        Returns:
            int: 版本号
        """
        version = self.get_next_version()
        timestamp = timestamp or datetime.now()
        
        change_record = {
            'version': version,
            'timestamp': timestamp,
            'data_type': data_type,
            'operation': operation,
            'details': details
        }
        
        self.version_history[version] = change_record
        logger.info(f"记录数据变更: version={version}, "
                   f"type={data_type}, operation={operation}")
        return version
    
    def get_version_history(self, data_type: Optional[str] = None,
                          start_version: Optional[int] = None,
                          end_version: Optional[int] = None) -> List[Dict]:
        """获取版本历史
        
        Args:
            data_type: 数据类型，可选
            start_version: 起始版本号，可选
            end_version: 结束版本号，可选
            
        Returns:
            List[Dict]: 版本历史记录列表
        """
        history = []
        for version, record in sorted(self.version_history.items()):
            if start_version and version < start_version:
                continue
            if end_version and version > end_version:
                continue
            if data_type and record['data_type'] != data_type:
                continue
            history.append(record)
        return history
    
    def get_version_details(self, version: int) -> Optional[Dict]:
        """获取特定版本的详细信息
        
        Args:
            version: 版本号
            
        Returns:
            Optional[Dict]: 版本详情，如果不存在则返回None
        """
        return self.version_history.get(version)
    
    def get_latest_version(self, data_type: Optional[str] = None) -> int:
        """获取最新版本号
        
        Args:
            data_type: 数据类型，可选
            
        Returns:
            int: 最新版本号
        """
        if not data_type:
            return self.current_version
        
        latest_version = 0
        for version, record in self.version_history.items():
            if record['data_type'] == data_type and version > latest_version:
                latest_version = version
        return latest_version
    
    def get_changes_since(self, version: int,
                         data_type: Optional[str] = None) -> List[Dict]:
        """获取指定版本之后的所有变更
        
        Args:
            version: 起始版本号
            data_type: 数据类型，可选
            
        Returns:
            List[Dict]: 变更记录列表
        """
        changes = []
        for v, record in sorted(self.version_history.items()):
            if v <= version:
                continue
            if data_type and record['data_type'] != data_type:
                continue
            changes.append(record)
        return changes
    
    def clean_old_versions(self, before_version: int) -> int:
        """清理旧版本记录
        
        Args:
            before_version: 要清理的版本号（不包含此版本）
            
        Returns:
            int: 清理的记录数量
        """
        versions_to_remove = []
        for version in self.version_history.keys():
            if version < before_version:
                versions_to_remove.append(version)
        
        for version in versions_to_remove:
            del self.version_history[version]
        
        logger.info(f"清理旧版本记录: 清理数量={len(versions_to_remove)}, "
                   f"before_version={before_version}")
        return len(versions_to_remove) 