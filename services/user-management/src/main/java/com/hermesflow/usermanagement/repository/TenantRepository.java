package com.hermesflow.usermanagement.repository;

import com.hermesflow.usermanagement.entity.Tenant;
import org.springframework.data.domain.Page;
import org.springframework.data.domain.Pageable;
import org.springframework.data.jpa.repository.JpaRepository;
import org.springframework.data.jpa.repository.Query;
import org.springframework.data.repository.query.Param;
import org.springframework.stereotype.Repository;

import java.time.LocalDateTime;
import java.util.List;
import java.util.Optional;
import java.util.UUID;

/**
 * 租户数据访问接口
 * 提供租户相关的数据库操作方法
 */
@Repository
public interface TenantRepository extends JpaRepository<Tenant, UUID> {

    /**
     * 根据租户代码查找租户
     * @param code 租户代码
     * @return 租户信息
     */
    Optional<Tenant> findByCode(String code);

    /**
     * 根据状态查找租户列表
     * @param status 租户状态
     * @return 租户列表
     */
    List<Tenant> findByStatus(Tenant.TenantStatus status);

    /**
     * 查找所有活跃租户
     * @return 活跃租户列表
     */
    @Query("SELECT t FROM Tenant t WHERE t.status = 'ACTIVE'")
    List<Tenant> findActiveTenants();

    /**
     * 分页查询租户
     * @param status 租户状态（可选）
     * @param pageable 分页参数
     * @return 分页租户列表
     */
    @Query("SELECT t FROM Tenant t WHERE (:status IS NULL OR t.status = :status) ORDER BY t.createdAt DESC")
    Page<Tenant> findByStatusWithPaging(@Param("status") Tenant.TenantStatus status, Pageable pageable);

    /**
     * 根据租户名称模糊查询
     * @param name 租户名称关键字
     * @param pageable 分页参数
     * @return 分页租户列表
     */
    @Query("SELECT t FROM Tenant t WHERE t.name LIKE %:name% ORDER BY t.createdAt DESC")
    Page<Tenant> findByNameContaining(@Param("name") String name, Pageable pageable);

    /**
     * 查找指定时间之后创建的租户
     * @param createdAfter 创建时间
     * @return 租户列表
     */
    List<Tenant> findByCreatedAtAfter(LocalDateTime createdAfter);

    /**
     * 统计指定状态的租户数量
     * @param status 租户状态
     * @return 租户数量
     */
    long countByStatus(Tenant.TenantStatus status);

    /**
     * 检查租户代码是否存在
     * @param code 租户代码
     * @return 是否存在
     */
    boolean existsByCode(String code);

    /**
     * 检查租户名称是否存在
     * @param name 租户名称
     * @return 是否存在
     */
    boolean existsByName(String name);

    /**
     * 查找需要清理的非活跃租户
     * @param inactiveThreshold 非活跃时间阈值
     * @return 需要清理的租户列表
     */
    @Query("SELECT t FROM Tenant t WHERE t.status = 'INACTIVE' AND t.updatedAt < :inactiveThreshold")
    List<Tenant> findTenantsForCleanup(@Param("inactiveThreshold") LocalDateTime inactiveThreshold);
} 