package com.hermesflow.permissionmanagement.entity;

import jakarta.persistence.*;
import org.springframework.data.annotation.CreatedDate;
import org.springframework.data.annotation.LastModifiedDate;
import org.springframework.data.jpa.domain.support.AuditingEntityListener;

import java.time.LocalDateTime;
import java.util.ArrayList;
import java.util.List;
import java.util.Objects;
import java.util.UUID;

/**
 * 角色实体类
 * 
 * @author HermesFlow Team
 * @version 1.0.0
 * @since 2024-12-30
 */
@Entity
@Table(name = "roles", 
       uniqueConstraints = @UniqueConstraint(columnNames = {"tenant_id", "code"}),
       indexes = {
           @Index(name = "idx_roles_tenant", columnList = "tenantId"),
           @Index(name = "idx_roles_code", columnList = "code"),
           @Index(name = "idx_roles_type", columnList = "roleType"),
           @Index(name = "idx_roles_parent", columnList = "parentRoleId"),
           @Index(name = "idx_roles_active", columnList = "isActive")
       })
@EntityListeners(AuditingEntityListener.class)
public class Role {

    @Id
    @GeneratedValue(strategy = GenerationType.UUID)
    private UUID id;

    /**
     * 租户ID
     */
    @Column(name = "tenant_id", nullable = false)
    private UUID tenantId;

    /**
     * 角色名称
     */
    @Column(name = "name", nullable = false, length = 100)
    private String name;

    /**
     * 角色代码
     */
    @Column(name = "code", nullable = false, length = 50)
    private String code;

    /**
     * 角色描述
     */
    @Column(name = "description", columnDefinition = "TEXT")
    private String description;

    /**
     * 角色类型: SYSTEM, PREDEFINED, CUSTOM
     */
    @Enumerated(EnumType.STRING)
    @Column(name = "role_type", nullable = false, length = 20)
    private RoleType roleType = RoleType.CUSTOM;

    /**
     * 父角色ID (支持角色继承)
     */
    @Column(name = "parent_role_id")
    private UUID parentRoleId;

    /**
     * 父角色 (支持角色继承)
     */
    @ManyToOne(fetch = FetchType.LAZY)
    @JoinColumn(name = "parent_role_id", insertable = false, updatable = false)
    private Role parentRole;

    /**
     * 子角色列表
     */
    @OneToMany(mappedBy = "parentRole", fetch = FetchType.LAZY)
    private List<Role> childRoles = new ArrayList<>();

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
     * 角色类型枚举
     */
    public enum RoleType {
        /**
         * 系统角色 (跨租户)
         */
        SYSTEM,
        
        /**
         * 预定义角色 (租户级别)
         */
        PREDEFINED,
        
        /**
         * 自定义角色 (租户级别)
         */
        CUSTOM
    }

    // 默认构造函数
    public Role() {}

    // 构造函数
    public Role(UUID tenantId, String name, String code) {
        this.tenantId = tenantId;
        this.name = name;
        this.code = code;
    }

    public Role(UUID tenantId, String name, String code, String description, RoleType roleType) {
        this.tenantId = tenantId;
        this.name = name;
        this.code = code;
        this.description = description;
        this.roleType = roleType;
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

    public RoleType getRoleType() {
        return roleType;
    }

    public void setRoleType(RoleType roleType) {
        this.roleType = roleType;
    }

    public UUID getParentRoleId() {
        return parentRoleId;
    }

    public void setParentRoleId(UUID parentRoleId) {
        this.parentRoleId = parentRoleId;
    }

    public Role getParentRole() {
        return parentRole;
    }

    public void setParentRole(Role parentRole) {
        this.parentRole = parentRole;
        if (parentRole != null) {
            this.parentRoleId = parentRole.getId();
        }
    }

    public List<Role> getChildRoles() {
        return childRoles;
    }

    public void setChildRoles(List<Role> childRoles) {
        this.childRoles = childRoles;
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
     * 检查角色是否激活
     */
    public boolean isActive() {
        return Boolean.TRUE.equals(this.isActive);
    }

    /**
     * 检查是否为系统角色
     */
    public boolean isSystemRole() {
        return RoleType.SYSTEM.equals(this.roleType);
    }

    /**
     * 检查是否为预定义角色
     */
    public boolean isPredefinedRole() {
        return RoleType.PREDEFINED.equals(this.roleType);
    }

    /**
     * 检查是否为自定义角色
     */
    public boolean isCustomRole() {
        return RoleType.CUSTOM.equals(this.roleType);
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
     * 添加子角色
     */
    public void addChildRole(Role childRole) {
        if (this.childRoles == null) {
            this.childRoles = new ArrayList<>();
        }
        this.childRoles.add(childRole);
        childRole.setParentRole(this);
    }

    /**
     * 移除子角色
     */
    public void removeChildRole(Role childRole) {
        if (this.childRoles != null) {
            this.childRoles.remove(childRole);
            childRole.setParentRole(null);
        }
    }

    /**
     * 获取角色的完整描述
     */
    public String getFullDescription() {
        return String.format("%s (%s) - %s", name, code, roleType);
    }

    /**
     * 激活角色
     */
    public void activate() {
        this.isActive = true;
    }

    /**
     * 停用角色
     */
    public void deactivate() {
        this.isActive = false;
    }

    @Override
    public boolean equals(Object o) {
        if (this == o) return true;
        if (o == null || getClass() != o.getClass()) return false;
        Role role = (Role) o;
        return Objects.equals(tenantId, role.tenantId) && 
               Objects.equals(code, role.code);
    }

    @Override
    public int hashCode() {
        return Objects.hash(tenantId, code);
    }

    @Override
    public String toString() {
        return "Role{" +
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