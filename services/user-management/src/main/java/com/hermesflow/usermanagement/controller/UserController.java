package com.hermesflow.usermanagement.controller;

import com.hermesflow.usermanagement.dto.LoginRequest;
import com.hermesflow.usermanagement.dto.UserCreateRequest;
import com.hermesflow.usermanagement.entity.User;
import com.hermesflow.usermanagement.entity.UserSession;
import com.hermesflow.usermanagement.service.UserService;
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
import java.util.Map;
import java.util.Optional;
import java.util.UUID;

/**
 * 用户管理控制器
 * 提供用户相关的REST API接口
 */
@RestController
@RequestMapping("/api/users")
public class UserController {

    private static final Logger logger = LoggerFactory.getLogger(UserController.class);

    private final UserService userService;

    @Autowired
    public UserController(UserService userService) {
        this.userService = userService;
    }

    /**
     * 用户注册
     */
    @PostMapping("/register")
    public ResponseEntity<Map<String, Object>> registerUser(@Valid @RequestBody UserCreateRequest request) {
        try {
            logger.info("用户注册请求: {} (租户: {})", request.getUsername(), request.getTenantCode());
            
            User user = userService.createUser(request);
            
            Map<String, Object> response = new HashMap<>();
            response.put("success", true);
            response.put("message", "用户注册成功");
            response.put("data", Map.of(
                "id", user.getId(),
                "username", user.getUsername(),
                "email", user.getEmail(),
                "status", user.getStatus(),
                "createdAt", user.getCreatedAt()
            ));
            
            return ResponseEntity.status(HttpStatus.CREATED).body(response);
        } catch (IllegalArgumentException e) {
            logger.warn("用户注册失败: {}", e.getMessage());
            
            Map<String, Object> response = new HashMap<>();
            response.put("success", false);
            response.put("message", e.getMessage());
            
            return ResponseEntity.badRequest().body(response);
        } catch (Exception e) {
            logger.error("用户注册异常", e);
            
            Map<String, Object> response = new HashMap<>();
            response.put("success", false);
            response.put("message", "服务器内部错误");
            
            return ResponseEntity.status(HttpStatus.INTERNAL_SERVER_ERROR).body(response);
        }
    }

    /**
     * 用户登录
     */
    @PostMapping("/login")
    public ResponseEntity<Map<String, Object>> login(@Valid @RequestBody LoginRequest request) {
        try {
            logger.info("用户登录请求: {} (租户: {})", request.getUsername(), request.getTenantCode());
            
            UserSession session = userService.login(request);
            
            Map<String, Object> response = new HashMap<>();
            response.put("success", true);
            response.put("message", "登录成功");
            response.put("data", Map.of(
                "sessionToken", session.getSessionToken(),
                "refreshToken", session.getRefreshToken(),
                "expiresAt", session.getExpiresAt(),
                "user", Map.of(
                    "id", session.getUser().getId(),
                    "username", session.getUser().getUsername(),
                    "email", session.getUser().getEmail(),
                    "firstName", session.getUser().getFirstName(),
                    "lastName", session.getUser().getLastName()
                )
            ));
            
            return ResponseEntity.ok(response);
        } catch (IllegalArgumentException | IllegalStateException e) {
            logger.warn("用户登录失败: {}", e.getMessage());
            
            Map<String, Object> response = new HashMap<>();
            response.put("success", false);
            response.put("message", e.getMessage());
            
            return ResponseEntity.badRequest().body(response);
        } catch (Exception e) {
            logger.error("用户登录异常", e);
            
            Map<String, Object> response = new HashMap<>();
            response.put("success", false);
            response.put("message", "服务器内部错误");
            
            return ResponseEntity.status(HttpStatus.INTERNAL_SERVER_ERROR).body(response);
        }
    }

    /**
     * 用户登出
     */
    @PostMapping("/logout")
    public ResponseEntity<Map<String, Object>> logout(@RequestHeader("Authorization") String authHeader) {
        try {
            String sessionToken = extractTokenFromHeader(authHeader);
            if (sessionToken == null) {
                Map<String, Object> response = new HashMap<>();
                response.put("success", false);
                response.put("message", "无效的授权头");
                return ResponseEntity.badRequest().body(response);
            }
            
            logger.info("用户登出请求");
            
            userService.logout(sessionToken);
            
            Map<String, Object> response = new HashMap<>();
            response.put("success", true);
            response.put("message", "登出成功");
            
            return ResponseEntity.ok(response);
        } catch (Exception e) {
            logger.error("用户登出异常", e);
            
            Map<String, Object> response = new HashMap<>();
            response.put("success", false);
            response.put("message", "服务器内部错误");
            
            return ResponseEntity.status(HttpStatus.INTERNAL_SERVER_ERROR).body(response);
        }
    }

