package com.hermesflow.usermanagement.service;

import com.hermesflow.usermanagement.dto.LoginRequest;
import com.hermesflow.usermanagement.dto.UserCreateRequest;
import com.hermesflow.usermanagement.entity.Tenant;
import com.hermesflow.usermanagement.entity.User;
import com.hermesflow.usermanagement.entity.UserSession;
import com.hermesflow.usermanagement.repository.UserRepository;
import com.hermesflow.usermanagement.repository.UserSessionRepository;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.data.domain.Page;
import org.springframework.data.domain.Pageable;
import org.springframework.security.crypto.password.PasswordEncoder;
import org.springframework.stereotype.Service;
import org.springframework.transaction.annotation.Transactional;

import java.time.LocalDateTime;
import java.util.List;
import java.util.Optional;
import java.util.UUID;

import com.hermesflow.usermanagement.config.TenantContext;

/**
 * 用户服务类
 * 提供用户管理的核心业务逻辑
 */
@Service
@Transactional
public class UserService {

    private static final Logger logger = LoggerFactory.getLogger(UserService.class);

    private final UserRepository userRepository;
    private final UserSessionRepository userSessionRepository;
    private final TenantService tenantService;
    private final PasswordEncoder passwordEncoder;

    @Autowired
    public UserService(UserRepository userRepository,
                      UserSessionRepository userSessionRepository,
                      TenantService tenantService,
                      PasswordEncoder passwordEncoder) {
        this.userRepository = userRepository;
        this.userSessionRepository = userSessionRepository;
        this.tenantService = tenantService;
        this.passwordEncoder = passwordEncoder;
    }

    /**
     * 创建用户
     * @param request 用户创建请求
     * @return 创建的用户
     */
    public User createUser(UserCreateRequest request) {
        logger.info("创建用户: {} (租户: {})", request.getUsername(), request.getTenantCode());

        // 获取租户
        Tenant tenant = tenantService.getTenantByCodeRequired(request.getTenantCode());

        // 检查用户名是否在租户内已存在
        if (userRepository.existsByUsernameAndTenant(request.getUsername(), tenant)) {
            throw new IllegalArgumentException("用户名在该租户内已存在: " + request.getUsername());
        }

        // 检查邮箱是否在租户内已存在
        if (userRepository.existsByEmailAndTenant(request.getEmail(), tenant)) {
            throw new IllegalArgumentException("邮箱在该租户内已存在: " + request.getEmail());
        }

        // 创建用户实体
        User user = new User(
            tenant,
            request.getUsername(),
            request.getEmail(),
            passwordEncoder.encode(request.getPassword())
        );

        // 设置可选字段
        if (request.getFirstName() != null) {
            user.setFirstName(request.getFirstName());
        }
        if (request.getLastName() != null) {
            user.setLastName(request.getLastName());
        }
        if (request.getPhone() != null) {
            user.setPhone(request.getPhone());
        }

        // 保存用户
        User savedUser = userRepository.save(user);
        logger.info("用户创建成功: {} (ID: {})", savedUser.getUsername(), savedUser.getId());

        return savedUser;
    }

    /**
     * 用户登录
     * @param request 登录请求
     * @return 用户会话
     */
    public UserSession login(LoginRequest request) {
        logger.info("用户登录: {} (租户: {})", request.getUsername(), request.getTenantCode());

        // 获取租户
        Tenant tenant = tenantService.getTenantByCodeRequired(request.getTenantCode());

        // 查找用户
        User user = userRepository.findByUsernameAndTenant(request.getUsername(), tenant)
                .orElseThrow(() -> new IllegalArgumentException("用户名或密码错误"));

        // 验证密码
        if (!passwordEncoder.matches(request.getPassword(), user.getPasswordHash())) {
            throw new IllegalArgumentException("用户名或密码错误");
        }

        // 检查用户状态
        if (user.getStatus() != User.UserStatus.ACTIVE) {
            throw new IllegalStateException("用户账户已被禁用");
        }

        // 检查租户状态
        if (tenant.getStatus() != Tenant.TenantStatus.ACTIVE) {
            throw new IllegalStateException("租户账户已被暂停");
        }

        // 终止用户的所有现有会话
        terminateAllUserSessions(user);

        // 创建新会话
        UserSession session = createUserSession(user, request);
        
        // 更新用户最后登录时间
        user.setLastLoginAt(LocalDateTime.now());
        userRepository.save(user);

        logger.info("用户登录成功: {} (会话ID: {})", user.getUsername(), session.getId());
        return session;
    }

