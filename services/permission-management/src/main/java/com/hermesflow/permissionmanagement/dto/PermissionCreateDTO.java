package com.hermesflow.permissionmanagement.dto;

import com.hermesflow.permissionmanagement.entity.Permission;
import jakarta.validation.constraints.NotBlank;
import jakarta.validation.constraints.NotNull;
import jakarta.validation.constraints.Size;

/**
 * 权限创建DTO
 * 
 * @author HermesFlow Team
 * @version 1.0.0
 * @since 2024-12-30
 */
public class PermissionCreateDTO {

    @NotBlank(message = "权限名称不能为空")
    @Size(max = 100, message = "权限名称长度不能超过100个字符")
    private String name;

    @NotBlank(message = "权限代码不能为空")
    @Size(max = 100, message = "权限代码长度不能超过100个字符")
    private String code;

    @NotBlank(message = "资源类型不能为空")
    @Size(max = 100, message = "资源类型长度不能超过100个字符")
    private String resource;

    @NotBlank(message = "操作类型不能为空")
    @Size(max = 50, message = "操作类型长度不能超过50个字符")
    private String action;

    @Size(max = 1000, message = "权限描述长度不能超过1000个字符")
    private String description;

    @NotNull(message = "权限类型不能为空")
    private Permission.PermissionType permissionType = Permission.PermissionType.FUNCTIONAL;

    private Boolean isSystem = false;

    // 默认构造函数
    public PermissionCreateDTO() {}

    // 构造函数
    public PermissionCreateDTO(String name, String code, String resource, String action) {
        this.name = name;
        this.code = code;
        this.resource = resource;
        this.action = action;
    }

    public PermissionCreateDTO(String name, String code, String resource, String action, 
                              String description, Permission.PermissionType permissionType, Boolean isSystem) {
        this.name = name;
        this.code = code;
        this.resource = resource;
        this.action = action;
        this.description = description;
        this.permissionType = permissionType;
        this.isSystem = isSystem;
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

    public String getResource() {
        return resource;
    }

    public void setResource(String resource) {
        this.resource = resource;
    }

    public String getAction() {
        return action;
    }

    public void setAction(String action) {
        this.action = action;
    }

    public String getDescription() {
        return description;
    }

    public void setDescription(String description) {
        this.description = description;
    }

    public Permission.PermissionType getPermissionType() {
        return permissionType;
    }

    public void setPermissionType(Permission.PermissionType permissionType) {
        this.permissionType = permissionType;
    }

    public Boolean getIsSystem() {
        return isSystem;
    }

    public void setIsSystem(Boolean isSystem) {
        this.isSystem = isSystem;
    }

    // 业务方法

    /**
     * 转换为实体对象
     */
    public Permission toEntity() {
        Permission permission = new Permission();
        permission.setName(this.name);
        permission.setCode(this.code);
        permission.setResource(this.resource);
        permission.setAction(this.action);
        permission.setDescription(this.description);
        permission.setPermissionType(this.permissionType != null ? this.permissionType : Permission.PermissionType.FUNCTIONAL);
        permission.setIsSystem(this.isSystem != null ? this.isSystem : false);
        return permission;
    }

    /**
     * 验证权限代码格式
     */
    public boolean isValidCodeFormat() {
        if (code == null || code.trim().isEmpty()) {
            return false;
        }
        // 权限代码格式: resource:action (如: user:create)
        return code.matches("^[a-zA-Z0-9_-]+:[a-zA-Z0-9_-]+$");
    }

    @Override
    public String toString() {
        return "PermissionCreateDTO{" +
                "name='" + name + '\'' +
                ", code='" + code + '\'' +
                ", resource='" + resource + '\'' +
                ", action='" + action + '\'' +
                ", permissionType=" + permissionType +
                ", isSystem=" + isSystem +
                '}';
    }
} 