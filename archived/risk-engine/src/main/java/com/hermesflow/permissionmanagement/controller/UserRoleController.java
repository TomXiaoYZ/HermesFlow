package com.hermesflow.permissionmanagement.controller;

import com.hermesflow.permissionmanagement.dto.UserRoleAssignDTO;
import com.hermesflow.permissionmanagement.dto.UserRoleDTO;
import com.hermesflow.permissionmanagement.service.UserRoleService;
import io.swagger.v3.oas.annotations.Operation;
import io.swagger.v3.oas.annotations.Parameter;
import io.swagger.v3.oas.annotations.tags.Tag;
import lombok.RequiredArgsConstructor;
import lombok.extern.slf4j.Slf4j;
import org.springframework.data.domain.Page;
import org.springframework.data.domain.Pageable;
import org.springframework.data.web.PageableDefault;
import org.springframework.format.annotation.DateTimeFormat;
import org.springframework.http.HttpStatus;
import org.springframework.http.ResponseEntity;
import org.springframework.validation.annotation.Validated;
import org.springframework.web.bind.annotation.*;

import jakarta.validation.Valid;
import jakarta.validation.constraints.NotBlank;
import jakarta.validation.constraints.NotNull;
import java.time.LocalDateTime;
import java.util.List;
import java.util.Set;
import java.util.UUID;

/**
 * 用户角色管理控制器
 * 
 * @author HermesFlow
 * @version 1.0
 * @since 2024-01-01
 */
@Slf4j
@RestController
@RequestMapping("/api/v1/user-roles")
@RequiredArgsConstructor
@Validated
@Tag(name = "用户角色管理", description = "用户角色分配和管理相关接口")
public class UserRoleController {

    private final UserRoleService userRoleService;

    @Operation(summary = "分配角色给用户", description = "为用户分配指定角色")
    @PostMapping
    public ResponseEntity<UserRoleDTO> assignRole(
            @Valid @RequestBody UserRoleAssignDTO assignDTO) {
        log.info("Assigning role {} to user {} in tenant {}", 
                assignDTO.getRoleId(), assignDTO.getUserId(), assignDTO.getTenantId());
        UserRoleDTO userRole = userRoleService.assignRole(assignDTO);
        return ResponseEntity.status(HttpStatus.CREATED).body(userRole);
    }

    @Operation(summary = "批量分配角色", description = "为用户批量分配多个角色")
    @PostMapping("/batch")
    public ResponseEntity<List<UserRoleDTO>> batchAssignRoles(
            @Parameter(description = "用户ID") @RequestParam @NotNull UUID userId,
            @Parameter(description = "租户ID") @RequestParam @NotNull UUID tenantId,
            @Parameter(description = "角色ID列表") @RequestBody Set<UUID> roleIds,
            @Parameter(description = "分配者ID") @RequestParam @NotNull UUID assignedBy,
            @Parameter(description = "过期时间") @RequestParam(required = false) 
            @DateTimeFormat(iso = DateTimeFormat.ISO.DATE_TIME) LocalDateTime expiresAt) {
        log.info("Batch assigning {} roles to user {} in tenant {}", roleIds.size(), userId, tenantId);
        List<UserRoleDTO> userRoles = userRoleService.batchAssignRoles(userId, tenantId, roleIds, assignedBy, expiresAt);
        return ResponseEntity.status(HttpStatus.CREATED).body(userRoles);
    }

    @Operation(summary = "撤销用户角色", description = "撤销用户的指定角色")
    @DeleteMapping("/{userRoleId}")
    public ResponseEntity<Void> revokeRole(
            @Parameter(description = "用户角色ID") @PathVariable UUID userRoleId) {
        log.info("Revoking user role: {}", userRoleId);
        userRoleService.revokeRole(userRoleId);
        return ResponseEntity.noContent().build();
    }

