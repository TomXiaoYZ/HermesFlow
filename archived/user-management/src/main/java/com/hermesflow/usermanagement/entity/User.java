package com.hermesflow.usermanagement.entity;

import jakarta.persistence.*;
import jakarta.validation.constraints.Email;
import jakarta.validation.constraints.NotBlank;
import jakarta.validation.constraints.NotNull;
import jakarta.validation.constraints.Size;
import org.springframework.data.annotation.CreatedDate;
import org.springframework.data.annotation.LastModifiedDate;
import org.springframework.data.jpa.domain.support.AuditingEntityListener;
import org.springframework.security.core.GrantedAuthority;
import org.springframework.security.core.authority.SimpleGrantedAuthority;
import org.springframework.security.core.userdetails.UserDetails;

import java.time.LocalDateTime;
import java.util.ArrayList;
import java.util.Collection;
import java.util.List;
import java.util.UUID;

/**
 * 用户实体类
 * 
 * 实现多租户用户管理，支持Spring Security UserDetails接口
 * 包含用户基本信息、安全策略和租户关联
 */
@Entity
@Table(name = "users", 
    indexes = {
        @Index(name = "idx_user_tenant_username", columnList = "tenant_id, username", unique = true),
        @Index(name = "idx_user_tenant_email", columnList = "tenant_id, email", unique = true),
        @Index(name = "idx_user_status", columnList = "status"),
        @Index(name = "idx_user_last_login", columnList = "last_login_at")
    },
    uniqueConstraints = {
        @UniqueConstraint(name = "uk_user_tenant_username", columnNames = {"tenant_id", "username"}),
        @UniqueConstraint(name = "uk_user_tenant_email", columnNames = {"tenant_id", "email"})
    }
)
@EntityListeners(AuditingEntityListener.class)
public class User implements UserDetails {

    @Id
    @GeneratedValue(strategy = GenerationType.UUID)
    @Column(name = "id", updatable = false, nullable = false)
    private UUID id;

    @NotNull(message = "租户ID不能为空")
    @ManyToOne(fetch = FetchType.LAZY)
    @JoinColumn(name = "tenant_id", nullable = false, foreignKey = @ForeignKey(name = "fk_user_tenant"))
    private Tenant tenant;

    @NotBlank(message = "用户名不能为空")
    @Size(min = 3, max = 50, message = "用户名长度必须在3-50个字符之间")
    @Column(name = "username", nullable = false, length = 50)
    private String username;

    @NotBlank(message = "邮箱不能为空")
    @Email(message = "邮箱格式不正确")
    @Size(max = 100, message = "邮箱长度不能超过100个字符")
    @Column(name = "email", nullable = false, length = 100)
    private String email;

    @NotBlank(message = "密码不能为空")
    @Column(name = "password_hash", nullable = false)
    private String passwordHash;

    @Size(max = 50, message = "名字长度不能超过50个字符")
    @Column(name = "first_name", length = 50)
    private String firstName;

    @Size(max = 50, message = "姓氏长度不能超过50个字符")
    @Column(name = "last_name", length = 50)
    private String lastName;

    @Size(max = 20, message = "电话号码长度不能超过20个字符")
    @Column(name = "phone", length = 20)
    private String phone;

    @NotNull(message = "用户状态不能为空")
    @Enumerated(EnumType.STRING)
    @Column(name = "status", nullable = false, length = 20)
    private UserStatus status = UserStatus.ACTIVE;

    @Column(name = "email_verified")
    private Boolean emailVerified = false;

    @Column(name = "phone_verified")
    private Boolean phoneVerified = false;

    @Column(name = "last_login_at")
    private LocalDateTime lastLoginAt;

    @Column(name = "password_changed_at")
    private LocalDateTime passwordChangedAt;

    @Column(name = "failed_login_attempts")
    private Integer failedLoginAttempts = 0;

    @Column(name = "locked_until")
    private LocalDateTime lockedUntil;

    @CreatedDate
    @Column(name = "created_at", nullable = false, updatable = false)
    private LocalDateTime createdAt;

    @LastModifiedDate
    @Column(name = "updated_at")
    private LocalDateTime updatedAt;

    // 关联关系
    @OneToMany(mappedBy = "user", cascade = CascadeType.ALL, fetch = FetchType.LAZY)
    private List<UserSession> sessions = new ArrayList<>();

    // 构造函数
    public User() {}

    public User(Tenant tenant, String username, String email, String passwordHash) {
        this.tenant = tenant;
        this.username = username;
        this.email = email;
        this.passwordHash = passwordHash;
        this.status = UserStatus.ACTIVE;
        this.passwordChangedAt = LocalDateTime.now();
    }

    // 业务方法
    public boolean isActive() {
        return status == UserStatus.ACTIVE;
    }

    public boolean isLocked() {
        return status == UserStatus.LOCKED || 
               (lockedUntil != null && lockedUntil.isAfter(LocalDateTime.now()));
    }

    public boolean isAccountExpired() {
        return status == UserStatus.EXPIRED;
    }

