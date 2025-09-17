package com.hermesflow.permissionmanagement.dto;

import com.fasterxml.jackson.annotation.JsonFormat;
import lombok.AllArgsConstructor;
import lombok.Builder;
import lombok.Data;
import lombok.NoArgsConstructor;

import java.time.LocalDateTime;
import java.util.UUID;

/**
 * 角色权限DTO
 * 
 * @author HermesFlow
 * @version 1.0
 * @since 2024-01-01
 */
@Data
@Builder
@NoArgsConstructor
@AllArgsConstructor
public class RolePermissionDTO {
    
    /**
     * 角色权限ID
     */
    private UUID id;
    
    /**
     * 角色ID
     */
    private UUID roleId;
    
    /**
     * 权限代码
     */
    private String permissionCode;
    
    /**
     * 权限ID
     */
    private UUID permissionId;
    
    /**
     * 租户ID
     */
    private UUID tenantId;
    
    /**
     * 是否激活
     */
    private Boolean isActive;
    
    /**
     * 授权人ID
     */
    private UUID grantedBy;
    
    /**
     * 授权时间
     */
    @JsonFormat(pattern = "yyyy-MM-dd HH:mm:ss")
    private LocalDateTime grantedAt;
    
    /**
     * 过期时间
     */
    @JsonFormat(pattern = "yyyy-MM-dd HH:mm:ss")
    private LocalDateTime expiresAt;
    
    /**
     * 撤销时间
     */
    @JsonFormat(pattern = "yyyy-MM-dd HH:mm:ss")
    private LocalDateTime revokedAt;
    
    /**
     * 创建时间
     */
    @JsonFormat(pattern = "yyyy-MM-dd HH:mm:ss")
    private LocalDateTime createdAt;
    
    /**
     * 更新时间
     */
    @JsonFormat(pattern = "yyyy-MM-dd HH:mm:ss")
    private LocalDateTime updatedAt;
    
    /**
     * 角色信息
     */
    private RoleDTO role;
    
    /**
     * 权限信息
     */
    private PermissionDTO permission;
} 