    @Operation(summary = "批量撤销角色", description = "批量撤销用户的多个角色")
    @DeleteMapping("/batch")
    public ResponseEntity<Void> batchRevokeRoles(
            @Parameter(description = "用户ID") @RequestParam @NotNull UUID userId,
            @Parameter(description = "租户ID") @RequestParam @NotNull UUID tenantId,
            @Parameter(description = "角色ID列表") @RequestBody Set<UUID> roleIds) {
        log.info("Batch revoking {} roles from user {} in tenant {}", roleIds.size(), userId, tenantId);
        userRoleService.batchRevokeRoles(userId, tenantId, roleIds);
        return ResponseEntity.noContent().build();
    }

    @Operation(summary = "更新角色过期时间", description = "更新用户角色的过期时间")
    @PutMapping("/{userRoleId}/expiration")
    public ResponseEntity<Void> updateRoleExpiration(
            @Parameter(description = "用户角色ID") @PathVariable UUID userRoleId,
            @Parameter(description = "新的过期时间") @RequestParam 
            @DateTimeFormat(iso = DateTimeFormat.ISO.DATE_TIME) LocalDateTime expiresAt) {
        log.info("Updating expiration for user role {} to {}", userRoleId, expiresAt);
        userRoleService.updateRoleExpiration(userRoleId, expiresAt);
        return ResponseEntity.ok().build();
    }

    @Operation(summary = "激活用户角色", description = "激活指定的用户角色")
    @PutMapping("/{userRoleId}/activate")
    public ResponseEntity<Void> activateUserRole(
            @Parameter(description = "用户角色ID") @PathVariable UUID userRoleId) {
        log.info("Activating user role: {}", userRoleId);
        userRoleService.activateUserRole(userRoleId);
        return ResponseEntity.ok().build();
    }

    @Operation(summary = "停用用户角色", description = "停用指定的用户角色")
    @PutMapping("/{userRoleId}/deactivate")
    public ResponseEntity<Void> deactivateUserRole(
            @Parameter(description = "用户角色ID") @PathVariable UUID userRoleId) {
        log.info("Deactivating user role: {}", userRoleId);
        userRoleService.deactivateUserRole(userRoleId);
        return ResponseEntity.ok().build();
    }

    @Operation(summary = "获取用户角色", description = "获取用户在指定租户下的所有角色")
    @GetMapping("/user/{userId}")
    public ResponseEntity<List<UserRoleDTO>> getUserRoles(
            @Parameter(description = "用户ID") @PathVariable UUID userId,
            @Parameter(description = "租户ID") @RequestParam @NotNull UUID tenantId) {
        log.debug("Getting roles for user {} in tenant {}", userId, tenantId);
        List<UserRoleDTO> userRoles = userRoleService.getUserRoles(userId, tenantId);
        return ResponseEntity.ok(userRoles);
    }

    @Operation(summary = "获取用户活跃角色", description = "获取用户在指定租户下的活跃角色")
    @GetMapping("/user/{userId}/active")
    public ResponseEntity<List<UserRoleDTO>> getActiveUserRoles(
            @Parameter(description = "用户ID") @PathVariable UUID userId,
            @Parameter(description = "租户ID") @RequestParam @NotNull UUID tenantId) {
        log.debug("Getting active roles for user {} in tenant {}", userId, tenantId);
        List<UserRoleDTO> userRoles = userRoleService.getActiveUserRoles(userId, tenantId);
        return ResponseEntity.ok(userRoles);
    }

    @Operation(summary = "获取用户有效角色", description = "获取用户在指定租户下的有效角色（活跃且未过期）")
    @GetMapping("/user/{userId}/valid")
    public ResponseEntity<List<UserRoleDTO>> getValidUserRoles(
            @Parameter(description = "用户ID") @PathVariable UUID userId,
            @Parameter(description = "租户ID") @RequestParam @NotNull UUID tenantId) {
        log.debug("Getting valid roles for user {} in tenant {}", userId, tenantId);
        List<UserRoleDTO> userRoles = userRoleService.getValidUserRoles(userId, tenantId);
        return ResponseEntity.ok(userRoles);
    }