    /**
     * 创建用户会话
     * @param user 用户
     * @param request 登录请求
     * @return 用户会话
     */
    private UserSession createUserSession(User user, LoginRequest request) {
        UserSession session = new UserSession(
            user,
            generateSessionToken(),
            generateRefreshToken(),
            LocalDateTime.now().plusHours(24) // 24小时后过期
        );

        if (request.getUserAgent() != null) {
            session.setUserAgent(request.getUserAgent());
        }

        return userSessionRepository.save(session);
    }

    /**
     * 生成会话令牌
     * @return 会话令牌
     */
    private String generateSessionToken() {
        return UUID.randomUUID().toString().replace("-", "");
    }

    /**
     * 生成刷新令牌
     * @return 刷新令牌
     */
    private String generateRefreshToken() {
        return UUID.randomUUID().toString().replace("-", "");
    }

    /**
     * 终止用户的所有会话
     * @param user 用户
     */
    public void terminateAllUserSessions(User user) {
        logger.info("终止用户所有会话: {}", user.getUsername());
        userSessionRepository.terminateAllUserSessions(user.getId());
    }

    /**
     * 根据会话令牌获取用户会话
     * @param sessionToken 会话令牌
     * @return 用户会话
     */
    @Transactional(readOnly = true)
    public Optional<UserSession> getSessionByToken(String sessionToken) {
        return userSessionRepository.findBySessionTokenAndIsActive(sessionToken, true);
    }

    /**
     * 验证会话是否有效
     * @param sessionToken 会话令牌
     * @return 是否有效
     */
    @Transactional(readOnly = true)
    public boolean isSessionValid(String sessionToken) {
        Optional<UserSession> session = getSessionByToken(sessionToken);
        return session.isPresent() && session.get().getExpiresAt().isAfter(LocalDateTime.now());
    }

    /**
     * 用户登出
     * @param sessionToken 会话令牌
     */
    public void logout(String sessionToken) {
        Optional<UserSession> sessionOpt = getSessionByToken(sessionToken);
        if (sessionOpt.isPresent()) {
            UserSession session = sessionOpt.get();
            session.setIsActive(false);
            userSessionRepository.save(session);
            logger.info("用户登出成功: {}", session.getUser().getUsername());
        }
    }

    /**
     * 更新用户信息
     * @param userId 用户ID
     * @param request 更新请求
     * @return 更新后的用户
     */
    public User updateUser(UUID userId, UserCreateRequest request) {
        logger.info("更新用户: {}", userId);

        User user = userRepository.findById(userId)
                .orElseThrow(() -> new IllegalArgumentException("用户不存在: " + userId));

        // 检查邮箱是否与其他用户冲突
        if (!user.getEmail().equals(request.getEmail()) && 
            userRepository.existsByEmailAndTenant(request.getEmail(), user.getTenant())) {
            throw new IllegalArgumentException("邮箱在该租户内已存在: " + request.getEmail());
        }

        // 更新用户信息
        user.setEmail(request.getEmail());
        user.setFirstName(request.getFirstName());
        user.setLastName(request.getLastName());
        user.setPhone(request.getPhone());

        // 如果提供了新密码，则更新密码
        if (request.getPassword() != null && !request.getPassword().trim().isEmpty()) {
            user.setPasswordHash(passwordEncoder.encode(request.getPassword()));
        }

        User updatedUser = userRepository.save(user);
        logger.info("用户更新成功: {}", userId);

        return updatedUser;
    }

    /**
     * 禁用用户
     * @param userId 用户ID
     * @return 更新后的用户
     */
    public User disableUser(UUID userId) {
        logger.info("禁用用户: {}", userId);

        User user = userRepository.findById(userId)
                .orElseThrow(() -> new IllegalArgumentException("用户不存在: " + userId));

        if (user.getStatus() == User.UserStatus.INACTIVE) {
            throw new IllegalStateException("用户已处于禁用状态: " + userId);
        }

        user.setStatus(User.UserStatus.INACTIVE);
        
        // 终止用户的所有会话
        terminateAllUserSessions(user);

        User updatedUser = userRepository.save(user);
        logger.info("用户禁用成功: {}", userId);

        return updatedUser;
    }

