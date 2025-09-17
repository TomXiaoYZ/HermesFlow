package com.hermesflow.usermanagement.controller;

import com.hermesflow.usermanagement.dto.TenantCreateRequest;
import com.hermesflow.usermanagement.entity.Tenant;
import com.hermesflow.usermanagement.service.TenantService;
import jakarta.validation.Valid;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.data.domain.Page;
import org.springframework.data.domain.PageRequest;
import org.springframework.data.domain.Pageable;
import org.springframework.data.domain.Sort;
import org.springframework.http.HttpStatus;
import org.springframework.http.ResponseEntity;
import org.springframework.web.bind.annotation.*;

import java.util.HashMap;
import java.util.List;
import java.util.Map;
import java.util.Optional;

/**
 * 租户管理控制器
 * 提供租户相关的REST API接口
 */
@RestController
@RequestMapping("/api/tenants")
public class TenantController {

    private static final Logger logger = LoggerFactory.getLogger(TenantController.class);

    private final TenantService tenantService;

    @Autowired
    public TenantController(TenantService tenantService) {
        this.tenantService = tenantService;
    }

    /**
     * 创建租户
     */
    @PostMapping
    public ResponseEntity<Map<String, Object>> createTenant(@Valid @RequestBody TenantCreateRequest request) {
        try {
            logger.info("创建租户请求: {}", request.getTenantCode());
            
            Tenant tenant = tenantService.createTenant(request);
            
            Map<String, Object> response = new HashMap<>();
            response.put("success", true);
            response.put("message", "租户创建成功");
            response.put("data", tenant);
            
            return ResponseEntity.status(HttpStatus.CREATED).body(response);
        } catch (IllegalArgumentException e) {
            logger.warn("租户创建失败: {}", e.getMessage());
            
            Map<String, Object> response = new HashMap<>();
            response.put("success", false);
            response.put("message", e.getMessage());
            
            return ResponseEntity.badRequest().body(response);
        } catch (Exception e) {
            logger.error("租户创建异常", e);
            
            Map<String, Object> response = new HashMap<>();
            response.put("success", false);
            response.put("message", "服务器内部错误");
            
            return ResponseEntity.status(HttpStatus.INTERNAL_SERVER_ERROR).body(response);
        }
    }

    /**
     * 根据租户代码获取租户信息
     */
    @GetMapping("/{tenantCode}")
    public ResponseEntity<Map<String, Object>> getTenant(@PathVariable String tenantCode) {
        try {
            logger.info("获取租户信息: {}", tenantCode);
            
            Optional<Tenant> tenant = tenantService.getTenantByCode(tenantCode);
            
            Map<String, Object> response = new HashMap<>();
            if (tenant.isPresent()) {
                response.put("success", true);
                response.put("data", tenant.get());
                return ResponseEntity.ok(response);
            } else {
                response.put("success", false);
                response.put("message", "租户不存在");
                return ResponseEntity.notFound().build();
            }
        } catch (Exception e) {
            logger.error("获取租户信息异常", e);
            
            Map<String, Object> response = new HashMap<>();
            response.put("success", false);
            response.put("message", "服务器内部错误");
            
            return ResponseEntity.status(HttpStatus.INTERNAL_SERVER_ERROR).body(response);
        }
    }

    /**
     * 更新租户信息
     */
    @PutMapping("/{tenantCode}")
    public ResponseEntity<Map<String, Object>> updateTenant(
            @PathVariable String tenantCode,
            @Valid @RequestBody TenantCreateRequest request) {
        try {
            logger.info("更新租户信息: {}", tenantCode);
            
            Tenant tenant = tenantService.updateTenant(tenantCode, request);
            
            Map<String, Object> response = new HashMap<>();
            response.put("success", true);
            response.put("message", "租户更新成功");
            response.put("data", tenant);
            
            return ResponseEntity.ok(response);
        } catch (IllegalArgumentException e) {
            logger.warn("租户更新失败: {}", e.getMessage());
            
            Map<String, Object> response = new HashMap<>();
            response.put("success", false);
            response.put("message", e.getMessage());
            
            return ResponseEntity.badRequest().body(response);
        } catch (Exception e) {
            logger.error("租户更新异常", e);
            
            Map<String, Object> response = new HashMap<>();
            response.put("success", false);
            response.put("message", "服务器内部错误");
            
            return ResponseEntity.status(HttpStatus.INTERNAL_SERVER_ERROR).body(response);
        }
    }

    /**
     * 暂停租户
     */
    @PostMapping("/{tenantCode}/suspend")
    public ResponseEntity<Map<String, Object>> suspendTenant(@PathVariable String tenantCode) {
        try {
            logger.info("暂停租户: {}", tenantCode);
            
            Tenant tenant = tenantService.suspendTenant(tenantCode);
            
            Map<String, Object> response = new HashMap<>();
            response.put("success", true);
            response.put("message", "租户暂停成功");
            response.put("data", tenant);
            
            return ResponseEntity.ok(response);
        } catch (IllegalArgumentException | IllegalStateException e) {
            logger.warn("租户暂停失败: {}", e.getMessage());
            
            Map<String, Object> response = new HashMap<>();
            response.put("success", false);
            response.put("message", e.getMessage());
            
            return ResponseEntity.badRequest().body(response);
        } catch (Exception e) {
            logger.error("租户暂停异常", e);
            
            Map<String, Object> response = new HashMap<>();
            response.put("success", false);
            response.put("message", "服务器内部错误");
            
            return ResponseEntity.status(HttpStatus.INTERNAL_SERVER_ERROR).body(response);
        }
    }

