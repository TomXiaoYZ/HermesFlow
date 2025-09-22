package com.hermesflow.usermanagement.controller;

import com.hermesflow.usermanagement.entity.TenantConfig;
import com.hermesflow.usermanagement.service.TenantConfigService;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.http.HttpStatus;
import org.springframework.http.ResponseEntity;
import org.springframework.web.bind.annotation.*;

import java.util.HashMap;
import java.util.List;
import java.util.Map;
import java.util.Optional;

/**
 * 租户配置管理控制器
 * 提供租户配置相关的REST API接口
 */
@RestController
@RequestMapping("/api/tenant-configs")
public class TenantConfigController {

    private static final Logger logger = LoggerFactory.getLogger(TenantConfigController.class);

    private final TenantConfigService tenantConfigService;

    @Autowired
    public TenantConfigController(TenantConfigService tenantConfigService) {
        this.tenantConfigService = tenantConfigService;
    }

    /**
     * 设置租户配置
     */
    @PostMapping("/{tenantCode}")
    public ResponseEntity<Map<String, Object>> setConfig(
            @PathVariable String tenantCode,
            @RequestParam String configKey,
            @RequestParam String configValue) {
        try {
            logger.info("设置租户配置: {} - {} = {}", tenantCode, configKey, configValue);
            
            TenantConfig config = tenantConfigService.setConfig(tenantCode, configKey, configValue);
            
            Map<String, Object> response = new HashMap<>();
            response.put("success", true);
            response.put("message", "配置设置成功");
            response.put("data", config);
            
            return ResponseEntity.ok(response);
        } catch (IllegalArgumentException e) {
            logger.warn("配置设置失败: {}", e.getMessage());
            
            Map<String, Object> response = new HashMap<>();
            response.put("success", false);
            response.put("message", e.getMessage());
            
            return ResponseEntity.badRequest().body(response);
        } catch (Exception e) {
            logger.error("配置设置异常", e);
            
            Map<String, Object> response = new HashMap<>();
            response.put("success", false);
            response.put("message", "服务器内部错误");
            
            return ResponseEntity.status(HttpStatus.INTERNAL_SERVER_ERROR).body(response);
        }
    }

    /**
     * 批量设置租户配置
     */
    @PostMapping("/{tenantCode}/batch")
    public ResponseEntity<Map<String, Object>> setConfigs(
            @PathVariable String tenantCode,
            @RequestBody Map<String, String> configs) {
        try {
            logger.info("批量设置租户配置: {} - {} 项", tenantCode, configs.size());
            
            List<TenantConfig> savedConfigs = tenantConfigService.setConfigs(tenantCode, configs);
            
            Map<String, Object> response = new HashMap<>();
            response.put("success", true);
            response.put("message", "批量配置设置成功");
            response.put("data", savedConfigs);
            
            return ResponseEntity.ok(response);
        } catch (IllegalArgumentException e) {
            logger.warn("批量配置设置失败: {}", e.getMessage());
            
            Map<String, Object> response = new HashMap<>();
            response.put("success", false);
            response.put("message", e.getMessage());
            
            return ResponseEntity.badRequest().body(response);
        } catch (Exception e) {
            logger.error("批量配置设置异常", e);
            
            Map<String, Object> response = new HashMap<>();
            response.put("success", false);
            response.put("message", "服务器内部错误");
            
            return ResponseEntity.status(HttpStatus.INTERNAL_SERVER_ERROR).body(response);
        }
    }

