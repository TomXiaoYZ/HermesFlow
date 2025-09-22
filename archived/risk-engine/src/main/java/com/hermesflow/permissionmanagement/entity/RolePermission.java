package com.hermesflow.permissionmanagement.entity;

import jakarta.persistence.*;
import org.springframework.data.annotation.CreatedDate;
import org.springframework.data.annotation.LastModifiedDate;
import org.springframework.data.jpa.domain.support.AuditingEntityListener;

import java.time.LocalDateTime;
import java.util.Objects;
import java.util.UUID;

/**
 * 角色权限关联实体类
 * 
 * @author HermesFlow Team
 * @version 1.0.0
 * @since 2024-12-30
 */
@Entity
@Table(name = "role_permissions",
       uniqueConstraints = @UniqueConstraint(columnNames = {"role_id", "permission_id", "tenant_id"}),
       indexes = {
           @Index(name = "idx_role_permissions_role", columnList = "roleId"),
           @Index(name = "idx_role_permissions_permission", columnList = "permissionId"),
           @Index(name = "idx_role_permissions_tenant", columnList = "tenantId"),
           @Index(name = "idx_role_permissions_active", columnList = "isActive"),
           @Index(name = "idx_role_permissions_role_tenant_active", columnList = "roleId, tenantId, isActive")
       })
@EntityListeners(AuditingEntityListener.class)
public class RolePermission {

    @Id
    @GeneratedValue(strategy = GenerationType.UUID)
    private UUID id;

    /**
     * 角色ID
     */
    @Column(name = "role_id", nullable = false)
    private UUID roleId;

    /**
     * 权限ID
     */
    @Column(name = "permission_id", nullable = false)
    private UUID permissionId;

    /**
     * 租户ID
     */
    @Column(name = "tenant_id", nullable = false)
    private UUID tenantId;

    /**
     * 角色关联 (懒加载)
     */
    @ManyToOne(fetch = FetchType.LAZY)
    @JoinColumn(name = "role_id", insertable = false, updatable = false)
    private Role role;

    /**
     * 权限关联 (懒加载)
     */
    @ManyToOne(fetch = FetchType.LAZY)
    @JoinColumn(name = "permission_id", insertable = false, updatable = false)
    private Permission permission;

    /**
     * 授权者ID
     */
    @Column(name = "granted_by")
    private UUID grantedBy;

    /**
     * 授权时间
     */
    @Column(name = "granted_at", nullable = false)
    private LocalDateTime grantedAt;

    /**
     * 过期时间
     */
    @Column(name = "expires_at")
    private LocalDateTime expiresAt;

    /**
     * 撤销时间
     */
    @Column(name = "revoked_at")
    private LocalDateTime revokedAt;

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

    // 默认构造函数
    public RolePermission() {
        this.grantedAt = LocalDateTime.now();
    }

    // 构造函数
    public RolePermission(UUID roleId, UUID permissionId, UUID tenantId) {
        this.roleId = roleId;
        this.permissionId = permissionId;
        this.tenantId = tenantId;
        this.grantedAt = LocalDateTime.now();
    }

    public RolePermission(UUID roleId, UUID permissionId, UUID tenantId, UUID grantedBy) {
        this.roleId = roleId;
        this.permissionId = permissionId;
        this.tenantId = tenantId;
        this.grantedBy = grantedBy;
        this.grantedAt = LocalDateTime.now();
    }

    // Getters and Setters
    public UUID getId() {
        return id;
    }

    public void setId(UUID id) {
        this.id = id;
    }

    public UUID getRoleId() {
        return roleId;
    }

    public void setRoleId(UUID roleId) {
        this.roleId = roleId;
    }

    public UUID getPermissionId() {
        return permissionId;
    }

    public void setPermissionId(UUID permissionId) {
        this.permissionId = permissionId;
    }

    public UUID getTenantId() {
        return tenantId;
    }

    public void setTenantId(UUID tenantId) {
        this.tenantId = tenantId;
    }

    public Role getRole() {
        return role;
    }

    public void setRole(Role role) {
        this.role = role;
        if (role != null) {
            this.roleId = role.getId();
        }
    }

    public Permission getPermission() {
        return permission;
    }

    public void setPermission(Permission permission) {
        this.permission = permission;
        if (permission != null) {
            this.permissionId = permission.getId();
        }
    }

    public UUID getGrantedBy() {
        return grantedBy;
    }

    public void setGrantedBy(UUID grantedBy) {
        this.grantedBy = grantedBy;
    }

    public LocalDateTime getGrantedAt() {
        return grantedAt;
    }

    public void setGrantedAt(LocalDateTime grantedAt) {
        this.grantedAt = grantedAt;
    }

    public LocalDateTime getExpiresAt() {
        return expiresAt;
    }

    public void setExpiresAt(LocalDateTime expiresAt) {
        this.expiresAt = expiresAt;
    }

    public LocalDateTime getRevokedAt() {
        return revokedAt;
    }

    public void setRevokedAt(LocalDateTime revokedAt) {
        this.revokedAt = revokedAt;
    }

    public Boolean getIsActive() {
        return isActive;
    }

    public void setIsActive(Boolean isActive) {
        this.isActive = isActive;
    }

    public void setActive(boolean active) {
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
     * 检查角色权限是否激活
     */
    public boolean isActive() {
        return Boolean.TRUE.equals(this.isActive);
    }

    /**
     * 激活角色权限
     */
    public void activate() {
        this.isActive = true;
    }

    /**
     * 停用角色权限
     */
    public void deactivate() {
        this.isActive = false;
    }

    /**
     * 检查是否有授权者
     */
    public boolean hasGrantedBy() {
        return this.grantedBy != null;
    }

    /**
     * 获取权限代码 (如果权限已加载)
     */
    public String getPermissionCode() {
        return this.permission != null ? this.permission.getCode() : null;
    }

    /**
     * 获取角色代码 (如果角色已加载)
     */
    public String getRoleCode() {
        return this.role != null ? this.role.getCode() : null;
    }

    /**
     * 获取权限描述信息
     */
    public String getPermissionDescription() {
        if (this.permission != null) {
            return String.format("%s (%s:%s)", 
                this.permission.getName(), 
                this.permission.getResource(), 
                this.permission.getAction());
        }
        return "权限信息未加载";
    }

    /**
     * 获取角色描述信息
     */
    public String getRoleDescription() {
        if (this.role != null) {
            return String.format("%s (%s)", this.role.getName(), this.role.getCode());
        }
        return "角色信息未加载";
    }

    /**
     * 检查是否为系统权限
     */
    public boolean isSystemPermission() {
        return this.permission != null && this.permission.isSystemPermission();
    }

    /**
     * 检查是否为系统角色
     */
    public boolean isSystemRole() {
        return this.role != null && this.role.isSystemRole();
    }

    @Override
    public boolean equals(Object o) {
        if (this == o) return true;
        if (o == null || getClass() != o.getClass()) return false;
        RolePermission that = (RolePermission) o;
        return Objects.equals(roleId, that.roleId) &&
               Objects.equals(permissionId, that.permissionId) &&
               Objects.equals(tenantId, that.tenantId);
    }

    @Override
    public int hashCode() {
        return Objects.hash(roleId, permissionId, tenantId);
    }

    @Override
    public String toString() {
        return "RolePermission{" +
                "id=" + id +
                ", roleId=" + roleId +
                ", permissionId=" + permissionId +
                ", tenantId=" + tenantId +
                ", grantedBy=" + grantedBy +
                ", grantedAt=" + grantedAt +
                ", isActive=" + isActive +
                '}';
    }
} 