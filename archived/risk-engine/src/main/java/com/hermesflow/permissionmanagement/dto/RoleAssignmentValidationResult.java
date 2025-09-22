package com.hermesflow.permissionmanagement.dto;

import lombok.AllArgsConstructor;
import lombok.Builder;
import lombok.Data;
import lombok.NoArgsConstructor;

import java.util.List;

/**
 * 角色分配验证结果
 * 
 * @author HermesFlow
 * @version 1.0
 * @since 2024-01-01
 */
@Data
@Builder
@NoArgsConstructor
@AllArgsConstructor
public class RoleAssignmentValidationResult {
    
    /**
     * 验证是否通过
     */
    private boolean valid;
    
    /**
     * 错误信息列表
     */
    private List<String> errors;
    
    /**
     * 警告信息列表
     */
    private List<String> warnings;
    
    /**
     * 冲突的角色列表
     */
    private List<String> conflictingRoles;
    
    /**
     * 创建成功的验证结果
     */
    public static RoleAssignmentValidationResult success() {
        return RoleAssignmentValidationResult.builder()
                .valid(true)
                .build();
    }
    
    /**
     * 创建失败的验证结果
     */
    public static RoleAssignmentValidationResult failure(List<String> errors) {
        return RoleAssignmentValidationResult.builder()
                .valid(false)
                .errors(errors)
                .build();
    }
} 