    /**
     * 启用用户
     * @param userId 用户ID
     * @return 更新后的用户
     */
    public User enableUser(UUID userId) {
        logger.info("启用用户: {}", userId);

        User user = userRepository.findById(userId)
                .orElseThrow(() -> new IllegalArgumentException("用户不存在: " + userId));

        if (user.getStatus() == User.UserStatus.ACTIVE) {
            throw new IllegalStateException("用户已处于启用状态: " + userId);
        }

        user.setStatus(User.UserStatus.ACTIVE);
        User updatedUser = userRepository.save(user);
        
        logger.info("用户启用成功: {}", userId);
        return updatedUser;
    }

    /**
     * 根据租户获取用户列表（分页）
     * @param tenantCode 租户代码
     * @param pageable 分页参数
     * @return 分页用户列表
     */
    @Transactional(readOnly = true)
    public Page<User> getUsersByTenant(String tenantCode, Pageable pageable) {
        Tenant tenant = tenantService.getTenantByCodeRequired(tenantCode);
        return userRepository.findByTenant(tenant, pageable);
    }

    /**
     * 根据租户和状态获取用户列表（分页）
     * @param tenantCode 租户代码
     * @param status 用户状态
     * @param pageable 分页参数
     * @return 分页用户列表
     */
    @Transactional(readOnly = true)
    public Page<User> getUsersByTenantAndStatus(String tenantCode, User.UserStatus status, Pageable pageable) {
        Tenant tenant = tenantService.getTenantByCodeRequired(tenantCode);
        return userRepository.findByTenantAndStatus(tenant, status, pageable);
    }

    /**
     * 搜索用户
     * @param tenantCode 租户代码
     * @param keyword 搜索关键字
     * @param pageable 分页参数
     * @return 分页用户列表
     */
    @Transactional(readOnly = true)
    public Page<User> searchUsers(String tenantCode, String keyword, Pageable pageable) {
        Tenant tenant = tenantService.getTenantByCodeRequired(tenantCode);
        return userRepository.findByTenantAndUsernameContaining(tenant, keyword, pageable);
    }

    /**
     * 统计租户用户数量
     * @param tenantCode 租户代码
     * @param status 用户状态（可选）
     * @return 用户数量
     */
    @Transactional(readOnly = true)
    public long countUsersByTenant(String tenantCode, User.UserStatus status) {
        Tenant tenant = tenantService.getTenantByCodeRequired(tenantCode);
        if (status == null) {
            return userRepository.countByTenant(tenant);
        }
        return userRepository.countByTenantAndStatus(tenant, status);
    }

    /**
     * 清理过期会话
     * @return 清理的会话数量
     */
    public int cleanupExpiredSessions() {
        logger.info("开始清理过期会话");
        int count = userSessionRepository.deleteExpiredSessions(LocalDateTime.now());
        logger.info("清理了{}个过期会话", count);
        return count;
    }

    /**
     * 清理非活跃会话
     * @param inactiveDays 非活跃天数阈值
     * @return 清理的会话数量
     */
    public int cleanupInactiveSessions(int inactiveDays) {
        logger.info("开始清理{}天未活跃的会话", inactiveDays);
        LocalDateTime threshold = LocalDateTime.now().minusDays(inactiveDays);
        int count = userSessionRepository.deleteOldTerminatedSessions(threshold);
        logger.info("清理了{}个非活跃会话", count);
        return count;
    }

    /**
     * 检查用户名是否存在（跨租户）
     * @param username 用户名
     * @return 是否存在
     */
    @Transactional(readOnly = true)
    public boolean existsByUsername(String username) {
        return userRepository.findByUsername(username).isPresent();
    }

    /**
     * 检查邮箱是否存在（跨租户）
     * @param email 邮箱
     * @return 是否存在
     */
    @Transactional(readOnly = true)
    public boolean existsByEmail(String email) {
        // 需要添加findByEmail方法到repository
        return false; // 临时返回false
    }
} 