package com.hermesflow.usermanagement;

import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.extension.ExtendWith;
import org.springframework.boot.test.context.SpringBootTest;
import org.springframework.test.context.ActiveProfiles;
import org.springframework.test.context.junit.jupiter.SpringExtension;
import org.springframework.transaction.annotation.Transactional;

import java.time.LocalDateTime;
import java.time.format.DateTimeFormatter;

/**
 * 测试基类
 * 
 * 提供通用的测试配置、注解和工具方法
 * 所有测试类都应该继承此基类
 */
@ExtendWith(SpringExtension.class)
@SpringBootTest(classes = UserManagementApplication.class)
@ActiveProfiles("test")
@Transactional
public abstract class BaseTest {

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
     * 清理测试数据
     * 子类可以重写此方法
     */
    protected void cleanupTestData() {
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

    /**
     * 断言字符串不为空且不为null
     */
    protected void assertNotBlank(String value, String message) {
        if (value == null || value.trim().isEmpty()) {
            throw new AssertionError(message + ": value is null or blank");
        }
    }

    /**
     * 断言对象不为null
     */
    protected void assertNotNull(Object value, String message) {
        if (value == null) {
            throw new AssertionError(message + ": value is null");
        }
    }

    /**
     * 断言布尔值为true
     */
    protected void assertTrue(boolean condition, String message) {
        if (!condition) {
            throw new AssertionError(message + ": condition is false");
        }
    }

    /**
     * 断言布尔值为false
     */
    protected void assertFalse(boolean condition, String message) {
        if (condition) {
            throw new AssertionError(message + ": condition is true");
        }
    }

    /**
     * 断言两个对象相等
     */
    protected void assertEquals(Object expected, Object actual, String message) {
        if (expected == null && actual == null) {
            return;
        }
        if (expected == null || actual == null || !expected.equals(actual)) {
            throw new AssertionError(
                String.format("%s: expected <%s> but was <%s>", message, expected, actual)
            );
        }
    }
} 