package com.hermesflow.usermanagement.controller;

import lombok.extern.slf4j.Slf4j;
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.http.ResponseEntity;
import org.springframework.web.bind.annotation.GetMapping;
import org.springframework.web.bind.annotation.RequestMapping;
import org.springframework.web.bind.annotation.RestController;

import javax.sql.DataSource;
import java.sql.Connection;
import java.time.LocalDateTime;
import java.util.HashMap;
import java.util.Map;

/**
 * 健康检查控制器
 * 
 * 提供服务状态检查和基本信息接口
 */
@Slf4j
@RestController
@RequestMapping("/health")
public class HealthController {

    @Autowired
    private DataSource dataSource;

    private final LocalDateTime startTime = LocalDateTime.now();

    @GetMapping
    public ResponseEntity<Map<String, Object>> health() {
        Map<String, Object> response = new HashMap<>();
        response.put("status", "UP");
        response.put("timestamp", LocalDateTime.now());
        response.put("service", "user-management");
        response.put("version", "1.0.0");
        return ResponseEntity.ok(response);
    }

    @GetMapping("/detailed")
    public ResponseEntity<Map<String, Object>> detailedHealth() {
        Map<String, Object> response = new HashMap<>();
        Map<String, Object> checks = new HashMap<>();
        
        // Database health check
        checks.put("database", checkDatabaseHealth());
        
        // Overall status
        boolean allHealthy = checks.values().stream()
                .allMatch(check -> "UP".equals(((Map<?, ?>) check).get("status")));
        
        response.put("status", allHealthy ? "UP" : "DOWN");
        response.put("timestamp", LocalDateTime.now());
        response.put("service", "user-management");
        response.put("checks", checks);
        
        return ResponseEntity.ok(response);
    }

    @GetMapping("/ready")
    public ResponseEntity<Map<String, Object>> readiness() {
        Map<String, Object> response = new HashMap<>();
        
        // Check if service is ready to accept requests
        boolean databaseReady = checkDatabaseHealth().get("status").equals("UP");
        
        response.put("status", databaseReady ? "UP" : "DOWN");
        response.put("timestamp", LocalDateTime.now());
        response.put("ready", databaseReady);
        
        return ResponseEntity.ok(response);
    }

    @GetMapping("/live")
    public ResponseEntity<Map<String, Object>> liveness() {
        Map<String, Object> response = new HashMap<>();
        response.put("status", "UP");
        response.put("timestamp", LocalDateTime.now());
        response.put("uptime", java.time.Duration.between(startTime, LocalDateTime.now()).toString());
        return ResponseEntity.ok(response);
    }

    private Map<String, Object> checkDatabaseHealth() {
        Map<String, Object> health = new HashMap<>();
        try (Connection connection = dataSource.getConnection()) {
            boolean isValid = connection.isValid(5);
            health.put("status", isValid ? "UP" : "DOWN");
            health.put("database", "PostgreSQL");
            if (isValid) {
                health.put("details", "Connection successful");
            } else {
                health.put("details", "Connection validation failed");
            }
        } catch (Exception e) {
            log.error("Database health check failed", e);
            health.put("status", "DOWN");
            health.put("error", e.getMessage());
        }
        return health;
    }
} 