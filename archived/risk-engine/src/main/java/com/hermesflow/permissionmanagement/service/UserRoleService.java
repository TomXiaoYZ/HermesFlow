package com.hermesflow.permissionmanagement.service;

import com.hermesflow.permissionmanagement.dto.UserRoleAssignDTO;
import com.hermesflow.permissionmanagement.dto.UserRoleDTO;
import org.springframework.data.domain.Page;
import org.springframework.data.domain.Pageable;

import java.time.LocalDateTime;
import java.util.List;
import java.util.Set;
import java.util.UUID;

/**
 * 用户角色管理服务接口
 * 
 * @author HermesFlow Team
 * @version 1.0.0
 * @since 2024-12-30
 */
public interface UserRoleService {

    /**
     * 为用户分配角色
     * 
     * @param assignDTO 用户角色分配DTO
     * @return 用户角色DTO
     */
    UserRoleDTO assignRole(UserRoleAssignDTO assignDTO);

    /**
     * 批量为用户分配角色
     * 
     * @param userId 用户ID
     * @param tenantId 租户ID
     * @param roleIds 角色ID集合
     * @param assignedBy 分配者ID
     * @param expiresAt 过期时间
     * @return 用户角色DTO列表
     */
    List<UserRoleDTO> batchAssignRoles(UUID userId, UUID tenantId, Set<UUID> roleIds, 
                                      UUID assignedBy, LocalDateTime expiresAt);

    /**
     * 撤销用户角色
     * 
     * @param userRoleId 用户角色ID
     */
    void revokeRole(UUID userRoleId);

    /**
     * 批量撤销用户角色
     * 
     * @param userId 用户ID
     * @param tenantId 租户ID
     * @param roleIds 角色ID集合
     */
    void batchRevokeRoles(UUID userId, UUID tenantId, Set<UUID> roleIds);

    /**
     * 更新用户角色过期时间
     * 
     * @param userRoleId 用户角色ID
     * @param expiresAt 新的过期时间
     * @return 用户角色DTO
     */
    UserRoleDTO updateRoleExpiration(UUID userRoleId, LocalDateTime expiresAt);

    /**
     * 激活用户角色
     * 
     * @param userRoleId 用户角色ID
     * @return 用户角色DTO
     */
    UserRoleDTO activateRole(UUID userRoleId);

    /**
     * 停用用户角色
     * 
     * @param userRoleId 用户角色ID
     * @return 用户角色DTO
     */
    UserRoleDTO deactivateRole(UUID userRoleId);

    /**
     * 获取用户在指定租户下的所有角色
     * 
     * @param userId 用户ID
     * @param tenantId 租户ID
     * @return 用户角色DTO列表
     */
    List<UserRoleDTO> getUserRoles(UUID userId, UUID tenantId);

    /**
     * 获取用户在指定租户下的有效角色
     * 
     * @param userId 用户ID
     * @param tenantId 租户ID
     * @return 用户角色DTO列表
     */
    List<UserRoleDTO> getValidUserRoles(UUID userId, UUID tenantId);

    /**
     * 获取用户在指定租户下的激活角色
     * 
     * @param userId 用户ID
     * @param tenantId 租户ID
     * @return 用户角色DTO列表
     */
    List<UserRoleDTO> getActiveUserRoles(UUID userId, UUID tenantId);

    /**
     * 获取指定角色的所有用户
     * 
     * @param roleId 角色ID
     * @param tenantId 租户ID
     * @param pageable 分页参数
     * @return 用户角色DTO分页结果
     */
    Page<UserRoleDTO> getUsersByRole(UUID roleId, UUID tenantId, Pageable pageable);

    /**
     * 检查用户是否拥有指定角色
     * 
     * @param userId 用户ID
     * @param roleId 角色ID
     * @param tenantId 租户ID
     * @return 是否拥有角色
     */
    boolean hasRole(UUID userId, UUID roleId, UUID tenantId);

    /**
     * 检查用户是否拥有有效的指定角色
     * 
     * @param userId 用户ID
     * @param roleId 角色ID
     * @param tenantId 租户ID
     * @return 是否拥有有效角色
     */
    boolean hasValidRole(UUID userId, UUID roleId, UUID tenantId);

    /**
     * 获取用户的所有权限代码
     * 
     * @param userId 用户ID
     * @param tenantId 租户ID
     * @return 权限代码集合
     */
    Set<String> getUserPermissions(UUID userId, UUID tenantId);

    /**
     * 检查用户是否拥有指定权限
     * 
     * @param userId 用户ID
     * @param tenantId 租户ID
     * @param permissionCode 权限代码
     * @return 是否拥有权限
     */
    boolean hasPermission(UUID userId, UUID tenantId, String permissionCode);

    /**
     * 检查用户是否拥有任一权限
     * 
     * @param userId 用户ID
     * @param tenantId 租户ID
     * @param permissionCodes 权限代码集合
     * @return 是否拥有任一权限
     */
    boolean hasAnyPermission(UUID userId, UUID tenantId, Set<String> permissionCodes);

