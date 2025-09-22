package com.hermesflow.permissionmanagement.dto;

import java.time.LocalDateTime;
import java.util.UUID;

/**
 * 角色使用情况DTO
 * 
 * @author HermesFlow Team
 * @version 1.0.0
 * @since 2024-12-30
 */
public class RoleUsageDTO {

    /**
     * 角色ID
     */
    private UUID roleId;

    /**
     * 角色代码
     */
    private String roleCode;

    /**
     * 角色名称
     */
    private String roleName;

    /**
     * 角色类型
     */
    private String roleType;

    /**
     * 分配的用户数量
     */
    private long assignedUserCount;

    /**
     * 活跃用户数量
     */
    private long activeUserCount;

    /**
     * 权限数量
     */
    private long permissionCount;

    /**
     * 最后分配时间
     */
    private LocalDateTime lastAssignedTime;

    /**
     * 最后使用时间
     */
    private LocalDateTime lastUsedTime;

    /**
     * 是否活跃
     */
    private boolean active;

    /**
     * 默认构造函数
     */
    public RoleUsageDTO() {}

    /**
     * 全参构造函数
     */
    public RoleUsageDTO(UUID roleId, String roleCode, String roleName, String roleType,
                       long assignedUserCount, long activeUserCount, long permissionCount,
                       LocalDateTime lastAssignedTime, LocalDateTime lastUsedTime, boolean active) {
        this.roleId = roleId;
        this.roleCode = roleCode;
        this.roleName = roleName;
        this.roleType = roleType;
        this.assignedUserCount = assignedUserCount;
        this.activeUserCount = activeUserCount;
        this.permissionCount = permissionCount;
        this.lastAssignedTime = lastAssignedTime;
        this.lastUsedTime = lastUsedTime;
        this.active = active;
    }

    // Getters and Setters
    public UUID getRoleId() {
        return roleId;
    }

    public void setRoleId(UUID roleId) {
        this.roleId = roleId;
    }

    public String getRoleCode() {
        return roleCode;
    }

    public void setRoleCode(String roleCode) {
        this.roleCode = roleCode;
    }

    public String getRoleName() {
        return roleName;
    }

    public void setRoleName(String roleName) {
        this.roleName = roleName;
    }

    public String getRoleType() {
        return roleType;
    }

    public void setRoleType(String roleType) {
        this.roleType = roleType;
    }

    public long getAssignedUserCount() {
        return assignedUserCount;
    }

    public void setAssignedUserCount(long assignedUserCount) {
        this.assignedUserCount = assignedUserCount;
    }

    public long getActiveUserCount() {
        return activeUserCount;
    }

    public void setActiveUserCount(long activeUserCount) {
        this.activeUserCount = activeUserCount;
    }

    public long getPermissionCount() {
        return permissionCount;
    }

    public void setPermissionCount(long permissionCount) {
        this.permissionCount = permissionCount;
    }

    public LocalDateTime getLastAssignedTime() {
        return lastAssignedTime;
    }

    public void setLastAssignedTime(LocalDateTime lastAssignedTime) {
        this.lastAssignedTime = lastAssignedTime;
    }

    public LocalDateTime getLastUsedTime() {
        return lastUsedTime;
    }

    public void setLastUsedTime(LocalDateTime lastUsedTime) {
        this.lastUsedTime = lastUsedTime;
    }

    public boolean isActive() {
        return active;
    }

    public void setActive(boolean active) {
        this.active = active;
    }

    @Override
    public String toString() {
        return "RoleUsageDTO{" +
                "roleId=" + roleId +
                ", roleCode='" + roleCode + '\'' +
                ", roleName='" + roleName + '\'' +
                ", roleType='" + roleType + '\'' +
                ", assignedUserCount=" + assignedUserCount +
                ", activeUserCount=" + activeUserCount +
                ", permissionCount=" + permissionCount +
                ", lastAssignedTime=" + lastAssignedTime +
                ", lastUsedTime=" + lastUsedTime +
                ", active=" + active +
                '}';
    }
} 