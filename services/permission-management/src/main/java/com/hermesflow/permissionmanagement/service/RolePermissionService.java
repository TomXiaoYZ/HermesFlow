package com.hermesflow.permissionmanagement.service;

import com.hermesflow.permissionmanagement.dto.PermissionDTO;
import com.hermesflow.permissionmanagement.dto.RolePermissionDTO;
import org.springframework.data.domain.Page;
import org.springframework.data.domain.Pageable;

import java.time.LocalDateTime;
import java.util.List;
import java.util.Set;
import java.util.UUID;

/**
 * 角色权限关联服务接口
 * 
 * @author HermesFlow Team
 * @version 1.0.0
 * @since 2024-12-30
 */
public interface RolePermissionService {

    /**
     * 为角色分配权限
     */
    void assignPermissionToRole(UUID roleId, UUID permissionId, UUID grantedBy);

    /**
     * 为角色分配权限（带过期时间）
     */
    void assignPermissionToRole(UUID roleId, UUID permissionId, UUID grantedBy, LocalDateTime expiresAt);

    /**
     * 批量为角色分配权限
     */
    void batchAssignPermissionsToRole(UUID roleId, Set<UUID> permissionIds, UUID grantedBy);

    /**
     * 批量为角色分配权限（带过期时间）
     */
    void batchAssignPermissionsToRole(UUID roleId, Set<UUID> permissionIds, UUID grantedBy, LocalDateTime expiresAt);

    /**
     * 从角色移除权限
     */
    void removePermissionFromRole(UUID roleId, UUID permissionId);

    /**
     * 批量从角色移除权限
     */
    void batchRemovePermissionsFromRole(UUID roleId, Set<UUID> permissionIds);

    /**
     * 更新角色权限过期时间
     */
    void updateRolePermissionExpiration(UUID roleId, UUID permissionId, LocalDateTime expiresAt);

    /**
     * 激活角色权限
     */
    void activateRolePermission(UUID roleId, UUID permissionId);

    /**
     * 停用角色权限
     */
    void deactivateRolePermission(UUID roleId, UUID permissionId);

    /**
     * 获取角色的所有权限
     */
    List<PermissionDTO> getRolePermissions(UUID roleId);

    /**
     * 获取角色的所有权限关联
     */
    List<RolePermissionDTO> getAllRolePermissions(UUID roleId);

    /**
     * 获取角色的有效权限
     */
    List<PermissionDTO> getValidRolePermissions(UUID roleId);

    /**
     * 分页获取角色权限
     */
    Page<RolePermissionDTO> getRolePermissions(UUID roleId, Pageable pageable);

    /**
     * 获取拥有指定权限的角色列表
     */
    List<UUID> getRolesByPermission(UUID permissionId);

    /**
     * 检查角色是否拥有指定权限
     */
    boolean roleHasPermission(UUID roleId, UUID permissionId);

    /**
     * 检查角色是否拥有指定权限代码
     */
    boolean roleHasPermissionCode(UUID roleId, String permissionCode);

    /**
     * 检查角色是否拥有指定权限
     */
    boolean hasPermission(UUID roleId, String permissionCode);

    /**
     * 检查角色是否拥有任意权限
     */
    boolean roleHasAnyPermission(UUID roleId, Set<UUID> permissionIds);

    /**
     * 检查角色是否拥有任意权限
     */
    boolean hasAnyPermission(UUID roleId, Set<String> permissionCodes);

    /**
     * 检查角色是否拥有所有权限
     */
    boolean roleHasAllPermissions(UUID roleId, Set<UUID> permissionIds);

    /**
     * 验证角色权限
     */
    boolean validateRolePermissions(UUID roleId, Set<String> permissionCodes);

    /**
     * 复制角色权限
     */
    void copyRolePermissions(UUID sourceRoleId, UUID targetRoleId, UUID grantedBy);

    /**
     * 同步角色权限（替换所有权限）
     */
    void syncRolePermissions(UUID roleId, Set<UUID> permissionIds, UUID grantedBy);

    /**
     * 获取即将过期的角色权限
     */
    List<RolePermissionDTO> getExpiringRolePermissions(UUID roleId, int days);

    /**
     * 获取已过期的角色权限
     */
    List<RolePermissionDTO> getExpiredRolePermissions(UUID roleId);

    /**
     * 清理过期的角色权限
     */
    int cleanupExpiredRolePermissions(UUID roleId);

    /**
     * 清理过期的权限
     */
    int cleanupExpiredPermissions();

