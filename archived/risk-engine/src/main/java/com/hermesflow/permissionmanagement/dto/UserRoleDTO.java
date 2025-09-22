package com.hermesflow.permissionmanagement.dto;

import com.hermesflow.permissionmanagement.entity.UserRole;

import java.time.LocalDateTime;
import java.util.UUID;

/**
 * 用户角色DTO
 * 
 * @author HermesFlow Team
 * @version 1.0.0
 * @since 2024-12-30
 */
public class UserRoleDTO {

    private UUID id;
    private UUID userId;
    private UUID roleId;
    private UUID tenantId;
    private String roleName;
    private String roleCode;
    private UUID assignedBy;
    private String assignedByName;
    private LocalDateTime assignedAt;
    private LocalDateTime expiresAt;
    private Boolean isActive;
    private LocalDateTime createdAt;
    private LocalDateTime updatedAt;

    // 默认构造函数
    public UserRoleDTO() {}

    // 构造函数
    public UserRoleDTO(UUID id, UUID userId, UUID roleId, UUID tenantId) {
        this.id = id;
        this.userId = userId;
        this.roleId = roleId;
        this.tenantId = tenantId;
    }

    // 从实体转换的构造函数
    public UserRoleDTO(UserRole userRole) {
        this.id = userRole.getId();
        this.userId = userRole.getUserId();
        this.roleId = userRole.getRoleId();
        this.tenantId = userRole.getTenantId();
        this.assignedBy = userRole.getAssignedBy();
        this.assignedAt = userRole.getAssignedAt();
        this.expiresAt = userRole.getExpiresAt();
        this.isActive = userRole.getIsActive();
        this.createdAt = userRole.getCreatedAt();
        this.updatedAt = userRole.getUpdatedAt();
        
        // 设置角色信息
        if (userRole.getRole() != null) {
            this.roleName = userRole.getRole().getName();
            this.roleCode = userRole.getRole().getCode();
        }
    }

    // Getters and Setters
    public UUID getId() {
        return id;
    }

    public void setId(UUID id) {
        this.id = id;
    }

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

    public String getRoleName() {
        return roleName;
    }

    public void setRoleName(String roleName) {
        this.roleName = roleName;
    }

    public String getRoleCode() {
        return roleCode;
    }

    public void setRoleCode(String roleCode) {
        this.roleCode = roleCode;
    }

    public UUID getAssignedBy() {
        return assignedBy;
    }

    public void setAssignedBy(UUID assignedBy) {
        this.assignedBy = assignedBy;
    }

    public String getAssignedByName() {
        return assignedByName;
    }

    public void setAssignedByName(String assignedByName) {
        this.assignedByName = assignedByName;
    }

    public LocalDateTime getAssignedAt() {
        return assignedAt;
    }

    public void setAssignedAt(LocalDateTime assignedAt) {
        this.assignedAt = assignedAt;
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

    public void setActive(Boolean active) {
        this.isActive = active;
    }

    public LocalDateTime getCreatedAt() {
        return createdAt;
    }

    public void setCreatedAt(LocalDateTime createdAt) {
        this.createdAt = createdAt;
    }

    public LocalDateTime getUpdatedAt() {
        return updatedAt;
    }

    public void setUpdatedAt(LocalDateTime updatedAt) {
        this.updatedAt = updatedAt;
    }

    // 业务方法

    /**
     * 检查用户角色是否激活
     */
    public boolean isActive() {
        return Boolean.TRUE.equals(this.isActive);
    }

    /**
     * 检查用户角色是否已过期
     */
    public boolean isExpired() {
        return this.expiresAt != null && this.expiresAt.isBefore(LocalDateTime.now());
    }

    /**
     * 检查用户角色是否有效 (激活且未过期)
     */
    public boolean isValid() {
        return isActive() && !isExpired();
    }

    /**
     * 检查用户角色是否即将过期
     */
    public boolean isExpiring(int days) {
        if (this.expiresAt == null) {
            return false;
        }
        LocalDateTime threshold = LocalDateTime.now().plusDays(days);
        return this.expiresAt.isBefore(threshold) && this.expiresAt.isAfter(LocalDateTime.now());
    }

    /**
     * 检查是否有分配者
     */
    public boolean hasAssignedBy() {
        return this.assignedBy != null;
    }

    /**
     * 获取角色描述信息
     */
    public String getRoleDescription() {
        if (this.roleName != null && this.roleCode != null) {
            return String.format("%s (%s)", this.roleName, this.roleCode);
        } else if (this.roleName != null) {
            return this.roleName;
        } else if (this.roleCode != null) {
            return this.roleCode;
        }
        return "角色信息未加载";
    }

    /**
     * 转换为实体对象
     */
    public UserRole toEntity() {
        UserRole userRole = new UserRole();
        userRole.setId(this.id);
        userRole.setUserId(this.userId);
        userRole.setRoleId(this.roleId);
        userRole.setTenantId(this.tenantId);
        userRole.setAssignedBy(this.assignedBy);
        userRole.setAssignedAt(this.assignedAt);
        userRole.setExpiresAt(this.expiresAt);
        userRole.setIsActive(this.isActive);
        return userRole;
    }

    @Override
    public String toString() {
        return "UserRoleDTO{" +
                "id=" + id +
                ", userId=" + userId +
                ", roleId=" + roleId +
                ", tenantId=" + tenantId +
                ", roleName='" + roleName + '\'' +
                ", roleCode='" + roleCode + '\'' +
                ", assignedBy=" + assignedBy +
                ", assignedAt=" + assignedAt +
                ", expiresAt=" + expiresAt +
                ", isActive=" + isActive +
                '}';
    }
} 