    /**
     * 验证会话
     */
    @GetMapping("/session/validate")
    public ResponseEntity<Map<String, Object>> validateSession(@RequestHeader("Authorization") String authHeader) {
        try {
            String sessionToken = extractTokenFromHeader(authHeader);
            if (sessionToken == null) {
                Map<String, Object> response = new HashMap<>();
                response.put("success", false);
                response.put("message", "无效的授权头");
                return ResponseEntity.badRequest().body(response);
            }
            
            boolean isValid = userService.isSessionValid(sessionToken);
            
            Map<String, Object> response = new HashMap<>();
            response.put("success", true);
            response.put("valid", isValid);
            
            if (isValid) {
                Optional<UserSession> session = userService.getSessionByToken(sessionToken);
                if (session.isPresent()) {
                    User user = session.get().getUser();
                    response.put("user", Map.of(
                        "id", user.getId(),
                        "username", user.getUsername(),
                        "email", user.getEmail(),
                        "firstName", user.getFirstName(),
                        "lastName", user.getLastName(),
                        "tenantCode", user.getTenant().getCode()
                    ));
                }
            }
            
            return ResponseEntity.ok(response);
        } catch (Exception e) {
            logger.error("会话验证异常", e);
            
            Map<String, Object> response = new HashMap<>();
            response.put("success", false);
            response.put("message", "服务器内部错误");
            
            return ResponseEntity.status(HttpStatus.INTERNAL_SERVER_ERROR).body(response);
        }
    }

    /**
     * 更新用户信息
     */
    @PutMapping("/{userId}")
    public ResponseEntity<Map<String, Object>> updateUser(
            @PathVariable UUID userId,
            @Valid @RequestBody UserCreateRequest request) {
        try {
            logger.info("更新用户信息: {}", userId);
            
            User user = userService.updateUser(userId, request);
            
            Map<String, Object> response = new HashMap<>();
            response.put("success", true);
            response.put("message", "用户更新成功");
            response.put("data", Map.of(
                "id", user.getId(),
                "username", user.getUsername(),
                "email", user.getEmail(),
                "firstName", user.getFirstName(),
                "lastName", user.getLastName(),
                "status", user.getStatus(),
                "updatedAt", user.getUpdatedAt()
            ));
            
            return ResponseEntity.ok(response);
        } catch (IllegalArgumentException e) {
            logger.warn("用户更新失败: {}", e.getMessage());
            
            Map<String, Object> response = new HashMap<>();
            response.put("success", false);
            response.put("message", e.getMessage());
            
            return ResponseEntity.badRequest().body(response);
        } catch (Exception e) {
            logger.error("用户更新异常", e);
            
            Map<String, Object> response = new HashMap<>();
            response.put("success", false);
            response.put("message", "服务器内部错误");
            
            return ResponseEntity.status(HttpStatus.INTERNAL_SERVER_ERROR).body(response);
        }
    }

    /**
     * 禁用用户
     */
    @PostMapping("/{userId}/disable")
    public ResponseEntity<Map<String, Object>> disableUser(@PathVariable UUID userId) {
        try {
            logger.info("禁用用户: {}", userId);
            
            User user = userService.disableUser(userId);
            
            Map<String, Object> response = new HashMap<>();
            response.put("success", true);
            response.put("message", "用户禁用成功");
            response.put("data", user);
            
            return ResponseEntity.ok(response);
        } catch (IllegalArgumentException e) {
            logger.warn("用户禁用失败: {}", e.getMessage());
            
            Map<String, Object> response = new HashMap<>();
            response.put("success", false);
            response.put("message", e.getMessage());
            
            return ResponseEntity.badRequest().body(response);
        } catch (Exception e) {
            logger.error("用户禁用异常", e);
            
            Map<String, Object> response = new HashMap<>();
            response.put("success", false);
            response.put("message", "服务器内部错误");
            
            return ResponseEntity.status(HttpStatus.INTERNAL_SERVER_ERROR).body(response);
        }
    }

