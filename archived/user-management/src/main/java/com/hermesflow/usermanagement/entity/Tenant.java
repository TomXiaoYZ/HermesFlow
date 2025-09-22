package com.hermesflow.usermanagement.entity;

import jakarta.persistence.*;
import jakarta.validation.constraints.NotBlank;
import jakarta.validation.constraints.NotNull;
import jakarta.validation.constraints.Size;
import org.springframework.data.annotation.CreatedDate;
import org.springframework.data.annotation.LastModifiedDate;
import org.springframework.data.jpa.domain.support.AuditingEntityListener;

import java.time.LocalDateTime;
import java.util.ArrayList;
import java.util.List;
import java.util.UUID;

/**
 * 租户实体类
 * 
 * 实现多租户架构的核心实体，每个租户代表一个独立的业务单元
 * 支持不同的订阅计划和资源配额管理
 */
@Entity
@Table(name = "tenants", indexes = {
    @Index(name = "idx_tenant_code", columnList = "code", unique = true),
    @Index(name = "idx_tenant_status", columnList = "status")
})
@EntityListeners(AuditingEntityListener.class)
public class Tenant {

    @Id
    @GeneratedValue(strategy = GenerationType.UUID)
    @Column(name = "id", updatable = false, nullable = false)
    private UUID id;

    @NotBlank(message = "租户名称不能为空")
    @Size(max = 100, message = "租户名称长度不能超过100个字符")
    @Column(name = "name", nullable = false, length = 100)
    private String name;

    @NotBlank(message = "租户代码不能为空")
    @Size(max = 50, message = "租户代码长度不能超过50个字符")
    @Column(name = "code", nullable = false, unique = true, length = 50)
    private String code;

    @NotNull(message = "订阅计划不能为空")
    @Enumerated(EnumType.STRING)
    @Column(name = "plan_type", nullable = false, length = 20)
    private PlanType planType = PlanType.BASIC;

    @NotNull(message = "租户状态不能为空")
    @Enumerated(EnumType.STRING)
    @Column(name = "status", nullable = false, length = 20)
    private TenantStatus status = TenantStatus.ACTIVE;

    @Column(name = "max_users")
    private Integer maxUsers = 10;

    @Column(name = "max_strategies")
    private Integer maxStrategies = 5;

    @Column(name = "max_asset_subscriptions")
    private Integer maxAssetSubscriptions = 50;

    @Column(name = "description", length = 500)
    private String description;

    @CreatedDate
    @Column(name = "created_at", nullable = false, updatable = false)
    private LocalDateTime createdAt;

    @LastModifiedDate
    @Column(name = "updated_at")
    private LocalDateTime updatedAt;

    // 关联关系
    @OneToMany(mappedBy = "tenant", cascade = CascadeType.ALL, fetch = FetchType.LAZY)
    private List<User> users = new ArrayList<>();

    @OneToMany(mappedBy = "tenant", cascade = CascadeType.ALL, fetch = FetchType.LAZY)
    private List<TenantConfig> configs = new ArrayList<>();

    // 构造函数
    public Tenant() {}

    public Tenant(String name, String code, PlanType planType) {
        this.name = name;
        this.code = code;
        this.planType = planType;
        this.status = TenantStatus.ACTIVE;
    }

    // 业务方法
    public boolean isActive() {
        return status == TenantStatus.ACTIVE;
    }

    public boolean canAddUser() {
        return users.size() < maxUsers;
    }

    public boolean canAddStrategy() {
        return maxStrategies == null || maxStrategies > 0;
    }

    public boolean canAddAssetSubscription() {
        return maxAssetSubscriptions == null || maxAssetSubscriptions > 0;
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

    public PlanType getPlanType() {
        return planType;
    }

    public void setPlanType(PlanType planType) {
        this.planType = planType;
    }

    public TenantStatus getStatus() {
        return status;
    }

    public void setStatus(TenantStatus status) {
        this.status = status;
    }

    public Integer getMaxUsers() {
        return maxUsers;
    }

    public void setMaxUsers(Integer maxUsers) {
        this.maxUsers = maxUsers;
    }

    public Integer getMaxStrategies() {
        return maxStrategies;
    }

    public void setMaxStrategies(Integer maxStrategies) {
        this.maxStrategies = maxStrategies;
    }

    public Integer getMaxAssetSubscriptions() {
        return maxAssetSubscriptions;
    }

    public void setMaxAssetSubscriptions(Integer maxAssetSubscriptions) {
        this.maxAssetSubscriptions = maxAssetSubscriptions;
    }

    public String getDescription() {
        return description;
    }

    public void setDescription(String description) {
        this.description = description;
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

    public List<User> getUsers() {
        return users;
    }

    public void setUsers(List<User> users) {
        this.users = users;
    }

    public List<TenantConfig> getConfigs() {
        return configs;
    }

    public void setConfigs(List<TenantConfig> configs) {
        this.configs = configs;
    }

    @Override
    public boolean equals(Object o) {
        if (this == o) return true;
        if (!(o instanceof Tenant tenant)) return false;
        return id != null && id.equals(tenant.id);
    }

    @Override
    public int hashCode() {
        return getClass().hashCode();
    }

    @Override
    public String toString() {
        return "Tenant{" +
                "id=" + id +
                ", name='" + name + '\'' +
                ", code='" + code + '\'' +
                ", planType=" + planType +
                ", status=" + status +
                '}';
    }

    /**
     * 订阅计划类型枚举
     */
    public enum PlanType {
        BASIC("基础版", 10, 5, 50),
        PRO("专业版", 50, 20, 200),
        ENTERPRISE("企业版", 200, 100, 1000);

        private final String displayName;
        private final int maxUsers;
        private final int maxStrategies;
        private final int maxAssetSubscriptions;

        PlanType(String displayName, int maxUsers, int maxStrategies, int maxAssetSubscriptions) {
            this.displayName = displayName;
            this.maxUsers = maxUsers;
            this.maxStrategies = maxStrategies;
            this.maxAssetSubscriptions = maxAssetSubscriptions;
        }

        public String getDisplayName() {
            return displayName;
        }

        public int getMaxUsers() {
            return maxUsers;
        }

        public int getMaxStrategies() {
            return maxStrategies;
        }

        public int getMaxAssetSubscriptions() {
            return maxAssetSubscriptions;
        }
    }

    /**
     * 租户状态枚举
     */
    public enum TenantStatus {
        ACTIVE("激活"),
        SUSPENDED("暂停"),
        DELETED("已删除");

        private final String displayName;

        TenantStatus(String displayName) {
            this.displayName = displayName;
        }

        public String getDisplayName() {
            return displayName;
        }
    }
} 