    /**
     * 获取租户配置
     */
    @GetMapping("/{tenantCode}/{configKey}")
    public ResponseEntity<Map<String, Object>> getConfig(
            @PathVariable String tenantCode,
            @PathVariable String configKey) {
        try {
            logger.info("获取租户配置: {} - {}", tenantCode, configKey);
            
            Optional<String> configValue = tenantConfigService.getConfig(tenantCode, configKey);
            
            Map<String, Object> response = new HashMap<>();
            if (configValue.isPresent()) {
                response.put("success", true);
                response.put("data", Map.of(
                    "tenantCode", tenantCode,
                    "configKey", configKey,
                    "configValue", configValue.get()
                ));
                return ResponseEntity.ok(response);
            } else {
                response.put("success", false);
                response.put("message", "配置不存在");
                return ResponseEntity.notFound().build();
            }
        } catch (Exception e) {
            logger.error("获取配置异常", e);
            
            Map<String, Object> response = new HashMap<>();
            response.put("success", false);
            response.put("message", "服务器内部错误");
            
            return ResponseEntity.status(HttpStatus.INTERNAL_SERVER_ERROR).body(response);
        }
    }

    /**
     * 获取租户的所有配置
     */
    @GetMapping("/{tenantCode}")
    public ResponseEntity<Map<String, Object>> getAllConfigs(@PathVariable String tenantCode) {
        try {
            logger.info("获取租户所有配置: {}", tenantCode);
            
            Map<String, String> configs = tenantConfigService.getAllConfigs(tenantCode);
            
            Map<String, Object> response = new HashMap<>();
            response.put("success", true);
            response.put("data", configs);
            
            return ResponseEntity.ok(response);
        } catch (Exception e) {
            logger.error("获取所有配置异常", e);
            
            Map<String, Object> response = new HashMap<>();
            response.put("success", false);
            response.put("message", "服务器内部错误");
            
            return ResponseEntity.status(HttpStatus.INTERNAL_SERVER_ERROR).body(response);
        }
    }

    /**
     * 获取租户的所有配置对象
     */
    @GetMapping("/{tenantCode}/objects")
    public ResponseEntity<Map<String, Object>> getAllConfigObjects(@PathVariable String tenantCode) {
        try {
            logger.info("获取租户所有配置对象: {}", tenantCode);
            
            List<TenantConfig> configs = tenantConfigService.getAllConfigObjects(tenantCode);
            
            Map<String, Object> response = new HashMap<>();
            response.put("success", true);
            response.put("data", configs);
            
            return ResponseEntity.ok(response);
        } catch (Exception e) {
            logger.error("获取所有配置对象异常", e);
            
            Map<String, Object> response = new HashMap<>();
            response.put("success", false);
            response.put("message", "服务器内部错误");
            
            return ResponseEntity.status(HttpStatus.INTERNAL_SERVER_ERROR).body(response);
        }
    }

    /**
     * 根据配置键前缀获取配置
     */
    @GetMapping("/{tenantCode}/prefix/{keyPrefix}")
    public ResponseEntity<Map<String, Object>> getConfigsByKeyPrefix(
            @PathVariable String tenantCode,
            @PathVariable String keyPrefix) {
        try {
            logger.info("根据前缀获取租户配置: {} - {}", tenantCode, keyPrefix);
            
            Map<String, String> configs = tenantConfigService.getConfigsByKeyPrefix(tenantCode, keyPrefix);
            
            Map<String, Object> response = new HashMap<>();
            response.put("success", true);
            response.put("data", configs);
            
            return ResponseEntity.ok(response);
        } catch (Exception e) {
            logger.error("根据前缀获取配置异常", e);
            
            Map<String, Object> response = new HashMap<>();
            response.put("success", false);
            response.put("message", "服务器内部错误");
            
            return ResponseEntity.status(HttpStatus.INTERNAL_SERVER_ERROR).body(response);
        }
    }

    /**
     * 删除租户配置
     */
    @DeleteMapping("/{tenantCode}/{configKey}")
    public ResponseEntity<Map<String, Object>> deleteConfig(
            @PathVariable String tenantCode,
            @PathVariable String configKey) {
        try {
            logger.info("删除租户配置: {} - {}", tenantCode, configKey);
            
            boolean deleted = tenantConfigService.deleteConfig(tenantCode, configKey);
            
            Map<String, Object> response = new HashMap<>();
            if (deleted) {
                response.put("success", true);
                response.put("message", "配置删除成功");
            } else {
                response.put("success", false);
                response.put("message", "配置不存在");
            }
            
            return ResponseEntity.ok(response);
        } catch (Exception e) {
            logger.error("删除配置异常", e);
            
            Map<String, Object> response = new HashMap<>();
            response.put("success", false);
            response.put("message", "服务器内部错误");
            
            return ResponseEntity.status(HttpStatus.INTERNAL_SERVER_ERROR).body(response);
        }
    }

