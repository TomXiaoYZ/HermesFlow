package com.hermesflow.usermanagement.config;

import com.hermesflow.usermanagement.security.JwtUtils;
import io.jsonwebtoken.Claims;
import jakarta.servlet.http.HttpServletRequest;
import jakarta.servlet.http.HttpServletResponse;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.stereotype.Component;
import org.springframework.util.StringUtils;
import org.springframework.web.servlet.HandlerInterceptor;

import javax.sql.DataSource;
import java.sql.Connection;
import java.sql.PreparedStatement;
import java.sql.SQLException;
import java.util.UUID;

/**
 * 租户拦截器
 * 
 * 负责从HTTP请求中提取租户信息，设置租户上下文，并配置数据库连接的租户隔离。
 * 在每个请求开始时设置租户上下文，请求结束时清理上下文。
 * 
 * @author HermesFlow
 * @version 1.0
 * @since 2024-12-19
 */
@Component
public class TenantInterceptor implements HandlerInterceptor {
    
    private static final Logger logger = LoggerFactory.getLogger(TenantInterceptor.class);
    
    @Autowired
    private TenantContext tenantContext;
    
    @Autowired
    private JwtUtils jwtUtils;
    
    @Autowired
    private DataSource dataSource;
    
    /**
     * 请求处理前的拦截方法
     * 
     * 从JWT令牌中提取租户信息，设置租户上下文，并配置数据库连接的租户隔离策略。
     * 
     * @param request HTTP请求对象
     * @param response HTTP响应对象
     * @param handler 处理器对象
     * @return 是否继续处理请求
     * @throws Exception 处理过程中可能抛出的异常
     */
    @Override
    public boolean preHandle(HttpServletRequest request, 
                           HttpServletResponse response, 
                           Object handler) throws Exception {
        
        logger.debug("TenantInterceptor.preHandle - Processing request: {} {}", 
                    request.getMethod(), request.getRequestURI());
        
        // 跳过公开接口的租户检查
        if (isPublicEndpoint(request)) {
            logger.debug("Skipping tenant context for public endpoint: {}", request.getRequestURI());
            return true;
        }
        
        try {
            // 从请求中提取JWT令牌
            String token = extractTokenFromRequest(request);
            
            if (token != null && jwtUtils.validateJwtToken(token)) {
                // 从JWT令牌中提取租户信息
                UUID tenantId = extractTenantFromToken(token);
                
                if (tenantId != null) {
                    // 设置租户上下文
                    tenantContext.setCurrentTenantId(tenantId);
                    
                    // 设置数据库连接的租户上下文
                    setDatabaseTenantContext(tenantId);
                    
                    logger.debug("Tenant context set successfully: {}", tenantId);
                } else {
                    logger.warn("No tenant information found in JWT token for request: {}", 
                              request.getRequestURI());
                }
            } else {
                logger.debug("No valid JWT token found for request: {}", request.getRequestURI());
            }
            
        } catch (Exception e) {
            logger.error("Error setting tenant context for request: {} - {}", 
                        request.getRequestURI(), e.getMessage(), e);
            // 不阻止请求继续处理，让Spring Security处理认证失败
        }
        
        return true;
    }
    
    /**
     * 请求处理完成后的清理方法
     * 
     * 清除租户上下文，防止内存泄漏和线程污染。
     * 
     * @param request HTTP请求对象
     * @param response HTTP响应对象
     * @param handler 处理器对象
     * @param ex 处理过程中的异常（如果有）
     * @throws Exception 清理过程中可能抛出的异常
     */
    @Override
    public void afterCompletion(HttpServletRequest request, 
                              HttpServletResponse response, 
                              Object handler, 
                              Exception ex) throws Exception {
        
        try {
            // 清除数据库租户上下文
            clearDatabaseTenantContext();
            
            // 清除应用层租户上下文
            tenantContext.clear();
            
            logger.debug("Tenant context cleared for request: {} {}", 
                        request.getMethod(), request.getRequestURI());
            
        } catch (Exception e) {
            logger.error("Error clearing tenant context for request: {} - {}", 
                        request.getRequestURI(), e.getMessage(), e);
        }
    }
    
