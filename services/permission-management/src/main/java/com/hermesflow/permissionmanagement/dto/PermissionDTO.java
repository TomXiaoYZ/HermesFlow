package com.hermesflow.permissionmanagement.dto;

import com.hermesflow.permissionmanagement.entity.Permission;

import java.time.LocalDateTime;
import java.util.UUID;

/**
 * 权限DTO
 * 
 * @author HermesFlow Team
 * @version 1.0.0
 * @since 2024-12-30
 */
public class PermissionDTO {

    private UUID id;
    private String name;
    private String code;
    private String resource;
    private String action;
    private String description;
    private Permission.PermissionType permissionType;
    private Boolean isSystem;
    private LocalDateTime createdAt;
    private LocalDateTime updatedAt;

    // 默认构造函数
    public PermissionDTO() {}

    // 构造函数
    public PermissionDTO(UUID id, String name, String code, String resource, String action) {
        this.id = id;
        this.name = name;
        this.code = code;
        this.resource = resource;
        this.action = action;
    }

    // 从实体转换的构造函数
    public PermissionDTO(Permission permission) {
        this.id = permission.getId();
        this.name = permission.getName();
        this.code = permission.getCode();
        this.resource = permission.getResource();
        this.action = permission.getAction();
        this.description = permission.getDescription();
        this.permissionType = permission.getPermissionType();
        this.isSystem = permission.getIsSystem();
        this.createdAt = permission.getCreatedAt();
        this.updatedAt = permission.getUpdatedAt();
    }

    // Getters and Setters
    public UUID getId() {
        return id;
    }

    public void setId(UUID id) {
        this.id = id;
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
     * 检查是否为系统权限
     */
    public boolean isSystemPermission() {
        return Boolean.TRUE.equals(this.isSystem);
    }

    /**
     * 获取权限的完整描述
     */
    public String getFullDescription() {
        return String.format("%s (%s:%s)", name, resource, action);
    }

    /**
     * 转换为实体对象
     */
    public Permission toEntity() {
        Permission permission = new Permission();
        permission.setId(this.id);
        permission.setName(this.name);
        permission.setCode(this.code);
        permission.setResource(this.resource);
        permission.setAction(this.action);
        permission.setDescription(this.description);
        permission.setPermissionType(this.permissionType);
        permission.setIsSystem(this.isSystem);
        return permission;
    }

    @Override
    public String toString() {
        return "PermissionDTO{" +
                "id=" + id +
                ", name='" + name + '\'' +
                ", code='" + code + '\'' +
                ", resource='" + resource + '\'' +
                ", action='" + action + '\'' +
                ", permissionType=" + permissionType +
                ", isSystem=" + isSystem +
                '}';
    }
} 