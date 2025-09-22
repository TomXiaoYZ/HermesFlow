package com.hermesflow.usermanagement.service;

import com.hermesflow.usermanagement.entity.Tenant;
import com.hermesflow.usermanagement.entity.TenantConfig;
import com.hermesflow.usermanagement.repository.TenantConfigRepository;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.stereotype.Service;
import org.springframework.transaction.annotation.Transactional;

import java.util.List;
import java.util.Map;
import java.util.Optional;
import java.util.stream.Collectors;

/**
 * 租户配置服务类
 * 提供租户配置管理的核心业务逻辑
 */
@Service
@Transactional
public class TenantConfigService {

    private static final Logger logger = LoggerFactory.getLogger(TenantConfigService.class);

    private final TenantConfigRepository tenantConfigRepository;
    private final TenantService tenantService;

    @Autowired
    public TenantConfigService(TenantConfigRepository tenantConfigRepository,
                              TenantService tenantService) {
        this.tenantConfigRepository = tenantConfigRepository;
        this.tenantService = tenantService;
    }

    /**
     * 设置租户配置
     * @param tenantCode 租户代码
     * @param configKey 配置键
     * @param configValue 配置值
     * @return 配置项
     */
    public TenantConfig setConfig(String tenantCode, String configKey, String configValue) {
        logger.info("设置租户配置: {} - {} = {}", tenantCode, configKey, configValue);

        Tenant tenant = tenantService.getTenantByCodeRequired(tenantCode);

        // 查找现有配置
        Optional<TenantConfig> existingConfig = tenantConfigRepository
                .findByTenantAndConfigKey(tenant, configKey);

        TenantConfig config;
        if (existingConfig.isPresent()) {
            // 更新现有配置
            config = existingConfig.get();
            config.setConfigValue(configValue);
            logger.info("更新租户配置: {} - {}", tenantCode, configKey);
        } else {
            // 创建新配置
            config = new TenantConfig(tenant, configKey, configValue);
            logger.info("创建租户配置: {} - {}", tenantCode, configKey);
        }

        TenantConfig savedConfig = tenantConfigRepository.save(config);
        logger.info("租户配置保存成功: {} - {}", tenantCode, configKey);

        return savedConfig;
    }

    /**
     * 批量设置租户配置
     * @param tenantCode 租户代码
     * @param configs 配置映射
     * @return 配置项列表
     */
    public List<TenantConfig> setConfigs(String tenantCode, Map<String, String> configs) {
        logger.info("批量设置租户配置: {} - {} 项", tenantCode, configs.size());

        Tenant tenant = tenantService.getTenantByCodeRequired(tenantCode);

        List<TenantConfig> savedConfigs = configs.entrySet().stream()
                .map(entry -> {
                    String configKey = entry.getKey();
                    String configValue = entry.getValue();

                    // 查找现有配置
                    Optional<TenantConfig> existingConfig = tenantConfigRepository
                            .findByTenantAndConfigKey(tenant, configKey);

                    TenantConfig config;
                    if (existingConfig.isPresent()) {
                        // 更新现有配置
                        config = existingConfig.get();
                        config.setConfigValue(configValue);
                    } else {
                        // 创建新配置
                        config = new TenantConfig(tenant, configKey, configValue);
                    }

                    return tenantConfigRepository.save(config);
                })
                .collect(Collectors.toList());

        logger.info("批量租户配置保存成功: {} - {} 项", tenantCode, savedConfigs.size());
        return savedConfigs;
    }

    /**
     * 获取租户配置
     * @param tenantCode 租户代码
     * @param configKey 配置键
     * @return 配置值
     */
    @Transactional(readOnly = true)
    public Optional<String> getConfig(String tenantCode, String configKey) {
        Tenant tenant = tenantService.getTenantByCodeRequired(tenantCode);
        
        Optional<TenantConfig> config = tenantConfigRepository
                .findByTenantAndConfigKey(tenant, configKey);
        
        return config.map(TenantConfig::getConfigValue);
    }

