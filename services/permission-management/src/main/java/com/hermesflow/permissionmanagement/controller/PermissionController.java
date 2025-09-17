package com.hermesflow.permissionmanagement.controller;

import com.hermesflow.permissionmanagement.dto.PermissionCreateDTO;
import com.hermesflow.permissionmanagement.dto.PermissionDTO;
import com.hermesflow.permissionmanagement.dto.PermissionUpdateDTO;
import com.hermesflow.permissionmanagement.service.PermissionService;
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
 * 权限管理控制器
 * 
 * @author HermesFlow
 * @version 1.0
 * @since 2024-01-01
 */
@Slf4j
@RestController
@RequestMapping("/api/v1/permissions")
@RequiredArgsConstructor
@Validated
@Tag(name = "权限管理", description = "权限管理相关接口")
public class PermissionController {

    private final PermissionService permissionService;

    @Operation(summary = "创建权限", description = "创建新的权限")
    @PostMapping
    public ResponseEntity<PermissionDTO> createPermission(
            @Valid @RequestBody PermissionCreateDTO createDTO) {
        log.info("Creating permission with code: {}", createDTO.getCode());
        PermissionDTO permission = permissionService.createPermission(createDTO);
        return ResponseEntity.status(HttpStatus.CREATED).body(permission);
    }

    @Operation(summary = "批量创建权限", description = "批量创建多个权限")
    @PostMapping("/batch")
    public ResponseEntity<List<PermissionDTO>> batchCreatePermissions(
            @Valid @RequestBody List<PermissionCreateDTO> createDTOs) {
        log.info("Batch creating {} permissions", createDTOs.size());
        List<PermissionDTO> permissions = permissionService.batchCreatePermissions(createDTOs);
        return ResponseEntity.status(HttpStatus.CREATED).body(permissions);
    }

    @Operation(summary = "更新权限", description = "根据ID更新权限信息")
    @PutMapping("/{id}")
    public ResponseEntity<PermissionDTO> updatePermission(
            @Parameter(description = "权限ID") @PathVariable UUID id,
            @Valid @RequestBody PermissionUpdateDTO updateDTO) {
        log.info("Updating permission: {}", id);
        PermissionDTO permission = permissionService.updatePermission(id, updateDTO);
        return ResponseEntity.ok(permission);
    }

    @Operation(summary = "删除权限", description = "根据ID删除权限")
    @DeleteMapping("/{id}")
    public ResponseEntity<Void> deletePermission(
            @Parameter(description = "权限ID") @PathVariable UUID id) {
        log.info("Deleting permission: {}", id);
        permissionService.deletePermission(id);
        return ResponseEntity.noContent().build();
    }

    @Operation(summary = "批量删除权限", description = "根据ID列表批量删除权限")
    @DeleteMapping("/batch")
    public ResponseEntity<Void> batchDeletePermissions(
            @RequestBody Set<UUID> ids) {
        log.info("Batch deleting {} permissions", ids.size());
        permissionService.batchDeletePermissions(ids);
        return ResponseEntity.noContent().build();
    }

    @Operation(summary = "获取权限详情", description = "根据ID获取权限详细信息")
    @GetMapping("/{id}")
    public ResponseEntity<PermissionDTO> getPermissionById(
            @Parameter(description = "权限ID") @PathVariable UUID id) {
        log.debug("Getting permission: {}", id);
        PermissionDTO permission = permissionService.getPermissionById(id);
        return ResponseEntity.ok(permission);
    }

    @Operation(summary = "根据代码获取权限", description = "根据权限代码获取权限信息")
    @GetMapping("/code/{code}")
    public ResponseEntity<PermissionDTO> getPermissionByCode(
            @Parameter(description = "权限代码") @PathVariable @NotBlank String code) {
        log.debug("Getting permission by code: {}", code);
        PermissionDTO permission = permissionService.getPermissionByCode(code);
        return ResponseEntity.ok(permission);
    }

    @Operation(summary = "分页查询权限", description = "分页获取权限列表")
    @GetMapping
    public ResponseEntity<Page<PermissionDTO>> getPermissions(
            @PageableDefault(size = 20) Pageable pageable) {
        log.debug("Getting permissions with pagination");
        Page<PermissionDTO> permissions = permissionService.getPermissions(pageable);
        return ResponseEntity.ok(permissions);
    }

