package com.hermesflow.permissionmanagement.dto;

import com.hermesflow.permissionmanagement.entity.Role;
import jakarta.validation.constraints.NotBlank;
import jakarta.validation.constraints.NotNull;
import jakarta.validation.constraints.Size;

import java.util.UUID;

/**
 * 角色创建DTO
 * 
 * @author HermesFlow Team
 * @version 1.0.0
 * @since 2024-12-30
 */
public class RoleCreateDTO {

    @NotNull(message = "租户ID不能为空")
    private UUID tenantId;

    @NotBlank(message = "角色名称不能为空")
    @Size(max = 100, message = "角色名称长度不能超过100个字符")
    private String name;

    @NotBlank(message = "角色代码不能为空")
    @Size(max = 50, message = "角色代码长度不能超过50个字符")
    private String code;

    @Size(max = 1000, message = "角色描述长度不能超过1000个字符")
    private String description;

    private Role.RoleType roleType = Role.RoleType.CUSTOM;

    private UUID parentRoleId;

    private Boolean isActive = true;

    // 默认构造函数
    public RoleCreateDTO() {}

    // 构造函数
    public RoleCreateDTO(UUID tenantId, String name, String code) {
        this.tenantId = tenantId;
        this.name = name;
        this.code = code;
    }

    public RoleCreateDTO(UUID tenantId, String name, String code, String description, Role.RoleType roleType) {
        this.tenantId = tenantId;
        this.name = name;
        this.code = code;
        this.description = description;
        this.roleType = roleType;
    }

    // Getters and Setters
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

    public Boolean getIsActive() {
        return isActive;
    }

    public void setIsActive(Boolean isActive) {
        this.isActive = isActive;
    }

    // 业务方法

    /**
     * 转换为实体对象
     */
    public Role toEntity() {
        Role role = new Role();
        role.setTenantId(this.tenantId);
        role.setName(this.name);
        role.setCode(this.code);
        role.setDescription(this.description);
        role.setRoleType(this.roleType != null ? this.roleType : Role.RoleType.CUSTOM);
        role.setParentRoleId(this.parentRoleId);
        role.setIsActive(this.isActive != null ? this.isActive : true);
        return role;
    }

    /**
     * 验证角色代码格式
     */
    public boolean isValidCodeFormat() {
        if (code == null || code.trim().isEmpty()) {
            return false;
        }
        // 角色代码格式: 字母、数字、下划线、连字符
        return code.matches("^[a-zA-Z0-9_-]+$");
    }

    @Override
    public String toString() {
        return "RoleCreateDTO{" +
                "tenantId=" + tenantId +
                ", name='" + name + '\'' +
                ", code='" + code + '\'' +
                ", roleType=" + roleType +
                ", parentRoleId=" + parentRoleId +
                ", isActive=" + isActive +
                '}';
    }
} 