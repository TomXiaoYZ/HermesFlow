"""
日志模块
"""
import os
import sys
import logging
from datetime import datetime
from logging.handlers import RotatingFileHandler
import structlog

def setup_logging(
    log_level: str = "INFO",
    log_dir: str = "logs",
    app_name: str = "data_service",
    max_bytes: int = 10 * 1024 * 1024,  # 10MB
    backup_count: int = 5
) -> None:
    """
    设置日志配置

    Args:
        log_level: 日志级别
        log_dir: 日志目录
        app_name: 应用名称
        max_bytes: 单个日志文件最大大小
        backup_count: 保留的日志文件数量
    """
    # 创建日志目录
    if not os.path.exists(log_dir):
        os.makedirs(log_dir)

    # 设置日志格式
    log_format = "%(asctime)s [%(levelname)s] %(name)s: %(message)s"
    formatter = logging.Formatter(log_format)

    # 设置控制台处理器
    console_handler = logging.StreamHandler(sys.stdout)
    console_handler.setFormatter(formatter)

    # 设置文件处理器
    log_file = os.path.join(log_dir, f"{app_name}_{datetime.now().strftime('%Y%m%d')}.log")
    file_handler = RotatingFileHandler(
        log_file,
        maxBytes=max_bytes,
        backupCount=backup_count,
        encoding="utf-8"
    )
    file_handler.setFormatter(formatter)

    # 配置structlog
    structlog.configure(
        processors=[
            structlog.stdlib.filter_by_level,
            structlog.stdlib.add_logger_name,
            structlog.stdlib.add_log_level,
            structlog.stdlib.PositionalArgumentsFormatter(),
            structlog.processors.TimeStamper(fmt="iso"),
            structlog.processors.StackInfoRenderer(),
            structlog.processors.format_exc_info,
            structlog.processors.UnicodeDecoder(),
            structlog.stdlib.render_to_log_kwargs,
        ],
        context_class=dict,
        logger_factory=structlog.stdlib.LoggerFactory(),
        wrapper_class=structlog.stdlib.BoundLogger,
        cache_logger_on_first_use=True,
    )

    # 获取根日志记录器
    root_logger = logging.getLogger()
    root_logger.setLevel(getattr(logging, log_level.upper()))

    # 清除现有的处理器
    root_logger.handlers.clear()

    # 添加处理器
    root_logger.addHandler(console_handler)
    root_logger.addHandler(file_handler)

def get_logger(name: str) -> structlog.BoundLogger:
    """
    获取日志记录器

    Args:
        name: 日志记录器名称

    Returns:
        structlog.BoundLogger: 结构化日志记录器
    """
    return structlog.get_logger(name) 