    @Operation(summary = "分页查询用户角色", description = "分页查询用户在指定租户下的角色")
    @GetMapping("/user/{userId}/paginated")
    public ResponseEntity<Page<UserRoleDTO>> getUserRoles(
            @Parameter(description = "用户ID") @PathVariable UUID userId,
            @Parameter(description = "租户ID") @RequestParam @NotNull UUID tenantId,
            @PageableDefault(size = 20) Pageable pageable) {
        log.debug("Getting paginated roles for user {} in tenant {}", userId, tenantId);
        Page<UserRoleDTO> userRoles = userRoleService.getUserRoles(userId, tenantId, pageable);
        return ResponseEntity.ok(userRoles);
    }

    @Operation(summary = "检查用户权限", description = "检查用户是否拥有指定权限")
    @GetMapping("/user/{userId}/permissions/{permissionCode}/check")
    public ResponseEntity<Boolean> checkUserPermission(
            @Parameter(description = "用户ID") @PathVariable UUID userId,
            @Parameter(description = "租户ID") @RequestParam @NotNull UUID tenantId,
            @Parameter(description = "权限代码") @PathVariable @NotBlank String permissionCode) {
        log.debug("Checking permission {} for user {} in tenant {}", permissionCode, userId, tenantId);
        boolean hasPermission = userRoleService.checkUserPermission(userId, tenantId, permissionCode);
        return ResponseEntity.ok(hasPermission);
    }

    @Operation(summary = "检查用户角色", description = "检查用户是否拥有指定角色")
    @GetMapping("/user/{userId}/roles/{roleCode}/check")
    public ResponseEntity<Boolean> checkUserRole(
            @Parameter(description = "用户ID") @PathVariable UUID userId,
            @Parameter(description = "租户ID") @RequestParam @NotNull UUID tenantId,
            @Parameter(description = "角色代码") @PathVariable @NotBlank String roleCode) {
        log.debug("Checking role {} for user {} in tenant {}", roleCode, userId, tenantId);
        boolean hasRole = userRoleService.checkUserRole(userId, tenantId, roleCode);
        return ResponseEntity.ok(hasRole);
    }

    @Operation(summary = "获取用户权限代码", description = "获取用户在指定租户下的所有权限代码")
    @GetMapping("/user/{userId}/permissions")
    public ResponseEntity<Set<String>> getUserPermissions(
            @Parameter(description = "用户ID") @PathVariable UUID userId,
            @Parameter(description = "租户ID") @RequestParam @NotNull UUID tenantId) {
        log.debug("Getting permissions for user {} in tenant {}", userId, tenantId);
        Set<String> permissions = userRoleService.getUserPermissions(userId, tenantId);
        return ResponseEntity.ok(permissions);
    }

    @Operation(summary = "获取角色用户", description = "获取拥有指定角色的所有用户")
    @GetMapping("/role/{roleId}/users")
    public ResponseEntity<Page<UserRoleDTO>> getRoleUsers(
            @Parameter(description = "角色ID") @PathVariable UUID roleId,
            @PageableDefault(size = 20) Pageable pageable) {
        log.debug("Getting users for role {}", roleId);
        Page<UserRoleDTO> userRoles = userRoleService.getRoleUsers(roleId, pageable);
        return ResponseEntity.ok(userRoles);
    }

    @Operation(summary = "获取租户角色统计", description = "获取指定租户下的角色统计信息")
    @GetMapping("/statistics")
    public ResponseEntity<UserRoleService.UserRoleStatistics> getUserRoleStatistics(
            @Parameter(description = "租户ID") @RequestParam @NotNull UUID tenantId) {
        log.debug("Getting role statistics for tenant {}", tenantId);
        UserRoleService.UserRoleStatistics statistics = userRoleService.getUserRoleStatistics(tenantId);
        return ResponseEntity.ok(statistics);
    }
} 