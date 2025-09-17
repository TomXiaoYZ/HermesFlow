package com.hermesflow.usermanagement.controller;

import com.hermesflow.usermanagement.dto.request.LoginRequest;
import com.hermesflow.usermanagement.dto.request.RegisterRequest;
import com.hermesflow.usermanagement.dto.response.JwtResponse;
import com.hermesflow.usermanagement.dto.response.MessageResponse;
import com.hermesflow.usermanagement.dto.UserCreateRequest;
import com.hermesflow.usermanagement.entity.Tenant;
import com.hermesflow.usermanagement.repository.TenantRepository;
import com.hermesflow.usermanagement.security.JwtUtils;
import com.hermesflow.usermanagement.service.UserService;
import io.swagger.v3.oas.annotations.Operation;
import io.swagger.v3.oas.annotations.tags.Tag;
import jakarta.validation.Valid;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.http.ResponseEntity;
import org.springframework.security.authentication.AuthenticationManager;
import org.springframework.security.authentication.UsernamePasswordAuthenticationToken;
import org.springframework.security.core.Authentication;
import org.springframework.security.core.context.SecurityContextHolder;
import org.springframework.security.crypto.password.PasswordEncoder;
import org.springframework.web.bind.annotation.*;
import com.hermesflow.usermanagement.security.UserPrincipal;
import com.hermesflow.usermanagement.entity.User;
import com.hermesflow.usermanagement.repository.UserRepository;

import java.util.UUID;
import java.util.HashMap;
import java.util.Map;

@RestController
@RequestMapping("/auth")
@Tag(name = "Authentication", description = "Authentication management APIs")
public class AuthController {

    private static final Logger logger = LoggerFactory.getLogger(AuthController.class);

    @Autowired
    private AuthenticationManager authenticationManager;

    @Autowired
    private UserService userService;

    @Autowired
    private JwtUtils jwtUtils;

    @Autowired
    private TenantRepository tenantRepository;

    @Autowired
    private PasswordEncoder passwordEncoder;

    @Autowired
    private UserRepository userRepository;

    @PostMapping("/login")
    @Operation(summary = "User login", description = "Authenticate user and return JWT token")
    public ResponseEntity<?> authenticateUser(@Valid @RequestBody LoginRequest loginRequest) {
        logger.info("登录请求: username={}", loginRequest.getUsername());
        
        try {
            Authentication authentication = authenticationManager.authenticate(
                    new UsernamePasswordAuthenticationToken(loginRequest.getUsername(), loginRequest.getPassword()));
            
            SecurityContextHolder.getContext().setAuthentication(authentication);
            String jwt = jwtUtils.generateJwtToken(authentication);
            
            logger.info("登录成功: username={}", loginRequest.getUsername());
            return ResponseEntity.ok(new JwtResponse(jwt, "Bearer"));
        } catch (Exception e) {
            logger.error("登录失败: username={}, error={}", loginRequest.getUsername(), e.getMessage(), e);
            throw e;
        }
    }

    @PostMapping("/register")
    @Operation(summary = "User registration", description = "Register a new user")
    public ResponseEntity<?> registerUser(@Valid @RequestBody RegisterRequest registerRequest) {
        logger.info("注册请求: username={}, tenantId={}", registerRequest.getUsername(), registerRequest.getTenantId());
        
        if (userService.existsByUsername(registerRequest.getUsername())) {
            return ResponseEntity.badRequest()
                .body(new MessageResponse("Error: Username is already taken!"));
        }

        if (userService.existsByEmail(registerRequest.getEmail())) {
            return ResponseEntity.badRequest()
                .body(new MessageResponse("Error: Email is already in use!"));
        }

        // Find tenant by ID and get tenant code
        UUID tenantId;
        try {
            tenantId = UUID.fromString(registerRequest.getTenantId());
        } catch (IllegalArgumentException e) {
            return ResponseEntity.badRequest()
                .body(new MessageResponse("Error: Invalid tenant ID format!"));
        }

        Tenant tenant = tenantRepository.findById(tenantId)
            .orElseThrow(() -> new IllegalArgumentException("租户不存在: " + registerRequest.getTenantId()));

        logger.info("找到租户: id={}, code={}, name={}", tenant.getId(), tenant.getCode(), tenant.getName());

        // Convert RegisterRequest to UserCreateRequest
        UserCreateRequest userCreateRequest = new UserCreateRequest(
            tenant.getCode(),
            registerRequest.getUsername(),
            registerRequest.getEmail(),
            registerRequest.getPassword()
        );
        userCreateRequest.setFirstName(registerRequest.getFirstName());
        userCreateRequest.setLastName(registerRequest.getLastName());

        logger.info("创建UserCreateRequest: tenantCode={}, username={}", userCreateRequest.getTenantCode(), userCreateRequest.getUsername());

        // Create new user
        userService.createUser(userCreateRequest);

        return ResponseEntity.ok(new MessageResponse("User registered successfully!"));
    }

    @PostMapping("/refresh")
    @Operation(summary = "Refresh token", description = "Refresh JWT token")
    public ResponseEntity<?> refreshToken(@RequestHeader("Authorization") String token) {
        if (token != null && token.startsWith("Bearer ")) {
            String jwt = token.substring(7);
            if (jwtUtils.validateJwtToken(jwt)) {
                String username = jwtUtils.getUserNameFromJwtToken(jwt);
                String newToken = jwtUtils.generateTokenFromUsername(username);
                return ResponseEntity.ok(new JwtResponse(newToken, "Bearer"));
            }
        }
        return ResponseEntity.badRequest().body(new MessageResponse("Invalid token"));
    }

    @GetMapping("/profile")
    @Operation(summary = "Get user profile", description = "Get current user profile information")
    public ResponseEntity<?> getUserProfile() {
        try {
            Authentication authentication = SecurityContextHolder.getContext().getAuthentication();
            if (authentication == null || !authentication.isAuthenticated()) {
                return ResponseEntity.status(401).body(new MessageResponse("User not authenticated"));
            }

            UserPrincipal userPrincipal = (UserPrincipal) authentication.getPrincipal();
            String username = userPrincipal.getUsername();
            
            User user = userRepository.findByUsername(username)
                .orElseThrow(() -> new RuntimeException("User not found: " + username));

            Map<String, Object> userInfo = new HashMap<>();
            userInfo.put("id", user.getId());
            userInfo.put("username", user.getUsername());
            userInfo.put("email", user.getEmail());
            userInfo.put("firstName", user.getFirstName());
            userInfo.put("lastName", user.getLastName());
            userInfo.put("phone", user.getPhone());
            userInfo.put("status", user.getStatus());
            userInfo.put("emailVerified", user.getEmailVerified());
            userInfo.put("phoneVerified", user.getPhoneVerified());
            userInfo.put("lastLoginAt", user.getLastLoginAt());
            userInfo.put("createdAt", user.getCreatedAt());
            userInfo.put("tenant", Map.of(
                "id", user.getTenant().getId(),
                "code", user.getTenant().getCode(),
                "name", user.getTenant().getName()
            ));

            return ResponseEntity.ok(userInfo);
        } catch (Exception e) {
            logger.error("获取用户信息失败", e);
            return ResponseEntity.status(500).body(new MessageResponse("Internal server error"));
        }
    }
} 