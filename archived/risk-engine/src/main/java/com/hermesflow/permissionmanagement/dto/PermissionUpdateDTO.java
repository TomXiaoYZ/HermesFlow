package com.hermesflow.permissionmanagement.dto;

import com.hermesflow.permissionmanagement.entity.Permission;
import jakarta.validation.constraints.Size;

/**
 * 权限更新DTO
 * 
 * @author HermesFlow Team
 * @version 1.0.0
 * @since 2024-12-30
 */
public class PermissionUpdateDTO {

    @Size(max = 100, message = "权限名称长度不能超过100个字符")
    private String name;

    @Size(max = 100, message = "权限代码长度不能超过100个字符")
    private String code;

    @Size(max = 1000, message = "权限描述长度不能超过1000个字符")
    private String description;

    private Permission.PermissionType permissionType;

    // 默认构造函数
    public PermissionUpdateDTO() {}

    // 构造函数
    public PermissionUpdateDTO(String name, String description) {
        this.name = name;
        this.description = description;
    }

    public PermissionUpdateDTO(String name, String code, String description, Permission.PermissionType permissionType) {
        this.name = name;
        this.code = code;
        this.description = description;
        this.permissionType = permissionType;
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

    public Permission.PermissionType getPermissionType() {
        return permissionType;
    }

    public void setPermissionType(Permission.PermissionType permissionType) {
        this.permissionType = permissionType;
    }

    // 业务方法

    /**
     * 检查是否有更新内容
     */
    public boolean hasUpdates() {
        return name != null || code != null || description != null || permissionType != null;
    }

    /**
     * 应用更新到实体对象
     */
    public void applyTo(Permission permission) {
        if (name != null) {
            permission.setName(name);
        }
        if (code != null) {
            permission.setCode(code);
        }
        if (description != null) {
            permission.setDescription(description);
        }
        if (permissionType != null) {
            permission.setPermissionType(permissionType);
        }
    }

    @Override
    public String toString() {
        return "PermissionUpdateDTO{" +
                "name='" + name + '\'' +
                ", code='" + code + '\'' +
                ", description='" + description + '\'' +
                ", permissionType=" + permissionType +
                '}';
    }
} 