    /**
     * 批量删除租户配置
     */
    @DeleteMapping("/{tenantCode}/batch")
    public ResponseEntity<Map<String, Object>> deleteConfigs(
            @PathVariable String tenantCode,
            @RequestBody List<String> configKeys) {
        try {
            logger.info("批量删除租户配置: {} - {} 项", tenantCode, configKeys.size());
            
            int deletedCount = tenantConfigService.deleteConfigs(tenantCode, configKeys);
            
            Map<String, Object> response = new HashMap<>();
            response.put("success", true);
            response.put("message", "批量删除成功");
            response.put("deletedCount", deletedCount);
            
            return ResponseEntity.ok(response);
        } catch (Exception e) {
            logger.error("批量删除配置异常", e);
            
            Map<String, Object> response = new HashMap<>();
            response.put("success", false);
            response.put("message", "服务器内部错误");
            
            return ResponseEntity.status(HttpStatus.INTERNAL_SERVER_ERROR).body(response);
        }
    }

    /**
     * 删除租户的所有配置
     */
    @DeleteMapping("/{tenantCode}")
    public ResponseEntity<Map<String, Object>> deleteAllConfigs(@PathVariable String tenantCode) {
        try {
            logger.info("删除租户所有配置: {}", tenantCode);
            
            int deletedCount = tenantConfigService.deleteAllConfigs(tenantCode);
            
            Map<String, Object> response = new HashMap<>();
            response.put("success", true);
            response.put("message", "所有配置删除成功");
            response.put("deletedCount", deletedCount);
            
            return ResponseEntity.ok(response);
        } catch (Exception e) {
            logger.error("删除所有配置异常", e);
            
            Map<String, Object> response = new HashMap<>();
            response.put("success", false);
            response.put("message", "服务器内部错误");
            
            return ResponseEntity.status(HttpStatus.INTERNAL_SERVER_ERROR).body(response);
        }
    }

    /**
     * 检查配置是否存在
     */
    @GetMapping("/{tenantCode}/{configKey}/exists")
    public ResponseEntity<Map<String, Object>> configExists(
            @PathVariable String tenantCode,
            @PathVariable String configKey) {
        try {
            logger.info("检查配置是否存在: {} - {}", tenantCode, configKey);
            
            boolean exists = tenantConfigService.configExists(tenantCode, configKey);
            
            Map<String, Object> response = new HashMap<>();
            response.put("success", true);
            response.put("exists", exists);
            
            return ResponseEntity.ok(response);
        } catch (Exception e) {
            logger.error("检查配置存在性异常", e);
            
            Map<String, Object> response = new HashMap<>();
            response.put("success", false);
            response.put("message", "服务器内部错误");
            
            return ResponseEntity.status(HttpStatus.INTERNAL_SERVER_ERROR).body(response);
        }
    }

    /**
     * 统计租户配置数量
     */
    @GetMapping("/{tenantCode}/count")
    public ResponseEntity<Map<String, Object>> countConfigs(@PathVariable String tenantCode) {
        try {
            logger.info("统计租户配置数量: {}", tenantCode);
            
            long count = tenantConfigService.countConfigs(tenantCode);
            
            Map<String, Object> response = new HashMap<>();
            response.put("success", true);
            response.put("count", count);
            
            return ResponseEntity.ok(response);
        } catch (Exception e) {
            logger.error("统计配置数量异常", e);
            
            Map<String, Object> response = new HashMap<>();
            response.put("success", false);
            response.put("message", "服务器内部错误");
            
            return ResponseEntity.status(HttpStatus.INTERNAL_SERVER_ERROR).body(response);
        }
    }
} 