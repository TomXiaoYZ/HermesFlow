package com.hermesflow.apigateway.filter;

import lombok.extern.slf4j.Slf4j;
import org.springframework.cloud.gateway.filter.GatewayFilterChain;
import org.springframework.cloud.gateway.filter.GlobalFilter;
import org.springframework.core.Ordered;
import org.springframework.http.server.reactive.ServerHttpRequest;
import org.springframework.http.server.reactive.ServerHttpResponse;
import org.springframework.stereotype.Component;
import org.springframework.web.server.ServerWebExchange;
import reactor.core.publisher.Mono;

import java.util.UUID;

/**
 * 全局日志过滤器：记录请求方法、路径、traceId、响应状态等关键信息。
 */
@Slf4j
@Component
public class LoggingFilter implements GlobalFilter, Ordered {
    @Override
    public Mono<Void> filter(ServerWebExchange exchange, GatewayFilterChain chain) {
        ServerHttpRequest request = exchange.getRequest();
        String traceId = UUID.randomUUID().toString();
        // 可将traceId注入header，便于后续链路追踪
        ServerHttpRequest mutatedRequest = request.mutate().header("X-Trace-Id", traceId).build();
        log.info("[API-Gateway] 请求: {} {} traceId={}", request.getMethod(), request.getURI().getPath(), traceId);
        return chain.filter(exchange.mutate().request(mutatedRequest).build())
                .doOnSuccess(aVoid -> {
                    ServerHttpResponse response = exchange.getResponse();
                    log.info("[API-Gateway] 响应: {} {} traceId={} status={}",
                            request.getMethod(), request.getURI().getPath(), traceId, response.getStatusCode());
                });
    }

    @Override
    public int getOrder() {
        return -50; // 日志优先级低于鉴权过滤器
    }
} 