    /**
     * 获取角色权限历史记录
     */
    Page<RolePermissionHistoryDTO> getRolePermissionHistory(UUID roleId, Pageable pageable);

    /**
     * 获取权限分配历史记录
     */
    Page<RolePermissionHistoryDTO> getPermissionAssignmentHistory(UUID permissionId, Pageable pageable);

    /**
     * 验证角色权限分配
     */
    RolePermissionValidationResult validateRolePermissionAssignment(UUID roleId, UUID permissionId);

    /**
     * 获取角色权限统计信息
     */
    RolePermissionStatistics getRolePermissionStatistics(UUID roleId);

    /**
     * 角色权限历史记录DTO
     */
    class RolePermissionHistoryDTO {
        private UUID id;
        private UUID roleId;
        private String roleName;
        private UUID permissionId;
        private String permissionCode;
        private String permissionName;
        private String action; // ASSIGNED, REMOVED, EXPIRED
        private UUID operatedBy;
        private String operatorName;
        private LocalDateTime operatedAt;
        private String reason;

        // 构造函数
        public RolePermissionHistoryDTO() {}

        public RolePermissionHistoryDTO(UUID id, UUID roleId, String roleName, UUID permissionId,
                                       String permissionCode, String permissionName, String action,
                                       UUID operatedBy, String operatorName, LocalDateTime operatedAt, String reason) {
            this.id = id;
            this.roleId = roleId;
            this.roleName = roleName;
            this.permissionId = permissionId;
            this.permissionCode = permissionCode;
            this.permissionName = permissionName;
            this.action = action;
            this.operatedBy = operatedBy;
            this.operatorName = operatorName;
            this.operatedAt = operatedAt;
            this.reason = reason;
        }

        // Getters and Setters
        public UUID getId() { return id; }
        public void setId(UUID id) { this.id = id; }
        public UUID getRoleId() { return roleId; }
        public void setRoleId(UUID roleId) { this.roleId = roleId; }
        public String getRoleName() { return roleName; }
        public void setRoleName(String roleName) { this.roleName = roleName; }
        public UUID getPermissionId() { return permissionId; }
        public void setPermissionId(UUID permissionId) { this.permissionId = permissionId; }
        public String getPermissionCode() { return permissionCode; }
        public void setPermissionCode(String permissionCode) { this.permissionCode = permissionCode; }
        public String getPermissionName() { return permissionName; }
        public void setPermissionName(String permissionName) { this.permissionName = permissionName; }
        public String getAction() { return action; }
        public void setAction(String action) { this.action = action; }
        public UUID getOperatedBy() { return operatedBy; }
        public void setOperatedBy(UUID operatedBy) { this.operatedBy = operatedBy; }
        public String getOperatorName() { return operatorName; }
        public void setOperatorName(String operatorName) { this.operatorName = operatorName; }
        public LocalDateTime getOperatedAt() { return operatedAt; }
        public void setOperatedAt(LocalDateTime operatedAt) { this.operatedAt = operatedAt; }
        public String getReason() { return reason; }
        public void setReason(String reason) { this.reason = reason; }
    }

    /**
     * 角色权限验证结果
     */
    class RolePermissionValidationResult {
        private final boolean valid;
        private final String message;
        private final List<String> violations;

        public RolePermissionValidationResult(boolean valid, String message, List<String> violations) {
            this.valid = valid;
            this.message = message;
            this.violations = violations;
        }

        public boolean isValid() { return valid; }
        public String getMessage() { return message; }
        public List<String> getViolations() { return violations; }
    }

    /**
     * 角色权限统计信息
     */
    class RolePermissionStatistics {
        private final long totalPermissions;
        private final long activePermissions;
        private final long expiredPermissions;
        private final long expiringPermissions;
        private final long systemPermissions;
        private final long businessPermissions;

        public RolePermissionStatistics(long totalPermissions, long activePermissions, long expiredPermissions,
                                       long expiringPermissions, long systemPermissions, long businessPermissions) {
            this.totalPermissions = totalPermissions;
            this.activePermissions = activePermissions;
            this.expiredPermissions = expiredPermissions;
            this.expiringPermissions = expiringPermissions;
            this.systemPermissions = systemPermissions;
            this.businessPermissions = businessPermissions;
        }

        public long getTotalPermissions() { return totalPermissions; }
        public long getActivePermissions() { return activePermissions; }
        public long getExpiredPermissions() { return expiredPermissions; }
        public long getExpiringPermissions() { return expiringPermissions; }
        public long getSystemPermissions() { return systemPermissions; }
        public long getBusinessPermissions() { return businessPermissions; }
    }
} 