    /**
     * 检查用户是否拥有所有权限
     * 
     * @param userId 用户ID
     * @param tenantId 租户ID
     * @param permissionCodes 权限代码集合
     * @return 是否拥有所有权限
     */
    boolean hasAllPermissions(UUID userId, UUID tenantId, Set<String> permissionCodes);

    /**
     * 获取即将过期的用户角色
     * 
     * @param tenantId 租户ID
     * @param days 天数
     * @return 用户角色DTO列表
     */
    List<UserRoleDTO> getExpiringRoles(UUID tenantId, int days);

    /**
     * 获取已过期的用户角色
     * 
     * @param tenantId 租户ID
     * @return 用户角色DTO列表
     */
    List<UserRoleDTO> getExpiredRoles(UUID tenantId);

    /**
     * 清理过期的用户角色
     * 
     * @param tenantId 租户ID
     * @return 清理的角色数量
     */
    int cleanupExpiredRoles(UUID tenantId);

    /**
     * 复制用户角色到另一个用户
     * 
     * @param sourceUserId 源用户ID
     * @param targetUserId 目标用户ID
     * @param tenantId 租户ID
     * @param assignedBy 分配者ID
     * @return 复制的角色数量
     */
    int copyUserRoles(UUID sourceUserId, UUID targetUserId, UUID tenantId, UUID assignedBy);

    /**
     * 获取租户下的用户角色统计
     * 
     * @param tenantId 租户ID
     * @return 用户角色统计信息
     */
    UserRoleStatistics getUserRoleStatistics(UUID tenantId);

    /**
     * 获取用户角色变更历史
     * 
     * @param userId 用户ID
     * @param tenantId 租户ID
     * @param pageable 分页参数
     * @return 用户角色DTO分页结果
     */
    Page<UserRoleDTO> getUserRoleHistory(UUID userId, UUID tenantId, Pageable pageable);

    /**
     * 验证角色分配的有效性
     * 
     * @param userId 用户ID
     * @param roleId 角色ID
     * @param tenantId 租户ID
     * @return 验证结果
     */
    RoleAssignmentValidation validateRoleAssignment(UUID userId, UUID roleId, UUID tenantId);

    /**
     * 激活用户角色
     * 
     * @param userRoleId 用户角色ID
     * @return 用户角色DTO
     */
    UserRoleDTO activateUserRole(UUID userRoleId);

    /**
     * 停用用户角色
     * 
     * @param userRoleId 用户角色ID
     * @return 用户角色DTO
     */
    UserRoleDTO deactivateUserRole(UUID userRoleId);

    /**
     * 分页获取用户角色
     * 
     * @param userId 用户ID
     * @param tenantId 租户ID
     * @param pageable 分页参数
     * @return 用户角色DTO分页结果
     */
    Page<UserRoleDTO> getUserRoles(UUID userId, UUID tenantId, Pageable pageable);

    /**
     * 检查用户权限
     * 
     * @param userId 用户ID
     * @param tenantId 租户ID
     * @param permissionCode 权限代码
     * @return 是否拥有权限
     */
    boolean checkUserPermission(UUID userId, UUID tenantId, String permissionCode);

    /**
     * 检查用户角色
     * 
     * @param userId 用户ID
     * @param tenantId 租户ID
     * @param roleCode 角色代码
     * @return 是否拥有角色
     */
    boolean checkUserRole(UUID userId, UUID tenantId, String roleCode);

    /**
     * 获取角色的用户列表
     * 
     * @param roleId 角色ID
     * @param pageable 分页参数
     * @return 用户角色DTO分页结果
     */
    Page<UserRoleDTO> getRoleUsers(UUID roleId, Pageable pageable);

    /**
     * 用户角色统计信息内部类
     */
    class UserRoleStatistics {
        private long totalAssignments;
        private long activeAssignments;
        private long expiredAssignments;
        private long expiringAssignments;
        private long usersWithRoles;

        // 构造函数
        public UserRoleStatistics(long totalAssignments, long activeAssignments, 
                                long expiredAssignments, long expiringAssignments, 
                                long usersWithRoles) {
            this.totalAssignments = totalAssignments;
            this.activeAssignments = activeAssignments;
            this.expiredAssignments = expiredAssignments;
            this.expiringAssignments = expiringAssignments;
            this.usersWithRoles = usersWithRoles;
        }

        // Getters
        public long getTotalAssignments() { return totalAssignments; }
        public long getActiveAssignments() { return activeAssignments; }
        public long getExpiredAssignments() { return expiredAssignments; }
        public long getExpiringAssignments() { return expiringAssignments; }
        public long getUsersWithRoles() { return usersWithRoles; }
    }

    /**
     * 角色分配验证结果内部类
     */
    class RoleAssignmentValidation {
        private boolean valid;
        private String reason;
        private List<String> warnings;

        // 构造函数
        public RoleAssignmentValidation(boolean valid, String reason, List<String> warnings) {
            this.valid = valid;
            this.reason = reason;
            this.warnings = warnings;
        }

        // Getters
        public boolean isValid() { return valid; }
        public String getReason() { return reason; }
        public List<String> getWarnings() { return warnings; }
    }
} 