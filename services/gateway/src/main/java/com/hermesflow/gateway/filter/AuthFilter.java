package com.hermesflow.gateway.filter;

import lombok.extern.slf4j.Slf4j;
import org.springframework.cloud.gateway.filter.GatewayFilterChain;
import org.springframework.cloud.gateway.filter.GlobalFilter;
import org.springframework.core.Ordered;
import org.springframework.http.HttpHeaders;
import org.springframework.http.HttpStatus;
import org.springframework.http.server.reactive.ServerHttpRequest;
import org.springframework.stereotype.Component;
import org.springframework.web.server.ServerWebExchange;
import reactor.core.publisher.Mono;

/**
 * JWT认证过滤器：校验Authorization头中的Bearer Token。
 * 未认证请求直接返回401。
 */
@Slf4j
@Component
public class AuthFilter implements GlobalFilter, Ordered {
    @Override
    public Mono<Void> filter(ServerWebExchange exchange, GatewayFilterChain chain) {
        ServerHttpRequest request = exchange.getRequest();
        String authHeader = request.getHeaders().getFirst(HttpHeaders.AUTHORIZATION);
        if (authHeader == null || !authHeader.startsWith("Bearer ")) {
            log.warn("[Auth] 缺少或非法的Authorization头: {}", authHeader);
            exchange.getResponse().setStatusCode(HttpStatus.UNAUTHORIZED);
            return exchange.getResponse().setComplete();
        }
        String token = authHeader.substring(7);
        // TODO: 实际项目中应调用JwtUtil校验token有效性
        if (!validateToken(token)) {
            log.warn("[Auth] 无效Token: {}", token);
            exchange.getResponse().setStatusCode(HttpStatus.UNAUTHORIZED);
            return exchange.getResponse().setComplete();
        }
        return chain.filter(exchange);
    }

    /**
     * 简单Token校验逻辑，实际应调用JwtUtil等工具类
     */
    private boolean validateToken(String token) {
        // 示例：仅校验token非空，实际应校验签名、过期等
        return token != null && !token.trim().isEmpty();
    }

    @Override
    public int getOrder() {
        return -100; // 优先级高于大多数过滤器
    }
} 