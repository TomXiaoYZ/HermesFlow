package com.hermesflow.apigateway;

import org.springframework.boot.SpringApplication;
import org.springframework.boot.autoconfigure.SpringBootApplication;

/**
 * HermesFlow API Gateway 启动类
 * 作为后端微服务统一入口网关，负责服务聚合、鉴权、路由等功能。
 */
@SpringBootApplication
public class ApiGatewayApplication {
    public static void main(String[] args) {
        SpringApplication.run(ApiGatewayApplication.class, args);
    }
} 