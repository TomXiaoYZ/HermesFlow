"""
GMGN Solana交易模块

实现Solana链上的交易功能，包括：
- Swap路由查询
- 交易构建和提交  
- 交易状态监控
- 反MEV支持
"""

import asyncio
import base64
import json
from typing import Dict, List, Optional, Any, Tuple
from datetime import datetime, timedelta
import logging

from .models import (
    ChainType, SwapMode, TransactionStatus,
    TokenInfo, SwapRoute, SwapQuote, TransactionResult,
    SolanaTokens, create_swap_route_from_gmgn_data
)

logger = logging.getLogger(__name__)


class SolanaTrading:
    """
    Solana链交易功能实现
    
    提供完整的Solana链交易能力：
    - 代币swap交易
    - 路由优化
    - 交易监控
    - MEV保护
    """
    
    def __init__(self, connector):
        """
        初始化Solana交易模块
        
        Args:
            connector: GMGN连接器实例
        """
        self.connector = connector
        self.chain = ChainType.SOLANA
        self.base_url = f"{connector.base_url}/defi/router/v1/sol"
        
        # 交易监控
        self._pending_transactions: Dict[str, TransactionResult] = {}
        
        logger.info("Solana交易模块初始化完成")
    
    async def get_swap_route(
        self,
        token_in_address: str,
        token_out_address: str,
        amount_in: str,
        from_address: str,
        slippage: float = 1.0,
        swap_mode: SwapMode = SwapMode.EXACT_IN,
        enable_anti_mev: bool = True,
        gas_fee: Optional[float] = None
    ) -> Optional[SwapRoute]:
        """
        获取Solana swap交易路由
        
        Args:
            token_in_address: 输入代币地址
            token_out_address: 输出代币地址  
            amount_in: 输入金额(lamports)
            from_address: 发送方钱包地址
            slippage: 滑点百分比 (默认1%)
            swap_mode: 交换模式
            enable_anti_mev: 是否启用反MEV
            gas_fee: Gas费用(SOL单位)
            
        Returns:
            Optional[SwapRoute]: 交换路由，失败时返回None
        """
        try:
            # 构建请求URL
            url = f"{self.base_url}/tx/get_swap_route"
            
            # 构建参数
            params = {
                'token_in_address': token_in_address,
                'token_out_address': token_out_address,
                'in_amount': amount_in,
                'from_address': from_address,
                'slippage': slippage,
                'swap_mode': swap_mode.value
            }
            
            # 添加可选参数
            if enable_anti_mev:
                params['is_anti_mev'] = 'true'
                # 反MEV需要最小费用
                if gas_fee is None:
                    gas_fee = max(self.connector.config.default_gas_fee, 0.002)
            
            if gas_fee is not None:
                params['fee'] = gas_fee
            
            # 添加合作伙伴标识
            params['partner'] = 'HermesFlow'
            
            logger.info(f"查询Solana swap路由: {token_in_address} -> {token_out_address}, 金额: {amount_in}")
            
            # 发起请求
            response = await self.connector._make_request('GET', url, params=params)
            
            if not response:
                logger.error("路由查询请求失败")
                return None
            
            if response.get('code') != 0:
                logger.error(f"路由查询失败: {response.get('msg', 'Unknown error')}")
                return None
            
            # 解析响应数据
            route_data = response.get('data')
            if not route_data:
                logger.error("路由数据为空")
                return None
            
            # 创建SwapRoute对象
            swap_route = create_swap_route_from_gmgn_data(route_data, self.chain)
            
            # 缓存路由
            cache_key = f"{token_in_address}:{token_out_address}:{amount_in}:{slippage}"
            self.connector._route_cache[cache_key] = (swap_route, datetime.now().timestamp())
            
            logger.info(f"路由查询成功，预计输出: {swap_route.quote.output_amount} lamports")
            return swap_route
            
        except Exception as e:
            logger.error(f"获取swap路由异常: {e}")
            return None
    
    async def submit_swap_transaction(
        self,
        signed_transaction: str,
        enable_anti_mev: bool = True
    ) -> Optional[TransactionResult]:
        """
        提交已签名的swap交易
        
        Args:
            signed_transaction: Base64编码的已签名交易
            enable_anti_mev: 是否启用反MEV
            
        Returns:
            Optional[TransactionResult]: 交易结果，失败时返回None
        """
        try:
            url = f"{self.connector.base_url}/txproxy/v1/send_transaction"
            
            # 构建请求数据
            data = {
                'chain': 'sol',
                'signed_tx': signed_transaction
            }
            
            if enable_anti_mev:
                data['isAntiMev'] = True
            
            logger.info("提交Solana swap交易")
            
            # 发起POST请求
            response = await self.connector._make_request('POST', url, data=data)
            
            if not response:
                logger.error("交易提交请求失败")
                return None
            
            if response.get('code') != 0:
                logger.error(f"交易提交失败: {response.get('msg', 'Unknown error')}")
                return None
            
            # 解析响应
            tx_data = response.get('data', {})
            tx_hash = tx_data.get('hash')
            
            if not tx_hash:
                logger.error("交易哈希为空")
                return None
            
            # 创建交易结果对象
            tx_result = TransactionResult(
                hash=tx_hash,
                chain=self.chain,
                status=TransactionStatus.PENDING,
                submitted_at=datetime.now()
            )
            
            # 添加到待监控列表
            self._pending_transactions[tx_hash] = tx_result
            
            logger.info(f"交易提交成功，哈希: {tx_hash}")
            return tx_result
            
        except Exception as e:
            logger.error(f"提交swap交易异常: {e}")
            return None
    
    async def get_transaction_status(
        self,
        transaction_hash: str,
        last_valid_block_height: int
    ) -> Optional[TransactionResult]:
        """
        查询交易状态
        
        Args:
            transaction_hash: 交易哈希
            last_valid_block_height: 最后有效区块高度
            
        Returns:
            Optional[TransactionResult]: 交易状态，失败时返回None
        """
        try:
            url = f"{self.base_url}/tx/get_transaction_status"
            
            params = {
                'hash': transaction_hash,
                'last_valid_height': last_valid_block_height
            }
            
            logger.debug(f"查询交易状态: {transaction_hash}")
            
            response = await self.connector._make_request('GET', url, params=params)
            
            if not response:
                logger.error("交易状态查询请求失败")
                return None
            
            if response.get('code') != 0:
                logger.error(f"交易状态查询失败: {response.get('msg', 'Unknown error')}")
                return None
            
            # 解析状态数据
            status_data = response.get('data', {})
            
            # 获取已有的交易结果或创建新的
            tx_result = self._pending_transactions.get(transaction_hash)
            if not tx_result:
                tx_result = TransactionResult(
                    hash=transaction_hash,
                    chain=self.chain,
                    status=TransactionStatus.PENDING
                )
            
            # 更新状态
            if status_data.get('success'):
                tx_result.status = TransactionStatus.SUCCESS
                tx_result.confirmed_at = datetime.now()
                logger.info(f"交易成功: {transaction_hash}")
            elif status_data.get('failed'):
                tx_result.status = TransactionStatus.FAILED
                tx_result.error_message = "Transaction failed on blockchain"
                logger.warning(f"交易失败: {transaction_hash}")
            elif status_data.get('expired'):
                tx_result.status = TransactionStatus.EXPIRED
                tx_result.error_message = "Transaction expired"
                logger.warning(f"交易过期: {transaction_hash}")
            
            # 如果交易已完成，从待监控列表移除
            if tx_result.status != TransactionStatus.PENDING:
                self._pending_transactions.pop(transaction_hash, None)
            
            return tx_result
            
        except Exception as e:
            logger.error(f"查询交易状态异常: {e}")
            return None
    
    async def wait_for_transaction(
        self,
        transaction_hash: str,
        last_valid_block_height: int,
        timeout: int = 60,
        check_interval: float = 1.0
    ) -> Optional[TransactionResult]:
        """
        等待交易确认
        
        Args:
            transaction_hash: 交易哈希
            last_valid_block_height: 最后有效区块高度
            timeout: 超时时间(秒)
            check_interval: 检查间隔(秒)
            
        Returns:
            Optional[TransactionResult]: 最终交易状态
        """
        start_time = datetime.now()
        timeout_delta = timedelta(seconds=timeout)
        
        logger.info(f"开始监控交易: {transaction_hash}")
        
        while datetime.now() - start_time < timeout_delta:
            try:
                tx_result = await self.get_transaction_status(
                    transaction_hash, 
                    last_valid_block_height
                )
                
                if tx_result and tx_result.status != TransactionStatus.PENDING:
                    logger.info(f"交易完成: {transaction_hash}, 状态: {tx_result.status.value}")
                    return tx_result
                
                # 等待下次检查
                await asyncio.sleep(check_interval)
                
            except Exception as e:
                logger.error(f"监控交易异常: {e}")
                await asyncio.sleep(check_interval)
        
        # 超时处理
        logger.warning(f"交易监控超时: {transaction_hash}")
        tx_result = self._pending_transactions.get(transaction_hash)
        if tx_result:
            tx_result.status = TransactionStatus.EXPIRED
            tx_result.error_message = "Monitoring timeout"
            self._pending_transactions.pop(transaction_hash, None)
        
        return tx_result
    
    async def execute_swap(
        self,
        token_in_address: str,
        token_out_address: str,
        amount_in: str,
        from_address: str,
        private_key: str,  # 注意：实际使用时需要安全处理
        slippage: float = 1.0,
        enable_anti_mev: bool = True,
        gas_fee: Optional[float] = None,
        wait_for_confirmation: bool = True
    ) -> Optional[TransactionResult]:
        """
        执行完整的swap交易流程
        
        注意：此方法需要私钥进行签名，实际使用时应该在客户端完成签名
        
        Args:
            token_in_address: 输入代币地址
            token_out_address: 输出代币地址
            amount_in: 输入金额(lamports)
            from_address: 发送方钱包地址
            private_key: 私钥(仅用于演示，实际应在客户端签名)
            slippage: 滑点百分比
            enable_anti_mev: 是否启用反MEV
            gas_fee: Gas费用
            wait_for_confirmation: 是否等待交易确认
            
        Returns:
            Optional[TransactionResult]: 交易执行结果
        """
        try:
            logger.info(f"开始执行swap: {token_in_address} -> {token_out_address}")
            
            # 第一步：获取交易路由
            swap_route = await self.get_swap_route(
                token_in_address=token_in_address,
                token_out_address=token_out_address,
                amount_in=amount_in,
                from_address=from_address,
                slippage=slippage,
                enable_anti_mev=enable_anti_mev,
                gas_fee=gas_fee
            )
            
            if not swap_route or not swap_route.swap_transaction:
                logger.error("获取交易路由失败")
                return None
            
            # 第二步：签名交易
            # 注意：这里应该调用外部签名服务或客户端签名
            # 为了演示，这里假设有一个签名函数
            signed_tx = await self._sign_transaction(
                swap_route.swap_transaction, 
                private_key
            )
            
            if not signed_tx:
                logger.error("交易签名失败")
                return None
            
            # 第三步：提交交易
            tx_result = await self.submit_swap_transaction(
                signed_tx, 
                enable_anti_mev
            )
            
            if not tx_result:
                logger.error("交易提交失败")
                return None
            
            # 第四步：等待确认(可选)
            if wait_for_confirmation and swap_route.last_valid_block_height:
                final_result = await self.wait_for_transaction(
                    tx_result.hash,
                    swap_route.last_valid_block_height
                )
                return final_result or tx_result
            
            return tx_result
            
        except Exception as e:
            logger.error(f"执行swap交易异常: {e}")
            return None
    
    async def _sign_transaction(
        self, 
        unsigned_tx_base64: str, 
        private_key: str
    ) -> Optional[str]:
        """
        签名交易 (演示用)
        
        实际实现中应该：
        1. 使用硬件钱包签名
        2. 在客户端本地签名
        3. 使用安全的密钥管理服务
        
        Args:
            unsigned_tx_base64: Base64编码的未签名交易
            private_key: 私钥
            
        Returns:
            Optional[str]: Base64编码的已签名交易
        """
        try:
            # 这里应该实现真正的Solana交易签名逻辑
            # 为了安全和简化，这里返回None，提示应在客户端签名
            logger.warning("交易签名应在客户端完成，不应在服务端处理私钥")
            return None
            
            # 实际签名代码示例（需要solana-py等库）:
            # from solana.transaction import VersionedTransaction
            # from solders.keypair import Keypair
            # import base64
            # 
            # # 解码交易
            # tx_bytes = base64.b64decode(unsigned_tx_base64)
            # transaction = VersionedTransaction.deserialize(tx_bytes)
            # 
            # # 创建密钥对
            # keypair = Keypair.from_base58_string(private_key)
            # 
            # # 签名
            # transaction.sign([keypair])
            # 
            # # 编码返回
            # return base64.b64encode(transaction.serialize()).decode()
            
        except Exception as e:
            logger.error(f"交易签名异常: {e}")
            return None
    
    def get_pending_transactions(self) -> Dict[str, TransactionResult]:
        """
        获取所有待确认交易
        
        Returns:
            Dict[str, TransactionResult]: 待确认交易列表
        """
        return self._pending_transactions.copy()
    
    def get_common_tokens(self) -> Dict[str, TokenInfo]:
        """
        获取Solana常用代币信息
        
        Returns:
            Dict[str, TokenInfo]: 常用代币字典
        """
        return {
            'SOL': TokenInfo(
                address=SolanaTokens.SOL,
                symbol='SOL',
                name='Solana',
                decimals=9,
                chain=ChainType.SOLANA,
                is_verified=True
            ),
            'USDC': TokenInfo(
                address=SolanaTokens.USDC,
                symbol='USDC',
                name='USD Coin',
                decimals=6,
                chain=ChainType.SOLANA,
                is_verified=True
            ),
            'USDT': TokenInfo(
                address=SolanaTokens.USDT,
                symbol='USDT',
                name='Tether USD',
                decimals=6,
                chain=ChainType.SOLANA,
                is_verified=True
            ),
            'RAY': TokenInfo(
                address=SolanaTokens.RAY,
                symbol='RAY',
                name='Raydium',
                decimals=6,
                chain=ChainType.SOLANA,
                is_verified=True
            )
        }
    
    async def get_token_price(
        self,
        token_address: str,
        quote_token: str = SolanaTokens.USDC
    ) -> Optional[float]:
        """
        获取代币价格 (通过小额swap查询实现)
        
        Args:
            token_address: 代币地址
            quote_token: 计价代币地址 (默认USDC)
            
        Returns:
            Optional[float]: 代币价格，失败时返回None
        """
        try:
            # 使用小额度查询价格 (0.001 SOL等值)
            test_amount = "1000000"  # 1 USDC (6 decimals)
            test_address = "11111111111111111111111111111111"  # 测试地址
            
            route = await self.get_swap_route(
                token_in_address=quote_token,
                token_out_address=token_address,
                amount_in=test_amount,
                from_address=test_address,
                slippage=5.0  # 使用较大滑点以获取报价
            )
            
            if route and route.quote.output_amount:
                # 计算价格 (USDC/Token)
                input_amount = float(route.quote.input_amount) / 1e6  # USDC 6 decimals
                output_amount = float(route.quote.output_amount) / 1e9  # 假设目标代币9 decimals
                
                if output_amount > 0:
                    price = input_amount / output_amount
                    logger.debug(f"代币 {token_address} 价格: ${price:.6f}")
                    return price
            
            return None
            
        except Exception as e:
            logger.error(f"获取代币价格异常: {e}")
            return None 