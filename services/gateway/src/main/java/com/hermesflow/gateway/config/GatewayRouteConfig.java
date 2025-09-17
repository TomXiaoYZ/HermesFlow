package com.hermesflow.gateway.config;

import org.springframework.cloud.gateway.route.RouteLocator;
import org.springframework.cloud.gateway.route.builder.RouteLocatorBuilder;
import org.springframework.context.annotation.Bean;
import org.springframework.context.annotation.Configuration;

/**
 * 路由配置类：可配置静态路由，也支持Nacos服务发现的动态路由。
 */
@Configuration
public class GatewayRouteConfig {

    /**
     * 示例：静态路由配置
     * 可根据实际业务扩展更多路由规则
     */
    @Bean
    public RouteLocator customRouteLocator(RouteLocatorBuilder builder) {
        return builder.routes()
                .route("user-service", r -> r.path("/api/user/**")
                        .uri("lb://user-management"))
                .route("permission-service", r -> r.path("/api/permission/**")
                        .uri("lb://permission-management"))
                .build();
    }
} 