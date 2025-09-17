package com.hermesflow.permissionmanagement.repository;

import com.hermesflow.permissionmanagement.entity.UserRole;
import org.springframework.data.domain.Page;
import org.springframework.data.domain.Pageable;
import org.springframework.data.jpa.repository.JpaRepository;
import org.springframework.data.jpa.repository.Modifying;
import org.springframework.data.jpa.repository.Query;
import org.springframework.data.repository.query.Param;
import org.springframework.stereotype.Repository;

import java.time.LocalDateTime;
import java.util.List;
import java.util.Optional;
import java.util.Set;
import java.util.UUID;

/**
 * 用户角色关联Repository接口
 * 
 * @author HermesFlow Team
 * @version 1.0.0
 * @since 2024-12-30
 */
@Repository
public interface UserRoleRepository extends JpaRepository<UserRole, UUID> {

    /**
     * 根据用户ID和租户ID查找用户角色
     */
    List<UserRole> findByUserIdAndTenantId(UUID userId, UUID tenantId);

    /**
     * 根据用户ID和租户ID查找激活的用户角色
     */
    List<UserRole> findByUserIdAndTenantIdAndIsActiveTrue(UUID userId, UUID tenantId);

    /**
     * 根据用户ID、租户ID查找有效的用户角色 (激活且未过期)
     */
    @Query("SELECT ur FROM UserRole ur WHERE ur.userId = :userId AND ur.tenantId = :tenantId " +
           "AND ur.isActive = true AND (ur.expiresAt IS NULL OR ur.expiresAt > :now)")
    List<UserRole> findValidUserRoles(@Param("userId") UUID userId, 
                                     @Param("tenantId") UUID tenantId, 
                                     @Param("now") LocalDateTime now);

    /**
     * 根据用户ID、角色ID和租户ID查找用户角色
     */
    Optional<UserRole> findByUserIdAndRoleIdAndTenantId(UUID userId, UUID roleId, UUID tenantId);

    /**
     * 根据角色ID查找所有用户角色
     */
    List<UserRole> findByRoleId(UUID roleId);

    /**
     * 根据角色ID查找激活的用户角色
     */
    List<UserRole> findByRoleIdAndIsActiveTrue(UUID roleId);

    /**
     * 根据租户ID查找所有用户角色
     */
    List<UserRole> findByTenantId(UUID tenantId);

    /**
     * 根据租户ID分页查询用户角色
     */
    Page<UserRole> findByTenantId(UUID tenantId, Pageable pageable);

    /**
     * 根据用户ID和租户ID分页查询用户角色
     */
    Page<UserRole> findByUserIdAndTenantId(UUID userId, UUID tenantId, Pageable pageable);

    /**
     * 查找即将过期的用户角色 (指定天数内)
     */
    @Query("SELECT ur FROM UserRole ur WHERE ur.tenantId = :tenantId " +
           "AND ur.isActive = true AND ur.expiresAt IS NOT NULL " +
           "AND ur.expiresAt BETWEEN :now AND :expiresBefore")
    List<UserRole> findExpiringSoon(@Param("tenantId") UUID tenantId,
                                   @Param("now") LocalDateTime now,
                                   @Param("expiresBefore") LocalDateTime expiresBefore);

    /**
     * 查找已过期的用户角色
     */
    @Query("SELECT ur FROM UserRole ur WHERE ur.tenantId = :tenantId " +
           "AND ur.isActive = true AND ur.expiresAt IS NOT NULL " +
           "AND ur.expiresAt < :now")
    List<UserRole> findExpiredUserRoles(@Param("tenantId") UUID tenantId,
                                       @Param("now") LocalDateTime now);

    /**
     * 检查用户是否拥有指定角色
     */
    boolean existsByUserIdAndRoleIdAndTenantIdAndIsActiveTrue(UUID userId, UUID roleId, UUID tenantId);

    /**
     * 检查用户是否拥有有效的指定角色
     */
    @Query("SELECT COUNT(ur) > 0 FROM UserRole ur WHERE ur.userId = :userId " +
           "AND ur.roleId = :roleId AND ur.tenantId = :tenantId " +
           "AND ur.isActive = true AND (ur.expiresAt IS NULL OR ur.expiresAt > :now)")
    boolean hasValidRole(@Param("userId") UUID userId, 
                        @Param("roleId") UUID roleId, 
                        @Param("tenantId") UUID tenantId,
                        @Param("now") LocalDateTime now);