    /**
     * 启用用户
     */
    @PostMapping("/{userId}/enable")
    public ResponseEntity<Map<String, Object>> enableUser(@PathVariable UUID userId) {
        try {
            logger.info("启用用户: {}", userId);
            
            User user = userService.enableUser(userId);
            
            Map<String, Object> response = new HashMap<>();
            response.put("success", true);
            response.put("message", "用户启用成功");
            response.put("data", user);
            
            return ResponseEntity.ok(response);
        } catch (IllegalArgumentException e) {
            logger.warn("用户启用失败: {}", e.getMessage());
            
            Map<String, Object> response = new HashMap<>();
            response.put("success", false);
            response.put("message", e.getMessage());
            
            return ResponseEntity.badRequest().body(response);
        } catch (Exception e) {
            logger.error("用户启用异常", e);
            
            Map<String, Object> response = new HashMap<>();
            response.put("success", false);
            response.put("message", "服务器内部错误");
            
            return ResponseEntity.status(HttpStatus.INTERNAL_SERVER_ERROR).body(response);
        }
    }

    /**
     * 获取租户用户列表（分页）
     */
    @GetMapping("/tenant/{tenantCode}")
    public ResponseEntity<Map<String, Object>> getUsersByTenant(
            @PathVariable String tenantCode,
            @RequestParam(defaultValue = "0") int page,
            @RequestParam(defaultValue = "20") int size,
            @RequestParam(defaultValue = "createdAt") String sortBy,
            @RequestParam(defaultValue = "desc") String sortDir,
            @RequestParam(required = false) String status) {
        try {
            logger.info("获取租户用户列表: {} - 页码: {}, 大小: {}", tenantCode, page, size);
            
            Sort sort = Sort.by(sortDir.equalsIgnoreCase("desc") ? 
                Sort.Direction.DESC : Sort.Direction.ASC, sortBy);
            Pageable pageable = PageRequest.of(page, size, sort);
            
            Page<User> users;
            if (status != null && !status.isEmpty()) {
                User.UserStatus userStatus = User.UserStatus.valueOf(status.toUpperCase());
                users = userService.getUsersByTenantAndStatus(tenantCode, userStatus, pageable);
            } else {
                users = userService.getUsersByTenant(tenantCode, pageable);
            }
            
            Map<String, Object> response = new HashMap<>();
            response.put("success", true);
            response.put("data", users.getContent());
            response.put("pagination", Map.of(
                "page", users.getNumber(),
                "size", users.getSize(),
                "totalElements", users.getTotalElements(),
                "totalPages", users.getTotalPages(),
                "first", users.isFirst(),
                "last", users.isLast()
            ));
            
            return ResponseEntity.ok(response);
        } catch (Exception e) {
            logger.error("获取租户用户列表异常", e);
            
            Map<String, Object> response = new HashMap<>();
            response.put("success", false);
            response.put("message", "服务器内部错误");
            
            return ResponseEntity.status(HttpStatus.INTERNAL_SERVER_ERROR).body(response);
        }
    }

    /**
     * 搜索用户
     */
    @GetMapping("/tenant/{tenantCode}/search")
    public ResponseEntity<Map<String, Object>> searchUsers(
            @PathVariable String tenantCode,
            @RequestParam String keyword,
            @RequestParam(defaultValue = "0") int page,
            @RequestParam(defaultValue = "20") int size) {
        try {
            logger.info("搜索用户: {} - {}", tenantCode, keyword);
            
            Pageable pageable = PageRequest.of(page, size, Sort.by("createdAt").descending());
            Page<User> users = userService.searchUsers(tenantCode, keyword, pageable);
            
            Map<String, Object> response = new HashMap<>();
            response.put("success", true);
            response.put("data", users.getContent());
            response.put("pagination", Map.of(
                "page", users.getNumber(),
                "size", users.getSize(),
                "totalElements", users.getTotalElements(),
                "totalPages", users.getTotalPages()
            ));
            
            return ResponseEntity.ok(response);
        } catch (Exception e) {
            logger.error("搜索用户异常", e);
            
            Map<String, Object> response = new HashMap<>();
            response.put("success", false);
            response.put("message", "服务器内部错误");
            
            return ResponseEntity.status(HttpStatus.INTERNAL_SERVER_ERROR).body(response);
        }
    }

    /**
     * 从Authorization头中提取token
     */
    private String extractTokenFromHeader(String authHeader) {
        if (authHeader != null && authHeader.startsWith("Bearer ")) {
            return authHeader.substring(7);
        }
        return null;
    }
} 