    /**
     * 激活租户
     */
    @PostMapping("/{tenantCode}/activate")
    public ResponseEntity<Map<String, Object>> activateTenant(@PathVariable String tenantCode) {
        try {
            logger.info("激活租户: {}", tenantCode);
            
            Tenant tenant = tenantService.activateTenant(tenantCode);
            
            Map<String, Object> response = new HashMap<>();
            response.put("success", true);
            response.put("message", "租户激活成功");
            response.put("data", tenant);
            
            return ResponseEntity.ok(response);
        } catch (IllegalArgumentException | IllegalStateException e) {
            logger.warn("租户激活失败: {}", e.getMessage());
            
            Map<String, Object> response = new HashMap<>();
            response.put("success", false);
            response.put("message", e.getMessage());
            
            return ResponseEntity.badRequest().body(response);
        } catch (Exception e) {
            logger.error("租户激活异常", e);
            
            Map<String, Object> response = new HashMap<>();
            response.put("success", false);
            response.put("message", "服务器内部错误");
            
            return ResponseEntity.status(HttpStatus.INTERNAL_SERVER_ERROR).body(response);
        }
    }

    /**
     * 获取租户列表（分页）
     */
    @GetMapping
    public ResponseEntity<Map<String, Object>> getTenants(
            @RequestParam(defaultValue = "0") int page,
            @RequestParam(defaultValue = "20") int size,
            @RequestParam(defaultValue = "createdAt") String sortBy,
            @RequestParam(defaultValue = "desc") String sortDir,
            @RequestParam(required = false) String status) {
        try {
            logger.info("获取租户列表 - 页码: {}, 大小: {}, 状态: {}", page, size, status);
            
            Sort sort = Sort.by(sortDir.equalsIgnoreCase("desc") ? 
                Sort.Direction.DESC : Sort.Direction.ASC, sortBy);
            Pageable pageable = PageRequest.of(page, size, sort);
            
            Page<Tenant> tenants;
            if (status != null && !status.isEmpty()) {
                Tenant.TenantStatus tenantStatus = Tenant.TenantStatus.valueOf(status.toUpperCase());
                tenants = tenantService.getTenantsByStatus(tenantStatus, pageable);
            } else {
                tenants = tenantService.getAllTenants(pageable);
            }
            
            Map<String, Object> response = new HashMap<>();
            response.put("success", true);
            response.put("data", tenants.getContent());
            response.put("pagination", Map.of(
                "page", tenants.getNumber(),
                "size", tenants.getSize(),
                "totalElements", tenants.getTotalElements(),
                "totalPages", tenants.getTotalPages(),
                "first", tenants.isFirst(),
                "last", tenants.isLast()
            ));
            
            return ResponseEntity.ok(response);
        } catch (Exception e) {
            logger.error("获取租户列表异常", e);
            
            Map<String, Object> response = new HashMap<>();
            response.put("success", false);
            response.put("message", "服务器内部错误");
            
            return ResponseEntity.status(HttpStatus.INTERNAL_SERVER_ERROR).body(response);
        }
    }

    /**
     * 搜索租户
     */
    @GetMapping("/search")
    public ResponseEntity<Map<String, Object>> searchTenants(
            @RequestParam String name,
            @RequestParam(defaultValue = "0") int page,
            @RequestParam(defaultValue = "20") int size) {
        try {
            logger.info("搜索租户: {}", name);
            
            Pageable pageable = PageRequest.of(page, size, Sort.by("createdAt").descending());
            Page<Tenant> tenants = tenantService.searchTenantsByName(name, pageable);
            
            Map<String, Object> response = new HashMap<>();
            response.put("success", true);
            response.put("data", tenants.getContent());
            response.put("pagination", Map.of(
                "page", tenants.getNumber(),
                "size", tenants.getSize(),
                "totalElements", tenants.getTotalElements(),
                "totalPages", tenants.getTotalPages()
            ));
            
            return ResponseEntity.ok(response);
        } catch (Exception e) {
            logger.error("搜索租户异常", e);
            
            Map<String, Object> response = new HashMap<>();
            response.put("success", false);
            response.put("message", "服务器内部错误");
            
            return ResponseEntity.status(HttpStatus.INTERNAL_SERVER_ERROR).body(response);
        }
    }

    /**
     * 获取活跃租户列表
     */
    @GetMapping("/active")
    public ResponseEntity<Map<String, Object>> getActiveTenants() {
        try {
            logger.info("获取活跃租户列表");
            
            List<Tenant> tenants = tenantService.getActiveTenants();
            
            Map<String, Object> response = new HashMap<>();
            response.put("success", true);
            response.put("data", tenants);
            
            return ResponseEntity.ok(response);
        } catch (Exception e) {
            logger.error("获取活跃租户列表异常", e);
            
            Map<String, Object> response = new HashMap<>();
            response.put("success", false);
            response.put("message", "服务器内部错误");
            
            return ResponseEntity.status(HttpStatus.INTERNAL_SERVER_ERROR).body(response);
        }
    }
} 