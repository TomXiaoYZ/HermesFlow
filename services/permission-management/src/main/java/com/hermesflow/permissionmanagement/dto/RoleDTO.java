package com.hermesflow.permissionmanagement.dto;

import com.hermesflow.permissionmanagement.entity.Role;

import java.time.LocalDateTime;
import java.util.ArrayList;
import java.util.List;
import java.util.UUID;

/**
 * 角色DTO
 * 
 * @author HermesFlow Team
 * @version 1.0.0
 * @since 2024-12-30
 */
public class RoleDTO {

    private UUID id;
    private UUID tenantId;
    private String name;
    private String code;
    private String description;
    private Role.RoleType roleType;
    private UUID parentRoleId;
    private String parentRoleName;
    private List<RoleDTO> childRoles;
    private Boolean isActive;
    private LocalDateTime createdAt;
    private LocalDateTime updatedAt;

    // 默认构造函数
    public RoleDTO() {}

    // 构造函数
    public RoleDTO(UUID id, UUID tenantId, String name, String code) {
        this.id = id;
        this.tenantId = tenantId;
        this.name = name;
        this.code = code;
    }

    // 从实体转换的构造函数
    public RoleDTO(Role role) {
        this.id = role.getId();
        this.tenantId = role.getTenantId();
        this.name = role.getName();
        this.code = role.getCode();
        this.description = role.getDescription();
        this.roleType = role.getRoleType();
        this.parentRoleId = role.getParentRoleId();
        this.isActive = role.getIsActive();
        this.createdAt = role.getCreatedAt();
        this.updatedAt = role.getUpdatedAt();
        
        // 设置父角色名称
        if (role.getParentRole() != null) {
            this.parentRoleName = role.getParentRole().getName();
        }
        
        // 转换子角色
        if (role.getChildRoles() != null && !role.getChildRoles().isEmpty()) {
            this.childRoles = new ArrayList<>();
            for (Role childRole : role.getChildRoles()) {
                this.childRoles.add(new RoleDTO(childRole));
            }
        }
    }

    // Getters and Setters
    public UUID getId() {
        return id;
    }

    public void setId(UUID id) {
        this.id = id;
    }

    public UUID getTenantId() {
        return tenantId;
    }

    public void setTenantId(UUID tenantId) {
        this.tenantId = tenantId;
    }

    public String getName() {
        return name;
    }

    public void setName(String name) {
        this.name = name;
    }

    public String getCode() {
        return code;
    }

    public void setCode(String code) {
        this.code = code;
    }

    public String getDescription() {
        return description;
    }

    public void setDescription(String description) {
        this.description = description;
    }

    public Role.RoleType getRoleType() {
        return roleType;
    }

    public void setRoleType(Role.RoleType roleType) {
        this.roleType = roleType;
    }

    public UUID getParentRoleId() {
        return parentRoleId;
    }

    public void setParentRoleId(UUID parentRoleId) {
        this.parentRoleId = parentRoleId;
    }

    public String getParentRoleName() {
        return parentRoleName;
    }

    public void setParentRoleName(String parentRoleName) {
        this.parentRoleName = parentRoleName;
    }

    public List<RoleDTO> getChildRoles() {
        return childRoles;
    }

    public void setChildRoles(List<RoleDTO> childRoles) {
        this.childRoles = childRoles;
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
     * 检查角色是否激活
     */
    public boolean isActive() {
        return Boolean.TRUE.equals(this.isActive);
    }

    /**
     * 检查是否为系统角色
     */
    public boolean isSystemRole() {
        return Role.RoleType.SYSTEM.equals(this.roleType);
    }

    /**
     * 检查是否为预定义角色
     */
    public boolean isPredefinedRole() {
        return Role.RoleType.PREDEFINED.equals(this.roleType);
    }

    /**
     * 检查是否为自定义角色
     */
    public boolean isCustomRole() {
        return Role.RoleType.CUSTOM.equals(this.roleType);
    }

    /**
     * 检查是否有父角色
     */
    public boolean hasParentRole() {
        return this.parentRoleId != null;
    }

    /**
     * 检查是否有子角色
     */
    public boolean hasChildRoles() {
        return this.childRoles != null && !this.childRoles.isEmpty();
    }

    /**
     * 获取角色的完整描述
     */
    public String getFullDescription() {
        return String.format("%s (%s) - %s", name, code, roleType);
    }

    /**
     * 转换为实体对象
     */
    public Role toEntity() {
        Role role = new Role();
        role.setId(this.id);
        role.setTenantId(this.tenantId);
        role.setName(this.name);
        role.setCode(this.code);
        role.setDescription(this.description);
        role.setRoleType(this.roleType);
        role.setParentRoleId(this.parentRoleId);
        role.setIsActive(this.isActive);
        return role;
    }

    @Override
    public String toString() {
        return "RoleDTO{" +
                "id=" + id +
                ", tenantId=" + tenantId +
                ", name='" + name + '\'' +
                ", code='" + code + '\'' +
                ", roleType=" + roleType +
                ", parentRoleId=" + parentRoleId +
                ", isActive=" + isActive +
                '}';
    }
} 