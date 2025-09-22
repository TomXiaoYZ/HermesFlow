package com.hermesflow.usermanagement.entity;

import jakarta.persistence.*;
import jakarta.validation.constraints.NotBlank;
import jakarta.validation.constraints.NotNull;
import jakarta.validation.constraints.Size;
import org.springframework.data.annotation.CreatedDate;
import org.springframework.data.annotation.LastModifiedDate;
import org.springframework.data.jpa.domain.support.AuditingEntityListener;

import java.time.LocalDateTime;
import java.util.UUID;

/**
 * 租户配置实体类
 * 
 * 存储租户级别的配置信息，支持加密存储敏感配置
 */
@Entity
@Table(name = "tenant_configs", 
    indexes = {
        @Index(name = "idx_tenant_config_key", columnList = "tenant_id, config_key", unique = true)
    },
    uniqueConstraints = {
        @UniqueConstraint(name = "uk_tenant_config", columnNames = {"tenant_id", "config_key"})
    }
)
@EntityListeners(AuditingEntityListener.class)
public class TenantConfig {

    @Id
    @GeneratedValue(strategy = GenerationType.UUID)
    @Column(name = "id", updatable = false, nullable = false)
    private UUID id;

    @NotNull(message = "租户ID不能为空")
    @ManyToOne(fetch = FetchType.LAZY)
    @JoinColumn(name = "tenant_id", nullable = false, foreignKey = @ForeignKey(name = "fk_tenant_config_tenant"))
    private Tenant tenant;

    @NotBlank(message = "配置键不能为空")
    @Size(max = 100, message = "配置键长度不能超过100个字符")
    @Column(name = "config_key", nullable = false, length = 100)
    private String configKey;

    @NotBlank(message = "配置值不能为空")
    @Column(name = "config_value", nullable = false, columnDefinition = "TEXT")
    private String configValue;

    @Size(max = 20, message = "配置类型长度不能超过20个字符")
    @Column(name = "config_type", length = 20)
    private String configType = "string";

    @Column(name = "is_encrypted")
    private Boolean isEncrypted = false;

    @Column(name = "description", length = 500)
    private String description;

    @CreatedDate
    @Column(name = "created_at", nullable = false, updatable = false)
    private LocalDateTime createdAt;

    @LastModifiedDate
    @Column(name = "updated_at")
    private LocalDateTime updatedAt;

    // 构造函数
    public TenantConfig() {}

    public TenantConfig(Tenant tenant, String configKey, String configValue) {
        this.tenant = tenant;
        this.configKey = configKey;
        this.configValue = configValue;
    }

    public TenantConfig(Tenant tenant, String configKey, String configValue, String configType) {
        this.tenant = tenant;
        this.configKey = configKey;
        this.configValue = configValue;
        this.configType = configType;
    }

    // 业务方法
    public boolean isStringType() {
        return "string".equals(configType);
    }

    public boolean isIntegerType() {
        return "integer".equals(configType);
    }

    public boolean isBooleanType() {
        return "boolean".equals(configType);
    }

    public boolean isJsonType() {
        return "json".equals(configType);
    }

    public Integer getIntegerValue() {
        if (isIntegerType()) {
            try {
                return Integer.valueOf(configValue);
            } catch (NumberFormatException e) {
                return null;
            }
        }
        return null;
    }

    public Boolean getBooleanValue() {
        if (isBooleanType()) {
            return Boolean.valueOf(configValue);
        }
        return null;
    }

    // Getters and Setters
    public UUID getId() {
        return id;
    }

    public void setId(UUID id) {
        this.id = id;
    }

    public Tenant getTenant() {
        return tenant;
    }

    public void setTenant(Tenant tenant) {
        this.tenant = tenant;
    }

    public String getConfigKey() {
        return configKey;
    }

    public void setConfigKey(String configKey) {
        this.configKey = configKey;
    }

    public String getConfigValue() {
        return configValue;
    }

    public void setConfigValue(String configValue) {
        this.configValue = configValue;
    }

    public String getConfigType() {
        return configType;
    }

    public void setConfigType(String configType) {
        this.configType = configType;
    }

    public Boolean getIsEncrypted() {
        return isEncrypted;
    }

    public void setIsEncrypted(Boolean isEncrypted) {
        this.isEncrypted = isEncrypted;
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

    @Override
    public boolean equals(Object o) {
        if (this == o) return true;
        if (!(o instanceof TenantConfig that)) return false;
        return id != null && id.equals(that.id);
    }

    @Override
    public int hashCode() {
        return getClass().hashCode();
    }

    @Override
    public String toString() {
        return "TenantConfig{" +
                "id=" + id +
                ", configKey='" + configKey + '\'' +
                ", configType='" + configType + '\'' +
                ", isEncrypted=" + isEncrypted +
                '}';
    }
} 