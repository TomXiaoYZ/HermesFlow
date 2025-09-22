package com.hermesflow.permissionmanagement.service;

import java.util.List;
import java.util.Set;
import java.util.UUID;

/**
 * 权限验证服务接口
 * 
 * @author HermesFlow Team
 * @version 1.0.0
 * @since 2024-12-30
 */
public interface PermissionValidationService {

    /**
     * 验证用户是否拥有指定权限
     * 
     * @param userId 用户ID
     * @param tenantId 租户ID
     * @param permissionCode 权限代码
     * @return 验证结果
     */
    PermissionValidationResult validatePermission(UUID userId, UUID tenantId, String permissionCode);

    /**
     * 验证用户是否拥有任一权限
     * 
     * @param userId 用户ID
     * @param tenantId 租户ID
     * @param permissionCodes 权限代码集合
     * @return 验证结果
     */
    PermissionValidationResult validateAnyPermission(UUID userId, UUID tenantId, Set<String> permissionCodes);

    /**
     * 验证用户是否拥有所有权限
     * 
     * @param userId 用户ID
     * @param tenantId 租户ID
     * @param permissionCodes 权限代码集合
     * @return 验证结果
     */
    PermissionValidationResult validateAllPermissions(UUID userId, UUID tenantId, Set<String> permissionCodes);

    /**
     * 验证用户对资源的操作权限
     * 
     * @param userId 用户ID
     * @param tenantId 租户ID
     * @param resource 资源名称
     * @param action 操作名称
     * @return 验证结果
     */
    PermissionValidationResult validateResourceAction(UUID userId, UUID tenantId, String resource, String action);

    /**
     * 验证用户对资源的多个操作权限
     * 
     * @param userId 用户ID
     * @param tenantId 租户ID
     * @param resource 资源名称
     * @param actions 操作名称集合
     * @return 验证结果
     */
    PermissionValidationResult validateResourceActions(UUID userId, UUID tenantId, String resource, Set<String> actions);

    /**
     * 批量验证用户权限
     * 
     * @param userId 用户ID
     * @param tenantId 租户ID
     * @param permissionCodes 权限代码集合
     * @return 权限验证结果映射
     */
    BatchPermissionValidationResult batchValidatePermissions(UUID userId, UUID tenantId, Set<String> permissionCodes);

    /**
     * 验证角色是否拥有指定权限
     * 
     * @param roleId 角色ID
     * @param permissionCode 权限代码
     * @return 验证结果
     */
    PermissionValidationResult validateRolePermission(UUID roleId, String permissionCode);

    /**
     * 获取用户的有效权限列表
     * 
     * @param userId 用户ID
     * @param tenantId 租户ID
     * @return 权限代码集合
     */
    Set<String> getUserEffectivePermissions(UUID userId, UUID tenantId);

    /**
     * 获取用户对指定资源的有效权限
     * 
     * @param userId 用户ID
     * @param tenantId 租户ID
     * @param resource 资源名称
     * @return 权限代码集合
     */
    Set<String> getUserResourcePermissions(UUID userId, UUID tenantId, String resource);

    /**
     * 检查权限是否即将过期
     * 
     * @param userId 用户ID
     * @param tenantId 租户ID
     * @param permissionCode 权限代码
     * @param days 天数
     * @return 是否即将过期
     */
    boolean isPermissionExpiring(UUID userId, UUID tenantId, String permissionCode, int days);

    /**
     * 获取用户权限的过期信息
     * 
     * @param userId 用户ID
     * @param tenantId 租户ID
     * @return 权限过期信息列表
     */
    List<PermissionExpirationInfo> getUserPermissionExpirations(UUID userId, UUID tenantId);

    /**
     * 验证权限层级关系
     * 
     * @param parentPermissionCode 父权限代码
     * @param childPermissionCode 子权限代码
     * @return 是否有效
     */
    boolean validatePermissionHierarchy(String parentPermissionCode, String childPermissionCode);

    /**
     * 获取权限的依赖关系
     * 
     * @param permissionCode 权限代码
     * @return 依赖的权限代码集合
     */
    Set<String> getPermissionDependencies(String permissionCode);

    /**
     * 权限验证结果内部类
     */
    class PermissionValidationResult {
        private boolean granted;
        private String reason;
        private List<String> missingPermissions;
        private List<String> expiredPermissions;
        private long validationTime;

        // 构造函数
        public PermissionValidationResult(boolean granted, String reason, 
                                        List<String> missingPermissions, 
                                        List<String> expiredPermissions) {
            this.granted = granted;
            this.reason = reason;
            this.missingPermissions = missingPermissions;
            this.expiredPermissions = expiredPermissions;
            this.validationTime = System.currentTimeMillis();
        }

        // Getters
        public boolean isGranted() { return granted; }
        public String getReason() { return reason; }
        public List<String> getMissingPermissions() { return missingPermissions; }
        public List<String> getExpiredPermissions() { return expiredPermissions; }
        public long getValidationTime() { return validationTime; }
    }

    /**
     * 批量权限验证结果内部类
     */
    class BatchPermissionValidationResult {
        private boolean allGranted;
        private int totalPermissions;
        private int grantedPermissions;
        private List<String> grantedPermissionCodes;
        private List<String> deniedPermissionCodes;
        private List<String> expiredPermissionCodes;

        // 构造函数
        public BatchPermissionValidationResult(boolean allGranted, int totalPermissions, 
                                             int grantedPermissions, List<String> grantedPermissionCodes,
                                             List<String> deniedPermissionCodes, List<String> expiredPermissionCodes) {
            this.allGranted = allGranted;
            this.totalPermissions = totalPermissions;
            this.grantedPermissions = grantedPermissions;
            this.grantedPermissionCodes = grantedPermissionCodes;
            this.deniedPermissionCodes = deniedPermissionCodes;
            this.expiredPermissionCodes = expiredPermissionCodes;
        }

        // Getters
        public boolean isAllGranted() { return allGranted; }
        public int getTotalPermissions() { return totalPermissions; }
        public int getGrantedPermissions() { return grantedPermissions; }
        public List<String> getGrantedPermissionCodes() { return grantedPermissionCodes; }
        public List<String> getDeniedPermissionCodes() { return deniedPermissionCodes; }
        public List<String> getExpiredPermissionCodes() { return expiredPermissionCodes; }
    }

    /**
     * 权限过期信息内部类
     */
    class PermissionExpirationInfo {
        private String permissionCode;
        private String permissionName;
        private String roleName;
        private java.time.LocalDateTime expiresAt;
        private long daysUntilExpiration;

        // 构造函数
        public PermissionExpirationInfo(String permissionCode, String permissionName, 
                                      String roleName, java.time.LocalDateTime expiresAt, 
                                      long daysUntilExpiration) {
            this.permissionCode = permissionCode;
            this.permissionName = permissionName;
            this.roleName = roleName;
            this.expiresAt = expiresAt;
            this.daysUntilExpiration = daysUntilExpiration;
        }

        // Getters
        public String getPermissionCode() { return permissionCode; }
        public String getPermissionName() { return permissionName; }
        public String getRoleName() { return roleName; }
        public java.time.LocalDateTime getExpiresAt() { return expiresAt; }
        public long getDaysUntilExpiration() { return daysUntilExpiration; }
    }
} 