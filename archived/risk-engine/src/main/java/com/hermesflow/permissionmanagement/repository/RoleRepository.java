package com.hermesflow.permissionmanagement.repository;

import com.hermesflow.permissionmanagement.entity.Role;
import org.springframework.data.domain.Page;
import org.springframework.data.domain.Pageable;
import org.springframework.data.jpa.repository.JpaRepository;
import org.springframework.data.jpa.repository.Query;
import org.springframework.data.repository.query.Param;
import org.springframework.stereotype.Repository;

import java.util.List;
import java.util.Optional;
import java.util.Set;
import java.util.UUID;

/**
 * 角色Repository接口
 * 
 * @author HermesFlow Team
 * @version 1.0.0
 * @since 2024-12-30
 */
@Repository
public interface RoleRepository extends JpaRepository<Role, UUID> {

    /**
     * 根据租户ID和角色代码查找角色
     */
    Optional<Role> findByTenantIdAndCode(UUID tenantId, String code);

    /**
     * 根据租户ID查找所有角色
     */
    List<Role> findByTenantId(UUID tenantId);

    /**
     * 根据租户ID查找激活的角色
     */
    List<Role> findByTenantIdAndIsActiveTrue(UUID tenantId);

    /**
     * 根据租户ID和角色类型查找角色
     */
    List<Role> findByTenantIdAndRoleType(UUID tenantId, Role.RoleType roleType);

    /**
     * 根据租户ID和角色类型查找激活的角色
     */
    List<Role> findByTenantIdAndRoleTypeAndIsActiveTrue(UUID tenantId, Role.RoleType roleType);

    /**
     * 根据父角色ID查找子角色
     */
    List<Role> findByParentRoleId(UUID parentRoleId);

    /**
     * 根据父角色ID查找激活的子角色
     */
    List<Role> findByParentRoleIdAndIsActiveTrue(UUID parentRoleId);

    /**
     * 查找顶级角色 (没有父角色)
     */
    List<Role> findByTenantIdAndParentRoleIdIsNull(UUID tenantId);

    /**
     * 查找激活的顶级角色
     */
    List<Role> findByTenantIdAndParentRoleIdIsNullAndIsActiveTrue(UUID tenantId);

    /**
     * 根据租户ID分页查询角色
     */
    Page<Role> findByTenantId(UUID tenantId, Pageable pageable);

    /**
     * 根据租户ID分页查询激活的角色
     */
    Page<Role> findByTenantIdAndIsActiveTrue(UUID tenantId, Pageable pageable);

    /**
     * 模糊查询角色 (根据名称或代码)
     */
    @Query("SELECT r FROM Role r WHERE r.tenantId = :tenantId AND " +
           "(LOWER(r.name) LIKE LOWER(CONCAT('%', :keyword, '%')) OR " +
           "LOWER(r.code) LIKE LOWER(CONCAT('%', :keyword, '%')) OR " +
           "LOWER(r.description) LIKE LOWER(CONCAT('%', :keyword, '%')))")
    Page<Role> searchByTenantIdAndKeyword(@Param("tenantId") UUID tenantId, 
                                         @Param("keyword") String keyword, 
                                         Pageable pageable);

    /**
     * 根据角色代码列表查找角色
     */
    List<Role> findByTenantIdAndCodeIn(UUID tenantId, Set<String> codes);

    /**
     * 检查租户内角色代码是否存在
     */
    boolean existsByTenantIdAndCode(UUID tenantId, String code);

    /**
     * 检查角色是否有子角色
     */
    boolean existsByParentRoleId(UUID parentRoleId);

    /**
     * 统计租户内角色数量
     */
    long countByTenantId(UUID tenantId);

    /**
     * 统计租户内激活角色数量
     */
    long countByTenantIdAndIsActiveTrue(UUID tenantId);

    /**
     * 根据角色类型统计数量
     */
    long countByTenantIdAndRoleType(UUID tenantId, Role.RoleType roleType);

    /**
     * 统计角色分配的用户数量
     */
    @Query("SELECT COUNT(ur) FROM UserRole ur WHERE ur.roleId = :roleId AND ur.isActive = true")
    long countUsersByRoleId(@Param("roleId") UUID roleId);

    /**
     * 统计角色分配的活跃用户数量
     */
    @Query("SELECT COUNT(ur) FROM UserRole ur WHERE ur.roleId = :roleId AND ur.isActive = true " +
           "AND (ur.expiresAt IS NULL OR ur.expiresAt > CURRENT_TIMESTAMP)")
    long countActiveUsersByRoleId(@Param("roleId") UUID roleId);
} 