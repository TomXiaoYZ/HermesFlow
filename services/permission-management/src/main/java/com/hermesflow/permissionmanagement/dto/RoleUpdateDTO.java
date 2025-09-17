package com.hermesflow.permissionmanagement.dto;

import com.hermesflow.permissionmanagement.entity.Role;
import jakarta.validation.constraints.Size;

import java.util.UUID;

/**
 * 角色更新DTO
 * 
 * @author HermesFlow Team
 * @version 1.0.0
 * @since 2024-12-30
 */
public class RoleUpdateDTO {

    @Size(max = 100, message = "角色名称长度不能超过100个字符")
    private String name;

    @Size(max = 50, message = "角色代码长度不能超过50个字符")
    private String code;

    @Size(max = 1000, message = "角色描述长度不能超过1000个字符")
    private String description;

    private Role.RoleType roleType;

    private UUID parentRoleId;

    private Boolean isActive;

    // 默认构造函数
    public RoleUpdateDTO() {}

    // 构造函数
    public RoleUpdateDTO(String name, String description) {
        this.name = name;
        this.description = description;
    }

    public RoleUpdateDTO(String name, String description, UUID parentRoleId, Boolean isActive) {
        this.name = name;
        this.description = description;
        this.parentRoleId = parentRoleId;
        this.isActive = isActive;
    }

    // Getters and Setters
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

    public Boolean getIsActive() {
        return isActive;
    }

    public void setIsActive(Boolean isActive) {
        this.isActive = isActive;
    }

    // 业务方法

    /**
     * 检查是否有更新
     */
    public boolean hasUpdates() {
        return name != null || code != null || description != null || 
               roleType != null || parentRoleId != null || isActive != null;
    }

    /**
     * 应用更新到角色实体
     */
    public void applyTo(Role role) {
        if (name != null) {
            role.setName(name);
        }
        if (code != null) {
            role.setCode(code);
        }
        if (description != null) {
            role.setDescription(description);
        }
        if (roleType != null) {
            role.setRoleType(roleType);
        }
        if (parentRoleId != null) {
            role.setParentRoleId(parentRoleId);
        }
        if (isActive != null) {
            role.setIsActive(isActive);
        }
    }

    @Override
    public String toString() {
        return "RoleUpdateDTO{" +
                "name='" + name + '\'' +
                ", code='" + code + '\'' +
                ", description='" + description + '\'' +
                ", roleType=" + roleType +
                ", parentRoleId=" + parentRoleId +
                ", isActive=" + isActive +
                '}';
    }
} 