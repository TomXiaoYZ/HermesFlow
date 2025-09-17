package com.hermesflow.usermanagement.dto;

import jakarta.validation.constraints.NotBlank;

/**
 * 登录请求DTO
 */
public class LoginRequest {

    @NotBlank(message = "租户代码不能为空")
    private String tenantCode;

    @NotBlank(message = "用户名不能为空")
    private String username;

    @NotBlank(message = "密码不能为空")
    private String password;

    private String deviceInfo;
    private String userAgent;

    // 构造函数
    public LoginRequest() {}

    public LoginRequest(String tenantCode, String username, String password) {
        this.tenantCode = tenantCode;
        this.username = username;
        this.password = password;
    }

    // Getter和Setter方法
    public String getTenantCode() {
        return tenantCode;
    }

    public void setTenantCode(String tenantCode) {
        this.tenantCode = tenantCode;
    }

    public String getUsername() {
        return username;
    }

    public void setUsername(String username) {
        this.username = username;
    }

    public String getPassword() {
        return password;
    }

    public void setPassword(String password) {
        this.password = password;
    }

    public String getDeviceInfo() {
        return deviceInfo;
    }

    public void setDeviceInfo(String deviceInfo) {
        this.deviceInfo = deviceInfo;
    }

    public String getUserAgent() {
        return userAgent;
    }

    public void setUserAgent(String userAgent) {
        this.userAgent = userAgent;
    }

    @Override
    public String toString() {
        return "LoginRequest{" +
                "tenantCode='" + tenantCode + '\'' +
                ", username='" + username + '\'' +
                ", deviceInfo='" + deviceInfo + '\'' +
                ", userAgent='" + userAgent + '\'' +
                '}';
    }
} 