"""
密钥管理器

安全管理HermesFlow系统的敏感信息：
- API密钥加密存储
- 数据库密码加密
- 配置文件密钥解密
- 密钥轮转支持

作者: HermesFlow Team
创建时间: 2024年12月26日
"""

import os
import base64
import logging
from typing import Optional, Dict, Any
from cryptography.fernet import Fernet
from cryptography.hazmat.primitives import hashes
from cryptography.hazmat.primitives.kdf.pbkdf2 import PBKDF2HMAC
import json
from pathlib import Path

logger = logging.getLogger(__name__)

class SecretManager:
    """密钥管理器"""
    
    def __init__(self, master_key: Optional[str] = None, key_file: str = ".hermesflow_key"):
        """
        初始化密钥管理器
        
        Args:
            master_key: 主密钥，如果未提供则从环境变量或文件加载
            key_file: 密钥文件路径
        """
        self.key_file = Path(key_file)
        self._fernet = None
        
        # 获取或生成主密钥
        if master_key:
            self._init_encryption(master_key)
        else:
            self._load_or_generate_key()
        
        logger.info("密钥管理器初始化完成")
    
    def _load_or_generate_key(self):
        """加载或生成主密钥"""
        # 首先尝试从环境变量获取
        master_key = os.getenv('HERMESFLOW_MASTER_KEY')
        if master_key:
            self._init_encryption(master_key)
            logger.info("从环境变量加载主密钥")
            return
        
        # 然后尝试从文件加载
        if self.key_file.exists():
            try:
                with open(self.key_file, 'rb') as f:
                    key_data = f.read()
                self._fernet = Fernet(key_data)
                logger.info("从文件加载主密钥")
                return
            except Exception as e:
                logger.error(f"加载密钥文件失败: {e}")
        
        # 生成新的主密钥
        self._generate_new_key()
    
    def _generate_new_key(self):
        """生成新的主密钥"""
        key = Fernet.generate_key()
        self._fernet = Fernet(key)
        
        try:
            # 保存密钥到文件
            self.key_file.parent.mkdir(parents=True, exist_ok=True)
            with open(self.key_file, 'wb') as f:
                f.write(key)
            
            # 设置文件权限（仅所有者可读写）
            os.chmod(self.key_file, 0o600)
            
            logger.info(f"生成新的主密钥并保存到: {self.key_file}")
        except Exception as e:
            logger.error(f"保存密钥文件失败: {e}")
    
    def _init_encryption(self, master_key: str):
        """初始化加密器"""
        try:
            # 如果是base64编码的密钥，直接使用
            if len(master_key) == 44 and master_key.endswith('='):
                key = master_key.encode()
            else:
                # 否则使用PBKDF2派生密钥
                password = master_key.encode()
                salt = b'hermesflow_salt_2024'  # 在生产环境中应该使用随机盐
                kdf = PBKDF2HMAC(
                    algorithm=hashes.SHA256(),
                    length=32,
                    salt=salt,
                    iterations=100000,
                )
                key = base64.urlsafe_b64encode(kdf.derive(password))
            
            self._fernet = Fernet(key)
        except Exception as e:
            logger.error(f"初始化加密器失败: {e}")
            raise
    
    def encrypt(self, plaintext: str) -> str:
        """
        加密明文
        
        Args:
            plaintext: 明文字符串
            
        Returns:
            str: 加密后的base64字符串
        """
        if not self._fernet:
            raise RuntimeError("加密器未初始化")
        
        try:
            encrypted_data = self._fernet.encrypt(plaintext.encode())
            return base64.urlsafe_b64encode(encrypted_data).decode()
        except Exception as e:
            logger.error(f"加密失败: {e}")
            raise
    
    def decrypt(self, ciphertext: str) -> str:
        """
        解密密文
        
        Args:
            ciphertext: 加密的base64字符串
            
        Returns:
            str: 解密后的明文字符串
        """
        if not self._fernet:
            raise RuntimeError("加密器未初始化")
        
        try:
            encrypted_data = base64.urlsafe_b64decode(ciphertext.encode())
            decrypted_data = self._fernet.decrypt(encrypted_data)
            return decrypted_data.decode()
        except Exception as e:
            logger.error(f"解密失败: {e}")
            raise
    
    def encrypt_config_value(self, value: str) -> str:
        """
        加密配置值（添加ENC:前缀）
        
        Args:
            value: 配置值
            
        Returns:
            str: 加密后的配置值，格式为 ENC:encrypted_value
        """
        encrypted = self.encrypt(value)
        return f"ENC:{encrypted}"
    
    def is_encrypted(self, value: str) -> bool:
        """
        检查值是否已加密
        
        Args:
            value: 值
            
        Returns:
            bool: 是否已加密
        """
        return value.startswith('ENC:')
    
    def decrypt_config_value(self, value: str) -> str:
        """
        解密配置值
        
        Args:
            value: 配置值，可能包含ENC:前缀
            
        Returns:
            str: 解密后的值
        """
        if self.is_encrypted(value):
            return self.decrypt(value[4:])  # 移除ENC:前缀
        return value
    
    def encrypt_connector_credentials(self, credentials: Dict[str, str]) -> Dict[str, str]:
        """
        加密连接器凭据
        
        Args:
            credentials: 凭据字典，包含api_key, api_secret等
            
        Returns:
            Dict[str, str]: 加密后的凭据字典
        """
        encrypted_credentials = {}
        
        for key, value in credentials.items():
            if key in ['api_key', 'api_secret', 'password', 'token']:
                encrypted_credentials[key] = self.encrypt_config_value(value)
            else:
                encrypted_credentials[key] = value
        
        return encrypted_credentials
    
    def decrypt_connector_credentials(self, credentials: Dict[str, str]) -> Dict[str, str]:
        """
        解密连接器凭据
        
        Args:
            credentials: 加密的凭据字典
            
        Returns:
            Dict[str, str]: 解密后的凭据字典
        """
        decrypted_credentials = {}
        
        for key, value in credentials.items():
            if isinstance(value, str) and self.is_encrypted(value):
                decrypted_credentials[key] = self.decrypt_config_value(value)
            else:
                decrypted_credentials[key] = value
        
        return decrypted_credentials
    
    def save_encrypted_config(self, config_data: Dict[str, Any], filepath: str):
        """
        保存加密的配置文件
        
        Args:
            config_data: 配置数据
            filepath: 文件路径
        """
        try:
            # 加密整个配置
            config_json = json.dumps(config_data, indent=2)
            encrypted_config = self.encrypt(config_json)
            
            # 保存到文件
            with open(filepath, 'w') as f:
                f.write(encrypted_config)
            
            logger.info(f"保存加密配置到: {filepath}")
        except Exception as e:
            logger.error(f"保存加密配置失败: {e}")
            raise
    
    def load_encrypted_config(self, filepath: str) -> Dict[str, Any]:
        """
        加载加密的配置文件
        
        Args:
            filepath: 文件路径
            
        Returns:
            Dict[str, Any]: 解密后的配置数据
        """
        try:
            # 读取加密文件
            with open(filepath, 'r') as f:
                encrypted_config = f.read()
            
            # 解密配置
            config_json = self.decrypt(encrypted_config)
            config_data = json.loads(config_json)
            
            logger.info(f"加载加密配置从: {filepath}")
            return config_data
        except Exception as e:
            logger.error(f"加载加密配置失败: {e}")
            raise
    
    def rotate_key(self, new_master_key: Optional[str] = None):
        """
        轮转主密钥
        
        Args:
            new_master_key: 新的主密钥，如果未提供则生成
        """
        logger.info("开始密钥轮转...")
        
        # 备份当前密钥文件
        if self.key_file.exists():
            backup_file = self.key_file.with_suffix('.bak')
            self.key_file.rename(backup_file)
            logger.info(f"备份当前密钥到: {backup_file}")
        
        try:
            # 生成新密钥
            if new_master_key:
                self._init_encryption(new_master_key)
            else:
                self._generate_new_key()
            
            logger.info("密钥轮转完成")
        except Exception as e:
            # 如果轮转失败，恢复备份
            if backup_file.exists():
                backup_file.rename(self.key_file)
                logger.error(f"密钥轮转失败，已恢复备份: {e}")
            raise
    
    def validate_key(self) -> bool:
        """
        验证当前密钥是否有效
        
        Returns:
            bool: 密钥是否有效
        """
        try:
            # 尝试加密和解密测试字符串
            test_string = "hermesflow_test_encryption"
            encrypted = self.encrypt(test_string)
            decrypted = self.decrypt(encrypted)
            
            return decrypted == test_string
        except Exception as e:
            logger.error(f"密钥验证失败: {e}")
            return False
    
    def get_key_info(self) -> Dict[str, Any]:
        """
        获取密钥信息
        
        Returns:
            Dict[str, Any]: 密钥信息
        """
        return {
            'key_file_exists': self.key_file.exists(),
            'key_file_path': str(self.key_file),
            'encryption_initialized': self._fernet is not None,
            'key_valid': self.validate_key()
        } 