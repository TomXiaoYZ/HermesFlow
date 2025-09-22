package com.hermesflow.permissionmanagement.controller;

import com.hermesflow.permissionmanagement.dto.RoleCreateDTO;
import com.hermesflow.permissionmanagement.dto.RoleDTO;
import com.hermesflow.permissionmanagement.dto.RoleUpdateDTO;
import com.hermesflow.permissionmanagement.dto.RoleUsageDTO;
import com.hermesflow.permissionmanagement.service.RoleService;
import io.swagger.v3.oas.annotations.Operation;
import io.swagger.v3.oas.annotations.Parameter;
import io.swagger.v3.oas.annotations.tags.Tag;
import lombok.RequiredArgsConstructor;
import lombok.extern.slf4j.Slf4j;
import org.springframework.data.domain.Page;
import org.springframework.data.domain.Pageable;
import org.springframework.data.web.PageableDefault;
import org.springframework.http.HttpStatus;
import org.springframework.http.ResponseEntity;
import org.springframework.validation.annotation.Validated;
import org.springframework.web.bind.annotation.*;

import jakarta.validation.Valid;
import jakarta.validation.constraints.NotBlank;
import jakarta.validation.constraints.NotNull;
import java.util.List;
import java.util.Set;
import java.util.UUID;

/**
 * 角色管理控制器
 * 
 * @author HermesFlow
 * @version 1.0
 * @since 2024-01-01
 */
@Slf4j
@RestController
@RequestMapping("/api/v1/roles")
@RequiredArgsConstructor
@Validated
@Tag(name = "角色管理", description = "角色管理相关接口")
public class RoleController {

    private final RoleService roleService;

    @Operation(summary = "创建角色", description = "创建新的角色")
    @PostMapping
    public ResponseEntity<RoleDTO> createRole(
            @Valid @RequestBody RoleCreateDTO createDTO) {
        log.info("Creating role with code: {}", createDTO.getCode());
        RoleDTO role = roleService.createRole(createDTO);
        return ResponseEntity.status(HttpStatus.CREATED).body(role);
    }

    @Operation(summary = "批量创建角色", description = "批量创建多个角色")
    @PostMapping("/batch")
    public ResponseEntity<List<RoleDTO>> batchCreateRoles(
            @Valid @RequestBody List<RoleCreateDTO> createDTOs) {
        log.info("Batch creating {} roles", createDTOs.size());
        List<RoleDTO> roles = roleService.batchCreateRoles(createDTOs);
        return ResponseEntity.status(HttpStatus.CREATED).body(roles);
    }

    @Operation(summary = "更新角色", description = "根据ID更新角色信息")
    @PutMapping("/{id}")
    public ResponseEntity<RoleDTO> updateRole(
            @Parameter(description = "角色ID") @PathVariable UUID id,
            @Valid @RequestBody RoleUpdateDTO updateDTO) {
        log.info("Updating role: {}", id);
        RoleDTO role = roleService.updateRole(id, updateDTO);
        return ResponseEntity.ok(role);
    }

    @Operation(summary = "删除角色", description = "根据ID删除角色")
    @DeleteMapping("/{id}")
    public ResponseEntity<Void> deleteRole(
            @Parameter(description = "角色ID") @PathVariable UUID id) {
        log.info("Deleting role: {}", id);
        roleService.deleteRole(id);
        return ResponseEntity.noContent().build();
    }

    @Operation(summary = "获取角色详情", description = "根据ID获取角色详细信息")
    @GetMapping("/{id}")
    public ResponseEntity<RoleDTO> getRoleById(
            @Parameter(description = "角色ID") @PathVariable UUID id) {
        log.debug("Getting role: {}", id);
        RoleDTO role = roleService.getRoleById(id);
        return ResponseEntity.ok(role);
    }

    @Operation(summary = "根据代码获取角色", description = "根据角色代码获取角色信息")
    @GetMapping("/code/{code}")
    public ResponseEntity<RoleDTO> getRoleByCode(
            @Parameter(description = "角色代码") @PathVariable @NotBlank String code,
            @Parameter(description = "租户ID") @RequestParam @NotNull UUID tenantId) {
        log.debug("Getting role by code: {} for tenant: {}", code, tenantId);
        RoleDTO role = roleService.getRoleByCode(code, tenantId);
        return ResponseEntity.ok(role);
    }

    @Operation(summary = "根据租户查询角色", description = "根据租户ID分页查询角色")
    @GetMapping("/tenant/{tenantId}")
    public ResponseEntity<Page<RoleDTO>> getRolesByTenantId(
            @Parameter(description = "租户ID") @PathVariable UUID tenantId,
            @PageableDefault(size = 20) Pageable pageable) {
        log.debug("Getting roles for tenant: {}", tenantId);
        Page<RoleDTO> roles = roleService.getRolesByTenantId(tenantId, pageable);
        return ResponseEntity.ok(roles);
    }

    @Operation(summary = "根据类型查询角色", description = "根据角色类型分页查询角色")
    @GetMapping("/type/{type}")
    public ResponseEntity<Page<RoleDTO>> getRolesByType(
            @Parameter(description = "角色类型") @PathVariable @NotBlank String type,
            @Parameter(description = "租户ID") @RequestParam @NotNull UUID tenantId,
            @PageableDefault(size = 20) Pageable pageable) {
        log.debug("Getting roles by type: {} for tenant: {}", type, tenantId);
        Page<RoleDTO> roles = roleService.getRolesByType(type, tenantId, pageable);
        return ResponseEntity.ok(roles);
    }

