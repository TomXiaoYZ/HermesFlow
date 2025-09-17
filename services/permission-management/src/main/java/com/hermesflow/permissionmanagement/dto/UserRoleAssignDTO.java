package com.hermesflow.permissionmanagement.dto;

import jakarta.validation.constraints.NotNull;

import java.time.LocalDateTime;
import java.util.UUID;

/**
 * 用户角色分配DTO
 * 
 * @author HermesFlow Team
 * @version 1.0.0
 * @since 2024-12-30
 */
public class UserRoleAssignDTO {

    @NotNull(message = "用户ID不能为空")
    private UUID userId;

    @NotNull(message = "角色ID不能为空")
    private UUID roleId;

    @NotNull(message = "租户ID不能为空")
    private UUID tenantId;

    private UUID assignedBy;

    private LocalDateTime expiresAt;

    private Boolean isActive = true;

    // 默认构造函数
    public UserRoleAssignDTO() {}

    // 构造函数
    public UserRoleAssignDTO(UUID userId, UUID roleId, UUID tenantId) {
        this.userId = userId;
        this.roleId = roleId;
        this.tenantId = tenantId;
    }

    public UserRoleAssignDTO(UUID userId, UUID roleId, UUID tenantId, UUID assignedBy) {
        this.userId = userId;
        this.roleId = roleId;
        this.tenantId = tenantId;
        this.assignedBy = assignedBy;
    }

    public UserRoleAssignDTO(UUID userId, UUID roleId, UUID tenantId, UUID assignedBy, LocalDateTime expiresAt) {
        this.userId = userId;
        this.roleId = roleId;
        this.tenantId = tenantId;
        this.assignedBy = assignedBy;
        this.expiresAt = expiresAt;
    }

    // Getters and Setters
    public UUID getUserId() {
        return userId;
    }

    public void setUserId(UUID userId) {
        this.userId = userId;
    }

    public UUID getRoleId() {
        return roleId;
    }

    public void setRoleId(UUID roleId) {
        this.roleId = roleId;
    }

    public UUID getTenantId() {
        return tenantId;
    }

    public void setTenantId(UUID tenantId) {
        this.tenantId = tenantId;
    }

    public UUID getAssignedBy() {
        return assignedBy;
    }

    public void setAssignedBy(UUID assignedBy) {
        this.assignedBy = assignedBy;
    }

    public LocalDateTime getExpiresAt() {
        return expiresAt;
    }

    public void setExpiresAt(LocalDateTime expiresAt) {
        this.expiresAt = expiresAt;
    }

    public Boolean getIsActive() {
        return isActive;
    }

    public void setIsActive(Boolean isActive) {
        this.isActive = isActive;
    }

    // 业务方法

    /**
     * 检查是否有过期时间
     */
    public boolean hasExpirationTime() {
        return this.expiresAt != null;
    }

    /**
     * 检查是否有分配者
     */
    public boolean hasAssignedBy() {
        return this.assignedBy != null;
    }

    /**
     * 检查过期时间是否有效 (未来时间)
     */
    public boolean isValidExpirationTime() {
        return this.expiresAt == null || this.expiresAt.isAfter(LocalDateTime.now());
    }

    @Override
    public String toString() {
        return "UserRoleAssignDTO{" +
                "userId=" + userId +
                ", roleId=" + roleId +
                ", tenantId=" + tenantId +
                ", assignedBy=" + assignedBy +
                ", expiresAt=" + expiresAt +
                ", isActive=" + isActive +
                '}';
    }
} 