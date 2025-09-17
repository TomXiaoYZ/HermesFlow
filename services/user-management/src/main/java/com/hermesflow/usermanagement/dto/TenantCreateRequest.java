package com.hermesflow.usermanagement.dto;

import jakarta.validation.constraints.Email;
import jakarta.validation.constraints.NotBlank;
import jakarta.validation.constraints.Size;

/**
 * 租户创建请求DTO
 */
public class TenantCreateRequest {

    @NotBlank(message = "租户代码不能为空")
    @Size(min = 2, max = 50, message = "租户代码长度必须在2-50个字符之间")
    private String tenantCode;

    @NotBlank(message = "租户名称不能为空")
    @Size(min = 2, max = 100, message = "租户名称长度必须在2-100个字符之间")
    private String name;

    @Size(max = 500, message = "描述长度不能超过500个字符")
    private String description;

    @Email(message = "联系邮箱格式不正确")
    @Size(max = 100, message = "联系邮箱长度不能超过100个字符")
    private String contactEmail;

    @Size(max = 20, message = "联系电话长度不能超过20个字符")
    private String contactPhone;

    // 默认构造函数
    public TenantCreateRequest() {
    }

    // 带参数的构造函数
    public TenantCreateRequest(String tenantCode, String name, String description) {
        this.tenantCode = tenantCode;
        this.name = name;
        this.description = description;
    }

    // Getter和Setter方法
    public String getTenantCode() {
        return tenantCode;
    }

    public void setTenantCode(String tenantCode) {
        this.tenantCode = tenantCode;
    }

    public String getName() {
        return name;
    }

    public void setName(String name) {
        this.name = name;
    }

    public String getDescription() {
        return description;
    }

    public void setDescription(String description) {
        this.description = description;
    }

    public String getContactEmail() {
        return contactEmail;
    }

    public void setContactEmail(String contactEmail) {
        this.contactEmail = contactEmail;
    }

    public String getContactPhone() {
        return contactPhone;
    }

    public void setContactPhone(String contactPhone) {
        this.contactPhone = contactPhone;
    }

    @Override
    public String toString() {
        return "TenantCreateRequest{" +
                "tenantCode='" + tenantCode + '\'' +
                ", name='" + name + '\'' +
                ", description='" + description + '\'' +
                ", contactEmail='" + contactEmail + '\'' +
                ", contactPhone='" + contactPhone + '\'' +
                '}';
    }
} 