package com.hermesflow.usermanagement;

import org.slf4j.Logger;
import org.slf4j.LoggerFactory;
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.boot.SpringApplication;
import org.springframework.boot.autoconfigure.SpringBootApplication;
import org.springframework.boot.context.event.ApplicationReadyEvent;
import org.springframework.context.ApplicationContext;
import org.springframework.context.event.EventListener;
import org.springframework.data.jpa.repository.config.EnableJpaAuditing;
import org.springframework.data.redis.repository.configuration.EnableRedisRepositories;
import org.springframework.scheduling.annotation.EnableAsync;
import org.springframework.transaction.annotation.EnableTransactionManagement;

/**
 * HermesFlow User Management Service
 * 
 * 多租户用户管理服务主启动类
 * 提供用户认证、授权、租户管理等核心功能
 * 
 * @author HermesFlow Team
 * @version 1.0.0
 */
@SpringBootApplication
@EnableJpaAuditing
@EnableRedisRepositories
@EnableAsync
@EnableTransactionManagement
public class UserManagementApplication {

    private static final Logger logger = LoggerFactory.getLogger(UserManagementApplication.class);

    @Autowired
    private ApplicationContext applicationContext;

    public static void main(String[] args) {
        logger.info("=== 用户管理服务启动开始 ===");
        SpringApplication.run(UserManagementApplication.class, args);
        logger.info("=== 用户管理服务启动完成 ===");
    }

    @EventListener(ApplicationReadyEvent.class)
    public void checkBeans() {
        logger.info("=== 检查 Spring Bean 加载情况 ===");
        String[] beanNames = applicationContext.getBeanDefinitionNames();
        for (String beanName : beanNames) {
            if (beanName.contains("DataInitializer") || beanName.contains("dataInitializer")) {
                logger.info("找到 DataInitializer Bean: {}", beanName);
            }
        }
        logger.info("=== Bean 检查完成 ===");
    }
} 