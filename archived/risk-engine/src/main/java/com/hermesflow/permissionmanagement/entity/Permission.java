package com.hermesflow.permissionmanagement.entity;

import jakarta.persistence.*;
import org.springframework.data.annotation.CreatedDate;
import org.springframework.data.annotation.LastModifiedDate;
import org.springframework.data.jpa.domain.support.AuditingEntityListener;

import java.time.LocalDateTime;
import java.util.Objects;
import java.util.UUID;

/**
 * 权限实体类
 * 
 * @author HermesFlow Team
 * @version 1.0.0
 * @since 2024-12-30
 */
@Entity
@Table(name = "permissions", indexes = {
    @Index(name = "idx_permissions_resource", columnList = "resource"),
    @Index(name = "idx_permissions_action", columnList = "action"),
    @Index(name = "idx_permissions_type", columnList = "permissionType"),
    @Index(name = "idx_permissions_system", columnList = "isSystem"),
    @Index(name = "idx_permissions_resource_action", columnList = "resource, action")
})
@EntityListeners(AuditingEntityListener.class)
public class Permission {

    @Id
    @GeneratedValue(strategy = GenerationType.UUID)
    private UUID id;

    /**
     * 权限名称
     */
    @Column(name = "name", nullable = false, length = 100)
    private String name;

    /**
     * 权限代码 (如: user:create)
     */
    @Column(name = "code", nullable = false, unique = true, length = 100)
    private String code;

    /**
     * 资源类型 (如: user, role, strategy)
     */
    @Column(name = "resource", nullable = false, length = 100)
    private String resource;

    /**
     * 操作类型 (如: create, read, update, delete)
     */
    @Column(name = "action", nullable = false, length = 50)
    private String action;

    /**
     * 权限描述
     */
    @Column(name = "description", columnDefinition = "TEXT")
    private String description;

    /**
     * 权限类型: FUNCTIONAL, DATA, SYSTEM
     */
    @Enumerated(EnumType.STRING)
    @Column(name = "permission_type", nullable = false, length = 20)
    private PermissionType permissionType = PermissionType.FUNCTIONAL;

    /**
     * 是否为系统权限
     */
    @Column(name = "is_system", nullable = false)
    private Boolean isSystem = false;

    /**
     * 是否激活
     */
    @Column(name = "is_active", nullable = false)
    private Boolean isActive = true;

    /**
     * 创建时间
     */
    @CreatedDate
    @Column(name = "created_at", nullable = false, updatable = false)
    private LocalDateTime createdAt;

    /**
     * 更新时间
     */
    @LastModifiedDate
    @Column(name = "updated_at", nullable = false)
    private LocalDateTime updatedAt;

    /**
     * 权限类型枚举
     */
    public enum PermissionType {
        /**
         * 功能权限
         */
        FUNCTIONAL,
        
        /**
         * 数据权限
         */
        DATA,
        
        /**
         * 系统权限
         */
        SYSTEM
    }

    // 默认构造函数
    public Permission() {}

    // 构造函数
    public Permission(String name, String code, String resource, String action) {
        this.name = name;
        this.code = code;
        this.resource = resource;
        this.action = action;
    }

    public Permission(String name, String code, String resource, String action, 
                     String description, PermissionType permissionType, Boolean isSystem) {
        this.name = name;
        this.code = code;
        this.resource = resource;
        this.action = action;
        this.description = description;
        this.permissionType = permissionType;
        this.isSystem = isSystem;
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

    public PermissionType getPermissionType() {
        return permissionType;
    }

    public void setPermissionType(PermissionType permissionType) {
        this.permissionType = permissionType;
    }

    public Boolean getIsSystem() {
        return isSystem;
    }

    public void setIsSystem(Boolean isSystem) {
        this.isSystem = isSystem;
    }

    public Boolean getIsActive() {
        return isActive;
    }

    public void setIsActive(Boolean isActive) {
        this.isActive = isActive;
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
     * 检查是否为系统权限 (别名方法)
     */
    public boolean isSystem() {
        return Boolean.TRUE.equals(this.isSystem);
    }

    /**
     * 检查是否激活
     */
    public boolean isActive() {
        return Boolean.TRUE.equals(this.isActive);
    }

    /**
     * 检查是否为功能权限
     */
    public boolean isFunctionalPermission() {
        return PermissionType.FUNCTIONAL.equals(this.permissionType);
    }

    /**
     * 检查是否为数据权限
     */
    public boolean isDataPermission() {
        return PermissionType.DATA.equals(this.permissionType);
    }

    /**
     * 获取权限的完整描述
     */
    public String getFullDescription() {
        return String.format("%s (%s:%s)", name, resource, action);
    }

    @Override
    public boolean equals(Object o) {
        if (this == o) return true;
        if (o == null || getClass() != o.getClass()) return false;
        Permission that = (Permission) o;
        return Objects.equals(code, that.code);
    }

    @Override
    public int hashCode() {
        return Objects.hash(code);
    }

    @Override
    public String toString() {
        return "Permission{" +
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