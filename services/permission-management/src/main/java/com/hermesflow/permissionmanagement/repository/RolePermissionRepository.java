package com.hermesflow.permissionmanagement.repository;

import com.hermesflow.permissionmanagement.entity.RolePermission;
import org.springframework.data.domain.Page;
import org.springframework.data.domain.Pageable;
import org.springframework.data.jpa.repository.JpaRepository;
import org.springframework.data.jpa.repository.Query;
import org.springframework.data.repository.query.Param;
import org.springframework.stereotype.Repository;

import java.time.LocalDateTime;
import java.util.List;
import java.util.Optional;
import java.util.Set;
import java.util.UUID;

/**
 * 角色权限关联Repository接口
 * 
 * @author HermesFlow Team
 * @version 1.0.0
 * @since 2024-12-30
 */
@Repository
public interface RolePermissionRepository extends JpaRepository<RolePermission, UUID> {

    /**
     * 根据角色ID和租户ID查找角色权限
     */
    List<RolePermission> findByRoleIdAndTenantId(UUID roleId, UUID tenantId);

    /**
     * 根据角色ID和租户ID查找激活的角色权限
     */
    List<RolePermission> findByRoleIdAndTenantIdAndIsActiveTrue(UUID roleId, UUID tenantId);

    /**
     * 根据角色ID、权限ID和租户ID查找角色权限
     */
    Optional<RolePermission> findByRoleIdAndPermissionIdAndTenantId(UUID roleId, UUID permissionId, UUID tenantId);

    /**
     * 根据权限ID查找所有角色权限
     */
    List<RolePermission> findByPermissionId(UUID permissionId);

    /**
     * 根据权限ID查找激活的角色权限
     */
    List<RolePermission> findByPermissionIdAndIsActiveTrue(UUID permissionId);

    /**
     * 根据租户ID查找所有角色权限
     */
    List<RolePermission> findByTenantId(UUID tenantId);

    /**
     * 根据租户ID分页查询角色权限
     */
    Page<RolePermission> findByTenantId(UUID tenantId, Pageable pageable);

    /**
     * 根据角色ID和租户ID分页查询角色权限
     */
    Page<RolePermission> findByRoleIdAndTenantId(UUID roleId, UUID tenantId, Pageable pageable);

    /**
     * 根据用户ID查询用户的所有权限 (通过角色)
     */
    @Query("SELECT DISTINCT rp FROM RolePermission rp " +
           "JOIN UserRole ur ON rp.roleId = ur.roleId " +
           "WHERE ur.userId = :userId AND ur.tenantId = :tenantId " +
           "AND ur.isActive = true AND rp.isActive = true " +
           "AND (ur.expiresAt IS NULL OR ur.expiresAt > CURRENT_TIMESTAMP)")
    List<RolePermission> findUserPermissions(@Param("userId") UUID userId, @Param("tenantId") UUID tenantId);

    /**
     * 根据用户ID和权限代码检查用户是否拥有权限
     */
    @Query("SELECT COUNT(rp) > 0 FROM RolePermission rp " +
           "JOIN UserRole ur ON rp.roleId = ur.roleId " +
           "JOIN Permission p ON rp.permissionId = p.id " +
           "WHERE ur.userId = :userId AND ur.tenantId = :tenantId " +
           "AND p.code = :permissionCode " +
           "AND ur.isActive = true AND rp.isActive = true " +
           "AND (ur.expiresAt IS NULL OR ur.expiresAt > CURRENT_TIMESTAMP)")
    boolean hasUserPermission(@Param("userId") UUID userId, 
                             @Param("tenantId") UUID tenantId, 
                             @Param("permissionCode") String permissionCode);

    /**
     * 根据用户ID查询用户的权限代码列表
     */
    @Query("SELECT DISTINCT p.code FROM RolePermission rp " +
           "JOIN UserRole ur ON rp.roleId = ur.roleId " +
           "JOIN Permission p ON rp.permissionId = p.id " +
           "WHERE ur.userId = :userId AND ur.tenantId = :tenantId " +
           "AND ur.isActive = true AND rp.isActive = true " +
           "AND (ur.expiresAt IS NULL OR ur.expiresAt > CURRENT_TIMESTAMP)")
    Set<String> findUserPermissionCodes(@Param("userId") UUID userId, @Param("tenantId") UUID tenantId);

    /**
     * 检查角色是否拥有指定权限
     */
    boolean existsByRoleIdAndPermissionIdAndTenantIdAndIsActiveTrue(UUID roleId, UUID permissionId, UUID tenantId);

    /**
     * 统计角色的权限数量
     */
    long countByRoleIdAndTenantIdAndIsActiveTrue(UUID roleId, UUID tenantId);

    /**
     * 统计权限被分配的角色数量
     */
    long countByPermissionIdAndIsActiveTrue(UUID permissionId);