    /**
     * 获取租户配置（带默认值）
     * @param tenantCode 租户代码
     * @param configKey 配置键
     * @param defaultValue 默认值
     * @return 配置值
     */
    @Transactional(readOnly = true)
    public String getConfig(String tenantCode, String configKey, String defaultValue) {
        return getConfig(tenantCode, configKey).orElse(defaultValue);
    }

    /**
     * 获取租户的所有配置
     * @param tenantCode 租户代码
     * @return 配置映射
     */
    @Transactional(readOnly = true)
    public Map<String, String> getAllConfigs(String tenantCode) {
        Tenant tenant = tenantService.getTenantByCodeRequired(tenantCode);
        
        List<TenantConfig> configs = tenantConfigRepository.findByTenant(tenant);
        
        return configs.stream()
                .collect(Collectors.toMap(
                        TenantConfig::getConfigKey,
                        TenantConfig::getConfigValue
                ));
    }

    /**
     * 获取租户的所有配置对象
     * @param tenantCode 租户代码
     * @return 配置对象列表
     */
    @Transactional(readOnly = true)
    public List<TenantConfig> getAllConfigObjects(String tenantCode) {
        Tenant tenant = tenantService.getTenantByCodeRequired(tenantCode);
        return tenantConfigRepository.findByTenant(tenant);
    }

    /**
     * 根据配置键前缀获取配置
     * @param tenantCode 租户代码
     * @param keyPrefix 配置键前缀
     * @return 配置映射
     */
    @Transactional(readOnly = true)
    public Map<String, String> getConfigsByKeyPrefix(String tenantCode, String keyPrefix) {
        Tenant tenant = tenantService.getTenantByCodeRequired(tenantCode);
        
        List<TenantConfig> configs = tenantConfigRepository
                .findByTenantAndConfigKeyStartingWith(tenant, keyPrefix);
        
        return configs.stream()
                .collect(Collectors.toMap(
                        TenantConfig::getConfigKey,
                        TenantConfig::getConfigValue
                ));
    }

    /**
     * 删除租户配置
     * @param tenantCode 租户代码
     * @param configKey 配置键
     * @return 是否删除成功
     */
    public boolean deleteConfig(String tenantCode, String configKey) {
        logger.info("删除租户配置: {} - {}", tenantCode, configKey);

        Tenant tenant = tenantService.getTenantByCodeRequired(tenantCode);
        
        Optional<TenantConfig> config = tenantConfigRepository
                .findByTenantAndConfigKey(tenant, configKey);
        
        if (config.isPresent()) {
            tenantConfigRepository.delete(config.get());
            logger.info("租户配置删除成功: {} - {}", tenantCode, configKey);
            return true;
        } else {
            logger.warn("租户配置不存在: {} - {}", tenantCode, configKey);
            return false;
        }
    }

    /**
     * 批量删除租户配置
     * @param tenantCode 租户代码
     * @param configKeys 配置键列表
     * @return 删除的配置数量
     */
    public int deleteConfigs(String tenantCode, List<String> configKeys) {
        logger.info("批量删除租户配置: {} - {} 项", tenantCode, configKeys.size());

        Tenant tenant = tenantService.getTenantByCodeRequired(tenantCode);
        
        int deletedCount = 0;
        for (String configKey : configKeys) {
            Optional<TenantConfig> config = tenantConfigRepository
                    .findByTenantAndConfigKey(tenant, configKey);
            
            if (config.isPresent()) {
                tenantConfigRepository.delete(config.get());
                deletedCount++;
            }
        }

        logger.info("批量删除租户配置完成: {} - 删除了 {} 项", tenantCode, deletedCount);
        return deletedCount;
    }

