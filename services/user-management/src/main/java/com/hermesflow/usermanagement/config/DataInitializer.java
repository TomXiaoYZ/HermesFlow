package com.hermesflow.usermanagement.config;

import com.hermesflow.usermanagement.entity.Tenant;
import com.hermesflow.usermanagement.entity.User;
import com.hermesflow.usermanagement.repository.TenantRepository;
import com.hermesflow.usermanagement.repository.UserRepository;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.boot.CommandLineRunner;
import org.springframework.security.crypto.password.PasswordEncoder;
import org.springframework.stereotype.Component;

import java.time.LocalDateTime;
import java.util.UUID;

/**
 * 数据初始化器
 * 
 * 在应用启动时创建默认租户和管理员用户
 */
@Component
public class DataInitializer implements CommandLineRunner {

    private static final Logger logger = LoggerFactory.getLogger(DataInitializer.class);

    @Autowired
    private TenantRepository tenantRepository;

    @Autowired
    private UserRepository userRepository;

    @Autowired
    private PasswordEncoder passwordEncoder;

    @Override
    public void run(String... args) throws Exception {
        logger.info("=== DataInitializer 开始执行 ===");
        logger.info("开始初始化默认数据...");

        try {
            // 创建默认租户
            logger.info("正在创建默认租户...");
            Tenant defaultTenant = createDefaultTenant();
            logger.info("默认租户创建完成: {}", defaultTenant != null ? defaultTenant.getCode() : "null");
            
            // 创建默认管理员用户
            logger.info("正在创建默认管理员用户...");
            createDefaultAdmin(defaultTenant);
            logger.info("默认管理员用户创建完成");

            logger.info("默认数据初始化完成");
        } catch (Exception e) {
            logger.error("数据初始化过程中发生错误: ", e);
            throw e;
        }
        
        logger.info("=== DataInitializer 执行完成 ===");
    }

    private Tenant createDefaultTenant() {
        // 检查是否已存在默认租户
        if (tenantRepository.findByCode("default").isPresent()) {
            logger.info("默认租户已存在，跳过创建");
            return tenantRepository.findByCode("default").get();
        }

        Tenant tenant = new Tenant();
        tenant.setName("默认租户");
        tenant.setCode("default");
        tenant.setDescription("系统默认租户，用于测试和演示");
        tenant.setStatus(Tenant.TenantStatus.ACTIVE);
        tenant.setPlanType(Tenant.PlanType.BASIC);
        tenant.setMaxUsers(100);

        tenant = tenantRepository.save(tenant);
        logger.info("创建默认租户: {}", tenant.getCode());
        
        return tenant;
    }

    private void createDefaultAdmin(Tenant tenant) {
        // 检查是否已存在管理员用户
        if (userRepository.findByUsernameAndTenant("admin", tenant).isPresent()) {
            logger.info("默认管理员用户已存在，跳过创建");
            return;
        }

        User admin = new User();
        admin.setTenant(tenant);
        admin.setUsername("admin");
        admin.setEmail("admin@hermesflow.com");
        admin.setPasswordHash(passwordEncoder.encode("admin123"));
        admin.setFirstName("System");
        admin.setLastName("Administrator");
        admin.setStatus(User.UserStatus.ACTIVE);
        admin.setEmailVerified(true);
        admin.setPhoneVerified(false);
        admin.setFailedLoginAttempts(0);

        admin = userRepository.save(admin);
        logger.info("创建默认管理员用户: {}", admin.getUsername());
    }
} 