    public boolean isCredentialsExpired() {
        // 密码90天过期策略
        return passwordChangedAt != null && 
               passwordChangedAt.plusDays(90).isBefore(LocalDateTime.now());
    }

    public void recordLoginSuccess() {
        this.lastLoginAt = LocalDateTime.now();
        this.failedLoginAttempts = 0;
        this.lockedUntil = null;
    }

    public void recordLoginFailure() {
        this.failedLoginAttempts = (this.failedLoginAttempts == null ? 0 : this.failedLoginAttempts) + 1;
        
        // 5次失败后锁定15分钟
        if (this.failedLoginAttempts >= 5) {
            this.status = UserStatus.LOCKED;
            this.lockedUntil = LocalDateTime.now().plusMinutes(15);
        }
    }

    public void unlock() {
        this.status = UserStatus.ACTIVE;
        this.failedLoginAttempts = 0;
        this.lockedUntil = null;
    }

    public String getFullName() {
        if (firstName != null && lastName != null) {
            return firstName + " " + lastName;
        } else if (firstName != null) {
            return firstName;
        } else if (lastName != null) {
            return lastName;
        }
        return username;
    }

    // Spring Security UserDetails 接口实现
    @Override
    public Collection<? extends GrantedAuthority> getAuthorities() {
        // 基础角色，后续会通过权限管理服务扩展
        List<GrantedAuthority> authorities = new ArrayList<>();
        authorities.add(new SimpleGrantedAuthority("ROLE_USER"));
        return authorities;
    }

    @Override
    public String getPassword() {
        return passwordHash;
    }

    @Override
    public String getUsername() {
        return username;
    }

    @Override
    public boolean isAccountNonExpired() {
        return !isAccountExpired();
    }

    @Override
    public boolean isAccountNonLocked() {
        return !isLocked();
    }

    @Override
    public boolean isCredentialsNonExpired() {
        return !isCredentialsExpired();
    }

    @Override
    public boolean isEnabled() {
        return isActive() && tenant.isActive();
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

    public void setUsername(String username) {
        this.username = username;
    }

    public String getEmail() {
        return email;
    }

    public void setEmail(String email) {
        this.email = email;
    }

    public String getPasswordHash() {
        return passwordHash;
    }

    public void setPasswordHash(String passwordHash) {
        this.passwordHash = passwordHash;
        this.passwordChangedAt = LocalDateTime.now();
    }

    public String getFirstName() {
        return firstName;
    }

    public void setFirstName(String firstName) {
        this.firstName = firstName;
    }

    public String getLastName() {
        return lastName;
    }

    public void setLastName(String lastName) {
        this.lastName = lastName;
    }

    public String getPhone() {
        return phone;
    }

    public void setPhone(String phone) {
        this.phone = phone;
    }

    public UserStatus getStatus() {
        return status;
    }

    public void setStatus(UserStatus status) {
        this.status = status;
    }

    public Boolean getEmailVerified() {
        return emailVerified;
    }

    public void setEmailVerified(Boolean emailVerified) {
        this.emailVerified = emailVerified;
    }

    public Boolean getPhoneVerified() {
        return phoneVerified;
    }

    public void setPhoneVerified(Boolean phoneVerified) {
        this.phoneVerified = phoneVerified;
    }

    public LocalDateTime getLastLoginAt() {
        return lastLoginAt;
    }

    public void setLastLoginAt(LocalDateTime lastLoginAt) {
        this.lastLoginAt = lastLoginAt;
    }

    public LocalDateTime getPasswordChangedAt() {
        return passwordChangedAt;
    }

    public void setPasswordChangedAt(LocalDateTime passwordChangedAt) {
        this.passwordChangedAt = passwordChangedAt;
    }

    public Integer getFailedLoginAttempts() {
        return failedLoginAttempts;
    }

    public void setFailedLoginAttempts(Integer failedLoginAttempts) {
        this.failedLoginAttempts = failedLoginAttempts;
    }

    public LocalDateTime getLockedUntil() {
        return lockedUntil;
    }

    public void setLockedUntil(LocalDateTime lockedUntil) {
        this.lockedUntil = lockedUntil;
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

    public List<UserSession> getSessions() {
        return sessions;
    }

    public void setSessions(List<UserSession> sessions) {
        this.sessions = sessions;
    }

    @Override
    public boolean equals(Object o) {
        if (this == o) return true;
        if (!(o instanceof User user)) return false;
        return id != null && id.equals(user.id);
    }

    @Override
    public int hashCode() {
        return getClass().hashCode();
    }

    @Override
    public String toString() {
        return "User{" +
                "id=" + id +
                ", username='" + username + '\'' +
                ", email='" + email + '\'' +
                ", status=" + status +
                ", tenantId=" + (tenant != null ? tenant.getId() : null) +
                '}';
    }

    /**
     * 用户状态枚举
     */
    public enum UserStatus {
        ACTIVE("激活"),
        INACTIVE("未激活"),
        LOCKED("锁定"),
        EXPIRED("过期");

        private final String displayName;

        UserStatus(String displayName) {
            this.displayName = displayName;
        }

        public String getDisplayName() {
            return displayName;
        }
    }
} 