    @Operation(summary = "获取活跃角色", description = "根据租户ID获取活跃角色")
    @GetMapping("/tenant/{tenantId}/active")
    public ResponseEntity<List<RoleDTO>> getActiveRolesByTenantId(
            @Parameter(description = "租户ID") @PathVariable UUID tenantId) {
        log.debug("Getting active roles for tenant: {}", tenantId);
        List<RoleDTO> roles = roleService.getActiveRolesByTenantId(tenantId);
        return ResponseEntity.ok(roles);
    }

    @Operation(summary = "搜索角色", description = "根据关键字模糊搜索角色")
    @GetMapping("/search")
    public ResponseEntity<Page<RoleDTO>> searchRoles(
            @Parameter(description = "搜索关键字") @RequestParam @NotBlank String keyword,
            @Parameter(description = "租户ID") @RequestParam @NotNull UUID tenantId,
            @PageableDefault(size = 20) Pageable pageable) {
        log.debug("Searching roles with keyword: {} for tenant: {}", keyword, tenantId);
        Page<RoleDTO> roles = roleService.searchRoles(keyword, tenantId, pageable);
        return ResponseEntity.ok(roles);
    }

    @Operation(summary = "获取子角色", description = "获取指定角色的所有子角色")
    @GetMapping("/{id}/children")
    public ResponseEntity<List<RoleDTO>> getChildRoles(
            @Parameter(description = "父角色ID") @PathVariable UUID id) {
        log.debug("Getting child roles for: {}", id);
        List<RoleDTO> roles = roleService.getChildRoles(id);
        return ResponseEntity.ok(roles);
    }

    @Operation(summary = "激活角色", description = "激活指定角色")
    @PutMapping("/{id}/activate")
    public ResponseEntity<Void> activateRole(
            @Parameter(description = "角色ID") @PathVariable UUID id) {
        log.info("Activating role: {}", id);
        roleService.activateRole(id);
        return ResponseEntity.ok().build();
    }

    @Operation(summary = "停用角色", description = "停用指定角色")
    @PutMapping("/{id}/deactivate")
    public ResponseEntity<Void> deactivateRole(
            @Parameter(description = "角色ID") @PathVariable UUID id) {
        log.info("Deactivating role: {}", id);
        roleService.deactivateRole(id);
        return ResponseEntity.ok().build();
    }

    @Operation(summary = "分配权限给角色", description = "为角色分配权限")
    @PostMapping("/{roleId}/permissions/{permissionId}")
    public ResponseEntity<Void> assignPermissionToRole(
            @Parameter(description = "角色ID") @PathVariable UUID roleId,
            @Parameter(description = "权限ID") @PathVariable UUID permissionId,
            @Parameter(description = "授权者ID") @RequestParam @NotNull UUID grantedBy) {
        log.info("Assigning permission {} to role {}", permissionId, roleId);
        roleService.assignPermissionToRole(roleId, permissionId, grantedBy);
        return ResponseEntity.ok().build();
    }

    @Operation(summary = "移除角色权限", description = "移除角色的指定权限")
    @DeleteMapping("/{roleId}/permissions/{permissionId}")
    public ResponseEntity<Void> removePermissionFromRole(
            @Parameter(description = "角色ID") @PathVariable UUID roleId,
            @Parameter(description = "权限ID") @PathVariable UUID permissionId) {
        log.info("Removing permission {} from role {}", permissionId, roleId);
        roleService.removePermissionFromRole(roleId, permissionId);
        return ResponseEntity.ok().build();
    }

    @Operation(summary = "检查角色权限", description = "检查角色是否拥有指定权限")
    @GetMapping("/{roleId}/permissions/{permissionCode}/check")
    public ResponseEntity<Boolean> checkRolePermission(
            @Parameter(description = "角色ID") @PathVariable UUID roleId,
            @Parameter(description = "权限代码") @PathVariable @NotBlank String permissionCode) {
        log.debug("Checking permission {} for role {}", permissionCode, roleId);
        boolean hasPermission = roleService.checkRolePermission(roleId, permissionCode);
        return ResponseEntity.ok(hasPermission);
    }

    @Operation(summary = "验证角色代码", description = "验证角色代码是否有效")
    @GetMapping("/validate/{code}")
    public ResponseEntity<Boolean> validateRoleCode(
            @Parameter(description = "角色代码") @PathVariable @NotBlank String code,
            @Parameter(description = "租户ID") @RequestParam @NotNull UUID tenantId) {
        log.debug("Validating role code: {} for tenant: {}", code, tenantId);
        boolean isValid = roleService.validateRoleCode(code, tenantId);
        return ResponseEntity.ok(isValid);
    }

    @Operation(summary = "获取角色使用情况", description = "获取角色的使用统计信息")
    @GetMapping("/{id}/usage")
    public ResponseEntity<RoleUsageDTO> getRoleUsage(
            @Parameter(description = "角色ID") @PathVariable UUID id) {
        log.debug("Getting usage for role: {}", id);
        RoleUsageDTO usage = roleService.getRoleUsage(id);
        return ResponseEntity.ok(usage);
    }

    @Operation(summary = "获取角色统计", description = "获取角色统计信息")
    @GetMapping("/statistics")
    public ResponseEntity<RoleService.RoleStatistics> getRoleStatistics(
            @Parameter(description = "租户ID") @RequestParam @NotNull UUID tenantId) {
        log.debug("Getting role statistics for tenant: {}", tenantId);
        RoleService.RoleStatistics statistics = roleService.getRoleStatistics(tenantId);
        return ResponseEntity.ok(statistics);
    }
} 