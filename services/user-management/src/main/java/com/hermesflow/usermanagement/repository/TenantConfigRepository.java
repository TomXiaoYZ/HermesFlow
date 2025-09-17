package com.hermesflow.usermanagement.repository;

import com.hermesflow.usermanagement.entity.Tenant;
import com.hermesflow.usermanagement.entity.TenantConfig;
import org.springframework.data.domain.Page;
import org.springframework.data.domain.Pageable;
import org.springframework.data.jpa.repository.JpaRepository;
import org.springframework.data.jpa.repository.Query;
import org.springframework.data.repository.query.Param;
import org.springframework.stereotype.Repository;

import java.util.List;
import java.util.Optional;
import java.util.UUID;

/**
 * 租户配置数据访问接口
 * 提供租户配置相关的数据库操作方法
 */
@Repository
public interface TenantConfigRepository extends JpaRepository<TenantConfig, UUID> {

    /**
     * 根据租户和配置键查找配置
     * @param tenant 租户
     * @param configKey 配置键
     * @return 配置信息
     */
    Optional<TenantConfig> findByTenantAndConfigKey(Tenant tenant, String configKey);

    /**
     * 查找租户的所有配置
     * @param tenant 租户
     * @return 配置列表
     */
    List<TenantConfig> findByTenant(Tenant tenant);

    /**
     * 分页查询租户的配置
     * @param tenant 租户
     * @param pageable 分页参数
     * @return 分页配置列表
     */
    Page<TenantConfig> findByTenant(Tenant tenant, Pageable pageable);

    /**
     * 根据配置键前缀查找租户配置
     * @param tenant 租户
     * @param keyPrefix 配置键前缀
     * @return 配置列表
     */
    @Query("SELECT tc FROM TenantConfig tc WHERE tc.tenant = :tenant AND tc.configKey LIKE :keyPrefix%")
    List<TenantConfig> findByTenantAndConfigKeyStartingWith(@Param("tenant") Tenant tenant, 
                                                           @Param("keyPrefix") String keyPrefix);

    /**
     * 根据配置值查找租户配置
     * @param tenant 租户
     * @param configValue 配置值
     * @return 配置列表
     */
    List<TenantConfig> findByTenantAndConfigValue(Tenant tenant, String configValue);

    /**
     * 根据配置值模糊查询租户配置
     * @param tenant 租户
     * @param valuePattern 配置值模式
     * @return 配置列表
     */
    @Query("SELECT tc FROM TenantConfig tc WHERE tc.tenant = :tenant AND tc.configValue LIKE %:valuePattern%")
    List<TenantConfig> findByTenantAndConfigValueContaining(@Param("tenant") Tenant tenant, 
                                                           @Param("valuePattern") String valuePattern);

    /**
     * 查找租户的系统配置（以system.开头的配置）
     * @param tenant 租户
     * @return 系统配置列表
     */
    @Query("SELECT tc FROM TenantConfig tc WHERE tc.tenant = :tenant AND tc.configKey LIKE 'system.%'")
    List<TenantConfig> findSystemConfigsByTenant(@Param("tenant") Tenant tenant);

    /**
     * 查找租户的用户配置（以user.开头的配置）
     * @param tenant 租户
     * @return 用户配置列表
     */
    @Query("SELECT tc FROM TenantConfig tc WHERE tc.tenant = :tenant AND tc.configKey LIKE 'user.%'")
    List<TenantConfig> findUserConfigsByTenant(@Param("tenant") Tenant tenant);

    /**
     * 查找租户的安全配置（以security.开头的配置）
     * @param tenant 租户
     * @return 安全配置列表
     */
    @Query("SELECT tc FROM TenantConfig tc WHERE tc.tenant = :tenant AND tc.configKey LIKE 'security.%'")
    List<TenantConfig> findSecurityConfigsByTenant(@Param("tenant") Tenant tenant);

    /**
     * 统计租户的配置数量
     * @param tenant 租户
     * @return 配置数量
     */
    long countByTenant(Tenant tenant);

    /**
     * 检查租户的配置键是否存在
     * @param tenant 租户
     * @param configKey 配置键
     * @return 是否存在
     */
    boolean existsByTenantAndConfigKey(Tenant tenant, String configKey);

    /**
     * 删除租户的指定配置
     * @param tenant 租户
     * @param configKey 配置键
     */
    void deleteByTenantAndConfigKey(Tenant tenant, String configKey);

    /**
     * 删除租户的所有配置
     * @param tenant 租户
     */
    void deleteByTenant(Tenant tenant);

    /**
     * 根据配置键模糊查询租户配置
     * @param tenant 租户
     * @param keyPattern 配置键模式
     * @param pageable 分页参数
     * @return 分页配置列表
     */
    @Query("SELECT tc FROM TenantConfig tc WHERE tc.tenant = :tenant AND tc.configKey LIKE %:keyPattern% ORDER BY tc.configKey")
    Page<TenantConfig> findByTenantAndConfigKeyContaining(@Param("tenant") Tenant tenant, 
                                                         @Param("keyPattern") String keyPattern, 
                                                         Pageable pageable);

    /**
     * 查找所有租户的指定配置键
     * @param configKey 配置键
     * @return 配置列表
     */
    List<TenantConfig> findByConfigKey(String configKey);

    /**
     * 批量查询租户的多个配置
     * @param tenant 租户
     * @param configKeys 配置键列表
     * @return 配置列表
     */
    @Query("SELECT tc FROM TenantConfig tc WHERE tc.tenant = :tenant AND tc.configKey IN :configKeys")
    List<TenantConfig> findByTenantAndConfigKeyIn(@Param("tenant") Tenant tenant, 
                                                 @Param("configKeys") List<String> configKeys);
} 