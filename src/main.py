#!/usr/bin/env python3
"""
HermesFlow 主启动脚本
支持多服务启动模式：data_collector、scheduler、api_gateway
"""
import argparse
import asyncio
import signal
import sys
import logging
from pathlib import Path
from typing import Dict, Any

# 添加项目根目录到Python路径
sys.path.insert(0, str(Path(__file__).parent.parent))

from src.config.config_manager import ConfigManager
from src.scheduler.scheduler import DataCollectionScheduler
from src.data.stream.stream_manager import StreamManager

# 设置日志
logging.basicConfig(
    level=logging.INFO,
    format='%(asctime)s - %(name)s - %(levelname)s - %(message)s'
)
logger = logging.getLogger(__name__)


class HermesFlowApplication:
    """HermesFlow 应用程序主类"""
    
    def __init__(self, service_type: str):
        """
        初始化应用程序
        
        Args:
            service_type: 服务类型 (data_collector, scheduler, api_gateway)
        """
        self.service_type = service_type
        self.config_manager = ConfigManager()
        self.config = self.config_manager  # 直接使用config_manager
        self.running = False
        self.services: Dict[str, Any] = {}
        
        logger.info(f"初始化 HermesFlow {service_type} 服务")
        logger.info(f"运行环境: {self.config_manager.environment.value}")
    
    async def start_data_collector(self):
        """启动数据采集服务"""
        logger.info("启动数据采集服务...")
        
        try:
            # 创建流配置对象
            from src.data.stream.config import StreamConfig
            stream_config = StreamConfig()
            
            # 初始化流管理器
            stream_manager = StreamManager(stream_config)
            self.services['stream_manager'] = stream_manager
            
            # 初始化数据流管理器
            if not await stream_manager.initialize():
                raise Exception("数据流管理器初始化失败")
            
            logger.info("数据采集服务启动成功")
            
            # 保持服务运行
            while self.running:
                await asyncio.sleep(1)
                
        except Exception as e:
            logger.error(f"数据采集服务启动失败: {e}")
            raise
    
    async def start_scheduler(self):
        """启动调度服务"""
        logger.info("启动调度服务...")
        
        try:
            # 初始化调度器
            scheduler = DataCollectionScheduler(self.config)
            self.services['scheduler'] = scheduler
            
            # 启动调度器
            await scheduler.start()
            
            # 添加默认任务
            await self._setup_default_jobs(scheduler)
            
            logger.info("调度服务启动成功")
            
            # 保持服务运行
            while self.running:
                await asyncio.sleep(1)
                
        except Exception as e:
            logger.error(f"调度服务启动失败: {e}")
            raise
    
    async def start_api_gateway(self):
        """启动API网关服务"""
        logger.info("启动API网关服务...")
        
        try:
            # 导入FastAPI相关模块
            from fastapi import FastAPI
            from fastapi.middleware.cors import CORSMiddleware
            import uvicorn
            
            # 创建FastAPI应用
            app = FastAPI(
                title="HermesFlow API",
                description="量化交易平台API网关",
                version="1.0.0"
            )
            
            # 添加CORS中间件
            app.add_middleware(
                CORSMiddleware,
                allow_origins=self.config.get('services', {}).get('api_gateway', {}).get('cors_origins', ["*"]),
                allow_credentials=True,
                allow_methods=["*"],
                allow_headers=["*"],
            )
            
            # 添加健康检查端点
            @app.get("/health")
            async def health_check():
                return {"status": "healthy", "service": "api_gateway"}
            
            @app.get("/")
            async def root():
                return {"message": "HermesFlow API Gateway", "version": "1.0.0"}
            
            # 启动服务器
            config = uvicorn.Config(
                app,
                host="0.0.0.0",
                port=8000,
                log_level="info"
            )
            server = uvicorn.Server(config)
            self.services['api_server'] = server
            
            await server.serve()
            
        except Exception as e:
            logger.error(f"API网关服务启动失败: {e}")
            raise
    
    async def start_strategy_engine(self):
        """启动策略引擎服务"""
        logger.info("启动策略引擎服务...")
        
        try:
            # 导入FastAPI相关模块
            from fastapi import FastAPI
            from fastapi.middleware.cors import CORSMiddleware
            import uvicorn
            
            # 创建FastAPI应用
            app = FastAPI(
                title="HermesFlow Strategy Engine",
                description="量化交易策略引擎",
                version="1.0.0"
            )
            
            # 添加CORS中间件
            app.add_middleware(
                CORSMiddleware,
                allow_origins=["*"],
                allow_credentials=True,
                allow_methods=["*"],
                allow_headers=["*"],
            )
            
            # 添加健康检查端点
            @app.get("/health")
            async def health_check():
                return {"status": "healthy", "service": "strategy_engine"}
            
            @app.get("/")
            async def root():
                return {"message": "HermesFlow Strategy Engine", "version": "1.0.0"}
            
            # 启动服务器
            config = uvicorn.Config(
                app,
                host="0.0.0.0",
                port=8003,
                log_level="info"
            )
            server = uvicorn.Server(config)
            self.services['strategy_server'] = server
            
            logger.info("策略引擎服务启动成功")
            await server.serve()
            
        except Exception as e:
            logger.error(f"策略引擎服务启动失败: {e}")
            raise
    
    async def _setup_default_jobs(self, scheduler):
        """设置默认的调度任务"""
        logger.info("设置默认调度任务...")
        
        # 获取调度器配置
        from src.scheduler.task_manager import TaskManager
        task_manager = TaskManager(scheduler)
        
        # 启动所有连接器的数据收集任务
        enabled_connectors = []
        connector_config = self.config.get('connectors', {})
        
        for connector_name, config in connector_config.items():
            if config.get('enabled', False):
                enabled_connectors.append(connector_name)
        
        if enabled_connectors:
            # 批量启动连接器任务
            await task_manager.start_connector_tasks(enabled_connectors)
            logger.info(f"已启动 {len(enabled_connectors)} 个连接器的调度任务")
        else:
            logger.warning("没有启用的连接器")
    
    async def start(self):
        """启动应用程序"""
        self.running = True
        
        try:
            if self.service_type == "data_collector":
                await self.start_data_collector()
            elif self.service_type == "scheduler":
                await self.start_scheduler()
            elif self.service_type == "api_gateway":
                await self.start_api_gateway()
            elif self.service_type == "strategy_engine":
                await self.start_strategy_engine()
            else:
                raise ValueError(f"未知的服务类型: {self.service_type}")
                
        except KeyboardInterrupt:
            logger.info("接收到中断信号，正在停止服务...")
        except Exception as e:
            logger.error(f"服务运行异常: {e}")
            raise
        finally:
            await self.stop()
    
    async def stop(self):
        """停止应用程序"""
        logger.info("正在停止服务...")
        self.running = False
        
        # 停止所有服务
        for service_name, service in self.services.items():
            try:
                if hasattr(service, 'stop'):
                    await service.stop()
                elif hasattr(service, 'shutdown'):
                    await service.shutdown()
                logger.info(f"已停止服务: {service_name}")
            except Exception as e:
                logger.error(f"停止服务 {service_name} 时出错: {e}")
        
        logger.info("所有服务已停止")
    
    def setup_signal_handlers(self):
        """设置信号处理器"""
        def signal_handler(signum, frame):
            logger.info(f"收到信号 {signum}")
            self.running = False
        
        signal.signal(signal.SIGINT, signal_handler)
        signal.signal(signal.SIGTERM, signal_handler)


def main():
    """主函数"""
    parser = argparse.ArgumentParser(description="HermesFlow 量化交易平台")
    parser.add_argument(
        "--service",
        choices=["data_collector", "scheduler", "api_gateway", "strategy_engine"],
        required=True,
        help="要启动的服务类型"
    )
    parser.add_argument(
        "--config",
        default="config",
        help="配置文件目录路径"
    )
    parser.add_argument(
        "--debug",
        action="store_true",
        help="启用调试模式"
    )
    
    args = parser.parse_args()
    
    # 设置日志级别
    if args.debug:
        logging.getLogger().setLevel(logging.DEBUG)
    
    # 创建应用程序实例
    app = HermesFlowApplication(args.service)
    
    # 设置信号处理器
    app.setup_signal_handlers()
    
    try:
        # 启动应用程序
        asyncio.run(app.start())
    except KeyboardInterrupt:
        logger.info("程序被用户中断")
    except Exception as e:
        logger.error(f"程序异常退出: {e}")
        sys.exit(1)


if __name__ == "__main__":
    main() 