package com.hermesflow.permissionmanagement.dto;

import com.fasterxml.jackson.annotation.JsonFormat;
import lombok.AllArgsConstructor;
import lombok.Builder;
import lombok.Data;
import lombok.NoArgsConstructor;

import java.time.LocalDateTime;
import java.util.UUID;

/**
 * 用户角色历史DTO
 * 
 * @author HermesFlow
 * @version 1.0
 * @since 2024-01-01
 */
@Data
@Builder
@NoArgsConstructor
@AllArgsConstructor
public class UserRoleHistoryDTO {
    
    /**
     * 历史记录ID
     */
    private UUID id;
    
    /**
     * 用户ID
     */
    private UUID userId;
    
    /**
     * 角色ID
     */
    private UUID roleId;
    
    /**
     * 租户ID
     */
    private UUID tenantId;
    
    /**
     * 操作类型（ASSIGN, REVOKE, ACTIVATE, DEACTIVATE）
     */
    private String operationType;
    
    /**
     * 操作人ID
     */
    private UUID operatedBy;
    
    /**
     * 操作时间
     */
    @JsonFormat(pattern = "yyyy-MM-dd HH:mm:ss")
    private LocalDateTime operatedAt;
    
    /**
     * 操作原因
     */
    private String reason;
    
    /**
     * 角色信息
     */
    private RoleDTO role;
} 