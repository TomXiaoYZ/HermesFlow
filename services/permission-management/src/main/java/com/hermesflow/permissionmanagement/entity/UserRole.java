package com.hermesflow.permissionmanagement.entity;

import jakarta.persistence.*;
import org.springframework.data.annotation.CreatedDate;
import org.springframework.data.annotation.LastModifiedDate;
import org.springframework.data.jpa.domain.support.AuditingEntityListener;

import java.time.LocalDateTime;
import java.util.Objects;
import java.util.UUID;

/**
 * 用户角色关联实体类
 * 
 * @author HermesFlow Team
 * @version 1.0.0
 * @since 2024-12-30
 */
@Entity
@Table(name = "user_roles",
       uniqueConstraints = @UniqueConstraint(columnNames = {"user_id", "role_id", "tenant_id"}),
       indexes = {
           @Index(name = "idx_user_roles_user", columnList = "userId"),
           @Index(name = "idx_user_roles_role", columnList = "roleId"),
           @Index(name = "idx_user_roles_tenant", columnList = "tenantId"),
           @Index(name = "idx_user_roles_active", columnList = "isActive"),
           @Index(name = "idx_user_roles_expires", columnList = "expiresAt"),
           @Index(name = "idx_user_roles_user_tenant_active", columnList = "userId, tenantId, isActive")
       })
@EntityListeners(AuditingEntityListener.class)
public class UserRole {

    @Id
    @GeneratedValue(strategy = GenerationType.UUID)
    private UUID id;

    /**
     * 用户ID
     */
    @Column(name = "user_id", nullable = false)
    private UUID userId;

    /**
     * 角色ID
     */
    @Column(name = "role_id", nullable = false)
    private UUID roleId;

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
     * 分配者ID
     */
    @Column(name = "assigned_by")
    private UUID assignedBy;

    /**
     * 分配时间
     */
    @Column(name = "assigned_at", nullable = false)
    private LocalDateTime assignedAt;

    /**
     * 过期时间 (可选)
     */
    @Column(name = "expires_at")
    private LocalDateTime expiresAt;

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
    public UserRole() {
        this.assignedAt = LocalDateTime.now();
    }

    // 构造函数
    public UserRole(UUID userId, UUID roleId, UUID tenantId) {
        this.userId = userId;
        this.roleId = roleId;
        this.tenantId = tenantId;
        this.assignedAt = LocalDateTime.now();
    }

    public UserRole(UUID userId, UUID roleId, UUID tenantId, UUID assignedBy) {
        this.userId = userId;
        this.roleId = roleId;
        this.tenantId = tenantId;
        this.assignedBy = assignedBy;
        this.assignedAt = LocalDateTime.now();
    }

    public UserRole(UUID userId, UUID roleId, UUID tenantId, UUID assignedBy, LocalDateTime expiresAt) {
        this.userId = userId;
        this.roleId = roleId;
        this.tenantId = tenantId;
        this.assignedBy = assignedBy;
        this.assignedAt = LocalDateTime.now();
        this.expiresAt = expiresAt;
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

    public Role getRole() {
        return role;
    }

    public void setRole(Role role) {
        this.role = role;
        if (role != null) {
            this.roleId = role.getId();
        }
    }

    public UUID getAssignedBy() {
        return assignedBy;
    }

    public void setAssignedBy(UUID assignedBy) {
        this.assignedBy = assignedBy;
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
     * 检查是否有过期时间
     */
    public boolean hasExpirationTime() {
        return this.expiresAt != null;
    }

    /**
     * 激活用户角色
     */
    public void activate() {
        this.isActive = true;
    }

    /**
     * 停用用户角色
     */
    public void deactivate() {
        this.isActive = false;
    }

    /**
     * 设置过期时间
     */
    public void setExpiration(LocalDateTime expiresAt) {
        this.expiresAt = expiresAt;
    }

    /**
     * 移除过期时间 (永不过期)
     */
    public void removeExpiration() {
        this.expiresAt = null;
    }

    /**
     * 延长过期时间
     */
    public void extendExpiration(long days) {
        if (this.expiresAt != null) {
            this.expiresAt = this.expiresAt.plusDays(days);
        } else {
            this.expiresAt = LocalDateTime.now().plusDays(days);
        }
    }

    /**
     * 获取剩余有效天数
     */
    public long getRemainingDays() {
        if (this.expiresAt == null) {
            return Long.MAX_VALUE; // 永不过期
        }
        LocalDateTime now = LocalDateTime.now();
        if (this.expiresAt.isBefore(now)) {
            return 0; // 已过期
        }
        return java.time.Duration.between(now, this.expiresAt).toDays();
    }

    /**
     * 检查是否即将过期 (7天内)
     */
    public boolean isExpiringSoon() {
        return hasExpirationTime() && getRemainingDays() <= 7 && getRemainingDays() > 0;
    }

    @Override
    public boolean equals(Object o) {
        if (this == o) return true;
        if (o == null || getClass() != o.getClass()) return false;
        UserRole userRole = (UserRole) o;
        return Objects.equals(userId, userRole.userId) &&
               Objects.equals(roleId, userRole.roleId) &&
               Objects.equals(tenantId, userRole.tenantId);
    }

    @Override
    public int hashCode() {
        return Objects.hash(userId, roleId, tenantId);
    }

    @Override
    public String toString() {
        return "UserRole{" +
                "id=" + id +
                ", userId=" + userId +
                ", roleId=" + roleId +
                ", tenantId=" + tenantId +
                ", assignedBy=" + assignedBy +
                ", assignedAt=" + assignedAt +
                ", expiresAt=" + expiresAt +
                ", isActive=" + isActive +
                '}';
    }
} 