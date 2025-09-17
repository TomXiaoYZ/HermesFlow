package com.hermesflow.gateway;

import org.springframework.boot.SpringApplication;
import org.springframework.boot.autoconfigure.SpringBootApplication;

/**
 * HermesFlow Gateway 启动类
 * 作为前端统一入口网关，负责路由转发、认证、限流等功能。
 */
@SpringBootApplication
public class GatewayApplication {
    public static void main(String[] args) {
        SpringApplication.run(GatewayApplication.class, args);
    }
} 