    /**
     * 从HTTP请求中提取JWT令牌
     * 
     * @param request HTTP请求对象
     * @return JWT令牌字符串，如果未找到则返回null
     */
    private String extractTokenFromRequest(HttpServletRequest request) {
        String bearerToken = request.getHeader("Authorization");
        
        if (StringUtils.hasText(bearerToken) && bearerToken.startsWith("Bearer ")) {
            return bearerToken.substring(7);
        }
        
        return null;
    }
    
    /**
     * 从JWT令牌中提取租户ID
     * 
     * @param token JWT令牌
     * @return 租户ID，如果未找到则返回null
     */
    private UUID extractTenantFromToken(String token) {
        try {
            Claims claims = jwtUtils.getClaimsFromToken(token);
            String tenantIdStr = claims.get("tenantId", String.class);
            
            if (StringUtils.hasText(tenantIdStr)) {
                return UUID.fromString(tenantIdStr);
            }
            
            // 如果JWT中没有租户信息，尝试从用户名查询租户
            String username = claims.getSubject();
            if (StringUtils.hasText(username)) {
                return getTenantIdByUsername(username);
            }
            
        } catch (Exception e) {
            logger.error("Error extracting tenant from JWT token: {}", e.getMessage(), e);
        }
        
        return null;
    }
    
    /**
     * 根据用户名查询租户ID
     * 
     * @param username 用户名
     * @return 租户ID，如果未找到则返回null
     */
    private UUID getTenantIdByUsername(String username) {
        String sql = "SELECT tenant_id FROM users WHERE username = ? LIMIT 1";
        
        try (Connection connection = dataSource.getConnection();
             PreparedStatement statement = connection.prepareStatement(sql)) {
            
            statement.setString(1, username);
            var resultSet = statement.executeQuery();
            
            if (resultSet.next()) {
                return UUID.fromString(resultSet.getString("tenant_id"));
            }
            
        } catch (SQLException e) {
            logger.error("Error querying tenant ID for username: {} - {}", username, e.getMessage(), e);
        }
        
        return null;
    }
    
    /**
     * 设置数据库连接的租户上下文
     * 
     * @param tenantId 租户ID
     */
    private void setDatabaseTenantContext(UUID tenantId) {
        try (Connection connection = dataSource.getConnection();
             PreparedStatement statement = connection.prepareStatement(
                     "SELECT set_current_tenant(?)")) {
            
            statement.setObject(1, tenantId);
            statement.execute();
            
            logger.debug("Database tenant context set: {}", tenantId);
            
        } catch (SQLException e) {
            logger.error("Error setting database tenant context: {}", e.getMessage(), e);
        }
    }
    
    /**
     * 清除数据库连接的租户上下文
     */
    private void clearDatabaseTenantContext() {
        try (Connection connection = dataSource.getConnection();
             PreparedStatement statement = connection.prepareStatement(
                     "SELECT clear_current_tenant()")) {
            
            statement.execute();
            logger.debug("Database tenant context cleared");
            
        } catch (SQLException e) {
            logger.error("Error clearing database tenant context: {}", e.getMessage(), e);
        }
    }
    
    /**
     * 检查是否为公开接口
     * 
     * @param request HTTP请求对象
     * @return 如果是公开接口返回true，否则返回false
     */
    private boolean isPublicEndpoint(HttpServletRequest request) {
        String uri = request.getRequestURI();
        
        // 公开接口列表
        String[] publicEndpoints = {
            "/api/v1/auth/login",
            "/api/v1/auth/register",
            "/api/v1/health",
            "/actuator/health",
            "/swagger-ui",
            "/v3/api-docs",
            "/favicon.ico"
        };
        
        for (String endpoint : publicEndpoints) {
            if (uri.startsWith(endpoint)) {
                return true;
            }
        }
        
        return false;
    }
} 