    /**
     * 根据授权者查找角色权限
     */
    List<RolePermission> findByGrantedBy(UUID grantedBy);

    /**
     * 检查角色是否拥有指定权限代码
     */
    @Query("SELECT COUNT(rp) > 0 FROM RolePermission rp " +
           "JOIN Permission p ON rp.permissionId = p.id " +
           "WHERE rp.roleId = :roleId AND p.code = :permissionCode AND rp.isActive = true")
    boolean existsByRoleIdAndPermissionCode(@Param("roleId") UUID roleId, @Param("permissionCode") String permissionCode);

    /**
     * 检查角色是否拥有任意权限代码
     */
    @Query("SELECT COUNT(rp) > 0 FROM RolePermission rp " +
           "JOIN Permission p ON rp.permissionId = p.id " +
           "WHERE rp.roleId = :roleId AND p.code IN :permissionCodes AND rp.isActive = true")
    boolean existsByRoleIdAndPermissionCodeIn(@Param("roleId") UUID roleId, @Param("permissionCodes") Set<String> permissionCodes);

    /**
     * 统计角色拥有的指定权限代码数量
     */
    @Query("SELECT COUNT(rp) FROM RolePermission rp " +
           "JOIN Permission p ON rp.permissionId = p.id " +
           "WHERE rp.roleId = :roleId AND p.code IN :permissionCodes AND rp.isActive = true")
    long countByRoleIdAndPermissionCodeIn(@Param("roleId") UUID roleId, @Param("permissionCodes") Set<String> permissionCodes);

    /**
     * 统计角色的活跃权限数量
     */
    long countByRoleIdAndIsActiveTrue(UUID roleId);

    /**
     * 查找有效的角色权限（活跃且未过期）
     */
    @Query("SELECT rp FROM RolePermission rp WHERE rp.roleId = :roleId " +
           "AND rp.isActive = true AND (rp.expiresAt IS NULL OR rp.expiresAt > CURRENT_TIMESTAMP)")
    List<RolePermission> findValidByRoleId(@Param("roleId") UUID roleId);

    /**
     * 获取角色的有效权限代码集合
     */
    @Query("SELECT p.code FROM RolePermission rp " +
           "JOIN Permission p ON rp.permissionId = p.id " +
           "WHERE rp.roleId = :roleId AND rp.isActive = true " +
           "AND (rp.expiresAt IS NULL OR rp.expiresAt > CURRENT_TIMESTAMP)")
    Set<String> findValidPermissionCodesByRoleId(@Param("roleId") UUID roleId);

    /**
     * 查找过期的权限
     */
    @Query("SELECT rp FROM RolePermission rp WHERE rp.isActive = true " +
           "AND rp.expiresAt IS NOT NULL AND rp.expiresAt <= CURRENT_TIMESTAMP")
    List<RolePermission> findExpiredPermissions();

    /**
     * 根据角色和权限ID查找历史记录（按授权时间倒序）
     */
    List<RolePermission> findByRoleIdAndPermissionIdOrderByGrantedAtDesc(UUID roleId, UUID permissionId);

    /**
     * 统计角色的总权限数量
     */
    long countByRoleId(UUID roleId);

    /**
     * 统计角色的过期权限数量
     */
    @Query("SELECT COUNT(rp) FROM RolePermission rp WHERE rp.roleId = :roleId " +
           "AND rp.expiresAt IS NOT NULL AND rp.expiresAt <= CURRENT_TIMESTAMP")
    long countExpiredByRoleId(@Param("roleId") UUID roleId);

    /**
     * 统计角色即将过期的权限数量
     */
    @Query("SELECT COUNT(rp) FROM RolePermission rp WHERE rp.roleId = :roleId " +
           "AND rp.isActive = true AND rp.expiresAt IS NOT NULL " +
           "AND rp.expiresAt > CURRENT_TIMESTAMP AND rp.expiresAt <= :threshold")
    long countExpiringByRoleId(@Param("roleId") UUID roleId, @Param("threshold") LocalDateTime threshold);

    /**
     * 分页查询角色权限
     */
    Page<RolePermission> findByRoleId(UUID roleId, Pageable pageable);

    /**
     * 检查角色是否拥有指定权限
     */
    boolean existsByRoleIdAndPermissionId(UUID roleId, UUID permissionId);

    /**
     * 根据角色ID和权限ID查找角色权限
     */
    Optional<RolePermission> findByRoleIdAndPermissionId(UUID roleId, UUID permissionId);

    /**
     * 删除角色权限关联
     */
    void deleteByRoleIdAndPermissionId(UUID roleId, UUID permissionId);

    /**
     * 根据角色ID查找激活的角色权限
     */
    List<RolePermission> findByRoleIdAndIsActiveTrue(UUID roleId);
} 