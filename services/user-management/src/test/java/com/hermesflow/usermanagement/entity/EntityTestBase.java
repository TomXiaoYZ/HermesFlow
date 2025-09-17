package com.hermesflow.usermanagement.entity;

import org.junit.jupiter.api.BeforeEach;

import java.time.LocalDateTime;
import java.time.format.DateTimeFormatter;
import java.util.UUID;

/**
 * 实体测试基类
 * 
 * 提供通用的测试工具方法，不依赖 Spring 上下文
 * 适用于简单的实体单元测试
 */
public abstract class EntityTestBase {

    protected static final DateTimeFormatter DATE_TIME_FORMATTER = 
        DateTimeFormatter.ofPattern("yyyy-MM-dd HH:mm:ss");

    /**
     * 每个测试方法执行前的初始化
     */
    @BeforeEach
    void setUp() {
        // 子类可以重写此方法进行特定的初始化
        initializeTestData();
    }

    /**
     * 初始化测试数据
     * 子类可以重写此方法
     */
    protected void initializeTestData() {
        // 默认实现为空，子类可以重写
    }

    /**
     * 获取当前时间
     */
    protected LocalDateTime now() {
        return LocalDateTime.now();
    }

    /**
     * 获取过去的时间
     */
    protected LocalDateTime pastTime(long minutes) {
        return LocalDateTime.now().minusMinutes(minutes);
    }

    /**
     * 获取未来的时间
     */
    protected LocalDateTime futureTime(long minutes) {
        return LocalDateTime.now().plusMinutes(minutes);
    }

    /**
     * 格式化时间为字符串
     */
    protected String formatDateTime(LocalDateTime dateTime) {
        return dateTime.format(DATE_TIME_FORMATTER);
    }

    /**
     * 生成测试用的随机字符串
     */
    protected String randomString(String prefix) {
        return prefix + "_" + System.currentTimeMillis();
    }

    /**
     * 生成测试用的随机邮箱
     */
    protected String randomEmail() {
        return "test_" + System.currentTimeMillis() + "@hermesflow.com";
    }

    /**
     * 生成测试用的随机用户名
     */
    protected String randomUsername() {
        return "user_" + System.currentTimeMillis();
    }

    /**
     * 生成测试用的随机租户代码
     */
    protected String randomTenantCode() {
        return "tenant_" + System.currentTimeMillis();
    }

    /**
     * 生成随机 UUID
     */
    protected UUID randomUUID() {
        return UUID.randomUUID();
    }

    /**
     * 断言两个时间相差在指定秒数内
     */
    protected void assertTimeWithinSeconds(LocalDateTime expected, LocalDateTime actual, long seconds) {
        long diff = Math.abs(java.time.Duration.between(expected, actual).getSeconds());
        if (diff > seconds) {
            throw new AssertionError(
                String.format("Time difference %d seconds exceeds threshold %d seconds. Expected: %s, Actual: %s", 
                    diff, seconds, formatDateTime(expected), formatDateTime(actual))
            );
        }
    }
} 