    /**
     * 批量停用过期的用户角色
     */
    @Modifying
    @Query("UPDATE UserRole ur SET ur.isActive = false WHERE ur.tenantId = :tenantId " +
           "AND ur.isActive = true AND ur.expiresAt IS NOT NULL AND ur.expiresAt < :now")
    int deactivateExpiredUserRoles(@Param("tenantId") UUID tenantId, @Param("now") LocalDateTime now);

    /**
     * 统计用户在租户内的角色数量
     */
    long countByUserIdAndTenantIdAndIsActiveTrue(UUID userId, UUID tenantId);

    /**
     * 统计角色的用户数量
     */
    long countByRoleIdAndIsActiveTrue(UUID roleId);

    /**
     * 根据分配者查找用户角色
     */
    List<UserRole> findByAssignedBy(UUID assignedBy);

    /**
     * 检查用户是否拥有指定角色代码
     */
    @Query("SELECT COUNT(ur) > 0 FROM UserRole ur " +
           "JOIN Role r ON ur.roleId = r.id " +
           "WHERE ur.userId = :userId AND r.code = :roleCode AND ur.tenantId = :tenantId " +
           "AND ur.isActive = true AND (ur.expiresAt IS NULL OR ur.expiresAt > CURRENT_TIMESTAMP)")
    boolean existsByUserIdAndRoleCodeAndTenantId(@Param("userId") UUID userId, 
                                                @Param("roleCode") String roleCode, 
                                                @Param("tenantId") UUID tenantId);

    /**
     * 统计用户在租户内的总角色数量
     */
    long countByUserIdAndTenantId(UUID userId, UUID tenantId);

    /**
     * 统计过期的用户角色数量
     */
    @Query("SELECT COUNT(ur) FROM UserRole ur WHERE ur.userId = :userId AND ur.tenantId = :tenantId " +
           "AND ur.expiresAt IS NOT NULL AND ur.expiresAt <= :now")
    long countExpiredUserRoles(@Param("userId") UUID userId, @Param("tenantId") UUID tenantId, @Param("now") LocalDateTime now);

    /**
     * 统计即将过期的用户角色数量
     */
    @Query("SELECT COUNT(ur) FROM UserRole ur WHERE ur.userId = :userId AND ur.tenantId = :tenantId " +
           "AND ur.expiresAt IS NOT NULL AND ur.expiresAt > CURRENT_TIMESTAMP AND ur.expiresAt <= :threshold")
    long countExpiringUserRoles(@Param("userId") UUID userId, @Param("tenantId") UUID tenantId, @Param("threshold") LocalDateTime threshold);

    /**
     * 查找即将过期的用户角色
     */
    @Query("SELECT ur FROM UserRole ur WHERE ur.userId = :userId AND ur.tenantId = :tenantId " +
           "AND ur.isActive = true AND ur.expiresAt IS NOT NULL " +
           "AND ur.expiresAt > :now AND ur.expiresAt <= :threshold")
    List<UserRole> findExpiringUserRoles(@Param("userId") UUID userId, 
                                        @Param("tenantId") UUID tenantId, 
                                        @Param("threshold") LocalDateTime threshold);

    /**
     * 统计租户内的总角色分配数量
     */
    long countByTenantId(UUID tenantId);

    /**
     * 统计租户内激活的角色分配数量
     */
    long countByTenantIdAndIsActiveTrue(UUID tenantId);

    /**
     * 统计租户内过期的用户角色数量
     */
    @Query("SELECT COUNT(ur) FROM UserRole ur WHERE ur.tenantId = :tenantId " +
           "AND ur.expiresAt IS NOT NULL AND ur.expiresAt <= :now")
    long countExpiredUserRoles(@Param("tenantId") UUID tenantId, @Param("now") LocalDateTime now);

    /**
     * 统计租户内即将过期的用户角色数量
     */
    @Query("SELECT COUNT(ur) FROM UserRole ur WHERE ur.tenantId = :tenantId " +
           "AND ur.expiresAt IS NOT NULL AND ur.expiresAt > CURRENT_TIMESTAMP AND ur.expiresAt <= :threshold")
    long countExpiringUserRoles(@Param("tenantId") UUID tenantId, @Param("threshold") LocalDateTime threshold);

    /**
     * 统计租户内拥有角色的不同用户数量
     */
    @Query("SELECT COUNT(DISTINCT ur.userId) FROM UserRole ur WHERE ur.tenantId = :tenantId AND ur.isActive = true")
    long countDistinctUsersByTenantId(@Param("tenantId") UUID tenantId);

    /**
     * 根据角色ID分页查询用户角色
     */
    Page<UserRole> findByRoleId(UUID roleId, Pageable pageable);
} 