    /**
     * 删除租户的所有配置
     * @param tenantCode 租户代码
     * @return 删除的配置数量
     */
    public int deleteAllConfigs(String tenantCode) {
        logger.info("删除租户所有配置: {}", tenantCode);

        Tenant tenant = tenantService.getTenantByCodeRequired(tenantCode);
        
        List<TenantConfig> configs = tenantConfigRepository.findByTenant(tenant);
        int count = configs.size();
        
        if (count > 0) {
            tenantConfigRepository.deleteAll(configs);
            logger.info("删除租户所有配置完成: {} - 删除了 {} 项", tenantCode, count);
        }

        return count;
    }

    /**
     * 检查配置是否存在
     * @param tenantCode 租户代码
     * @param configKey 配置键
     * @return 是否存在
     */
    @Transactional(readOnly = true)
    public boolean configExists(String tenantCode, String configKey) {
        Tenant tenant = tenantService.getTenantByCodeRequired(tenantCode);
        return tenantConfigRepository.existsByTenantAndConfigKey(tenant, configKey);
    }

    /**
     * 统计租户配置数量
     * @param tenantCode 租户代码
     * @return 配置数量
     */
    @Transactional(readOnly = true)
    public long countConfigs(String tenantCode) {
        Tenant tenant = tenantService.getTenantByCodeRequired(tenantCode);
        return tenantConfigRepository.countByTenant(tenant);
    }

    /**
     * 根据配置值搜索配置
     * @param tenantCode 租户代码
     * @param valuePattern 配置值模式
     * @return 配置列表
     */
    @Transactional(readOnly = true)
    public List<TenantConfig> searchConfigsByValue(String tenantCode, String valuePattern) {
        Tenant tenant = tenantService.getTenantByCodeRequired(tenantCode);
        return tenantConfigRepository.findByTenantAndConfigValueContaining(tenant, valuePattern);
    }

    /**
     * 复制配置到另一个租户
     * @param sourceTenantCode 源租户代码
     * @param targetTenantCode 目标租户代码
     * @param configKeys 要复制的配置键列表（为空则复制所有）
     * @return 复制的配置数量
     */
    public int copyConfigs(String sourceTenantCode, String targetTenantCode, List<String> configKeys) {
        logger.info("复制租户配置: {} -> {}", sourceTenantCode, targetTenantCode);

        Tenant sourceTenant = tenantService.getTenantByCodeRequired(sourceTenantCode);
        Tenant targetTenant = tenantService.getTenantByCodeRequired(targetTenantCode);

        List<TenantConfig> sourceConfigs;
        if (configKeys == null || configKeys.isEmpty()) {
            // 复制所有配置
            sourceConfigs = tenantConfigRepository.findByTenant(sourceTenant);
        } else {
            // 复制指定配置
            sourceConfigs = configKeys.stream()
                    .map(key -> tenantConfigRepository.findByTenantAndConfigKey(sourceTenant, key))
                    .filter(Optional::isPresent)
                    .map(Optional::get)
                    .collect(Collectors.toList());
        }

        int copiedCount = 0;
        for (TenantConfig sourceConfig : sourceConfigs) {
            // 检查目标租户是否已有该配置
            Optional<TenantConfig> existingConfig = tenantConfigRepository
                    .findByTenantAndConfigKey(targetTenant, sourceConfig.getConfigKey());

            if (existingConfig.isPresent()) {
                // 更新现有配置
                TenantConfig config = existingConfig.get();
                config.setConfigValue(sourceConfig.getConfigValue());
                tenantConfigRepository.save(config);
            } else {
                // 创建新配置
                TenantConfig newConfig = new TenantConfig(
                        targetTenant,
                        sourceConfig.getConfigKey(),
                        sourceConfig.getConfigValue()
                );
                tenantConfigRepository.save(newConfig);
            }
            copiedCount++;
        }

        logger.info("复制租户配置完成: {} -> {} - 复制了 {} 项", 
                   sourceTenantCode, targetTenantCode, copiedCount);
        return copiedCount;
    }
} 