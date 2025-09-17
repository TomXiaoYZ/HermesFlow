package com.hermesflow.usermanagement.config;

import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.context.annotation.Configuration;
import org.springframework.web.servlet.config.annotation.InterceptorRegistry;
import org.springframework.web.servlet.config.annotation.WebMvcConfigurer;

/**
 * Web MVC 配置类
 * 
 * 配置Spring MVC相关的组件，包括拦截器、CORS、静态资源等。
 * 主要用于注册租户拦截器，实现多租户数据隔离。
 * 
 * @author HermesFlow
 * @version 1.0
 * @since 2024-12-19
 */
@Configuration
public class WebConfig implements WebMvcConfigurer {
    
    @Autowired
    private TenantInterceptor tenantInterceptor;
    
    /**
     * 注册拦截器
     * 
     * 将租户拦截器注册到Spring MVC拦截器链中，
     * 确保每个请求都经过租户上下文处理。
     * 
     * @param registry 拦截器注册器
     */
    @Override
    public void addInterceptors(InterceptorRegistry registry) {
        registry.addInterceptor(tenantInterceptor)
                .addPathPatterns("/api/**")  // 拦截所有API请求
                .excludePathPatterns(
                    "/api/v1/auth/login",    // 排除登录接口
                    "/api/v1/auth/register", // 排除注册接口
                    "/api/v1/health",        // 排除健康检查接口
                    "/actuator/**",          // 排除监控端点
                    "/swagger-ui/**",        // 排除Swagger UI
                    "/v3/api-docs/**",       // 排除API文档
                    "/favicon.ico"           // 排除图标请求
                );
    }
} 