    @Operation(summary = "根据类型查询权限", description = "根据权限类型分页查询权限")
    @GetMapping("/type/{type}")
    public ResponseEntity<Page<PermissionDTO>> getPermissionsByType(
            @Parameter(description = "权限类型") @PathVariable @NotBlank String type,
            @PageableDefault(size = 20) Pageable pageable) {
        log.debug("Getting permissions by type: {}", type);
        Page<PermissionDTO> permissions = permissionService.getPermissionsByType(type, pageable);
        return ResponseEntity.ok(permissions);
    }

    @Operation(summary = "根据资源查询权限", description = "根据资源类型分页查询权限")
    @GetMapping("/resource/{resource}")
    public ResponseEntity<Page<PermissionDTO>> getPermissionsByResource(
            @Parameter(description = "资源类型") @PathVariable @NotBlank String resource,
            @PageableDefault(size = 20) Pageable pageable) {
        log.debug("Getting permissions by resource: {}", resource);
        Page<PermissionDTO> permissions = permissionService.getPermissionsByResource(resource, pageable);
        return ResponseEntity.ok(permissions);
    }

    @Operation(summary = "搜索权限", description = "根据关键字模糊搜索权限")
    @GetMapping("/search")
    public ResponseEntity<Page<PermissionDTO>> searchPermissions(
            @Parameter(description = "搜索关键字") @RequestParam @NotBlank String keyword,
            @PageableDefault(size = 20) Pageable pageable) {
        log.debug("Searching permissions with keyword: {}", keyword);
        Page<PermissionDTO> permissions = permissionService.searchPermissions(keyword, pageable);
        return ResponseEntity.ok(permissions);
    }

    @Operation(summary = "获取系统权限", description = "获取所有系统权限")
    @GetMapping("/system")
    public ResponseEntity<List<PermissionDTO>> getSystemPermissions() {
        log.debug("Getting system permissions");
        List<PermissionDTO> permissions = permissionService.getSystemPermissions();
        return ResponseEntity.ok(permissions);
    }

    @Operation(summary = "激活权限", description = "激活指定权限")
    @PutMapping("/{id}/activate")
    public ResponseEntity<Void> activatePermission(
            @Parameter(description = "权限ID") @PathVariable UUID id) {
        log.info("Activating permission: {}", id);
        permissionService.activatePermission(id);
        return ResponseEntity.ok().build();
    }

    @Operation(summary = "停用权限", description = "停用指定权限")
    @PutMapping("/{id}/deactivate")
    public ResponseEntity<Void> deactivatePermission(
            @Parameter(description = "权限ID") @PathVariable UUID id) {
        log.info("Deactivating permission: {}", id);
        permissionService.deactivatePermission(id);
        return ResponseEntity.ok().build();
    }

    @Operation(summary = "初始化默认权限", description = "初始化系统默认权限")
    @PostMapping("/initialize")
    public ResponseEntity<List<PermissionDTO>> initializeDefaultPermissions() {
        log.info("Initializing default permissions");
        List<PermissionDTO> permissions = permissionService.initializeDefaultPermissions();
        return ResponseEntity.ok(permissions);
    }

    @Operation(summary = "验证权限代码", description = "验证权限代码是否有效")
    @GetMapping("/validate/{code}")
    public ResponseEntity<Boolean> validatePermissionCode(
            @Parameter(description = "权限代码") @PathVariable @NotBlank String code) {
        log.debug("Validating permission code: {}", code);
        boolean isValid = permissionService.validatePermissionCode(code);
        return ResponseEntity.ok(isValid);
    }

    @Operation(summary = "获取权限统计", description = "获取权限统计信息")
    @GetMapping("/statistics")
    public ResponseEntity<PermissionService.PermissionStatistics> getPermissionStatistics() {
        log.debug("Getting permission statistics");
        PermissionService.PermissionStatistics statistics = permissionService.getPermissionStatistics();
        return ResponseEntity.ok(statistics);
    }
} 