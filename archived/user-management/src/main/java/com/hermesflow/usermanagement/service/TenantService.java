package com.hermesflow.usermanagement.service;

import com.hermesflow.usermanagement.dto.TenantCreateRequest;
import com.hermesflow.usermanagement.entity.Tenant;
import com.hermesflow.usermanagement.repository.TenantRepository;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.data.domain.Page;
import org.springframework.data.domain.Pageable;
import org.springframework.stereotype.Service;
import org.springframework.transaction.annotation.Transactional;

import java.time.LocalDateTime;
import java.util.List;
import java.util.Optional;

/**
 * 租户服务类
 * 提供租户管理的核心业务逻辑
 */
@Service
@Transactional
public class TenantService {

    private static final Logger logger = LoggerFactory.getLogger(TenantService.class);

    private final TenantRepository tenantRepository;

    @Autowired
    public TenantService(TenantRepository tenantRepository) {
        this.tenantRepository = tenantRepository;
    }

    /**
     * 创建租户
     * @param request 租户创建请求
     * @return 创建的租户
     */
    public Tenant createTenant(TenantCreateRequest request) {
        logger.info("创建租户: {}", request.getTenantCode());

        // 检查租户代码是否已存在
        if (tenantRepository.existsByCode(request.getTenantCode())) {
            throw new IllegalArgumentException("租户代码已存在: " + request.getTenantCode());
        }

        // 检查租户名称是否已存在
        if (tenantRepository.existsByName(request.getName())) {
            throw new IllegalArgumentException("租户名称已存在: " + request.getName());
        }

        // 创建租户实体
        Tenant tenant = new Tenant(
            request.getName(),
            request.getTenantCode(),
            Tenant.PlanType.BASIC
        );
        
        if (request.getDescription() != null) {
            tenant.setDescription(request.getDescription());
        }

        // 保存租户
        Tenant savedTenant = tenantRepository.save(tenant);
        logger.info("租户创建成功: {} (ID: {})", savedTenant.getCode(), savedTenant.getId());

        return savedTenant;
    }

    /**
     * 根据租户代码获取租户
     * @param tenantCode 租户代码
     * @return 租户信息
     */
    @Transactional(readOnly = true)
    public Optional<Tenant> getTenantByCode(String tenantCode) {
        return tenantRepository.findByCode(tenantCode);
    }

    /**
     * 根据租户代码获取租户（必须存在）
     * @param tenantCode 租户代码
     * @return 租户信息
     * @throws IllegalArgumentException 如果租户不存在
     */
    @Transactional(readOnly = true)
    public Tenant getTenantByCodeRequired(String tenantCode) {
        return tenantRepository.findByCode(tenantCode)
                .orElseThrow(() -> new IllegalArgumentException("租户不存在: " + tenantCode));
    }

    /**
     * 更新租户信息
     * @param tenantCode 租户代码
     * @param request 更新请求
     * @return 更新后的租户
     */
    public Tenant updateTenant(String tenantCode, TenantCreateRequest request) {
        logger.info("更新租户: {}", tenantCode);

        Tenant tenant = getTenantByCodeRequired(tenantCode);

        // 检查名称是否与其他租户冲突
        if (!tenant.getName().equals(request.getName()) && 
            tenantRepository.existsByName(request.getName())) {
            throw new IllegalArgumentException("租户名称已存在: " + request.getName());
        }

        // 更新租户信息
        tenant.setName(request.getName());
        tenant.setDescription(request.getDescription());

        Tenant updatedTenant = tenantRepository.save(tenant);
        logger.info("租户更新成功: {}", tenantCode);

        return updatedTenant;
    }

    /**
     * 暂停租户
     * @param tenantCode 租户代码
     * @return 更新后的租户
     */
    public Tenant suspendTenant(String tenantCode) {
        logger.info("暂停租户: {}", tenantCode);

        Tenant tenant = getTenantByCodeRequired(tenantCode);
        
        if (tenant.getStatus() == Tenant.TenantStatus.SUSPENDED) {
            throw new IllegalStateException("租户已处于暂停状态: " + tenantCode);
        }

        tenant.setStatus(Tenant.TenantStatus.SUSPENDED);
        Tenant updatedTenant = tenantRepository.save(tenant);
        
        logger.info("租户暂停成功: {}", tenantCode);
        return updatedTenant;
    }

    /**
     * 激活租户
     * @param tenantCode 租户代码
     * @return 更新后的租户
     */
    public Tenant activateTenant(String tenantCode) {
        logger.info("激活租户: {}", tenantCode);

        Tenant tenant = getTenantByCodeRequired(tenantCode);
        
        if (tenant.getStatus() == Tenant.TenantStatus.ACTIVE) {
            throw new IllegalStateException("租户已处于激活状态: " + tenantCode);
        }

        tenant.setStatus(Tenant.TenantStatus.ACTIVE);
        Tenant updatedTenant = tenantRepository.save(tenant);
        
        logger.info("租户激活成功: {}", tenantCode);
        return updatedTenant;
    }

    /**
     * 获取所有租户（分页）
     * @param pageable 分页参数
     * @return 分页租户列表
     */
    @Transactional(readOnly = true)
    public Page<Tenant> getAllTenants(Pageable pageable) {
        return tenantRepository.findByStatusWithPaging(null, pageable);
    }

    /**
     * 根据状态获取租户（分页）
     * @param status 租户状态
     * @param pageable 分页参数
     * @return 分页租户列表
     */
    @Transactional(readOnly = true)
    public Page<Tenant> getTenantsByStatus(Tenant.TenantStatus status, Pageable pageable) {
        return tenantRepository.findByStatusWithPaging(status, pageable);
    }

    /**
     * 根据名称搜索租户
     * @param name 租户名称关键字
     * @param pageable 分页参数
     * @return 分页租户列表
     */
    @Transactional(readOnly = true)
    public Page<Tenant> searchTenantsByName(String name, Pageable pageable) {
        return tenantRepository.findByNameContaining(name, pageable);
    }

    /**
     * 获取活跃租户列表
     * @return 活跃租户列表
     */
    @Transactional(readOnly = true)
    public List<Tenant> getActiveTenants() {
        return tenantRepository.findActiveTenants();
    }

    /**
     * 统计租户数量
     * @param status 租户状态（可选）
     * @return 租户数量
     */
    @Transactional(readOnly = true)
    public long countTenants(Tenant.TenantStatus status) {
        if (status == null) {
            return tenantRepository.count();
        }
        return tenantRepository.countByStatus(status);
    }

    /**
     * 检查租户是否存在
     * @param tenantCode 租户代码
     * @return 是否存在
     */
    @Transactional(readOnly = true)
    public boolean existsByTenantCode(String tenantCode) {
        return tenantRepository.existsByCode(tenantCode);
    }

    /**
     * 清理非活跃租户
     * @param inactiveDays 非活跃天数阈值
     * @return 清理的租户列表
     */
    public List<Tenant> cleanupInactiveTenants(int inactiveDays) {
        logger.info("开始清理{}天未活跃的租户", inactiveDays);

        LocalDateTime threshold = LocalDateTime.now().minusDays(inactiveDays);
        List<Tenant> tenantsToCleanup = tenantRepository.findTenantsForCleanup(threshold);

        if (!tenantsToCleanup.isEmpty()) {
            logger.info("找到{}个需要清理的租户", tenantsToCleanup.size());
            // 这里可以添加具体的清理逻辑，比如删除或标记为已清理
        }

        return tenantsToCleanup;
    }
} 