package com.hermesflow.apigateway.filter;

import lombok.extern.slf4j.Slf4j;
import org.springframework.cloud.gateway.filter.GatewayFilterChain;
import org.springframework.cloud.gateway.filter.GlobalFilter;
import org.springframework.core.Ordered;
import org.springframework.http.HttpHeaders;
import org.springframework.http.HttpStatus;
import org.springframework.stereotype.Component;
import org.springframework.web.server.ServerWebExchange;
import reactor.core.publisher.Mono;

/**
 * 服务间鉴权过滤器：校验自定义Header（如X-Internal-Token），未认证请求返回401。
 */
@Slf4j
@Component
public class ServiceAuthFilter implements GlobalFilter, Ordered {
    private static final String INTERNAL_TOKEN_HEADER = "X-Internal-Token";
    private static final String EXPECTED_TOKEN = "hermes-internal-token"; // 示例，实际应配置化

    @Override
    public Mono<Void> filter(ServerWebExchange exchange, GatewayFilterChain chain) {
        String token = exchange.getRequest().getHeaders().getFirst(INTERNAL_TOKEN_HEADER);
        if (token == null || !EXPECTED_TOKEN.equals(token)) {
            log.warn("[API-Gateway] 服务间鉴权失败，Header: {}，Token: {}", INTERNAL_TOKEN_HEADER, token);
            exchange.getResponse().setStatusCode(HttpStatus.UNAUTHORIZED);
            return exchange.getResponse().setComplete();
        }
        return chain.filter(exchange);
    }

    @Override
    public int getOrder() {
        return -100; // 优先级高于大多数过滤器
    }
} 