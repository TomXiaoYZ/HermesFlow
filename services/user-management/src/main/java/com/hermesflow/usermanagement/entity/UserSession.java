package com.hermesflow.usermanagement.entity;

import jakarta.persistence.*;
import jakarta.validation.constraints.NotBlank;
import jakarta.validation.constraints.NotNull;
import org.springframework.data.annotation.CreatedDate;
import org.springframework.data.jpa.domain.support.AuditingEntityListener;

import java.net.InetAddress;
import java.time.LocalDateTime;
import java.util.UUID;

/**
 * 用户会话实体类
 * 
 * 管理用户登录会话，支持多设备登录和会话控制
 */
@Entity
@Table(name = "user_sessions", 
    indexes = {
        @Index(name = "idx_session_token", columnList = "session_token", unique = true),
        @Index(name = "idx_refresh_token", columnList = "refresh_token", unique = true),
        @Index(name = "idx_user_active", columnList = "user_id, is_active"),
        @Index(name = "idx_session_expires", columnList = "expires_at")
    }
)
@EntityListeners(AuditingEntityListener.class)
public class UserSession {

    @Id
    @GeneratedValue(strategy = GenerationType.UUID)
    @Column(name = "id", updatable = false, nullable = false)
    private UUID id;

    @NotNull(message = "用户ID不能为空")
    @ManyToOne(fetch = FetchType.LAZY)
    @JoinColumn(name = "user_id", nullable = false, foreignKey = @ForeignKey(name = "fk_session_user"))
    private User user;

    @NotBlank(message = "会话令牌不能为空")
    @Column(name = "session_token", nullable = false, unique = true)
    private String sessionToken;

    @NotBlank(message = "刷新令牌不能为空")
    @Column(name = "refresh_token", nullable = false, unique = true)
    private String refreshToken;

    @Column(name = "ip_address", columnDefinition = "inet")
    private InetAddress ipAddress;

    @Column(name = "user_agent", columnDefinition = "TEXT")
    private String userAgent;

    @NotNull(message = "过期时间不能为空")
    @Column(name = "expires_at", nullable = false)
    private LocalDateTime expiresAt;

    @Column(name = "is_active")
    private Boolean isActive = true;

    @Column(name = "last_activity_at")
    private LocalDateTime lastActivityAt;

    @CreatedDate
    @Column(name = "created_at", nullable = false, updatable = false)
    private LocalDateTime createdAt;

    // 构造函数
    public UserSession() {}

    public UserSession(User user, String sessionToken, String refreshToken, LocalDateTime expiresAt) {
        this.user = user;
        this.sessionToken = sessionToken;
        this.refreshToken = refreshToken;
        this.expiresAt = expiresAt;
        this.isActive = true;
        this.lastActivityAt = LocalDateTime.now();
    }

    // 业务方法
    public boolean isExpired() {
        return expiresAt.isBefore(LocalDateTime.now());
    }

    public boolean isValid() {
        return isActive && !isExpired();
    }

    public void updateActivity() {
        this.lastActivityAt = LocalDateTime.now();
    }

    public void invalidate() {
        this.isActive = false;
    }

    public void extend(LocalDateTime newExpiresAt) {
        this.expiresAt = newExpiresAt;
        updateActivity();
    }

    // Getters and Setters
    public UUID getId() {
        return id;
    }

    public void setId(UUID id) {
        this.id = id;
    }

    public User getUser() {
        return user;
    }

    public void setUser(User user) {
        this.user = user;
    }

    public String getSessionToken() {
        return sessionToken;
    }

    public void setSessionToken(String sessionToken) {
        this.sessionToken = sessionToken;
    }

    public String getRefreshToken() {
        return refreshToken;
    }

    public void setRefreshToken(String refreshToken) {
        this.refreshToken = refreshToken;
    }

    public InetAddress getIpAddress() {
        return ipAddress;
    }

    public void setIpAddress(InetAddress ipAddress) {
        this.ipAddress = ipAddress;
    }

    public String getUserAgent() {
        return userAgent;
    }

    public void setUserAgent(String userAgent) {
        this.userAgent = userAgent;
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

    public LocalDateTime getLastActivityAt() {
        return lastActivityAt;
    }

    public void setLastActivityAt(LocalDateTime lastActivityAt) {
        this.lastActivityAt = lastActivityAt;
    }

    public LocalDateTime getCreatedAt() {
        return createdAt;
    }

    public void setCreatedAt(LocalDateTime createdAt) {
        this.createdAt = createdAt;
    }

    @Override
    public boolean equals(Object o) {
        if (this == o) return true;
        if (!(o instanceof UserSession that)) return false;
        return id != null && id.equals(that.id);
    }

    @Override
    public int hashCode() {
        return getClass().hashCode();
    }

    @Override
    public String toString() {
        return "UserSession{" +
                "id=" + id +
                ", sessionToken='" + sessionToken.substring(0, Math.min(10, sessionToken.length())) + "...'" +
                ", isActive=" + isActive +
                ", expiresAt=" + expiresAt +
                '}';
    }
} 