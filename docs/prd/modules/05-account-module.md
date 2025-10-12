# 账户模块详细需求文档

**模块名称**: 账户模块 (Account Module)  
**技术栈**: Java 21 + Spring Boot 3.x  
**版本**: v2.0.0  
**最后更新**: 2024-12-20

---

## 1. 模块概述

### 1.1 模块职责

1. **多账户管理**: 管理多个交易所账户
2. **API密钥管理**: 安全存储和使用API密钥
3. **资金划拨**: 交易所间资金转移
4. **账户同步**: 余额、资产实时同步
5. **手续费统计**: 交易成本分析

---

## 2. Epic详述

### Epic 1: 多账户管理 [P0]

#### 功能描述

支持绑定多个交易所账户，统一管理。

#### 用户故事

```gherkin
Feature: 绑定交易所账户
  作为一个交易者
  我想要绑定我的Binance账户
  以便进行自动化交易

Scenario: 添加Binance账户
  Given 我登录到系统
  When 我进入账户管理页面
  And 我点击"添加账户"
  And 我选择"Binance"
  And 我输入API Key和API Secret
  And 我点击"验证并保存"
  Then 系统应该验证API密钥有效性
  And 系统应该加密存储API密钥
  And 系统应该同步账户余额
  And 我应该看到账户已添加成功
```

#### 验收标准

- [ ] 支持至少5个交易所
- [ ] API密钥AES-256加密存储
- [ ] 支持API密钥权限设置（只读/交易/提现）
- [ ] 账户验证成功率 > 99%

---

### Epic 2: API密钥管理 [P0]

#### 技术实现

```java
@Service
public class ApiKeyService {
    
    @Autowired
    private ApiKeyRepository apiKeyRepository;
    
    @Autowired
    private EncryptionService encryptionService;
    
    /**
     * 保存API密钥
     */
    @Transactional
    public ApiKey saveApiKey(SaveApiKeyRequest request) {
        // 1. 验证API密钥
        boolean valid = validateApiKey(request);
        if (!valid) {
            throw new BusinessException("API密钥无效");
        }
        
        // 2. 加密存储
        byte[] encryptedKey = encryptionService.encrypt(request.getApiKey());
        byte[] encryptedSecret = encryptionService.encrypt(request.getApiSecret());
        
        // 3. 保存到数据库
        ApiKey apiKey = ApiKey.builder()
            .tenantId(SecurityContext.getCurrentTenant())
            .userId(SecurityContext.getCurrentUser())
            .exchange(request.getExchange())
            .apiKeyEncrypted(encryptedKey)
            .apiSecretEncrypted(encryptedSecret)
            .permissions(request.getPermissions())
            .build();
        
        return apiKeyRepository.save(apiKey);
    }
    
    /**
     * 获取解密后的API密钥
     */
    public ApiKeyPair getDecryptedApiKey(UUID tenantId, String exchange) {
        ApiKey apiKey = apiKeyRepository.findByTenantIdAndExchange(tenantId, exchange)
            .orElseThrow(() -> new ResourceNotFoundException("API密钥不存在"));
        
        String decryptedKey = encryptionService.decrypt(apiKey.getApiKeyEncrypted());
        String decryptedSecret = encryptionService.decrypt(apiKey.getApiSecretEncrypted());
        
        return new ApiKeyPair(decryptedKey, decryptedSecret);
    }
}
```

#### 验收标准

- [ ] API密钥加密存储
- [ ] 支持密钥轮换
- [ ] 最后使用时间追踪
- [ ] 密钥泄露检测

---

### Epic 3: 资金划拨 [P1]

#### 功能描述

支持交易所间资金转移。

#### 验收标准

- [ ] 支持内部划拨
- [ ] 支持提现到钱包
- [ ] 划拨记录完整

---

**文档维护者**: Account Team  
**最后更新**: 2024-12-20

