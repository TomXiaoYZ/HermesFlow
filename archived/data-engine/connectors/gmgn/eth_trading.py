"""
GMGN ETH系链交易模块

实现ETH/Base/BSC链上的交易功能，包括：
- 多链swap路由查询
- Gas价格估算
- 交易模拟验证
- 滑点推荐
"""

import asyncio
from typing import Dict, List, Optional, Any, Tuple
from datetime import datetime
import logging

from .models import (
    ChainType, SwapMode, TransactionStatus,
    TokenInfo, SwapRoute, SwapQuote, TransactionResult,
    EthereumTokens, create_swap_route_from_gmgn_data
)

logger = logging.getLogger(__name__)


class ETHTrading:
    """
    ETH系链交易功能实现
    
    支持链：
    - Ethereum主网
    - Base L2
    - BSC (Binance Smart Chain)
    
    功能特性：
    - 多协议路由查询
    - 智能Gas估算
    - 交易模拟验证
    - 滑点优化推荐
    """
    
    def __init__(self, connector, chain: ChainType = ChainType.ETHEREUM):
        """
        初始化ETH系链交易模块
        
        Args:
            connector: GMGN连接器实例
            chain: 目标链类型
        """
        self.connector = connector
        self.chain = chain
        self.base_url = f"{connector.base_url}/defi/router/v1/{chain.value}"
        
        # 链特定配置
        self.chain_configs = {
            ChainType.ETHEREUM: {
                'name': 'Ethereum',
                'native_token': EthereumTokens.ETH,
                'gas_unit': 'gwei',
                'block_time': 12
            },
            ChainType.BASE: {
                'name': 'Base',
                'native_token': '0x4200000000000000000000000000000000000006',  # Base ETH
                'gas_unit': 'gwei',
                'block_time': 2
            },
            ChainType.BSC: {
                'name': 'BSC',
                'native_token': '0xbb4CdB9CBd36B01bD1cBaEBF2De08d9173bc095c',  # WBNB
                'gas_unit': 'gwei',
                'block_time': 3
            }
        }
        
        self.config = self.chain_configs.get(chain, self.chain_configs[ChainType.ETHEREUM])
        
        logger.info(f"{self.config['name']}交易模块初始化完成")
    
    async def get_available_routes(
        self,
        token_in_address: str,
        token_out_address: str,
        amount_in: str,
        from_address: str,
        slippage: Optional[float] = None
    ) -> Optional[List[Dict[str, Any]]]:
        """
        获取可用的交易路由
        
        Args:
            token_in_address: 输入代币地址
            token_out_address: 输出代币地址
            amount_in: 输入金额 (Wei单位)
            from_address: 发送方地址
            slippage: 滑点百分比 (可选)
            
        Returns:
            Optional[List[Dict]]: 可用路由列表
        """
        try:
            url = f"{self.base_url}/tx/get_routes"
            
            params = {
                'tokenIn': token_in_address,
                'tokenOut': token_out_address,
                'amountIn': amount_in,
                'from': from_address
            }
            
            if slippage is not None:
                params['slippage'] = slippage
            
            logger.info(f"查询{self.config['name']}可用路由")
            
            response = await self.connector._make_request('GET', url, params=params)
            
            if not response or response.get('code') != 0:
                logger.error("路由查询失败")
                return None
            
            routes = response.get('data', {}).get('routes', [])
            logger.info(f"找到 {len(routes)} 条可用路由")
            
            return routes
            
        except Exception as e:
            logger.error(f"获取可用路由异常: {e}")
            return None
    
    async def get_recommended_slippage(
        self,
        token_address: str
    ) -> Optional[float]:
        """
        获取代币推荐滑点
        
        Args:
            token_address: 代币地址
            
        Returns:
            Optional[float]: 推荐滑点百分比
        """
        try:
            url = f"{self.base_url}/slippage/recommend"
            
            params = {
                'token': token_address
            }
            
            logger.debug(f"查询代币 {token_address} 推荐滑点")
            
            response = await self.connector._make_request('GET', url, params=params)
            
            if not response or response.get('code') != 0:
                # 如果查询失败，返回默认滑点
                default_slippage = 0.5  # 0.5%
                logger.warning(f"滑点查询失败，使用默认值: {default_slippage}%")
                return default_slippage
            
            slippage = response.get('data', {}).get('slippage')
            if slippage is not None:
                logger.debug(f"推荐滑点: {slippage}%")
                return float(slippage)
            
            return 0.5  # 默认滑点
            
        except Exception as e:
            logger.error(f"获取推荐滑点异常: {e}")
            return 0.5
    
    async def get_recommended_gas_price(self) -> Optional[Dict[str, Any]]:
        """
        获取推荐Gas价格
        
        Returns:
            Optional[Dict]: Gas价格信息
        """
        try:
            url = f"{self.base_url}/gas/recommend"
            
            logger.debug(f"查询{self.config['name']}推荐Gas价格")
            
            response = await self.connector._make_request('GET', url)
            
            if not response or response.get('code') != 0:
                logger.error("Gas价格查询失败")
                return None
            
            gas_data = response.get('data', {})
            
            # 解析Gas价格信息
            gas_info = {
                'slow': {
                    'gas_price': gas_data.get('slow', {}).get('gasPrice'),
                    'estimate_time': gas_data.get('slow', {}).get('estimateTime', '> 5 min')
                },
                'standard': {
                    'gas_price': gas_data.get('standard', {}).get('gasPrice'),
                    'estimate_time': gas_data.get('standard', {}).get('estimateTime', '~ 3 min')
                },
                'fast': {
                    'gas_price': gas_data.get('fast', {}).get('gasPrice'),
                    'estimate_time': gas_data.get('fast', {}).get('estimateTime', '< 1 min')
                }
            }
            
            logger.debug(f"Gas价格获取成功: {gas_info}")
            return gas_info
            
        except Exception as e:
            logger.error(f"获取推荐Gas价格异常: {e}")
            return None
    
    async def simulate_swap_transaction(
        self,
        token_in_address: str,
        token_out_address: str,
        amount_in: str,
        from_address: str,
        route_data: Optional[Dict] = None,
        slippage: Optional[float] = None,
        gas_price: Optional[str] = None
    ) -> Optional[Dict[str, Any]]:
        """
        模拟swap交易
        
        Args:
            token_in_address: 输入代币地址
            token_out_address: 输出代币地址
            amount_in: 输入金额
            from_address: 发送方地址
            route_data: 指定路由数据 (可选)
            slippage: 滑点 (可选)
            gas_price: Gas价格 (可选)
            
        Returns:
            Optional[Dict]: 模拟结果
        """
        try:
            url = f"{self.base_url}/tx/simulate"
            
            # 构建模拟参数
            data = {
                'tokenIn': token_in_address,
                'tokenOut': token_out_address,
                'amountIn': amount_in,
                'from': from_address
            }
            
            # 添加可选参数
            if route_data:
                data['route'] = route_data
            
            if slippage is not None:
                data['slippage'] = slippage
            
            if gas_price:
                data['gasPrice'] = gas_price
            
            logger.info(f"模拟{self.config['name']}交易")
            
            response = await self.connector._make_request('POST', url, data=data)
            
            if not response or response.get('code') != 0:
                logger.error("交易模拟失败")
                return None
            
            simulation_result = response.get('data', {})
            
            # 解析模拟结果
            result = {
                'success': simulation_result.get('success', False),
                'gas_estimate': simulation_result.get('gasEstimate'),
                'gas_price': simulation_result.get('gasPrice'),
                'output_amount': simulation_result.get('outputAmount'),
                'price_impact': simulation_result.get('priceImpact'),
                'route': simulation_result.get('route'),
                'error': simulation_result.get('error')
            }
            
            if result['success']:
                logger.info(f"交易模拟成功: 预计输出 {result['output_amount']}")
            else:
                logger.warning(f"交易模拟失败: {result['error']}")
            
            return result
            
        except Exception as e:
            logger.error(f"模拟swap交易异常: {e}")
            return None
    
    async def build_swap_transaction(
        self,
        token_in_address: str,
        token_out_address: str,
        amount_in: str,
        from_address: str,
        slippage: Optional[float] = None,
        gas_price: Optional[str] = None,
        auto_optimize: bool = True
    ) -> Optional[SwapRoute]:
        """
        构建完整的swap交易
        
        Args:
            token_in_address: 输入代币地址
            token_out_address: 输出代币地址
            amount_in: 输入金额
            from_address: 发送方地址
            slippage: 滑点 (可选，自动推荐)
            gas_price: Gas价格 (可选，自动推荐)
            auto_optimize: 是否自动优化参数
            
        Returns:
            Optional[SwapRoute]: 构建的交易路由
        """
        try:
            logger.info(f"构建{self.config['name']}交易: {token_in_address} -> {token_out_address}")
            
            # 第一步：获取推荐参数 (如果启用自动优化)
            if auto_optimize:
                if slippage is None:
                    slippage = await self.get_recommended_slippage(token_out_address)
                
                if gas_price is None:
                    gas_info = await self.get_recommended_gas_price()
                    if gas_info:
                        gas_price = gas_info['standard']['gas_price']
            
            # 第二步：获取最优路由
            routes = await self.get_available_routes(
                token_in_address,
                token_out_address,
                amount_in,
                from_address,
                slippage
            )
            
            if not routes:
                logger.error("未找到可用路由")
                return None
            
            # 选择最优路由 (通常是第一个)
            best_route = routes[0]
            
            # 第三步：模拟交易
            simulation = await self.simulate_swap_transaction(
                token_in_address,
                token_out_address,
                amount_in,
                from_address,
                best_route,
                slippage,
                gas_price
            )
            
            if not simulation or not simulation['success']:
                logger.error(f"交易模拟失败: {simulation.get('error') if simulation else 'Unknown'}")
                return None
            
            # 第四步：构建SwapRoute对象
            # 这里需要根据GMGN API的实际响应格式来适配
            mock_route_data = {
                'quote': {
                    'inputMint': token_in_address,
                    'outputMint': token_out_address,
                    'inAmount': amount_in,
                    'outAmount': simulation['output_amount'],
                    'slippageBps': int((slippage or 0.5) * 100),
                    'priceImpactPct': simulation.get('price_impact', '0'),
                    'otherAmountThreshold': simulation['output_amount'],
                    'swapMode': 'ExactIn',
                    'routePlan': best_route.get('steps', [])
                },
                'raw_tx': {
                    'transactionData': best_route.get('transaction'),
                    'gasEstimate': simulation['gas_estimate'],
                    'gasPrice': simulation['gas_price']
                }
            }
            
            swap_route = create_swap_route_from_gmgn_data(mock_route_data, self.chain)
            
            logger.info(f"交易构建成功: 预计输出 {simulation['output_amount']}")
            return swap_route
            
        except Exception as e:
            logger.error(f"构建swap交易异常: {e}")
            return None
    
    async def estimate_transaction_cost(
        self,
        token_in_address: str,
        token_out_address: str,
        amount_in: str,
        from_address: str
    ) -> Optional[Dict[str, Any]]:
        """
        估算交易成本
        
        Args:
            token_in_address: 输入代币地址
            token_out_address: 输出代币地址
            amount_in: 输入金额
            from_address: 发送方地址
            
        Returns:
            Optional[Dict]: 成本估算信息
        """
        try:
            # 获取Gas价格信息
            gas_info = await self.get_recommended_gas_price()
            if not gas_info:
                return None
            
            # 模拟交易以获取Gas估算
            simulation = await self.simulate_swap_transaction(
                token_in_address,
                token_out_address,
                amount_in,
                from_address
            )
            
            if not simulation:
                return None
            
            gas_estimate = simulation.get('gas_estimate', 0)
            
            # 计算不同速度下的交易费用
            costs = {}
            for speed, info in gas_info.items():
                gas_price_gwei = float(info['gas_price']) if info['gas_price'] else 0
                
                # 转换为ETH (1 ETH = 1e18 Wei, 1 Gwei = 1e9 Wei)
                cost_eth = (gas_estimate * gas_price_gwei * 1e9) / 1e18
                
                costs[speed] = {
                    'gas_price_gwei': gas_price_gwei,
                    'gas_estimate': gas_estimate,
                    'cost_eth': cost_eth,
                    'estimate_time': info['estimate_time']
                }
            
            result = {
                'chain': self.config['name'],
                'costs': costs,
                'price_impact': simulation.get('price_impact', '0'),
                'output_amount': simulation.get('output_amount')
            }
            
            logger.info(f"交易成本估算完成: {result}")
            return result
            
        except Exception as e:
            logger.error(f"估算交易成本异常: {e}")
            return None
    
    def get_common_tokens(self) -> Dict[str, TokenInfo]:
        """
        获取当前链的常用代币信息
        
        Returns:
            Dict[str, TokenInfo]: 常用代币字典
        """
        if self.chain == ChainType.ETHEREUM:
            return {
                'ETH': TokenInfo(
                    address=EthereumTokens.ETH,
                    symbol='ETH',
                    name='Ethereum',
                    decimals=18,
                    chain=self.chain,
                    is_verified=True
                ),
                'USDC': TokenInfo(
                    address=EthereumTokens.USDC,
                    symbol='USDC',
                    name='USD Coin',
                    decimals=6,
                    chain=self.chain,
                    is_verified=True
                ),
                'USDT': TokenInfo(
                    address=EthereumTokens.USDT,
                    symbol='USDT',
                    name='Tether USD',
                    decimals=6,
                    chain=self.chain,
                    is_verified=True
                ),
                'DAI': TokenInfo(
                    address=EthereumTokens.DAI,
                    symbol='DAI',
                    name='Dai Stablecoin',
                    decimals=18,
                    chain=self.chain,
                    is_verified=True
                )
            }
        elif self.chain == ChainType.BASE:
            return {
                'ETH': TokenInfo(
                    address='0x4200000000000000000000000000000000000006',
                    symbol='ETH',
                    name='Ethereum (Base)',
                    decimals=18,
                    chain=self.chain,
                    is_verified=True
                ),
                'USDC': TokenInfo(
                    address='0x833589fCD6eDb6E08f4c7C32D4f71b54bdA02913',
                    symbol='USDC',
                    name='USD Coin (Base)',
                    decimals=6,
                    chain=self.chain,
                    is_verified=True
                )
            }
        elif self.chain == ChainType.BSC:
            return {
                'BNB': TokenInfo(
                    address='0xbb4CdB9CBd36B01bD1cBaEBF2De08d9173bc095c',
                    symbol='BNB',
                    name='Binance Coin',
                    decimals=18,
                    chain=self.chain,
                    is_verified=True
                ),
                'USDT': TokenInfo(
                    address='0x55d398326f99059fF775485246999027B3197955',
                    symbol='USDT',
                    name='Tether USD (BSC)',
                    decimals=18,
                    chain=self.chain,
                    is_verified=True
                ),
                'USDC': TokenInfo(
                    address='0x8AC76a51cc950d9822D68b83fE1Ad97B32Cd580d',
                    symbol='USDC',
                    name='USD Coin (BSC)',
                    decimals=18,
                    chain=self.chain,
                    is_verified=True
                )
            }
        
        return {}
    
    def get_chain_info(self) -> Dict[str, Any]:
        """
        获取链信息
        
        Returns:
            Dict[str, Any]: 链配置信息
        """
        return {
            'chain': self.chain.value,
            'name': self.config['name'],
            'native_token': self.config['native_token'],
            'gas_unit': self.config['gas_unit'],
            'block_time': self.config['block_time'],
            'common_tokens': list(self.get_common_tokens().keys())
        } 